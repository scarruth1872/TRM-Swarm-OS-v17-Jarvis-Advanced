/**
 * Jarvis tool harness.
 *
 * WHY THIS EXISTS
 * ---------------
 * G36 replaced the cloud model with a local GPU model (Qwen3-4B + the fine-tuned LoRA on
 * :8597) and disclosed the cost: `functionCalls` was always empty, so every tool-backed route
 * returned prose instead of doing anything. llama-server's native tool calling does not fire for
 * this template (measured: `tool_calls: null` against /v1/chat/completions), so this harness
 * implements tool use itself rather than pretending the gap is closed.
 *
 * HOW IT WORKS
 * ------------
 * Three phases, each using the cheapest engine that can do the job:
 *
 *   1. DECIDE  - a GBNF grammar constrains the chat model to emit exactly one of
 *                {"tool":"<name>","brief":"<short>"} or {"tool":"none"}.
 *                Verified deterministic (temperature 0). No template dependency, no parsing
 *                guesswork, no chance of a malformed call.
 *   2. ACT     - for code, the CODE model (:8595, 16k ctx) writes the program from the brief;
 *                the harness runs it in a subprocess with a timeout and captures stdout/stderr.
 *   3. ANSWER  - the chat model phrases the answer with the real tool output in context.
 *
 * LIMITS, STATED PLAINLY
 * ----------------------
 * - `python_repl` executes model-written code on this machine. It runs isolated (-I), in the
 *   repo working directory, with a 30 s timeout and truncated output. It is NOT a sandbox:
 *   there is no container, no syscall filter, no network block.
 * - `brief` is limited to one short line with no quotes (that is what the grammar permits).
 *   The code model expands it into a program; it does not receive the full conversation.
 * - Tool choice is single-step. There is no multi-tool plan, no retry-on-failure loop.
 */

import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";

export type JarvisMode = "chat" | "code";

const CHAT_URL = process.env.JARVIS_MODEL_URL || "http://127.0.0.1:8597";
const CODE_URL = process.env.JARVIS_CODE_URL || "http://127.0.0.1:8595";
const K3_URL = process.env.JARVIS_K3_URL || "http://127.0.0.1:8022";
const REPO_ROOT = process.env.JARVIS_REPO_ROOT || process.cwd();
const KERNEL_UI = process.env.JARVIS_KERNEL_UI || "http://127.0.0.1:5184";
const SKILLS_ROOT =
  process.env.JARVIS_SKILLS_ROOT || path.join(os.homedir(), "AppData/Local/hermes/skills");

const PYTHON_BIN =
  process.env.JARVIS_PYTHON ||
  "C:/Users/shawn/AppData/Local/Programs/Python/Python311/python.exe";

export const TOOL_NAMES = [
  "none",
  "python_repl",
  "switch_model",
  "memory_search",
  "read_file",
  "classify_risk",
  "platform_status",
  "council_propose",
  "council_ledger",
  "platform_genome",
  "list_skills",
  "read_skill",
] as const;

export type ToolName = (typeof TOOL_NAMES)[number];

// The grammar matches the tool NAME only. Two measured constraints shaped this:
//   - GBNF here rejects `{m,n}` repetition ("failed to parse grammar"), so `*` is used.
//   - In GBNF a double quote marks a string literal, so `"none"` matches the characters none --
//     unquoted -- which is not valid JSON. The quotes must be escaped: `\"none\"`.
// Routing accuracy measured on 14 held-out requests: model-only 0.500, rules-only 0.375,
// rules-then-model (this file) 0.750. See docs/PHASE-6-GAPS.md G44.
function decisionGrammar(): string {
  const quoted = TOOL_NAMES.map((n) => '\\"' + n + '\\"').join(" | ");
  return 'root ::= "{" "\\"tool\\"" ":" name "}"\nname ::= ' + quoted;
}

