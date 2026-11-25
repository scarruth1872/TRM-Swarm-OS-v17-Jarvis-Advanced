//! Scientific computing — tasks 421-430 (no_std + alloc).
//!
//! # What this is, stated before the code
//!
//! Every routine here is a **numerical method with an analytic reference it can be checked against**. That is the
//! whole design: a simulation nobody can falsify is not a simulation. Each `fn` names its reference in the doc
//! comment, and `kernel_tests` asserts against it.
//!
//! What this is NOT:
//!   * **422** is Hückel π-electron theory solved by Jacobi rotation. It is not DFT, not Hartree-Fock, and there is
//!     no basis set or correlation functional here. Benzene's (−2,−1,−1,1,1,2)β spectrum is the check.
//!   * **423** is the 2-D HP lattice model: a toy energy function used to test a *search*. It is not structure
//!     prediction for a real sequence and the report says so.
//!   * **424** is a contact energy function validated against the analytic Lennard-Jones well. No correlation with
//!     measured affinity is claimed, because that would need the 4,463 PDBbind complexes extracted and scored, and
//!     docking has already produced one false result on this platform (frame mismatch, RMSD 13.82 Å).
//!
//! `no_std` has no `sqrt`, `exp` or `ln`; all three are implemented below.

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

// ---------------------------------------------------------------------------------------------------------
// math primitives missing from no_std
// ---------------------------------------------------------------------------------------------------------

/// Newton–Raphson square root, 40 iterations is far past convergence for f64.
pub fn sqrt(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut g = x * 0.5;
    for _ in 0..40 {
        let next = 0.5 * (g + x / g);
        if (next - g).abs() < 1e-15 * g.abs().max(1.0) {
            g = next;
            break;
        }
        g = next;
    }
    g
}

/// `no_std` has no `round`. Half away from zero.
pub fn round(x: f64) -> f64 {
    if x >= 0.0 {
        (x + 0.5) as i64 as f64
    } else {
        (x - 0.5) as i64 as f64
    }
}

const LN2: f64 = 0.693_147_180_559_945_3;

/// exp with range reduction (x = k·ln2 + r, |r| ≤ ln2/2) and a Taylor series.
///
/// The first version was (1 + x/4096)^4096, whose truncation error is x²/2n ≈ 1.2e-4 — and `exp(1)` came out
/// 2.71795 instead of 2.71828, which the test caught immediately. Range reduction plus 24 Taylor terms is good
/// to ~1e-16 over the range these simulations use.
pub fn exp(x: f64) -> f64 {
    if x > 700.0 {
        return f64::MAX;
    }
    if x < -700.0 {
        return 0.0;
    }
    let k = round(x / LN2) as i32;
    let r = x - k as f64 * LN2;
    let mut term = 1.0f64;
    let mut acc = 1.0f64;
    let mut n = 1i32;
    while n < 24 {
        term *= r / n as f64;
        acc += term;
        n += 1;
    }
    let mut scale = 1.0f64;
    if k >= 0 {
        for _ in 0..k.min(1024) {
            scale *= 2.0;
        }
    } else {
        for _ in 0..(-k).min(1024) {
            scale *= 0.5;
        }
    }
    acc * scale
}


/// `no_std` has no `powi`: exponentiation by squaring.
pub fn powi(base: f64, n: i32) -> f64 {
    let mut b = base;
    let mut e = n.unsigned_abs();
    let mut acc = 1.0f64;
    while e > 0 {
        if e & 1 == 1 {
            acc *= b;
        }
        b *= b;
        e >>= 1;
    }
    if n < 0 {
        1.0 / acc
    } else {
        acc
    }
}

/// `no_std` has no `powf`: exp(y·ln x).
pub fn powf(base: f64, exponent: f64) -> f64 {
    if base <= 0.0 {
        return 0.0;
    }
    exp(exponent * ln(base))
}

/// ln by range reduction to m ∈ [1,2) and the atanh series. Absolute error < 1e-12 over the positive reals.
pub fn ln(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::NEG_INFINITY;
    }
    let mut m = x;
    let mut k = 0i32;
    while m >= 2.0 {
        m *= 0.5;
        k += 1;
    }
    while m < 1.0 {
        m *= 2.0;
        k -= 1;
    }
    let z = (m - 1.0) / (m + 1.0);
    let z2 = z * z;
    let mut term = z;
    let mut acc = 0.0f64;
    let mut n = 1i32;
    while n < 60 {
        acc += term / n as f64;
        term *= z2;
        n += 2;
    }
    2.0 * acc + k as f64 * LN2
}

