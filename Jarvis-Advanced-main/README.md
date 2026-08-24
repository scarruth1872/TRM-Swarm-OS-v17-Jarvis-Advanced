# J.A.R.V.I.S. OS

A highly polished, interactive, and autonomous cognitive companion modeled after the iconic digital butler interface. Powered by advanced Gemini multimodal comprehension models, J.A.R.V.I.S. maintains active self-awareness, permanent autobiographical memory layers, customizable tactical skills, and a real-time wireless messaging bridge via Telegram.

## Model Training (July 2026)

Four LoRA adapters have been trained on Jarvis system knowledge (318+ pairs, 16 modules, 9 skills):

| Model | Loss | Epochs | GPU | Adapter Size |
|---|---|---|---|---|
| **Qwen3-4B** 🏆 | **0.0148** | 8 | A6000 (48GB) | 47MB | **100%** |
| SmolLM2-1.7B | 0.84 | 5 | RX 6700 XT | 17MB | 50% |
| Qwen2.5-1.5B | 1.07 | 1 | RX 6700 XT | 17MB | 50% |
| Gemma 270M | 0.92 | 5 | RX 6700 XT | 5.6MB | 41% |

Best adapter: `jarvis_qwen3_adapter/` (loss 0.0148, 100% pass). Model registry: `jarvis_model_registry.json`.

---

## Component Architecture

Jarvis OS is constructed using a high-fidelity client-server model designed to combine interactive, low-latency browser portals with secure, state-retaining server engines.

```
                  ┌───────────────────────────────┐
                  │      Sensory Web Client      │
                  │ (Voice React, Camera Eye, UI) │
                  └───────┬───────────────▲───────┘
                          │ HTTP API      │ Server Sent
                          │ Requests      │ Status / HTML
                          ▼               │
┌─────────────────────────────────────────┴─────────────────────────────────────────┐
│                                 Express App Server                                │
├───────────────────────────────────────────────────────────────────────────────────┤
│                                     COGNITION                                     │
│  - Gemini Multimodal Brain (with continuous learning through custom tools)       │
│  - Background Telegram Uplink Daemon (continuous poll syncing updates)            │
├───────────────────────────────────────────────────────────────────────────────────┤
│                                 PERSISTENT STATE                                 │
│  - Core Memory Profile  - Episodic Memory Array  - Semantic Relations Graph Graph │
│  - Skills Capabilities  - Chat Buffer History    - User Key preferences           │
└───────────────────────────────────────────────────────────────────────────────────┘
```

---

## Core Pillars & System Capabilities

### 1. Multi-Tiered Memory Hierarchy & OKF Specialist Hub
J.A.R.V.I.S. is capable of true continuous, lifetime learning by dynamically classifying and persisting facts extracted during conversational turns. It is fully integrated with the **Open Knowledge Format (OKF) Spec v0.1**, allowing portable, vendor-neutral knowledge exchange:
- **Core Memory Profile** (`jarvis_core_memory.json`): Retains key biographical traits such as name, job title, and overarching system protocols.
- **Episodic Memory Sequence** (`jarvis_episodic_memory.json`): A temporal list cataloging recent occurrences, milestones, and important events.
- **Semantic Relationship Graph** (`jarvis_semantic_graph.json`): A multi-directional concept diagram pairing entities (e.g., `Boss` -> `Created` -> `TypeScript`) to maintain structural worldview knowledge.
- **Persistent Chat History** (`jarvis_chat_history.json`): Chronological log of conversation messages across browser interfaces and mobile telegram clients.
- **Open Knowledge Format (OKF) Hub**: A specialized suite that compiles J.A.R.V.I.S.' state dynamically into a compliant OKF portable bundle containing `okf.yaml` (manifest), `concepts/user_profile.md` (author/supervisor bio), `concepts/semantic_networks.md` (D3 relations tree), and `concepts/episodic_diaries.md` (episodic memories). It also supports **Cognitive Injection**, validating and parsing externally provided OKF markdown models back into the active J.A.R.V.I.S. neuron network.