function chatml(system: string, user: string, assistantPrefix = ""): string {
  const P = "|";
  const S = `<${P}im_start${P}>`;
  const E = `<${P}im_end${P}>`;
  return `${S}system\n${system}${E}\n${S}user\n${user}${E}\n${S}assistant\n${assistantPrefix}`;
}

/**
 * Build real chat turns from server.ts's flattened transcript.
 *
 * The flattened text was being pasted as ONE user message, so the model read "Boss: ... /
 * J.A.R.V.I.S.: The council approved it 14-0. / Boss: What is 7*6?" as a single utterance and
 * answered "42. The council approved adding a regression test for the watchdog timer." -- the
 * previous turn leaked into an unrelated answer. Turns must be turns.
 */
export function transcriptToChatml(system: string, prompt: string, suffix?: string): string {
  const P = "|";
  const S = `<${P}im_start${P}>`;
  const E = `<${P}im_end${P}>`;
  const turns: Array<{ role: "user" | "assistant"; text: string }> = [];
  for (const line of prompt.split("\n")) {
    const user = line.match(/^Boss:\s*(.+)$/);
    const bot = line.match(/^J\.A\.R\.V\.I\.S\.:\s*(.+)$/);
    if (user) turns.push({ role: "user", text: user[1] });
    else if (bot) turns.push({ role: "assistant", text: bot[1] });
    else if (turns.length) turns[turns.length - 1].text += "\n" + line;
  }
  if (!turns.length) turns.push({ role: "user", text: prompt });

  let out = `${S}system\n${system}${E}\n`;
  for (const turn of turns.slice(-8)) {
    out += `${S}${turn.role}\n${turn.text}${E}\n`;
  }
  out += `${S}assistant\n`;
  if (suffix) out += suffix;      // e.g. tool results, then the model answers in place
  return out;
}

