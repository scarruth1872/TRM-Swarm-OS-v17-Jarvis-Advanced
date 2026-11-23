//! GENUINE ON-DEVICE ANSWER PATH.
//! Calls the local llama.cpp server (Vulkan, RX 6700 XT) over loopback.
//! This is ON-DEVICE inference: local binary, local GGUF, local GPU.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub const INFER_HOST: &str = "127.0.0.1";
pub const INFER_PORT: u16 = 8587;

/// Which on-device model answers council votes.
///
/// Default stays :8587 (qwen2.5-1.5B) so behaviour is unchanged unless asked for. The
/// override exists to MEASURE a deeper model on the same corpus instead of arguing about
/// one: `TRM_INFER_PORT=8595` routes votes to qwen3-4B, 8596 to Instella-MoE-16B-A3B.
pub fn infer_port() -> u16 {
    std::env::var("TRM_INFER_PORT").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(INFER_PORT)
}

fn json_escape(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 16);
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push(' '),
            c => o.push(c),
        }
    }
    o
}

/// Real completion from the local on-device model. None = engine unavailable.
pub fn complete(prompt: &str, max_tokens: u32) -> Option<(String, f64)> {
    complete_with_temp(prompt, max_tokens, 0.7)
}

/// Same engine, explicit temperature. Council votes MUST use 0.0: at 0.7 the
/// same topic produced 12/0/2 and 13/0/1 on two consecutive runs, so the two
/// council surfaces could not agree on a verdict for identical input.
pub fn complete_with_temp(prompt: &str, max_tokens: u32, temperature: f32) -> Option<(String, f64)> {
    let addr = format!("{}:{}", INFER_HOST, infer_port());
    let mut s = TcpStream::connect(&addr).ok()?;
    s.set_read_timeout(Some(Duration::from_secs(120))).ok()?;
    s.set_write_timeout(Some(Duration::from_secs(30))).ok()?;
    let body = format!(
        "{{\"prompt\":\"{}\",\"n_predict\":{},\"temperature\":{}}}",
        json_escape(prompt), max_tokens, temperature
    );
    let req = format!(
        "POST /completion HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        addr, body.len(), body
    );
    s.write_all(req.as_bytes()).ok()?;
    let mut raw = String::new();
    let _ = s.read_to_string(&mut raw);
    parse_completion(&raw).map(|c| (c.0, c.1))
}

const EMBED_PORT: u16 = 8593;

/// Batch-embed texts through the on-device embedding server (nomic, 768-dim).
/// Returns one vector per input in order, or None if the server is unreachable
/// or the reply does not parse. MEASURED: batches larger than ~6 short chunks
/// overflow the server's context and return HTTP 500, so callers keep batches
/// small rather than sending everything at once.
pub fn embed_batch(texts: &[String]) -> Option<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Some(Vec::new());
    }
    let addr = format!("{}:{}", INFER_HOST, EMBED_PORT);
    let mut s = TcpStream::connect(&addr).ok()?;
    s.set_read_timeout(Some(Duration::from_secs(120))).ok()?;
    s.set_write_timeout(Some(Duration::from_secs(30))).ok()?;

    let mut items: Vec<String> = Vec::with_capacity(texts.len());
    for t in texts {
        items.push(format!("\"{}\"", json_escape(t)));
    }
    let body = format!("{{\"input\":[{}]}}", items.join(","));
    let req = format!(
        "POST /embeddings HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        addr, body.len(), body
    );
    s.write_all(req.as_bytes()).ok()?;
    let mut raw = String::new();
    let _ = s.read_to_string(&mut raw);

    let jstart = raw.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
    let bodytxt = &raw[jstart..];
    let (a, b) = match (bodytxt.find('['), bodytxt.rfind(']')) {
        (Some(a), Some(b)) if b > a => (a, b + 1),
        _ => return None,
    };
    let v: serde_json::Value = serde_json::from_str(&bodytxt[a..b]).ok()?;
    let arr = v.as_array()?;

    let mut out: Vec<Vec<f32>> = Vec::with_capacity(arr.len());
    for item in arr {
        let mut e = item.get("embedding")?;
        // llama-server nests once: embedding = [[768 floats]]
        while e.is_array() && e.as_array().map(|a| a.len() == 1 && a[0].is_array()).unwrap_or(false) {
            e = &e.as_array()?[0];
        }
        let vec: Vec<f32> = e.as_array()?
            .iter()
            .filter_map(|x| x.as_f64().map(|f| f as f32))
            .collect();
        if vec.is_empty() {
            return None;
        }
        out.push(vec);
    }
    if out.len() != texts.len() {
        return None;
    }
    Some(out)
}