### 2. Tactical Skills Subsystem
The core capability limits of J.A.R.V.I.S. can be modified dynamically via the **System Skills Registry**:
- Comes standard with four modules:
  1. **GitHub Code Auditor**: Reviews local repository structures for optimization opportunities.
  2. **Web Oracle Grounding**: Connects context engines directly to live open-web search indicators.
  3. **Telegram Uplink Bot Matrix**: Enables mobile and wireless integrations.
  4. **System Optimizer**: Reallocates container threads and clears internal diagnostics.
- Integrators can easily inject **Custom Script Skills** through the web interface, defining trigger prompts and terminal execution pathways (e.g. `npx tsx scripts/sensor-alert.ts`).

### 3. Integrated Wireless Uplink (Telegram Bot)
J.A.R.V.I.S. features native wireless mobilization through Telegram:
- **No Client Dependencies**: The Express backend spins up an autonomous background polling thread matching the `TELEGRAM_BOT_TOKEN`.
- **Dynamic Context Synchronization**: Messages sent via Telegram are pushed through the exact same Gemini semantic brain, retaining absolute access to current memory logs, knowledge states, and credentials.
- **Symmetric Delivery**: J.A.R.V.I.S.' replies are automatically routed back to your phone to facilitate swift on-the-go automation commands.

### 4. Interactive Sensory Inputs
The browser UI is styled with a "Cosmic Slate" aesthetic layout and supports:
- **Optical Multimodal Analysis**: Upload, capture, or stream camera frames into J.A.R.V.I.S.' vision matrix.
- **Web Speech Recognition**: Continuous listening to dictate spoken audio cues smoothly.
- **Auditory Synthesizer**: SpeechSynthesis voice synthesis rendering eloquent British butler vocal indicators locally.

### 5. ASCILINE Self-Correcting Cognitive Engine
J.A.R.V.I.S. integrates a powerful multi-stage inference self-criticism pipeline:
- **Three-Stage Iterative Loops**: Breaks down hypothesis exploration into drafting, critique, and convergence intervals during which errors are detected, logged, and corrected.
- **Visual Convergence Metrics**: Tracks and plots dynamic progression rates for mathematical Coherence Consensus (%) and Training/Optimization Loss.
- **Strict JSON Boundaries**: Runs server-side sanity cleaning that translates math expressions to stable Unicode sequences, preventing parsing crashes on complex character symbols.

### 6. Autonomous Robotics Simulation Lab (Correll Textbook Sandbox)
A high-fidelity dynamic sandbox replicating curriculum topics from Nikolaus Correll’s *Introduction to Autonomous Robots*:
- **Wheel Kinematics Sandbox**: Simulates differential drive state vectors, enabling adjustments to left/right wheel angular velocities (ωL, ωR), track width (L), and wheel radius (R) to chart instantaneous center of curvature (ICC).
- **Monte Carlo Localization (Particle Filters)**: Simulates active probabilistic state estimations. Projects particles using a prediction-and-correction cycle against discrete distance landmark checks, demonstrating sample survival and odometry drift recovery.
- **Grid Path Planning**: Real-time pathfinding visualization engine allowing toggles between heuristic A* (optimizing f(n) = g(n) + h(n)) and contour-tracing boundary-following Bug-2 algorithms.
- **Potential Fields (LIDAR sweeps)**: Visualizes gradient vectors that navigate a robot away from repulsive obstacles while maintaining attractive velocity pulls toward goal states, bolstered by high-density, multi-beam spatial LIDAR scan rays.