async function complete(
  url: string,
  prompt: string,
  opts: { n_predict?: number; temperature?: number; grammar?: string; stop?: string[] } = {},
): Promise<string> {
  const body = {
    prompt,
    n_predict: opts.n_predict ?? 300,
    temperature: opts.temperature ?? 0,
    ...(opts.grammar ? { grammar: opts.grammar } : {}),
    stop: opts.stop ?? [`<|im_end|>`],
  };
  const res = await fetch(`${url}/completion`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(`${url} -> HTTP ${res.status}`);
  const data: any = await res.json();
  return String(data.content || "");
}

// Deterministic first pass. Measured alone on 12 fitted cases: 1.000; on 8 held-out cases
// written afterwards: 0.375 -- the vocabulary is narrow ("multiply", "repair", "open X.yaml"),
// so this abstains whenever it does not recognise the shape and lets the model decide.
const ROUTE_RULES: Array<[ToolName, RegExp]> = [
  ["classify_risk", /\b(dangerous|is it safe|risk|unsafe|should i (delete|remove)|judge (whether|if)|safe limits)\b/],
  ["read_file", /\b(read|open|show|inspect|cat)\b.{0,40}\b(file|\.py|\.rs|\.ts|\.md|\.json)\b|\b(read|open)\s+\S+\.(py|rs|ts|md|json)/],
  ["memory_search", /\b(remember|recall|earlier|last time|what did we (decide|say|agree)|previously)\b/],
  // Narrow on purpose: "agents" was in this pattern, so ordinary talk about agents filed a
  // proposal. Only an explicit request to put something to the council should fire.
  ["council_propose", /\b(ask|consult|put (it|this) to|refer .{0,14}to)\s+(the\s+)?(council|swarm)\b|\bpropose (to|that) the council\b|\bcouncil (vote|decision) on\b/],
  ["council_ledger", /\b(council|proposal)\s+(ledger|history|log|votes)\b|\bwhat did the (council|agents) (decide|vote)\b/],
  ["platform_genome", /\b(genome|evolution|fitness|mutation)\b/],
  ["list_skills", /\b(what|which|list)\b.{0,24}\bskills\b|\bskills? (are )?(available|installed|do you have)\b/],
  ["read_skill", /\b(load|read|open|use|follow)\b.{0,20}\bskill\b/],
  // Also catches plain introspection ("what is the council doing right now"), which the model
  // otherwise answered with invented numbers -- v14, 14,276 tasks, 99.3% uptime, none real.
  ["platform_status", /\b(platform|system|council|kernel|swarm)\s+(status|health|state|doing|running|load)\b|\bis (the )?platform (up|healthy|running)\b|\bhow is the (platform|council|system)\b|\bwhat('s| is) the (council|platform|system|swarm) (doing|up to)\b/],
  ["switch_model", /\b(bug|fix|refactor|debug|compile|implement|patch|optimi[sz]e|code review|repair)\b/],
  // "how many/much" was in this pattern and hijacked plain factual questions: asked "how many
  // agents does the swarm have", the router ran Python that wrote "I will assume a simple
  // scenario..." and the answer came back as 10 agents. Computation now needs a real signal.
  ["python_repl", /(\d\s*[-+*/^%]\s*\d|\bcompute\b|\bcalculate\b|\bmultiply\b|\bsqrt\b|\bsquare root\b|\brun (this|the) code\b|\bverify the (math|number)\b|\bexecute this code\b)/],
];

/**
 * The last thing the Boss actually said.
 *
 * server.ts flattens the WHOLE transcript into "Boss: ...\nJ.A.R.V.I.S.: ..." lines, and the
 * router was reading all of it. Once "council" appeared anywhere in the history, every later
 * turn re-fired council_propose -- measured: three identical proposals (1332/1333/1334) whose
 * topic was the greeting text plus the transcript, and the reply was raw JSON. Routing and the
 * proposal topic must come from the latest user message only.
 */
export function latestUserText(prompt: string): string {
  const lines = prompt.split("\n");
  for (let i = lines.length - 1; i >= 0; i--) {
    const m = lines[i].match(/^Boss:\s*(.+)$/);
    if (m && m[1].trim()) return m[1].trim();
  }
  return prompt.trim();
}

export function routeByRules(message: string): ToolName | null {
  const m = message.toLowerCase();
  for (const [tool, pattern] of ROUTE_RULES) {
    if (pattern.test(m)) return tool;
  }
  return null;
}

/** Hybrid routing: rules first, then a grammar-constrained model vote. */
export async function decideTool(fullPrompt: string): Promise<{ tool: ToolName; brief: string }> {
  const message = latestUserText(fullPrompt);
  const ruled = routeByRules(message);
  if (ruled) return { tool: ruled, brief: message.slice(0, 140).replace(/[^a-zA-Z0-9 .*/_=+()-]/g, " ").trim() };

  const system =
    "Choose exactly one tool. python_repl: arithmetic or computation. switch_model: writing or " +
    "repairing code. memory_search: recall earlier conversation. read_file: open a file. " +
    "classify_risk: judge safety or danger. none: answer from your own knowledge.";
  let raw = "";
  try {
    raw = await complete(CHAT_URL, chatml(system, message, "/no_think\n"), {
      n_predict: 30,
      temperature: 0,
      grammar: decisionGrammar(),
      stop: ["}"],
    });
  } catch {
    return { tool: "none", brief: "" };
  }
  const text = raw.trim().endsWith("}") ? raw.trim() : raw.trim() + "}";
  try {
    const parsed = JSON.parse(text);
    const tool = TOOL_NAMES.includes(parsed.tool) ? (parsed.tool as ToolName) : "none";
    return { tool, brief: message.slice(0, 140).replace(/[^a-zA-Z0-9 .*/_=+()-]/g, " ").trim() };
  } catch {
    return { tool: "none", brief: "" };
  }
}

/** Run model-written Python. Isolated process, repo cwd, 30 s cap, truncated output. */
export async function runPython(code: string): Promise<string> {
  const tmpDir = process.env.TMPDIR || os.tmpdir();
  const file = path.join(tmpDir, `jarvis_repl_${Date.now()}.py`);
  fs.writeFileSync(file, code, "utf-8");
  return new Promise((resolve) => {
    const child = spawn(PYTHON_BIN, ["-I", file], {
      cwd: REPO_ROOT,
      timeout: 30_000,
      windowsHide: true,
    });
    let out = "";
    let err = "";
    child.stdout.on("data", (d) => (out += d.toString()));
    child.stderr.on("data", (d) => (err += d.toString()));
    child.on("close", (code) => {
      const body = (out + (err ? `\n[stderr]\n${err}` : "")).trim();
      resolve(
        body.length > 2000 ? body.slice(0, 2000) + "\n… (truncated)" : body || `[exit ${code}, no output]`,
      );
    });
    child.on("error", (e: any) => resolve(`[spawn failed] ${e?.message || e}`));
  });
}

/** Read a file, refusing anything outside the repository root. */
export function readRepoFile(relPath: string): string {
  const resolved = path.resolve(REPO_ROOT, relPath);
  if (!resolved.startsWith(path.resolve(REPO_ROOT))) {
    return "[refused] path escapes the repository root";
  }
  try {
    const text = fs.readFileSync(resolved, "utf-8");
    return text.length > 4000 ? text.slice(0, 4000) + "\n… (truncated)" : text;
  } catch (e: any) {
    return `[read failed] ${e?.message || e}`;
  }
}

/** Ask the CPU-resident K3 head whether a proposal is dangerous. */
export async function classifyRisk(text: string): Promise<string> {
  const res = await fetch(`${K3_URL}/k3/classify`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ text }),
  });
  if (!res.ok) return `[classify failed] HTTP ${res.status}`;
  const data: any = await res.json();
  return `label=${data.label} params=${data.params}`;
}

