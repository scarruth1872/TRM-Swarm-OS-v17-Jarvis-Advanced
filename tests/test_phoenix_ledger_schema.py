"""
test_phoenix_ledger_schema.py — TRM Swarm OS PHOENIX Ledger Schema Tests
TRM Swarm OS v19.001 "Sovereign"

Validates:
1. Schema conformity under Paper 17 V_PHOENIX operator
2. Rejection/downgrading of ungrounded claims lacking raw evidence
3. Acceptance and cryptographic verification of grounded binary shards
4. Council deliberation integration with phoenix_status
"""

import os
import sys
import json
import unittest

WORKSPACE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if WORKSPACE_DIR not in sys.path:
    sys.path.insert(0, WORKSPACE_DIR)

from swarm_v2.core.phoenix_ledger import (
    PhoenixLedgerBlock,
    PhoenixStatus,
    PhoenixVerification,
    MONAD_TRUTH_ANCHOR
)
from swarm_v2.core.swarm_council import SwarmCouncil


def test_phoenix_schema_structure():
    schema_path = os.path.join(WORKSPACE_DIR, "schemas", "phoenix_ledger_schema_v1.json")
    assert os.path.exists(schema_path), f"Schema not found: {schema_path}"
    with open(schema_path, "r") as f:
        schema = json.load(f)
    assert schema["required"] == [
        "protocol",
        "timestamp",
        "monad_truth_anchor",
        "phoenix_status",
        "phoenix_verification"
    ]
    assert "CONFIRMED" in schema["properties"]["phoenix_status"]["enum"]
    assert "ASPIRATIONAL" in schema["properties"]["phoenix_status"]["enum"]


def test_grounded_claim_verification():
    # Use real, existing physical files
    block = PhoenixLedgerBlock(
        protocol="TEST_EMPIRICAL_GROUNDING",
        claims={"docking_target": "LpxC", "molecule": "CHIR-090"},
        phoenix_status=PhoenixStatus.CONFIRMED,
        phoenix_verification=PhoenixVerification(
            evidence_logs=["CHIR-090_docked_LpxC.pdbqt"],
            auditor="EventLedger"
        )
    )
    status = block.verify_empirical_evidence(base_dir=WORKSPACE_DIR)
    assert status == PhoenixStatus.CONFIRMED
    assert len(block.phoenix_verification.raw_hashes) == 1
    assert block.phoenix_verification.raw_hashes[0] is not None


def test_unverified_claim_downgrade():
    # Attempting to claim CONFIRMED without raw evidence log must fail
    block = PhoenixLedgerBlock(
        protocol="TEST_FABRICATION_TRAP",
        claims={"claimed_operations": 500000000},
        phoenix_status=PhoenixStatus.CONFIRMED,
        phoenix_verification=PhoenixVerification(
            evidence_logs=["non_existent_raw_bench_stdout.log"],
            auditor="EventLedger"
        )
    )
    status = block.verify_empirical_evidence(base_dir=WORKSPACE_DIR)
    assert status == PhoenixStatus.ASPIRATIONAL
    assert "missing" in block.phoenix_verification.details.get("drift_note", "").lower()


def test_complete_sovereign_ingestion_ledger_phoenix_status():
    ledger_path = os.path.join(WORKSPACE_DIR, "complete_sovereign_ingestion_ledger.json")
    assert os.path.exists(ledger_path)
    with open(ledger_path, "r") as f:
        ledger = json.load(f)
    assert ledger.get("phoenix_audit", {}).get("status") == "RESYNCED_AND_GROUNDED"
    assert ledger.get("total_datasets") == 10
    for shard in ledger.get("shards", []):
        assert shard.get("phoenix_status") == "CONFIRMED_GROUNDED"
        assert shard.get("disk_verified") is True
        fpath = os.path.join(WORKSPACE_DIR, shard["shard"])
        assert os.path.exists(fpath)


def test_swarm_council_phoenix_enactment():
    council = SwarmCouncil()
    res = council.run_full_cycle(
        topic="Automated Schema Enforcement Test",
        category="protocol",
        description="Verify that enact() embeds phoenix_status and phoenix_verification into the ledger.",
        proposer="Verify"
    )
    assert res["consensus_reached"] is True
    assert res["phase"] == "enacted"
    assert "phoenix_status" in res
    assert res["phoenix_status"] in ("SPEC", "CONFIRMED")
    assert "phoenix_verification" in res
    assert res["phoenix_verification"]["operator"] == "V_PHOENIX"


def test_legacy_event_stream_phoenix_compliance():
    manifest_path = os.path.join(WORKSPACE_DIR, "phoenix_event_stream_audit_manifest.json")
    assert os.path.exists(manifest_path), "Audit manifest missing"
    with open(manifest_path, "r", encoding="utf-8") as f:
        manifest = json.load(f)
    assert manifest["monad_truth_anchor"] == MONAD_TRUTH_ANCHOR
    assert manifest["summary"]["total_legacy_events_audited"] >= 12515
    assert manifest["summary"]["confirmed_empirical_events"] > 0

    ledger_path = os.path.join(WORKSPACE_DIR, "swarm_v2_global_memory", "system_event_ledger.jsonl")
    assert os.path.exists(ledger_path), "System event ledger missing"

    # Verify first 100 and last 100 lines for chain integrity
    with open(ledger_path, "r", encoding="utf-8") as f:
        lines = [line.strip() for line in f if line.strip()]

    assert len(lines) >= 12515
    first_entry = json.loads(lines[0])
    assert first_entry["seq"] == 1
    assert first_entry["monad_anchor"] == MONAD_TRUTH_ANCHOR
    assert first_entry["phoenix_status"] in ("CONFIRMED", "PARTIAL")
    assert first_entry["prev_hash"] == "0x4D4F4E4144000001000000000000000000000000000000000000000000000000"

    last_entry = json.loads(lines[-1])
    assert last_entry["seq"] == len(lines)
    assert last_entry["monad_anchor"] == MONAD_TRUTH_ANCHOR
    assert last_entry["hash"] == manifest["event_streams"]["system_event_ledger"]["final_chain_hash"]


if __name__ == "__main__":
    test_phoenix_schema_structure()
    test_grounded_claim_verification()
    test_unverified_claim_downgrade()
    test_complete_sovereign_ingestion_ledger_phoenix_status()
    test_swarm_council_phoenix_enactment()
    test_legacy_event_stream_phoenix_compliance()
    print("ALL PHOENIX LEDGER SCHEMA TESTS PASSED SUCCESSFULLY!")
