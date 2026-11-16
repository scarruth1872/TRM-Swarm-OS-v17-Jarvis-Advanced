extern crate alloc;
// PHOENIX docking results analysed with the PLATFORM'S OWN primitives:
//   * K3 Kleene ternary logic  (k3_logic.rs - the real engine, real thresholds)
//   * TurboVec 1.58-bit quantizer (turbovec.rs - real 16x compression)
//   * measured phi_v from the voltage binder (passed in, NEVER a constant)
//
// HONEST SCOPE: the ternary verdict below is a ternary decision ON A DOCKING
// SCORE. It is not a binding truth and not a therapeutic result. Vina predicts a
// binding energy for one rigid pose; K3 folds several such predictions into one
// of three states so a pipeline can triage them.
mod float_math;
mod turbovec;
mod time;
mod k3_logic;

use k3_logic::{KleeneTernaryEngine, TRUE, UNKNOWN, FALSE, DEFAULT_TAU_UPPER, DEFAULT_TAU_LOWER};
use turbovec::{TurboVec158Quantizer, TurboVec16xQuantizer};

// ── CALIBRATION OF THE BAND ────────────────────────────────────────────────
// FIRST ATTEMPT WAS WRONG. Mapping affinity with norm = -affinity/12.0 put
// almost every vina score above tau_upper and returned BIND for 9 of 10 targets
// - including one at -5.26 kcal/mol, which is barely above vina's decoy range.
// A classifier that says BIND for 90% of its inputs is not discriminating; it is
// the same defect as the 87% parity figure that turned out to be agreement
// between two models that both declined to decide.
//
// Vina scores are NOT calibrated binding energies. Its scoring function
// over-predicts affinity broadly, so absolute values mean little without a
// reference distribution. The band below is placed on a STATED basis:
//
//   affinity <= -8.5 kcal/mol  ->  +1  BIND     (at or better than the range
//                                               where a real lead is expected)
//   affinity >= -5.5 kcal/mol  ->  -1  REJECT   (vina's decoy/weak range)
//   between                    ->   0  UNKNOWN
//
// Anchored by the validation run: CHIR-090, a potent known inhibitor of its own
// target, scored -10.13. Anything the platform reports as BIND should be
// comparable to a molecule known to work.
// MEASURED, 2026-09-22. Was -8.5 on a STATED basis; the decoy/ROC experiment
// (evidence/phoenix_roc_analysis.txt) FALSIFIED it. 40 property-matched decoys
// docked against the same LpxC receptor reach -8.541, so a cut at -8.5 admits a
// decoy as BIND. At -9.0 zero decoys pass - at the cost of dropping one of the
// two crystal-confirmed actives, a false negative accepted deliberately because
// for a triage step a missed candidate is cheaper than a fabricated one.
//
//   6 of 40 decoys scored better than the WEAKER crystal-confirmed active
//   (NVS-LPXC-01, -8.087). The decoy floor sits BETWEEN the two actives.
//   0 of 40 decoys beat the stronger one (CHIR-090, -10.170).
//
// So: NO single threshold cleanly separates this active set from these decoys,
// and any BIND verdict resting on a bare vina score is weak evidence. The band
// below is the best available compromise, not a validated cut - validated would
// require a larger, experimentally-characterised active set.
// MEASURED, 2026-09-22 - and the measurement REMOVED the basis for this band.
//
// The decoy/ROC work (evidence/phoenix_roc_analysis.txt) falsified -8.5: decoys
// reach -8.541. So a larger, cleaner set was built - 24 ChEMBL actives against
// CHEMBL3855 (P. aeruginosa LpxC, the same protein as PDB 5U39) versus 10
// MEASURED-INACTIVE compounds (pChEMBL <= 4.5, IC50 >= 32 uM, tested against this
// exact target) plus the 40 synthetic decoys.
//
// Against synthetic decoys the protocol looked excellent: AUC 0.988.
// Against EXPERIMENTAL inactives it collapsed: AUC 0.771, CI [0.529, 0.963].
//
// The active/inactive separation is not there:
//   CHEMBL3356191, MEASURED INACTIVE, IC50 33,113 nM, docks at -10.360 - rank 2
//     of 32, beating every active but one.
//   CHEMBL4780842, MEASURED INACTIVE, IC50 63,096 nM, docks at -9.993 - rank 7.
//   0.20 kcal/mol separates a 33 uM compound from a 0.5 nM compound.
//   Spearman rho between docking score and measured pChEMBL = -0.395 (weak).
//   Measured-inactive ranks: 2, 7, 14, 25, 27, 28, 29, 30, 31, 32.
//
// CONCLUSION: no threshold on this score can triage. A cut that admits CHIR-090
// (-10.17) also admits a 33 uM inactive (-10.36). The K3 layer therefore must NOT
// emit BIND from a docking score alone - the honest verdict is UNKNOWN, and BIND
// requires corroborating evidence this pipeline does not have.
//
// The constants are kept only so the band remains computable for reference; the
// differential below the report reflects that scores are not decisive.
const BIND_AT: f64 = -9.0;
/// vina's weak/decoy range. Decoys here bottom out near -6.1, so -5.5
/// deliberately sits below the entire observed decoy distribution.
const REJECT_AT: f64 = -5.5;
/// Maps affinity onto the K3 band so that tau_upper 0.35 lands on BIND_AT and
/// tau_lower -0.35 lands on REJECT_AT.
const BAND_SPAN: f64 = BIND_AT - REJECT_AT;   // -3.0
const BAND_MID: f64 = (BIND_AT + REJECT_AT) / 2.0;  // -7.0