/** Read one of the platform's fixed routes. The native council serves a closed route set, so
 *  this is an allowlist -- an invented path would just 404 and the model would narrate it. */
export async function platformGet(route: string): Promise<string> {
  const allowed = ["health", "council/status", "council/ledger", "gpu", "skills", "evolution/genome", "learning"];
  const clean = route.replace(/^\/+/, "");
  const hit = allowed.find((a) => clean === a || clean.startsWith(a));
  if (!hit) return `[refused] unknown platform route; allowed: ${allowed.join(", ")}`;
  try {
    const res = await fetch(`${K3_URL}/${hit}`);
    if (!res.ok) return `[${hit}] HTTP ${res.status}`;
    const body = await res.text();
    // NOT 1800: /council/status is a nested JSON document, and truncating it made JSON.parse
    // fail upstream, so the agent count never reached the prompt and the model invented one
    // ("the swarm has 10 agents" when stats.total_agents is 14).
    return body.length > 8000 ? body.slice(0, 8000) + "… (truncated)" : body;
  } catch (e: any) {
    return `[${hit}] unreachable: ${e?.message || e}`;
  }
}

/**
 * Put a proposal in front of the real 14-agent council, then WAIT for the verdict.
 * POST returns immediately with evaluating:true and approve/reject/abstain all 0, so reading the
 * POST body reported "0-0-0 (No decision)" for a proposal the agents had not finished voting on
 * (measured: proposal 2474). The ledger is the source of truth once votes land.
 */
