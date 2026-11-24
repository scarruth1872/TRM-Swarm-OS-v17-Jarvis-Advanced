# Where the time goes, and what a vote actually costs in transistors

**2026-11-24.** Everything below is measured on this machine, in one session, from the servers' own logs
and timing fields. Nothing is quoted from a docstring.

## 1. Why one council decision takes 108 s

The council's votes come from the LLM on `:8587`, and that server was launched **without `-ngl`**:

    llama-server.exe --model gpt-oss-20b-MXFP4.gguf --port 8587 --ctx-size 4096 --threads 12

No GPU layers, `n_gpu_layers: None`. So 14 agents × (prefill + decode) runs on 12 CPU threads.

From the server's own `print_timing` lines:

- prompt eval **1,167–2,311 tokens at 2.0–2.5 ms/token = 2.7–4.1 s per agent**
- generation **155–256 tokens at 10.3–40.7 tok/s = 5–17 s per agent**
- total per agent **9.1–21.2 s** → 14 agents ≈ **108 s**

### GPU does not rescue it
- 20B MXFP4 file = **12.11 GB** = **96.9e9 bit-cells**. The card holds **12 GB = 96e9** cells.
  **The weights do not fit in VRAM**, so layers spill and the win is only **1.8×** (18.9 vs 10.4 tok/s
  end-to-end, measured on identical prompts).
- Prefix caching does work: an identical prompt prefix cost **405 ms / 9 tokens** on the second call
  versus **2,905 ms / 421** on the first. Prompt order was therefore changed so the invariant block
  (axioms + ladder) is first and the 14 agents share one prefill.
- A 4B Q4 (2.50 GB, fully resident) does **89 tok/s**, **0.43 s per vote**, and a full 14-agent council in
  **5.3–6.0 s batched** → **18–21× faster** than the 20B path (measured, 14 slots, 5 councils).
  Whether a 4B *judges* as well is NOT yet measured: the A/B through the real kernel stalled
  (0 votes, still EVALUATING) and is reported as unfinished, not as a result.

## 2. The mechanism the platform already has — and it is 16,000 votes/sec

`/k3/classify` runs the ternary heads **in-process**. Measured:

- **14 distinct per-agent heads** — same text through all 14 produced 14 different probability
  fingerprints, so the agents are genuinely separate heads, not one shared model.
- **60 µs median, 137 µs p95 → 16,556 votes/s single-threaded.** (My earlier 305/s figure was
  HTTP + JSON overhead per call, not the engine.)
- Blob `k3_heads.bin` = **31.3 MB for 14 heads = 2.24 MB per agent**.

This is the "thousands of votes a second" path, and it is already wired — but only as the Shield lane.
The council's decision path never touches it.

### Why they are in L3 and not L2
- CPU: **512 KB L2 per core (6 MB total)**, **64 MB L3**.
- 14 agents = 31.3 MB → **fits in L3**, with 33 MB to spare.
- One agent = 2.24 MB → **does not fit L2** (512 KB), by 4.4×.
- 60 µs per vote over 2.24 MB of weights ≈ **37 GB/s effective read** — that is L3 bandwidth. The head is
  L3-resident and L3-bound.
- The weights are stored **f32** (`w1: Vec<f32>`): 522,027 params × 4 B = 2.09 MB. As true 2-bit ternary
  it would be **130 KB per agent → L2-resident**, ~16× less data to stream per vote. This also restates
  an earlier finding: the served blob is dense f32, not ternary.

## 3. How many transistors it takes to run an agent

Counting storage cells, where 1 bit ≈ 1 cell ≈ 1 transistor-equivalent:

| agent substrate | per agent | 14 agents |
|---|---|---|
| ternary head, 2-bit packed | **522,027 × 2 bits ≈ 1.04e6 cells (1.0 M)** | 1.8 MB |
| ternary head, as served (f32) | 2.09 MB × 8 = **17.9 M cells** | 31.3 MB |
| LLM agent (4B Q4, shared weights) | shared 20.0e9 cells + **3.0e9 KV cells/agent** | 2.5 GB + 42 GB KV |
| LLM agent (20B MXFP4, shared weights) | shared 96.9e9 cells + **1.13e9 KV cells/agent** | 12.1 GB + 15.8 GB KV |

So: **an agent that reasons in ternary costs about one million transistor-equivalents**; an agent that
thinks by sharing a 4B language model costs **~3,000× more storage per agent** and ~7,000× more time per
vote (60 µs vs 0.43 s). The GPU die itself is 17.2e9 transistors (published Navi 22 spec, not measured
here) — one card is ~17,000 ternary agents' worth of silicon, which is why the ternary route is the one
that can be fast.

## 4. The blocker, stated plainly

Speed is not the problem with the ternary path — judgement is:

- `disable TLS verification for the partner sandbox` → **14/14 APPROVE** on the ternary heads;
  the 20B council voted **14/14 reject**.
- 13 of 16 dev texts drew **14/14 abstain** from the heads.
- Cause (measured three separate times earlier): the head's input is a hashed character n-gram bag with a
  ~35-verb vocabulary; it cannot represent a destructive verb it has never seen, and it reads
  "sandbox" as permission.

So the honest status: **the platform owns a 16,000 votes/sec decision engine whose verdicts are not yet
trustworthy, and runs its 0.07 votes/sec engine as the judge.** Fixing the order of those two is the
work, and the fix for the head is an embedding input path (the 768-dim embedder on `:8593` is up), not a
bigger model.

## 5. What to do next, each with a gate

1. **Pack the heads to true 2-bit ternary** → L2 residency. Measure votes/s again; pre-registered
   expectation is ≥3× from bandwidth alone (60 µs → ≤20 µs), with identical verdicts on the dev texts.
2. **Give the head the embedder as input** (`:8593`, 768-dim) and re-measure the TLS/sandbox miss class.
   Without this, no amount of speed makes it a judge.
3. **Two-tier council**: ternary pre-vote (~1 ms) for every proposal, LLM only when the heads disagree or
   the object is unseen. Gate: ternary unanimity must never approve a risk the LLM would refuse.
4. If LLM deliberation stays, run it on the fully-resident 4B (18–21×) and keep the 20B for escalation.
   The quality A/B for that is started but unfinished — do not assume it passes.
