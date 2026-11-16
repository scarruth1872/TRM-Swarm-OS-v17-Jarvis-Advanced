# Wiring the three skeletons to their strips (the boundary is now crossed)

**Block 146 companion.** The previous round made the three modules *reachable* and proved parity; the modules
still loaded their strip and never called it. This round makes them call it — and proves that crossing the
boundary changes no answer.

## What "wired" means here, per module

| module | wired call | hazard avoided |
|---|---|---|
| `chaos_engine` | `should_drop_connection()` → a **stateless** export `strip_chaos_should_drop_pure(enabled, rate, sample)` | the engine's native copy could go stale if a caller assigns `enabled`/`dropout_rate` directly, so the three values the method already holds are passed IN |
| `expert_registry` | `get_agent(identifier)` → `strip_expert_resolve` returns a row **index**; the agent object stays in python | the row table is re-mirrored on **every** call, so a wholesale `register_team` replacement cannot be answered from a stale table with another agent's index — an error that looks like a hit |
| `persona_matrix` | `enforce()` → `strip_persona_decide`; `build_compiled_prompt()` → `strip_persona_prompt` | the clock is passed IN (`time.time() - last_reset_time`), the kernel contents are mirrored per call, and the signature keeps its single owner in python |

In every case the pure-python rule stays in place as the oracle and as the **fallback**: `None` from the
native helper means "run the python rule", and any native failure sets `_NATIVE_LOADED = False` with an honest
`_NATIVE_REASON` — so a missing or broken strip changes *which* code runs, never the answer it produces.

## The rulers had to change too (or they would have become circular)

Once the oracle delegates to the strip, comparing "the module" against "the strip" compares the strip with
itself. Each ruler therefore **forces the module onto its pure path** (`mod._NATIVE_LOADED = False`) to obtain
an independent reference, and then adds a wiring check: force `True`, replay the same table through the
module's public API, and require identical answers.

| ruler | checks | includes |
|---|---|---|
| `verify_chaos_parity.py` | **14** | pure-vs-strip on 28 (rate, draw) pairs · **wired == pure on 28 pairs** · stateless export agrees (no stale-state hazard) |
| `verify_expert_parity.py` | **6** | pure-vs-strip on 18 identifiers · **wired == pure on 18** · a **replaced** team is answered from the new table, not a stale mirror |
| `verify_persona_parity.py` | **8** | pure-vs-strip on 8 decision cases · **wired == pure on 8** · **wired prompt is the same signed 661-byte string** · signature still single-owner |

**28 checks, 0 failed.** Three of them assert the thing that actually changed this round: the wired path and
the pure path agree, case for case.

## Verified through the public API, not just the harness

```
chaos   loaded=True   e.enable(); e.set_dropout_rate(1.0); draw 0.0 -> should_drop_connection() == True
expert  loaded=True   get_agent('** Logic (Reasoning Engine)') -> the registered agent (native index)
                      get_agent('nope')                        -> None
persona loaded=True   build_compiled_prompt('Logic', kernel)   -> <!-- SIGNATURE:718b53b8... --> + 552-byte body
```

Real importers (from a repo grep, excluding `scratch/` and the checkpoint copies): `swarm_v2/routes/swarm.py`
and `swarm_v2/routes/system.py` import `chaos_engine`; `swarm_v2/core/base_agent.py`,
`swarm_v2/core/agent_intelligence_bridge.py` and `integration_test.py` import `expert_registry` /
`persona_matrix`. So the wired paths sit behind the agent loop and the routes, not only behind a test.

## Honest bounds

- **Wired ≠ faster.** No speedup is claimed or measured here: these are boundary crossings for a float
  comparison, an index lookup and a string build. The claim is narrower and exact — the platform's native
  artifacts now execute on the module's real call paths, and the answer is unchanged.
- Every native path is *fallback-first*: the python rule still runs when the strip is absent, so a deployment
  without the DLLs behaves exactly as before.
- `persona_matrix.enforce` applies `violation_count`/`last_reset_time` on the python side in the oracle's
  order, including not touching either on the LOCKED return. That side-effect parity is covered by the 8 cases,
  not by a separate state trace.
- `GenerationContext.identity_kernel` remains a `u64` handle in the kernel struct; the wired path passes the
  kernel's contents directly to the typed functions.
- The strips are still DLLs on the module path — none of the three is a `host_council_server` route, so the
  kernel/dashboard surface does not expose them (the GPU strip has that gap too).
