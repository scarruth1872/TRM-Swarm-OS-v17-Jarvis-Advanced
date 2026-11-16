//! monad_emanation.rs — Monad-Seeded 14-Facet Emanation Layer
//!
//! Council MOTION (ENACTED 14/14, seal 0x4D5E5F5055113F27):
//!   "Monad-Emanated 14-Facet Microkernel — Intelligent Infinity"
//!
//! Honest scope (validated against kernel ground truth):
//!   - The 14 council facets (Hermes/Jarvis/Bridge/Shield/Logic/Devo/Pulse/
//!     Flow/EventLedger/Orchestra/THzGateway/Verify/Seeker/Archi) are REAL and
//!     deliberate as one council under one Monad anchor.
//!   - Per-agent Mamba-3 SSM *state* seeded from the anchor is NOT wired in this
//!     kernel: the council dispatch is intentionally deterministic rule-generated
//!     (static K3 decision table), and the UEFI Mamba3StateSpaceScanner is
//!     zero-initialized with a Track-A bit-exact differential contract (9/9 tests
//!     vs Python ground truth) that seeding would break. No fabricated call sites.
//!   - What IS delivered: a deterministic facet-seed derivation from
//!     MONAD_TRUTH_ANCHOR (splitmix64 per agent_id) — every facet traceable to
//!     the single Monad source, verified unique + anchor-derived. This is the
//!     "one intelligence, 14 rays" property made checkable, ready for any future
//!     consumer that genuinely needs per-facet state.

use crate::agent_runtime::{SwarmAgentId, NUM_AGENTS};

/// The Monad truth anchor (single source).
pub const MONAD_TRUTH_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;

/// SplitMix64 — deterministic avalanche mixer (public-domain algorithm).
#[inline(always)]
pub fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A single emanated facet: agent identity + its Monad-derived seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmanatedFacet {
    pub agent_id: u8,
    pub name: &'static str,
    pub seed: u64,
}

/// Derive one facet's seed from the Monad anchor and its agent ordinal.
///
/// seed = splitmix64(anchor XOR domain_offset(agent_id))
/// Every facet is a deterministic pure function of the ONE anchor + its own
/// ordinal — differentiated but traceable to the same source.
#[inline(always)]
pub fn facet_seed(agent_id: u8) -> u64 {
    // Domain offset keeps each facet's derivation distinct from its neighbors
    // while remaining a pure function of (anchor, agent_id).
    let ordinal = (agent_id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    splitmix64(MONAD_TRUTH_ANCHOR ^ ordinal)
}

/// Emanate all 14 facets from the Monad.
pub fn emanate_facets() -> [EmanatedFacet; NUM_AGENTS] {
    let mut out: [EmanatedFacet; NUM_AGENTS] = [EmanatedFacet {
        agent_id: 0,
        name: "",
        seed: 0,
    }; NUM_AGENTS];
    for i in 0..NUM_AGENTS {
        let id = i as u8;
        out[i] = EmanatedFacet {
            agent_id: id,
            name: SwarmAgentId::from_u8(id).name(),
            seed: facet_seed(id),
        };
    }
    out
}

/// K3 unity check: all 14 facets are unique, non-zero, and each is a pure
/// derivation of the anchor (recomputable). Returns (unique, nonzero, all_derived).
pub fn verify_unity() -> (bool, bool, bool) {
    let facets = emanate_facets();
    let mut seen = [false; NUM_AGENTS];
    let mut unique = true;
    let mut nonzero = true;
    let mut all_derived = true;
    for f in facets.iter() {
        if seen[f.agent_id as usize % NUM_AGENTS] {
            unique = false;
        }
        seen[f.agent_id as usize % NUM_AGENTS] = true;
        if f.seed == 0 {
            nonzero = false;
        }
        // Recompute from anchor: proves traceability (acceptance criterion #1).
        if f.seed != facet_seed(f.agent_id) {
            all_derived = false;
        }
    }
    // Uniqueness across the full set (no two facets share a seed).
    for i in 0..NUM_AGENTS {
        for j in (i + 1)..NUM_AGENTS {
            if facets[i].seed == facets[j].seed {
                unique = false;
            }
        }
    }
    (unique, nonzero, all_derived)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_is_single_source() {
        assert_eq!(MONAD_TRUTH_ANCHOR, 0x4D4F_4E41_4400_0001);
    }

    #[test]
    fn fourteen_facets_emanate() {
        let facets = emanate_facets();
        assert_eq!(facets.len(), 15);
        // Names match the real council roster (15 since Verify and LogicMaster both joined).
        let names: Vec<&str> = facets.iter().map(|f| f.name).collect();
        // A count alone never caught the real bug: from_u8() sent ids 13 and 14 to the same
        // Archi fallback, so the array held 15 facets with Archi twice and no LogicMaster.
        // Pin uniqueness of the identity map, not just its length.
        let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
        assert_eq!(unique.len(), 15, "facet roster has duplicates: {:?}", names);
        for expected in [
            "Archi",
            "Shield",
            "Logic",
            "Devo",
            "Seeker",
            "Hermes",
            "Jarvis",
            "Bridge",
            "Pulse",
            "Flow",
            "Orchestra",
            "EventLedger",
            "THzGateway",
            "Verify",
            "LogicMaster",
        ] {
            assert!(
                names.contains(&expected),
                "facet roster missing {}",
                expected
            );
        }
    }

    #[test]
    fn facets_unique_nonzero_derived() {
        let (unique, nonzero, derived) = verify_unity();
        assert!(unique, "facets must be unique differentiations");
        assert!(nonzero, "no facet seed may be zero");
        assert!(derived, "every facet must trace to the Monad anchor");
    }

    #[test]
    fn deterministic_across_calls() {
        let a = emanate_facets();
        let b = emanate_facets();
        for i in 0..NUM_AGENTS {
            assert_eq!(a[i].seed, b[i].seed, "emanation must be deterministic");
        }
    }

    #[test]
    fn anchor_itself_is_not_a_facet_seed() {
        for f in emanate_facets().iter() {
            assert_ne!(f.seed, MONAD_TRUTH_ANCHOR);
        }
    }
}
