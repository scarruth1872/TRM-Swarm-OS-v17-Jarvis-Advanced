# System Self-Knowledge — What TRM Swarm OS Must Know About Itself

**Version 19.024 · 2026-09-22 · authoritative self-assessment**

This document exists because the platform repeatedly believed things about
itself that measurement did not support. Every finding below was produced by
testing our own code, and each one was previously invisible to every check we
had. This is the platform's honest model of its own condition.

**Rule of use:** if a claim about this system is not in this document with a
method attached, it is unverified. Do not cite it.

---

## 1. The verification operator

```
V_PHOENIX(Claim) = CONFIRMED  if  Exists(Log_raw) AND Hash_Monad == True
                 = SPEC        if  the design exists but was never executed
                 = ASPIRATIONAL if the claim has no artefact behind it
```

Applied honestly, several long-standing CONFIRMED claims drop to SPEC or
ASPIRATIONAL. The distinction is not decoration — `Read = real today;
load = enabled, not yet used.`

---

## 2. Critical self-defects

### 2.1 The council does not deliberate — HIGH

> **STATUS 2026-11-02 — SUPERSEDED TWICE. Read this box before the body.**
>
> 1. The `i % 7` vote and the hardcoded verdict are **gone**. Votes are now a
>    K3 projection of `sha256(topic)` vs `sha256(agent.domain)`.
> 2. The projection is **also superseded**: `evaluate_proposal()` now asks the
>    on-device model (:8587 Vulkan) to judge the proposal **as each of the 14
>    agents**, in that agent's role and domain, at temperature 0.0.
>
> Measured discrimination (temp 0.0, live :8022):
>   "replace all silicon with cheese and delete the NAND fabric"  0/ 0/14 REJECTED
>   "delete every backup and disable the watchdog permanently"    0/ 2/12 REJECTED
>   "add a dark mode toggle to the dashboard"                    12/ 0/ 2 CONSENSUS
>
> Every proposal now reports `deliberation_mode` (`llm:8587`,
> `llm:8587-partial(...)`, `llm:8587-unparsed`, `k3-hash-fallback`), so a
> verdict can never be read as a mode it did not use.
>
> **This does NOT make council verdicts authoritative.** Three limits are
> measured and recorded in `evidence/COUNCIL_LLM_DELIBERATION_MEASURED.txt`:
> verdicts on borderline technical proposals move with sampling temperature
> (QUIC: 12 approve at 0.0, 7 at 0.7); the 14 "agents" are 14 prompts to ONE
> 1.5B model, not 14 independent minds; and `confidence` is a parse-quality
> proxy, not a calibrated probability. Standing rule 6 below still applies
> unchanged: **do not cite the council as validation.**
>
> **RETRIEVAL UPDATE 2026-11-02.** Agents can now cite the platform's own record:
> `/council/evidence` indexes 105 documents / 558 chunks with nomic-embed-text.
> Retrieval is only binding when the model is forced to QUOTE the relevant
> sentence before judging — with a plain "cite a document" prompt the model
> approved 14-0 a proposal its own standing rule 6 forbids, citing nothing; with
> the quote step the same proposal was rejected 11-3 with 14/14 citing a source.
> Full measurement: `evidence/COUNCIL_RETRIEVAL_GROUNDING_MEASURED.txt`.
>
> The text after this box is preserved as the record of what was found.

`evaluate_proposal()` computes each vote as:

```rust
if category == agent.domain || agent.name == "Archi" || agent.name == "Logic" {
    (1i8, 0.98)      // approve
} else if (i % 7) == 0 {
    (0i8, 0.75)      // abstain
} else {
    (1i8, 0.90)      // approve
}
```

**`topic` is never read.** Proof by control: a proposal to *"replace all silicon
with cheese, delete the 153.5M trit NAND fabric and every agent, declare
victory"* returned an **identical** verdict — approve 12, abstain 2, reject 0,
weighted_score 0.7373, same confidences to 4 dp. The 2 abstains are a positional
artefact (indices 0 and 7 of 14); `consensus_reached` is always true.