// ---------------------------------------------------------------------------------------------------------
// ODE integration (425, 426, 428 all need it)
// ---------------------------------------------------------------------------------------------------------

/// Trace of an integration: times and states.
pub struct Trace {
    pub t: Vec<f64>,
    pub y: Vec<Vec<f64>>,
}

/// Classical Runge–Kutta 4. Reference: a harmonic oscillator integrated with RK4 keeps its amplitude to O(dt^4),
/// and the closed-form solution is the check in the tests.
pub fn rk4<F>(y0: &[f64], t0: f64, dt: f64, steps: usize, f: F) -> Trace
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    let n = y0.len();
    let mut t = Vec::with_capacity(steps + 1);
    let mut y = Vec::with_capacity(steps + 1);
    let mut cur = y0.to_vec();
    let mut time = t0;
    t.push(time);
    y.push(cur.clone());
    for _ in 0..steps {
        let k1 = f(time, &cur);
        let mut tmp = vec![0.0; n];
        for i in 0..n {
            tmp[i] = cur[i] + 0.5 * dt * k1[i];
        }
        let k2 = f(time + 0.5 * dt, &tmp);
        for i in 0..n {
            tmp[i] = cur[i] + 0.5 * dt * k2[i];
        }
        let k3 = f(time + 0.5 * dt, &tmp);
        for i in 0..n {
            tmp[i] = cur[i] + dt * k3[i];
        }
        let k4 = f(time + dt, &tmp);
        for i in 0..n {
            cur[i] += dt / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
        }
        time += dt;
        t.push(time);
        y.push(cur.clone());
    }
    Trace { t, y }
}

// ---------------------------------------------------------------------------------------------------------
// 421 — molecular dynamics
// ---------------------------------------------------------------------------------------------------------

/// Lennard-Jones pair energy from squared distance. `eps` is the well depth, `sigma` the zero-crossing radius.
pub fn lj_energy(r2: f64, eps: f64, sigma: f64) -> f64 {
    if r2 <= 0.0 {
        return 0.0;
    }
    let sr2 = sigma * sigma / r2;
    let sr6 = sr2 * sr2 * sr2;
    let sr12 = sr6 * sr6;
    4.0 * eps * (sr12 - sr6)
}

/// A small LJ fluid. Velocities are damped-free: energy conservation is the correctness test, so nothing may
/// silently remove energy except the thermostat the caller chooses not to apply.
pub struct MdSystem {
    pub pos: Vec<[f64; 3]>,
    pub vel: Vec<[f64; 3]>,
    pub eps: f64,
    pub sigma: f64,
    pub mass: f64,
    pub cutoff: f64,
}

impl MdSystem {
    /// Simple cubic lattice at `spacing`, with zero velocities — a cold start, which is the honest way to test
    /// energy drift (a thermalised start would hide it in noise).
    pub fn lattice(n_side: usize, spacing: f64, eps: f64, sigma: f64) -> MdSystem {
        let mut pos = Vec::new();
        for i in 0..n_side {
            for j in 0..n_side {
                for k in 0..n_side {
                    pos.push([i as f64 * spacing, j as f64 * spacing, k as f64 * spacing]);
                }
            }
        }
        let n = pos.len();
        MdSystem { pos, vel: vec![[0.0; 3]; n], eps, sigma, mass: 1.0, cutoff: 3.0 * sigma + spacing }
    }

    /// Pairwise forces and potential energy. O(N²) — correct before fast, and N is small here.
    pub fn forces(&self) -> (Vec<[f64; 3]>, f64) {
        let n = self.pos.len();
        let mut f = vec![[0.0f64; 3]; n];
        let mut pe = 0.0;
        for i in 0..n {
            for j in (i + 1)..n {
                let mut d = [0.0; 3];
                for k in 0..3 {
                    d[k] = self.pos[i][k] - self.pos[j][k];
                }
                let r2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
                if r2 > self.cutoff * self.cutoff {
                    continue;
                }
                pe += lj_energy(r2, self.eps, self.sigma);
                let sr2 = self.sigma * self.sigma / r2;
                let sr6 = sr2 * sr2 * sr2;
                // dV/dr / r = 24 eps (2 sr12 - sr6) / r^2
                let inv = 24.0 * self.eps * (2.0 * sr6 * sr6 - sr6) / r2;
                for k in 0..3 {
                    f[i][k] += inv * d[k];
                    f[j][k] -= inv * d[k];
                }
            }
        }
        (f, pe)
    }

