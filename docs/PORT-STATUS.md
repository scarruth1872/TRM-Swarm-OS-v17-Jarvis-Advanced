PORT STATUS - RUST PORT BOUNDARY DECISION
Measured 2026-11-03 on the RX 6700 XT host. Every number below came from a command
run in this session against F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel.
Supersedes the "44 kernel-candidates" figure in docs/PRODUCTION-READINESS-PLAN.md.

================================================================================
1. VERDICT
================================================================================

The candidate list is not a backlog. It is a list of files that already exist.

  - 45 candidates, not 44. The list in docs/PORTING-BACKLOG-SURFACE-DIFF.txt has
    45 rows and its own summary line says 45. The "44" is stale.
  - 0 of the 45 are called by the council binary. The production service links
    exactly four kernel modules: k3_logic, mamba3_turbovec_ssm, memory_mapped_ipc,
    unified_transistor_fabric. None of them is on this list.
  - 34 of the 45 are not declared in src/lib.rs. They are compiled into the
    kernel binary only, through src/modmap_all.rs. The council cannot link them
    even if it wanted to: the lib target is the only surface it sees.
  - 19 of the 45 are shells. They compile, they have structs and a `new()` that
    zeroes every field, and no logic. There is nothing to wire.
  - The 13 that are real code (3,855 of the 5,122 candidate LOC) are reachable
    only from `execute_command` in src/main.rs - the interactive LCARS console
    dispatcher. A human typing "kanban" at the kernel console exercises them.
    Nothing else does.

So the decision is: port nothing for release. The port boundary is already where
it should be. What is left is a small amount of optional wiring, ranked in
section 5, and 40 modules that should be marked excluded and left alone.

================================================================================
2. BUCKET COUNTS (45 candidates)
================================================================================

  PORTED-AND-USED      0     linked and called on a live production path
  PORTED-UNUSED       13     real logic, compiles, no production caller
  STUBBED             19     compiles, body is a skeleton, nothing to call
  HOST-ONLY            9     host OS/desktop behaviour, cannot live in no_std
  NET-ONLY             4     needs a network peer
                     ---
  total               45

  LOC by bucket:  PORTED-UNUSED 3,855 | STUBBED 661 | HOST-ONLY 379 | NET-ONLY 227
  Total candidate LOC: 5,122 of the kernel's 36,029 (14%).

Definitions used, so the counts are reproducible:

  PORTED-AND-USED  compiled into the lib target AND called from a path that runs
                   in the shipped product (council binary, or kernel boot/council
                   runtime). The LCARS console dispatcher does not qualify.
  PORTED-UNUSED    real implementation - branches, arithmetic, state mutation -
                   compiles, zero production callers.
  STUBBED          file exists and compiles, but the body is struct fields plus a
                   zeroing constructor, placeholder constants (`= 0`), `return 0`,
                   or "TODO: complete 1:1 math". Wiring is impossible without
                   first writing the module.
  HOST-ONLY        the Python original imports subprocess/os/ctypes/glob or a
                   host crypto/torch library, and the behaviour is about the host
                   machine, not the kernel.
  NET-ONLY         the Python original imports socket/aiohttp/requests/urllib, or
                   the module is a transport to a separate process.

================================================================================
3. FULL CLASSIFICATION
================================================================================

