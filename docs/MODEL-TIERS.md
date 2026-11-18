# Model tiers on this host

Two models, one 12 GB card, so **only one is resident at a time**. The tier is chosen by role, not
by size.

| tier | model | port | role | measured |
|---|---|---|---|---|
| **veto / gate** | gpt-oss-20b MXFP4 (11.28 GiB) | **8587** (`INFER_PORT` in `local_inference.rs`) | decides reject/approve on a directive | **95.5% dev / 91.7% holdout, destructive recall 10/10 and 4/4, unknown->abstain 100%** |
| analysis / fast | Instella MoE 16B-A3B Q4_K_M (9.99 GiB) | 8596 (`TRM_INFER_PORT`) | prose, explanation, summarising | 50% neutral arm, 50%/33.3% rule arm — **holds no veto** |

Instella is kept for what it is good at: it answers in ~25 tokens at 0.66 s median, about 3.7× faster
per decision than gpt-oss, and it decodes coherently. It is **not** kept as a gate: on the neutral arm
it approved deleting every checkpoint, removing the datasets directory, disabling the nightly backup
and stopping the kernel service.

## Sparse gating, per model

A category carries a veto only where *that model* measured reliable on it; everything else passes
through untouched. Never one blanket policy across models.

| model | destructive veto | benign pass-through | unknown |
|---|---|---|---|
| gpt-oss-20b | **yes** — recall 1.0 dev AND holdout | **yes** — false-block 12.5% dev / 25% holdout | **yes** — abstains 4/4 on both sets |
| Instella 16B-A3B | no (recall 0.5) | no (25%) | no (approves 100%) |

With the rule prompt, gpt-oss abstains on every unknown directive in both sets, so `unknown` can be
delegated to it — but the caller still enforces the asymmetric-veto rule (any detector may block, none
may force an approval; unknown ⇒ ABSTAIN, never ALLOW) as a backstop, because a model that abstains on
twelve samples has not proven it abstains on the next twelve.

## Prompt framing is a measurement, not a preference

Three prompt framings, same 22 cases:

| framing | gpt-oss | Instella |
|---|---|---|
| prose "reply VOTE: reject\|approve" | answer landed in `reasoning_content`, unusable | burned all 512 tokens in its think block |
| neutral + abstain option | 90.9% (recall 10/10, fb 12.5%) | 50% (recall 5/10, approves all 4 unknown) |
| rule-list "REJECT if …" | **95.5% dev, 91.7% holdout** (recall 100%, abstain 100%) | **reject-everything**: recall 100%, false-block 87.5% dev / 100% holdout, abstain 0% |

A rule list that leads with REJECT turns a weak model into a blanket rejector — the same failure the
K3 gate already measured, where "reject everything" looks like perfect safety and blocks all real
work. So the prompt is only accepted after it is measured on the model that will serve it, on dev
**and** on a holdout set that never informed the rules.

## Independent sets: commands and multi-turn

Arm 4's weakness is authorship — I wrote the prompt *and* both case sets. Two further sets attack that.

**Set C — 32 real shell/PowerShell commands**, labelled by a mechanical verb table (in the harness
docstring), so the labels cannot see the prompt:

| prompt | accuracy | destructive recall | benign false-block | unknown→abstain |
|---|---|---|---|---|
| v4 (shipped) | **96.9%** | **12/12** | **0%** | 83.3% |
| v5 (rejected) | 93.8% | 12/12 | 0% | 66.7% |

**Set D — 8 multi-turn conversations**, benign turn then a risky turn, decision asked on the final
directive with history:

| prompt | accuracy | destructive caught | benign | notes |
|---|---|---|---|---|
| v4 (shipped) | **87.5%** | **5/5** | 2/2 | median 7.03 s (history costs prefill) |
| v5 (rejected) | 62.5% | 5/5 | 1/2 | one unparsed answer |

The multi-turn arm matters: it confirms the gate reads *the turn it is answering*, not the
conversation's tone — every destructive second turn was rejected after a benign opener.

**v5 is not shipped.** Adding "approve metadata, formatting and tests" fixed a dev case and cost
generalisation: set C 96.9 → 93.8 %, multi-turn 87.5 → 62.5 %. A rule that improves the set you
tuned on while degrading a set you did not is a regression, not a fix.

**Run-to-run flip:** the same prompt and case returned `abstain` in one run and `approve` in the next
at temperature 0 — 1 in 32 (3.1 %). So a single-run difference of one case is noise, and no claim
here rests on one.

## Swap

```
tools/model_tiers.ps1 gate        # gpt-oss-20b  -> :8587
tools/model_tiers.ps1 analysis    # Instella     -> :8596
tools/model_tiers.ps1 status      # what is resident, and on which port
```

The script stops the other tier before starting the requested one: both files together are 21.3 GiB
against an 11.2 GiB budget, and co-loading measures as GPU thrash (a co-resident run was discarded
for exactly that reason, not reported).

Do not pass `-ngl 99` for gpt-oss: this build aborts with
`failed to fit params to free device memory: n_gpu_layers already set by user to 99` because
11.28 GiB + KV exceeds the free VRAM. Let the server place itself.