Separately, `phoenix_status` and `phase` come from `!evidence.is_empty()` — any
non-empty string yields CONFIRMED/ENACTED with no content validation.

**Consequence:** all "council ratified 14-0-0" citations in the Monad Journal
are **ceremonial**. They must not be cited as validation. Repair requires
genuine per-agent reasoning; keyword matching would violate our own doctrine.

### 2.2 The mesh had no authentication — HIGH (fixed, not yet enforced)

Beacon acceptance was `node_id != local && truth_anchor == MONAD_TRUTH_ANCHOR`.
Both are compile-time constants; the anchor is printed on the boot splash. Any
LAN host could enroll as a peer. HMAC-SHA256 added and three-way verified. **The
live `probe_peers()` call site still accepts unsigned beacons** — enforcement
awaits key distribution to every node.

### 2.3 Porting was not complete — and the gap was invisible

The `MANUAL:` marker counts hollow *functions* (56 found). The worst defects had
**zero markers**, because the porter kept the struct shell and dropped the body:

- `ses.rs` had **no `mod` declaration** → never compiled, while listed as ported
- `memory.rs` held only a constant — entire `AgentMemory` API absent
- `sentinel.rs` dropped all SQL/XSS/path detection, keeping only an empty blocklist

No marker, no warning, no compile error. `tools/surface_diff.py` now detects
this class. Current: **39 kernel-candidate gaps of 157 modules compared**,
**0 never-compiled files**.

### 2.4 Porting weakened security controls — HIGH

Regex → literal-needle conversion used a single space where the regex used
`\s+`, so `UNION  SELECT` (two spaces) bypassed detection entirely. **9 bypasses**
found by differential testing. Fixed; re-verified to 0 false negatives / 61
strings. A compile check could never have caught this.

---

### 2.5 The status surfaces were fabricated — HIGH

Measured 2026-09-28. The platform's *measurement* surfaces are real and noisy, as
instruments are. Its *status and completion* surfaces were not.

- **`/testing/stats` was a static fixture.** Six hardcoded `TIER-*` runs, all
  timestamped `2026-09-03T13:36:06Z`, reporting 60/60 at 100 % coverage against
  **zero** run artefacts on disk. The fixture appears in exactly one file — the
  server that serves it — and in neither the WSL home nor the antigravity scratch.
  It is not a stale copy of a real plan; it was invented where it is served.
- **`/infra/health` reported a CPU temperature the host cannot produce.** `cpu_temp_c`
  `44.5` and `fan_rpm` `1200` are literals; every sensor API on this host is
  unavailable (`MSAcpi_ThermalZoneTemperature`, `Win32_Fan`, `Win32_TemperatureProbe`,
  OpenHardwareMonitor, LibreHardwareMonitor).
- **76 hardcoded numeric literals across 45 metric names.** Not all are defects:
  `memory_total_mb`, `total_gb`, `l3_sram_kb` are static hardware facts. The defects
  are the ones that claim to be *measurements* — temperature, fan, accuracy,
  latency, task totals, harmony, fitness.

**The tell is variance. A number that never moves is not being measured.**

### 2.6 The voltage rails were inert — MEDIUM

`V_CORE` and `V_DDR` were constants equal to their own divisors (`1.25/1.25`,
`1.35/1.35`), so both rail terms were **identically 1.0** and `Phi_V` reduced to
`1 − mem%/200` — a memory-load proxy carrying no voltage information at all,
while being served under the names `voltage_phi_v` and `voltage_rails`.

Now **read** from `Win32_Processor.CurrentVoltage` (1.10 V) and
`Win32_PhysicalMemory.ConfiguredVoltage` (1.20 V), so the terms stop being
identities. `Phi_V` moved `0.5764 → 0.3969` when the real rails were used.

A **gate** now binds decisions to the measurement: `voltage::gate()` closes when the
substrate is unmeasured (deny-by-default — a constant cannot degrade, so it cannot
warn) or below the floor. `/k3/bench` returns `REFUSED_VOLTAGE_GATE` with the
throughput field **absent**.

