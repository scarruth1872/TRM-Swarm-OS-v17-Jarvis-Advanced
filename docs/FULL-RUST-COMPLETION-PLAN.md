# FULL RUST PLATFORM — COMPLETION PLAN

Derived from measurement on 2026-11-06, not from documentation claims. Every number below
was read from the running host or counted from source in this session. Nothing here is
scheduled on the strength of a doc saying it is done.

---

## 0. Definition of done

The critical path is Rust. Operationally:

  - No Python process and no Node process is required for the platform to answer a request,
    run an agent turn, retrieve a memory, or report telemetry.
  - One command starts the platform and its own supervisor proves each unit is alive.
  - Every number any endpoint publishes is read from the host or from a store, with a
    falsifiable source. No literals.
  - Python remains only as tooling and as a test oracle, never as a serving tier.

Explicitly NOT part of "done":
  - Removing llama.cpp. Model weights need a C/C++ engine; the question is whether it is
    linked in-process or spoken to over HTTP (decision D1).
  - Training. This plan does not depend on any checkpoint being good.

---

## 1. Measured baseline (2026-11-06)

SERVICES NOW RUNNING — 10/10 healthy, one of them is why this plan exists

  port  process                     language   role                              routes
  8022  host_council_server.exe     Rust       council, K3 heads, fabric, arc     148 entries
                                               voltage, telemetry                 (26 exact + 122 prefix)
  8099  python -m swarm_v2.app_v2   Python     council API v5, kanban, artifacts, 277 routes
                                               learning, synthesis, mesh, memory  (9 inline + 268 router)
  3000  node / tsx server.ts        Node/TS    Jarvis backend                     47 routes
  3001  python trm_companion_proxy  Python     companion proxy                    small
  3002  trm_companion_bridge.exe    Rust       companion bridge                   small
  5183  vite dev server             Node       LCARS dashboard (UI)               n/a
  8587  llama-server.exe            C++        qwen2.5-1.5b                       n/a
  8588  python tools/trm_server.py  Python     455M ARC TRM wrapper               100 LOC
  8591  llama-server.exe            C++        Bonsai-27B deep tier               n/a
  8593  llama-server.exe            C++        nomic embeddings                   n/a

