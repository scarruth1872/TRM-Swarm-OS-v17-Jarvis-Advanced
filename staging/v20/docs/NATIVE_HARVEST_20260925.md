# The native harvest — four crates wired, nine modules, eleven operations (2026-09-25)

**Blocks 150–153 companion.** `/native` began as four kernel modules with bit-exact strips and no crate behind
them. This round made it the platform's *live* surface: four v18 crates that no route reached now answer on it,
each with a ruler that compares the route against an oracle written from the crate's own source.

## The rule that decided every wiring

A live Cargo path-dependency is the module of record. The crate decides; the kernel adds no rule, clamp,
default, threshold or correction at the boundary. Nothing is ported — a copy is a divergence waiting to happen.
The census at `/native/modules` measures each crate's **own source bytes at request time** and reads the parity
figures from that module's own artifact, so a stale number cannot be served from a literal.

| crate | routes | source measured | ruler |
|---|---|---|---|
| `swarm-proof` (P2) | `POST /verification/proof` | `wsl_crates/trm-rust/swarm-proof/src/lib.rs` | `verify_p2_wired_proof.py` **9/9** |
| `swarm-drug` | `POST /native/drug/triage`, `/native/drug/sdf` | `.../swarm-drug/src/lib.rs` | `verify_drug_wiring.py` **11/0** |
| `swarm-docking` | `POST /native/docking/score-pair`, `/native/docking/dock` | `.../swarm-docking/src/lib.rs` (7,505 B) | `verify_docking_wiring.py` **11/0** |
| `swarm-intel` | `GET /native/intel/agents`, `POST /native/intel/{route,vote,decide,ensemble}` | `.../swarm-intel/src/lib.rs` + `.../ensemble_council.rs` | `verify_intel_wiring.py` **17/0** |

Kernel modules (`sram_pattern_computer`, `gpu_infinity_computer`, `chaos_engine`, `expert_registry`,
`persona_matrix`) stay linked from the kernel's own source with `#[path]`; `/native/modules` reports **9**
entries and eleven operations. `GET` on a POST route is `405`; a missing parameter is a `400` naming it; an
unknown sub-path is a `404` listing the routes.

## Findings recorded, not smoothed

- **`council_vote`'s verdict is a constant ACT.** Its signals are `[domain-fit, 0.9, 0.7]` at τ = 0.35 and
  `swarm-k3::collapse` maps `v > eps → 1`, `v < -eps → -1`, else `0`. Domain-fit is only 0.0/1.0/2.0 and the
  other two signals are positive constants, so the trits can only be `[0,1,1]` or `[1,1,1]` — measured exactly
  those two shapes over ten tasks. `pros ≥ 2` and `blocks == 0` in every case: **BLOCK and ANALYZE are
  unreachable**, and a council verdict of BLOCK from this path is a defect signal, not a decision.
  `ensemble_vote` cannot reach −1 either (only the field signal may be negative, so `blocks ≤ 1 < pros`).
- **The ensemble field is the fallback here.** `EnsembleField::load()` reads absolute WSL paths
  (`/home/shawn/trm-rust/field_qwen.bin`, `field_bonsai.bin`) that do not exist on this host, so the field is
  the crate's deterministic 32-dim fallback. The route returns `from_gpu: false` and says why; the fallback is
  never presented as a GPU capture.
- **`swarm-drug`'s `tau` is declared and never read** (the compiler says so on every build) — same class as
  `persona_matrix`'s `required_lexicon`.
- **`swarm-docking`'s H-bond ramp is the crate's own correction**, documented in its source: the old ramp
  saturated at 1 from d = −0.2 and still paid 0.4 beyond contact, which drove a pose Vina scores at −7.31 to
  −11.16. The route reports what the crate computes; the correction belongs to the crate.

## P7 step 2 — the mirror was NOT retired, on evidence

The duplicate census found `swarm_v2_integrated/` = 15,410 files with **13,888 byte-identical to a copy outside
it** (820 MB; 765 MB of that is one file, `_peerpath_inventory.json`), 1,522 with no twin, and 62 live files
naming the tree. A rule of "identical twin **and** path not named by a live file" selected 13,888 files — and
then the consumers were read: `artifacts_status.json` records the artifact pipeline's `integrated_path` **inside
that directory**, the dashboard's `ModuleMatrix.jsx` displays it as the Monitored Directory,
`module_registry.json` registers it, and `nexus_usage_example.py` imports from it. Byte-identity is not
redundancy when the directory is a live destination. **Nothing was moved**; the decision and its evidence are
block 151. (The scan itself had to be fixed first: v1 hashed the repo twice and concatenated every candidate
into one string — 21 minutes, no output; v2 reuses the census's duplicate clusters and bounds the reference test
to the files that name the mirror: seconds.)

