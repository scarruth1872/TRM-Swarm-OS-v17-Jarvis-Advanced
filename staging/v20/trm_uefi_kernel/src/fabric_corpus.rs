//! The readable half of the intelligence on the boot volume.
//!
//! `swarm_v2_nand_fabric.k3` is the 76 MB trit fabric. Beside it the drive carries
//! `swarm_v2_nand_fabric.text.jsonl` - 180 records of `{bytes, key, sha256, text}`, about 1.4 MB -
//! which IS small enough to hold in memory on bare metal, plus a 45 KB `turbovec` index.
//!
//! Mounting the fabric proves the volume is readable and proves nothing about use. Until now no boot
//! path read either companion, so the kernel reported a fabric it never opened and answered from an
//! in-memory synthetic model. This module loads the corpus and recalls from it, so an answer can
//! state the document it came from and how many query terms it matched - a basis, not a score.

use alloc::string::String;
use r_efi::efi;
use alloc::vec::Vec;

/// Ceiling on what will be held in memory. The drive's corpus is ~1.4 MB; this leaves room for a
/// larger one without letting a bogus size field ask for an unbounded allocation.
pub const MAX_CORPUS: usize = 8 * 1024 * 1024;

static mut CORPUS: Option<Vec<u8>> = None;
static mut DOCS: usize = 0;
static mut VOLUME: Option<usize> = None;
static mut VOLUMES_SEARCHED: usize = 0;

/// Load the readable corpus from ANY volume the firmware exposes.
///
/// Returns `(volume, bytes, documents)`. `volume` is None when no volume carries it - an absent
/// corpus is reported as absent, never as empty.
///
/// The corpus is SEARCHED for, not assumed to sit beside the boot image. "The full intelligence
/// resides on the drives": the boot volume is one of them, and hard-coding that assumption is what
/// made a corpus present on the volume report as not loaded.
///
/// SAFETY: bs must be live Boot Services. Requires the allocator.
pub fn load(bs: *mut efi::BootServices) -> (Option<usize>, usize, usize) {
    let path = crate::nand_fs::CORPUS_PATH;
    let nvol = unsafe { crate::nand_fs::volume_count(bs) };
    unsafe {
        VOLUMES_SEARCHED = nvol;
    }
    for i in 0..nvol {
        let size = match unsafe { crate::nand_fs::size_on_volume(bs, i, path) } {
            Some(s) if s > 0 && s <= MAX_CORPUS as u64 => s as usize,
            _ => continue,
        };
        let mut buf = Vec::with_capacity(size);
        buf.resize(size, 0u8);
        let got = match unsafe { crate::nand_fs::read_on_volume(bs, i, path, &mut buf) } {
            Some(n) if n > 0 => n,
            _ => continue,
        };
        buf.truncate(got);
        let docs = count_docs(&buf);
        unsafe {
            CORPUS = Some(buf);
            DOCS = docs;
            VOLUME = Some(i);
        }
        return (Some(i), got, docs);
    }
    (None, 0, 0)
}

/// Which volume the corpus was loaded from, if any.
pub fn volume() -> Option<usize> {
    unsafe { VOLUME }
}

/// How many volumes were enumerated during the search.
pub fn volumes_searched() -> usize {
    unsafe { VOLUMES_SEARCHED }
}

/// Whether a corpus is held in memory.
pub fn is_loaded() -> bool {
    unsafe { CORPUS.is_some() }
}

/// Bytes held.
pub fn bytes() -> usize {
    unsafe { CORPUS.as_ref().map(|v| v.len()).unwrap_or(0) }
}

/// Documents held (records, not lines - a pretty-printed file would over-count otherwise).
pub fn docs() -> usize {
    unsafe { DOCS }
}

/// Records = lines that open a JSON object.
fn count_docs(buf: &[u8]) -> usize {
    let mut n = 0usize;
    let mut at_line_start = true;
    for &b in buf {
        if at_line_start && b == b'{' {
            n += 1;
        }
        at_line_start = b == b'\n';
    }
    n
}

/// Byte substring search.
fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    (0..=hay.len() - needle.len()).find(|&i| &hay[i..i + needle.len()] == needle)
}