struct Target { name: &'static str, uniprot: &'static str, affinity: f64, lig_eff: f64 }

fn main() {
    let args: alloc::vec::Vec<alloc::string::String> = std::env::args().collect();
    let phi_v: f64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(f64::NAN);
    if phi_v.is_nan() {
        println!("REFUSING: no measured phi_v supplied. The voltage doctrine forbids a constant here.");
        return;
    }
    println!("measured phi_v = {:.4}  (supplied by the voltage binder, not a literal)", phi_v);
    println!("K3 thresholds: tau_upper={} tau_lower={}", DEFAULT_TAU_UPPER, DEFAULT_TAU_LOWER);
    println!();

    let targets: alloc::vec::Vec<Target> = std::env::args().skip(2)
        .filter_map(|a| {
            let p: alloc::vec::Vec<&str> = a.split(',').collect();
            if p.len() < 4 { return None; }
            Some(Target { name: Box::leak(p[0].to_string().into_boxed_str()),
                          uniprot: Box::leak(p[1].to_string().into_boxed_str()),
                          affinity: p[2].parse().ok()?, lig_eff: p[3].parse().ok()? })
        }).collect();

    if targets.is_empty() {
        println!("no target rows supplied");
        return;
    }

    let mut eng = KleeneTernaryEngine::new();
    println!("{:<10} {:>9} {:>9} {:>6} {:>9} {:>18}", "target", "affinity", "norm", "trit", "lig_eff", "verdict");
    let mut binds = 0; let mut rejects = 0; let mut unknown = 0;

    for t in &targets {
        // affinity (negative = favourable) -> +1 end of the band
        // calibrated: norm >= 0.35 means affinity <= -8.5, norm <= -0.35 means >= -5.5
        let norm = (((t.affinity - BAND_MID) / BAND_SPAN) * 2.0 * 0.35) as f32;
        // the REAL reduction, then the REAL bypass decision
        let psi = eng.quick_reduction(&[norm]);
        let st = eng.bypass_decision(&psi);
        // ENFORCED: the ROC measured that a docking score cannot separate actives
        // from measured inactives (a 33 uM inactive docks at -10.36, beating
        // CHIR-090). So a score alone may never yield BIND. It is downgraded to
        // UNKNOWN, and the row says why. Corroborating evidence is required, and
        // this pipeline has none for these targets.
        let (trit, verdict) = if st.is_rejected() {
            (FALSE, "REJECT")
        } else if st.is_bypassable() {
            (UNKNOWN, "UNKNOWN(score-only)")
        } else {
            (UNKNOWN, "UNKNOWN")
        };
        match trit { TRUE => binds += 1, FALSE => rejects += 1, _ => unknown += 1 }
        println!("{:<10} {:>9.2} {:>9.4} {:>6} {:>9.3} {:>18}",
                 t.name, t.affinity, norm, trit, t.lig_eff, verdict);
    }

    // TurboVec 1.58-bit: quantize the (affinity, ligand-efficiency) feature
    // vectors and rank them by cosine similarity to the strongest reference.
    println!();
    println!("TurboVec 1.58-bit quantisation of the [affinity, lig_efficiency] vectors:");
    let mut ref_vec: alloc::vec::Vec<f32> = alloc::vec::Vec::new();
    for t in &targets {
        let v = [((t.affinity - BAND_MID) / BAND_SPAN) as f32, t.lig_eff as f32];
        let (packed, scale) = TurboVec158Quantizer::quantize_158(&v);
        let back = TurboVec158Quantizer::dequantize_158(&packed, v.len(), scale);
        let sim = TurboVec16xQuantizer::fast_femto_cosine_similarity(&back, &v);
        if ref_vec.is_empty() { ref_vec = back.clone(); }
        println!("  {:<10} packed={:>3}B  raw={:?}  dequant={:?}  cos={:.6}",
                 t.name, packed.len(), v, back, sim);
    }

    println!();
    println!("K3 SUMMARY over {} targets: BIND={} UNKNOWN={} REJECT={}", targets.len(), binds, unknown, rejects);
    println!("Ternary state legend: +1={} 0={} -1={}", TRUE, UNKNOWN, FALSE);
    println!("NOTE: BIND is UNREACHABLE from a docking score alone. See the ROC evidence:");
    println!("      a measured-inactive at IC50 33 uM docks at -10.36, outranking CHIR-090.");
}
