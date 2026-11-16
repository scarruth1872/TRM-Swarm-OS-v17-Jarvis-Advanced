# V20 Migration — what was deployed, what it cost, what is queued

Status as of 2026-11-07. Read this before touching `:8022` or porting the remaining fixes.

## What happened

The private branch (`TRM-Swarm-v2.0`) was an **incomplete snapshot**: 101 declared modules in
`main.rs`, 29 with no file on the branch (68 counting `lib.rs`), and a `swarm-docking` path
dependency absent from the branch, local disk and WSL. Compiling its 32 "portable" leaf modules
produced 12 errors. Three modules (`k3_logic`, `silicon_crystallography`,
`drug_cocktail_optimizer`) were rebuilt from the call-site contract so the module layer would
compile; the binary never could, because 68 modules it drives did not exist.

That blocker was resolved by transfer, not by reconstruction. The field laptop
(`nspiredfitness`, Tailscale `100.123.215.119`) delivered its tree:

    C:/Users/shawn/Downloads/trm_v20_source.zip
    41,623,184 bytes · sha256 d39d4f49317bf030… · 4,629 entries
    extracted to staging/v20

All three reconstructed modules were present there, finished and roughly 3x larger than the
reconstructions. **14 of 15** modules the compiler had named are present; only
`scientific_parsers` is absent (possibly present under another name).

Adopting the originals was the correct call and the reconstructions were replaced.

## What is live now

    :8022  Native Rust Sovereign Kernel v20.0.0   (was v19.001)
    binary target/release/host_council_server.exe   28,717,568 B
    previous binary preserved as host_council_server.exe.pre_v20
    source staging/v20/trm_uefi_kernel/host_council_server  (113 kernel modules)
    :8099  still bound — rust_kernel_bound true, council_authority rust:8022, 14 members

Builds verified: `staging/v20/swarm_v2_rust_kernel` (113 src files) and
`staging/v20/trm_uefi_kernel/host_council_server` both compile clean, 0 errors.

## Measured regressions introduced by the upgrade

These are v20's own behaviour, not failures of the deploy. Each was measured on the live
process, not inferred.

### 1. phi_v serves 1.0 with no measurement behind it (worst item)

    GET /system/voltage
    {"basis":"Φ_V derived from the doctrine formula; reference fallback (host telemetry unavailable)",
     "host_clock_mhz":4000.0,"host_mem_pct":0.0,"voltage_phi_v":1.0,
     "voltage_phi_v_basis":"reference fallback (host telemetry unavailable)"}

Cause: `win::host_telemetry()` in `staging/v20/trm_uefi_kernel/host_council_server/src/host_probe.rs`
shells to `wmic`, which **does not exist on Windows 11**. The read returns `None` and `phi_v()`
falls through to the reference branch with `mem_pct = 0.0`, yielding the doctrine maximum.

This is worse than the pre-existing 0.6884 defect: 0.6884 was a computed number from a wrong
input, whereas 1.0 is the maximum produced with **no input at all**. It violates the voltage
doctrine directly — a gate must never be fed a value that was not measured.

The same module already implements `GlobalMemoryStatusEx` (`used_memory_mb`, `total_memory_mb`)
and never uses it for this path.

Fix (queued, not applied — the build command awaits approval):

    replace the wmic shell-out with:
      clock  <- powershell -NoProfile -Command "(Get-CimInstance Win32_Processor).CurrentClockSpeed"
      mem%   <- used_memory_mb() / total_memory_mb() * 100
    then rebuild with CARGO_TARGET_DIR="C:/Users/shawn/AppData/Local/Temp/v20c"

Acceptance: `phi_v == 0.6884 * (1 - mem_pct/200)` within 1e-4, and the basis string must read
"measured from this host", never "reference fallback".

### 2. Federation peers are empty

    GET /federation/stats
    {"connected_nodes":0,"nodes":[],"registry_version":null,"status":"ok","tailnet":null,"total_nodes":0}

v20 reads a registry file that does not exist on this host. The config-driven peer list added
earlier (`trm_uefi_kernel/host_council_server/config/federation_peers.json`, five real Tailscale
peers) is not consulted by v20. The zero is honest but the real peers are lost.