/// Copy the JSON string field `name` out of `line` into `out`, resolving the escapes JSON allows.
/// Returns false when the field is absent or is not a string. Non-printable bytes are dropped rather
/// than reinterpreted: the terminal renders ASCII, and a mangled glyph would be a wrong answer.
fn extract(line: &[u8], name: &str, out: &mut String) -> bool {
    out.clear();
    let mut needle = Vec::with_capacity(name.len() + 2);
    needle.push(b'"');
    needle.extend_from_slice(name.as_bytes());
    needle.push(b'"');
    let pos = match find(line, &needle) {
        Some(p) => p,
        None => return false,
    };
    let mut i = pos + needle.len();
    while i < line.len() && (line[i] == b' ' || line[i] == b':') {
        i += 1;
    }
    if i >= line.len() || line[i] != b'"' {
        return false;
    }
    i += 1;
    while i < line.len() {
        let c = line[i];
        if c == b'"' {
            return true;
        }
        if c == b'\\' && i + 1 < line.len() {
            let e = line[i + 1];
            out.push(match e {
                b'n' | b't' | b'r' => ' ',
                b'"' => '"',
                b'\\' => '\\',
                b'/' => '/',
                other => other as char,
            });
            i += 2;
            continue;
        }
        if (0x20..0x7f).contains(&c) {
            out.push(c as char);
        }
        i += 1;
    }
    true
}

/// Case-insensitive substring search; returns the offset of the first hit.
fn contains_ci(hay: &[u8], needle_lower: &[u8]) -> Option<usize> {
    if needle_lower.is_empty() || hay.len() < needle_lower.len() {
        return None;
    }
    let mut i = 0usize;
    while i + needle_lower.len() <= hay.len() {
        let mut ok = true;
        for (k, &n) in needle_lower.iter().enumerate() {
            let c = hay[i + k];
            if (if c.is_ascii_uppercase() { c + 32 } else { c }) != n {
                ok = false;
                break;
            }
        }
        if ok {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Query terms: alphanumeric runs of 3+ characters, lowercased.
fn terms(query: &str) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut cur: Vec<u8> = Vec::new();
    for &b in query.as_bytes() {
        if b.is_ascii_alphanumeric() {
            cur.push(if b.is_ascii_uppercase() { b + 32 } else { b });
        } else {
            if cur.len() >= 3 {
                out.push(cur.clone());
            }
            cur.clear();
        }
    }
    if cur.len() >= 3 {
        out.push(cur);
    }
    out
}

/// Recall the best document for `query`.
///
/// Returns `(matched_terms, key, snippet)`. `matched_terms` counts DISTINCT query terms found in the
/// document, so an answer states its basis - which record, how many terms - instead of a similarity
/// score it cannot derive. `None` when nothing matches, which is a result and is reported as one.
pub fn recall(query: &str) -> Option<(usize, String, String)> {
    let qs = terms(query);
    if qs.is_empty() || !is_loaded() {
        return None;
    }
    let buf: &[u8] = unsafe {
        match CORPUS.as_ref() {
            Some(v) => v.as_slice(),
            None => return None,
        }
    };
    let mut best: Option<(usize, String, String, usize)> = None;
    for line in buf.split(|&b| b == b'\n') {
        if line.first() != Some(&b'{') {
            continue;
        }
        let mut text = String::new();
        if !extract(line, "text", &mut text) || text.is_empty() {
            continue;
        }
        let hay = text.as_bytes();
        let mut hits = 0usize;
        let mut first: Option<usize> = None;
        for term in qs.iter() {
            if let Some(p) = contains_ci(hay, term) {
                hits += 1;
                if first.map_or(true, |f| p < f) {
                    first = Some(p);
                }
            }
        }
        if hits > 0 && best.as_ref().map_or(true, |b| hits > b.0) {
            let mut key = String::new();
            extract(line, "key", &mut key);
            best = Some((hits, key, text, first.unwrap_or(0)));
        }
    }
    best.map(|(hits, key, text, at)| {
        let hay = text.as_bytes();
        let start = at.saturating_sub(70);
        let end = (at + 150).min(hay.len());
        let mut snip = String::new();
        for &c in &hay[start..end] {
            snip.push(if (0x20..0x7f).contains(&c) { c as char } else { ' ' });
        }
        (hits, key, snip)
    })
}