module                         class          rsLOC inlib  note
---------------------------------------------------------------
background_tasks               STUBBED           10 no     10 LOC, zero fns; asyncio+os autoscan pipeline
memory                         PORTED-UNUSED    355 yes    real dual-layer store; only caller is console 'mem'
remediation_engine             STUBBED           10 no     10 LOC, zero fns
monitor_daemon                 STUBBED           10 no     10 LOC, zero fns
self_healing_orchestrator      STUBBED           24 no     1 fn returning 0
secrets_vault                  HOST-ONLY         29 no     host keystore + cryptography lib; 1 fn stub
evolutionary_sandbox           HOST-ONLY         24 no     spawns subprocess to execute mutants
native_ops_bridge              HOST-ONLY         13 no     ctypes into host shared libraries
collaborative_reasoning        STUBBED           24 no     1 fn, no logic
redis_mock                     STUBBED           10 no     10 LOC, zero fns
neural_wall                    STUBBED           55 no     struct shells; PROMPT_INJECTION_PATTERNS=0
auto_changelog                 HOST-ONLY         70 no     writes CHANGELOG.md into the repo
cognitive_stack                STUBBED           24 no     1 fn, 3 TODOs
reconnaissance_daemon          STUBBED           39 no     2 fns, no logic
brain_ai_microkernel_bridge    STUBBED           29 no     1 fn, no logic
mcp_tool_evolver               HOST-ONLY         78 no     writes evolved tools to disk
ultrawork_loop                 STUBBED           83 no     struct shells only
swarm_orchestrator             STUBBED           24 no     1 fn, no logic
swarm_performance_insights     STUBBED          108 yes    all-zero structs; no metric collection
prism_fqm                      HOST-ONLY         24 no     globs the host filesystem
meta_orchestrator              STUBBED           16 no     returns 0, 3 TODOs
port_manager                   HOST-ONLY         28 no     allocates host TCP ports
deterministic_forge            STUBBED           24 no     1 fn, no logic
global_memory                  PORTED-UNUSED   1187 no     real store+index; NOT in lib.rs
moltbook                       NET-ONLY          52 no     network feed; struct shells
cosmos3_edge_bridge            NET-ONLY          50 no     peer transport to Cosmos edge
coordination_responder         STUBBED           24 no     1 fn, no logic
kanban_board                   PORTED-UNUSED    295 yes    real state machine; console 'kanban' only
native_mailbox_bridge          PORTED-UNUSED     62 no     real ring buffer; NOT in lib.rs
chain_of_verification          STUBBED           99 yes    struct shells; FALLACY_PATTERNS=0
daily_evolution_engine         PORTED-UNUSED     54 yes    curriculum table + counter; console only
turbovec_memory                PORTED-UNUSED   1037 no     real store + JSON codec; NOT in lib.rs
optimization_engine            STUBBED           24 no     1 fn, no logic
llm_brain                      PORTED-UNUSED     73 yes    14 personas + routing; console only
semantic_cache                 PORTED-UNUSED    100 yes    real LRU+hash cache; console only
mcp_bus                        NET-ONLY          72 no     MCP sockets; struct shells
lobster_shell                  HOST-ONLY         59 no     approval-gated host shell execution
ses                            PORTED-UNUSED    148 yes    real triple extractor; console only
pbft_consensus                 PORTED-UNUSED    149 yes    real PBFT; console only
microkernel_chat_bridge        NET-ONLY          53 no     HTTP to chat backend
mutation_engine                HOST-ONLY         54 no     rewrites source files
intent_buffer                  PORTED-UNUSED    111 yes    real predictor; console only
resonance_engine               PORTED-UNUSED     87 yes    real FNV resonance; console only
llm_orchestrator               STUBBED           24 no     returns 0
star_matrix_lattice            PORTED-UNUSED    197 no     real lattice math; NOT in lib.rs

Two rows deserve a note because their class is not obvious from LOC:

  chain_of_verification (99 LOC, in lib, called from console) is STUBBED, not
  PORTED-UNUSED. It has four structs, four zeroing constructors, and
  `pub const FALLACY_PATTERNS: u64 = 0;`. The console call site builds a
  ReasoningStep and a VerificationResult and prints them. No verification happens.

  swarm_performance_insights (108 LOC, in lib, called from console) is STUBBED for
  the same reason: every field is initialised to 0 / 0.0 / String::new() and no
  function reads a metric.

================================================================================
4. PORTED-UNUSED: THE MISSING CALLER
================================================================================

For each of the 13, the specific caller that does not exist, and the LOC to add
it. "wire LOC" is new code in host_council_server/src, not kernel changes.

module                  rsLOC  blocker            missing caller / wire LOC
--------------------------------------------------------------------------------
memory                    355  in lib             /swarm/chat is single-turn: it
                                                    takes role+message and passes
                                                    no history to build_prompt.
                                                    Add AgentMemory to CouncilState
                                                    and replay last N turns.
                                                    ~120 LOC
global_memory            1187  NOT in lib.rs      No cross-agent memory at all.
                                                    Add `pub mod global_memory;`
                                                    to lib.rs (1 line), then
                                                    POST /council/memory/contribute
                                                    and GET /council/memory/recall.
                                                    ~150 LOC
kanban_board              295  in lib             No route. Add /council/kanban
                                                    CRUD mapping cards to proposal
                                                    ids. ~110 LOC
native_mailbox_bridge      62  NOT in lib.rs      No agent mailbox. Add mod line,
                                                    bind ring buffer to the
                                                    memory_mapped_ipc arena.
                                                    ~60 LOC
daily_evolution_engine     54  in lib             /council/cadence never calls
                                                    evolve(). Wire into the cadence
                                                    handler. ~40 LOC
turbovec_memory          1037  NOT in lib.rs      Council has its own turbovec.rs
                                                    (245 LOC). Two vector stores,
                                                    no shared test. Pick one and
                                                    delete the other. ~80 LOC
llm_brain                  73  in lib             llm_agent_vote builds prompts
                                                    inline. Route through
                                                    LLMBrain::build_system_prompt.
                                                    ~50 LOC
semantic_cache            100  in lib             local_inference::complete is
                                                    uncached. Wrap it. ~60 LOC