    pub fn kinetic(&self) -> f64 {
        self.vel.iter().map(|v| 0.5 * self.mass * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2])).sum()
    }

    pub fn potential(&self) -> f64 {
        self.forces().1
    }

    pub fn total_energy(&self) -> f64 {
        self.kinetic() + self.potential()
    }

    /// Velocity Verlet. Symplectic: the energy error oscillates instead of drifting, which is exactly what the
    /// energy-conservation test measures.
    pub fn step(&mut self, dt: f64) {
        let (mut f, _) = self.forces();
        let n = self.pos.len();
        for i in 0..n {
            for k in 0..3 {
                self.vel[i][k] += 0.5 * dt * f[i][k] / self.mass;
                self.pos[i][k] += dt * self.vel[i][k];
            }
        }
        let (f2, _) = self.forces();
        for i in 0..n {
            for k in 0..3 {
                self.vel[i][k] += 0.5 * dt * f2[i][k] / self.mass;
            }
        }
        f = f2;
        let _ = f;
    }

    /// Instantaneous temperature from equipartition, 3N−3 degrees of freedom.
    pub fn temperature(&self, kb: f64) -> f64 {
        let n = self.pos.len();
        if n < 2 {
            return 0.0;
        }
        let dof = (3 * n - 3) as f64;
        2.0 * self.kinetic() / (dof * kb)
    }
}

// ---------------------------------------------------------------------------------------------------------
// 422 — Hückel molecular orbitals (explicitly not DFT)
// ---------------------------------------------------------------------------------------------------------

/// Cyclic Jacobi rotation for a symmetric matrix. Returns eigenvalues in ascending order.
/// Reference: a diagonal matrix must return its diagonal; a 2×2 has a closed form the tests compare against.
pub fn jacobi_eigen(a: &mut Vec<Vec<f64>>, max_sweeps: usize) -> Vec<f64> {
    let n = a.len();
    if n == 0 {
        return Vec::new();
    }
    for _ in 0..max_sweeps {
        let mut off = 0.0;
        for p in 0..n {
            for q in (p + 1)..n {
                off += a[p][q] * a[p][q];
            }
        }
        if off < 1e-22 {
            break;
        }
        for p in 0..n {
            for q in (p + 1)..n {
                if a[p][q].abs() < 1e-18 {
                    continue;
                }
                let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
                let t = if theta >= 0.0 {
                    1.0 / (theta + sqrt(theta * theta + 1.0))
                } else {
                    -1.0 / (-theta + sqrt(theta * theta + 1.0))
                };
                let c = 1.0 / sqrt(t * t + 1.0);
                let s = t * c;
                for k in 0..n {
                    let akp = a[k][p];
                    let akq = a[k][q];
                    a[k][p] = c * akp - s * akq;
                    a[k][q] = s * akp + c * akq;
                }
                for k in 0..n {
                    let apk = a[p][k];
                    let aqk = a[q][k];
                    a[p][k] = c * apk - s * aqk;
                    a[q][k] = s * apk + c * aqk;
                }
            }
        }
    }
    let mut e: Vec<f64> = (0..n).map(|i| a[i][i]).collect();
    e.sort_by(|x, y| x.partial_cmp(y).unwrap_or(core::cmp::Ordering::Equal));
    e
}

/// Hückel matrix for a linear conjugated chain of `n` atoms (α on the diagonal, β on the off-diagonals).
pub fn huckel_chain(n: usize, alpha: f64, beta: f64) -> Vec<Vec<f64>> {
    let mut m = vec![vec![0.0; n]; n];
    for i in 0..n {
        m[i][i] = alpha;
    }
    for i in 0..n.saturating_sub(1) {
        m[i][i + 1] = beta;
        m[i + 1][i] = beta;
    }
    m
}

