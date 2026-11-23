<!-- Raw output of `cargo run --release` in this directory (host_sramsim), 2026-11-23. -->

```
SRAM INTEGRATION SIMULATION — 6 policies x 8 scenarios
primitives: kernel's own k3_ternary_sram + sram_pattern_computer, included by path

  S0 gate model resident on :8587 (today's state)
  S1 analysis model resident on :8596
  S2 a model actually on :8597 (the old default)
  S3 no model resident at all
  S4 model mid-swap: /health ok, completion refused
  S5 kernel unreachable, model up
  S6 unknown model on a port no list contains
  S7 Harmony-template model, caller builds ChatML

id   policy                                           S0    S1    S2    S3    S4    S5    S6    S7
P1   hardcoded :8597 + ChatML (status quo ante)       FAILS FAILS WORKS FAILS FAILS WORKS FAILS DEGRAD
P2   probe model ports + ChatML (shipped today)       WORKS WORKS WORKS ABSTAINWORKS WORKS ABSTAINDEGRAD
P3   kernel /swarm/chat, caller grounding dropped     DEGRADDEGRADDEGRADABSTAINDEGRADFAILS DEGRADWORKS 
P4   kernel-mediated, caller system + grounding       WORKS WORKS WORKS ABSTAINWORKS ABSTAINWORKS WORKS 
P6   kernel-mediated + direct-port fallback (hybrid)  WORKS WORKS WORKS ABSTAINWORKS WORKS WORKS WORKS 
P5   cloud LLM (OpenAI-compatible API)                FORBIDFORBIDFORBIDFORBIDFORBIDFORBIDFORBIDFORBID

-- Kleene trisum over each policy's own outcome vector --
  P1  trit=-1 confidence=0.625 ambiguity=0.125 cam_hits=0x1
  P2  trit=+0 confidence=0.625 ambiguity=0.375 cam_hits=0x1
  P3  trit=-1 confidence=0.125 ambiguity=0.750 cam_hits=0x1
  P4  trit=+0 confidence=0.750 ambiguity=0.250 cam_hits=0x1
  P6  trit=+0 confidence=0.875 ambiguity=0.125 cam_hits=0x1
  P5  trit=-1 confidence=1.000 ambiguity=0.000 cam_hits=0x1

-- TCAM frontier (who has WORKS in each scenario) --
  S0 -> P2, P4, P6
  S3 -> nobody
  S5 -> P1, P2, P6
  S7 -> P3, P4, P6

-- Ising energy over the doctrine spins [no-cloud, single-resident, no-stall, honest] --
  P1  spins=[1, 1, 1, -1] energy=-4 (ground state -16)
  P2  spins=[1, 1, 1, 1] energy=-16 (ground state -16)
  P3  spins=[1, 1, 1, 1] energy=-16 (ground state -16)
  P4  spins=[1, 1, 1, 1] energy=-16 (ground state -16)
  P6  spins=[1, 1, 1, 1] energy=-16 (ground state -16)
  P5  spins=[-1, 1, 1, -1] energy=+0 (ground state -16)

-- Hopfield stability (2 flipped cells out of 8) --
  P1  agreement 7/8  recalled=[-1, -1, 1, -1, -1, 1, -1, 1]
  P2  agreement 5/8  recalled=[1, 1, 1, 1, 1, 1, 1, 1]
  P3  agreement 1/8  recalled=[1, 1, 1, 1, 1, -1, 1, -1]
  P4  agreement 6/8  recalled=[1, 1, 1, 1, 1, 1, 1, 1]
  P6  agreement 7/8  recalled=[1, 1, 1, 1, 1, 1, 1, 1]
  P5  agreement 8/8  recalled=[-1, -1, -1, -1, -1, -1, -1, -1]

-- ranking (usable desc, measured-usable desc, doctrine energy desc) --
  1. P6  kernel-mediated + direct-port fallback (hybrid) usable=7/8 measured_usable=2 energy=-16 hopfield=7/8
  2. P4  kernel-mediated, caller system + grounding     usable=6/8 measured_usable=1 energy=-16 hopfield=6/8
  3. P2  probe model ports + ChatML (shipped today)     usable=5/8 measured_usable=2 energy=-16 hopfield=5/8
  4. P1  hardcoded :8597 + ChatML (status quo ante)     usable=2/8 measured_usable=0 energy=-4 hopfield=7/8
  5. P3  kernel /swarm/chat, caller grounding dropped   usable=1/8 measured_usable=1 energy=-16 hopfield=1/8
  6. P5  cloud LLM (OpenAI-compatible API)              usable=0/8 measured_usable=0 energy=+0 hopfield=8/8

Kleene trisum winner (trit>=0, highest confidence): P6 trit=+0 conf=0.875 amb=0.125

MEASURED/MODELLED SPLIT: 21 of 48 cells measured across all policies.
Policies with NO measured cell (pure hypothesis): 
Winner is only as good as that split — re-run after the kernel change to convert the
modelled cells into measurements.
```