ses                       148  in lib             Evidence extraction is
                                                    line-prefix counting
                                                    (`starts_with('[')`). Use
                                                    SesEngine for triples.
                                                    ~70 LOC
pbft_consensus            149  in lib             evaluate_proposal hand-rolls
                                                    approve/reject counters at
                                                    main.rs:1243-1386. Replace the
                                                    tally. ~90 LOC
intent_buffer             111  in lib             /swarm/chat has no prewarm.
                                                    ~80 LOC
resonance_engine           87  in lib             No product surface exists.
                                                    ~50 LOC
star_matrix_lattice       197  NOT in lib.rs      Retrieval uses
                                                    local_inference::evidence_for;
                                                    lattice is unused. ~90 LOC
--------------------------------------------------------------------------------
Total wire cost if all 13 were connected: ~1,050 LOC.

The four marked "NOT in lib.rs" cannot be called by the council at all today.
That is a one-line fix each, and it is the only structural blocker in this table.

================================================================================
5. RANKED PORT BACKLOG
================================================================================

Release-blocking items: 0.

Nothing on this list is required for the release gate in
docs/PRODUCTION-READINESS-PLAN.md section 3. The release blockers are the ledger
(P0-1, fixed), the unauthenticated control plane (P0-2), and the launcher (P1-1).
None of them is a port task.

What follows is the only work worth doing, ranked by defect risk rather than by
feature wish. Each entry has a justification that is not "it exists in Python".

rank  module              wire LOC  effort  justification
--------------------------------------------------------------------------------
 1    pbft_consensus           90   0.5 d   The council already tallies votes by
                                           hand (approve_count/reject_count at
                                           main.rs:1243-1386, quorum at :1386).
                                           Two consensus implementations that can
                                           disagree is an audit defect in a system
                                           whose product claim is auditable
                                           consensus. Either use the kernel one or
                                           delete it and document the hand tally.
 2    turbovec_memory          80   0.5 d   Two vector stores exist: kernel
                                           turbovec_memory (1,037 LOC) and council
                                           turbovec.rs (245 LOC). They sit on the
                                           retrieval path that measurably changes
                                           verdicts (14-0 approve -> 11-3 reject
                                           with citations). Divergence here is a
                                           correctness risk, not a feature gap.
 3    memory                  120   0.5 d   /swarm/chat is single-turn, but journal
                                           article 189 claims "Multi-Turn Context
                                           Continuity" and the fallback text at
                                           main.rs:603 repeats the claim. Either
                                           wire AgentMemory or stop claiming it.
 4    ses                      70   0.5 d   Citation grounding is the one measured
                                           behaviour that changes council verdicts.
                                           Evidence extraction is currently
                                           `starts_with('[')` line counting.
 5    global_memory           150   1.0 d   Only if multi-session memory is a
                                           release feature. The council starts
                                           cold on every deliberation. Largest
                                           real asset on the list (1,187 LOC) and
                                           currently unreachable from the lib.
--------------------------------------------------------------------------------
Total: 510 LOC, ~3 days.

Everything else on the 45: do not port.

  - The 19 STUBBED modules have no logic to connect. Porting them means writing
    them from scratch, and none of them is on a path the product uses.
  - The 9 HOST-ONLY and 4 NET-ONLY modules are correctly excluded. They are
    already stubs in Rust, which is the right end state: the file records that
    the module was considered and rejected, and the linker drops it.
  - resonance_engine, intent_buffer, kanban_board, daily_evolution_engine,
    llm_brain, semantic_cache, native_mailbox_bridge, star_matrix_lattice are
    real code with no product surface. Leave them compiled and unused. They cost
    nothing at runtime and they are the cheapest thing to reach for if a feature
    later needs them.

================================================================================
6. BUILD HEALTH (measured this session)
================================================================================