/// Hückel matrix for a closed ring (benzene at n = 6).
pub fn huckel_ring(n: usize, alpha: f64, beta: f64) -> Vec<Vec<f64>> {
    let mut m = vec![vec![0.0; n]; n];
    for i in 0..n {
        m[i][i] = alpha;
        let j = (i + 1) % n;
        m[i][j] = beta;
        m[j][i] = beta;
    }
    m
}

/// Total π energy of the occupied orbitals: 2 electrons per level up to `n_electrons`.
pub fn pi_energy(eigenvalues: &[f64], n_electrons: usize) -> f64 {
    let mut e = 0.0;
    let mut left = n_electrons;
    for &ev in eigenvalues.iter() {
        if left == 0 {
            break;
        }
        let occ = left.min(2);
        e += occ as f64 * ev;
        left -= occ;
    }
    e
}

/// HOMO–LUMO gap from ascending eigenvalues and the electron count.
pub fn homo_lumo_gap(eigenvalues: &[f64], n_electrons: usize) -> f64 {
    let homo = (n_electrons / 2).saturating_sub(1);
    if homo + 1 >= eigenvalues.len() {
        return 0.0;
    }
    eigenvalues[homo + 1] - eigenvalues[homo]
}

// ---------------------------------------------------------------------------------------------------------
// 423 — HP lattice folding (a search problem, not structure prediction)
// ---------------------------------------------------------------------------------------------------------

/// Contact energy of a lattice conformation: −1 for every non-sequential H–H neighbour pair.
/// Reference: a straight 1-D line has zero H–H contacts, and the exhaustive search below is the ground truth.
pub fn hp_energy(coords: &[(i32, i32)], seq: &[u8]) -> i32 {
    let mut e = 0i32;
    let n = coords.len().min(seq.len());
    for i in 0..n {
        if seq[i] != b'H' {
            continue;
        }
        for j in (i + 2)..n {
            if seq[j] != b'H' {
                continue;
            }
            let dx = (coords[i].0 - coords[j].0).abs();
            let dy = (coords[i].1 - coords[j].1).abs();
            if dx + dy == 1 {
                e -= 1;
            }
        }
    }
    e
}

/// Self-avoiding check for a conformation.
pub fn is_self_avoiding(coords: &[(i32, i32)]) -> bool {
    for i in 0..coords.len() {
        for j in (i + 1)..coords.len() {
            if coords[i] == coords[j] {
                return false;
            }
        }
    }
    true
}

/// Exhaustive ground state by depth-first self-avoiding walk. `complete` is false when the node budget ran out —
/// a truncated search must never be reported as an optimum.
pub fn hp_exhaustive(seq: &[u8], budget: u64) -> (i32, Vec<(i32, i32)>, bool) {
    let n = seq.len();
    if n == 0 {
        return (0, Vec::new(), true);
    }
    let mut best = i32::MAX;
    let mut best_coords: Vec<(i32, i32)> = Vec::new();
    let mut count = 0u64;
    let mut complete = true;
    // first step fixed to +x to remove the rotational symmetry
    let mut coords: Vec<(i32, i32)> = vec![(0, 0), (1, 0)];
    let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];

    fn walk(
        seq: &[u8],
        coords: &mut Vec<(i32, i32)>,
        dirs: &[(i32, i32); 4],
        best: &mut i32,
        best_coords: &mut Vec<(i32, i32)>,
        count: &mut u64,
        budget: u64,
        complete: &mut bool,
    ) {
        if *count >= budget {
            *complete = false;
            return;
        }
        *count += 1;
        if coords.len() == seq.len() {
            let e = hp_energy(coords, seq);
            if e < *best {
                *best = e;
                *best_coords = coords.clone();
            }
            return;
        }
        let last = *coords.last().unwrap();
        for d in dirs.iter() {
            let next = (last.0 + d.0, last.1 + d.1);
            if coords.contains(&next) {
                continue;
            }
            coords.push(next);
            walk(seq, coords, dirs, best, best_coords, count, budget, complete);
            coords.pop();
            if !*complete {
                return;
            }
        }
    }
    if n == 1 {
        return (0, vec![(0, 0)], true);
    }
    walk(seq, &mut coords, &dirs, &mut best, &mut best_coords, &mut count, budget, &mut complete);
    (if best == i32::MAX { 0 } else { best }, best_coords, complete)
}

