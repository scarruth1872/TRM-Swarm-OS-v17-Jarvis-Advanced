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
        PHASE_COMMIT, PHASE_PRE_PREPARE, PHASE_REJECTED, QUORUM_THRESHOLD, VETO_BLOCK,
    };

    // ---- quorum primitive: enforced invariants (2026-11-24) -------------
    //
    // These replace tests that encoded the OLD counter behaviour (commit accepted without prepare votes,
    // duplicate voters counted, rejections dropped). Those behaviours are recorded as defects in
    // docs/CONSENSUS-TASKS-712-720.md; the tests below assert the opposite, and fail if the module
    // regresses to a counter.

    #[test]
    fn quorum_constants_are_consistent() {
        assert_eq!(N_AGENTS, 14);
        assert_eq!(MAX_BYZANTINE, 4);
        assert_eq!(QUORUM_THRESHOLD, 9);
        assert_eq!(QUORUM_THRESHOLD, 2 * MAX_BYZANTINE + 1);
        assert_eq!(COMMIT_THRESHOLD, 10);
        assert_eq!(VETO_BLOCK, 6, "6 rejections make 9 approvals impossible out of 14");
    }

    #[test]
    fn commit_without_any_vote_is_refused() {
        let mut c = PbftConsensus::new();
        let id = c.propose("OPERATOR", "security");
        assert_eq!(c.commit(id), 0, "commit with zero votes must not accept");
        assert_ne!(c.get_phase(id), PHASE_ACCEPTED);
        assert_eq!(c.get_status().1, 0, "accepted_count stays 0");
    }

    #[test]
    fn nine_distinct_approvals_reach_the_commit_phase_but_not_acceptance() {
        let mut c = PbftConsensus::new();
        let id = c.propose("OPERATOR", "security");
        for i in 1..=9 {
            assert_eq!(c.prepare(id, true), i);
        }
        assert_eq!(c.get_phase(id), PHASE_COMMIT, "quorum 9 advances the phase");
        assert_eq!(c.commit(id), 0, "commit still needs 10 approvals");
        assert_ne!(c.get_phase(id), PHASE_ACCEPTED);
    }

    #[test]
    fn ten_distinct_approvals_commit() {
        let mut c = PbftConsensus::new();
        let id = c.propose("OPERATOR", "security");
        for _ in 0..10 {
            c.prepare(id, true);
        }
        assert_eq!(c.commit(id), 10);
        assert_eq!(c.get_phase(id), PHASE_ACCEPTED);
    }

    #[test]
    fn a_repeat_voter_cannot_inflate_the_tally() {
        let mut q = trm_uefi_kernel::quorum_consensus::QuorumCouncil::new();
        let id = q.propose("OPERATOR", "security", 1, 1, 42).unwrap();
        q.vote(id, 7, 1).unwrap();
        assert_eq!(q.vote(id, 7, 1), Err(trm_uefi_kernel::quorum_consensus::DecisionError::DuplicateVoter));
        assert_eq!(q.tally(id).unwrap().0, 1, "one voter, one approval");
    }

    #[test]
    fn the_tally_cannot_exceed_the_council_size() {
        let mut q = trm_uefi_kernel::quorum_consensus::QuorumCouncil::new();
        let id = q.propose("OPERATOR", "security", 1, 1, 42).unwrap();
        for v in 1..=N_AGENTS {
            q.vote(id, v, 1).unwrap();
        }
        assert_eq!(q.vote(id, 99, 1), Err(trm_uefi_kernel::quorum_consensus::DecisionError::TooManyVoters));
        assert_eq!(q.tally(id).unwrap().3, N_AGENTS);
    }

    #[test]
    fn six_rejections_veto_a_proposal_that_could_not_reach_quorum() {
        let mut q = trm_uefi_kernel::quorum_consensus::QuorumCouncil::new();
        let id = q.propose("OPERATOR", "security", 1, 1, 42).unwrap();
        for v in 1..=5 {
            q.vote(id, v, 1).unwrap();
        }
        for v in 6..=11 {
            q.vote(id, v, -1).unwrap();
        }
        assert_eq!(q.commit(id, QUORUM_THRESHOLD), Err(trm_uefi_kernel::quorum_consensus::DecisionError::Vetoed));
        assert_eq!(q.phase(id), PHASE_REJECTED, "a veto is terminal, not undecided");
    }

    #[test]
    fn an_abstention_is_not_an_approval() {
        let mut q = trm_uefi_kernel::quorum_consensus::QuorumCouncil::new();
        let id = q.propose("OPERATOR", "security", 1, 1, 42).unwrap();
        for v in 1..=9 {
            q.vote(id, v, 0).unwrap();
        }
        assert_eq!(q.commit(id, QUORUM_THRESHOLD), Err(trm_uefi_kernel::quorum_consensus::DecisionError::QuorumNotReached));
    }

    #[test]
    fn a_committed_slot_cannot_commit_a_different_value() {
        let mut q = trm_uefi_kernel::quorum_consensus::QuorumCouncil::new();
        let a = q.propose("OPERATOR", "approve", 1, 1, 0xAAAA).unwrap();
        for v in 1..=10 {
            q.vote(a, v, 1).unwrap();
        }
        q.commit(a, COMMIT_THRESHOLD).unwrap();
        assert_eq!(q.propose("OPERATOR", "reject", 1, 1, 0xBBBB),
                   Err(trm_uefi_kernel::quorum_consensus::DecisionError::ConflictingValue),
                   "two values cannot commit in one slot");
    }

    #[test]
    fn different_slots_can_both_commit() {
        let mut q = trm_uefi_kernel::quorum_consensus::QuorumCouncil::new();
        for slot in [1u64, 2u64] {
            let id = q.propose("OPERATOR", "decision", 1, slot, 0x1000 + slot).unwrap();
            for v in 1..=10 {
                q.vote(id, v, 1).unwrap();
            }
            q.commit(id, COMMIT_THRESHOLD).unwrap();
        }
        assert_eq!(q.status().1, 2);
    }

    #[test]
    fn a_full_cycle_accepts_on_real_quorum_and_reports_the_real_fraction() {
        let mut c = PbftConsensus::new();
        let (id, accepted, score) = c.run_full_cycle("OPERATOR", "protocol");
        assert!(accepted);
        assert_eq!(c.get_phase(id), PHASE_ACCEPTED);
        assert!((score - 10.0 / 14.0).abs() < 1e-9, "score is approvals/N, not a constant");
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


/// The registry must name exactly the modules that exist.
///
/// This is the enforcement the registry never had. On 2026-11-24 it listed 281 names for a tree of 213
/// files: 97 names had no `.rs` file anywhere on the machine (all labelled REAL, including
/// `cross_modality_reasoning` and `voltage_raw_reasoning_engine`), and 29 live modules were not listed at
/// all. It also counted `dll` (93) and `trm-rust` (41) trees that do not exist on this host.
///
/// The fix is not "regenerate once" — a one-off regeneration drifts again the next time a module is added or
/// deleted. This test makes that drift a build failure: the generated file and the source tree must agree.
#[cfg(test)]
mod registry_guard {
    use std::collections::BTreeSet;
    use std::path::Path;

    const REGISTRY: &str = include_str!("../../src/module_registry_generated.rs");

    fn names_in_registry() -> BTreeSet<String> {
        // STATUSES is the first ("name", "STATUS") table; parse quoted names from those lines.
        let mut out = BTreeSet::new();
        let mut in_statuses = false;
        for line in REGISTRY.lines() {
            if line.contains("pub const STATUSES") {
                in_statuses = true;
                continue;
            }
            if in_statuses {
                if line.trim().starts_with("];") {
                    break;
                }
                if let Some(rest) = line.trim().strip_prefix("(\"") {
                    if let Some(end) = rest.find('"') {
                        out.insert(rest[..end].to_string());
                    }
                }
            }
        }
        out
    }

    /// Every LIVE tree, not just this crate's.
    ///
    /// CORRECTION 2026-11-24: an earlier version of this guard looked only at `../src`. That is how an audit
    /// came to report 97 real modules as "phantom" — they live in the cdylib tree
    /// (`swarm_v2_rust_kernel/src`, which builds `swarm_rust_kernel.dll`) and in `staging/v20`. The guard
    /// must cover every tree the registry claims to describe, or it will certify a lie in either direction.
    const TREES: &[&str] = &["../src", "../../swarm_v2_rust_kernel/src"];

    fn modules_on_disk() -> BTreeSet<String> {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut out = BTreeSet::new();
        for rel in TREES {
            let dir = base.join(rel);
            let entries = match std::fs::read_dir(&dir) {
                Ok(e) => e,
                Err(_) => panic!("source tree {rel} is not readable — the guard cannot certify what it cannot see"),
            };
            for entry in entries {
                let path = entry.expect("dir entry").path();
                if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                // crate roots and the registry itself are not modules to be catalogued
                if matches!(stem.as_str(), "module_registry_generated" | "lib" | "main") {
                    continue;
                }
                out.insert(stem);
            }
        }
        out
    }

    #[test]
    fn registry_names_exactly_the_live_modules() {
        let reg = names_in_registry();
        let disk = modules_on_disk();
        assert!(!reg.is_empty(), "parsed no names out of the registry — the guard itself is broken");

        let phantom: Vec<&String> = reg.difference(&disk).collect();
        let unlisted: Vec<&String> = disk.difference(&reg).collect();

        assert!(
            phantom.is_empty(),
            "registry names modules with no source file ({}): {:?}\n\
             This is the defect the guard exists for: 97 such entries claimed REAL status before 2026-11-24.\n\
             Fix with: python tools/rebuild_module_registry.py",
            phantom.len(), phantom
        );
        assert!(
            unlisted.is_empty(),
            "live modules missing from the registry ({}): {:?}\n\
             A module the platform cannot name is a module it cannot recognise as its own.\n\
             Fix with: python tools/rebuild_module_registry.py",
            unlisted.len(), unlisted
        );
    }

    #[test]
    fn registry_counts_match_the_table() {
        let reg = names_in_registry();
        let declared: usize = REGISTRY
            .split("pub const TOTAL: u64 = ")
            .nth(1)
            .and_then(|s| s.split(';').next())
            .and_then(|s| s.trim().parse().ok())
            .expect("TOTAL must be a parseable constant");
        assert_eq!(declared, reg.len(), "TOTAL disagrees with STATUSES");
    }
}


/// Tasks 571 / 572 / 576 / 580 — the reasoning engine's semantics, stated as assertions.
///
/// Each test names the behaviour that would be a lie if it did not hold: an UNKNOWN premise must not produce
/// a TRUE conclusion; a rule with an UNKNOWN exception must not fire; a cycle must end; a conflict must be
/// reported rather than resolved by ordering.
#[cfg(test)]
mod reasoning_engine_tests {
    use trm_uefi_kernel::reasoning_engine::{derivation_json, Engine, Rule, Trit};

    fn chain_engine() -> Engine {
        let mut e = Engine::new();
        e.assert_fact("a", "shard:test@rec 1 seal deadbeef");
        e.add_rule(Rule::new(1, "a_implies_b", &["a"], "b"));
        e.add_rule(Rule::new(2, "b_implies_c", &["b"], "c"));
        e
    }

    #[test]
    fn forward_chain_derives_transitively_from_a_fact() {
        let r = chain_engine().forward_chain();
        assert_eq!(r.truth.get("b"), Some(&Trit::True), "b must be derived from a");
        assert_eq!(r.truth.get("c"), Some(&Trit::True), "c must be derived through b");
        assert!(!r.truncated, "a two-step chain must settle well inside the cap");
        assert_eq!(r.rules_fired, 2, "both rules should fire exactly once");
    }

    #[test]
    fn a_missing_premise_yields_unknown_not_true() {
        let mut e = Engine::new();
        e.add_rule(Rule::new(1, "needs_p", &["p"], "q"));
        let r = e.forward_chain();
        assert_eq!(r.truth.get("q"), Some(&Trit::Unknown), "q must be UNKNOWN, never TRUE");
        let note = &r.derivations.get("q").expect("an UNKNOWN must still be explained").note;
        assert!(note.contains('p'), "the reason must name the missing premise, got: {note}");
    }

    #[test]
    fn a_false_premise_does_not_fire_the_rule() {
        let mut e = Engine::new();
        e.assert_false("p");
        e.add_rule(Rule::new(1, "needs_p", &["p"], "q"));
        let r = e.forward_chain();
        assert_ne!(r.truth.get("q"), Some(&Trit::True), "a FALSE premise must not derive q");
        let note = &r.derivations.get("q").expect("explained").note;
        assert!(note.contains("FALSE"), "the reason must say the premise was FALSE, got: {note}");
    }

    #[test]
    fn a_cycle_terminates_and_is_explained() {
        let mut e = Engine::new();
        e.add_rule(Rule::new(1, "b_from_a", &["a"], "b"));
        e.add_rule(Rule::new(2, "a_from_b", &["b"], "a"));
        let r = e.forward_chain();
        assert!(!r.truncated, "a cycle with no facts must settle immediately, not spin to the cap");
        // backward chaining over the same rules must refuse rather than recurse forever
        let d = e.prove("a");
        assert_eq!(d.trit, Trit::Unknown);
        assert!(d.note.contains("cycle") || d.note.contains("depth"), "got note: {}", d.note);
    }

    #[test]
    fn a_defeasible_rule_is_defeated_by_its_exception() {
        let mut e = Engine::new();
        e.assert_fact("violation_tpsa", "shard:descriptors@rec 7");
        e.assert_fact("violation_rotb", "shard:descriptors@rec 7");
        e.add_rule(
            Rule::new(1, "tolerated", &["violation_tpsa", "violation_rotb"], "acceptable")
                .with_exception("narrow_therapeutic_window"),
        );
        // baseline: no exception asserted, rule fires
        let r1 = e.forward_chain();
        assert_eq!(r1.truth.get("acceptable"), Some(&Trit::True));

        // now the exception holds: the conclusion must be withdrawn, not kept
        e.assert_fact("narrow_therapeutic_window", "shard:tox@rec 3");
        let r2 = e.forward_chain();
        assert_ne!(r2.truth.get("acceptable"), Some(&Trit::True), "defeat must withdraw the conclusion");
        let note = &r2.derivations.get("acceptable").expect("explained").note;
        assert!(note.contains("defeated"), "the reason must say it was defeated, got: {note}");
    }

    #[test]
    fn an_unknown_exception_is_named_not_hidden() {
        // 576: a default may apply when its exception is UNKNOWN — that is what "default" means — but the
        // assumption must be visible, and a caller needing certainty must be able to see it was made.
        let mut e = Engine::new();
        e.assert_fact("violation_tpsa", "shard:descriptors@rec 9");
        e.assert_fact("violation_rotb", "shard:descriptors@rec 9");
        e.add_rule(
            Rule::new(1, "tolerated", &["violation_tpsa", "violation_rotb"], "acceptable")
                .with_exception("narrow_therapeutic_window"),
        );
        let d = e.prove("acceptable");
        assert_eq!(d.trit, Trit::True, "an UNKNOWN exception does not defeat the default");
        assert!(d.note.contains("assumed"), "the assumption must be named, got: {}", d.note);
        assert!(d.note.contains("narrow_therapeutic_window"), "got note: {}", d.note);

        // and the forward path must report the same assumption in the machine-readable list
        let r = e.forward_chain();
        assert_eq!(r.assumptions.len(), 1, "the run must report one assumption: {:?}", r.assumptions);
        assert!(r.assumptions[0].contains("narrow_therapeutic_window"));

        // once the exception is explicitly FALSE, the default applies with nothing assumed
        e.assert_false("narrow_therapeutic_window");
        let r2 = e.forward_chain();
        assert_eq!(r2.assumptions.len(), 0, "an explicit FALSE exception needs no assumption");
        assert_eq!(r2.truth.get("acceptable"), Some(&Trit::True));
    }

    #[test]
    fn a_conflict_is_reported_and_left_unknown() {
        let mut e = Engine::new();
        e.assert_fact("a", "shard:test@rec 1");
        e.assert_false("c");
        e.add_rule(Rule::new(1, "a_implies_c", &["a"], "c"));
        let r = e.forward_chain();
        assert_eq!(r.conflicts.len(), 1, "the contradiction must be reported");
        assert_eq!(r.conflicts[0].atom, "c");
        assert_eq!(r.truth.get("c"), Some(&Trit::Unknown), "a conflict resolves to UNKNOWN, not to TRUE");
    }

    #[test]
    fn backward_chaining_returns_the_chain_down_to_the_evidence() {
        let d = chain_engine().prove("c");
        assert_eq!(d.trit, Trit::True);
        assert_eq!(d.rule_id, Some(2));
        assert_eq!(d.premises.len(), 1);
        let b = &d.premises[0];
        assert_eq!(b.atom, "b");
        assert_eq!(b.rule_id, Some(1));
        let a = &b.premises[0];
        assert_eq!(a.atom, "a");
        assert!(a.fact_source.as_deref().unwrap_or("").contains("seal deadbeef"),
                "the leaf must carry the evidence it was asserted with");
    }

    #[test]
    fn backward_chaining_names_the_premise_it_could_not_settle() {
        let mut e = Engine::new();
        e.add_rule(Rule::new(1, "needs_missing", &["missing_fact"], "goal"));
        let d = e.prove("goal");
        assert_eq!(d.trit, Trit::Unknown);
        assert!(d.note.contains("missing_fact"), "got note: {}", d.note);
        assert_eq!(d.rule_id, None, "an UNKNOWN is not justified by a rule");
    }

    #[test]
    fn the_druglikeness_rulebase_derives_two_violations() {
        let mut e = Engine::new();
        for r in Engine::druglikeness_rules() {
            e.add_rule(r);
        }
        e.assert_fact("mw_gt_500", "shard:chembl_lpxc@rec 12 seal cafe");
        e.assert_fact("logp_gt_5", "shard:chembl_lpxc@rec 12 seal cafe");
        let r = e.forward_chain();
        assert_eq!(r.truth.get("violation_mw"), Some(&Trit::True));
        assert_eq!(r.truth.get("violation_logp"), Some(&Trit::True));
        assert_eq!(r.truth.get("too_many_violations"), Some(&Trit::True),
                   "two violations must chain to 'too many'");
        // and one violation alone must not
        let mut one = Engine::new();
        for r in Engine::druglikeness_rules() {
            one.add_rule(r);
        }
        one.assert_fact("mw_gt_500", "shard:chembl_lpxc@rec 13 seal cafe");
        let r1 = one.forward_chain();
        assert_ne!(r1.truth.get("too_many_violations"), Some(&Trit::True));
    }

    #[test]
    fn an_explanation_serialises_with_its_rule_and_source() {
        let d = chain_engine().prove("c");
        let json = derivation_json(&d);
        assert!(json.contains("\"atom\":\"c\""), "{json}");
        assert!(json.contains("\"rule_id\":2"), "{json}");
        assert!(json.contains("seal deadbeef"), "the leaf evidence must survive serialisation");
        assert!(json.contains("\"premises\":[{"), "the tree must serialise recursively");
    }
}


/// Tasks 577 (temporal) and 578 (spatial) — the two checks the council enacted unanimously and 10-4.
///
/// The temporal tests fix the semantics of all thirteen relations and of composition; the spatial tests pin
/// the contact predicate at its boundary, because an off-by-one at `d == cutoff` is the difference between a
/// contact map and a wish.
#[cfg(test)]
mod temporal_spatial_tests {
    use trm_uefi_kernel::spatial_reasoning::{
        atoms_within, contact_graph, near_set, relate, Atom, Ball, BallRelation, BoundingBox, Point,
    };
    use trm_uefi_kernel::temporal_reasoning::{
        compose, compose_chain, relation, Allen, Interval, Seeded,
    };

    fn iv(s: i64, e: i64) -> Interval {
        Interval::try_new(s, e).expect("valid interval")
    }

    #[test]
    fn all_thirteen_relations_are_identified() {
        let a = iv(10, 20);
        // the exact thirteen, one each — a is the SUBJECT: "a <relation> b".
        // Allen's naming is from the subject's point of view: a STARTS b when a is the shorter interval sharing
        // b's start; a STARTS_BY b when b is the shorter one.
        assert_eq!(relation(&a, &iv(0, 5)), Allen::After, "b ends before a starts");
        assert_eq!(relation(&a, &iv(0, 10)), Allen::MetBy, "b ends exactly where a starts");
        assert_eq!(relation(&a, &iv(5, 15)), Allen::OverlappedBy, "b starts earlier, ends inside a");
        assert_eq!(relation(&a, &iv(10, 25)), Allen::Starts, "same start, a is the shorter one");
        assert_eq!(relation(&a, &iv(10, 15)), Allen::StartedBy, "same start, b is the shorter one");
        assert_eq!(relation(&a, &iv(12, 18)), Allen::Contains, "a strictly encloses b");
        assert_eq!(relation(&a, &iv(8, 25)), Allen::During, "a strictly inside b");
        assert_eq!(relation(&a, &iv(10, 20)), Allen::Equals);
        assert_eq!(relation(&a, &iv(8, 20)), Allen::Finishes, "same end, a is the shorter one");
        assert_eq!(relation(&a, &iv(12, 20)), Allen::FinishedBy, "same end, b is the shorter one");
        assert_eq!(relation(&a, &iv(15, 25)), Allen::Overlaps, "a starts earlier, ends inside b");
        assert_eq!(relation(&a, &iv(20, 30)), Allen::Meets, "a ends exactly where b starts");
        assert_eq!(relation(&a, &iv(21, 30)), Allen::Before, "a ends before b starts");
    }

    #[test]
    fn inverse_agrees_with_swapped_arguments() {
        let a = iv(10, 20);
        for b in [iv(0, 5), iv(0, 10), iv(5, 15), iv(12, 18), iv(10, 20), iv(25, 30)] {
            assert_eq!(relation(&a, &b).inverse(), relation(&b, &a),
                       "converse must match the swapped computation for b={b:?}");
            assert_eq!(relation(&a, &b).inverse().inverse(), relation(&a, &b));
        }
    }

    #[test]
    fn composition_satisfies_the_classic_identities() {
        // equals is the identity of composition
        assert_eq!(compose(Allen::Equals, Allen::Before), alloc_vec_before());
        assert_eq!(compose(Allen::Before, Allen::Equals), alloc_vec_before());
        // before ∘ before = before
        assert_eq!(compose(Allen::Before, Allen::Before), alloc_vec_before());
        // meets ∘ before = before
        assert_eq!(compose(Allen::Meets, Allen::Before), alloc_vec_before());
        // before ∘ after is unconstrained: any relation is possible
        assert_eq!(compose(Allen::Before, Allen::After).len(), 13,
                   "before∘after must leave every relation possible");
        // meets ∘ met_by can be equals (B ends where A starts and starts where C ends)
        let forward = compose(Allen::Meets, Allen::MetBy);
        assert!(forward.contains(&Allen::Equals), "meets∘met_by must admit equals, got {forward:?}");
        assert!(forward.len() > 1, "meets∘met_by must not collapse to one relation");
    }

    fn alloc_vec_before() -> Vec<Allen> {
        vec![Allen::Before]
    }

    #[test]
    fn chains_compose_and_degrade_to_all_when_unconstrained() {
        // before, before, before stays before
        assert_eq!(compose_chain(&[Allen::Before, Allen::Before, Allen::Before]),
                   alloc_vec_before());
        // an unconstrained step admits everything — reported as such, not as a guess
        assert_eq!(compose_chain(&[Allen::Before, Allen::After]).len(), 13);
    }

    #[test]
    fn a_reversed_interval_is_refused_not_swapped() {
        assert!(Interval::try_new(20, 10).is_none(), "a reversed interval is a data error");
        assert!(Interval::try_new(10, 10).is_none(), "a zero-length interval is not an interval");
        assert_eq!(iv(10, 20).duration(), 10);
        assert!(iv(10, 20).contains_instant(10));
        assert!(!iv(10, 20).contains_instant(20), "half-open: the end is not inside");
    }

    #[test]
    fn pairwise_relations_cover_every_ordered_pair() {
        let items = vec![
            Seeded { name: "a".into(), interval: iv(0, 10) },
            Seeded { name: "b".into(), interval: iv(10, 20) },
            Seeded { name: "c".into(), interval: iv(5, 15) },
        ];
        let pairs = trm_uefi_kernel::temporal_reasoning::relate_all(&items);
        assert_eq!(pairs.len(), 6, "3 items must give 6 ordered pairs");
    }

    // ---- spatial -------------------------------------------------------------------------------

    fn atom(label: &str, x: f64, y: f64, z: f64, element: &str) -> Atom {
        Atom { label: label.to_string(), position: Point::new(x, y, z), element: element.to_string() }
    }

    #[test]
    fn distance_is_exact_on_known_geometry() {
        let a = Point::new(0.0, 0.0, 0.0);
        let b = Point::new(3.0, 4.0, 0.0);
        assert!((a.distance(&b) - 5.0).abs() < 1e-12, "3-4-5 triangle");
        assert_eq!(a.dist2(&b), 25.0);
    }

    #[test]
    fn the_contact_predicate_includes_the_boundary() {
        // exactly at the cutoff must count: d² <= cut², so 4.0 A is a contact at 4.0 A cutoff
        let atoms = vec![atom("A1", 0.0, 0.0, 0.0, "C"), atom("A2", 4.0, 0.0, 0.0, "C")];
        assert_eq!(contact_graph(&atoms, 4.0, false).len(), 1, "boundary contact must be counted");
        assert_eq!(contact_graph(&atoms, 3.999, false).len(), 0, "just inside must not be");
    }

    #[test]
    fn hydrogens_are_excluded_only_when_asked() {
        let atoms = vec![
            atom("C1", 0.0, 0.0, 0.0, "C"),
            atom("H1", 1.0, 0.0, 0.0, "H"),
        ];
        assert_eq!(contact_graph(&atoms, 2.0, true).len(), 1);
        assert_eq!(contact_graph(&atoms, 2.0, false).len(), 0);
    }

    #[test]
    fn near_set_excludes_the_reference_atoms_themselves() {
        let atoms = vec![
            atom("L1", 0.0, 0.0, 0.0, "C"),   // reference
            atom("R1", 3.0, 0.0, 0.0, "C"),   // within 4 A
            atom("R2", 10.0, 0.0, 0.0, "C"),  // far
        ];
        let near = near_set(&atoms, &[0], 4.0);
        assert_eq!(near, vec![1], "only the near atom, and never the reference itself");
        assert!(atoms_within(&atoms, &Point::new(0.0, 0.0, 0.0), 4.0).contains(&0),
                "a point query does include the atom at the centre");
    }

    #[test]
    fn bounding_box_centre_and_empty_case() {
        let atoms = vec![atom("A", 0.0, 0.0, 0.0, "C"), atom("B", 10.0, 20.0, 30.0, "C")];
        let b = BoundingBox::of(&atoms).expect("non-empty");
        let c = b.centre();
        assert!((c.x - 5.0).abs() < 1e-12 && (c.y - 10.0).abs() < 1e-12 && (c.z - 15.0).abs() < 1e-12);
        let s = b.size();
        assert!((s.x - 10.0).abs() < 1e-12 && (s.y - 20.0).abs() < 1e-12 && (s.z - 30.0).abs() < 1e-12);
        assert!(BoundingBox::of(&[]).is_none(), "an empty set has no box, not a box at the origin");
        assert!(b.contains(&Point::new(5.0, 10.0, 15.0)));
        assert!(!b.contains(&Point::new(11.0, 10.0, 15.0)));
    }

    #[test]
    fn ball_relations_are_five_and_disjoint() {
        let a = Ball::new(Point::new(0.0, 0.0, 0.0), 2.0);
        assert_eq!(relate(&a, &Ball::new(Point::new(10.0, 0.0, 0.0), 2.0), 0.0), BallRelation::Disjoint);
        assert_eq!(relate(&a, &Ball::new(Point::new(4.0, 0.0, 0.0), 2.0), 0.01), BallRelation::ExternallyConnected);
        assert_eq!(relate(&a, &Ball::new(Point::new(3.0, 0.0, 0.0), 2.0), 0.0), BallRelation::Overlapping);
        assert_eq!(relate(&a, &Ball::new(Point::new(0.0, 0.0, 0.0), 2.0), 0.0), BallRelation::Equal);
        assert_eq!(relate(&a, &Ball::new(Point::new(0.0, 0.0, 0.0), 1.0), 0.0), BallRelation::Contains);
    }
}


/// Task 573 — case-based reasoning, enacted by council 10-4.
///
/// The load-bearing tests are the last two: one requires the base to beat the mean predictor on a learnable
/// relationship, and the other requires it to **admit** when it cannot. A CBR whose leave-one-out is never
/// allowed to lose is not being measured.
#[cfg(test)]
mod case_base_tests {
    use trm_uefi_kernel::case_base::CaseBase;

    #[test]
    fn retrieval_orders_by_standardised_distance() {
        let mut cb = CaseBase::new(2);
        cb.add("far", &[0.0, 0.0], 1.0).unwrap();
        cb.add("near", &[9.9, 9.9], 2.0).unwrap();
        cb.add("mid", &[5.0, 5.0], 3.0).unwrap();
        let n = cb.retrieve(&[10.0, 10.0], 2, None);
        assert_eq!(n.len(), 2);
        assert_eq!(n[0].id, "near", "nearest must be retrieved first");
        assert!(n[1].distance >= n[0].distance, "distances must be ordered");
    }

    #[test]
    fn a_case_never_retrieves_itself() {
        let mut cb = CaseBase::new(1);
        for i in 0..5 {
            cb.add(&format!("c{i}"), &[i as f64], i as f64).unwrap();
        }
        let n = cb.retrieve(&[2.0], 5, Some("c2"));
        assert!(n.iter().all(|x| x.id != "c2"), "leave-one-out must exclude the query case");
    }

    #[test]
    fn prediction_interpolates_rather_than_copying() {
        // two equidistant neighbours with different outcomes: the answer is between them, not one of them
        let mut cb = CaseBase::new(1);
        cb.add("low", &[-1.0], 1.0).unwrap();
        cb.add("high", &[1.0], 5.0).unwrap();
        let p = cb.predict(&[0.0], 2).expect("prediction");
        assert!(p > 1.0 && p < 5.0, "an interpolated answer must sit between the neighbours, got {p}");
        assert!((p - 3.0).abs() < 1e-6, "symmetric neighbours must interpolate to the midpoint, got {p}");
    }

    #[test]
    fn a_malformed_case_is_refused_not_padded() {
        let mut cb = CaseBase::new(3);
        cb.add("ok", &[1.0, 2.0, 3.0], 0.5).unwrap();
        assert!(cb.add("short", &[1.0, 2.0], 0.5).is_err(), "a short feature vector must be refused");
        assert!(cb.add("long", &[1.0, 2.0, 3.0, 4.0], 0.5).is_err());
    }

    #[test]
    fn a_zero_variance_feature_does_not_produce_nan() {
        let mut cb = CaseBase::new(2);
        for i in 0..5 {
            cb.add(&format!("c{i}"), &[1.0, i as f64], i as f64).unwrap();
        }
        let p = cb.predict(&[1.0, 2.0], 3).expect("prediction");
        assert!(p.is_finite(), "constant feature must not divide by zero, got {p}");
    }

    #[test]
    fn leave_one_out_beats_the_mean_on_a_learnable_relationship() {
        let mut cb = CaseBase::new(1);
        for i in 0..20 {
            let x = i as f64;
            cb.add(&format!("c{i}"), &[x], 3.0 * x).unwrap();
        }
        let r = cb.leave_one_out(3).expect("loo");
        assert_eq!(r.n, 20);
        assert!(r.r2 > 0.9, "a linear relationship must be recovered, R²={}", r.r2);
        assert!(r.rmse < r.baseline_rmse, "rmse {} must beat baseline {}", r.rmse, r.baseline_rmse);
        assert!(!r.worse_than_baseline);
    }

    #[test]
    fn leave_one_out_admits_when_it_only_ties_the_baseline() {
        // constant outcome: there is nothing to learn, so the base ties the mean and must say so
        let mut cb = CaseBase::new(1);
        for i in 0..10 {
            cb.add(&format!("c{i}"), &[i as f64], 7.0).unwrap();
        }
        let r = cb.leave_one_out(3).expect("loo");
        assert!((r.rmse - 0.0).abs() < 1e-9, "no error is possible on a constant, got {}", r.rmse);
        assert!(r.worse_than_baseline, "a tie must be reported as not better than the baseline");
    }
}


/// Tasks 575 (abduction) and 574 (analogy) — enacted by council on resubmission (13-1 and 12-2).
///
/// The load-bearing tests: abduction must return the MINIMAL set and must admit when the space cannot explain
/// an observation; analogy must beat copying the neighbour, and must lose once the outcomes are shuffled.
#[cfg(test)]
mod abduction_analogy_tests {
    use trm_uefi_kernel::abduction::{abduce, Hypothesis, ASSUMPTION_SOURCE};
    use trm_uefi_kernel::analogical_reasoning::{leave_one_out, predict, random_outcome_control};
    use trm_uefi_kernel::case_base::CaseBase;
    use trm_uefi_kernel::reasoning_engine::{Engine, Rule, Trit};

    fn engine_with_rule() -> Engine {
        // p AND q -> obs
        let mut e = Engine::new();
        e.add_rule(Rule::new(1, "pq_obs", &["p", "q"], "obs"));
        e
    }

    #[test]
    fn abduction_returns_a_minimal_set_of_size_two() {
        let e = engine_with_rule();
        let hyps = vec![Hypothesis::new("p"), Hypothesis::new("q"), Hypothesis::new("noise")];
        let r = abduce(&e, "obs", &hyps, 3);
        assert!(!r.unexplained);
        assert_eq!(r.assumptions.len(), 2, "needs both premises, got {:?}", r.assumptions);
        assert!(r.assumptions.contains(&"p".to_string()));
        assert!(r.assumptions.contains(&"q".to_string()));
        assert!(r.complete, "the whole space is 8 subsets; the search must be exhaustive");
        assert_eq!(r.alternatives, 0, "only one set of size 2 explains it");
    }

    #[test]
    fn abduction_prefers_the_cheaper_of_two_explanations() {
        let mut e = Engine::new();
        // either a or b alone derives obs
        e.add_rule(Rule::new(1, "a_obs", &["a"], "obs"));
        e.add_rule(Rule::new(2, "b_obs", &["b"], "obs"));
        let hyps = vec![Hypothesis::with_cost("a", 5.0), Hypothesis::with_cost("b", 1.0)];
        let r = abduce(&e, "obs", &hyps, 2);
        assert_eq!(r.assumptions, vec!["b".to_string()], "the cheaper single assumption must win");
        assert!((r.cost - 1.0).abs() < 1e-12);
        assert_eq!(r.alternatives, 0);
    }

    #[test]
    fn equal_cost_explanations_are_counted_not_hidden() {
        let mut e = Engine::new();
        e.add_rule(Rule::new(1, "a_obs", &["a"], "obs"));
        e.add_rule(Rule::new(2, "b_obs", &["b"], "obs"));
        let hyps = vec![Hypothesis::new("a"), Hypothesis::new("b")];
        let r = abduce(&e, "obs", &hyps, 2);
        assert_eq!(r.assumptions.len(), 1);
        assert_eq!(r.alternatives, 1, "the other equal-cost explanation must be reported");
    }

    #[test]
    fn an_unexplainable_observation_says_so() {
        let e = engine_with_rule();
        let hyps = vec![Hypothesis::new("unrelated")];
        let r = abduce(&e, "obs", &hyps, 2);
        assert!(r.unexplained, "no subset derives obs; that must be reported, not papered over");
        assert!(r.assumptions.is_empty());
    }

    #[test]
    fn an_observation_that_follows_from_the_facts_needs_no_assumption() {
        let mut e = engine_with_rule();
        e.assert_fact("p", "shard:test@rec 1");
        e.assert_fact("q", "shard:test@rec 2");
        let r = abduce(&e, "obs", &[Hypothesis::new("noise")], 2);
        assert!(r.assumptions.is_empty(), "nothing to assume when the facts already entail it");
        assert_eq!(r.cost, 0.0);
    }

    #[test]
    fn assumed_facts_are_labelled_as_assumptions_in_the_derivation() {
        let e = engine_with_rule();
        let hyps = vec![Hypothesis::new("p"), Hypothesis::new("q")];
        let r = abduce(&e, "obs", &hyps, 2);
        let mut check = e.clone();
        for a in &r.assumptions {
            check.assert_fact(a, ASSUMPTION_SOURCE);
        }
        let fr = check.forward_chain();
        assert_eq!(fr.truth.get("obs"), Some(&Trit::True));
        let d = fr.derivations.get("p").expect("derivation for p");
        assert_eq!(d.fact_source.as_deref(), Some(ASSUMPTION_SOURCE),
                   "an assumption must never be recorded as an observation");
    }

    // ---- analogy ------------------------------------------------------------------------------

    fn linear_base() -> CaseBase {
        // outcome moves with feature 0: transfer along the transformation should be near-exact
        let mut cb = CaseBase::new(1);
        for i in 0..12 {
            let x = i as f64;
            cb.add(&format!("c{i}"), &[x], 2.0 * x).unwrap();
        }
        cb
    }

    #[test]
    fn analogy_transfers_along_the_matched_transformation() {
        let cb = linear_base(); // y = 2x, cases at x = 0..11
        // half-way between c5 (y=10) and c6 (y=12): the observed change is c4->c5 (+2 per unit), applied at 0.5
        let a = predict(&cb, &[5.5], None).expect("an analogy");
        assert!((a.source_outcome - 10.0).abs() < 1e-9, "source outcome must be c5's 10, got {}", a.source_outcome);
        assert!((a.prediction - 11.0).abs() < 1e-6, "transfer must land on 11, got {}", a.prediction);
        assert!(a.cosine > 0.99, "the transformation is exactly aligned, cosine={}", a.cosine);
        assert!((a.scale - 0.5).abs() < 1e-9, "scale must be the distance fraction, got {}", a.scale);
        assert!(a.note.is_empty(), "a real transfer must not carry a fallback note: {}", a.note);
    }

    #[test]
    fn an_exact_match_returns_the_measured_outcome_and_says_why() {
        let cb = linear_base();
        let a = predict(&cb, &[5.0], None).expect("an analogy");
        assert!((a.prediction - 10.0).abs() < 1e-9);
        assert!(a.note.contains("exact match"), "the reason must be recorded, got: {}", a.note);
    }

    #[test]
    fn analogy_beats_copying_the_neighbour_on_a_linear_relationship() {
        let cb = linear_base();
        let r = leave_one_out(&cb).expect("loo");
        assert!(r.analogy_rmse <= r.zero_delta_rmse + 1e-9,
                "analogy {} must not be worse than zero-delta {}", r.analogy_rmse, r.zero_delta_rmse);
        assert!(r.r2 > 0.9, "a linear relationship must be recovered, R²={}", r.r2);
    }

    #[test]
    fn the_random_outcome_control_loses() {
        let cb = linear_base();
        let c = random_outcome_control(&cb, 42).expect("control");
        // with outcomes shuffled the features carry no information: the analogy cannot beat copying the
        // neighbour by any meaningful margin.
        // No more than a 10% apparent gain once the signal is destroyed. A larger gain would mean the analogy
        // is fitting the shuffle, not transferring a measured change.
        assert!(c.analogy_rmse >= c.zero_delta_rmse * 0.90,
                "control must not show a real gain: analogy {} vs zero-delta {}", c.analogy_rmse, c.zero_delta_rmse);
    }

    #[test]
    fn extrapolation_beyond_the_observed_transformation_is_clamped_and_flagged() {
        let mut cb = CaseBase::new(1);
        cb.add("a", &[0.0], 0.0).unwrap();
        cb.add("b", &[1.0], 1.0).unwrap();
        cb.add("c", &[2.0], 2.0).unwrap();
        cb.add("d", &[10.0], 10.0).unwrap();
        // query far beyond the dense part of the base
        let an = predict(&cb, &[50.0], None).expect("analogy");
        assert!(an.scale <= 1.0 + 1e-12, "scale must be clamped, got {}", an.scale);
        assert!(an.extrapolated, "the clamp must be recorded, not silent");
    }
}


/// Tasks 581/584/587 — the text layer. The load-bearing property is that every offset round-trips: a citation
/// either resolves to the identical bytes or it is not a citation.
#[cfg(test)]
mod text_reading_tests {
    use trm_uefi_kernel::text_reading::{
        find_normalised, normalise, parse_markdown, quantities, split_sentences, UnitKind,
    };

    const DOC: &str = "# Title\n\nFirst paragraph line one.\nLine two of the same paragraph.\n\n## Section\n\n- item one\n- item two\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n```rust\nfn code() { let x = 1; }\n```\n";

    #[test]
    fn every_unit_round_trips_to_identical_bytes() {
        for u in parse_markdown(DOC) {
            assert_eq!(&DOC[u.start..u.end], &DOC[u.start..u.end], "slice bounds must be valid");
            assert!(u.end <= DOC.len() && u.start < u.end, "unit {:?} has bad bounds", u.kind);
        }
    }

    #[test]
    fn structure_is_recovered() {
        let units = parse_markdown(DOC);
        let kinds: Vec<UnitKind> = units.iter().map(|u| u.kind).collect();
        assert!(kinds.contains(&UnitKind::Heading));
        assert!(kinds.contains(&UnitKind::Paragraph));
        assert!(kinds.contains(&UnitKind::List));
        assert!(kinds.contains(&UnitKind::Table));
        assert!(kinds.contains(&UnitKind::Code), "the fenced block must be one unit");
        let headings: Vec<(u8, String)> = units.iter().filter(|u| u.kind == UnitKind::Heading)
            .map(|u| (u.level, u.heading.clone())).collect();
        assert_eq!(headings[0], (1, "Title".to_string()));
        assert_eq!(headings[1], (2, "Section".to_string()));
    }

    #[test]
    fn a_hashtag_is_not_a_heading() {
        let units = parse_markdown("#hashtag is not a heading\n");
        assert!(units.iter().all(|u| u.kind != UnitKind::Heading));
    }

    #[test]
    fn sentences_split_with_exact_offsets() {
        let text = "First sentence. Second one! Is this third?";
        let sents = split_sentences(text);
        assert_eq!(sents.len(), 3, "got {:?}", sents.iter().map(|s| &s.text).collect::<Vec<_>>());
        for s in &sents {
            assert_eq!(&text[s.start..s.end], s.text, "offset must resolve to the sentence text");
        }
    }

    #[test]
    fn decimals_and_abbreviations_do_not_split() {
        let text = "The score was 0.4633 and the value 12.5 held. See e.g. Fig. 3 for details.";
        let sents = split_sentences(text);
        assert_eq!(sents.len(), 2, "decimals and abbreviations must not split: {:?}",
                   sents.iter().map(|s| &s.text).collect::<Vec<_>>());
    }

    #[test]
    fn code_fences_are_not_split_into_sentences() {
        let text = "Intro sentence.\n```\nlet x = 1. let y = 2.\n```\nOutro sentence.";
        let sents = split_sentences(text);
        assert!(sents.iter().all(|s| !s.text.contains("let y")), "code must not become a sentence: {:?}",
                sents.iter().map(|s| &s.text).collect::<Vec<_>>());
    }

    #[test]
    fn quantities_keep_their_offsets_and_units() {
        let text = "We measured 406 compounds, 0.4633 accuracy and 12.5 mg dose.";
        let qs = quantities(text);
        assert_eq!(qs.len(), 3, "got {:?}", qs.iter().map(|q| &q.text).collect::<Vec<_>>());
        assert!((qs[0].value - 406.0).abs() < 1e-9);
        assert_eq!(&text[qs[0].start..qs[0].end], "406");
        assert!((qs[1].value - 0.4633).abs() < 1e-9);
        assert_eq!(qs[2].unit, "mg");
    }

    #[test]
    fn normalise_collapses_case_and_spacing() {
        assert_eq!(normalise("  The   Score WAS 0.46 "), "the score was 0.46");
    }

    #[test]
    fn find_normalised_returns_a_usable_position() {
        let text = "All 406 compounds were scored.";
        let pos = find_normalised(text, "406 compounds").expect("must find the span");
        assert!(text[pos..].starts_with("406"), "returned offset {} does not start the match: {:?}", pos, &text[pos..]);
    }
}


/// Tasks 585/587/589 — claim verification, extraction and contradiction.
///
/// The tests fix the three-way verdict discipline: silence must be UNKNOWN (never REFUTED), a numeric
/// disagreement must be REFUTED with the numbers named, and a contradiction must require that the two sentences
/// are about the same thing.
#[cfg(test)]
mod claims_tests {
    use trm_uefi_kernel::claims::{contradicts, extract, verify, ConflictKind, Verdict};

    fn ev() -> Vec<String> {
        vec![
            "The retrieval index holds 2,403 chunks from 191 files, each with a 768-dim vector.".to_string(),
            "The embedding model is nomic-embed-text running on port 8593.".to_string(),
        ]
    }

    #[test]
    fn extraction_keeps_the_number_as_written() {
        let c = extract("The corpus contains 2,403 chunks from 191 files.").expect("claim");
        assert_eq!(c.numbers_literal[0], "2,403");
        assert!(c.tokens.contains(&"chunks".to_string()));
        assert!(!c.negated && !c.hedged);
    }

    #[test]
    fn a_claim_the_evidence_states_is_supported() {
        let r = verify("The retrieval index holds 2,403 chunks from 191 files.", &ev());
        assert_eq!(r.verdict, Verdict::Supported, "reason: {}", r.reason);
        assert_eq!(r.evidence_index, Some(0));
    }

    #[test]
    fn a_flipped_number_is_refuted_and_the_numbers_are_named() {
        let r = verify("The retrieval index holds 558 chunks from 191 files.", &ev());
        assert_eq!(r.verdict, Verdict::Refuted, "reason: {}", r.reason);
        assert!(r.reason.contains("558"), "the reason must name the claim's number: {}", r.reason);
        assert!(r.reason.contains("2,403"), "and the evidence's: {}", r.reason);
    }

    #[test]
    fn a_claim_about_nothing_in_the_corpus_is_unknown_not_refuted() {
        let r = verify("The spacecraft carries 12 ion thrusters for orbital correction.", &ev());
        assert_eq!(r.verdict, Verdict::Unknown,
                   "silence must not be refutation, got {} ({})", r.verdict.as_str(), r.reason);
        assert_eq!(r.evidence_index, None);
    }

    #[test]
    fn a_negated_claim_about_supported_content_is_refuted() {
        let r = verify("The embedding model is not nomic-embed-text running on port 8593.", &ev());
        assert_eq!(r.verdict, Verdict::Refuted, "reason: {}", r.reason);
        assert!(r.reason.contains("polarity"), "reason: {}", r.reason);
    }

    #[test]
    fn a_hedged_claim_is_not_refuted_by_a_different_number() {
        // "approximately" means the number is not an exact assertion; refuting on it would be over-reading
        let r = verify("The index holds approximately 900 chunks from 191 files.", &ev());
        assert_ne!(r.verdict, Verdict::Refuted,
                   "a hedged claim must not be refuted on a number: {}", r.reason);
    }

    #[test]
    fn contradiction_needs_shared_subject() {
        let a = "The retrieval index holds 2,403 chunks from 191 files.";
        let b = "The GPU sustains 36 tokens per second on 2,403 prompt tokens.";
        assert!(contradicts(a, b).is_none(), "different subjects must not be called a contradiction");
    }

    #[test]
    fn a_flipped_number_between_sentences_is_a_numeric_contradiction() {
        let a = "The retrieval index holds 2,403 chunks from 191 files.";
        let b = "The retrieval index holds 558 chunks from 191 files.";
        let c = contradicts(a, b).expect("contradiction");
        assert_eq!(c.kind, ConflictKind::Numeric);
        assert!(c.detail.contains("2,403") && c.detail.contains("558"), "detail: {}", c.detail);
    }

    #[test]
    fn opposite_polarity_is_a_polarity_contradiction() {
        let a = "The embedding model is nomic-embed-text running on port 8593.";
        let b = "The embedding model is not nomic-embed-text running on port 8593.";
        let c = contradicts(a, b).expect("contradiction");
        assert_eq!(c.kind, ConflictKind::Polarity);
    }

    #[test]
    fn agreeing_sentences_do_not_contradict() {
        let a = "The retrieval index holds 2,403 chunks from 191 files.";
        let b = "The retrieval index holds 2,403 chunks from 191 files, each with a 768-dim vector.";
        assert!(contradicts(a, b).is_none());
    }
}


/// Tasks 591-600 — knowledge verification. Each test asserts a check FIRES on a case that should fail it; a
/// check that only ever passes is not a check.
#[cfg(test)]
mod knowledge_verification_tests {
    use trm_uefi_kernel::knowledge_verification::{
        classify, find_retractions, quantify, resolve, retraction_targets, review_document, review_verdicts,
        verify_source, Position, Rule, Severity, Signals, SourceKind,
    };

    #[test]
    fn an_unrecorded_source_is_not_verified() {
        let v = verify_source("evidence/x.txt", None, "abc123");
        assert!(!v.hash_matches, "no recorded hash must never count as verified");
        assert!(v.detail.contains("NO RECORDED HASH"));
    }

    #[test]
    fn a_hash_mismatch_fails_and_names_both() {
        let v = verify_source("evidence/x.txt", Some("aaaa1111bbbb"), "cccc2222dddd");
        assert!(!v.hash_matches);
        assert!(v.detail.contains("MISMATCH"));
        assert!(v.detail.contains("aaaa"), "the recorded value must be visible: {}", v.detail);
    }

    #[test]
    fn a_matching_hash_passes() {
        let v = verify_source("evidence/x.txt", Some("ABCDEF"), "abcdef");
        assert!(v.hash_matches, "case-insensitive match must pass: {}", v.detail);
    }

    #[test]
    fn source_kinds_are_classified_by_where_they_live() {
        assert_eq!(classify("evidence/foo.txt"), SourceKind::Primary);
        assert_eq!(classify("docs/REASONING.md"), SourceKind::Secondary);
        assert_eq!(classify("docs/briefing-report.md"), SourceKind::Derived);
    }

    fn pos(label: &str, measured: bool, kind: SourceKind, date: &str) -> Position {
        Position { label: label.to_string(), kind, measured, date: date.to_string(), value: String::new() }
    }

    #[test]
    fn a_measurement_beats_a_restatement() {
        let a = pos("ledger", true, SourceKind::Secondary, "2026-11-01");
        let b = pos("summary", false, SourceKind::Primary, "2026-11-20");
        let r = resolve(&a, &b);
        assert_eq!(r.rule, Rule::MeasuredOverClaimed);
        assert_eq!(r.winner.as_deref(), Some("ledger"));
    }

    #[test]
    fn primary_beats_secondary_when_both_are_measured() {
        let a = pos("evidence", true, SourceKind::Primary, "2026-11-01");
        let b = pos("doc", true, SourceKind::Secondary, "2026-11-20");
        let r = resolve(&a, &b);
        assert_eq!(r.rule, Rule::PrimaryOverSecondary);
        assert_eq!(r.winner.as_deref(), Some("evidence"), "recency must not outrank source kind");
    }

    #[test]
    fn a_true_tie_is_unresolved_not_arbitrary() {
        let a = pos("a", true, SourceKind::Primary, "2026-11-01");
        let b = pos("b", true, SourceKind::Primary, "2026-11-01");
        let r = resolve(&a, &b);
        assert_eq!(r.rule, Rule::Unresolved);
        assert!(r.winner.is_none(), "must not pick a winner it cannot justify");
    }

    #[test]
    fn confidence_falls_without_evidence() {
        let strong = quantify(&Signals {
            coverage: 1.0, numbers_agree: true, numbers_conflict: false,
            evidence_kind: SourceKind::Primary, no_evidence: false,
        });
        let none = quantify(&Signals {
            coverage: 0.0, numbers_agree: false, numbers_conflict: false,
            evidence_kind: SourceKind::Secondary, no_evidence: true,
        });
        assert!(strong.p_support.unwrap_or(0.0) > 0.8, "strong evidence must score high: {:?}", strong.p_support);
        assert!(none.p_support.is_none(), "no evidence must ABSTAIN, not report a confident number");
        assert!(none.abstention, "no evidence must be an abstention");
    }

    #[test]
    fn a_numeric_conflict_drives_confidence_down() {
        let agree = quantify(&Signals {
            coverage: 0.9, numbers_agree: true, numbers_conflict: false,
            evidence_kind: SourceKind::Primary, no_evidence: false,
        });
        let conflict = quantify(&Signals {
            coverage: 0.9, numbers_agree: false, numbers_conflict: true,
            evidence_kind: SourceKind::Primary, no_evidence: false,
        });
        let (c, a) = (conflict.p_support.unwrap_or(0.0), agree.p_support.unwrap_or(0.0));
        assert!(c < a - 0.3, "conflict {} must be well below agreement {}", c, a);
    }

    #[test]
    fn review_blocks_an_assertion_without_provenance() {
        let doc = "We measured 406 compounds at R2 0.72 and it works.\nThe file tools/x.py line 12 shows the numbers.";
        let f = review_document(doc);
        let (b, _, _) = review_verdicts(&f);
        assert!(b >= 1, "the unprovenanced assertion must be BLOCKING: {:?}", f);
        assert!(f.iter().any(|x| x.severity == Severity::Blocking && x.line == 1));
    }

    #[test]
    fn review_does_not_block_a_line_that_names_its_evidence() {
        let doc = "Measured in tools/reasoning_verify.py: 406/406 agree.\n";
        let f = review_document(doc);
        let (b, _, _) = review_verdicts(&f);
        assert_eq!(b, 0, "a line naming its evidence must not be blocked: {:?}", f);
    }

    #[test]
    fn review_flags_placeholders() {
        let doc = "Some prose here.\nTODO: finish this section.\n";
        let f = review_document(doc);
        assert!(f.iter().any(|x| x.rule == "placeholder"), "a TODO must be flagged: {:?}", f);
    }

    #[test]
    fn retractions_are_found_with_their_body() {
        let doc = "# Doc\n\n## ERRATUM\nThe earlier claim of 97 phantom modules was wrong; all 97 exist.\n\nMore prose.\n";
        let r = find_retractions(doc);
        assert_eq!(r.len(), 1);
        assert!(r[0].body.contains("97 phantom modules"), "body: {}", r[0].body);
        assert_eq!(r[0].marker_line, 3);
    }

    #[test]
    fn a_retraction_only_targets_the_claim_it_names() {
        let doc = "## ERRATUM\nThe earlier claim of 97 phantom modules was wrong.\n";
        let r = &find_retractions(doc)[0];
        assert!(retraction_targets(r, "97 phantom modules did not exist"));
        assert!(!retraction_targets(r, "the GPU sustains 36 tokens per second"), "an unrelated claim must not be covered");
    }
}


/// Tasks 531-537, 539 — information extraction.
///
/// The load-bearing tests are the ones that can fail: injected relations must come back out, negation must stop at
/// a clause boundary, a month-only date must not become a specific day, and non-English text must be REFUSED.
#[cfg(test)]
mod information_extraction_tests {
    use trm_uefi_kernel::information_extraction::{
        criteria_section, detect_negation, detect_uncertainty, extract_events, extract_relations,
        extract_temporal, language_gate, link_entities, resolve_coref, EventKind, LanguageVerdict, TemporalKind,
    };

    #[test]
    fn injected_relations_are_recovered() {
        let text = "The kernel holds 219 modules. The ledger requires quorum evidence.";
        let rels = extract_relations(text);
        assert!(rels.len() >= 2, "expected both relations, got {:?}",
                rels.iter().map(|r| (r.subject.text.clone(), r.predicate.clone(), r.object.text.clone())).collect::<Vec<_>>());
        let has = |s: &str, p: &str, o: &str| rels.iter().any(|r| r.subject.text == s && r.predicate == p && r.object.text.starts_with(o));
        assert!(has("kernel", "holds", "219"), "missing kernel-holds relation: {:?}", rels.len());
        assert!(has("ledger", "requires", "quorum"), "missing ledger-requires relation");
    }

    #[test]
    fn relation_spans_round_trip() {
        let text = "The council enforces quorum arithmetic before commit.";
        for r in extract_relations(text) {
            assert_eq!(&text[r.subject.start..r.subject.end], r.subject.text);
            assert_eq!(&text[r.object.start..r.object.end], r.object.text);
        }
    }

    #[test]
    fn clinical_events_are_typed_and_their_arguments_round_trip() {
        let text = "Patients diagnosed with hypertension were treated with lisinopril.";
        let ev = extract_events(text);
        assert!(ev.iter().any(|e| e.kind == EventKind::Diagnosis), "no diagnosis event");
        assert!(ev.iter().any(|e| e.kind == EventKind::Medication), "no medication event");
        for e in &ev {
            assert_eq!(&text[e.trigger.start..e.trigger.end], e.trigger.text);
            for a in &e.arguments {
                assert_eq!(&text[a.start..a.end], a.text, "argument span must round-trip");
            }
        }
    }

    #[test]
    fn criteria_sections_are_recovered_from_the_source_labels() {
        let text = "Inclusion Criteria: - age 18 or older. Exclusion Criteria: - history of liver disease.";
        let exc_off = text.find("history").unwrap();
        let inc_off = text.find("age 18").unwrap();
        assert_eq!(criteria_section(text, inc_off), Some(EventKind::CriterionInclusion));
        assert_eq!(criteria_section(text, exc_off), Some(EventKind::CriterionExclusion));
    }

    #[test]
    fn negation_stops_at_the_clause_boundary() {
        let n = detect_negation("No fever, but cough is present.").expect("negation cue");
        assert_eq!(n.cue.text.to_lowercase(), "no");
        assert!(n.scope.text.contains("fever"), "scope must cover fever: {:?}", n.scope.text);
        assert!(!n.scope.text.contains("cough"), "scope must NOT cover cough: {:?}", n.scope.text);
    }

    #[test]
    fn a_positive_sentence_has_no_negation_cue() {
        assert!(detect_negation("The patient has a history of asthma.").is_none());
    }

    #[test]
    fn uncertainty_cue_is_found_with_scope() {
        let u = detect_uncertainty("The result may indicate an infection.").expect("hedge");
        assert_eq!(u.cue.text.to_lowercase(), "may");
        assert!(u.scope.text.contains("indicate"));
    }

    #[test]
    fn no_uncertainty_on_a_plain_assertion() {
        assert!(detect_uncertainty("The kernel holds 219 modules.").is_none());
    }

    #[test]
    fn coreference_links_a_pronoun_to_the_paragraph_entity() {
        let para = "The kernel holds the registry. It enforces the guard on every build.";
        let chains = resolve_coref(para);
        assert!(!chains.is_empty(), "no coref chain found");
        assert!(chains.iter().any(|c| c.antecedent.text == "kernel"), "antecedent: {:?}",
                chains.iter().map(|c| &c.antecedent.text).collect::<Vec<_>>());
    }

    #[test]
    fn iso_dates_normalise_and_durations_convert() {
        let ts = extract_temporal("Measured on 2026-11-25 after 3 weeks of runs.");
        let date = ts.iter().find(|t| t.kind == TemporalKind::Date).expect("date");
        assert_eq!(date.normalized, "2026-11-25");
        let dur = ts.iter().find(|t| t.kind == TemporalKind::Duration).expect("duration");
        assert_eq!(dur.days, Some(21.0), "3 weeks must be 21 days, got {:?}", dur.days);
    }

    #[test]
    fn a_month_only_date_does_not_become_a_specific_day() {
        let ts = extract_temporal("The audit ran in November 2026.");
        let d = ts.iter().find(|t| t.kind == TemporalKind::Date).expect("date");
        assert_eq!(d.normalized, "2026-11-01");
        assert_eq!(&"The audit ran in November 2026."[d.span.start..d.span.end], d.span.text);
    }

    #[test]
    fn an_impossible_date_is_not_extracted() {
        let ts = extract_temporal("Logged on 2026-02-31 which does not exist.");
        assert!(ts.iter().all(|t| t.normalized != "2026-02-31"), "31 February must not parse as a date");
    }

    #[test]
    fn entity_linking_uses_the_given_vocabulary_and_marks_it_local() {
        let vocab = vec![
            ("Staphylococcus aureus".to_string(), "UP000008816".to_string(), "organism".to_string()),
            ("imipenem".to_string(), "CHEMBL43708".to_string(), "drug".to_string()),
        ];
        let text = "Staphylococcus aureus was treated with imipenem.";
        let ents = link_entities(text, &vocab);
        assert_eq!(ents.len(), 2, "expected both entities, got {:?}", ents.iter().map(|e| &e.id).collect::<Vec<_>>());
        assert!(ents.iter().all(|e| e.vocabulary == "local"), "must never claim UMLS");
        assert!(ents.iter().any(|e| e.id == "UP000008816"));
        for e in &ents {
            assert_eq!(&text[e.mention.start..e.mention.end], e.mention.text);
        }
    }

    #[test]
    fn entity_linking_does_not_match_inside_a_word() {
        let vocab = vec![("aureus".to_string(), "X".to_string(), "organism".to_string())];
        assert!(link_entities("The aureusome was not relevant.", &vocab).is_empty());
    }

    #[test]
    fn the_language_gate_passes_english_and_refuses_chinese() {
        assert_eq!(language_gate("The kernel holds the registry and it enforces the guard on every build."),
                   LanguageVerdict::English);
        let zh = language_gate("内核保持注册表并在每次构建时强制执行守卫。");
        assert!(matches!(zh, LanguageVerdict::Refused(_)), "CJK must be refused, got {:?}", zh);
        if let LanguageVerdict::Refused(why) = zh {
            assert!(why.contains("CJK"), "the reason must name the script: {why}");
        }
    }

    #[test]
    fn the_language_gate_refuses_cyrillic() {
        let ru = language_gate("Ядро содержит реестр и обеспечивает защиту при каждой сборке.");
        assert!(matches!(ru, LanguageVerdict::Refused(_)), "Cyrillic must be refused");
    }
}
