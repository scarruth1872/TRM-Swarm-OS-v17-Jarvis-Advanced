# Where the Intelligence Lands — agent chat measured

Measured 2026-11-04 against the live platform (`POST :8099/swarm/chat`, 15 agents
from `/swarm/experts` and `/mailbox/agents`). Raw data in
`evidence/AGENT_CHAT_MEASURED.txt` and the four JSON artifacts beside it.

The question was "test the chat functions for all the agents to see where the
intelligence lands." This is the answer as measured, not as described.

## The headline

**Intelligence lands in exactly one place: `qwen2.5-1.5b` on `:8587`.**

14 of 15 agents report `engine = "llama.cpp-vulkan@8587-qwen1.5b"`. One — Hermes —
reports `TRM_7M`. There is no per-agent model, no per-agent fine-tune and no
per-agent engine. The swarm is one 1.5B model wearing 15 name badges.

## 1. Reachability — clean

15/15 answered, zero transport failures. The chat surface works.

## 2. Role differentiation — cosmetic, and the source proves why

One neutral question to all 14 LLM agents produced **7 unique replies**, with six
agents returning byte-identical text (Logic, Verify, Archi, Bridge, Orchestra,
EventLedger) and two more pairs collapsing (Shield/Pulse, Seeker/Jarvis). Only
four agents produced anything unique.

The cause is in `swarm_v2/core/voice.py`:

```
SYSTEM = "You are {agent}, one agent in a technical swarm. Answer factually and
          concisely. If you do not know, say so plainly. Never invent numeric
          results, IC50 values, CVE status, or clinical findings."
```

Only the **name** is substituted. The role, background, specialties and skills
that `/swarm/experts` advertises are never sent to the model. The metadata is
decoration; it cannot influence an answer. Remaining variation is decoding noise
at temperature 0.2 — which is why Verify and Pulse happened to say "No" to "Is 91
prime?" while 12 others said "Yes".

## 3. Hermes — a canned template, falsified

Four different questions — quorum, arithmetic, primes, sky colour — answered in
**0.6–0.9 µs** each. After stripping the echoed question, all four reduce to one
frame:

```
[Hermes | TRM 7M | +TRUE | conf=0.33]

I coordinate the swarm through the TRM 7M Mamba-3 SSM — 7M parameters, 77.4ns
per step, O(1) memory. 14 agents, 688 datasets, 345M steps.

Decision: +TRUE on '<question>'
```

The trit is `+TRUE` and confidence `0.33` for *every* input, including "what
colour is the sky". Hermes scored 2/6 on the battery only because the expected
numbers appear in its boilerplate. This is the `route_agent_intelligence()`
failure mode named in memory: a template whose only variable is the question echo.

## 4. Correctness — where the model actually fails

| question | correct |
|---|---|
| 17×23 = 391 | 14/15 |
| 144/12 = 12 | 15/15 |
| 2/3 of 9 = 6 | 15/15 |
| 2^10 = 1024 | 14/15 |
| **7th prime = 17** | **0/15** |
| **91 is not prime** | **2/15** |

Every agent said **13** for the 7th prime. Fourteen independent agents, one shared
model, all confidently wrong with the same wrong number. A system with real
per-agent diversity could not fail 15/15 identically. This single row is the
clearest proof that the roster is one model.

## 5. Two defects found and fixed while testing

**Memory never reached chat.** Every response reported `memories_used = 0` while
agents advertise 103,363 memories and the store holds 104,473 entries.
`global_memory.query()` guarded on `if not CHROMA_AVAILABLE:` — but TurboVec
initialises first and returns early, leaving `self.client` as `None` while
`CHROMA_AVAILABLE` is `True`. The guard passed, called `.query()` on `None`, raised
`AttributeError`, and `_intelligent_respond` swallowed it in a bare `except`.
Fixed by testing the instance binding; verified `memories_used = 1` live.

*Honest limit:* retrieval now runs but is not semantic. Three different queries
returned the same generic context, because the keyword-overlap fallback matches
boilerplate. Plumbing connected; quality not useful. Wiring TurboVec's vector
search in is the real fix and is not done.

**The deep tier was a phantom.** `DEEP_URL` pointed at `:8591` "Bonsai-27B-Q1_0",
which returns `000` — nothing listens there. Every deep-word question
(`explain`, `why`, `design`, `prove`…) paid **2.04 s** to fail before falling back
to `:8587`. Fixed with a cached reachability probe: 1.95 → 1.40 → 1.21 s over three
calls. The tier now honestly does nothing. Either run a real deep model or delete
the tier.

## 6. What is genuine, and what is not

Genuine: 14 agents return real model output, not templates. Answers are coherent,
arithmetic is right, provenance is honest about which engine spoke.

Not genuine: the "swarm" is one model with 15 labels; Hermes is canned; memory is
absent from the prompt; there is no deep model; and a trivial prime question fails
15/15 identically.

The next lever is not more agents. It is putting role, specialty and retrieved
memory into the prompt — the pieces already exist in the roster and the store, and
none of them currently reach the model.