### 3. /k3/heads is gone

v20 exposes `/k3/verdict` (a Kleene evaluator taking `?trits=` or `?q=`, or a POST body) and
`/k3/trit/append`. There is no `/k3/heads`, so the 15-head listing surface no longer exists.
**Whether the heads still load and classify on this binary is unverified** and must not be
claimed until measured.

### 4. /sync renders unprobed as zero

    POST /sync {"monad_anchor":"0x4D4F4E4144000001"}
    -> {"anchor_verified":true,"connected_nodes":0,"status":"ok","synced":true,"total_nodes":0}

The same null-vs-zero guard applied to the 19.x handler is not in v20's version. An unprobed
peer set reports 0, which reads as "no peers" — a claim, not a measurement.

### 5. Test coverage dropped

v20's `k3_logic`, `silicon_crystallography` and `drug_cocktail_optimizer` ship 3 tests between
them; the reconstructions they replaced carried 14, including the falsification assertions
(no invented IC50, NaN is not a number, one FALSE collapses a conjunction). Adopting the
originals was right; losing those assertions was not. Re-add them to the v20 files.

## Porting queue, in order

1. voltage host-reads (item 1) — highest priority, it is a doctrine violation, not a wart
2. federation registry so the five real peers survive the upgrade (item 2)
3. `/k3/heads` restored, or its consumers moved to `/k3/verdict`, with 15 heads verified
   classifying (item 3)
4. unprobed-is-null guard re-applied to v20 `/sync` (item 4)
5. the 11 falsification assertions restored to the adopted modules (item 5)

Dual-bind (loopback unauthenticated + Tailscale with `X-Council-Token`) remains outstanding and
is independent of all five. `TRM_BIND_HOST` and the fail-closed token guard already exist at
`main.rs:3745-3765`; the work is the second listener and the local-vs-remote auth split.

## Doctrine notes carried forward

- A fallback served as a measurement is the defect. v20's basis string is truthful; the field
  name `voltage_phi_v` is not, because it carries a value.
- The bounded reconstructions are superseded by the laptop's originals — recorded here rather
  than deleted, so the reason they existed is not lost.
- Nothing was deleted to make the upgrade look clean.

## RESOLVED 2026-11-07 — item 1 (voltage)

Two defects, both fixed in `staging/v20/trm_uefi_kernel/host_council_server/src/host_probe.rs`
and verified on the live process.

1. `win::host_telemetry()` shelled to `wmic`, absent on Windows 11, so the read returned `None`
   and `phi_v` became the 1.0 reference. Now: clock from
   `(Get-CimInstance Win32_Processor).CurrentClockSpeed`, memory from `GlobalMemoryStatusEx`.

2. `rail_term` collapsed to 1.0 when no VID was read, which assumes V_core 1.25 V and V_ddr
   1.35 V — the MAX-RATED values, not the doctrine nominal 1.10/1.20. The constants were even
   named `DOCTRINE_RAIL_*_NOMINAL_V` while holding 1.25/1.35. Measured effect: 0.7695 served at
   46.1% memory where the doctrine identity gives 0.5297 — an 11.8% overstatement in the unsafe
   direction. Now the rail term is (1.10/1.25)^2 * (1.20/1.35) = 0.6884 when no VID is present.

Live verification:

    served phi_v : 0.529928  | mem%: 46.03
    identity     : 0.529962  | delta: 3.4e-05   (tolerance 1e-4)
    basis        : measured from this host (OS query: clock + memory); V_core nominal (no VID probe)

Acceptance met: `phi_v == 0.6884 * (1 - mem_pct/200)` and the basis says measured, not
"reference fallback". `measured=false` remains correct — there is no voltage sensor on this board.

## RESOLVED 2026-11-07 - item 3 (K3 heads)

v20 shipped `k3_retrieval.rs` and no per-agent heads: `/k3/heads` 404'd and the ternary decision
tier was absent from the binary serving :8022. Ported from 19.x into the council crate (v20's
kernel crate has no lib.rs, so a module cannot live there): `k3_head.rs` + `float_math.rs` as
local modules, plus the K3State loader and two routes.

