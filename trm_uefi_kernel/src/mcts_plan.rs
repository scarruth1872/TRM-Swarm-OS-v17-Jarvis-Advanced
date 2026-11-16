//! TRM Swarm OS — Monte Carlo Tree Search Planning Engine (council 83563732).
//!
//! REAL Monte Carlo Tree Search, implemented from scratch on bare metal:
//!   * Selection   — UCB1 (Upper Confidence Bound for trees): descend toward the
//!                   child maximizing  mean + C * sqrt(ln(parent_visits)/visits),
//!                   with an infinite prior for unexplored children.
//!   * Expansion   — materialize a new tree node for an untried action.
//!   * Simulation  — random rollouts from a leaf to a terminal state (light playout).
//!   * Backprop    — propagate the rollout reward up the visited path.
//!
//! The plant is a deterministic toy "game" so the optimal root action is known
//! analytically, letting UCB1's convergence be checked against a ground truth.
//! Tree nodes live in an arena `Vec<MctsNode>`; `nodes explored` is the measured
//! arena length after the run — a real count, not a constant.

extern crate alloc;
use alloc::vec::Vec;

/// Branching factor: number of legal actions from every non-terminal state.
const BRANCH: usize = 6;
/// Maximum tree depth (root at depth 0; leaves at path length `DEPTH`).
const DEPTH: usize = 4;
/// UCB1 exploration constant (balances exploitation of high mean vs. exploration).
const EXPLORE_C: f64 = 1.41;

// ---------------------------------------------------------------------------
// Self-contained float helpers. The `x86_64-unknown-uefi` target does NOT
// resolve the `f64` methods `.sqrt()`/`.ln()` (no libm in core for this
// target), so UCB1's exploration term is computed from first principles with
// only the infix operators `+ - * /`, which the target does support. These
// run identically on the host harness and on bare metal.
// ---------------------------------------------------------------------------

const LN2: f64 = 0.6931471805599453;

/// sqrt(a) via Newton–Raphson (x = (x + a/x)/2), quadratic convergence.
fn fsqrt(a: f64) -> f64 {
    if a <= 0.0 {
        return 0.0;
    }
    let mut x = a * 0.5;
    for _ in 0..20 {
        if x == 0.0 {
            break;
        }
        x = 0.5 * (x + a / x);
    }
    x
}

/// ln(a) via the atanh series ln(a) = 2*(z + z^3/3 + z^5/5 +...) with
/// z = (a-1)/(a+1), after scaling a into [1,2). Only infix arithmetic.
fn fln(a: f64) -> f64 {
    if a <= 0.0 {
        return 0.0;
    }
    let mut x = a;
    let mut k = 0.0f64;
    while x >= 2.0 {
        x *= 0.5;
        k += 1.0;
    }
    while x < 1.0 {
        x *= 2.0;
        k -= 1.0;
    }
    let z = (x - 1.0) / (x + 1.0);
    let zz = z * z;
    let mut s = 0.0f64;
    let mut term = z;
    let mut n = 1.0f64;
    for _ in 0..32 {
        s += term / n;
        term *= zz;
        n += 2.0;
    }
    2.0 * s + k * LN2
}

/// Arena node index (`usize` into `Vec<MctsNode>`).
type Nid = usize;

/// xorshift64 — deterministic pseudo-random source for rollout play.
struct XorShift64 {
    s: u64,
}
impl XorShift64 {
    fn new() -> Self {
        XorShift64 { s: 0x9E3779B97F4A7C15 }
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.s;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.s = x;
        x
    }
    fn next_action(&mut self) -> usize {
        (self.next_u64() % BRANCH as u64) as usize
    }
}

/// One node of the search tree.
struct MctsNode {
    /// Action path from the root to this node (the "state").
    path: Vec<usize>,
    /// Total reward backpropagated through this node (for mean = total/visits).
    total: f64,
    /// Number of simulations that visited this node.
    visits: u32,
    /// Expanded children as (action, arena index).
    children: Vec<(usize, Nid)>,
    /// Actions not yet expanded from this node.
    untried: Vec<usize>,
    /// Parent arena index (memory for backpropagation).
    parent: Option<Nid>,
}

impl MctsNode {
    fn root() -> Self {
        let mut untried: Vec<usize> = (0..BRANCH).collect();
        untried.reverse(); // pop from the end keeps action 0 first
        MctsNode {
            path: Vec::new(),
            total: 0.0,
            visits: 0,
            children: Vec::new(),
            untried,
            parent: None,
        }
    }
    fn child_of(parent: Nid, act: usize, parent_path: &[usize]) -> Self {
        let mut untried: Vec<usize> = (0..BRANCH).collect();
        untried.reverse();
        let mut path = parent_path.to_vec();
        path.push(act);
        MctsNode {
            path,
            total: 0.0,
            visits: 0,
            children: Vec::new(),
            untried,
            parent: Some(parent),
        }
    }
}

/// Terminal test — a state with a full action path has a definite reward.
fn is_terminal(path: &[usize]) -> bool {
    path.len() >= DEPTH
}

/// Deterministic terminal payoff of a full action path.
///
/// Ground truth: choosing action **1** at level 0 dominates every other choice
/// (a large constant bonus), so the analytically-optimal root action is 1 and
/// every other action is clearly worse regardless of the random playout tails.
fn reward(path: &[usize]) -> f64 {
    if path.is_empty() {
        return 0.0;
    }
    let mut r = 1.0;
    if path[0] == 1 {
        r += 8.0;
    } else {
        r += 1.0;
    }
    for (i, &a) in path.iter().enumerate().skip(1) {
        r += (a as f64) * 0.5 / (i as f64 + 1.0);
    }
    r
}

