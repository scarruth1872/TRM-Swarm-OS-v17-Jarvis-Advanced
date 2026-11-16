"""Canonical store paths for the NAND fabric.

THREE formats were sharing ONE path, which is why the K3 store reads 47% illegal:

  1. swarm_v2_nand_fabric.k3        Rust-owned BINARY K3 trit array.
                                   2 trits/byte, 4-bit nibble codes 0b00/0b01/0b11.
                                   Owned by swarm_v2_rust_kernel/src/nand_persistence.rs.
                                   Memory-mapped by the kernel. NOTHING ELSE MAY OPEN IT.
  2. swarm_v2_nand_fabric.turbovec  TurboVec 16x packed vector index (binary, "wb").
  3. swarm_v2_nand_text.jsonl       Python-written JSON records (append, utf-8).

`ingest_sovereign_datasets.py` used to open the .k3 store in "wb" and TRUNCATE the
kernel's trit array; six other scripts text-appended UTF-8 into it. A memory-mapped
binary store written by a text handle is silent corruption, and a binary index that
truncates a mounted store is data loss.

Council MOTION-007 (12/0/2, enacted) - Shield condition: no agent is granted write
access to a store already proven inconsistent. Hence: one owner per store.
"""
import os

K3_TRIT_STORE_BASENAME = "swarm_v2_nand_fabric.k3"          # Rust-owned. Hands off.
TURBOVEC_INDEX_BASENAME = "swarm_v2_nand_fabric.turbovec"   # Python-owned binary index.
TEXT_SIDECAR_BASENAME = "swarm_v2_nand_text.jsonl"          # Python-owned text records.

_K3_ALIASES = frozenset({
    "swarm_v2_nand_fabric.k3",
    "swarm_v2_nand_fabric.K3",
})

# Any name in this namespace is a K3 trit store, whatever root it sits under.
_K3_SUFFIXES = (".k3",)


def k3_trit_store(root: str) -> str:
    """Path of the Rust-owned binary K3 store. Read-only contact for Python: never
    open this in text or write mode. Exposed so tooling can SIZE it, not write it."""
    return os.path.join(root, K3_TRIT_STORE_BASENAME)


def turbovec_index(root: str) -> str:
    """Path of the Python-owned TurboVec binary index (safe to open 'wb')."""
    return os.path.join(root, TURBOVEC_INDEX_BASENAME)


def text_records(root: str) -> str:
    """Path of the Python-owned append-only text record sidecar."""
    return os.path.join(root, TEXT_SIDECAR_BASENAME)


def is_k3_store(path) -> bool:
    """True if `path` names a K3 trit store under ANY root, by name or suffix."""
    s = str(path)
    return os.path.basename(s) in _K3_ALIASES or s.lower().endswith(_K3_SUFFIXES)


def assert_not_k3(path, mode=None, encoding=None) -> None:
    """Guard for every Python writer. Refuse the kernel's trit store.

    Two independent refusals, because either alone leaves a hole open:

      1. ONE OWNER PER STORE (council MOTION-007 / Shield condition). The Rust
         kernel memory-maps `swarm_v2_nand_fabric.k3`; a second writer is silent
         corruption even in binary append mode, so every write mode is refused.
      2. A BINARY STORE HAS NO TEXT ENCODING. `open(p, "a", encoding="utf-8")`
         is the exact shape that corrupted the root fabric. A text handle on a
         trit array turns the first newline into a nibble boundary shift.

    Only a binary *read* handle ("r"/"rb") is allowed through -- tooling may
    size or inspect the store, never write it.

    `mode`/`encoding` are optional so the original writer-guard call
    `assert_not_k3(p)` keeps refusing by name alone rather than silently
    widening to "allowed" when the caller passes nothing.
    """
    if not is_k3_store(path):
        return

    if mode is None:
        # Bare call == "I am about to write here". Preserve the original contract.
        writing, why = True, "a writer guard was invoked with no mode"
    else:
        m = str(mode)
        writing = any(c in m for c in "wax+")
        if writing:
            why = f"mode {m!r} opens it for writing"
        elif encoding is not None:
            writing, why = True, f"encoding={encoding!r} opens a binary trit array as text"
        else:
            why = ""

    if writing:
        raise RuntimeError(
            f"REFUSED: {path} is the Rust-owned binary K3 trit store ({why}). "
            "The kernel memory-maps it; Python must not open it in text or write mode. "
            "Use text_records(root) for JSON records or turbovec_index(root) for the "
            "binary index. See council MOTION-007 / Shield condition."
        )

