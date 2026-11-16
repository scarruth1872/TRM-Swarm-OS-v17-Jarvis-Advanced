//! Host verification for the no_std MCTS planning module.
mod mcts_plan;

fn main() {
    for n in [64u32, 120, 256] {
        let rep = mcts_plan::run(n);
        println!("=== MCTS {} simulations ===", rep.simulations);
        println!("nodes explored: {}", rep.nodes);
        println!("best action={} | visits={} | mean={:.3} | root_visits={}",
            rep.best_action, rep.best_visits, rep.best_mean, rep.root_visits);
        for (a, v, m) in &rep.child_stats {
            println!("  action={} visits={} mean={:.2}", a, v, m);
        }
    }
}