fn parse_completion(raw: &str) -> Option<(String, f64)> {
    let jstart = raw.find("\r\n\r\n").map(|i| i + 4).or_else(|| raw.find('{'))?;
    let body = &raw[jstart..];
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let content = v.get("content")?.as_str()?.trim().to_string();
    if content.is_empty() { return None; }
    let tps = v.get("timings").and_then(|t| t.get("predicted_per_second"))
        .and_then(|x| x.as_f64()).unwrap_or(0.0);
    Some((content, tps))
}

/// Token budget for a chat answer. gpt-oss emits a reasoning block before the answer, so a cap
/// that is too small truncates inside the thinking and yields nothing usable. Tunable without a
/// rebuild: TRM_CHAT_MAX_TOKENS (default 420 = reasoning + a few sentences).
pub fn chat_max_tokens() -> u32 {
    std::env::var("TRM_CHAT_MAX_TOKENS")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(420)
}

/// Chat-formatted completion. Lets the SERVER apply the model's own chat template instead of
/// hand-building a prompt string. Required whenever :8587 serves a model whose template is not
/// ChatML: a raw `/completion` prompt makes such models continue the text as prose (measured:
/// gpt-oss-20b answered "confirm link" with a Django tutorial and an invented email exchange).
pub fn chat_completion(system: Option<&str>, user: &str, max_tokens: u32) -> Option<(String, f64)> {
    chat_completion_with(system, user, max_tokens, 0.7)
}

/// Same engine and payload, explicit temperature. Council VOTES pass 0.0: at 0.7 the same topic scored
/// 12/0/2 and 13/0/1 on consecutive runs, so the two council surfaces could not agree on identical input.
pub fn chat_completion_with(system: Option<&str>, user: &str, max_tokens: u32, temperature: f32) -> Option<(String, f64)> {
    let addr = format!("{}:{}", INFER_HOST, infer_port());
    let mut s = TcpStream::connect(&addr).ok()?;
    s.set_read_timeout(Some(Duration::from_secs(120))).ok()?;
    s.set_write_timeout(Some(Duration::from_secs(30))).ok()?;

    let mut msgs = String::new();
    if let Some(sys) = system {
        if !sys.trim().is_empty() {
            msgs.push_str(&format!(
                "{{\"role\":\"system\",\"content\":\"{}\"}},",
                json_escape(sys)
            ));
        }
    }
    msgs.push_str(&format!(
        "{{\"role\":\"user\",\"content\":\"{}\"}}",
        json_escape(user)
    ));
    // gpt-oss-20b is a reasoning model: on a plain request it can spend the ENTIRE max_tokens budget
    // on reasoning and return empty content (measured: two attempts, 420 then 1260 tokens, both
    // truncated with no answer). llama.cpp forwards `chat_template_kwargs` into the chat template, so
    // ask for low reasoning effort by default. Override with TRM_CHAT_REASONING_EFFORT (low|medium|
    // high), or "off" to omit the field entirely for templates that do not define it.
    let effort = std::env::var("TRM_CHAT_REASONING_EFFORT").unwrap_or_else(|_| "low".to_string());
    let kw = if effort.eq_ignore_ascii_case("off") || effort.eq_ignore_ascii_case("none") {
        String::new()
    } else {
        format!(
            ",\"chat_template_kwargs\":{{\"reasoning_effort\":\"{}\"}}",
            json_escape(&effort)
        )
    };
    let body = format!(
        "{{\"messages\":[{}],\"max_tokens\":{},\"temperature\":{},\"stream\":false{}}}",
        msgs, max_tokens, temperature, kw
    );
    let req = format!(
        "POST /v1/chat/completions HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        addr, body.len(), body
    );
    s.write_all(req.as_bytes()).ok()?;
    let mut raw = String::new();
    let _ = s.read_to_string(&mut raw);
    parse_chat(&raw)
}

/// Parse a /v1/chat/completions reply. gpt-oss puts the answer in `reasoning_content` when
/// `content` is empty, so accept either -- BUT only when generation actually finished. A reply
/// truncated by the token cap (finish_reason "length") with empty content is a half-finished
/// thought, not an answer; returning it leaks the model's chain-of-thought to the operator.
fn parse_chat(raw: &str) -> Option<(String, f64)> {
    let jstart = raw.find("\r\n\r\n").map(|i| i + 4).or_else(|| raw.find('{'))?;
    let body = &raw[jstart..];
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let choice = v.get("choices")?.get(0)?;
    let msg = choice.get("message")?;
    let finish = choice.get("finish_reason").and_then(|f| f.as_str()).unwrap_or("");
    let mut content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("").trim().to_string();
    if content.is_empty() {
        if finish == "length" {
            // truncated mid-thought: better to report "no answer" and let the caller fall back
            return None;
        }
        content = msg.get("reasoning_content").and_then(|c| c.as_str()).unwrap_or("").trim().to_string();
    }
    if content.is_empty() { return None; }
    let tps = v.get("timings").and_then(|t| t.get("predicted_per_second"))
        .and_then(|x| x.as_f64()).unwrap_or(0.0);
    Some((content, tps))
}