### 7. Phase VII Metacognitive Recalibration Portal
A specialized cryptographic subsystem for maintaining neural consensus alignment:
- **Metacognitive Cerebral Sync**: Monitors realtime noospheric frequencies and diagnostic streams to trace agent drift.
- **Cognitive Frequency Tuning**: Interactive delta/theta wave tuners syncing active memory graphs with visual biofeedback fields.
- **Vector Injections**: Injects high-dimensional calibration embeddings into active memory layers to self-correct cognitive fatigue.

### 8. Native Desktop & Mobile Integration (Local Model Sync)
An offline-first companion module enabling raw data sovereignty and direct localized hardware execution:
- **Direct Local LLM Handshake**: Integrates browser views directly with client-side inference servers like Ollama and LM Studio.
- **Native Wrapper Specifications**: Pre-configured build boilerplates for tauri (Rust-based ultra lightweight wrappers, <10MB package sizes) and Electron (Chromium frames bypassing CORS blocks using Node-level capabilities).
- **Mobile Companion Simulation**: Responsive touch-focused mobile interface (44px target sizes) featuring local Wi-Fi pairing state-indicators, device diagnostics (LTE & battery metrics), and secure biometric sensor verification gates.

### 9. Mixture of Experts (MoE) Local Fine-Tuning & Thunder Compute Orchestration
An end-to-end sandbox enabling actual training and alignment of highly efficient, sub-3B parameters Mixture-of-Experts architectures:
- **Smallest MoE Baselines**: Optimized for resource-constrained systems, enabling swift deployment of **Qwen-1.5-MoE-A2.7B** (0.93B active parameters per token, perfect for <3GB RAM devices) or **JetMoE-8B** (2.0B active / 8.0B total parameters, outperforming larger full LLaMA2-7B runs).
- **Interactive Fine-Tuning Monitor**: Visualizes training metrics, real-time cross-entropy step loss convergence, and live Top-2 routing distributions mapping token allocation across specific Expert neural heads (Reasoning, Database, Robotics, Systems, Episodic, Translative, Telemetry, and Auxiliary).
- **Cognitive Context Injector**: Dynamically extracts active J.A.R.V.I.S. episodic registry logs and D3 semantic relationship nodes, transforming them into high-dimensional data vector inputs to adjust gating weights without catastrophic forgetting.
- **Thunder Compute Cluster Integration**: High-performance cloud GPU container integration. Instant provisioning hooks allocate physical RTX 4090s, L4s, H100s, or A100 SXM5 architectures via `thundercompute.com` to kickstart actual deep learning tasks. Includes pre-configured Python scripts utilizing PEFT (LoRA/QLoRA) to fine-tune active gating networks directly over secure SSH tunnel bridges.

### 10. Biometric Wave-Space Vision Layer & Pulse Intelligence (Pulse-Fi & WiPose)
An ambient-intelligence sensory suite designed to achieve spatial awareness and physiological tracking without intrusive optical cameras:
- **RF-Space Positioning (WiPose)**: Real-time 3D orthographic projection rendering skeletal joint geometry from ambient radio frequency reflections. Includes interactive camera angles (Yaw, Pitch, Zoom) and posture tracking.
- **Pulse-Fi Vitals Resolver**: Extracts cardiac heartbeat spikes (QRS-like impulses) and deep diaphragmatic respiration rates directly from channel state amplitude perturbations.
- **UQI Swarm Interceptor Filters**: Includes customizable, multi-agent filters (PetFilter, FanOscillation, Multipath Suppression) to isolate biological waveforms from complex household electromagnetic noise.
- **Synaptic Graph Sync**: Instantly synthesizes spatial findings and high-risk posture events directly into J.A.R.V.I.S.' continuous memory profile and D3 semantic relationship loops.
- **UQI Multi-Agent Orchestration**: Monitored by cooperative virtual agents—Aether (fusion orchestration), Vanguard (realtime fall detection and physical wellness ethic auditing), Forge (3D movement grading), and Synaptic Navigator (contextual ambient voice synthesizers).

