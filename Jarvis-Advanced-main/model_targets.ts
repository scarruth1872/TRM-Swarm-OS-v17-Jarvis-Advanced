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