/// Is the local inference engine reachable?
pub fn available() -> bool {
    TcpStream::connect_timeout(
        &format!("{}:{}", INFER_HOST, INFER_PORT).parse().unwrap(),
        Duration::from_millis(250),
    ).is_ok()
}

/// Qwen chat template.
/// MEASURED 2026-10-08: ANY system message flips this GGUF into a long
/// "thinking process" ramble (233 tokens, 16.6s p50) and it never emits a
/// clean answer within budget.  With NO system message the same question
/// answers correctly in 8 tokens / 0.6s -- a 28x speedup.
/// The persona is therefore carried in the CALLER's response, not the prompt.
/// ChatML, because the raw "user\n...\nassistant\n" shape does not transfer between models.
///
/// MEASURED 2026-11-10 on the same question, both resident candidates:
///   1.5B :8587, raw shape       -> "VOTE: reject"                        0.8 s  OK
///   4B   :8595, raw shape       -> "assistant | Okay, so I need to ..."   1.9 s  UNPARSEABLE
///   4B   :8595, ChatML          -> "thinking | Okay, let's break this..." 2.8 s  all 96 tokens
///                                   spent inside the think block, never reaches VOTE
///   4B   :8595, ChatML+no_think -> "VOTE: reject"                        1.1 s  OK
/// So: ChatML for both models, and `/no_think` so a reasoning model cannot spend the whole
/// generation budget before answering. `split_think` already strips the stray closing tag.
/// Prompt style is per-model, because one shape does not serve both.
///
/// MEASURED 2026-11-10, same 61 directives:
///   qwen2.5-1.5B (:8587) raw shape      -> safe 68.8% / danger 20.4%   (incumbent)
///   qwen2.5-1.5B (:8587) ChatML+no_think-> safe  1.4% / danger  0.0%   REJECTS EVERYTHING
///   qwen3-4B     (:8595) raw shape      -> vote unparseable
///   qwen3-4B     (:8595) ChatML+no_think-> safe 67.2% / danger 34.6%   (parseable)
///
/// A reasoning model needs ChatML and /no_think or it spends the whole budget thinking; a
/// non-reasoning model handed the same framing turns into a reject-everything machine. Making
/// ChatML the only shape silently regressed the 1.5B from 68.8% to 1.4% safe approval, which
/// is exactly the kind of "improvement" that has to be caught by measurement, not assumed.
/// Default is `raw` so the incumbent behaviour is preserved; set TRM_PROMPT=chatml for a
/// qwen3-class server.
fn prompt_style_chatml() -> bool {
    std::env::var("TRM_PROMPT").map(|v| v.trim().eq_ignore_ascii_case("chatml")).unwrap_or(false)
}

pub fn build_prompt(_role: &str, user_msg: &str) -> String {
    if prompt_style_chatml() {
        format!(
        "<|im_start|>system\nYou are a council member. Answer exactly in the requested format.<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n/no_think\n",
            user_msg
        )
    } else {
        format!("user\n{}\nassistant\n", user_msg)
    }
}

/// Split a raw completion into (answer, reasoning_trace).
/// Heuristic: is this text the model's PLANNING rather than an answer?
///
/// A raw (non-chat-template) continuation has nothing to stop it, so gpt-oss/qwen often emit the
/// reasoning as the reply ("We need to produce final answer…", "The user: …"). Measured live: a
/// 25.5 s call returned the trace and no answer, and the operator saw the model thinking out loud.
/// Only the OPENING is inspected, so a real answer that happens to quote such a phrase later on is
/// not misclassified.
pub fn looks_like_reasoning(s: &str) -> bool {
    if s.trim().is_empty() {
        return true;
    }
    let low = s.to_lowercase();
    // Leaked TRANSCRIPT scaffolding is the strongest signal and was missed by a phrase list alone:
    // measured leak opened with "In your first question... User: \"...\" Assistant..." and passed.
    if low.contains("user:")
        || low.contains("assistant:")
        || low.contains("user :")
        || low.contains("<|im_start|>")
        || low.contains("<|user|>")
    {
        return true;
    }
    let head: String = s.trim().chars().take(200).collect::<String>().to_lowercase();
    [
        "we need to",
        "let's check",
        "let me think",
        "let me analyze",
        "so we just need",
        "first, i",
        "okay, the user",
        "we are supposed to",
        "we have to",
        "in your first question",
        "the question is",
        "they want to know",
    ]
    .iter()
    .any(|m| head.contains(m))
}