### 11. VibeThinker Reasoning Consensus & Subprocess Pipeline (WeiboAI Integration)
A high-performance logical verification system that executes Python-based multi-trajectory inference audits:
- **Subprocess Spawn Engine**: Launches `vibe_consensus.py` with custom CLI argument parsing and stderr diagnostic filters.
- **Claim-Level Reliability (CLR)**: Verifies logical steps by cross-verifying mathematical claims across 3 independent, varying temperature reasoning paths, generating weighted consensus confidence metrics.
- **Interactive Emerald Console**: Midnight emerald dashboard featuring parallel trajectory cards, step accordions, extracted facts, and agreement coefficients.

### 12. Semantic Vector Memory Recall (Gemini Embeddings)
Upgraded search pipeline utilizing multi-dimensional cloud-based semantic embeddings:
- **Cloud Embedding Core**: Connects with Gemini `gemini-embedding-2` / `text-embedding-004` API vectors.
- **Cosine Similarity Engine**: Ranks historical memory sequences dynamically in vector space.
- **Contextual Injector**: Retrieves the top 4 most semantically similar events and injects them as prompt context.

### 13. DeepMind Biomimetic Molecular Lab (AlphaFold & AlphaGenome APIs)
Fully integrated agentic biology suite for molecular diagnostics and genetic variant analysis:
- **AlphaFold 3 Pipeline**: Fetches real protein coordinates (MMCIF files) from the EBI database to visualize dimensions, active sites, and structural confidence (pLDDT) indicators.
- **AlphaGenome Pathogenicity Classifier**: Evaluates variant impact (such as CFTR F508del or KRAS G12D) across regulatory segments, splicing transitions, and clinical pathogenicity classifications.
- **Agentic Integration**: Wired up as autonomous LLM tools (`research_alphafold`, `research_alphagenome`) allowing J.A.R.V.I.S. to run biological diagnostics in response to chat prompts.

### 14. Interactive Career & Job Application Tracker
A high-fidelity Kanban-style career planning tool tailored to your profile:
- **Kanban Board Column Sync**: Tracks applications across Applied, Phone Screen, Interviewing, and Offer stages.
- **AI Suitability Scorer**: Evaluates job roles against your local profile details (Inspired-Fitness LLC, Network Administration, AV Training, and Systems Architecture) to calculate percentage fit scores.
- **Direct Search Shortcuts**: Direct access to major job listing hubs (LinkedIn, Indeed, ZipRecruiter) pre-filtered for your specific skillset.
- **Multimodal File Attachment Upload**: Upload and analyze resumes, PDFs, or audio notes directly within the chat input bar.

---

## REST API Interface Diagrams

### Cognition Matrix
* **`POST /api/chat`**
  * Submits chat inputs to the agent.
  * **Payload Schema:**
    ```json
    {
      "message": "Verify the telemetry, Jarvis.",
      "history": [],
      "visualData": "data:image/jpeg;base64,..."
    }
    ```
  * **Response:** Returns synthesized textual responses along with data detailing secondary agentic tool triggers.

* **`POST /api/vibe/consensus`**
  * Spawns the VibeThinker Python subprocess to perform multi-trajectory consensus validation.
  * **Payload Schema:**
    ```json
    {
      "prompt": "A box contains 3 red balls and 7 blue balls...",
      "model": "Ternary-Bonsai-4B-Q2_0.gguf"
    }
    ```
  * **Response:** Returns JSON object containing the consensus answer, confidence score, best trajectory details, and all trajectory steps.

* **`POST /api/asciline/process`**
  * Spawns an ASCILINE multi-pass iterative hypothesis correction routine.
  * **Payload Schema:**
    ```json
    {
      "hypothesis": "Describe the thermal transfer dissipation properties of a graphene shield."
    }
    ```
  * **Response:** Returns a compiled structured intelligence report mapping three sequential correction stages, mathematical TRP formulas, conflict ratio scores, and integration playbooks.

