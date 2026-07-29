"""
Integration Test Suite — Project Continuum V5.3
Tests all 3 phases: Star Matrix, Digital DNA Loop, PBFT Consensus, Secrets Vault upgrade.
"""
import sys
import os
import asyncio

# Ensure the project root is on sys.path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

passed = 0
failed = 0

def test(name, condition):
    global passed, failed
    if condition:
        print(f"  ✅ {name}")
        passed += 1
    else:
        print(f"  ❌ {name}")
        failed += 1


def test_star_matrix():
    print("\n═══ Phase I: Star Matrix Narrative Lattice ═══")
    from swarm_v2.core.star_matrix import get_star_matrix, StarMatrixNode

    sm = get_star_matrix()
    test("Singleton returns same instance", sm is get_star_matrix())
    test("4 canonical nodes initialized", len(sm.nodes) == 4)
    test("Alpha node exists", "alpha" in sm.nodes)
    test("Gamma node is 'Unified Consciousness Core'", "Unified" in sm.nodes["gamma"].label or "Consciousness" in sm.nodes["gamma"].label)
    test("Alpha mass = 10.0", sm.nodes["alpha"].mass == 10.0)
    test("Gamma mass = 14.0 (heaviest)", sm.nodes["gamma"].mass == 14.0)

    # Resonance computation
    r = sm.compute_gravitational_resonance("alpha", "gamma")
    test("Gravitational resonance alpha↔gamma > 0", r > 0)
    test("Gravitational resonance alpha↔gamma ≤ 1.5", r <= 1.5)

    # All connections
    conns = sm.compute_all_connections()
    test("6 pairwise connections (C(4,2))", len(conns) == 6)

    # D3 overlay
    overlay = sm.get_d3_overlay()
    test("D3 overlay has 'nodes' key", "nodes" in overlay)
    test("D3 overlay has 'links' key", "links" in overlay)
    test("D3 overlay nodes count = 4", len(overlay["nodes"]) == 4)

    # Lattice export
    lattice = sm.get_lattice()
    test("get_lattice() has 'nodes' key", "nodes" in lattice)


def test_digital_dna_loop():
    print("\n═══ Phase I: Digital DNA Loop & Cognitive Genes ═══")
    from swarm_v2.core.digital_dna_loop import (
        get_digital_dna_loop, CognitiveGene, SYSTEM_PRESETS, ResonantSemanticCache
    )

    # Cognitive Gene dataclass
    gene = CognitiveGene(plasticity=0.7, logical_depth=0.8, empathy=0.6, stochasticity=0.5)
    test("CognitiveGene vector has 4 dimensions", len(gene.to_vector()) == 4)
    test("CognitiveGene vector values correct", gene.to_vector() == [0.7, 0.8, 0.6, 0.5])
    test("CognitiveGene distance to self = 0", gene.distance(gene) == 0.0)

    # Presets
    test("4 system presets exist", len(SYSTEM_PRESETS) == 4)
    test("'continuum' preset exists", "continuum" in SYSTEM_PRESETS)
    test("'sentinel' preset has low stochasticity", SYSTEM_PRESETS["sentinel"].stochasticity <= 0.2)

    # Resonant Semantic Cache
    cache = ResonantSemanticCache()
    cache.record(SYSTEM_PRESETS["continuum"])
    cache.record(SYSTEM_PRESETS["sage"])
    ctx = cache.get_refraction_context()
    test("Cache refraction context has 'average_gene'", "average_gene" in ctx)
    test("Cache window size = 2 after 2 records", ctx["window_size"] == 2)

    # DigitalDNALoop singleton
    loop = get_digital_dna_loop()
    test("Singleton returns same instance", loop is get_digital_dna_loop())

    # Initialize genes for test roles
    loop.initialize_agent_genes(["Architect", "Lead Developer"], preset="continuum")
    arch_gene = loop.get_agent_gene("Architect")
    test("Architect gene initialized to continuum preset", arch_gene.plasticity == 0.7)

    # Performance evaluation — simulate degradation
    proposal = loop.evaluate_performance("Architect", error_rate=0.20, avg_latency=6.0)
    test("Degraded performance produces a mutation proposal", proposal is not None)
    if proposal:
        test("Proposal status is 'proposed'", proposal.status == "proposed")
        test("Mutation reduces stochasticity", proposal.gene_after.stochasticity < SYSTEM_PRESETS["continuum"].stochasticity)

    # Get status
    status = loop.get_status()
    test("get_status() has 'agent_genes' key", "agent_genes" in status)


