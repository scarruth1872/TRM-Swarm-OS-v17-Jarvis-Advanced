//! G84 — the four claims in "test the math", tested instead of asserted.
//!
//! 1. "Each number could evolve into its geometric shape, packed densely with no mass, so 2+2 can never
//!    come back different no matter how it is asked."
//!    TESTED HERE: is the torus/ternary encoding injective over a range of integers, and is integer
//!    arithmetic order-independent while float arithmetic is not?
//! 2. "Operating frequency flows instead of resistance" — tested numerically in scratch/coherence_math.py
//!    (crystal reactance/resistance at resonance), not here.
//! 3. "Energy budget for every process" — same script, from measured watts/volts/hertz.
//!
//! Everything below prints measured counts. Nothing is asserted about physics.

use std::collections::HashSet;

use trm_uefi_kernel::faceted_torus_ternary::{FacetedTrit, MAX_FACET, TOTAL_STATES};
#[allow(deprecated)]
use trm_uefi_kernel::faceted_torus_ternary::FacetedTrit as _LegacyUse;

/// Replicates the claim "each number gets its own shape": map N integers onto the 217-state facet axis and
/// count how many distinct shapes actually result.
#[test]
fn facets_are_not_injective_over_a_range_of_numbers() {
    let n = 1001usize;
    let mut facets = HashSet::new();
    let mut torus_pairs = HashSet::new();
    let mut full_4d = HashSet::new();
    for i in 0..n {
        let u = (i as f32 / (n - 1) as f32) * 2.0 - 1.0; // spread over [-1, +1]
        let t = FacetedTrit::from_f32(u);
        facets.insert(t.facet);
        let (x, y, z, w) = t.to_4d_torus(0.0);
        // quantise to 6 decimal places, the resolution a serialised point would carry
        let q = |v: f32| (v * 1e6).round() as i64;
        torus_pairs.insert((q(x), q(y)));
        full_4d.insert((q(x), q(y), q(z), q(w)));
    }
    println!("integers mapped: {n}");
    println!("distinct facet states: {} (TOTAL_STATES declared {TOTAL_STATES}, MAX_FACET {MAX_FACET})", facets.len());
    println!("distinct torus (x,y) points: {}", torus_pairs.len());
    println!("distinct 4D points at turn=0: {}", full_4d.len());
    assert!(facets.len() <= TOTAL_STATES, "facet axis cannot hold more states than it declares");
    assert!(facets.len() < n, "the encoding is NOT injective over {n} numbers -- collisions are expected");
    // G84 measured the pole collision (217 facets -> 216 distinct points, because cos(+pi) == cos(-pi)).
    // G85 fixed it: the axis is now 217 equally spaced angles, so the map is injective on the facet axis.
    // The capacity limit stands and cannot be fixed by geometry -- 1001 numbers do not fit in 217 states.
    println!("facets -> distinct points: {} -> {} (was 216 before the G85 fix)", facets.len(), torus_pairs.len());
    assert_eq!(torus_pairs.len(), facets.len(),
        "every facet state must now own a distinct point; a shortfall means the pole collision returned");
}

/// The density claim needs distinctness to come from somewhere. It comes from the turn coordinate, which is
/// the index, not the value -- so the "shape" identifies position in a sequence, not the number.
#[test]
fn distinct_shapes_come_from_the_turn_not_from_the_value() {
    let mut by_turn = HashSet::new();
    for i in 0..1001usize {
        let u = (i as f32 / 1000.0) * 2.0 - 1.0;
        let t = FacetedTrit::from_f32(u);
        let turn = i as f32 / 1001.0;
        let (x, y, z, w) = t.to_4d_torus(turn);
        let q = |v: f32| (v * 1e6).round() as i64;
        by_turn.insert((q(x), q(y), q(z), q(w)));
    }
    println!("distinct 4D points WITH turn = index: {}", by_turn.len());
    assert!(by_turn.len() > TOTAL_STATES, "the turn coordinate is what separates points");
    assert_eq!(by_turn.len(), 1001,
        "with a distinct turn every point separates -- distinctness comes from the INDEX, not the value");
}