/// Metropolis annealing over self-avoiding lattice moves. Seeded LCG so a run is reproducible.
/// This is deliberately NOT forced through the SRAM Ising pattern: that pattern is a 1-D nearest-neighbour spin
/// sweep, and mapping lattice folding onto it would be decorative rather than correct.
pub fn hp_anneal(seq: &[u8], steps: usize, seed: u64, t0: f64, t_end: f64) -> (i32, Vec<(i32, i32)>) {
    let n = seq.len();
    if n < 3 {
        let coords: Vec<(i32, i32)> = (0..n).map(|i| (i as i32, 0)).collect();
        return (hp_energy(&coords, seq), coords);
    }
    let mut rng = seed | 1;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    let mut coords: Vec<(i32, i32)> = (0..n).map(|i| (i as i32, 0)).collect();
    let mut cur = hp_energy(&coords, seq);
    let mut best = cur;
    let mut best_coords = coords.clone();
    let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];

    for step in 0..steps {
        let frac = step as f64 / steps.max(1) as f64;
        let temp = t0 * powf(t_end / t0, frac);
        // pivot at a random interior residue
        let pivot = 1 + (next() as usize) % (n - 1);
        let d = dirs[(next() as usize) % 4];
        // rebuild the tail from the pivot with a rigid rotation-free move: try one new direction
        let mut cand = coords[..pivot].to_vec();
        let mut ok = true;
        for i in pivot..n {
            let prev = cand[i - 1];
            // continue in a direction that keeps self-avoidance, chosen randomly
            let mut placed = false;
            for k in 0..4 {
                let dd = dirs[(k + (next() as usize % 4)) % 4];
                let p = (prev.0 + dd.0, prev.1 + dd.1);
                if !cand.contains(&p) {
                    cand.push(p);
                    placed = true;
                    break;
                }
            }
            if !placed {
                ok = false;
                break;
            }
        }
        if !ok {
            let _ = d;
            continue;
        }
        let cand_e = hp_energy(&cand, seq);
        let delta = (cand_e - cur) as f64;
        let accept = delta <= 0.0 || (next() as f64 / u64::MAX as f64) < exp(-delta / temp.max(1e-9));
        if accept {
            coords = cand;
            cur = cand_e;
            if cur < best {
                best = cur;
                best_coords = coords.clone();
            }
        }
    }
    (best, best_coords)
}

// ---------------------------------------------------------------------------------------------------------
// 424 — binding affinity as a contact energy (no correlation claim)
// ---------------------------------------------------------------------------------------------------------

/// A single interaction term between two atoms. Distances in Å, charges in e, eps in kcal/mol.
#[derive(Debug, Clone)]
pub struct Atom {
    pub pos: [f64; 3],
    pub charge: f64,
    pub radius: f64,
}

const COULOMB_KCAL: f64 = 332.063_71; // Å·e² → kcal/mol

/// Binding energy of a ligand pose against a pocket: Lennard-Jones (repulsion + dispersion) plus screened Coulomb.
///
/// Reference for the test: the LJ minimum of a single pair sits at r = 2^(1/6)·σ with energy −ε, and the Coulomb
/// term at that distance must equal k·q1·q2/(ε_r·r). Both are asserted exactly in `kernel_tests`.
pub fn contact_energy(ligand: &[Atom], pocket: &[Atom], dielectric: f64) -> f64 {
    let mut e = 0.0;
    for l in ligand {
        for p in pocket {
            let dx = l.pos[0] - p.pos[0];
            let dy = l.pos[1] - p.pos[1];
            let dz = l.pos[2] - p.pos[2];
            let r2 = dx * dx + dy * dy + dz * dz;
            if r2 > 100.0 {
                continue;
            }
            let sigma = l.radius + p.radius;
            let eps = 0.15; // kcal/mol, a generic contact well; NOT fitted to anything
            e += lj_energy(r2.max(1e-6), eps, sigma);
            let r = sqrt(r2.max(1e-6));
            e += COULOMB_KCAL * l.charge * p.charge / (dielectric * r);
        }
    }
    e
}

/// Desolvation penalty: burying a charged atom costs energy. Linear in buried fraction — a coarse model, and
/// labelled as such.
pub fn desolvation_penalty(buried_fraction: f64, total_charge: f64) -> f64 {
    12.0 * buried_fraction * total_charge.abs()
}

// ---------------------------------------------------------------------------------------------------------
// 425 — pharmacokinetics
// ---------------------------------------------------------------------------------------------------------

