"""Council proposal: local model integration (TurboVec ternary quantization wave)."""
from swarm_v2.core.swarm_council import SwarmCouncil

c = SwarmCouncil()
topic = (
    "LOCAL MODEL INTEGRATION. Inventory complete: 40+ GB HF-cache models "
    "(Instella-MoE-16B-A3B 31.7GB, Qwen3-4B 8GB quantized-verified) + 32 "
    "Ollama GGUF models (75GB: bitnet-b1.58-2B native ternary, bonsai-27b, "
    "gemma4, deepseek-r1, qwen2.5-coder, llava, embeddings). Integration "
    "wave: apply the verified Qwen3-4B template (safetensors/GGUF -> "
    "TurboVec 1.58-bit ternary -> K3 fabric shard -> Monad seal -> NAND) "
    "across the priority set. HARD RULES: on-device only (no API); "
    "every compression ratio and reconstruction cosine MEASURED and "
    "ledgered honestly; GGUF Q4_K_M re-quantization reports real added "
    "loss; Ollama stays host-tooling (quantization feeds K3 fabric, not "
    "Ollama replacement). Priority: (1) bitnet-b1.58 native ternary, "
    "(2) Instella-MoE-16B experts+router (the instella_router's actual "
    "brain), (3) edge models gemma-270m/Qwen3-1.7B, (4) capability "
    "expansion llava/CLIP/whisper embeddings, (5) code/reasoning set."
)
prop = c.propose(topic=topic, category="local-model-integration", proposer="Shawn (Operator)")
c.deliberate(prop.proposal_id)
c.vote(prop.proposal_id)
e = c.enact(prop.proposal_id)
print("COUNCIL:", e.get("weighted_score"), e.get("tally"), str(e.get("monad_seal"))[:16])
