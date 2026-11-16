//! Runnable test harness for the `trm_uefi_kernel` no_std library.
//!
//! Why this crate exists: `trm_uefi_kernel/src/main.rs` is `#![no_std]`
//! `#![no_main]` and owns a `#[panic_handler]`, so `cargo test` inside the
//! kernel crate dies with E0152 (duplicate lang item `panic_impl`) and runs
//! zero tests. `trm_uefi_kernel/src/lib.rs` is `#![no_std]` but defines no
//! panic handler, so a std crate may link it and supply the panic machinery.
//!
//! Every assertion below was written after reading the implementation, not
//! from an assumption about what Kleene K3 "should" do.

#![allow(clippy::bool_assert_comparison)]

#[cfg(test)]
mod k3_logic_tests {
    use trm_uefi_kernel::k3_logic::{
        sha256, task_hash_vector, KleeneState, KleeneTernaryEngine, DEFAULT_TAU_LOWER,
        DEFAULT_TAU_UPPER, FALSE, TRUE, UNKNOWN,
    };

    const TRITS: [i8; 3] = [FALSE, UNKNOWN, TRUE];

    // ---- 2.1 operators: full 3x3 truth tables -------------------------

    #[test]
    fn kleene_and_is_min_over_full_truth_table() {
        // src/k3_logic.rs: `kleene_and(x, y) = x.min(y)`
        let expected = [
            [-1, -1, -1], // x = FALSE
            [-1, 0, 0],   // x = UNKNOWN
            [-1, 0, 1],   // x = TRUE
        ];
        for (xi, &x) in TRITS.iter().enumerate() {
            for (yi, &y) in TRITS.iter().enumerate() {
                assert_eq!(
                    KleeneTernaryEngine::kleene_and(x, y),
                    expected[xi][yi],
                    "AND({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn kleene_or_is_max_over_full_truth_table() {
        // src/k3_logic.rs: `kleene_or(x, y) = x.max(y)`
        let expected = [
            [-1, 0, 1],
            [0, 0, 1],
            [1, 1, 1],
        ];
        for (xi, &x) in TRITS.iter().enumerate() {
            for (yi, &y) in TRITS.iter().enumerate() {
                assert_eq!(
                    KleeneTernaryEngine::kleene_or(x, y),
                    expected[xi][yi],
                    "OR({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn kleene_not_is_negation() {
        assert_eq!(KleeneTernaryEngine::kleene_not(FALSE), TRUE);
        assert_eq!(KleeneTernaryEngine::kleene_not(UNKNOWN), UNKNOWN);
        assert_eq!(KleeneTernaryEngine::kleene_not(TRUE), FALSE);
    }

    #[test]
    fn kleene_imply_is_not_x_or_y_over_full_truth_table() {
        // src/k3_logic.rs: `kleene_imply(x, y) = (-x).max(y)`
        let expected = [
            [1, 1, 1],   // FALSE -> anything is TRUE
            [0, 0, 1],   // UNKNOWN -> UNKNOWN unless consequent TRUE
            [-1, 0, 1],  // TRUE -> y
        ];
        for (xi, &x) in TRITS.iter().enumerate() {
            for (yi, &y) in TRITS.iter().enumerate() {
                assert_eq!(
                    KleeneTernaryEngine::kleene_imply(x, y),
                    expected[xi][yi],
                    "IMPLY({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn kleene_equiv_is_biconditional_over_full_truth_table() {
        // src/k3_logic.rs: `kleene_equiv(x, y) = min(max(-x,y), max(-y,x))`
        let expected = [
            [1, 0, -1],
            [0, 0, 0],
            [-1, 0, 1],
        ];
        for (xi, &x) in TRITS.iter().enumerate() {
            for (yi, &y) in TRITS.iter().enumerate() {
                assert_eq!(
                    KleeneTernaryEngine::kleene_equiv(x, y),
                    expected[xi][yi],
                    "EQUIV({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn kleene_not_is_involutive() {
        for &v in &TRITS {
            assert_eq!(KleeneTernaryEngine::kleene_not(KleeneTernaryEngine::kleene_not(v)), v);
        }
    }

    #[test]
    fn de_morgan_holds_for_and() {
        // NOT(AND(x,y)) == OR(NOT x, NOT y) holds in K3 because
        // -min(x,y) == max(-x,-y).
        for &x in &TRITS {
            for &y in &TRITS {
                let lhs = KleeneTernaryEngine::kleene_not(KleeneTernaryEngine::kleene_and(x, y));
                let rhs = KleeneTernaryEngine::kleene_or(
                    KleeneTernaryEngine::kleene_not(x),
                    KleeneTernaryEngine::kleene_not(y),
                );
                assert_eq!(lhs, rhs, "De Morgan AND at ({x}, {y})");
            }
        }
    }

    #[test]
    fn de_morgan_holds_for_or() {
        for &x in &TRITS {
            for &y in &TRITS {
                let lhs = KleeneTernaryEngine::kleene_not(KleeneTernaryEngine::kleene_or(x, y));
                let rhs = KleeneTernaryEngine::kleene_and(
                    KleeneTernaryEngine::kleene_not(x),
                    KleeneTernaryEngine::kleene_not(y),
                );
                assert_eq!(lhs, rhs, "De Morgan OR at ({x}, {y})");
            }
        }
    }

    #[test]
    fn excluded_middle_fails_in_k3() {
        // The defining non-classical property: UNKNOWN OR NOT UNKNOWN is
        // UNKNOWN, not TRUE. If this ever returns TRUE the engine has
        // silently become boolean.
        assert_eq!(
            KleeneTernaryEngine::kleene_or(UNKNOWN, KleeneTernaryEngine::kleene_not(UNKNOWN)),
            UNKNOWN
        );
        // ...while it does hold for the determinate values.
        for &v in &[FALSE, TRUE] {
            assert_eq!(
                KleeneTernaryEngine::kleene_or(v, KleeneTernaryEngine::kleene_not(v)),
                TRUE
            );
        }
    }

    // ---- 2.2 state-space reduction ------------------------------------

    #[test]
    fn quick_reduction_uses_strict_threshold_bands() {
        // src/k3_logic.rs: `v > tau_upper` -> TRUE, `v < tau_lower` -> FALSE,
        // else UNKNOWN. The comparisons are STRICT, so the exact boundary
        // values 0.35 and -0.35 fall into UNKNOWN.
        let e = KleeneTernaryEngine::new();
        assert_eq!(e.tau_upper, DEFAULT_TAU_UPPER);
        assert_eq!(e.tau_lower, DEFAULT_TAU_LOWER);
        assert_eq!(DEFAULT_TAU_UPPER, 0.35);
        assert_eq!(DEFAULT_TAU_LOWER, -0.35);

        let out = e.quick_reduction(&[
            0.36, 0.35, 0.0, -0.35, -0.36, 1.0, -1.0, 0.349_999, -0.349_999,
        ]);
        assert_eq!(
            out,
            vec![TRUE, UNKNOWN, UNKNOWN, UNKNOWN, FALSE, TRUE, FALSE, UNKNOWN, UNKNOWN]
        );
    }

    #[test]
    fn state_reduction_identity_matches_quick_reduction() {
        let e = KleeneTernaryEngine::new();
        let v = [0.9f32, 0.35, 0.0, -0.35, -0.9];
        assert_eq!(e.state_reduction(&v, None, None), e.quick_reduction(&v));
    }

    #[test]
    fn state_reduction_applies_projection_and_bias() {
        // proj_i = sum_j v_j * W_ij + b_i, then the same strict bands.
        let e = KleeneTernaryEngine::new();
        let v = [1.0f32, 1.0];
        let row0: &[f32] = &[0.2, 0.2]; // 0.4 + 0.0 = 0.4  -> TRUE
        let row1: &[f32] = &[0.2, 0.2]; // 0.4 - 0.1 = 0.3  -> UNKNOWN
        let row2: &[f32] = &[-0.2, -0.2]; // -0.4 + 0.0 = -0.4 -> FALSE
        let w: [&[f32]; 3] = [row0, row1, row2];
        let b = [0.0f32, -0.1, 0.0];
        assert_eq!(
            e.state_reduction(&v, Some(&w), Some(&b)),
            vec![TRUE, UNKNOWN, FALSE]
        );
    }

    // ---- 2.3 bypass decision ------------------------------------------

    #[test]
    fn bypass_decision_is_conjunction_with_confidence() {
        let mut e = KleeneTernaryEngine::new();

        // All TRUE -> TRUE, confidence 1.0, bypass counted.
        let s = e.bypass_decision(&[TRUE, TRUE, TRUE, TRUE]);
        assert_eq!(s.trit, TRUE);
        assert_eq!(s.confidence, 1.0);
        assert_eq!(s.ambiguity_ratio, 0.0);
        assert!(s.is_bypassable());
        assert_eq!(e.bypass_count, 1);

        // Any FALSE dominates (min) -> FALSE, confidence = false fraction.
        let s = e.bypass_decision(&[TRUE, FALSE, TRUE, TRUE]);
        assert_eq!(s.trit, FALSE);
        assert_eq!(s.confidence, 0.25);
        assert!(s.is_rejected());
        assert!(!s.is_bypassable());

        // UNKNOWN with no FALSE -> UNKNOWN, confidence = 1 - ambiguity.
        let s = e.bypass_decision(&[TRUE, UNKNOWN, TRUE, TRUE]);
        assert_eq!(s.trit, UNKNOWN);
        assert_eq!(s.ambiguity_ratio, 0.25);
        assert_eq!(s.confidence, 0.75);
        assert!(!s.is_bypassable());

        assert_eq!(e.operation_count, 3);
        assert_eq!(e.bypass_count, 1);
    }

    #[test]
    fn bypass_decision_on_empty_slice_is_unknown_with_full_confidence() {
        // Documented quirk of the implementation: total == 0.0 makes
        // ambiguity_ratio 0.0 and confidence 1.0 - 0.0 == 1.0, while the
        // decision trit is UNKNOWN. It is NOT bypassable because the trit
        // is not TRUE.
        let mut e = KleeneTernaryEngine::new();
        let s = e.bypass_decision(&[]);
        assert_eq!(s.trit, UNKNOWN);
        assert_eq!(s.ambiguity_ratio, 0.0);
        assert_eq!(s.confidence, 1.0);
        assert!(!s.is_bypassable());
        assert_eq!(e.bypass_count, 0);
    }

    #[test]
    fn bypass_requires_confidence_at_least_0_88() {
        // is_bypassable: trit == TRUE && confidence >= 0.88
        let yes = KleeneState { trit: TRUE, confidence: 0.88, ambiguity_ratio: 0.0 };
        let no = KleeneState { trit: TRUE, confidence: 0.879_999, ambiguity_ratio: 0.0 };
        assert!(yes.is_bypassable());
        assert!(!no.is_bypassable());
    }

    // ---- packing / bitvectors -----------------------------------------

    #[test]
    fn pack_unpack_roundtrips_all_243_encodings() {
        for byte in 0u16..243 {
            let trits = KleeneTernaryEngine::unpack_5_trits(byte as u8);
            assert_eq!(
                KleeneTernaryEngine::pack_5_trits(trits),
                byte as u8,
                "roundtrip failed for byte {byte}"
            );
            for t in trits {
                assert!((-1..=1).contains(&t), "unpacked trit out of range: {t}");
            }
        }
        // 3^5 - 1 is the largest representable byte.
        assert_eq!(KleeneTernaryEngine::pack_5_trits([TRUE; 5]), 242);
        assert_eq!(KleeneTernaryEngine::pack_5_trits([FALSE; 5]), 0);
    }

    #[test]
    fn dual_bitvector_roundtrip() {
        let psi = [TRUE, FALSE, UNKNOWN, TRUE, FALSE];
        let (p, n) = KleeneTernaryEngine::trit_vector_to_bitvectors(&psi);
        assert_eq!(p, vec![1, 0, 0, 1, 0]);
        assert_eq!(n, vec![0, 1, 0, 0, 1]);
        assert_eq!(KleeneTernaryEngine::bitvectors_to_trit(&p, &n), psi.to_vec());
    }

    #[test]
    fn vectorized_ops_match_scalar_ops() {
        let a = [TRUE, FALSE, UNKNOWN, TRUE];
        let b = [UNKNOWN, FALSE, TRUE, TRUE];
        for i in 0..a.len() {
            assert_eq!(
                KleeneTernaryEngine::vectorized_and(&a, &b)[i],
                KleeneTernaryEngine::kleene_and(a[i], b[i])
            );
            assert_eq!(
                KleeneTernaryEngine::vectorized_or(&a, &b)[i],
                KleeneTernaryEngine::kleene_or(a[i], b[i])
            );
        }
        assert_eq!(
            KleeneTernaryEngine::vectorized_not(&a),
            vec![FALSE, TRUE, UNKNOWN, FALSE]
        );
    }

    // ---- offload routing ----------------------------------------------

    #[test]
    fn offload_routing_uses_ambiguity_ratio_bands() {
        // ratio == 0.0 -> NATIVE_BYPASS; <= 0.60 -> INSTELLA_MOE; else BONSAI.
        let e = KleeneTernaryEngine::new();
        assert_eq!(e.evaluate_offload(&[TRUE, FALSE, TRUE]), "NATIVE_BYPASS");
        assert_eq!(e.evaluate_offload(&[TRUE, UNKNOWN, TRUE, TRUE, TRUE]), "INSTELLA_MOE"); // 0.2
        assert_eq!(e.evaluate_offload(&[UNKNOWN, UNKNOWN, UNKNOWN, TRUE, TRUE]), "INSTELLA_MOE"); // 0.6
        assert_eq!(e.evaluate_offload(&[UNKNOWN, UNKNOWN, UNKNOWN, UNKNOWN, TRUE]), "BONSAI_TERNARY"); // 0.8
        // Empty slice: total == 0 -> ratio 0.0 -> NATIVE_BYPASS.
        assert_eq!(e.evaluate_offload(&[]), "NATIVE_BYPASS");
    }

    // ---- hashing -------------------------------------------------------

    #[test]
    fn sha256_matches_fips_180_4_vectors() {
        // Known-answer tests; a broken SHA-256 would silently corrupt every
        // task_hash_vector and every council seal.
        let hex = |b: [u8; 32]| -> String {
            b.iter().map(|x| format!("{x:02x}")).collect()
        };
        assert_eq!(
            hex(sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex(sha256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq")),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn task_hash_vector_maps_bytes_into_unit_interval() {
        let v = task_hash_vector(b"sovereign motion");
        assert_eq!(v.len(), 32);
        for x in v {
            assert!((-1.0..=1.0).contains(&x), "out of range: {x}");
        }
        // Byte 0 -> -1.0, byte 255 -> +1.0.
        let h = sha256(b"sovereign motion");
        assert_eq!(v[0], (h[0] as f32 / 255.0) * 2.0 - 1.0);
    }
}

#[cfg(test)]
mod council_tests {
    use trm_uefi_kernel::council::{
        NativeSwarmCouncil, ProposalCategory, ProposalPhase,
    };
    use trm_uefi_kernel::pbft_consensus::{
        PbftConsensus, COMMIT_THRESHOLD, MAX_BYZANTINE, N_AGENTS, PHASE_ACCEPTED,
        PHASE_COMMIT, PHASE_PRE_PREPARE, PHASE_REJECTED, QUORUM_THRESHOLD,
    };

    // ---- PBFT quorum arithmetic (the tally rule, isolated) -------------

    #[test]
    fn pbft_constants_match_the_documented_model() {
        // N=14, f=4, quorum 2f+1=9, commit 10.
        assert_eq!(N_AGENTS, 14);
        assert_eq!(MAX_BYZANTINE, 4);
        assert_eq!(QUORUM_THRESHOLD, 9);
        assert_eq!(COMMIT_THRESHOLD, 10);
        assert_eq!(QUORUM_THRESHOLD, 2 * MAX_BYZANTINE + 1);
    }

    #[test]
    fn pbft_quorum_advances_to_commit_at_exactly_nine_approvals() {
        let mut c = PbftConsensus::new();
        let id = c.propose("OPERATOR", "security");
        assert_eq!(c.get_phase(id), PHASE_PRE_PREPARE);

        for i in 1..=8 {
            assert_eq!(c.prepare(id, true), i);
            assert_eq!(c.get_phase(id), PHASE_PRE_PREPARE, "advanced too early at {i}");
        }
        // The 9th approval is the quorum boundary.
        assert_eq!(c.prepare(id, true), 9);
        assert_eq!(c.get_phase(id), PHASE_COMMIT);
    }

    #[test]
    fn pbft_abstention_does_not_count_toward_quorum() {
        // `prepare(id, false)` is an abstain/reject vote: it returns the
        // unchanged count and never advances the phase.
        let mut c = PbftConsensus::new();
        let id = c.propose("OPERATOR", "security");
        for _ in 0..20 {
            assert_eq!(c.prepare(id, false), 0);
        }
        assert_eq!(c.get_phase(id), PHASE_PRE_PREPARE);

        // 8 approvals + 20 abstentions is still short of quorum.
        for _ in 0..8 {
            c.prepare(id, true);
        }
        assert_eq!(c.get_phase(id), PHASE_PRE_PREPARE);
        c.prepare(id, true);
        assert_eq!(c.get_phase(id), PHASE_COMMIT);
    }

    #[test]
    fn pbft_commit_requires_ten_votes_not_nine() {
        let mut c = PbftConsensus::new();
        let id = c.propose("OPERATOR", "code");
        for _ in 0..9 {
            c.prepare(id, true);
        }
        for i in 1..=9 {
            assert_eq!(c.commit(id), i);
            assert_eq!(c.get_phase(id), PHASE_COMMIT, "accepted at {i} commit votes");
        }
        assert_eq!(c.commit(id), 10);
        assert_eq!(c.get_phase(id), PHASE_ACCEPTED);
        assert_eq!(c.get_status().1, 1, "accepted_count");
    }

    #[test]
    fn pbft_commit_is_idempotent_after_acceptance() {
        // The `phase < PHASE_ACCEPTED` guard means extra commits must not
        // double-count accepted_count.
        let mut c = PbftConsensus::new();
        let id = c.propose("OPERATOR", "code");
        for _ in 0..10 {
            c.commit(id);
        }
        assert_eq!(c.get_phase(id), PHASE_ACCEPTED);
        for _ in 0..5 {
            c.commit(id);
        }
        assert_eq!(c.get_status().1, 1);
    }

    #[test]
    fn pbft_reject_overrides_phase_and_counts() {
        let mut c = PbftConsensus::new();
        let id = c.propose("OPERATOR", "data");
        for _ in 0..9 {
            c.prepare(id, true);
        }
        c.reject(id);
        assert_eq!(c.get_phase(id), PHASE_REJECTED);
        assert_eq!(c.get_status().2, 1, "rejected_count");
        // Unknown proposal ids are reported as rejected, not panicking.
        assert_eq!(c.get_phase(9999), PHASE_REJECTED);
        assert_eq!(c.prepare(9999, true), 0);
        assert_eq!(c.commit(9999), 0);
    }

    #[test]
    fn pbft_full_cycle_accepts_with_n_minus_f_honest_votes() {
        let mut c = PbftConsensus::new();
        let (id, accepted, score) = c.run_full_cycle("OPERATOR", "protocol");
        assert!(accepted);
        assert_eq!(score, 0.98);
        assert_eq!(c.get_phase(id), PHASE_ACCEPTED);
        let p = &c.proposals[0];
        assert_eq!(p.prepare_votes, N_AGENTS - MAX_BYZANTINE); // 10
        assert_eq!(p.commit_votes, COMMIT_THRESHOLD); // 10
    }

    // ---- 14-agent council tally ---------------------------------------

    #[test]
    fn council_tally_is_consistent_with_its_own_consensus_rule() {
        // NOTE: the tally lives in src/council.rs::NativeSwarmCouncil::evaluate,
        // not in main.rs (main.rs only renders it). The rule is
        //   consensus_reached = approve_count >= 10 && weighted_score > 0.35
        // and the runtime's dispatch() currently returns +1 for every agent,
        // so the observable tally is 14/0/0. We assert the invariants that
        // must hold for ANY vote distribution, plus the concrete outcome.
        let p = NativeSwarmCouncil::evaluate("harden the mesh", ProposalCategory::Security, "OPERATOR");

        assert_eq!(p.votes.len(), 14);
        assert_eq!(p.deliberations.len(), 14);
        assert_eq!(
            p.approve_count + p.abstain_count + p.reject_count,
            p.votes.len(),
            "every vote must be classified exactly once"
        );

        // Counts must agree with the raw trits.
        let approve = p.votes.iter().filter(|v| v.k3_trit == 1).count();
        let reject = p.votes.iter().filter(|v| v.k3_trit == -1).count();
        let abstain = p.votes.iter().filter(|v| v.k3_trit == 0).count();
        assert_eq!(p.approve_count, approve);
        assert_eq!(p.reject_count, reject);
        assert_eq!(p.abstain_count, abstain);

        // The consensus predicate must match the published rule exactly.
        let expected = p.approve_count >= 10 && p.weighted_score > 0.35;
        assert_eq!(p.consensus_reached, expected);
        assert_eq!(
            p.phase,
            if expected { ProposalPhase::Enacted } else { ProposalPhase::Rejected }
        );

        // weighted_score is a weighted mean of trits in [-1, 1].
        assert!((-1.0..=1.0).contains(&p.weighted_score), "score {}", p.weighted_score);

        // Current runtime: all 14 agents approve, so consensus must be reached.
        assert_eq!(p.approve_count, 14);
        assert_eq!(p.reject_count, 0);
        assert_eq!(p.abstain_count, 0);
        assert!(p.consensus_reached);
        assert_eq!(p.phase, ProposalPhase::Enacted);
    }

    #[test]
    fn council_seal_is_deterministic_for_identical_inputs() {
        // The seal folds trits and confidences only; it must not depend on
        // wall-clock time (proposal_id and duration_us do).
        let a = NativeSwarmCouncil::evaluate("seal check", ProposalCategory::Protocol, "OPERATOR");
        let b = NativeSwarmCouncil::evaluate("seal check", ProposalCategory::Protocol, "OPERATOR");
        assert_eq!(a.monad_seal, b.monad_seal);
        assert_eq!(a.weighted_score, b.weighted_score);
        assert_eq!(a.approve_count, b.approve_count);
    }

    #[test]
    fn council_category_parsing_and_domain_weights() {
        assert_eq!(ProposalCategory::from_str("security"), ProposalCategory::Security);
        assert_eq!(ProposalCategory::from_str("SEC"), ProposalCategory::Security);
        assert_eq!(ProposalCategory::from_str("arch"), ProposalCategory::Architecture);
        assert_eq!(ProposalCategory::from_str("nonsense"), ProposalCategory::General);
        assert_eq!(ProposalCategory::Security.name(), "Security");

        use trm_uefi_kernel::agent_runtime::SwarmAgentId;
        use trm_uefi_kernel::council::get_domain_weight;
        // Domain specialists get 1.0, everyone else the 0.45 floor.
        assert_eq!(get_domain_weight(SwarmAgentId::Shield, ProposalCategory::Security), 1.0);
        assert_eq!(get_domain_weight(SwarmAgentId::Archi, ProposalCategory::Architecture), 1.0);
        assert_eq!(get_domain_weight(SwarmAgentId::Seeker, ProposalCategory::Security), 0.45);
    }
}

#[cfg(test)]
mod k3_ternary_sram_tests {
    use trm_uefi_kernel::k3_logic::{FALSE, TRUE, UNKNOWN};
    use trm_uefi_kernel::k3_ternary_sram::{K3Sram, CAM_LINES, CAM_TAG_LEN, SRAM_TRITS};

    fn padded(key: &[u8]) -> [u8; CAM_TAG_LEN] {
        let mut p = [0u8; CAM_TAG_LEN];
        let n = key.len().min(CAM_TAG_LEN);
        p[..n].copy_from_slice(&key[..n]);
        p
    }

    /// Independent scalar reference: count tag rows equal to the padded key.
    fn scalar_scan(sram: &K3Sram, key: &[u8]) -> u64 {
        let k = padded(key);
        sram.tags.iter().filter(|t| **t == k).count() as u64
    }

    #[test]
    fn cam_lookup_matches_scalar_scan() {
        let mut sram = K3Sram::new();
        let keys: [&[u8]; 5] = [b"alpha", b"beta", b"gamma", b"delta", b"epsilon"];
        for (i, k) in keys.iter().enumerate() {
            sram.store_entry(k, if i % 2 == 0 { TRUE } else { FALSE }, 0.5 + i as f32 * 0.1);
        }

        for k in keys {
            let expected = scalar_scan(&sram, k);
            assert_eq!(sram.cam_lookup(k), expected, "CAM vs scalar for {:?}", k);
            assert_eq!(expected, 1, "each stored key must occupy exactly one line");
        }

        // A key that was never stored matches nothing.
        assert_eq!(sram.cam_lookup(b"zeta"), 0);
        assert_eq!(scalar_scan(&sram, b"zeta"), 0);
    }

    #[test]
    fn cam_lookup_counts_duplicate_tags() {
        // store_pattern writes tags[0] directly; a second identical tag can
        // only arrive via store_entry, which dedupes. So duplicates are
        // created by writing the tag plane, and the scan must count them.
        let mut sram = K3Sram::new();
        sram.store_entry(b"dup", TRUE, 1.0);
        sram.tags[7] = padded(b"dup");
        assert_eq!(sram.cam_lookup(b"dup"), 2);
        assert_eq!(sram.cam_lookup(b"dup"), scalar_scan(&sram, b"dup"));
    }

    #[test]
    fn store_entry_updates_existing_key_in_place() {
        let mut sram = K3Sram::new();
        let i0 = sram.store_entry(b"key", TRUE, 0.4);
        let i1 = sram.store_entry(b"key", FALSE, 0.9);
        assert_eq!(i0, i1, "same key must reuse the same line");
        assert_eq!(sram.lookup_entry(b"key"), Some((FALSE, 0.9)));
        assert_eq!(sram.evictions, 0);
    }

    #[test]
    fn lookup_entry_misses_and_tracks_hits() {
        let mut sram = K3Sram::new();
        assert_eq!(sram.lookup_entry(b"absent"), None);
        assert_eq!(sram.cam_hits, 0);
        assert_eq!(sram.cam_lookups, 1);

        sram.store_entry(b"present", UNKNOWN, 0.25);
        assert_eq!(sram.lookup_entry(b"present"), Some((UNKNOWN, 0.25)));
        assert_eq!(sram.cam_hits, 1);
        assert_eq!(sram.cam_hit_ratio(), 0.5);
    }

    #[test]
    fn cam_evicts_least_recently_used_line_when_full() {
        let mut sram = K3Sram::new();
        // Fill every line; line i is stored at tick i+1.
        for i in 0..CAM_LINES {
            let key = [i as u8, 0xAA];
            sram.store_entry(&key, TRUE, 1.0);
        }
        assert_eq!(sram.evictions, 0);

        // Touch line 0 so it is no longer the oldest.
        assert!(sram.lookup_entry(&[0u8, 0xAA]).is_some());

        // One more distinct key must evict line 1 (tick 2), not line 0.
        sram.store_entry(&[0xFF, 0xBB], TRUE, 1.0);
        assert_eq!(sram.evictions, 1);
        assert!(sram.lookup_entry(&[0u8, 0xAA]).is_some(), "recently used line was evicted");
        assert_eq!(sram.lookup_entry(&[1u8, 0xAA]), None, "LRU line survived");
    }

    #[test]
    fn store_pattern_writes_plane_and_tags_line_zero() {
        let mut sram = K3Sram::new();
        let n = sram.store_pattern(b"pattern-a", &[TRUE, UNKNOWN, FALSE, TRUE]);
        assert_eq!(n, 4);
        assert_eq!(sram.stored(), 4);
        assert_eq!(&sram.trits[..4], &[TRUE, UNKNOWN, FALSE, TRUE]);
        assert_eq!(sram.trits[4], UNKNOWN, "untouched cells stay UNKNOWN");
        assert_eq!(sram.cam_lookup(b"pattern-a"), 1);
        // decision on line 0 is the conjunction (min) of the pattern.
        assert_eq!(sram.lookup_entry(b"pattern-a").map(|(d, _)| d), Some(FALSE));
    }

    #[test]
    fn store_pattern_truncates_to_plane_size() {
        let mut sram = K3Sram::new();
        let big = vec![TRUE; SRAM_TRITS + 100];
        assert_eq!(sram.store_pattern(b"big", &big), SRAM_TRITS);
        assert_eq!(sram.stored(), SRAM_TRITS);
    }

    #[test]
    fn k3_conjunction_applies_kleene_and_in_place() {
        let mut sram = K3Sram::new();
        sram.store_pattern(b"conj", &[TRUE, UNKNOWN, FALSE, TRUE]);
        let out = sram.k3_conjunction(0, 4, UNKNOWN);
        // min(x, 0) == 0 for x in {1,0,-1} except -1 -> -1
        assert_eq!(out, vec![UNKNOWN, UNKNOWN, FALSE, UNKNOWN]);
        assert_eq!(&sram.trits[..4], &[UNKNOWN, UNKNOWN, FALSE, UNKNOWN]);

        // Clamped window: start+len beyond the plane must not panic.
        let out = sram.k3_conjunction(SRAM_TRITS - 2, 10, TRUE);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn utilization_counts_non_unknown_cells_over_plane_size() {
        let mut sram = K3Sram::new();
        assert_eq!(sram.utilization(), 0.0);
        sram.store_pattern(b"u", &[TRUE, UNKNOWN, FALSE]);
        // 2 determinate cells out of SRAM_TRITS (not out of `stored`).
        assert_eq!(sram.utilization(), 2.0 / SRAM_TRITS as f32);
    }

    #[test]
    fn bypass_over_stored_reports_unknown_before_any_pattern() {
        let mut sram = K3Sram::new();
        let s = sram.bypass_over_stored();
        assert_eq!(s.trit, UNKNOWN);
        assert_eq!(s.confidence, 0.0);
        assert_eq!(s.ambiguity_ratio, 1.0);

        sram.store_pattern(b"all-true", &[TRUE, TRUE, TRUE]);
        let s = sram.bypass_over_stored();
        assert_eq!(s.trit, TRUE);
        assert_eq!(s.confidence, 1.0);
        assert!(s.is_bypassable());
    }

    #[test]
    fn tcam_match_honours_wildcard_mask() {
        let mut sram = K3Sram::new();
        sram.store_pattern(b"tcam", &[TRUE, FALSE, UNKNOWN, TRUE]);
        // mask 0 == wildcard, non-zero == must match.
        assert!(sram.tcam_match(&[TRUE, FALSE, UNKNOWN, TRUE], &[1, 1, 1, 1]));
        assert!(sram.tcam_match(&[TRUE, TRUE, TRUE, TRUE], &[1, 0, 0, 0]));
        assert!(!sram.tcam_match(&[FALSE, FALSE, UNKNOWN, TRUE], &[1, 1, 1, 1]));
    }
}