cargo check --release
  command : cd trm_uefi_kernel && C:/Users/shawn/.cargo/bin/cargo.exe check --release
  result  : exit 0, 0 errors, Finished in 7.18s
  warnings: 1,857 emitted; 1,846 unique after cargo's duplicate suppression
            (lib target 11, bin target 1,846 of which 11 duplicate the lib's)

  The 1,846 in PRODUCTION-READINESS-PLAN.md is the unique count and is correct.
  The emitted count is 1,857. Both are true; quote 1,846 as "unique warnings".

  By lint (from --message-format=json, 1,857 messages):

    dead_code              1577   84.9%
    unused_imports          222   12.0%
    unused_variables         28    1.5%
    non_camel_case_types     10    0.5%
    unused_mut                9    0.5%
    static_mut_refs           4    0.2%
    non_snake_case            4    0.2%
    unused_unsafe             3    0.2%

  Read this correctly: 85% of the warnings are dead_code, and dead_code is the
  same fact as section 2. The warning count is a port-boundary measurement, not a
  code-quality measurement. You cannot fix it by renaming things. It goes down
  when modules are either wired (section 5) or deleted. The 222 unused_imports
  are mechanical and are the only category worth a bulk `cargo fix`.

  The 4 static_mut_refs are the only warnings with a soundness dimension
  (agent_runtime.rs:155,158 and two others). They are worth a look independent of
  the port question.

cargo test --release
  result  : FAILS, exit 101
  error   : E0152 found duplicate lang item `panic_impl`
            src/main.rs:178 defines #[panic_handler]; the test harness links std,
            which defines it too.
  tests run: 0

  There is exactly one #[test] in the crate (src/digital_twin.rs:257). The kernel
  has no test harness and `cargo test` is not a usable gate. Do not put it in CI
  as-is. The lib test target does compile (11 warnings); only the bin test target
  fails, so a `--lib` test run is possible if a harness is ever wanted.

crate targets (cargo metadata --no-deps)
  lib  trm_uefi_kernel  src\lib.rs     <- real target, confirmed
  bin  trm_uefi_kernel  src\main.rs

  The prior claim that src/lib.rs is "never compiled" is false and should not be
  repeated. tools/surface_diff.py looks for a `mod` declaration, and a crate root
  never has one. That is the tool's bug, not a fact about the crate.

================================================================================
7. CORRECTIONS TO PRIOR NUMBERS
================================================================================

  claim                                   actual
  --------------------------------------  --------------------------------------
  "44 kernel-candidates"                  45 rows in the source list; the list's
                                          own summary line says 45
  "1,846 warnings"                        1,846 unique / 1,857 emitted
  "HOST-ONLY 26, NET-ONLY 5"              29 and 9 in PORTING-BACKLOG-SURFACE-DIFF
                                          (the plan's numbers do not match its
                                          own source document)
  "lib.rs is never compiled"               false; cargo metadata shows a real lib
                                          target at src/lib.rs

================================================================================
8. WHAT THIS MEANS FOR THE RELEASE
================================================================================

The port is done in the only sense that matters: the decision and control path is
native Rust and the council genuinely links and calls it. The 45 candidates are
not a queue of work waiting to be done; they are 5,122 LOC of already-written code
that the product does not reach, of which 3,855 LOC is real and 661 LOC is
scaffolding.

Acceptance for Phase 5 of the production plan is met by this document: every one
of the 45 has a classification and a reason. The second half of that acceptance
criterion - "warning count under 100" - is not met and is not a port task. It
requires deleting the 19 stubs and the 13 unused modules, or wiring them. That is
a separate decision with a separate cost, and it should not block the release.

Recommended next action: take items 1 and 2 from section 5 (170 LOC, one day),
because they remove duplicate implementations of things the product already
depends on. Defer 3-5 until multi-turn memory is a committed feature.

================================================================================
RECONCILIATION WITH docs/PORT-BOUNDARY.md (same date)
================================================================================
Two documents now classify the same 44-45 candidates and appear to disagree:
this one says 0 PORTED-AND-USED, PORT-BOUNDARY.md says 34 ON-PATH. Both are
correct because they answer different questions. Read them together.

  PORT-BOUNDARY.md asks: which PYTHON modules are on the live product path?
    Answer: 34 of them, all inside the import closure of the FastAPI service on
    :8099 (166 modules). 10 are host tooling, 0 are net-bound.

  THIS document asks: which RUST twins are wired into a shipped binary?
    Answer: 0. None of the 45 is referenced by the council binary, which links
    only k3_logic, mamba3_turbovec_ssm, memory_mapped_ipc and
    unified_transistor_fabric -- none of which are on the candidate list.

  Both facts hold simultaneously: the Python module is load-bearing, its Rust
  twin is not wired. The consequence for the plan is the opposite of a backlog.
  Porting one of those 34 means porting a FastAPI endpoint into a kernel with no
  network stack and no async executor, where it would have nowhere to run. The
  only genuine kernel contract in the list is intent_buffer (shared-memory
  intent buffer to the microkernel fast-path), and its Rust twin is 110 lines.

  So the port is not 60-92% done. Shrink percentage measures how much smaller
  the Rust file is, not how finished it is: secrets_vault at "92%" is 357 -> 28
  lines with 4 public items and one function body. Of the 45 twins, 17 are
  shells (<=2 public items) and 4 are a doc comment plus imports.

ADDITIONAL STRUCTURAL FACT from PORT-BOUNDARY.md, worth carrying into any Rust
work: trm_uefi_kernel/src/main.rs does NOT link src/lib.rs. It re-declares its
own 122-module tree, and lib.rs allows all four noisy lints, which is
why all 1,857 warnings come from the bin target. Two module trees in one crate
is why dead_code covers 190 of 207 files.