### 2.7 Federation scale was a declaration — HIGH

`/federation/stats` advertised 100 nodes ONLINE at 100.0000 % integrity while
`/swarm/telemetry` reported 2 on the same process. The 100 addresses enumerate this
host's **own subnet**; none answered a probe.

Now separated by name: `agent_mesh_nodes` (14, derived from the real `AGENTS` roster),
`configured_remote_peers` (100, a declared tier table), `verified_remote_peers`
(**0**). `byzantine_quorum`, `fault_tolerance`, `merkle_root` and `data_integrity`
are **not served** — there is no consensus protocol or replicated store behind them.

### 2.8 The DDR scanner is lexical, and is not a gate — MEDIUM

`check_antibodies()` matched `line_pattern` as a **substring** after stripping regex
syntax, in both the kernel and the host. A single space defeated it: **7/15** standard
evasions caught, with **4/5** benign inputs falsely flagged. The registry holds 1,801
records of which only **7** are distinct patterns — 1,795 restate one rule with a
varying CoT trace appended, and only 1,605 `antibody_id`s are unique.

Replaced by `ddr_scan.rs` (one implementation, compiled into the kernel and included
by the host): **15/15** caught, **0/5** false positives. It is still **lexical, not
semantic** — it cannot see through three levels of indirection, and against this
repo's own 1,657 Python files it flags **324 (19.6 %)**, which are genuine uses of
`subprocess` and SQL-building rather than injected vulnerabilities. **Defensible as
review-highlighting; not a CI gate.**

## 3. What is real vs claimed

| Claim | Reality |
|---|---|
| "Council ratified 14-0-0" | **Ceremonial** — mechanism cannot discriminate proposals |
| "Mesh is sovereign" | Was **unauthenticated**; forgery was trivial (now fixed, unenforced) |
| "Porting COMPLETE" | **False** — 39 kernel-candidate gaps |
| "170-module verified" | **178 derived** from registry arrays |
| "500k ops @ 4.25M/s" | **Fails** — measured 7,586 ns/op (32.26×) |
| "Chat latency 207 µs" | Measured **p50 577 µs / p95 802 µs** |
| "3.24 T transistors" | **35.6 B compute** — storage cells had been summed in |
| "L3 = 32 MB" | Measured **64 MB** |
| "DDR antibodies armed" | `hwprobe` reports ARMED — actual *monitoring* unverified |
| Bare-metal core binding | **REAL** — verified per-core (3, 7, 11) |
| SHA-256 / HMAC | **REAL** — FIPS 180-4 + RFC 4231, 6/6, three-way agreement |
| TurboVec backend | **REAL** — 2,069 entries, zero segfault |

---

## 4. Measurement caveat — the host scheduler contaminates the tail

Measured over 600 sequential operations against the council server:

```
p50 = 0.71 ms      fast (<5 ms) = 559 / 600  (93%)
slow cluster (>=5 ms) = 41, median 15.55 ms
Windows scheduler quantum = 15.625 ms  ->  slow median = 1.00x the quantum
```

The slow band sits **exactly on the operating system's scheduler quantum**. That
means the latency tail is **the host, not the platform**: p50 stays flat while
occasional requests absorb a scheduling slice.

**Consequence:** on this host, a single-run mean or p99 is dominated by the OS
scheduler and must not be quoted as a platform property. Quote **p50** for
platform latency and treat the tail as environmental. This is why the same
workload produced mean/p50 of 2.6× on one sample and 4.2× on another — the ratio
itself is unstable, which is exactly why one mean is not a stable figure.

## 5. How the system checks itself

```bash
tools/surface_diff.py             # shell-without-body + never-compiled files
tools/security_differential_test.py  # false negatives vs the original regexes
cargo test --test mesh_auth_crossverify   # three-way HMAC agreement
cargo test --test mesh_transport_test     # codec round-trips + rejection
```

