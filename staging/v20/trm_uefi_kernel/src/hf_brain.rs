//! TRM Swarm OS - hf_brain (no_std port).
//!
//! Two things live in this module, because two things live behind the name:
//!
//! 1. The strip's ternary kernel, ported 1:1 from
//!    `swarm_v2_artifacts/native_voltage_strips/hf_brain.rs`. That source is a SYNTHESIZED strip: it
//!    holds a 256-entry ternary weight vector `TW` and a scale `SF`, and `hf_brain_execute` computes
//!    `sum_j i[j] * TW[j] * SF / 16.0` (the 16 being TurboVec's quantization factor), classifies the
//!    result into K3 with a 1e-3 band around zero, and returns 1 -- or 0xDEAD_BEEF for a null/empty
//!    input. Inputs longer than TW are truncated, as in the strip.
//! 2. The host module's generation rules from `swarm_v2/core/hf_brain.py`, which ARE portable even
//!    though the inference around them is not: the prompt framing (including the fallback template),
//!    the decoding constants, and the `<think>` block extraction.
//!
//! Not ported, because firmware has no torch and no model weights: `_load_model`, `_generate_sync`'s
//! actual generation call, and `hf_chat`'s async wrapper.

extern crate alloc;
use alloc::string::String;

/// The strip's scale, extracted from its source rather than retyped.
pub const SF: f32 = 31.9498;

/// The strip's 256-entry ternary weight vector, extracted from its source.
pub const TW: [i8; 256] = [
    1, 1, 1, 1, 0, 0, 1, -1, -1, 1, 0, 0, -1, 0, -1, 0, 1, 1, 1, 0, -1, 1, 0, 1, 1, 1, 1, 0, -1, 0, 1, -1,
    1, 1, 1, 0, -1, 0, 0, 0, -1, 0, 1, -1, -1, 1, -1, 0, 1, 1, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0,
    0, 1, 1, 1, 0, -1, 1, 0, -1, 1, 1, -1, -1, 1, -1, -1, 1, 1, 1, 0, -1, 1, 1, 0, 1, 1, 1, 0, 0, 1, 0, -1,
    0, 0, 0, 0, 0, -1, 1, 0, 0, 1, 0, -1, 0, 1, 0, -1, 0, 1, 0, 1, -1, 0, 1, 1, 1, 1, 0, 1, -1, 1, 0, -1,
    1, 1, 0, 1, -1, -1, 1, -1, 0, 1, 0, 0, 0, 0, -1, 0, 0, 0, 0, 1, -1, 1, 1, 0, 1, 1, 0, 0, 0, 1, 0, -1,
    1, 0, 0, 1, -1, -1, 0, -1, -1, 0, 1, 0, 0, 0, -1, -1, 0, 1, 1, 1, -1, 0, 0, 0, 1, 0, 1, 0, -1, 1, 1, 0,
    1, 1, 0, 0, -1, -1, 0, 0, -1, 1, 1, -1, -1, 1, -1, 0, 1, 0, 1, 0, -1, 1, 1, 1, 0, 0, 0, 1, -1, 1, 1, -1,
    1, 0, 0, 0, -1, -1, 0, -1, -1, 1, 1, 0, -1, 1, -1, -1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, -1, 1, 1, -1,
];

/// The strip's score: `sum_j i[j] * TW[j] * SF / 16.0` over `min(len, 256)` dimensions, accumulated
/// left to right in f32 exactly as the strip does.
pub fn ternary_dot(i: &[f32]) -> f32 {
    let n = i.len().min(TW.len());
    let mut s: f32 = 0.0;
    for j in 0..n {
        s += i[j] * (TW[j] as f32) * SF / 16.0;
    }
    s
}

/// The strip's K3 band: `|r| < 1e-3` is UNKNOWN, positive is TRUE, negative is FALSE.
pub fn k3_of(r: f32) -> &'static str {
    if r.abs() < 1e-3 {
        "UNKNOWN"
    } else if r > 0.0 {
        "TRUE"
    } else {
        "FALSE"
    }
}

/// `hf_brain_execute`: (status, score). 0xDEAD_BEEF is the strip's own guard for a null or zero-length
/// input; 1 is its success code.
pub fn execute(i: &[f32]) -> (u32, f32) {
    if i.is_empty() {
        return (0xDEAD_BEEF, 0.0);
    }
    (0x0000_0001, ternary_dot(i))
}

// ── the host module's generation rules (swarm_v2/core/hf_brain.py) ───────────────────────────────

/// `max_new_tokens: int = 256` -- the python default.
pub const MAX_NEW_TOKENS_DEFAULT: u64 = 256;
/// `temperature=1.0`, `do_sample=False` (greedy for determinism), `repetition_penalty=1.2`.
pub const TEMPERATURE: f64 = 1.0;
pub const REPETITION_PENALTY: f64 = 1.2;
pub const DO_SAMPLE: bool = false;

/// The python's fallback prompt template, verbatim:
/// `f"<start_of_turn>user\n{system_prompt}\n\n{user_message}<end_of_turn>\n<start_of_turn>model\n"`.
/// The primary path uses the tokenizer's chat template, which does not exist in firmware.
pub fn frame_prompt(system_prompt: &str, user_message: &str) -> String {
    let mut s = String::new();
    s.push_str("<start_of_turn>user\n");
    s.push_str(system_prompt);
    s.push_str("\n\n");
    s.push_str(user_message);
    s.push_str("<end_of_turn>\n<start_of_turn>model\n");
    s
}

fn find_from(h: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || from + needle.len() > h.len() {
        return None;
    }
    let mut i = from;
    while i + needle.len() <= h.len() {
        if &h[i..i + needle.len()] == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn trim(s: &[u8]) -> &str {
    let mut a = 0usize;
    let mut b = s.len();
    while a < b && s[a].is_ascii_whitespace() {
        a += 1;
    }
    while b > a && s[b - 1].is_ascii_whitespace() {
        b -= 1;
    }
    core::str::from_utf8(&s[a..b]).unwrap_or("")
}

/// Remove every `<think>...</think>` block (non-greedy: each opener pairs with the next closer), which
/// is what the python's `re.sub(r'<think>.*?</think>', '', generated, flags=re.DOTALL)` does. An
/// unclosed `<think>` matches nothing and is therefore left in place, also as in python.
fn remove_think_blocks(b: &[u8]) -> String {
    let open = b"<think>";
    let close = b"</think>";
    let mut out: alloc::vec::Vec<u8> = alloc::vec::Vec::new();
    let mut i = 0usize;
    while i < b.len() {
        if i + open.len() <= b.len() && &b[i..i + open.len()] == open {
            if let Some(j) = find_from(b, i + open.len(), close) {
                i = j + close.len();
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The python's `<think>` handling: return (thought, content). Note the python's own shape, kept
/// here: `content` starts as the whole `generated` string UNSTRIPPED, and the strip-and-remove only
/// happens when a complete `<think>...</think>` pair is present.
pub fn split_thought(generated: &str) -> (String, String) {
    let b = generated.as_bytes();
    let open = b"<think>";
    let close = b"</think>";
    let mut thought = String::new();
    let mut content = String::from(generated);
    if let Some(i) = find_from(b, 0, open) {
        // `find_from` returns an ABSOLUTE index; an earlier version added it to the opener again, so
        // the thought slice ran past the closing tag. Both call sites take it as absolute now.
        if let Some(j) = find_from(b, i + open.len(), close) {
            thought = String::from(trim(&b[i + open.len()..j]));
            content = String::from(trim(remove_think_blocks(b).as_bytes()));
        }
    }
    (thought, content)
}
