"""Quorum arithmetic for the PBFT consensus layer — pinned by execution, not by inspection.

These tests exist because the consensus layer looked correct and was not: the self-modification
API proposed as a node that does not exist and then counted votes from three more invented names,
so every architectural self-modification was rejected before voting, and the "quorum" that was
recorded could be forged by any in-process caller.

Every assertion here is about an observable outcome (phase, result, ledger length), so the tests
fail if the arithmetic drifts.
"""

import pytest

from swarm_v2.core.pbft_consensus import (
    PBFT_NODES,
    ConsensusPhase,
    PBFTConsensusLayer,
)


@pytest.fixture()
def layer():
    # Construct directly instead of via get_pbft_consensus(): the module-level singleton
    # accumulates a committed ledger across tests, which would make ledger-length assertions
    # order-dependent.
    return PBFTConsensusLayer()


def _proposal(layer, proposer="Archi"):
    return layer.propose(
        proposer=proposer,
        proposal_type="architectural_self_modification",
        payload={"target_file": "swarm_v2/core/sample_target.py", "description": "t"},
    )


def test_quorum_follows_the_textbook_formula_for_this_roster(layer):
    """f=(N-1)//3, prepare=2f, commit=2f+1 — for N=14 that is f=4, 8, 9."""
    n = len(PBFT_NODES)
    # 15 heads: the 14 agents plus LogicMaster (agent_id=14 in swarm_council.COUNCIL_MEMBERS).
    # ARCHITECTURE.md advertises "15-Agent Native Swarm Council ... PBFT Quorum >= 9"; with N=15,
    # f=(15-1)//3=4 and 2f+1=9, so the documented quorum holds and every head votes.
    assert n == 15, f"roster size changed to {n}; the quorum numbers below are derived from 15"
    assert layer.f == 4
    assert 2 * layer.f == 8  # prepare quorum
    assert 2 * layer.f + 1 == 9  # commit quorum


def test_prepare_quorum_advances_and_one_short_does_not(layer):
    """7 prepares keep the proposal in PREPARE; the 8th advances it to COMMIT."""
    p = _proposal(layer)
    assert layer.pre_prepare(p.proposal_id) is True, "pre_prepare rejected a valid proposer"
    assert p.phase is ConsensusPhase.PREPARE

    voters = [n for n in PBFT_NODES if n != "Archi"]
    for voter in voters[:7]:
        layer.prepare(p.proposal_id, voter, True)
    assert p.phase is ConsensusPhase.PREPARE, "advanced to COMMIT on 7/8 prepares"

    layer.prepare(p.proposal_id, voters[7], True)
    assert p.phase is ConsensusPhase.COMMIT, "did not advance on the 8th prepare"


def test_commit_quorum_finalizes_at_nine_and_not_at_eight(layer):
    """8 commit signals are not enough; the 9th finalizes and appends to the ledger."""
    p = _proposal(layer)
    layer.pre_prepare(p.proposal_id)
    for voter in [n for n in PBFT_NODES if n != "Archi"]:
        layer.prepare(p.proposal_id, voter, True)
    assert p.phase is ConsensusPhase.COMMIT

    ledger_before = len(layer.audit_ledger())
    nodes = list(PBFT_NODES)
    for committer in nodes[:8]:
        layer.commit(p.proposal_id, committer)
    assert p.phase is ConsensusPhase.COMMIT, "finalized on 8/9 commit signals"
    assert p.result != "committed"
    assert len(layer.audit_ledger()) == ledger_before, "committed to the ledger early"

    layer.commit(p.proposal_id, nodes[8])
    assert p.phase is ConsensusPhase.FINALIZED
    assert p.result == "committed"
    assert len(layer.audit_ledger()) == ledger_before + 1


def test_an_unknown_proposer_is_rejected(layer):
    """"ORCH" is not in PBFT_NODES; pre_prepare must refuse it."""
    assert "ORCH" not in PBFT_NODES
    p = _proposal(layer, proposer="ORCH")
    layer.pre_prepare(p.proposal_id)
    assert p.phase is ConsensusPhase.REJECTED
    assert p.result == "rejected"


def test_votes_from_non_members_are_not_counted(layer):
    """Fabricated voter names must not move the quorum — this is the forgery hole."""
    p = _proposal(layer)
    layer.pre_prepare(p.proposal_id)
    for fake in ("ORCH", "SAGE", "MUSE", "SENTINEL", "Nobody", "A" * 64):
        layer.prepare(p.proposal_id, fake, True)
    assert p.votes == {}, f"non-member prepares were counted: {p.votes}"
    assert p.phase is ConsensusPhase.PREPARE

    ledger_before = len(layer.audit_ledger())
    for fake in ("ORCH", "SAGE", "MUSE", "SENTINEL", "Nobody", "B" * 64, "C" * 64, "D" * 64, "E" * 64):
        layer.commit(p.proposal_id, fake)
    assert p.commit_signals == {}, f"non-member commits were counted: {p.commit_signals}"
    assert p.result != "committed"
    assert len(layer.audit_ledger()) == ledger_before


def test_votes_cannot_resurrect_a_rejected_proposal(layer):
    """A pre_prepare rejection is final — later votes must not flip it to COMMIT.

    Regression: prepare() counted approvals without checking the phase, so a proposal rejected
    for an invalid proposer was flipped to COMMIT by the roster's votes, and callers that read
    ``phase.value in ("commit", "finalized")`` reported it as approved. That masked the invalid
    proposer completely once the self-modification path started voting from real nodes.
    """
    p = _proposal(layer, proposer="ORCH")
    layer.pre_prepare(p.proposal_id)
    assert p.phase is ConsensusPhase.REJECTED

    for node in PBFT_NODES:
        layer.prepare(p.proposal_id, node, True)
    assert p.phase is ConsensusPhase.REJECTED, "votes resurrected a rejected proposal"
    assert p.votes == {}, f"rejected proposal still accumulated votes: {p.votes}"

    for node in PBFT_NODES:
        layer.commit(p.proposal_id, node)
    assert p.phase is ConsensusPhase.REJECTED
    assert p.result == "rejected", f"rejected proposal reported result={p.result!r}"
    assert layer.audit_ledger() == [], "rejected proposal reached the committed ledger"

def test_self_modification_is_approved_end_to_end(layer):
    """The EDSMA path must produce an approved transaction against the real roster."""
    from swarm_v2.core.self_modification_api import SelfModificationAPI

    api = SelfModificationAPI()
    tx = api.propose_modification(
        target_file="swarm_v2/core/sample_target.py",
        description="Optimize memory footprint",
        original_code="def old(): pass",
        proposed_code="def new_upgraded(): return True",
    )
    assert tx.verification_passed is True, f"FVE rejected: {tx.status}"
    assert tx.consensus_approved is True, f"PBFT rejected: {tx.status}"
    assert tx.status == "approved"
