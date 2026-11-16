# IPC transport + the K3 namespace (2026-09-24)

Two gaps closed in the Rust kernel. Both were found by following the platform's own doctrine
(derive, never assert) and its own documentation rather than my HTTP-shaped assumptions.

## What changed

**1. `/mailbox/ring` — the kernel now reads its own IPC transport** (`mailbox_ring.rs`, new)

The internal transport is the shared-memory ring bus `swarm_mailbox.ring`, not HTTP. The route reads
the file's own bytes and derives per-agent state from them:

```
header 64 B; per agent i: base = 64 + i*262272
  head u64 LE @ base, tail u64 LE @ base+64, data @ base+128, slot (n % 4096) * 64
message 64 B: msg_id u64@0, sender u8@8, recipient u8@9, k3_trit i8@10, priority u8@11,
              payload_size u32@12, inline_data 40 B@16, timestamp_ns u64@56
15 * 262272 + 64 == 3,934,144  (exactly the file size — the layout is self-checking)
```

`?detail=1` additionally tallies the trits of the occupied slots and states its scope. A short or
unreadable file reports `unscannable`/`unavailable`, never a clean empty bus.

**2. `/k3/status` + `/k3/stats`** (`k3_retrieval::status_handle`)

The README documents 49 namespaces including `/k3`, and `/k3/` was routed — but every sub-path except
`retrieve`/`tercode` returned 404, so the namespace was documented and unimplemented. The new route
derives (never asserts): the fabric store's path/bytes/trits, a bounded first-4 KiB audit with the
platform's own legal-code rule (`0b10` illegal), the tercode identity, and an IPC cross-reference.
Dispatched **before** the `!starts_with("/k3/retrieve")` prefix guard — a sibling route placed after
it can never be reached.

## Receipts (measured, not asserted)

| item | value |
|---|---|
| deployed exe | `27a9e6b0a3caeac1`, 1,682,944 B (was `e8ea2f3802c1179d`) |
| `/k3/status` fabric | 76,761,697 B → 153,523,394 trits; first 4 KiB: **0 illegal codes**, 10,897 neg / 5,404 zero / 83 pos |
| tercode store | 44,078 B |
| `/mailbox/ring` | 3,934,144 B, 15 agents, `head == tail == 4096`, pending **0** |
| `:5183` proxy | both routes return JSON (vite already proxies `/k3` and `/mailbox`) |
| ledger | block 102 sealed, `ENACT`, matches the tip's key names exactly |

## The ABI repair behind it

`AgentMailboxMessage` was **68 bytes** against a 64-byte ring slot: a redundant `_padding` field whose
comment read "Pad to 64 bytes" while the fields already summed to 64. `push` truncated to the slot and
`pop` then raised `ValueError: Buffer size too small (64 instead of at least 68)` — **no message could
ever be received**. The Rust ABI is the oracle: `native_mailbox_bridge.rs` declares `MSG_SIZE = 64`
with `assert!(size_of::<CAgentMailboxMessage>() == MSG_SIZE)` and no padding field, and the sibling
Python struct already agreed. Fix = drop the padding field; proven red-then-green (600/600 trit-tagged
verdicts round-trip after, `ValueError` before).

## Ring processed and drained

61,440 queued messages (15 × 4096, every ring saturated) were **archived before removal** —
`scratch/mailbox_ring_archive_20260924_121303.jsonl`, 9.09 MB — then drained **by consumption**, which
is what a consumer does rather than resetting counters. Pre-drain backup sha `93bb330c94fe3598` in
`backups/mailbox_ring_20260924_121237/`. The traffic was a synthetic benchmark fill (`task_00000000`,
`alert_00000`, `test`, one timestamp), not organic work.

## Honest state

- **Ledger chain**: 92 of 102 blocks reproduce and link; blocks **92–101 were already unreproducible
  before this seal** and remain so. This block did not cause them and does not fix them.
- **The USB boot drive (`E:`) still carries `host_council_server.exe` sha `a38854bcb5114447`** — two
  builds behind the deployed `27a9e6b0a3caeac1`. Updating it is a deliberate step, not done here.
- **`monad_anchor_hits_in_sample: 0`** in the store's first 4 KiB — a sample-scoped statement; no
  whole-store anchor claim is made.

## Update (2026-10-13): full-duplex — the kernel now writes too

This doc originally described the ring as a read-only telemetry surface. That is no longer true.
`mailbox_ring.rs::push(sender, recipient, trit, payload)` now writes the 64-byte ABI (single-producer
per agent, tail-advanced, fail-soft on a full/absent ring), and `evaluate_proposal` publishes each
council verdict as a trit-tagged message (Orchestra → EventLedger, `trit=k3_trit`). Cross-language
round-trip verified: a Python `SwarmMailboxBus.receive('EventLedger')` reads back the exact message the
Rust kernel wrote (block 348). The ring is a live bus; the Monad ledger remains the durable record.
