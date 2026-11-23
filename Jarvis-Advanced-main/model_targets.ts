/**
 * model_targets.ts — where the local model actually is, resolved at call time.
 *
 * WHY THIS EXISTS
 * ---------------
 * Three modules hardcoded a model port (`server.ts` and `jarvis_harness.ts` both defaulted to :8597,
 * the harness also to :8595) and the port that is live depends on which tier is resident:
 *
 *   :8587  veto / gate model   (tools/model_tiers.ps1 gate)      gpt-oss-20b
 *   :8596  analysis model      (tools/model_tiers.ps1 analysis)  Instella MoE
 *   :8597  spare / bench slot  (nothing normally listens here)
 *   :8595  qwen3-4b slot       (only when started on demand)
 *
 * Measured 2026-11-23: at least 16 `ECONNREFUSED 127.0.0.1:8597` failures in logs/jarvis_3000.log
 * while :8587 was serving the whole time, so the UI reported a model outage that did not exist.
 * A constant cannot answer "which model is up right now" — only a probe can.
 *
 * ORDER: 8596 before 8587 because both files build a ChatML prompt, which Instella speaks natively;
 * the gpt-oss gate model expects Harmony formatting and will answer a ChatML prompt poorly. A
 * dedicated slot still wins if it is up, and JARVIS_MODEL_URL overrides everything.
 */
const ENV_URL = process.env.JARVIS_MODEL_URL || "";
const ENV_CODE_URL = process.env.JARVIS_CODE_URL || "";

const PORTS = (process.env.JARVIS_MODEL_PORTS || "8596,8597,8587")
  .split(",")
  .map((s) => Number(s.trim()))
  .filter((n) => Number.isFinite(n) && n > 0);

let cachedModelUrl: string | null = ENV_URL || null;
let cachedCodeUrl: string | null = ENV_CODE_URL || null;

async function answers(url: string, timeoutMs = 800): Promise<boolean> {
  const ctrl = new AbortController();
  const timer = setTimeout(() => ctrl.abort(), timeoutMs);
  try {
    const r = await fetch(`${url}/health`, { signal: ctrl.signal });
    return r.ok;
  } catch {
    return false;
  } finally {
    clearTimeout(timer);
  }
}

/** The chat/analysis model. Re-probes when forced, or when nothing has been resolved yet. */
export async function resolveModelUrl(force = false): Promise<string> {
  if (cachedModelUrl && !force) return cachedModelUrl;
  if (ENV_URL) {
    cachedModelUrl = ENV_URL;
    return cachedModelUrl;
  }
  for (const port of PORTS) {
    const url = `http://127.0.0.1:${port}`;
    if (await answers(url)) {
      cachedModelUrl = url;
      return url;
    }
  }
  cachedModelUrl = null;
  throw new Error(
    `no local model answering on ${PORTS.join(", ")} — start one with ` +
      `tools\\model_tiers.ps1 gate|analysis or start_model.bat <alias>`,
  );
}

/** The coding model. Falls back to the chat model rather than to a dead port. */
export async function resolveCodeUrl(force = false): Promise<string> {
  if (cachedCodeUrl && !force) return cachedCodeUrl;
  if (ENV_CODE_URL) {
    cachedCodeUrl = ENV_CODE_URL;
    return cachedCodeUrl;
  }
  const chat = await resolveModelUrl(force);
  cachedCodeUrl = chat;
  return chat;
}

/** What the process is currently talking to, for /api/status style reporting. */
export function modelTargets(): { model: string | null; code: string | null; pool: number[] } {
  return { model: cachedModelUrl, code: cachedCodeUrl, pool: PORTS };
}

// ── kernel-mediated inference (the P6 hybrid the SRAM simulation selected) ────────────────────────
//
// Why this exists: /swarm/chat on :8022 runs the resident model through /v1/chat/completions, so the
// SERVER applies that model's own chat template. Jarvis was hand-building a ChatML prompt against a raw
// /completion port, which is why gpt-oss on :8587 leaked Harmony control tokens and why a ChatML prompt
// to a Harmony model answered poorly. The kernel also chooses which resident tier thinks, so Jarvis no
// longer needs to know a port at all.
//
// The direct-port path stays as the fallback: the SRAM simulation's TCAM frontier showed the
// kernel-only policy (P4) is the ONE candidate that dies when the kernel is down (S5), while the hybrid
// (P6) answers either way. Measured: kernel path 1.73 s and correctly grounded, against 6.39 s for the
// kernel without caller grounding and 16 refusals for the hardcoded port.
const KERNEL_URL = process.env.JARVIS_KERNEL_URL || "http://127.0.0.1:8022";

export async function kernelChat(
  system: string,
  message: string,
  role = "Jarvis",
  timeoutMs = 120_000,
): Promise<string | null> {
  const ctrl = new AbortController();
  const timer = setTimeout(() => ctrl.abort(), timeoutMs);
  try {
    const r = await fetch(`${KERNEL_URL}/swarm/chat`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      // `system` carries Jarvis's grounding. The kernel appends its own role framing and grounding rule,
      // so live figures survive the hop instead of being replaced by the kernel's static persona.
      body: JSON.stringify({ role, message, system }),
      signal: ctrl.signal,
    });
    if (!r.ok) return null;
    const data: any = await r.json();
    const text = String(data?.content ?? "").trim();
    return text.length > 0 ? text : null;
  } catch {
    return null;
  } finally {
    clearTimeout(timer);
  }
}

/** Where a chat request would go right now, for /api/status style reporting. */
export function chatPaths(): { kernel: string; directPool: number[] } {
  return { kernel: KERNEL_URL, directPool: PORTS };
}