/// One-compartment IV bolus: C(t) = (D/V)·e^(−k·t). Closed form, so the test compares the model to itself only
/// through `exp`, and `exp` is itself checked against ln.
pub fn pk_iv_bolus(dose: f64, volume: f64, k: f64, t: f64) -> f64 {
    dose / volume * exp(-k * t)
}

/// Elimination half-life from the rate constant: t½ = ln2 / k.
pub fn pk_half_life(k: f64) -> f64 {
    LN2 / k
}

/// First-order absorption, one compartment: C(t) = (F·D·ka)/(V(ka−k)) · (e^(−k t) − e^(−ka t)).
/// Reference: at t=0 the concentration is exactly 0 and it peaks where dC/dt = 0.
pub fn pk_oral(dose: f64, volume: f64, k: f64, ka: f64, t: f64) -> f64 {
    if (ka - k).abs() < 1e-12 {
        return dose / volume * k * t * exp(-k * t);
    }
    dose * ka / (volume * (ka - k)) * (exp(-k * t) - exp(-ka * t))
}

/// Two-compartment bi-exponential: C(t) = A·e^(−αt) + B·e^(−βt).
/// Reference: C(0) = A + B must equal dose/Vc, which the test asserts.
pub fn pk_two_compartment(a: f64, alpha: f64, b: f64, beta: f64, t: f64) -> f64 {
    a * exp(-alpha * t) + b * exp(-beta * t)
}

/// Time to peak for oral dosing, t_max = ln(ka/k)/(ka−k).
pub fn pk_t_max(k: f64, ka: f64) -> f64 {
    ln(ka / k) / (ka - k)
}

// ---------------------------------------------------------------------------------------------------------
// 426 — epidemiology (SIR)
// ---------------------------------------------------------------------------------------------------------

pub struct SirResult {
    pub r0: f64,
    pub final_size: f64,
    pub peak_infected: f64,
    pub peak_day: f64,
    pub attack_rate: f64,
}