## Deploy rules (learned the hard way, now the standard)

1. **`CARGO_TARGET_DIR` decides where the build lands** — export it in the *same* command as the build and the
   swap; it is not guaranteed to persist between shells. (It points at the Hermes scratch dir here, not the
   crate's `target/`.)
2. **Stop the process LISTENING on the port** (`netstat`), never a remembered pid — a later process can hold the
   port, and the deploy then silently keeps the old image serving.
3. **Content-guard the candidate binary** before swapping: the route strings must be present. A stale artifact
   offered as the new image was refused by exactly this check.
4. **Rename the running exe aside, then copy** — replacing a mapped exe in place fails (`WinError 5`).
5. **The proof is the serving process's start time AFTER the deployed exe's mtime**, plus `/health` — **except when a supervisor owns the port** (see rule 6): then that ordering proves nothing about *which bytes* are mapped, because the supervisor can respawn the old image inside the swap window.
6. **Copy first, kill second.** `swarm_supervisor.py` (python, pid 29168) respawns `host_council_server.exe` on the port within seconds of it dying. Kill → copy leaves a window where the supervisor restarts the OLD file; copy → kill cannot, because the new bytes are already at the path when the respawn happens.
7. **Prove the swap by BEHAVIOUR the old image cannot produce**, not by a strings-grep on the binary. `grep -a Orchestra` proves nothing — other modules contain that word. The load/fuzz round's proof was `roster_len` 12 (the old image's roster is 14) and the two rules that only exist in the new image firing (`BUFFER_OVERFLOW`, `HARDCODED_SECRET`). Two false greens on 2026-09-26 came from skipping this.

Correction on record: the `swarm_supervisor` was never down. It runs as `python.exe`, so
`Get-Process swarm_supervisor` matches nothing — a wrong-name measurement, not a defect. PID 20484 has been up
since 07:50:45 with all three services ONLINE. It is also the reason rules 6 and 7 exist: on the load/fuzz
deploy it respawned the old image at 09:19:27, three seconds before the copy landed at 09:19:28.889, so the
"new" process (pid 17432) served the old code while the file on disk was correct — a live process is not the
same measurement as a correct file.

## P7 step 3 — the harvest completes: fit, then brain (blocks 154–155)

Two more crates were wired the same way (path dependency, crate decides, no port):

- **`swarm-fit`** — `POST /native/fit/{encode,verdict,serve}`. Its verdict reaches **all three** outcomes
  (`BOOKED`, `BLOCKED`, `ANALYZE`), and its `serve` path performs a real NAND append through the crate's own
  `swarm_paths::resolve` — **offset 64, `byte_identical: true`**, with the ruler checking the *file on disk*
  (`<temp>/trm_fit_ruler_file_check.k3`, 68 bytes) rather than only the route's own flag. Ruler **15/0**; the
  oracle reproduces the crate's FNV-1a/mix encoding u64-and-all (wrapping rebuilt by masking) and matches
  trit-for-trit over 12 encodings plus 72 verdict cases at three taus.
- **`swarm-brain`** — `POST /native/brain/{connectome,neural,collapse,switch}`, three source modules measured
  separately. Ruler **19/0**; the oracle reimplements *both* PRNGs, the sparse firing selection, the K3 collapse
  and **all four architecture interpretations** (K3, CA rule 30, Ising release, NAND pass-through) and compares
  outputs *and the crate's own notes*. Measured: hub trits 80,000 for 1e6 neurons at a 1% hub fraction; neural
  weights 320,000 with packed 80,000; firing **100/10,000 = 1.00%** against the crate's sparse rule;
  `switch_loop` returning all four architectures in 3.1 µs.

Two things worth keeping:

- **The crate's labels are the specification.** `pattern_switch.rs:43` writes `trits.len() - t` in the FALSE
  slot, so the crate's "FALSE" bucket also absorbs the zeros. My first oracle rendered `count(-1)` and failed
  the ruler; the *oracle* was wrong. Reproduce the expression the source actually contains, not a tidier one.
- **`neuron_collapse` can return 0** (a 3-true/3-false tie). The verdict space here is genuinely three-valued,
  so the constant-verdict property recorded against `swarm_intel::council_vote` (block 153) is specific to that
  crate's signal shapes and must not be generalised.

