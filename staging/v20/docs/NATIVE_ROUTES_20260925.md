# `/native` — the native modules as kernel routes, and the crates behind them

**Block 147 companion; extended by blocks 150–159.** `/native/modules` now returns **15** entries and the namespace serves **37** operations (the harvest list is empty — see `docs/NATIVE_HARVEST_20260925.md`; the load/fuzz round is block 159). The strips for `gpu_infinity_computer`, `chaos_engine`,
`expert_registry` and `persona_matrix` were bit-exact against their python oracles and wired into the python
modules, but they were still DLLs on a module path: an operator had no route to ask them anything, and the
kernel did not surface them at all. This round gives them a namespace.

**Since then the namespace took on the v18 crates.** `swarm-proof`, `swarm-drug`, `swarm-docking` and
`swarm-intel` are wired as Cargo path dependencies — the crate's own function decides and the kernel adds no
rule — so `/native/modules` now reports **nine** entries and the namespace serves **eleven** operations. The
counts, the rulers and the findings (including that `council_vote` can only ever return ACT) are recorded in
`docs/NATIVE_HARVEST_20260925.md`.

## The wiring rule (the same one `/sram` uses)

Every module is linked from the kernel's own source with `#[path = "../../src/<module>.rs"]`, so the route calls
the function the strip wraps. There is no second implementation to drift:

```rust
#[path = "../../src/gpu_infinity_computer.rs"] mod gpu_infinity_computer;
#[path = "../../src/chaos_engine.rs"]          mod chaos_engine;
#[path = "../../src/expert_registry.rs"]       mod expert_registry;
#[path = "../../src/persona_matrix.rs"]        mod persona_matrix;
```

| route | input | answer |
|---|---|---|
| `GET /native/modules` | — | the five linked modules with their **measured** source size and the parity figures read from each module's own artifact |
| `GET /native/gpu/architectures` | — | the architectures whose arithmetic this namespace serves |
| `POST /native/chaos/drop` | `{enabled,rate,sample}` | the drop decision (`sample` = the uniform draw, passed IN) |
| `POST /native/chaos/clamp` | `{rate}` | `requested` beside `stored`, so a caller cannot believe an out-of-range rate stuck |
| `POST /native/expert/resolve` | `{identifier,team:[…]}` | a row **index** + the resolved key; agents never cross the boundary |
| `POST /native/persona/check` | `{output,forbidden,max_violations,locked,secs_since_reset,window}` | violations, state, status, content |
| `POST /native/persona/prompt` | `{persona,axioms,forbidden}` | the prompt **body** (unsigned, see below) |
| `POST /native/gpu/pattern` | `{arch,size_mb}` | the pattern math for `fpga`/`neural`/`cellular`/`k3` |

`GET` on a POST route is a `405`; a missing parameter is a `400` naming it; an unknown sub-path is a `404`.

## Verified: every route answer equals the python oracle's answer

`scratch/probe_native_routes.py` drives the live server and compares against the **python modules of record**
(never the strip, which the route's code path already is):

| group | checks |
|---|---|
| census | 2 — five modules, every source measured > 0, every file present |
| chaos | 2 — 6 `(rate, draw)` pairs · clamp at `1.5 → 1.0`, `-0.3 → 0.0`, `0.42 → 0.42` |
| expert | 1 — 10 identifiers including `logic`, `** Logic (Reasoning Engine)`, `the archivist role`, `nope`, `"   "` |
| persona | 1 — 6 enforcement cases (violations, state, status, content) including locked-in-window and locked-past-window |
| prompt | 3 — body identical (585 B) · `sha3_256(route body) == the python signature` · the route declares it unsigned |
| gpu | 4 — `fpga` at 1/96/1024 MB · `neural(64)` · `cellular(96)` · `k3` |
| refusals | 5 — 405, 400 naming `rate`, 400 listing the four arches, 404, the architectures list |

**18 checks, 0 failed.**

One probe bug is worth recording because it is the doctrine's own failure mode: the first persona run compared a
route told `secs_since_reset = 0.0` against an oracle whose `last_reset_time` was still `0.0` **epoch** — an
elapsed of ~1.7 × 10⁹ s, so the oracle unlocked itself while the route stayed locked. Two implementations were
never in disagreement; two different inputs were. The probe now injects the same clock into both sides
(`m.last_reset_time = time.time() - secs`).

## Stated limits (on the payloads, not in a footnote)

- **The persona signature has one owner.** SHA3-256 is not implemented in this kernel tree, so
  `/native/persona/prompt` returns the unsigned body and names its owner (`persona_matrix.build_compiled_prompt`).
  Hashing the returned body reproduces the signature exactly — that is a check in the probe, not a claim.
- **The GPU figures are the design target's arithmetic** (RX 6700 XT, 96 MB Infinity Cache), not a measurement
  of this host's GPU. `/system` reports the host's own measured L3 (16384 KB) — a different quantity, and both
  are correct.
- **No speedup is claimed.** A route is a boundary crossing; these routes make native code reachable.
- **The LCARS UI does not surface `/native` yet** — the Vite dashboard has no tab for it, the same gap `/sram`
  has. The routes are reachable from the API and the proxy.

## Deployment

| | |
|---|---|
| built | 2,947,072 B, `sha256 4e7b04cd29f46f4c…` |
| deployed | `trm_uefi_kernel/host_council_server/target/release/host_council_server.exe` |
| swap | rename-aside-then-move (Windows locks a running exe's image; renaming it is allowed) |
| prior build | 2,857,984 B (`ef31419bf0f1241d`) kept aside, not deleted |
| proof | process start **13:42:52** > exe mtime **13:41:59** → the new image is the one mapped; `/health` 200 |

The proof reader itself was wrong twice before it was right, both times by mixing time frames: PowerShell's
`-UFormat %s` renders local time as if it were UTC (making a fresh process look four hours old), and CIM's
rendered datetime text is not stable — the same query returned `9/25/2026 1:42:52 PM` and
`Friday, September 25, 2026 1:42:52 PM`, and the second form made a single-format parse fall through while the
script reported a warning about a live process. It now asks for `yyyy-MM-dd HH:mm:ss` explicitly.

## P7 step 4 — the load/fuzz modules get routes (block 159)

The two modules that were path-included, declared in the census and *unrouted* now serve **11 operations**:

| operations | module | oracle |
|---|---|---|
| `POST /native/load/{reset,start,end,agent,summary,redistribute,capable}` | `src/cognitive_load_balancer.rs` | `swarm_v2/core/cognitive_load_balancer.py` |
| `POST /native/fuzz/{scan-lines,scan-text,scan-all,save-plan}` | `src/self_fuzzing_engine.rs` | `swarm_v2/core/self_fuzzing_engine.py` |

Namespace: **15 modules / 37 operations**. Ruler: `swarm_v2_artifacts/load_fuzz_parity_ruler.json`
(**65/0**), from `scratch/probe_load_fuzz_parity.py` — the python module and the route driven with the same
inputs, the clock pinned on both sides, fingerprints compared and not only counts.

The three defects that ruler caught before it went green — a `take_while()` that swallowed the closing quote
and silenced both literal rules, a save-plan payload whose count and list came from two different collections,
and a view that defaulted `last_active` to the construction clock — are recorded in
`docs/NATIVE_HARVEST_20260925.md`. So are the two corrected deploy rules: **when a supervisor owns the port,
copy first and kill second, then prove the swap by behaviour the old image cannot produce.** The proof line in
the Deployment table above (start time > mtime) is necessary but NOT sufficient under those conditions.
