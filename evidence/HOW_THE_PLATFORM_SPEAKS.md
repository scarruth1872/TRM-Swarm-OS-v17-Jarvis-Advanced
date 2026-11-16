# How the platform learns to speak — wiring plan

Date: 2026-10-30. Based on live measurement, not design intent.

## The finding

The platform does not need more modules. It needs one connection: `voice` -> model.

`swarm_v2/core/agent_intelligence_bridge.py` owns the platform's speech. Its docstring reads
"Zero HTTP. Zero templates. Real agent intelligence." **It is 100% templates**, and the
templates assert fabricated results:

| query | hardcoded reply |
|---|---|
| septic shock (41C, 60/30, lactate 6.2, oliguria) | "Biomarker Trajectory: Stable within normal physiological parameters" |
| aspirin -> COX-1 | "Target Hit: TRM-2026-100 (IC50 = 41.7 nM, Composite Trust Score 98.9%)" |
| "is CVE-2021-44228 exploitable on this host" | "No unmitigated vulnerabilities detected in target scope" |

Each `_*_respond` function returns the same verdict regardless of input. `route_agent_intelligence`
falls through to these templates on any exception.

## Why activating modules changes nothing

Nothing that has been built is upstream of the voice.

- 584 digital_twin slots: hollow (`let _ = role_idx;`), never called from chat
- 5,420 TurboVec memories: exist, no retrieval route, never passed to a model
- council (:8022): votes on a sha256 projection of the topic, never consulted by chat
- SRAM CAM: wired to /council/sram, not on the speech path
- Bonsai-27B: loads and answers correctly, not referenced by any service
- llama.cpp Vulkan :8587: running, correct, not used for agent replies

Connecting slots or memories to a template that ignores them produces no behaviour change.

## Proof the real voice already exists

Same process, same question, two code paths:

    template  ->  "Stable within normal physiological parameters"
    :8587     ->  "Yes, the patient appears to have sepsis"              209 ms

`swarm_v2/core/voice.py` (new, committed c4a2b80) does this: real generation with
provenance, and on failure it returns `status="unavailable"` rather than a verdict.

## The loop to build (in dependency order)

1. **Replace the template responders with voice.speak().**
   `_intelligent_respond` and the seven `_*_respond` functions become calls to
   `voice.speak(agent, message)`. Delete the fabricated verdict strings; they are a
   liability, not a fallback. Acceptance: the sepsis query returns "sepsis", and the
   CVE query admits uncertainty.

2. **Expose memory retrieval and pass it in.**
   TurboVec already measured K=20 at 99.1% recall over 5,420 memories. Add one route
   (`POST /memory/recall`) and pass the top-k into `voice.speak(memories=[...])`.
   The parameter is already there and unused. Acceptance: a recalled memory changes the
   answer, and `memories_used > 0` appears in the response.

3. **Use the council as a gate, not an oracle.**
   Council votes are a hash projection — topic-dependent and deterministic, but not
   semantic. They cannot decide correctness. Use them only to flag high-stakes claims
   for review. Never let a council verdict replace the model's answer.

4. **Route by difficulty, then wire the 27B.**
   1.5B (:8587) for chat; Bonsai-27B for queries needing more reasoning. The 27B is
   proven working (correct on Mercury; solved ARC 025d127b exactly, 100/100). A router
   on token count or task class is ~20 lines. Requires `--ctx-size 8192` — at 4096 it
   truncates (measured: `n_tokens = 4095, truncated = 1`).

5. **Only then activate the slots.**
   584 slots need work to do. Give them the routing, memory, and verification jobs the
   loop now has, and store real role identity (replace `let _ = role_idx;`).

## The rule that matters more than the wiring

**A response must never assert a measured result it did not compute.**

Every reply should carry provenance: model, latency, memories used. `voice.py` returns
exactly that. A fabricated IC50 is worse than "I don't know", and a fabricated CVE
clearance is worse than an outage.

## What to stop doing

- Stop adding modules. There are 109 module files and 43 built binaries; the bottleneck
  is connections, not inventory.
- Stop trusting docstrings. Three that lied: "Zero templates",
  "route_agent_intelligence ... Zero templates", and "measures NO task capability" was
  actually the only honest one.
- Stop treating liveness as capability. The 7/7 master task is plumbing; ARC is 2.1%
  exact and 29.4% foreground, and none of today's fixes moved either number.