**The harvest list is now empty** — `swarm-proof` (P2), `swarm-drug`, `swarm-docking`, `swarm-intel`,
`swarm-fit`, `swarm-brain` are all wired, and `/native/modules` returns **fifteen** entries (the
`cognitive_load_balancer` and `self_fuzzing_engine` port is block 159). The LCARS tab is now in place too: the
dashboard forwards `/native` and `/sram` (it did not before) and tab **31 NATIVE SURFACE** renders the census
and an operation console driven entirely by `GET /native/modules` and `GET /native/routes`. What remains in P7
is not harvesting: the `host_tranche` residual port candidates (closed by block 159) and WSL reconciliation
step 4 (closed by block 160 — the WSL side's role is decided by measurement, and
`swarm_v2/core/libswarm_kernel_ops.so` now exists; see `docs/WSL_STEP4_KERNEL_OPS_20260926.md`).
`host_tranche` was classified on 2026-09-26 (block 158): it is the 2026-08-28 skeleton tranche of the
Python-to-Rust port, never compiled and never reachable, superseded as a source of record by the kernel's own
same-named modules and KEPT (no byte-identical twin, so the retirement rule blocks retiring it). What the port
still owes is now measured rather than implied -- 123 of 232 python names have a live counterpart, six
residual candidate groups remain: `docs/P7_HOST_TRANCHE_CENSUS_20260926.md`. Of those six, **groups 1 and 2
were the only real missing behaviour** (the fuzzing file-scan and the balancer's live metrics); both closed in
block 159. The other four are naming and design differences.

## P7 step 4 — the residual gaps close: the load/fuzz port (block 159)

`cognitive_load_balancer.rs` and `self_fuzzing_engine.rs` were path-included and *declared* in the census while
having no route — the "declared not measured" class itself. They now serve **11 operations**
(`POST /native/load/{reset,start,end,agent,summary,redistribute,capable}`, `POST
/native/fuzz/{scan-lines,scan-text,scan-all,save-plan}`) over the same files the kernel compiles, taking the
namespace to **15 modules / 37 operations**.

The oracle is `scratch/probe_load_fuzz_parity.py` → `swarm_v2_artifacts/load_fuzz_parity_ruler.json`:
**65 checks, 65 passed, 0 failed**, driving the python source module and the route with identical inputs and
with the clock pinned on *both* sides (the source reads `time.time()`; the route takes `epoch_s`).

- **The port is of the source, not of the neighbourhood.** The rule set is the source's 15 `VULN_PATTERNS` as
  predicate functions over a line, in dict order — a needle table would silently drop every pattern shaped
  `A ... B`. The balancer's roster is the source's **12**-role `_capabilities` dict, not the kernel's 14-role
  council; porting the council would have been porting the wrong module's roster.
- **Three defects the ruler found before it went green.** (1) `take_while()` consumes and *discards* the first
  non-matching char, so the closing-quote check never saw the quote and **both literal rules could never fire**
  — a silent 0-finding rule that a count-only oracle would have passed. (2) The save-plan route reported `new`
  (a filtered count) beside `new_fingerprints` (an unfiltered list): one payload, two collections; the filter
  now runs once and both outputs come from it. (3) The view defaulted `last_active` to the construction clock
  where the source's dict defaults to `0.0`.
- **Seven limits are declared in the ruler, not hidden**: the line-splitting set (`str::lines` vs
  `splitlines`), Unicode vs ASCII case folding on the two distinct paths, the `\w` definition, `scan_all`'s
  `time_s`/`throughput` deliberately **absent** (the module has no clock and will not put a number where a
  measurement belongs), the source's `antibody_id` being process-salted (`hash()`) so only its shape is
  comparable, `RACE_CONDITION`/`USE_AFTER_FREE` containing a newline and so never firing in a per-line loop
  (carried as explicit false predicates so the names resolve), and four declared extra keys on the view.

Deployment followed the corrected rules 6 and 7 above: copy-then-kill, and the proof was behaviour the old
image cannot produce (`roster_len` 12, not 14).

## Not claimed

The routes relay the crates' own results; they do not make the platform *do* anything new. A mapped
device-visible Vulkan buffer is not Infinity Cache (`docs/VULKAN_PATH_20260925.md`). The fifteen modules are
reachable, ruler-gated, and now rendered in the dashboard's tab 31, but they are not adopted by the kernel's own
decision paths. That tab was verified by compile and fetch path, not by an observed browser paint -- see block
156 for that limit.
