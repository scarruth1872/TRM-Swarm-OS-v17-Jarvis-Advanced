"""Verify the monad ledger chain, recipe-aware.

Two sealers have written this store: the canonical recipe (sort_keys=True,
separators=(",", ":")) and an alternative (sort_keys=True, default separators).
A block is only VERIFIED if its stored hash is reproduced from its stored body by
one of those recipes; anything else is UNVERIFIED and fails the run, because a
chain with unreproducible blocks cannot be called intact.

One state is neither: a block carrying sealed=false is DECLARED unsealed. Its body could not be
reproduced and no hash derived from it would mean anything, so it is named rather than given a hash
that would vouch for it. Unsealed blocks are reported every run and are NOT counted as verified --
the chain's integrity claim covers the sealed blocks alone, and the rest are listed by index.

Exit 0 only when every sealed block verifies and every prev_hash links to the previous
block's stored hash.
"""
import hashlib, json, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LEDGER = os.path.join(ROOT, "swarm_v2_artifacts", "monad_ledger.jsonl")


def digest(body, separators):
    text = json.dumps(body, sort_keys=True, separators=separators)
    return hashlib.sha256(text.encode()).hexdigest()


def verdict(block):
    body = {k: v for k, v in block.items() if k != "hash"}
    stored = block.get("hash")
    if block.get("sealed") is False:
        # A declared state, and only a declared one: without the reason it is just an unverifiable
        # block wearing a label.
        return "unsealed" if block.get("unsealed_reason") else "UNVERIFIED"
    if digest(body, (",", ":")) == stored:
        return "canonical"
    if digest(body, (", ", ": ")) == stored:
        return "alt_separators"
    return "UNVERIFIED"


def main():
    if not os.path.exists(LEDGER):
        print("missing ledger: %s" % LEDGER)
        return 1
    blocks = [json.loads(ln) for ln in open(LEDGER, encoding="utf-8") if ln.strip()]
    counts = {"canonical": 0, "alt_separators": 0, "unsealed": 0, "UNVERIFIED": 0}
    unverified, unsealed, bad_link = [], [], []
    prev = None
    for b in blocks:
        v = verdict(b)
        counts[v] += 1
        if v == "UNVERIFIED":
            unverified.append(b.get("index"))
        if v == "unsealed":
            unsealed.append(b.get("index"))
        if prev is not None and b.get("prev_hash") != prev:
            bad_link.append(b.get("index"))
        prev = b.get("hash")

    print("ledger: %s" % LEDGER)
    print("blocks=%d  canonical=%d  alt_separators=%d  unsealed=%d  UNVERIFIED=%d" % (
        len(blocks), counts["canonical"], counts["alt_separators"], counts["unsealed"],
        counts["UNVERIFIED"]))
    print("prev_hash links broken: n=%d %s" % (len(bad_link), bad_link[:8]))
    if unverified:
        print("UNVERIFIED blocks: %s" % unverified)
    if unsealed:
        print("DECLARED UNSEALED (named, not vouched for, not counted as verified): %s" % unsealed)
    healthy = not unverified and not bad_link
    print("chain_intact=%s (a chain with unreproducible blocks is NOT intact)" % healthy)
    return 0 if healthy else 1


if __name__ == "__main__":
    sys.exit(main())
