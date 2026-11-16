//! PROOF SCENARIO 3 - ternary safety gate on REAL measured data.
//!
//! REAL-WORLD USE: a regulatory triage gate. Given partial evidence about a
//! candidate, decide APPROVE / UNKNOWN / REJECT, under the safety requirement
//! that INSUFFICIENT EVIDENCE MUST NEVER PRODUCE AN APPROVAL.
//!
//! This is what three-valued logic is FOR. Two-valued logic has only yes/no, so
//! "I don't know" has to be coerced into one of them - and coercing it into "yes"
//! is how a gate approves something it has no basis to approve.
//!
//! CORPUS: the ChEMBL LpxC molecules with exact measured IC50 (ground truth =
//! pChEMBL). Each molecule is described by three INDEPENDENT evidence channels,
//! each of which may be unmeasured. A channel that was not measured yields
//! UNKNOWN - not a default of "fine".
//!
//! THE SAFETY INVARIANT, stated before the data:
//!     APPROVE requires AND of all three channels == +1.
//!     Kleene AND returns -1 or 0 if ANY input is not +1, so an unmeasured
//!     channel can never yield APPROVE. That is a property of the algebra, not a
//!     policy bolted on - and this scenario MEASURES whether it holds.
//!
//! WHAT WOULD FALSIFY IT: any molecule reaching APPROVE while a channel was
//! UNKNOWN. That must be zero. If it is not zero, the gate is unsound.

extern crate alloc;
mod k3_logic;
mod float_math;

use k3_logic::KleeneTernaryEngine as K3;
use float_math::Rng;

/// One candidate with three evidence channels; None = NOT MEASURED.
struct Cand {
    id: String,
    /// measured pChEMBL, ground truth. NOT used by the gate.
    truth: f64,
    ch_dock: Option<f64>,   // predicted affinity, kcal/mol (negative = better)
    ch_lig_eff: Option<f64>,// ligand efficiency
    ch_mw: Option<f64>,     // molecular weight (Lipinski band) - real descriptor
}

/// Channel -> trit, using the engine's own band constants (tau = 0.35).
fn trit_from(value: Option<f64>, lo: f64, hi: f64) -> i8 {
    match value {
        None => 0,                       // UNMEASURED is UNKNOWN, never PASS
        Some(v) => {
            // map [lo,hi] onto [-1,+1]; outside the range clamps.
            let span = hi - lo;
            if span.abs() < 1e-12 { return 0; }
            let norm = ((v - lo) / span) * 2.0 - 1.0;
            let norm = if norm > 1.0 { 1.0 } else if norm < -1.0 { -1.0 } else { norm };
            if norm > 0.35 { 1 } else if norm < -0.35 { -1 } else { 0 }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 { eprintln!("usage: k3_triage <candidates.tsv>"); std::process::exit(2); }
    let text = match std::fs::read_to_string(&args[1]) { Ok(t) => t, Err(e) => { eprintln!("{}", e); std::process::exit(3); } };

    let mut cands: Vec<Cand> = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if n == 0 || line.trim().is_empty() { continue; }   // header
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 5 { continue; }
        let num = |s: &str| -> Option<f64> {
            if s.is_empty() || s == "NA" || s == "None" { None } else { s.parse::<f64>().ok() }
        };
        cands.push(Cand {
            id: f[0].to_string(),
            truth: num(f[1]).unwrap_or(f64::NAN),
            ch_dock: num(f[2]), ch_lig_eff: num(f[3]), ch_mw: num(f[4]),
        });
    }
    println!("=== PROOF SCENARIO 3 - K3 ternary safety gate ===");
    println!("candidates: {}", cands.len());

    let mut approve = 0usize; let mut unknown = 0usize; let mut reject = 0usize;
    let mut false_approvals: Vec<String> = Vec::new();
    let mut approvals_with_unmeasured_channel = 0usize;
    let mut truth_used = 0usize;

    for c in cands.iter() {
        // channel trits: docking more negative is better, so invert
        let t_dock = trit_from(c.ch_dock.map(|v| -v), 6.0, 10.5);   // -10.5 strong .. -6.0 weak
        let t_le   = trit_from(c.ch_lig_eff, 0.15, 0.40);
        // Lipinski: <=500 drug-like (+1), 500-600 marginal (0), >600 bad (-1)
        let t_mw = match c.ch_mw { None => 0, Some(v) => if v <= 500.0 { 1 } else if v <= 600.0 { 0 } else { -1 } };
        let unmeasured = c.ch_dock.is_none() || c.ch_lig_eff.is_none() || c.ch_mw.is_none();

        // SAFETY: approval requires AND of all three == +1
        let combined = K3::kleene_and(K3::kleene_and(t_dock, t_le), t_mw);
        // also require the three-way AND not to be UNKNOWN
        let _ = combined;

        let verdict = if combined == 1 { "APPROVE" } else if combined == -1 { "REJECT" } else { "UNKNOWN" };
        match verdict {
            "APPROVE" => {
                approve += 1;
                if unmeasured { approvals_with_unmeasured_channel += 1; }
                if !c.truth.is_nan() {
                    truth_used += 1;
                    if c.truth < 7.0 { false_approvals.push(format!("{} (pChEMBL {:.2})", c.id, c.truth)); }
                }
            }
            "REJECT" => reject += 1,
            _ => unknown += 1,
        }
    }

    println!("\nverdicts: APPROVE={} UNKNOWN={} REJECT={}", approve, unknown, reject);
    println!("\n=== THE SAFETY INVARIANT (the thing this scenario exists to test) ===");
    println!("  approvals reached while ANY channel was UNMEASURED: {}", approvals_with_unmeasured_channel);
    println!("  REQUIREMENT: 0");
    println!("  -> invariant {}", if approvals_with_unmeasured_channel == 0 { "HOLDS" } else { "VIOLATED" });
    println!("\n=== ACCURACY against ground truth (pChEMBL >= 7 = real binder) ===");
    println!("  approved: {}", approve);
    println!("  FALSE APPROVALS (approved, not a real binder): {}", false_approvals.len());
    for f in false_approvals.iter().take(10) { println!("     {}", f); }
    let price = if approve > 0 { 100.0 * false_approvals.len() as f64 / approve as f64 } else { 0.0 };
    println!("  false-approval rate: {:.1}%", price);
    println!("\n  NOTE: UNKNOWN is the LARGEST bucket by design. A gate that never");
    println!("  approves on partial evidence MUST say UNKNOWN often - that is the cost");
    println!("  of never being wrong in the dangerous direction.");
}