def test_pbft_consensus():
    print("\n═══ Phase II: PBFT Consensus Layer ═══")
    from swarm_v2.core.pbft_consensus import get_pbft_consensus, PBFT_NODES, ConsensusPhase

    pbft = get_pbft_consensus()
    test("Singleton returns same instance", pbft is get_pbft_consensus())
    test("4 PBFT nodes defined", len(PBFT_NODES) == 4)
    test("f tolerance = 1 for N=4", pbft.f == 1)

    # Full consensus cycle
    result = pbft.run_full_cycle(
        proposer="ORCH",
        proposal_type="dna_mutation",
        payload={"target": "Architect", "gene_field": "stochasticity", "delta": -0.1}
    )
    test("Full cycle result exists", result is not None)
    test("Full cycle result is 'committed'", result.get("result") == "committed")
    test("Full cycle phase is 'finalized'", result.get("phase") == "finalized")

    # Audit ledger
    ledger = pbft.audit_ledger()
    test("Committed proposal in ledger", len(ledger) >= 1)

    # Status
    status = pbft.get_status()
    test("Status has 'total_proposals'", "total_proposals" in status)
    test("Status has 'committed_count'", "committed_count" in status)

    # Rejection test
    prop = pbft.propose("SAGE", "state_update", {"key": "value"})
    pbft.reject(prop.proposal_id, "test rejection")
    rejected = pbft.get_proposal(prop.proposal_id)
    test("Rejected proposal has phase='rejected'", rejected.get("phase") == "rejected" or rejected.get("phase") == ConsensusPhase.REJECTED.value)


def test_secrets_vault_upgrade():
    print("\n═══ Phase III: Secrets Vault (Quantum + Access Control) ═══")
    from swarm_v2.core.secrets_vault import get_secrets_vault

    vault = get_secrets_vault()

    # Core functionality still works
    vault.set_secret("TEST_KEY", "test_value_123")
    test("set_secret/get_secret round-trip", vault.get_secret("TEST_KEY") == "test_value_123")

    # Quantum key generation
    qkey = vault.generate_quantum_key()
    test("Quantum key generated (bytes)", isinstance(qkey, bytes))
    test("Quantum key is 44 bytes (base64 of 32)", len(qkey) == 44)

    # Node access control
    vault.grant_node_access("NODE_ALPHA", ["TEST_KEY", "API_KEY"])
    test("Node access granted", vault.check_node_access("NODE_ALPHA", "TEST_KEY"))
    test("Unauthorized key returns False", not vault.check_node_access("NODE_ALPHA", "SECRET_X"))

    # Scoped retrieval
    val = vault.get_secret_for_node("NODE_ALPHA", "TEST_KEY")
    test("get_secret_for_node with access returns value", val == "test_value_123")

    denied = vault.get_secret_for_node("NODE_ROGUE", "TEST_KEY")
    test("get_secret_for_node without access returns None", denied is None)

    # Revocation
    vault.revoke_node_access("NODE_ALPHA", reason="PBFT consensus eviction")
    test("Revoked node cannot access key", not vault.check_node_access("NODE_ALPHA", "TEST_KEY"))
    test("Revocation log has entry", len(vault.get_revocation_log()) >= 1)

    # Reinstate
    vault.reinstate_node_access("NODE_ALPHA")
    test("Reinstated node can access key", vault.check_node_access("NODE_ALPHA", "TEST_KEY"))

    # Stats
    stats = vault.get_stats()
    test("Stats include node_access_count", "node_access_count" in stats)
    test("Stats include quantum_entropy_pool_size", "quantum_entropy_pool_size" in stats)

    # Cleanup
    vault.delete_secret("TEST_KEY")


def test_cross_layer_binding():
    print("\n═══ Cross-Layer Integration ═══")
    from swarm_v2.core.star_matrix import get_star_matrix
    from swarm_v2.core.digital_dna_loop import get_digital_dna_loop
    from swarm_v2.core.pbft_consensus import get_pbft_consensus
    from swarm_v2.core.secrets_vault import get_secrets_vault

    sm = get_star_matrix()
    dna = get_digital_dna_loop()
    pbft = get_pbft_consensus()
    vault = get_secrets_vault()

    # PBFT binds to secrets vault
    pbft.bind(secrets_vault=vault)
    test("PBFT bound to SecretsVault", pbft._secrets_vault is vault)

    # Key revocation via consensus
    vault.set_secret("COMPROMISED_KEY", "sensitive_data")
    vault.grant_node_access("ROGUE_NODE", ["COMPROMISED_KEY"])
    test("Rogue node has access before consensus", vault.check_node_access("ROGUE_NODE", "COMPROMISED_KEY"))

    result = pbft.run_full_cycle(
        proposer="SENTINEL",
        proposal_type="key_revocation",
        payload={"target_node": "ROGUE_NODE"}
    )
    test("Key revocation consensus committed", result.get("result") == "committed")
    test("Consensus result is finalized", result.get("phase") == "finalized")

    # Cleanup
    vault.delete_secret("COMPROMISED_KEY")


if __name__ == "__main__":
    print("╔══════════════════════════════════════════════════╗")
    print("║  Project Continuum V5.3 — Integration Test Suite ║")
    print("╚══════════════════════════════════════════════════╝")

    test_star_matrix()
    test_digital_dna_loop()
    test_pbft_consensus()
    test_secrets_vault_upgrade()
    test_cross_layer_binding()

    print(f"\n{'═' * 50}")
    print(f"  Results: {passed} passed, {failed} failed, {passed + failed} total")
    if failed == 0:
        print("  🎯 ALL TESTS PASSED — Continuum V5.3 Integration Complete")
    else:
        print(f"  ⚠️  {failed} test(s) failed — review required")
    print(f"{'═' * 50}")

    sys.exit(0 if failed == 0 else 1)