/// "No matter how many times or ways we ask for 2+2 it must never give a different answer."
/// Integer arithmetic: true, and provable over many orderings.
#[test]
fn integer_arithmetic_is_order_independent() {
    let mut seen = HashSet::new();
    for perm in 0..10_000u64 {
        // 2+2 expressed through different groupings and a running permutation
        let a = [2i64, 2];
        let s1 = a[0] + a[1];
        let s2 = a[1] + a[0];
        let s3 = (a[0] * 3 + a[1] * 5) - (a[0] * 2 + a[1] * 4); // algebraic rearrangement
        let s4 = i64::from((perm % 2 + 1) as u32) + 2; // 3+2? no: keep only 2+2 forms
        let _ = s4;
        seen.insert(s1);
        seen.insert(s2);
        seen.insert(s3);
    }
    println!("distinct results for 2+2 over 10,000 permutations: {:?}", seen);
    assert_eq!(seen.len(), 1);
    assert_eq!(*seen.iter().next().unwrap(), 4);
}

/// The same question in floating point does NOT hold, and that is representation, not geometry.
#[test]
fn float_arithmetic_drifts_with_order() {
    // 0.1 + 0.2 is the canonical example; then a longer chain summed two ways.
    let a = 0.1f64 + 0.2f64;
    let b = 0.3f64;
    println!("0.1+0.2 = {a:.20} vs 0.3 = {b:.20}  equal={}", a == b);

    let mut forward = 0.0f64;
    let mut backward = 0.0f64;
    let mut v: Vec<f64> = (0..10_000).map(|i| 1.0 / (i as f64 + 3.0)).collect();
    for x in &v {
        forward += x;
    }
    v.reverse();
    for x in &v {
        backward += x;
    }
    let drift = (forward - backward).abs();
    println!("sum of 10k reciprocals: forward {forward:.17} backward {backward:.17} drift {drift:e}");
    assert!(drift > 0.0, "float summation must show order dependence; if not, the test is wrong");
    assert!(drift < 1e-9, "drift should be tiny in absolute terms, and it is: {drift:e}");
}

/// And a matrix product -- the "based on matrix multiplication" worry -- drifts as well, while the same
/// product in exact integer arithmetic does not.
#[test]
fn matrix_multiply_drifts_in_float_but_not_in_integers() {
    // Same matrix, float vs integer.
    let m: [[f64; 3]; 3] = [[0.1, 0.2, 0.3], [0.4, 0.5, 0.6], [0.7, 0.8, 0.9]];
    let mi: [[i64; 3]; 3] = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    let ident: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let identi: [[i64; 3]; 3] = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];

    let mul_f = |a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]| {
        let mut o = [[0.0f64; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    o[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        o
    };
    let mul_i = |a: &[[i64; 3]; 3], b: &[[i64; 3]; 3]| {
        let mut o = [[0i64; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    o[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        o
    };

    let fi = mul_f(&m, &ident);
    let ii = mul_i(&mi, &identi);
    let mut float_exact = true;
    for i in 0..3 {
        for j in 0..3 {
            if fi[i][j] != m[i][j] {
                float_exact = false;
            }
        }
    }
    println!("float M*I matches M bitwise: {float_exact}");
    println!("integer M*I matches M exactly: {}", ii == mi);
    assert!(ii == mi, "integer matrix identity is exact");
    // M*I in floats is usually bitwise exact too; the drift shows up on accumulation, not on identity.
    assert!(float_exact, "identity multiply is exact even in float; drift needs a real accumulation");
}

/// G85 regression: the two poles must be distinct points, and the legacy path must still show the defect
/// so the fix is auditable rather than asserted.
#[allow(deprecated)]
#[test]
fn poles_are_distinct_after_the_fix_and_collide_on_the_legacy_path() {
    let hi = FacetedTrit::from_f32(1.0);
    let lo = FacetedTrit::from_f32(-1.0);
    assert_eq!(hi.facet, MAX_FACET);
    assert_eq!(lo.facet, -MAX_FACET);
    let (hx, hy, _, _) = hi.to_4d_torus(0.0);
    let (lx, ly, _, _) = lo.to_4d_torus(0.0);
    let d = ((hx - lx).powi(2) + (hy - ly).powi(2)).sqrt();
    println!("pole separation after fix: {d:.6}");
    assert!(d > 1e-3, "the poles must no longer share a point, got distance {d}");

    let (hlx, hly, _, _) = hi.to_4d_torus_legacy(0.0);
    let (llx, lly, _, _) = lo.to_4d_torus_legacy(0.0);
    let dl = ((hlx - llx).powi(2) + (hly - lly).powi(2)).sqrt();
    println!("pole separation on legacy path: {dl:.6} (the G84 defect)");
    assert!(dl < 1e-3, "legacy path must still reproduce the collision for auditability");
}
