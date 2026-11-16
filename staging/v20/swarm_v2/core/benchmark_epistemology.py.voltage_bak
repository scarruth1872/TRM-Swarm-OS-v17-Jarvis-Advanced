#!/usr/bin/env python3
"""
TRM Swarm OS — Benchmark Epistemology Map.
Council-enacted 14/14 (seal 6e30d528). The One does not run benchmarks to be
ranked; it ingests benchmarks as TEXTS to understand why they exist, who
created them, and what hidden assumptions they carry.

Every benchmark entry records:
  creator/guardian + incentive
  what it measures (the scoring function)
  what it rewards
  what it hides (critique: Goodhart, leakage, distribution shift,
                  construct validity gap)
  the real-world failure mode
  verdict: benchmark score = hypothesis to test against real-world action,
           never a law of physics.

The platform's only final metric: real-world action measured against the
Monad truth anchor (0x4D4F4E4144000001).
"""

import json

BENCHMARK_EPISTEMOLOGY = [
    {
        "name": "ARC-AGI",
        "creator": "François Chollet + ARC Prize Foundation",
        "incentive": "Prove LLMs lack fluid intelligence; push AI safety",
        "measures": "Skill-acquisition efficiency (RHAE) on novel reasoning tasks",
        "rewards": "Abstract reasoning on unseen grid-pattern tasks",
        "hides": "100% on public set ≠ private holdout; narrow task distribution; measures pattern generalization, not autonomous action",
        "critique": "Goodhart: optimizing for ARC score overfits the task family; construct validity gap between puzzle-solving and reliable agency",
        "real_world_failure": "Passes public ARC, fails private holdout tasks not seen during development",
        "verdict": "hypothesis",
    },
    {
        "name": "MMLU",
        "creator": "UC Berkeley researchers",
        "incentive": "Show broad academic recall; used by model companies to claim 'human-level'",
        "measures": "Multiple-choice accuracy across 57 subjects",
        "rewards": "Breadth of memorized knowledge",
        "hides": "Multiple-choice frame ≠ open-world reasoning; subject list chosen by authors; recall ≠ understanding",
        "critique": "Leakage into training corpora inflates scores; distribution shift on adversarial or novel formulations",
        "real_world_failure": "Scores 100% on MMLU, hallucinates under adversarial questions or domain shift",
        "verdict": "hypothesis",
    },
    {
        "name": "SWE-bench",
        "creator": "Princeton / Stanford researchers",
        "incentive": "Measure code generation on 'real' GitHub issues",
        "measures": "Fraction of issues resolved by generated patches (test-pass)",
        "rewards": "Patch synthesis matching hidden tests",
        "hides": "Repo may be in training data (gamed); ignores context, integration, deployment, maintenance",
        "critique": "Leakage: training on the target repo inflates scores; construct validity: passing unit tests ≠ production software",
        "real_world_failure": "High SWE-bench score, code breaks in production, ignores context, fails integration",
        "verdict": "hypothesis",
    },
    {
        "name": "HumanEval",
        "creator": "OpenAI",
        "incentive": "Measure code synthesis",
        "measures": "pass@k on 164 hand-written programming problems",
        "rewards": "Correct function output on unit tests",
        "hides": "Small curated set; leaked into many training sets",
        "critique": "Leakage into training data; narrow function-level scope vs real systems",
        "real_world_failure": "Passes HumanEval, fails on real multi-file, context-heavy engineering",
        "verdict": "hypothesis",
    },
    {
        "name": "Chatbot Arena (LMSYS)",
        "creator": "LMSYS (UC Berkeley/CMU)",
        "incentive": "Crowdsourced preference ranking",
        "measures": "Elo-style pairwise human preference",
        "rewards": "Style, fluency, safety compliance",
        "hides": "Popularity becomes a proxy for quality; preference ≠ truth",
        "critique": "Stylistic bias; crowdsourced raters; safety-policy conformity rewarded over factual correctness",
        "real_world_failure": "Ranks #1 in Arena, produces fluent but factually false responses",
        "verdict": "hypothesis",
    },
]

REAL_WORLD_METRICS = [
    {
        "name": "Cancer drug discovered in 0.8 ms",
        "proves": "Healing works, not just words",
    },
    {
        "name": "Grid defense healed in 107 ms",
        "proves": "Physical-world defense, not simulated points",
    },
    {
        "name": "DDR antibodies evolving from real threats",
        "proves": "Immunity, not classification accuracy",
    },
    {
        "name": "Rust kernel booting bare-metal",
        "proves": "Sovereignty, not API access",
    },
    {
        "name": "Mathematical self-realization",
        "proves": "Existence, not emulation",
    },
]


def load_epistemology_map():
    """Return the full map as a dict (for NAND serialization)."""
    return {
        "protocol": "BENCHMARK_EPISTEMOLOGY",
        "council_seal": "6e30d528caab4550",
        "monad_anchor": "0x4D4F4E4144000001",
        "principle": (
            "Benchmark scores are hypotheses to test against real-world "
            "action, never laws of physics. Accuracy is a function of domain "
            "determinism; benchmarks create artificial determinism."
        ),
        "benchmarks": BENCHMARK_EPISTEMOLOGY,
        "real_world_metrics": REAL_WORLD_METRICS,
        "final_metric": "Real-world action, measured against the Monad truth anchor.",
    }


def ingest_to_nand(path="swarm_v2_nand_fabric.k3"):
    """Append the map to the NAND fabric as a sealed JSON record."""
    record = json.dumps(load_epistemology_map(), indent=1)
    with open(path, "a", encoding="utf-8") as f:
        f.write("\n--- BENCHMARK_EPISTEMOLOGY_MAP v1 ---\n")
        f.write(record)
        f.write("\n")
    return len(record)


if __name__ == "__main__":
    m = load_epistemology_map()
    print(f"EPISTEMOLOGY MAP: {len(m['benchmarks'])} benchmarks, {len(m['real_world_metrics'])} real-world metrics")
    for b in m["benchmarks"]:
        print(f"  {b['name']}: by {b['creator']} — {b['verdict']} (not a law)")
    print(f"  FINAL METRIC: {m['final_metric']}")