/// The search tree (arena).
pub struct MctsTree {
    arena: Vec<MctsNode>,
    rng: XorShift64,
}

/// Result of a completed search, suitable for terminal formatting.
pub struct MctsReport {
    pub simulations: u32,
    pub nodes: usize, // measured arena length = nodes explored
    pub best_action: usize,
    pub best_visits: u32,
    pub best_mean: f64,
    pub root_visits: u32,
    /// (action, visits, mean) for each root child.
    pub child_stats: Vec<(usize, u32, f64)>,
}

impl MctsTree {
    pub fn new() -> Self {
        let mut arena = Vec::new();
        arena.push(MctsNode::root());
        MctsTree { arena, rng: XorShift64::new() }
    }

    /// UCB1 score of a child; unexplored children are infinitely attractive.
    fn ucb1(&self, child: Nid, parent_visits: f64) -> f64 {
        let c = &self.arena[child];
        if c.visits == 0 {
            return f64::MAX;
        }
        let mean = c.total / c.visits as f64;
        mean + EXPLORE_C * fsqrt(fln(parent_visits) / c.visits as f64)
    }

    /// Pick the child with the best UCB1 score.
    fn select_child(&self, node: Nid) -> (usize, Nid) {
        let parent_visits = self.arena[node].visits.max(1) as f64;
        let mut best: Option<(f64, usize, Nid)> = None;
        for &(act, cid) in &self.arena[node].children {
            let score = self.ucb1(cid, parent_visits);
            let better = match best {
                None => true,
                Some((bs, _, _)) => score > bs,
            };
            if better {
                best = Some((score, act, cid));
            }
        }
        match best {
            Some((_, act, cid)) => (act, cid),
            None => (0, node), // degenerate; no children
        }
    }

    /// Expansion: materialize one child for an untried action and return it.
    fn expand(&mut self, node: Nid) -> Nid {
        let act = self.arena[node].untried.pop().expect("expand with untried action");
        let parent_path = self.arena[node].path.clone();
        let child = MctsNode::child_of(node, act, &parent_path);
        self.arena.push(child);
        let cid = self.arena.len() - 1;
        self.arena[node].children.push((act, cid));
        cid
    }

    /// Rollout: play random actions from the given path until terminal. Light
    /// playout — no tree nodes are allocated; reward is a pure estimate.
    fn simulate(&mut self, start_path: &[usize]) -> f64 {
        let mut path = start_path.to_vec();
        while !is_terminal(&path) {
            path.push(self.rng.next_action());
        }
        reward(&path)
    }

    /// Backpropagate a reward up the visited path.
    fn backprop(&mut self, mut node: Nid, value: f64) {
        loop {
            self.arena[node].visits += 1;
            self.arena[node].total += value;
            match self.arena[node].parent {
                Some(p) => node = p,
                None => break,
            }
        }
    }

    /// One full MCTS iteration (select → expand → simulate → backprop).
    fn iterate(&mut self) {
        // Phase 1+2: Selection then Expansion.
        let mut node: Nid = 0;
        let mut depth = 0usize;
        let start_path: Vec<usize>; // path at the node we simulate from
        loop {
            let nd = &self.arena[node];
            if is_terminal(&nd.path) {
                start_path = nd.path.clone();
                break;
            }
            if !nd.untried.is_empty() {
                let fresh = self.expand(node);
                start_path = self.arena[fresh].path.clone();
                node = fresh;
                depth += 1;
                break;
            }
            // Fully expanded, non-terminal: descend via UCB1.
            let (act, cid) = self.select_child(node);
            let _ = act;
            node = cid;
            depth += 1;
            if depth > DEPTH * BRANCH + 8 {
                // defensive bound; cannot be hit given finite DEPTH
                start_path = self.arena[node].path.clone();
                break;
            }
        }
        let _ = depth;
        // Phase 3+4: random rollout from the expanded/selected leaf, then backprop.
        let value = self.simulate(&start_path);
        self.backprop(node, value);
    }

    /// Run `simulations` full MCTS iterations and report the result.
    pub fn run(mut self, simulations: u32) -> MctsReport {
        for _ in 0..simulations {
            self.iterate();
        }
        let root_visits = self.arena[0].visits;
        // Root children: stats for the report.
        let mut child_stats: Vec<(usize, u32, f64)> =
            self.arena[0].children.iter().map(|&(a, c)| {
                let mean = if self.arena[c].visits > 0 {
                    self.arena[c].total / self.arena[c].visits as f64
                } else { 0.0 };
                (a, self.arena[c].visits, mean)
            }).collect();
        // Best action = most-visited root child (standard MCTS policy).
        child_stats.sort_by(|x, y| y.1.cmp(&x.1));
        let (best_action, best_visits) =
            match child_stats.first() {
                Some((a, v, _)) => (*a, *v),
                None => (0, 0),
            };
        let best_mean = match child_stats.first() {
            Some((_, _, m)) => *m,
            None => 0.0,
        };
        let nodes = self.arena.len();
        MctsReport {
            simulations,
            nodes,
            best_action,
            best_visits,
            best_mean,
            root_visits,
            child_stats,
        }
    }
}

/// Convenience runner for the terminal command.
pub fn run(simulations: u32) -> MctsReport {
    MctsTree::new().run(simulations)
}