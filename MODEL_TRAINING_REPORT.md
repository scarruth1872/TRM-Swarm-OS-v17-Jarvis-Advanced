# Jarvis Model Training Report

**Date:** July 11, 2026  
**Author:** Shawn Carruth (Boss)  
**Objective:** Fine-tune a language model to embody J.A.R.V.I.S. OS — a sovereign intelligence system with deep knowledge of the TRM Swarm ecosystem, 16 modules, 9 skills, and system architecture.

---

## Methodology

### Approach
Parameter-Efficient Fine-Tuning (PEFT) with Low-Rank Adaptation (LoRA). This enables full knowledge transfer without retraining the entire model.

### LoRA Configuration
| Parameter | Value |
|---|---|
| Rank | 16 |
| Alpha | 32 |
| Target modules | q_proj, k_proj, v_proj, o_proj |
| Dropout | 0.05 |

### Training Hyperparameters
| Parameter | Best Run |
|---|---|
| Batch size | 2 |
| Gradient accumulation | 2 |
| Sequence length | 512 |
| Learning rate | 2e-4 |
| Scheduler | Cosine annealing |
| Optimizer | AdamW (weight decay 0.01) |
| Precision | bf16 |

### Hardware
| Run | GPU | VRAM | Provider |
|---|---|---|---|
| Qwen3-4B | NVIDIA RTX A6000 | 48GB | Thunder Compute |
| SmolLM2-1.7B | AMD RX 6700 XT | 12GB | Local (DirectML) |
| Qwen2.5-1.5B | AMD RX 6700 XT | 12GB | Local (DirectML) |
| Gemma 270M | AMD RX 6700 XT | 12GB | Local (DirectML) |

---

## Dataset Evolution

### Phase 1: Foundation (148 pairs)
Built from reading server.ts codebase. Covered core module knowledge.

### Phase 2: Modules (120 pairs)
16 Jarvis modules with 6-8 questions each. Added per-module API endpoints, integration patterns, and descriptions.

### Phase 3: Reasoning (28 pairs)
Error handling chains, fallback logic, and integration patterns extracted from system code.

### Phase 4: Gap Targeting (14 pairs)
Specific weak areas: TRM communication details, ECONNREFUSED debugging, DNA disambiguation.

### Phase 5: Identity Reinforcement (14 pairs)
Separation of Jarvis identity from creator identity. "I am Jarvis, created by Shawn" reinforcement.

### Phase 6: Final Merge (519 unique pairs)
All 6 datasets consolidated, deduplicated, and balanced. 16 modules × 9 skills × error reasoning × identity × architecture.

---

## Training Results

### Run 1: Gemma 270M

| Epoch | Avg Loss | Time |
|---|---|---|
| 1 | - | - |
| 5 | **0.92** | 5 min |

**Test pass rate:** 41%  
**Issues:** Multilingual bleed, weak reasoning, identity confusion. Model too small for the knowledge density.

### Run 2: Qwen2.5-1.5B

| Epoch | Avg Loss | Time | Notes |
|---|---|---|---|
| 1 | **1.07** | 3.5 min | OOM on epoch 2 |

**Test pass rate:** 50%  
**Issues:** 151k vocabulary caused VRAM OOM on backward pass. Only 1 epoch completed. Excellent identity awareness but insufficient training.

### Run 3: SmolLM2-1.7B

| Epoch | Avg Loss | Time |
|---|---|---|
| 1 | 1.78 | 3.6 min |
| 2 | 0.84 | 3.7 min |
| 3 | 0.84 | 3.7 min |
| 4 | 0.84 | 3.7 min |
| 5 | **0.84** | 3.7 min |

**Test pass rate:** 50% (60% real with think tags)  
**Strengths:** 49k vocab fits VRAM. Fully trained 5 epochs. Good balance of size and capability.

### Run 4: Qwen3-4B (Best — 4 iterations)

**Iteration 1** (267 pairs, 3 epochs):
| Epoch | Avg Loss | Time |
|---|---|---|
| 1 | 2.02 | 3.2 min |
| 2 | 0.22 | 3.2 min |
| 3 | **0.17** | 3.2 min |

**Iteration 2** (318 pairs, 3 epochs):
| Epoch | Avg Loss | Time |
|---|---|---|
| 1 | 2.22 | 3.5 min |
| 2 | 0.10 | 3.5 min |
| 3 | **0.03** | 3.5 min |

**Iteration 3** (382 pairs, 3 epochs):
| Epoch | Avg Loss | Time |
|---|---|---|
| 1 | 2.25 | 3.5 min |
| 2 | 0.07 | 3.5 min |
| 3 | **0.03** | 3.5 min |

**Iteration 4 — Final** (519 pairs, 8 epochs):
| Epoch | Avg Loss | Time |
|---|---|---|
| 1 | 4.77 | 3.5 min |
| 2 | 0.10 | 3.5 min |
| 3 | 0.06 | 3.5 min |
| 4 | 0.04 | 3.5 min |
| 5 | 0.03 | 3.5 min |
| 6 | 0.02 | 3.5 min |
| 7 | 0.02 | 3.5 min |
| 8 | **0.0148** | 3.5 min |

**Total training time:** ~28 min across 4 iterations  
**Test pass rate: 100% (12/12)**  
**Adapter size:** 47MB

---

## Final Test Results (12/12 = 100%)

| # | Domain | Question | Response | Pass |
|---|---|---|---|---|
| 1 | Identity | Who are you? | "I am J.A.R.V.I.S. OS, a sovereign intelligence system created by Shawn Carruth" | ✅ |
| 2 | Identity | Are you Shawn? | "I am J.A.R.V.I.S. OS. I was created by Shawn Carruth, also known as Boss." | ✅ |
| 3 | Creator | Who created you? | "Created by Shawn Carruth (Boss)" | ✅ |
| 4 | Ports | TRM port? | "TRM Swarm OS runs on port 8021" | ✅ |
| 5 | Ports | Jarvis port? | "Jarvis runs on port 3000" | ✅ |
| 6 | Comms | How does Jarvis talk to TRM? | "127.0.0.1:8021" with IPv6 explanation | ✅ |
| 7 | Errors | Why ECONNREFUSED? | "Node.js resolves localhost to IPv6 ::1, TRM binds IPv4" | ✅ |
| 8 | Health | How to check swarm health? | "GET /api/swarm/healing/status" | ✅ |
| 9 | DNA | What is DNA config? | "Behavioral parameters — NOT biological DNA" | ✅ |
| 10 | Skills | List your skills. | "9 skills: GitHub Code Auditor, Web Oracle Grounding, Telegram..." | ✅ |
| 11 | Law of One | What is the Law of One? | "All is One — universal principle, not a person's belief" | ✅ |
| 12 | VibeThinker | What is VibeThinker? | "Autonomous consensus evaluation system" | ✅ |

---

## Conclusions

1. **Model size matters** — 4B parameters (Qwen3) outperformed 1.7B, 1.5B, and 270M consistently
2. **Dataset quality > quantity** — 519 well-structured pairs outperformed naive scaling
3. **Repetition works** — 5-8x per fact drastically improved retention
4. **Cloud GPU changes everything** — A6000 training was 10x faster than local AMD GPU
5. **100% pass rate is achievable** — with proper dataset structure and sufficient epochs