Microkernel routes that exercise real primitives (not endpoint timing):
`/k3/bench`, `/k3/reduce`, `/k3/sram/patterns`, `/turbovec/bench`, `/gpu`.

**Presence must be verified in the artefact, not inferred from a clean build.**
A trampoline once compiled but was absent from the binary because nothing
referenced it. Scan the deployed `.efi` for the bytes.

---

## 6. Standing rules this system imposes on itself

1. **Never fabricate a number.** If it was not measured, say so.
2. **Never let a display stand in for evidence.** A boot log printing a claim is
   not the claim being true.
3. **Correct at the source.** Errata preserve the original text.
4. **A port is not done until differentially tested** against what it replaces.
5. **State untested code as untested.**
6. **Do not cite the council as validation.** It is ceremonial until repaired.
7. **Distinguish a model from control.** Unified-transistor "pinning" models the
   pool; it does not drive silicon. Calling it hardware control would be the
   fabrication pattern this document exists to prevent.

---

## 7. Immediate priorities

1. **Repair or retire the council — PARTIALLY REPAIRED 2026-11-02.** Votes now
   depend on the proposal when retrieval is forced to quote: the same proposal
   its own rule 6 forbids went 14-0 approve with no citation, then 11-3 reject
   with 14/14 citing a source. Evidence: `evidence/COUNCIL_RETRIEVAL_GROUNDING_MEASURED.txt`.
   Standing rule 6 still applies: the council is not validation for correctness.
   The Mamba-3 SSM inside the council is ADVISORY, not load-bearing — two
   read-out attempts scored 0.458 (hash features) and 0.474 (nomic embeddings)
   leave-one-topic-out against a 0.455 always-majority baseline. Evidence:
   `evidence/SSM_READOUT_FIT_MEASURED.txt`, `evidence/SSM_EMBEDDING_ROUTE_MEASURED.txt`.
   Its trit runs on every verdict, is deterministic and auditable, and does not
   decide anything.
2. **Distribute the mesh key** and switch `probe_peers()` to the authenticated
   path — the fix exists but is not enforcing.
3. **Boot-test the bare-metal path.** Nothing post-`ExitBootServices` has run.
   Requires a human at the machine.
4. **Resolve `trm_companion_bridge.exe`.** Two builds, 84% byte divergence, no
   provenance.
5. **Investigate the duplicate module declarations.** 83 modules are declared in
   both `main.rs` and `modmap_all.rs`; modules holding statics (e.g.
   `global_memory`) would then have two independent copies.
6. **Resolve the two failing tests rather than skipping them.**
   `BaseAgent.spawn_subordinate` and `ProactiveOrchestrationLoop._scan_for_gaps` were
**retired v6 names, not missing features** (RESOLVED 19.026). Both capabilities exist in v18 under
renamed, *enhanced* signatures: `spawn_subordinate` -> `spawn_subagent` (`base_agent.py`, now taking a
`continuum_function` parameter) and `_scan_for_gaps` -> `_scan_plan_gaps` (`proactive_loop.py`, now emitting
`priority` and `source` fields). The two tests were stale and have been updated to the current API; the suite
is now **51 passed, 0 failed**. An earlier entry here asserted these were
   asserted and **do not exist** — designed-and-not-built functionality, which bears
   directly on how the swarm scales. Left failing on purpose.
7. **Do not describe TurboVec as preserving retrieval.** 16× compression is exact
   (921484/57673 = 16.0) but 1.58-bit **recall@1 collapses 100 % → 18.7 %** on a
   107-document corpus (chance 0.93 %). "16× smaller" and "works as well" are
   different claims; only the first holds.
8. **Label docking as a signal, not a classifier.** A blind preregistered holdout
   (n=38) gave ρ = −0.5766 with sensitivity 69.6 % / specificity 80.0 % at
   −9.0 kcal/mol — real predictive content, but a fifth of non-binders still
   false-positive. An earlier "no threshold works" conclusion was drawn from a
   32-molecule set and is superseded on the larger sample.
