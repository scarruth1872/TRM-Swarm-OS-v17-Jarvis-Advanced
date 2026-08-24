# J.A.R.V.I.S COGNITIVE RECONSTRUCTORS: GEMINI AI SPECIFICATION
This specification configures high-level reasoning and grounding parameters for the `@google/genai` TypeScript client.

---

## 🎯 RECOMMENDED MODELS FOR J.A.R.V.I.S.

| Task Domain | Recommended Model | Rationale |
| :--- | :--- | :--- |
| **Simple Text summarizing & Biomimetic Predictions** | `gemini-3.5-flash` | Optimized for sub-second, structured JSON outputs without requiring paid flow confirmations. |
| **Advanced Cognitive Code Generation & Restructures** | `gemini-3.1-pro-preview` | Deep structural coding and architectural modifications. |
| **High-Scale Embeddings & Semantic Search** | `gemini-embedding-2` / `text-embedding-004` | Generating 3072-dimension and 768-dimension vectors for J.A.R.V.I.S.'s semantic episodic search layer. |

---

## 🔒 SECURITY & API INTEGRITY PARADIGMS

1. **Anti-Leakage Principle**:
   - `GEMINI_API_KEY` is loaded lazily via `process.env.GEMINI_API_KEY` inside `server.ts`.
   - Never inject variables prefixed with `VITE_` representing sensitive API keys. All core queries should run through server-side proxy handlers (e.g. `app.post("/api/biomimetic")`).
2. **Strict Payload Safety**:
   - Ensure Gemini returns raw JSON fields. Avoid string wrapper prefixes (e.g., `\`\`\`json`) by passing `responseMimeType: "application/json"` in the model's generation config blocks.
3. **Graceful Fallbacks**:
   - Always prepare dynamic local algorithms in case of server timeouts or key omissions. The client must remain beautiful and interactive, providing high-fidelity visual representations even during network disconnects.

---

## 🔬 BIOLOGY ANNOTATION PROMPTING PATTERNS

When querying Gemini for molecular folding predictions:
- Supply the full FASTA amino acid string block.
- Instruct the model to return structured properties representing the folding trajectory.
- Ensure the description explains how helices map locally over coordinates to provide high-quality biological explanations.

---

## 📡 WI-FI VISION & MULTI-AGENT PROMPTING PATTERNS

When querying Gemini to assist or interpret Wi-Fi CSI spatial streams:
- Frame Wi-Fi reflections as a non-invasive, privacy-first camera substitute.
- Represent bodily postures and cardiac rhythms as wave perturbations mapped directly to active virtual agent triggers (Aether, Vanguard, Forge).
- Anchor hazardous skeletal alignments (e.g., collapses) to immediate safety alerts.