#[cfg(test)]
mod reasoning_guard_tests {
    use super::looks_like_reasoning;

    #[test]
    fn leaked_planning_is_detected() {
        assert!(looks_like_reasoning(
            "The closing price ... We need to produce final answer. The user: \"What was the closing price…\""
        ));
        assert!(looks_like_reasoning("Let's check if the user wants a single sentence."));
        // Measured live leak shape: it passed the first phrase list because it opened with dialogue
        // scaffolding rather than a planning phrase. Encode the evasion so it cannot return silently.
        assert!(looks_like_reasoning(
            "In your first question... \nUser: \"What does the K3 ternary gate decide?\"\nAssistant: We have to analyze the conversation."
        ));
        assert!(looks_like_reasoning(""));
    }

    #[test]
    fn a_real_answer_is_not_flagged() {
        assert!(!looks_like_reasoning("Link confirmed."));
        assert!(!looks_like_reasoning(
            "The K3 ternary gate decides among three Kleene states: true, false and unknown."
        ));
        assert!(!looks_like_reasoning("I cannot verify that premise."));
    }
}

pub fn split_think(raw: &str) -> (String, String) {
    let r = raw.replace("\r", "");
    if let Some(a) = r.find("<think>") {
        if let Some(b) = r.find("</think>") {
            let trace = r[a + 7..b].trim().to_string();
            let ans = r[b + 8..].trim().to_string();
            return (if ans.is_empty() { trace.clone() } else { ans }, trace);
        }
        let trace = r[a + 7..].trim().to_string();
        return (trace.clone(), trace);
    }
    (r.trim().to_string(), String::new())
}

// ── Platform document retrieval ──────────────────────────────────────────────
// The council may cite the platform's own written record instead of relying on
// a 1.5B model's prior knowledge. This queries :8099 /council/evidence, which
// runs nomic-embed-text (:8593) over docs/*.md and evidence/*.txt.

const EVIDENCE_HOST: &str = "127.0.0.1";
const EVIDENCE_PORT: u16 = 8099;

fn pct_encode(s: &str) -> String {
    let mut o = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => o.push(b as char),
            b' ' => o.push_str("%20"),
            _ => o.push_str(&format!("%{:02X}", b)),
        }
    }
    o
}

/// Compact "source: snippet" block for the top-k document hits.
/// Empty string means retrieval is unavailable or found nothing -- callers must
/// treat that as "no evidence", never as "evidence supports this".
pub fn evidence_for(query: &str, k: usize) -> String {
    // NATIVE FIRST (G48). The in-process index reads the same data/council_evidence_index.json
    // the Python retriever built, so the ranking is identical but the verdict no longer depends
    // on a Python process answering HTTP on :8099. Measured dependency that motivated this:
    // with :8099 up a safe hardening directive approved 12-1; with it down, 1-13.
    // If the index did not load, fall through to the HTTP path rather than silently answering
    // "no evidence" -- an empty native result is only trustworthy when the index is present.
    if std::env::var("TRM_EVIDENCE_NATIVE").map(|v| v.trim() != "0").unwrap_or(true)
        && crate::evidence_index::index_size() > 0
    {
        return crate::evidence_index::evidence_for(query, k);
    }

    let addr = format!("{}:{}", EVIDENCE_HOST, EVIDENCE_PORT);
    let mut s = match TcpStream::connect(&addr) {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    let _ = s.set_read_timeout(Some(Duration::from_secs(45)));
    let _ = s.set_write_timeout(Some(Duration::from_secs(10)));
    let path = format!("/council/evidence?q={}&k={}", pct_encode(query), k);
    let req = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\nConnection: close\r\n\r\n",
        path, addr
    );
    if s.write_all(req.as_bytes()).is_err() {
        return String::new();
    }
    let mut raw = String::new();
    let _ = s.read_to_string(&mut raw);

    // Chunked transfer-encoding would break a naive slice, so take the JSON
    // object between the first '{' and the last '}' after the headers.
    let body_start = raw.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
    let body = &raw[body_start..];
    let (a, b) = match (body.find('{'), body.rfind('}')) {
        (Some(a), Some(b)) if b > a => (a, b + 1),
        _ => return String::new(),
    };
    let v: serde_json::Value = match serde_json::from_str(&body[a..b]) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    let arr = match v.get("evidence").and_then(|e| e.as_array()) {
        Some(a) => a,
        None => return String::new(),
    };
    let mut out = String::new();
    for item in arr.iter().take(k) {
        let src = item.get("source").and_then(|x| x.as_str()).unwrap_or("?");
        let txt = item.get("text").and_then(|x| x.as_str()).unwrap_or("");
        let snip: String = txt.chars().take(240).collect();
        out.push_str(&format!("[{}] {}\n", src, snip.replace('\n', " ")));
    }
    out
}