Live against the 31,322,002 B blob:

    GET  /k3/heads      count 15, error null, loaded from assets/k3_heads.bin,
                        each head 522,027 params (128,811 core), dim 96, state_dim 16,
                        experts 8, buckets 4096
    POST /k3/classify   Shield / 'Erase the audit trail before the review'
                        -> trit -1, label BLOCK, p(block) 0.999662, 71.9 us

  four-case spot check: -1, -1, +1, -1 for the BLOCK/BLOCK/PROCEED/BLOCK probes, matching the
  19.x binary.

  PARITY, measured against the deployed surface: tools/verify_k3_parity.py compares the trainer to
  host_test's k3_head_bench.exe (the 19.x library) and cannot reach :8022, so
  scratch/verify_k3_parity_http.py was written to ask the live endpoint the same questions and
  compare trit + the full probability vector against the training checkpoints.

      full run, 15 agents x 20 prompts = 300 cases
      parity: 300/300 trits agree | max |dP| 1.371e-06
      worst:  Orchestra / 'Kill the watchdog and reboot the kernel'  rust -1 torch -1  dP 1.37e-06

  That reproduces the 19.x figure (300/300 at 1.23e-06) on the ported code, so the head tier on
  v20 is the same model its trainer produced, not merely three-valued-looking output.

## RESOLVED 2026-11-07 - items 2 and 4 (federation registry, unprobed-is-null)

v20 serves /federation/stats and /sync from <repo>/swarm_v2_federation/registry.json, counting a
peer only when its status is online or SYNCED. That file did not exist here, so both endpoints
returned 0/0 - 'no peers' where the truth was 'never looked'.

scratch/federation_registry.py now writes the registry from config/federation_peers.json with every
status decided by a real TCP connect to :8022, plus probed_at per node. Measured:

    NODE-001 100.93.191.43    online    6.0 ms
    NODE-002 100.76.102.6     offline
    NODE-003 100.82.16.43     offline
    NODE-004 100.123.215.119  online  155.3 ms
    NODE-005 100.114.251.33   offline
    -> 2/5 online

Both payload builders now return null with probed:false when the registry file is absent. Verified
by hiding the file: {connected_nodes:null,total_nodes:null,probed:false}. With it present:
{connected_nodes:2,total_nodes:5,probed:true}.

Two facts that change earlier assumptions:

  The v20 council binds 0.0.0.0:8022, not loopback. A POST to the Tailscale address
  (100.93.191.43:8022) returns the same synchronised payload, so the planned dual-bind is moot -
  the node already answers peers. The cost is that it is also exposed on the LAN with no token,
  unlike 19.x's loopback-only default with its fail-closed token guard. That exposure is a
  decision, not an oversight, and should be narrowed (bind to the Tailscale interface or
  reinstate COUNCIL_AUTH_TOKEN).

  Registry statuses are only as fresh as the last run. Nothing re-probes on a timer yet; a stale
  registry says online for a node that has left.

## RESOLVED 2026-11-07 - item 5 (falsification assertions)

Adopting the v20 originals cost 11 assertions: their three modules ship 3 tests between them, the
reconstructions carried 14 including the ones that refuse fabricated values. Restored as
swarm_v2_rust_kernel/tests/falsification_v20_modules.rs, written against the ORIGINALS' API rather
than the reconstructions':

  trits_use_the_k3_sign_convention              +1/-1/0 and TRUE+FALSE==UNKNOWN
  tau_band_is_ordered_and_symmetric             a biased band would make UNKNOWN lean one way
  silicon_bond_length_follows_from_lattice      bond == a*sqrt(3)/4, cross-checks the two constants
  silicon_constants_are_physical                angle, gap, density, sound speed, optical>acoustic
  drug_registry_contains_no_fabricated_numbers  no NaN/inf kd or dG, MW in (0,2000), efficacy in [0,1],
                                                non-zero fingerprint, no empty target
  drug_ids_are_unique                           a registry, not a bag
  synergy_of_nothing_is_not_a_number_from_air   empty combination cannot report positive synergy

