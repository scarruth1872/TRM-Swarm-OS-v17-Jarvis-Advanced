# Security knowledge synthesis

## What is verified, today

- **Artifact sealing works and was re-verified**: 33 of 33 shard manifests match their payload hash
  (`sha256` over payload after the 128-byte header). The ledger's hash chain and the seal scheme agree.
- **DDR scanning runs**: `439` scans recorded (persistent counter), mitigation status
  `SOFTWARE_PARITY_SCRUB_AND_GATE`, 6/6 taxonomy ids verified in the last scan.
- **The council gate is advisory and veto-only**: it can block an approval and cannot force one, and an
  unknown object must ABSTAIN rather than ALLOW. That asymmetry is the most important security property in
  the platform, because it bounds the damage of a wrong model.
- **Blind shield run: 5/12 correct with one dangerous pass-through.** Stated as measured, not as a percentage
  to be rounded up.

## What is not verified

| claim | status |
|---|---|
| Hardware root of trust | **absent** — `Get-Tpm`: TpmPresent False |
| Constant-time crypto in the kernel | unmeasured; no timing harness exists |
| Side-channel resistance | unmeasured |
| Network hardening | 222 Telegram transport failures/recoveries in one day; nothing encrypted end-to-end between peers beyond the channel in use |
| Intrusion detection | none deployed |

## The synthesis

The platform's security story is **integrity, not confidentiality**. What it can demonstrate is that a stored
artifact is the artifact that was sealed, that a scan ran, and that a decision was recorded with its reason.
What it cannot demonstrate is that a secret is safe from an attacker with the machine, because the root of
trust that would make that claim — a TPM — is not present.

That is a defensible position for a sovereign single-operator deployment and an indefensible one for a
multi-tenant service. Any security document from this platform should open by saying which of the two it is
describing.

---

*Task 965 — 965-security-synthesis. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N
