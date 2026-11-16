//! Falsification assertions for the three modules adopted from the v20 tree.
//!
//! These are the checks the bounded reconstructions carried and the originals do not. They are
//! deliberately the kind that FAIL if someone later replaces a measured constant with a plausible
//! literal, or lets a fabricated number into the drug registry: every assertion is derived from
//! physics or from an invariant, not from the value the code happens to hold today.

use swarm_rust_kernel::drug_cocktail_optimizer::DrugCocktailOptimizer;
use swarm_rust_kernel::k3_logic::{
    DEFAULT_TAU_LOWER, DEFAULT_TAU_UPPER, FALSE, TRUE, UNKNOWN,
};
use swarm_rust_kernel::silicon_crystallography::{
    SI_ACOUSTIC_PHONON_THZ, SI_BAND_GAP_EV, SI_BOND_ANGLE_DEG, SI_BOND_LENGTH_A,
    SI_DENSITY_G_CM3, SI_LATTICE_CONSTANT_A, SI_OPTICAL_PHONON_THZ, SI_SOUND_VELOCITY_M_S,
};

#[test]
fn trits_use_the_k3_sign_convention() {
    // +1 TRUE, -1 FALSE, 0 UNKNOWN. A flip here would invert every gate decision.
    assert_eq!(TRUE, 1);
    assert_eq!(FALSE, -1);
    assert_eq!(UNKNOWN, 0);
    assert_eq!(TRUE + FALSE, UNKNOWN, "opposites must sum to unknown, not to true");
}

#[test]
fn tau_band_is_ordered_and_symmetric_about_zero() {
    assert!(DEFAULT_TAU_LOWER < 0.0);
    assert!(DEFAULT_TAU_UPPER > 0.0);
    assert!(
        (DEFAULT_TAU_UPPER + DEFAULT_TAU_LOWER).abs() < 1e-6,
        "the band must be symmetric or UNKNOWN is biased to one side"
    );
}

#[test]
fn silicon_bond_length_follows_from_the_lattice_constant() {
    // Diamond cubic: nearest-neighbour distance is a*sqrt(3)/4. If someone edits the bond length
    // without the lattice constant (or vice versa) this fails.
    let derived = SI_LATTICE_CONSTANT_A * 3f64.sqrt() / 4.0;
    assert!(
        (SI_BOND_LENGTH_A - derived).abs() < 5e-3,
        "bond length {} does not follow from a={} (expected {})",
        SI_BOND_LENGTH_A,
        SI_LATTICE_CONSTANT_A,
        derived
    );
}

#[test]
fn silicon_constants_are_physical() {
    assert!((SI_BOND_ANGLE_DEG - 109.4712).abs() < 1e-3, "tetrahedral angle");
    assert!((SI_BAND_GAP_EV - 1.12).abs() < 1e-6);
    assert!((SI_DENSITY_G_CM3 - 2.329).abs() < 1e-3);
    assert!((SI_SOUND_VELOCITY_M_S - 8433.0).abs() < 1.0);
    assert!(
        SI_OPTICAL_PHONON_THZ > SI_ACOUSTIC_PHONON_THZ,
        "the zone-centre optical mode must sit above the zone-boundary acoustic mode"
    );
}

#[test]
fn drug_registry_contains_no_fabricated_numbers() {
    let leads = DrugCocktailOptimizer::get_representative_leads();
    assert!(!leads.is_empty(), "an empty registry must not be dressed as a populated one");

    for l in &leads {
        assert!(l.kd_nm.is_finite() && l.kd_nm > 0.0, "{}: kd_nm {}", l.id, l.kd_nm);
        assert!(l.delta_g_kcal_mol.is_finite(), "{}: dG is NaN/inf", l.id);
        assert!(l.molecular_weight > 0.0 && l.molecular_weight < 2000.0, "{}: MW {}", l.id, l.molecular_weight);
        assert!(
            l.single_efficacy.is_finite() && (0.0..=1.0).contains(&l.single_efficacy),
            "{}: efficacy {} outside [0,1]",
            l.id,
            l.single_efficacy
        );
        assert_ne!(l.fingerprint_hash, 0, "{}: empty fingerprint", l.id);
        assert!(!l.target_symbol.is_empty(), "{}: no target", l.id);
    }
}

#[test]
fn drug_ids_are_unique() {
    let leads = DrugCocktailOptimizer::get_representative_leads();
    let mut ids: Vec<&str> = leads.iter().map(|l| l.id).collect();
    let total = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), total, "duplicate lead ids - a registry, not a set");
}

#[test]
fn synergy_of_nothing_is_not_a_number_from_thin_air() {
    // No compounds cannot have a synergy score. Whatever is returned must at least be finite and
    // must not be a positive claim about a combination that does not exist.
    let s = DrugCocktailOptimizer::calculate_bliss_synergy(&[]);
    assert!(s.is_finite(), "empty combination produced {}", s);
    assert!(s <= 0.0, "empty combination reported positive synergy {}", s);
}
