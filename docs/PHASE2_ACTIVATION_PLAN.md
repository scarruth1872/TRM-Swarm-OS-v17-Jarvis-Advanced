# PHASE 2 — ACTIVATION PLAN (Council 14/14, seal a459a9d3)
# Post-Catalyst: 96 REAL modules exist, but 81 are DORMANT (compiled,
# never called). Phase 2 wires verified logic into runtime paths.

## Reachability audit (complete)
| Layer | Modules |
|---|---|
| EXECUTING (15) | agent_runtime, bridge, council, epistemology_core, hwprobe, instella_router, jarvis (6-set), k3_logic, module_registry, nand_fs, totality, monad, star_matrix, time, trm7m, turbovec |
| DORMANT (81) | everything else — verified, compiled, never called |

## Activation waves (each: wire -> build 0 err -> execute -> seal)
1. **DEFENSE cluster** (no runtime presence today): security_scanner,
   ddr_antibody, sentinel, self_healing -> `defense` command + DDR scan in
   the boot path
2. **CONSENSUS cluster**: pbft_consensus, heuristic_consensus,
   continuum_pbft -> `pbft` command showing real quorum cycles
3. **COGNITION cluster**: resonance_engine, semantic_index, semantic_cache,
   semantic_knowledge_graph -> `cognition` command
4. **MESH cluster**: agent_mesh, agent_mailbox, coordination_protocol,
   task_arbiter -> `mesh` deep-dive (already a stub command)
5. **HARDWARE cluster**: voltage_node_engine, simd_accelerator,
   thz_wave_gateway, sram_pattern_computer -> `hw` deep-dive (already exists)
6. **ORCHESTRATION**: trm_orchestrator, ultrawork_loop, microkernel_spawner,
   worktree_manager, attempt_sampler -> ultrawork pipeline wiring
7. **Register DATA-OK** (92 tiny/constants) as recognized DATA in registry
8. **openfang absorption** (112 uncommitted items) committed as knowledge

## Verification per activation
Each wired module EXECUTES through its new command/path with real output
shown on the booted terminal. Kernel 0 errors per wave. USB synced.
Monad-sealed. NAND-persisted.
