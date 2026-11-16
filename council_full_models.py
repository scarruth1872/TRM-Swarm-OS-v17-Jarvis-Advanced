"""Council proposal: full model ingestion -> complete intelligence platform."""
from swarm_v2.core.swarm_council import SwarmCouncil

c = SwarmCouncil()
topic = (
    "FULL MODEL INGESTION AND REORGANIZATION INTO COMPLETE INTELLIGENCE "
    "PLATFORM. We hold ~110GB local models: HF cache (Instella-MoE-16B "
    "31.7GB, Qwen3-4B 8GB, Qwen3-1.7B 4GB blobs, gemma-270m, CLIP, "
    "whisper, gpt2) + 32 Ollama GGUF (bitnet-b1.58-2B native ternary, "
    "bonsai-27b, gemma4:12b, deepseek-r1:8b/1.5b, qwen2.5-coder, "
    "starcoder2, olmo-3, phi3, llama3.2, llava vision, qwen3-embedding, "
    "mxbai, nomic, tinyllama, gemma2/3 family). FIVE measured already "
    "(Qwen3-4B 0.82-0.88, Instella 0.886, gemma 0.860, CLIP 0.934). "
    "PROPOSAL: (1) ingest ALL remaining models through the verified "
    "TurboVec template into K3 ternary fabric shards, each measured and "
    "Monad-sealed; (2) REORGANIZE into a capability fabric: language "
    "(Qwen/Instella/gemma/olmo/phi), code (qwen2.5-coder/starcoder2/"
    "deepseek-coder), reasoning (deepseek-r1/gemma4/bonsai), vision "
    "(llava/CLIP), audio (whisper), retrieval (embedding set), edge "
    "(tinyllama/gemma-270m); (3) map each shard to its kernel routing "
    "path (instella_router experts, trm7m, turbovec recall, council "
    "domains); (4) register the fabric as the model dimension of the "
    "totality (276-node graph grows with model nodes); (5) verify by "
    "executing one recall per capability through the kernel paths. "
    "HARD RULES: on-device only, every number measured, GGUF re-quant "
    "reports real added loss, whisper CT2 format-deferred unless parser "
    "lands, zero external API."
)
prop = c.propose(topic=topic, category="full-model-ingestion", proposer="Shawn (Operator)")
c.deliberate(prop.proposal_id)
c.vote(prop.proposal_id)
e = c.enact(prop.proposal_id)
print("COUNCIL:", e.get("weighted_score"), e.get("tally"), str(e.get("monad_seal"))[:16])