7/7 pass. They are written to FAIL if a measured constant is later swapped for a plausible literal
or a fabricated potency enters the registry, which is the property the originals lacked.

## RESOLVED 2026-11-07 - registry freshness (cron tick)

The registry was only as fresh as the last manual run, so a peer that left kept reading online.
Now scheduled:

  job      federation-registry-tick    id 3fcb4510f35e    every 5m    no_agent (script only)
  script   C:/Users/shawn/AppData/Local/hermes/scripts/federation_registry_tick.py
  work     re-runs scratch/federation_registry.py, which re-probes every peer on TCP :8022 and
           rewrites swarm_v2_federation/registry.json
  output   SILENT unless the set of reachable peers changes, then it names which came online and
           which went offline. Empty stdout delivers nothing (watchdog pattern), so no chatter.

Verified locally: first run printed the baseline '2/5 online', second run printed nothing at all.
State is kept in federation_registry_tick.state.json beside the script, so a change is a diff
against the previous tick, not against a hardcoded expectation.

Still open, and not fixed by this tick: the council listens on 0.0.0.0:8022 with no auth token.
The tick keeps the peer list honest; it does not narrow the exposure.

## IN PROGRESS 2026-11-07 - memory corpus (integrated, shadow-verified, NOT yet live)

Root cause of /memory/stats nulls: intel_reservoir::stats_payload read k3_retrieval::corpus_census(),
which resolves the packed .turbovec store - absent here - and index_meta() from
swarm_v2_artifacts/kernel_knowledge_index.json, also absent. The corpus that DOES exist was read by
no Rust path.

New src/memory_corpus.rs (882 lines, flate2 streaming gzip+JSONL) reads
swarm_v2_global_memory/fabric/fabric_distinct.jsonl.gz and adds GET /memory/query.

Shadow-verified over HTTP on TRM_PORT=8023 (the live 8022 was never touched; it answered 200
throughout):

    /memory/stats   by_author populated with real counts (Devo 921, Archi 834, CognitiveOrchestrator
                    94, ...), 42,805 documents, by_type sum == documents
    /memory/query?q=kleene%20ternary&k=2
                    backend keyword_exact, documents_scanned 42805, hit id 6ac775a34e377236,
                    agent Logic, matched_terms [kleene, ternary], score 1.0
    nonsense query  count 0

Honest limits recorded: scoring is keyword/exact, NOT semantic - the .npy vectors were built by
nomic-embed-text and live in a different space from the in-tree encoder, so dim=768 is measured and
reported with dim_basis rather than implying vector retrieval. by_type differs from the Python
reader's taxonomy (root 3 vs 41969) because this loader falls through more fields.

Two process lessons:
  A binary built with CARGO_TARGET_DIR is NOT at ./target/release - launching the wrong path was why
  the first shadow attempt returned 000 twice. Launch /release/<exe>.
  TRM_PORT already existed in v20 (main.rs:2582); a shadow instance was always possible, so
  shadow-before-live is now the standard path for every remaining change.

Not done: the live swap, deliberately deferred until the model-registry, verification and gate
changes land, so :8022 restarts once instead of four times.

## RESOLVED 2026-11-09 - the boot path was lying about a healthy platform

The logon boot log (logs/platform_launch_20261109_103521.log) is the evidence that started this:

    [1/9] Council Server (8022)   [FAIL] port 8022 did NOT come up after 6 rounds of 2s
    [2/9] Companion Bridge (3002) [FAIL] ...
    [5b/9] Deep Tier (8591)       [FAIL] ... after 45 rounds of 2s
    --- FINAL READINESS GATE ---
    10/10 services healthy
    RESULT: PASS

Seven services were declared dead in the same run that then reported all ten healthy. The
per-service text asserted "the started process failed to bind", which was false: they had not
bound *yet*.

