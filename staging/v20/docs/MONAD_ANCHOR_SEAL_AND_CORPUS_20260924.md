# Monad anchor seal + task corpus (2026-09-24)

## The anchor is in the store now — as its trit projection, because nothing else is possible

The anchor was absent from the fabric under **every** encoding searched (raw bytes: 0 hits; trit
projection: 0 hits). It is now written as a **genesis seal: 32 trits at indices 0..31**, through the
store's owning writer `rust_nand_append_decision` (rc 0 on all 32), with the 73 MB store backed up
first (`backups/fabric_pre_anchor_20260924_122329/`, `ec338a2117b60a79` -> `2fff39b583b9126d`).

**It cannot be byte-identical, and saying so is the point.** The store's alphabet is one nibble per
trit with legal nibbles `0b0000`/`0b0001`/`0b0011` and `MAX_ILLEGAL_NIBBLE_FRACTION = 0.0`. The
anchor `0x4D4F4E4144000001` carries the 2-bit group `0b10` in its `0x4E` byte — a code the engine's
map cannot produce. So it reads back `0x4D4F4D4144000001`, differing in exactly **byte 2**. The
kernel reports this as a derived check (`byte_identical: false`, `differing_bytes: [2]`), never as a
pass. That is also why the anchor has always lived in headers, strings and ledgers, where raw bytes
are representable, and never in the trit store.

## The ruler was wrong before the value was

`/k3/status` audited the store with **2-bit fields** (the mailbox alphabet) rather than the store's
own **nibble** rule. Corrected against `nand_persistence.rs`. Live result after the fix: the first
4 KiB is **8192/8192 legal**, trit shape `-1=2718 / 0=5402 / +1=72`; auditing with the wrong
alphabet reported a number that meant nothing for this store.

## USB boot drive

`E:` refreshed twice this session (`a38854bcb5114447` -> `27a9e6b0a3caeac1` -> `bc5360f24d839adf`),
each **hash-verified after the copy**, rollback in `backups/usb_pre_r16_20260924_122159/`.
`trm_swarm_boot.exe` was deliberately **not** replaced: the tree copy is dated 09-20, older than its
09-23 source, so the drive's 09-23 build is the newer artifact — copying the tree's would downgrade it.

## Task results are now answerable from the platform's own fabric

Six completed-task documents added to the kernel corpus and re-embedded with the sovereign native
encoder: **180 -> 186 documents**; the tercode store rebuilt **180 -> 186 rows** (45,453 B, payload
`ae885860d508e626`). Verified by retrieval — `ASSET_INVENTORY_AND_FIRST_TASK_20260924.md` ranks **#2**
for the RDKit descriptor query. On the anchor and docking queries platform papers outrank mine
(scores 89/89/86/84 on docking): the fabric's discrimination remains its known weak spot, recorded
rather than papered over.

## In flight

RDKit feature rebuild at **2,179,824 / 2,897,818 rows (75.2%)** and still writing. When it lands:
validate it (row count, hand-checks, stream integrity) and run the ternary verification at corpus
scale through the native strip, then publish those verdicts over IPC and seal them.
