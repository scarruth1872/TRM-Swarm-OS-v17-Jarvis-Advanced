"""Integration test: the TRM 7M reasoning brain loads its checkpoint and reasons.

REWRITTEN 2026-11-03. The previous version could not fail:
  * it checked `brain._model` immediately after `get_trm_brain()`, but TRMBrain
    became LAZY (trm_brain.py: "LAZY LOAD ... the checkpoint loads on first real
    use"), so `_model` was always None at that point and the test did `return` —
    reporting PASS while printing "Failed to initialize TRM model."
  * it had no assertions at all and wrapped everything in `except Exception`, so
    a genuine load failure also reported PASS.
It now drives the lazy load through the public `reason()` path and asserts on
the result.
"""

import os
import sys

import pytest

# Ensure we can import from the root
sys.path.append(os.getcwd())

from swarm_v2.core.trm_brain import get_trm_brain


@pytest.mark.integration  # loads a real torch checkpoint; excluded from the default run
def test_trm():
    brain = get_trm_brain()

    # Dummy tokens within vocab size 12. reason() triggers the lazy checkpoint load.
    test_input = [1, 2, 3, 4, 5, 0, 0, 0]
    preds = brain.reason(test_input)

    assert brain._model is not None, (
        f"TRM model failed to load from {brain.checkpoint_path!r}; "
        "reason() returned no predictions"
    )
    assert isinstance(preds, list) and len(preds) > 0, (
        "TRM Brain returned an empty prediction list"
    )
    # reason() emits 12 positions per H_cycle (legacy behaviour).
    h_cycles = brain.config["H_cycles"]
    assert len(preds) == 12 * h_cycles, (
        f"expected {12 * h_cycles} predictions ({h_cycles} H_cycles x 12), got {len(preds)}"
    )
    vocab = brain.config["vocab_size"]
    assert all(isinstance(p, int) and 0 <= p < vocab for p in preds), (
        f"predictions outside vocab range 0..{vocab - 1}: {preds[:20]}"
    )


if __name__ == "__main__":
    test_trm()