### Memory Configurations
* **`GET /api/memory`**: Fetches the core user profile, episodic records, and semantic connections.
* **`POST /api/memory/profile`**: Modifies user credentials (alias, occupation, or primary objectives).
* **`POST /api/memory/clear`**: Purges cognitive files and reinitializes with clean states.
* **`GET /api/chat-history`**: Pulls logs of previous conversations.
* **`POST /api/chat-history/clear`**: Flushes conversational buffers.
* **`POST /api/memory/import-okf`**: Validates and imports external markdown documentation conforming to Open Knowledge Format v0.1. Parses metadata frontmatter properties and relates source/target entities dynamically.

### Skills Control
* **`GET /api/skills`**: Lists active and deactivated capabilities.
* **`POST /api/skills/toggle`**: Enables or disables a functional module by ID.
* **`POST /api/skills/add`**: Registers a new script skill to the database.

### Telegram Controls
* **`POST /api/telegram/config`**: Registers a Bot API key token and initiates background terminal listeners.
* **`GET /api/telegram/status`**: Reports online sync statuses.

---

## Quickstart & Local Setup

### 1. Requirements
Ensure you have the following installed on your host system:
* Node.js v18 or newer
* npm package manager
* **Optional (For Local Inference & GPU Acceleration):**
  * Ollama (runs DeepSeek R1 natively on Windows AMD GPUs via ROCm).
  * Docker Desktop (configured with WSL2 backend).
  * Vulkan compatible drivers (for GPU accelerated Bonsai/BitNet llama.cpp containers compiled with `-DGGML_VULKAN=ON`).

### 2. Environment Setup
Create a `.env` configuration file in the project's root folder utilizing `.env.example` as a template:
```env
# Gemini API Brain Key
GEMINI_API_KEY=your_api_key_here

# Telemetry Wireless bot credentials (Optional)
TELEGRAM_BOT_TOKEN=your_telegram_bot_token

# Thunder Compute GPU token (Optional)
THUNDER_COMPUTE_TOKEN=your_thunder_compute_token_here
```

### 3. Dependency Installation
Initialize the sandbox environment's configurations:
```bash
npm install
```

### 4. Execute Development Services
Boot the multi-threaded Express-Vite backend:
```bash
npm run dev
```
The client portal resolves at `http://localhost:3000`.

### 5. Production Compilation
Create optimized and bundled executable builds:
```bash
npm run build
```
This optimizes both server-side transpilation and frontend static compilations into standard self-contained CJS packages ready for Cloud container runtimes.

---

## Google AI Studio -> GitHub Synchronization & Troubleshooting

If you are experiencing problems saving, pushing, or linking your active workspace directly to a remote GitHub repository from the digital sandbox interface, perform the following troubleshooting operations to secure your updates:

### Method A: Direct Export via Settings Menu (Fastest)
1. In the Google AI Studio menu bar, click on **Settings** (located in the top-right corner of the browser view).
2. Look for the **"Export to GitHub"** option. This will request your authorization key to push directly to a public or private repository.
3. If authorization fails or hangs:
   - Click **"Download ZIP"** from the Settings menu.
   - Extract the project ZIP stream to your local machine.

### Method B: Manual CLI Sync (Local Fallback)
If the browser environment fails to connect to upstream git objects, you can initialize a standard local git tracking tree via any standard command-line shell:

```bash
# 1. Initialize local registry files
git init

# 2. Add remote origin linking to your repository target
git remote add origin https://github.com/shawncarruth/jarvis-os-moe-hub.git

# 3. Cache modifications
git add .

# 4. Commit code updates
git commit -m "feat: integrate Mixture of Experts alignment rig and Thunder Compute bridge"

# 5. Set default tracking branch
git branch -M main

# 6. Push local elements securely to GitHub
git push -u origin main
```
This ensures your files, cognitive configurations, and local training scripts are always synchronized and fully accounted for.

---