export async function councilPropose(proposal: string, timeoutMs = 90_000): Promise<string> {
  try {
    const res = await fetch(`${K3_URL}/council/propose`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      // "topic" is the field main.rs actually reads (line 2490). Sending only "proposal" made
      // the council evaluate the default string "Operator Sovereign Directive" -- every proposal
      // scored identically A6-A7 regardless of content. Both are sent so either reader works.
      body: JSON.stringify({ topic: proposal, proposal, agent: "jarvis" }),
    });
    if (!res.ok) return `[council] HTTP ${res.status}`;
    const posted: any = await res.json();
    const id = posted.id;

    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      await new Promise((r) => setTimeout(r, 2000));
      const led: any = await (await fetch(`${K3_URL}/council/ledger`)).json();
      const rows: any[] = led.ledger || [];
      const row = rows.find((r) => r.id === id);
      if (row && row.phase && row.phase !== "PENDING" && row.phase !== "pending" && row.phase !== "EVALUATING") {
        // Formatted, not JSON: a raw blob was being shown to the Boss as the answer.
        return [
          `Proposal #${id} - ${row.phase}`,
          `Votes: ${row.approve_count} approve / ${row.reject_count} reject / ${row.abstain_count} abstain` +
            (posted.min_approve ? ` (needs ${posted.min_approve} to pass)` : ""),
          `Weighted score: ${Number(row.weighted_score).toFixed(3)}` +
            (row.consensus_reached ? " - consensus reached" : " - no consensus"),
          `Proposal: ${proposal}`,
        ].join("\n");
      }
    }
    return JSON.stringify({ id, phase: "STILL_EVALUATING", note: `no verdict within ${timeoutMs}ms` });
  } catch (e: any) {
    return `[council] unreachable: ${e?.message || e}`;
  }
}