NON-RUST CODE THAT MUST BE ELIMINATED FROM THE CRITICAL PATH

  swarm_v2/routes/*.py            4,865 LOC   268 routes   the real porting mass
  swarm_v2/app_v2.py                770 LOC     9 routes + 6 mounted routers
  swarm_v2/core/*.py             51,808 LOC              the agent/logic layer behind it
  Jarvis-Advanced-main/server.ts  7,029 LOC    47 routes
  tools/trm_server.py               100 LOC              wrapper around the 455M model
  trm_companion_proxy.py          (small)                 proxy tier

RUST ASSETS ALREADY PRESENT — this is why the plan is assembly, not a rewrite

  openfang (absorbed workspace, 14 crates, Cargo.lock present, rustc 1.99.1)
    VERIFIED 2026-11-06: `cargo check --workspace --release` finished in 3m 12s with exit 0.
    Every crate checked in dependency order — memory, skills, extensions, channels, runtime,
    kernel, api, cli — so the workspace compiles on the installed toolchain (1.99.1), which
    is the same toolchain that builds host_council_server. One future-incompat warning
    (imap-proto 0.10.2), no errors.
    openfang-runtime   51,734 LOC  agent_loop, drivers/, mcp, tools/host_functions,
                                   model_catalog, routing, compactor, context_budget,
                                   embedding, sandbox, process_manager, a2a
    openfang-kernel    20,473 LOC  supervisor, scheduler, cron, event_bus, heartbeat,
                                   registry, approval, auth, capabilities, metering, workflow
    openfang-api       20,371 LOC  172 routes, openai_compat, ws, sessions, webchat
    openfang-memory                semantic, substrate, knowledge, consolidation, session
    openfang-channels  33,124 LOC  external channel adapters
  trm_uefi_kernel      36,545 LOC  208 files: host_council_server (HTTP), trm7m (1,635 LOC),
                                   k3_head, k3_ternary_sram, voltage, nand_persistence,
                                   host_tranche, host_fanout, companion bridge
  swarm_v2_rust_kernel            trm_swarm_boot.exe v19.001: LCARS compositor, input driver,
                                   agent_runtime, council, mesh_discovery (UDP 8022),
                                   nand_persistence. No HTTP surface at all.

GAPS THAT ARE MEASURED FACTS, NOT OPINIONS

  G-A  277 Python routes on 8099 with 51,808 LOC of core behind them.
  G-B  47 Node routes on 3000, 7,029 LOC.
  G-C  The 455M TRM is served by a 100-line Python wrapper; the Rust TRM (trm7m.rs) is a
       different, smaller model (STATE_DIM 462, 3 blocks, 3,910,833 params) and its
       checkpoint is seed-42 random. There is no Rust path to the 455M weights.
  G-D  Fabric retrieval runs in Python (TurboVec JSONL + semantic_fabric, 42,805 vectors).
       The Rust side publishes literals at /memory/stats (5,420 memories, 241,800,000 trits)
       against a live store of ~130k rows.
  G-E  No llama.cpp FFI anywhere (0 matches in Cargo.lock). Inference is HTTP to
       llama-server, so "no HTTP" is currently false even for Rust callers.
  G-F  Voltage gate is closed by design (no sensor; measured=false). Heavy fabric endpoints
       refuse: /k3/sram/patterns -> REFUSED_VOLTAGE_GATE.
  G-G  16 gguf_*_trits.bin (~12 GB) have no reader; only one genuinely loadable ternary
       model exists (Bonsai-27B-Q1_0).
  G-H  Startup is a 21 KB .bat with service-specific knowledge; the kernel binary cannot
       supervise anything because it has no service-management role.
  G-I  /system/resources still carries optical_phonon_thz 15.53 with no source.

---

## 2. Target architecture

Two binaries own the machine; everything else is a library inside them.

  TRM SWARM OS (trm_swarm_boot.exe)            — the console and the supervisor
    LCARS compositor + input driver (exists)
    supervisor: start/stop/prove each unit (port openfang-kernel::supervisor into it)
    mesh discovery UDP 8022 (exists)
    NAND persistence (exists)
    reads the platform's own health and refuses to render "OPERATIONAL" unless it is

  TRM COUNCIL / SUBSTRATE (host_council_server.exe)  — the authority
    council + PBFT (exists, 14 members)
    K3 ternary heads (exists, 15 heads, 522,027 params each, parity 300/300)
    voltage doctrine (exists, now reads the host)
    TRM inference host (extend: 455M path, see P3)
    fabric: dedupe + vectors + query, in Rust (G-D)
    telemetry: every number read (G-I, G-H)

  Embedded libraries, linked not served:
    openfang-runtime  for agent loops, tools, MCP, context management
    openfang-memory   for semantic store + consolidation
    openfang-api      for the OpenAI-compatible surface and websocket sessions
    openfang-kernel   for cron/scheduler/events/approval

  llama.cpp: linked in-process via FFI, or kept as a local sidecar (decision D1).
  Python/Node: gone from the serving path.

Why this shape: the platform already has two Rust binaries that work and 148 + 172 + 47
route surfaces' worth of behaviour between them. Rewriting the Python in hand would mean
retiring 51,808 LOC of behaviour with 51,734 LOC of new, untested behaviour. Linking
openfang gives tested behaviour (measured earlier: 881 runtime tests, 255 kernel, 84 api,
50 cli) and leaves only the TRM-specific glue to write.

---

## 3. Phases

Ordering rule: prove the instrument, then make the surface smaller, then move traffic, then
delete. No phase begins before the previous phase's acceptance test has passed and its
number is recorded in evidence/.

### P0 — Make "is it Rust?" measurable (0.5 day)

  Work
    1. Add `GET /system/processes` to 8022: for each expected port, the listening PID, the
       image name, and whether that image is a registered Rust build (map pid -> exe path ->
       known binary hash). This turns "the platform is Rust" into a fact per request.
    2. Add `tools/rust_share.py`: counts requests per port over a window and reports the
       fraction served by Rust binaries. It fails closed if a port cannot be attributed.
  Acceptance
    - /system/processes returns all 10 units with pid + image + rust=true/false.
    - rust_share.py reports a number for all 10 ports, or exits nonzero naming the port it
      could not attribute.
  Rollback: additive endpoints only; nothing else changes.

### P1 — Retire the Python API surface (3–5 days)

  Work, in this order
    1. Instrument 8099 for 24 h to rank its 277 routes by actual calls. Routes with zero
       calls are not ported; they are documented as retired with the measurement.
    2. Move the council-shaped routes to 8022 (they duplicate what the Rust council already
       answers). Each moved route gets a parity test that asserts identical JSON shape and
       values against the Python implementation running side by side.
    3. Port the survivors as thin Rust handlers over openfang-runtime/openfang-memory.
    4. Flip 8099 to a readonly shadow, then stop it, then remove it from the launcher.
  Acceptance
    - For every ported route: same request to old and new returns equal JSON after key
      ordering (parity test in CI, runs against both until the flip).
    - After the flip: `netstat` shows no listener on 8099, the launcher's smoke test passes
      10/10 without it, and a chat turn still answers end to end.
  Effort driver, measured: of the 268 route declarations extracted mechanically from
  swarm_v2/routes/*.py, **84 already answer on 8022** (kanban 19, mesh 9, batch_endpoints 9,
  swarm 7, artifacts/learning/system 6 each), 151 answer on 8099, and 117 answer on neither
  port (they need path parameters or a request body this probe did not supply, so they are
  not necessarily dead). See evidence/ROUTE_PARITY_8022_VS_8099.txt. The real porting mass is
  concentrated in swarm (38 of 65 Python-only), system (18 of 21), kanban (11 of 21) and
  changelog/insights/untapped, which answer nowhere and must be measured with parameters
  before any is retired. At 20–30 routes/day including parity tests, the remaining work is
  2–3 days rather than 9–13 — but only after the parameterised probe, which is now step 1b.
  Risk: silent behaviour drift on routes that call into swarm_v2/core. Mitigation: parity
  tests are differential, not golden-file, so they fail on any value change.

### P2 — Retire the Node backend (3–4 days)

  Work
    1. Same instrument-first step for 3000's 47 routes.
    2. Serve LCARS (5183) as static files from 8022 — it is a Vite build, not a service.
    3. Move the chat/session/memory endpoints to openfang-api, which already implements
       sessions, websockets and an OpenAI-compatible surface.
  Acceptance
    - 5183 serves from the Rust binary with no `node` process in the process tree.
    - Chat history, memory update and noospheric status return equal JSON to today's.
  Risk: the Jarvis UI may depend on dev-server HMR. Mitigation: build once, serve static.

### P3 — TRM inference in Rust (5–8 days, or 1 day if decision D2 lands)

  Two candidate paths, measured
    (a) Port the 455M checkpoint loader + forward pass to Rust alongside trm7m.rs. The
        existing trm7m.rs proves the shape of a TRM forward pass but is a different, smaller
        model with random weights, so it is a template, not a path.
    (b) Keep the weights on llama.cpp and delete the Python wrapper by having the Rust
        caller speak directly to the model engine (see D1).
  Recommendation: (b) for the 455M text model (it is a llama-architecture checkpoint), (a)
  only where the TRM-specific ternary forward pass is required for the K3 layers.
  Acceptance
    - Either: 8588 no longer exists and a Rust endpoint returns the same answer for a fixed
      prompt set as the Python wrapper did.
    - Or: the Rust loader parses the checkpoint, runs a forward pass, and matches the
      Python implementation's logits within 1e-5 on 20 fixed inputs.
  Risk: the checkpoint format is undocumented outside the Python loader. Mitigation: read
  the loader first; it is 100 lines and is the only specification that exists.

### P4 — Fabric in Rust (3–4 days)

  Work
    1. Port dedupe + content addressing (already correct in Python) to Rust using the
       fabric_manifest.json contract as the spec.
    2. Move the 42,805 x 768 vectors into MMAP'd Rust storage. Query in Rust; keep the
       Python semantic_fabric as the oracle in tests.
    3. Replace the /memory/stats literals with reads from that store.
  Acceptance
    - Rust query returns the same top-10 (same ids, same order) as the Python path for 100
      fixed queries.
    - /memory/stats total_memories equals the row count of the store file, asserted by a
      test that reads the file itself.
  Risk: float16 dot-product drift changing ranking. Mitigation: compare on ids, not scores,
  and document any id-order differences rather than rounding them away.

### P5 — Kernel supervision (2–3 days)

  Work
    1. Port openfang-kernel::supervisor into trm_swarm_boot: it starts each unit, watches
       it, and restarts on exit.
    2. Move the .bat's service knowledge into a config file the supervisor reads.
    3. The .bat becomes a one-line launcher or is deleted.
  Acceptance
    - Killing any unit by PID results in it being restarted within 5 s, logged with the old
      and new pid.
    - Killing the supervisor does not leave orphans: children die with it.
    - The TUI shows real per-unit state (already list the units; now they must be live).
  Why this matters now: today a fix can be sitting on disk while the running process serves
  the old binary (measured tonight with the voltage fix), and the launcher can report failure
  on a healthy platform.

### P6 — Decommission (1 day)

  Work
    - Remove Python/Node from the startup path; keep pytest as the oracle suite.
    - Record the final rust_share number and the process census in evidence/.
  Acceptance
    - Cold boot with no Python on PATH: platform reaches 10/10 (or the reduced unit set) and
      answers a chat turn, an agent turn, a memory query and a K3 classify.
    - process census shows zero python.exe / node.exe in the platform process tree.

---

## 4. Decisions needed from you

  D1  llama.cpp: link in-process (FFI) or keep as a local sidecar?
      FFI removes HTTP from the inference path and makes "no HTTP" literally true; it also
      means a crash in the engine takes the council with it. Sidecar keeps isolation.
      Recommendation: sidecar now, FFI later for the K3-heavy path.

  D2  Adopt openfang as the agent runtime, or port swarm_v2/core by hand?
      Recommendation: adopt. It is 51,734 LOC of tested Rust that overlaps the 51,808 LOC of
      Python almost exactly, and hand-porting it is the plan's single largest risk.

  D3  Who supervises: the kernel (P5) or the .bat?
      Recommendation: the kernel. The .bat has already produced two false verdicts tonight.

  D4  Fabric: openfang-memory substrate, or the K3 fabric in host_council_server?
      Recommendation: openfang-memory for retrieval, K3 for the ternary decision layer. They
      answer different questions and both already work.

---

## 5. Anti-goals

  - Do not delete the 130k-row fabric or the 42,805-vector index. They are the only knowledge
    the platform has; move them, never rebuild them blind.
  - Do not port routes nobody calls. Measure first; retire with the measurement attached.
  - Do not let "the platform is Rust" become a slogan. P0 exists so it is a per-request fact.
  - Do not touch the voltage gate's policy. It is closed for a correct reason (no sensor).
    Opening it requires a sensor or an accepted proxy policy, not a code change.
  - Do not do this in one heroic branch. Each phase flips traffic and is reversible.

---

## 6. Timeline and the honest total

  P0  0.5 d      P1  3–5 d      P2  3–4 d      P3  1–8 d (decision-dependent)
  P4  3–4 d      P5  2–3 d      P6  1 d
  Total 13.5–25.5 working days of focused effort, dominated by P1 and P3.

  The number that decides the shape of this plan is P1's 24-hour call ranking. If half the
  277 Python routes are dead, P1 halves and the whole plan lands near two weeks. If they are
  all live, this is a three-week port and the openfang route (D2) is the only sane one.