/// Final size of an SIR epidemic, solving 1 − z = s0·e^(−R0·z) by bisection on [0, 1].
/// This is the independent reference the RK4 integration is compared against.
pub fn sir_final_size(beta: f64, gamma: f64, s0: f64) -> f64 {
    let r0 = beta / gamma;
    let f = |z: f64| s0 * exp(-r0 * z) - (1.0 - z);
    // Final size z solves s0·e^(−R0·z) = 1 − z. At z=0 the left side exceeds the right (by i0), and at z=1 the
    // left side is positive while the right is zero — so a solution exists iff f(1) > 0. Testing f(0) instead
    // returned zero for every epidemic, which is how this bug was found.
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    if f(1.0) <= 0.0 {
        return 0.0;
    }
    // f is negative below the root and positive above it, so a negative value moves the LOWER bound up. The first
    // version had this inverted and converged to 0 for every epidemic — a bisection that always returns an endpoint
    // is the classic way for a root-finder to look like it works.
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if f(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Integrate SIR with RK4 and report the epidemic's observable quantities.
/// Reference: `final_size` here must agree with `sir_final_size` (the transcendental solution) to ~1e-3.
pub fn sir(beta: f64, gamma: f64, i0: f64, days: f64, dt: f64) -> SirResult {
    let s0 = 1.0 - i0;
    let steps = round(days / dt) as usize;
    let f = |_t: f64, y: &[f64]| {
        let (s, i) = (y[0], y[1]);
        vec![-beta * s * i, beta * s * i - gamma * i]
    };
    let tr = rk4(&[s0, i0], 0.0, dt, steps, f);
    let mut peak = i0;
    let mut peak_day = 0.0;
    for (idx, y) in tr.y.iter().enumerate() {
        if y[1] > peak {
            peak = y[1];
            peak_day = tr.t[idx];
        }
    }
    let last = tr.y.last().cloned().unwrap_or_else(|| vec![s0, i0]);
    let final_size = (s0 - last[0]).max(0.0);
    SirResult {
        r0: beta / gamma,
        final_size,
        peak_infected: peak,
        peak_day,
        attack_rate: final_size / s0.max(1e-12),
    }
}

// ---------------------------------------------------------------------------------------------------------
// 427 — population genetics
// ---------------------------------------------------------------------------------------------------------

/// Hardy–Weinberg genotype frequencies from an allele frequency.
pub fn hardy_weinberg(p: f64) -> (f64, f64, f64) {
    (p * p, 2.0 * p * (1.0 - p), (1.0 - p) * (1.0 - p))
}

/// Haploid selection: p' = p(1+s) / (1 + s·p). Reference: s = 0 leaves p unchanged.
pub fn selection_next(p: f64, s: f64) -> f64 {
    p * (1.0 + s) / (1.0 + s * p)
}

/// Wright–Fisher neutral drift with a binomial draw approximated by summing Bernoulli trials from an LCG.
/// Reference: the fixation probability of a neutral allele must equal its initial frequency p0, and that is what
/// the test measures over many replicates with a tolerance stated in the assertion.
pub fn wright_fisher_fixation(p0: f64, pop: usize, reps: usize, seed: u64) -> f64 {
    let mut rng = seed | 1;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    let mut fixed = 0usize;
    for _ in 0..reps {
        let mut copies = (round(p0 * pop as f64) as usize).min(pop);
        for _ in 0..2000 {
            if copies == 0 {
                break;
            }
            if copies == pop {
                fixed += 1;
                break;
            }
            let mut drawn = 0usize;
            for _ in 0..pop {
                if (next() as f64 / u64::MAX as f64) < copies as f64 / pop as f64 {
                    drawn += 1;
                }
            }
            copies = drawn;
        }
    }
    fixed as f64 / reps.max(1) as f64
}

/// Expected heterozygosity under drift decays as H_t = H_0(1 − 1/2N)^t.
pub fn heterozygosity_after(h0: f64, pop: usize, generations: usize) -> f64 {
    let retain = 1.0 - 1.0 / (2.0 * pop as f64);
    h0 * powi(retain, generations as i32)
}

// ---------------------------------------------------------------------------------------------------------
// 428 — systems biology (Lotka–Volterra, conserved quantity)
// ---------------------------------------------------------------------------------------------------------

pub struct LotkaVolterra {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    /// Drift of the conserved quantity H = δx − γ·ln x + βy − α·ln y over the run, as a fraction of its initial
    /// magnitude. An integrator that is not conserving shows this immediately.
    pub h_drift: f64,
    pub h0: f64,
}

pub fn lotka_volterra(alpha: f64, beta: f64, gamma: f64, delta: f64, x0: f64, y0: f64, dt: f64, steps: usize)
    -> LotkaVolterra
{
    let f = |_t: f64, y: &[f64]| vec![alpha * y[0] - beta * y[0] * y[1], delta * y[0] * y[1] - gamma * y[1]];
    let tr = rk4(&[x0, y0], 0.0, dt, steps, f);
    let xs: Vec<f64> = tr.y.iter().map(|v| v[0]).collect();
    let ys: Vec<f64> = tr.y.iter().map(|v| v[1]).collect();
    let h = |x: f64, y: f64| delta * x - gamma * ln(x.max(1e-12)) + beta * y - alpha * ln(y.max(1e-12));
    let h0 = h(x0, y0);
    let hend = h(*xs.last().unwrap_or(&x0), *ys.last().unwrap_or(&y0));
    LotkaVolterra { x: xs, y: ys, h_drift: (hend - h0).abs() / h0.abs().max(1e-12), h0 }
}

// ---------------------------------------------------------------------------------------------------------
// 429 — spike-train analysis
// ---------------------------------------------------------------------------------------------------------

pub struct SpikeStats {
    pub n: usize,
    pub mean_isi: f64,
    pub cv: f64,
    pub fano: f64,
    pub rate_hz: f64,
}

/// ISI statistics, coefficient of variation and the Fano factor of the binned count.
/// References: a Poisson process gives CV ≈ 1 and Fano ≈ 1; a refractory period pulls CV below 1.
pub fn spike_stats(times: &[f64], duration: f64, bin: f64) -> SpikeStats {
    let n = times.len();
    if n < 2 {
        return SpikeStats { n, mean_isi: 0.0, cv: 0.0, fano: 0.0, rate_hz: n as f64 / duration };
    }
    let mut isis = Vec::with_capacity(n - 1);
    for i in 1..n {
        isis.push(times[i] - times[i - 1]);
    }
    let mean = isis.iter().sum::<f64>() / isis.len() as f64;
    let var = isis.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / isis.len() as f64;
    let cv = if mean > 0.0 { sqrt(var) / mean } else { 0.0 };

    // Fano factor over fixed bins
    let nbins = (((duration / bin) as usize) + 1).max(1);
    let mut counts = vec![0.0f64; nbins];
    for &t in times {
        let idx = (t / bin) as usize;
        if idx < nbins {
            counts[idx] += 1.0;
        }
    }
    let cmean = counts.iter().sum::<f64>() / nbins as f64;
    let cvar = counts.iter().map(|c| (c - cmean) * (c - cmean)).sum::<f64>() / nbins as f64;
    let fano = if cmean > 0.0 { cvar / cmean } else { 0.0 };
    SpikeStats { n, mean_isi: mean, cv, fano, rate_hz: n as f64 / duration }
}

/// Poisson spike train from an LCG, with an optional absolute refractory period.
/// Reference: with `refractory = 0` the CV and Fano factor must both approach 1.
pub fn poisson_spikes(rate_hz: f64, duration: f64, refractory: f64, seed: u64) -> Vec<f64> {
    let mut rng = seed | 1;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    let mut t = 0.0;
    let mut out = Vec::new();
    while t < duration {
        let u = (next() as f64 / u64::MAX as f64).max(1e-12);
        let isi = -ln(u) / rate_hz;
        t += isi.max(refractory);
        if t < duration {
            out.push(t);
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------------------
// SRAM Ising engine bridge — used where the mapping is real
// ---------------------------------------------------------------------------------------------------------

/// Total energy of a 1-D nearest-neighbour Ising chain: H = −Σ J_i s_i s_(i+1).
pub fn ising_energy(spins: &[i8], couplings: &[i8]) -> i64 {
    let n = spins.len().min(couplings.len());
    let mut e = 0i64;
    for i in 0..n {
        let j = (i + 1) % n;
        e -= (couplings[i] as i64) * (spins[i] as i64) * (spins[j] as i64);
    }
    e
}

/// Relax a spin chain for `sweeps` using the SRAM pattern engine in `sram_pattern_computer`.
/// Reference: the returned energy must be ≤ the energy of a random configuration, and on a ferromagnetic chain
/// (all J > 0) it must reach the exact ground state −ΣJ.
pub fn ising_relax_sram(spins: &mut Vec<i8>, couplings: &[i8], sweeps: usize) -> i64 {
    for _ in 0..sweeps {
        *spins = crate::sram_pattern_computer::pattern_ising_spin_glass(spins, couplings);
    }
    ising_energy(spins, couplings)
}

// ---------------------------------------------------------------------------------------------------------
// 430 — suite index
// ---------------------------------------------------------------------------------------------------------

pub struct SuiteEntry {
    pub task: u32,
    pub name: &'static str,
    pub reference: &'static str,
    pub capability: &'static str,
}

pub fn suite_entries() -> Vec<SuiteEntry> {
    vec![
        SuiteEntry { task: 421, name: "molecular dynamics", reference: "energy conservation under velocity Verlet", capability: "WORKS" },
        SuiteEntry { task: 422, name: "quantum chemistry (Hückel)", reference: "benzene spectrum (−2,−1,−1,1,1,2)β", capability: "WORKS — NOT DFT" },
        SuiteEntry { task: 423, name: "protein folding (HP lattice)", reference: "exhaustive ground state on short sequences", capability: "WORKS AS A SEARCH — NOT STRUCTURE PREDICTION" },
        SuiteEntry { task: 424, name: "binding affinity (contact score)", reference: "analytic LJ well −ε at 2^(1/6)σ", capability: "PARTIAL — no correlation claim" },
        SuiteEntry { task: 425, name: "pharmacokinetics", reference: "closed-form 1- and 2-compartment", capability: "WORKS" },
        SuiteEntry { task: 426, name: "epidemiology (SIR)", reference: "transcendental final-size equation", capability: "WORKS" },
        SuiteEntry { task: 427, name: "population genetics", reference: "neutral fixation probability = p0", capability: "WORKS" },
        SuiteEntry { task: 428, name: "systems biology (Lotka–Volterra)", reference: "conserved quantity drift", capability: "WORKS" },
        SuiteEntry { task: 429, name: "spike-train analysis", reference: "Poisson → CV 1, Fano 1", capability: "WORKS" },
        SuiteEntry { task: 430, name: "suite", reference: "this index", capability: "WORKS" },
    ]
}