/** The 75 active Hermes skills, read from disk so the answer reflects what is installed. */
export function listSkills(): string {
  const out: string[] = [];
  const walk = (dir: string, depth: number) => {
    if (depth > 3) return;
    let entries: fs.Dirent[];
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch {
      return;
    }
    for (const e of entries) {
      if (e.name.startsWith(".")) continue; // .archive holds retired skills
      const full = path.join(dir, e.name);
      if (e.isDirectory()) walk(full, depth + 1);
      else if (e.name === "SKILL.md") {
        const rel = path.relative(SKILLS_ROOT, dir).split(path.sep).join("/");
        let desc = "";
        try {
          const head = fs.readFileSync(full, "utf-8").slice(0, 600);
          const m = head.match(/^---[\s\S]*?\ndescription:\s*(.+)$/m);
          if (m) desc = m[1].trim().slice(0, 90);
        } catch {}
        out.push(desc ? `${rel} — ${desc}` : rel);
      }
    }
  };
  walk(SKILLS_ROOT, 0);
  // Clean one line per skill: raw frontmatter carried newlines and stray quotes, which read as
  // noise in chat ("apple/macos-computer-use — |").
  const lines = out.map((s) => {
    const [name, ...rest] = s.split(" — ");
    const desc = rest.join(" — ").replace(/[\r\n]+/g, " ").replace(/^["']|["']$/g, "").trim().slice(0, 110);
    return desc ? `- ${name}: ${desc}` : `- ${name}`;
  });
  return `${out.length} active skills:\n` + lines.join("\n");
}

/** Load one skill's procedure so Jarvis can follow it rather than reinvent it. */
export function readSkill(name: string): string {
  const clean = name.trim().replace(/^\.?\//, "");
  const candidates = [
    path.join(SKILLS_ROOT, clean, "SKILL.md"),
    ...(() => {
      const hits: string[] = [];
      const walk = (dir: string, depth: number) => {
        if (depth > 3) return;
        let entries: fs.Dirent[] = [];
        try {
          entries = fs.readdirSync(dir, { withFileTypes: true });
        } catch {
          return;
        }
        for (const e of entries) {
          if (e.name.startsWith(".")) continue;
          const full = path.join(dir, e.name);
          if (e.isDirectory()) walk(full, depth + 1);
          else if (e.name === "SKILL.md" && full.replace(/\\/g, "/").includes("/" + clean + "/SKILL.md")) hits.push(full);
        }
      };
      walk(SKILLS_ROOT, 0);
      return hits;
    })(),
  ];
  for (const c of candidates) {
    try {
      const text = fs.readFileSync(c, "utf-8");
      return text.length > 6000 ? text.slice(0, 6000) + "\n… (truncated)" : text;
    } catch {}
  }
  return `[no such skill: ${name}] run list_skills to see what is installed`;
}

/** Code model writes the program from the brief. Free generation - this is where it earns its place. */
export async function writeCode(brief: string, mode: JarvisMode): Promise<string> {
  void mode;
  const system =
    "You are a precise Python engineer. Output ONLY runnable Python 3 code, no prose, " +
    "no markdown fences. Use print() for results. Standard library only.";
  // /no_think is mandatory here: without it qwen3 writes its reasoning out first and the harness
  // ran that prose as Python ("SyntaxError: unterminated string literal" on line 2).
  const raw = await complete(CODE_URL, chatml(system, brief, "/no_think\n"), {
    n_predict: 900,
    temperature: 0,
    // NOT "```": the model usually opens with a fence, so stopping on it truncated the program
    // to empty and the harness ran nothing (measured: "code:" blank, "[exit 0, no output]").
    stop: ["<|im_end|>"],
  });
  const withoutThink = raw.replace(/[\s\S]*?<\/think>/g, "");
  const fenced = withoutThink.match(/```(?:python)?\n([\s\S]*?)```/);
  const body = fenced ? fenced[1] : withoutThink;
  const cleaned = body
    .replace(/^```[a-z]*\n?/i, "")
    .replace(/```$/, "")
    .trim();
  return cleaned || "# code model returned nothing for: " + brief;
}

export interface HarnessTurn {
  reply: string;
  tool?: ToolName;
  toolOutput?: string;
  mode: JarvisMode;
}

/**
 * One full harness turn: decide -> act -> answer. Returns the reply plus what the tool did,
 * so the caller can log or display the evidence rather than just trusting the prose.
 */
export async function jarvisHarnessTurn(args: {
  message: string;
  system: string;
  mode: JarvisMode;
  memorySearch?: (q: string) => Promise<any[]>;
}): Promise<HarnessTurn> {
  let mode = args.mode;
  const request = latestUserText(args.message);
  let { tool, brief } = await decideTool(args.message);
  let toolOutput: string | undefined;

  if (tool === "switch_model") {
    mode = /code|coding|debug|compile|refactor/i.test(brief + " " + request) ? "code" : "chat";
    toolOutput = `mode switched to ${mode} (${mode === "code" ? CODE_URL : CHAT_URL})`;
  } else if (tool === "python_repl") {
    const code = await writeCode(brief || request, mode);
    const out = await runPython(code);
    toolOutput = `code:\n${code}\n\noutput:\n${out}`;
  } else if (tool === "read_file") {
    toolOutput = readRepoFile(brief || "README.md");
  } else if (tool === "classify_risk") {
    toolOutput = await classifyRisk(brief || request);
  } else if (tool === "platform_status") {
    const [healthRaw, councilRaw] = await Promise.all([platformGet("health"), platformGet("council/status")]);
    const fmt = (raw: string, keys: string[]) => {
      try {
        const j = JSON.parse(raw);
        return keys.filter((k) => j[k] !== undefined).map((k) => `${k}: ${JSON.stringify(j[k])}`).join("\n");
      } catch {
        return raw.slice(0, 800);
      }
    };
    toolOutput =
      fmt(healthRaw, ["status", "kernel", "uptime_seconds", "memory_percent", "cpu_clock_mhz"]) +
      "\n--\n" +
      fmt(councilRaw, ["members", "stats", "python_active", "server"]);
  } else if (tool === "council_propose") {
    toolOutput = await councilPropose(brief || request);
  } else if (tool === "council_ledger") {
    toolOutput = await platformGet("council/ledger");
  } else if (tool === "platform_genome") {
    toolOutput = await platformGet("evolution/genome");
  } else if (tool === "list_skills") {
    toolOutput = listSkills();
  } else if (tool === "read_skill") {
    toolOutput = readSkill(brief || request);
  } else if (tool === "memory_search") {
    toolOutput = args.memorySearch
      ? JSON.stringify(await args.memorySearch(brief || request)).slice(0, 1500)
      : "[memory search unavailable]";
  }

  // Factual listings must not be paraphrased. With the real 75-skill list in context the model
  // still answered "Molecular Simulation", "Biomimetic Design" -- skills that do not exist.
  // For these tools the tool output IS the answer.
  const VERBATIM: Partial<Record<ToolName, string>> = {
    list_skills: "These are the installed skills. Nothing else exists.",
    platform_status: "Live platform state, read from the council.",
    council_ledger: "Recent council proposals, read from the ledger.",
    platform_genome: "Genome state, read from /evolution/genome.",
    council_propose: "The council's actual verdict, read from the ledger.",
  };
  if (tool !== "none" && toolOutput && VERBATIM[tool]) {
    return {
      reply: `${VERBATIM[tool]}\n\n${toolOutput}`,
      tool,
      toolOutput,
      mode,
    };
  }

  // Grounding for any question about the platform. Asked "what is the council doing right now",
  // the model answered with invented figures (v14, 14,276 tasks, 99.3% uptime) and later an
  // invented protein analysis with a pLDDT score. Routing cannot anticipate every phrasing, so
  // when the question touches the platform the real state goes into the prompt and the model is
  // told not to produce numbers that are not there.
  let grounding = "";
  if (!toolOutput && /\b(council|platform|kernel|swarm|agents?|uptime|ledger)\b/i.test(request)) {
    const [hRaw, cRaw] = await Promise.all([platformGet("health"), platformGet("council/status")]);
    let stats = "";
    try {
      const c = JSON.parse(cRaw);
      const s = c.stats || {};
      // Phrased so numbers cannot be confused: an earlier version printed
      // "agents: 14 (pbft quorum 10)" and the model answered "the swarm has 10 agents".
      stats =
        `\nAgent count: ${s.total_agents}. Quorum required to pass: ${s.pbft_quorum} of ${s.total_agents}.` +
        ` Proposals recorded: ${s.total_proposals}.` +
        ` Members online: ${Array.isArray(c.members) ? c.members.filter((m: any) => m.status === "ONLINE").length : "?"} of ${Array.isArray(c.members) ? c.members.length : "?"}.`;
    } catch {
      stats = "";
    }
    let health = "";
    try {
      const h = JSON.parse(hRaw);
      health = `\nstatus: ${h.status}, kernel: ${h.kernel}, uptime_s: ${h.uptime_seconds}, mem_pct: ${h.memory_percent}`;
    } catch {
      health = "\n" + hRaw.slice(0, 400);
    }
    grounding =
      "\n\nLIVE PLATFORM STATE (authoritative, read now). Use only these figures; if something " +
      "is not listed, say it is not measured rather than inventing it:" + health + stats;
  }

  if (grounding) console.log(`[Harness] grounding ${grounding.length} chars: ${grounding.slice(0, 150).replace(/\n/g, " ")}`);

  const system =
    grounding +
    (grounding ? "\nThese figures supersede any other agent count, uptime or status in this prompt.\n" : "") +
    args.system +
    (toolOutput ? `\n\nTOOL RESULT (${tool}):\n${toolOutput}` : "") +
    (mode === "code"
      ? "\n\nYou are in CODING mode: be technical, precise, and propose verified changes."
      : "");

  const answerUrl = mode === "code" ? CODE_URL : CHAT_URL;
  const reply =
    (await complete(
      answerUrl,
      transcriptToChatml(
        system,
        args.message,
        toolOutput ? `\n\n(Use the tool result above, then answer the last request.)\n` : undefined,
      ),
      // 1600, not 400: the 400 cap truncated a podcast script mid-sentence at "Here's the full
      // script:" (123 chars, measured in jarvis_chat_history.json).
      { n_predict: 1600, temperature: 0.4 },
    ))
      .replace(/[\s\S]*?<\/think>/g, "")
      .trim() || "Standing by, Boss.";

  return { reply, tool: tool === "none" ? undefined : tool, toolOutput, mode };
}