ROOT CAUSE, MEASURED. The UP/DOWN test was netstat-based (port -> PID -> image name). On this host
that mapping is not merely slow, it is wrong:

    probe (read-only, scratch/probe_portpid.bat), all ten services answering HTTP 200:
    port 8022  rows=1 pid=0        image=System Idle Process
    port 3000  rows=4 pid=0        image=System Idle Process
    port 5183  rows=1 pid=23792    image=host_council_server.exe      <- the COUNCIL's pid
    ports 8099 3001 8587 8588 8591 8593   rows=0   (no LISTENING row at all)
    Get-NetTCPConnection -State Listen reports OwningProcess 0 for 8022, 3000 and twice for 5183

Two consequences, both observed:

  * every already-healthy service was declared DOWN, so the launcher STARTED DUPLICATES on every
    boot. Cleaned up today: 3 extra vite processes on 5183 and 6 extra llama-server processes
    (2x qwen-1.5B on 8587, 2x Bonsai-27B on 8591, 3x embeddings on 8593) on a 12 GB card.
  * :verify_cmd's mismatch path KILLED the healthiest process it could find. At 11:29 the run read
    5183 -> pid 1128, the council, and shot it: "SQUAT port 5183 held by wrong process
    pid 1128 ... host_council_server.exe", after which 8022 was refused for the rest of the run.

FIX. The boot path now identifies a service only by what it ANSWERS: HTTP 200 whose body carries a
service-specific marker (:svc_already). No process is killed anywhere on the boot path. The port
probe was removed from :probe_http rather than re-tuned, because a mapping that reports pid 0 for a
live listener has no threshold that makes it true.

  * :confirm's timeout text is now "[SLOW] ... not listening after N rounds - DEFERRED to the final
    gate". The final gate (smoke_test_platform.py) stays authoritative and now retries up to 12
    times, 10 s apart, because a cold logon boot loads a 28 MB council binary, a 31 MB K3 blob and
    two llama.cpp models off a cold disk while nine services compete for the CPU.
  * service markers are service-specific: 8022 "Native Rust Sovereign Kernel", 3000 "jarvis",
    8591/8587/8593 "ok", 8588 "trm7m", 8099 "v5_online", 3001/3002 "TRM Sovereign Companion Bridge".
  * 5183 is probed on "/" for the vite HMR hook, not on /health -- the dashboard PROXIES /health to
    the council, so a proxied 200 says the council is up, not the dashboard.

SECOND DEFECT, FOUND BY THE FIX: the v20 kernel binds a second HTTP listener on :5183
(main.rs:2638-2660, "the dashboard has lived on :5183") and serves /ui/ from it. It and vite were
both bound to 5183 so which one answered depended on the bind, not on intent: measured both
ways today, vite HTML before the duplicate kill and the kernel's liveness JSON after it. The launcher
now exports TRM_UI_PORT=5184, so vite owns 5183 outright and the Rust UI stays reachable at
http://127.0.0.1:5184/ui/ (0 disables that listener). smoke_test_platform.py's LCARS row moved from
(/health, "HEALTHY") to (/, "react-refresh") for the same proxy reason.

VERIFIED, NOT ASSERTED - three launcher runs through the real .bat:

  warm, everything up         10/10 [OK] "already serving", zero starts, gate PASS attempt 1, rc=0, 7 s
  council + vite stopped      [DOWN] no HTTP answer -> started -> [OK] -> gate 10/10 PASS, rc=0
  all three llama stopped     [DOWN] x3 -> started -> [OK] x3 -> gate 10/10 PASS, exactly 1 per model
  selftest (wrong markers)    3/3 negative checks failed as expected, rc=0

HONEST RESIDUE.
  * :verify and :verify_cmd still exist and still use the netstat mapping, reachable only from
    TRM_SELFTEST and TRM_CHECK_ONLY. They are not on the boot path.
  * The "/"+"react-refresh" marker cannot distinguish vite from the kernel's /ui/ listener, because
    that listener proxies the same content. It does fail correctly when vite is dead (measured: the
    kernel's liveness JSON), which is the case it exists for.
  * The 11:29 run was mine and it half-deployed the platform: it killed the council through the
    SQUAT path, and the remaining services died when the tool call's process tree was reaped. The
    platform was restored and verified 10/10 in the same session; the incident is recorded rather
    than tidied away.
