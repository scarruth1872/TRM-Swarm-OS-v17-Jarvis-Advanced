# Why the expert mixture is not connected to the agents (measured)

**2026-11-24.** Question asked: why are the MoE experts inside the model not directly connected to the
agents? Answer: because in this design the **agent IS the model**, and its 8 experts are private,
anonymous capacity inside it — not domain seats shared with anyone. Three independent lines of evidence.

## 1. Structural: each agent owns a private MoE; nothing names an expert

`tools/export_k3_heads.py` builds the blob from `swarm_v2_global_memory/trm_7m_agents/<Agent>_*.pt`
— **46 checkpoint files = 15 agents × 3 variants**, each a small PyTorch module with its own
`experts.{0..7}.0/.2` and `gate.0`. The K3HD v2 format stores those blocks in index order and carries
**no expert metadata**: no name, no domain, no owner. Grepping the kernel for expert naming finds only
`experts[idx]` indexing and an unrelated `digital_twin` "expert_mask".

So expert 3 in Shield is unrelated to expert 3 in Devo, and neither belongs to any council agent — they
belong to the head that contains them.

**Also found:** the blob holds **15 heads**, and one of them, **LogicMaster**, has no council seat
(the roster has 14). A trained agent that nothing seats is an inventory lie in the flattering direction.

## 2. Behavioural: the router ignores the text

Across the 12 dev texts (6 risk, 6 benign), **every head selects the same two expert indices on all 12**:

    Shield {0:12, 4:12}   Logic {4:12, 6:12}   Devo {2:12, 7:12}   Seeker {2:12, 3:12}
    Verify {3:12, 6:12}   Archi {3:12, 6:12}   Bridge {3:12, 7:12}  EventLedger {6:12, 7:12}
    Pulse {0:12, 7:12}    Jarvis {3:12, 7:12}  Hermes {4:12, 5:12}

Per head, 2 of 8 experts are ever used (3 for Flow/Orchestra/THzGateway, which flip on a single text);
**7 distinct indices are used out of 14×8 = 112 slots.** And the index means nothing across heads:
**0/12 texts** route to the same index in all 14 heads.

The gate is nearly flat, which is why the argmax never moves: entropy **1.90–1.99** against a 2.079
uniform ceiling, top-2 mass **0.435–0.525** (0.25 would be uniform). The mixture is close to an average
of all eight experts with a slight tilt.

## 3. Ablation: the experts carry ~20 % of the decision

Same 168 (head, text) pairs, verdict compared to the served top-2 mixture:

| variant | agreement | mean abs Δtrit |
|---|---|---|
| expert branch deleted (`moe = 0`) | **135/168 = 80.4 %** | 0.30 |
| always experts 0+1, gate ignored | 135/168 = 80.4 % | 0.32 |
| equal average of all 8 experts | 137/168 = 81.5 % | 0.24 |

The experts are not collapsed — inside one head their outputs are genuinely diverse (mean pairwise
cosine **0.064**, L2 norms 0.70–4.38). They are simply **not selected by content**, so the head's answer
comes mostly from the embedding mean, the K3 block and the residual. That is also why 14 different heads
land on the same class for the same text.

## What "connecting them" would mean, with gates

1. **One shared expert pool + a router over the 14 agents** — the real council-as-MoE: experts specialise
   by domain, the router is the agent selector. Needs retraining with a shared pool; gate on
   (a) expert selection varying by topic and (b) no loss on the dev verdicts.
2. **Keep per-agent heads, train the gate with a load-balance/entropy penalty** — the gate currently
   ignores input (entropy ≈ uniform). Gate: routing distribution must differ across topics
   (χ²/KL against the current per-head constant), with verdict agreement on the 12 dev texts preserved.
3. **Seat LogicMaster or drop it from the exporter** — pick one; a 15-head blob behind a 14-seat roster
   is exactly the class of mismatch this campaign keeps finding.

Scripts: `scratch/k3_moe_link.py` (routing/utilisation), `scratch/k3_moe_ablate.py` (ablation).
