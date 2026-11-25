//! TRM Swarm OS — text reading primitives (no_std, alloc). Tasks 581, 584, 587.
//!
//! Pure functions over `&str`. File I/O stays in the host: these take text and return **byte offsets**, so a
//! caller can cite `file[offset..offset+len]` and the citation either resolves to the identical bytes or it
//! does not. That round-trip is the whole point — an evidence span without offsets is an opinion.
//!
//! # Offsets are byte offsets, and they are exact
//!
//! Every offset returned indexes the same `&str` that was passed in. Multi-byte UTF-8 is never split: the
//! scanners only break at ASCII delimiters (`\n`, `#`, `|`, `. `, digits), which cannot occur inside a UTF-8
//! continuation byte. A caller that concatenates or normalises the text before slicing will get wrong spans,
//! and [`Unit::text`] exists so the check is trivial to make.
//!
//! # Segmentation is conservative on purpose
//!
//! Sentence splitting refuses to break at a decimal point (`0.31`), at a known abbreviation (`e.g.`, `i.e.`,
//! `Fig.`, `vs.`, `approx.`, `No.`, `Dr.`, `et al.`) or inside a fenced code block. A splitter that is too
//! eager invents sentences that were never written; one that is too shy returns a paragraph, which the caller
//! can still cite. The failure modes are not symmetric, so this errs toward the larger unit.

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// What a parsed unit is. Markdown structure is the corpus's structure; there is no PDF or OCR path here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    Heading,
    Paragraph,
    Table,
    Code,
    List,
    Quote,
}

impl UnitKind {
    pub fn as_str(self) -> &'static str {
        match self {
            UnitKind::Heading => "heading",
            UnitKind::Paragraph => "paragraph",
            UnitKind::Table => "table",
            UnitKind::Code => "code",
            UnitKind::List => "list",
            UnitKind::Quote => "quote",
        }
    }
}

/// One structural unit, with the byte range it occupies in the source text.
#[derive(Debug, Clone)]
pub struct Unit {
    pub kind: UnitKind,
    pub start: usize,
    pub end: usize,
    /// 1-based line number of the unit's first line, so a reader can find it in an editor.
    pub line: usize,
    /// Heading text (without the leading `#`s) for `Heading` units, otherwise empty.
    pub heading: String,
    /// Heading level 1-6 for headings, 0 otherwise.
    pub level: u8,
}

/// Line span: (start, end excluding the newline, 1-based number).
fn lines(text: &str) -> Vec<(usize, usize, usize)> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut start = 0usize;
    let mut n = 1usize;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            let mut end = i;
            if end > start && bytes[end - 1] == b'\r' {
                end -= 1;
            }
            out.push((start, end, n));
            start = i + 1;
            n += 1;
        }
    }
    if start < text.len() {
        out.push((start, text.len(), n));
    }
    out
}

fn heading_prefix(line: &str) -> Option<(u8, usize)> {
    let mut level = 0u8;
    for c in line.chars() {
        if c == '#' {
            level += 1;
            if level > 6 {
                return None;
            }
        } else {
            break;
        }
    }
    if level == 0 {
        return None;
    }
    // require a space after the hashes (a lone "#hashtag" is not a heading)
    let after = line.get(level as usize..)?;
    if after.starts_with(' ') {
        Some((level, level as usize))
    } else {
        None
    }
}

/// Parse markdown into units with byte offsets. Fenced code blocks are kept whole; tables are kept as one
/// unit per contiguous run of `|` lines; list items are individual units.
pub fn parse_markdown(text: &str) -> Vec<Unit> {
    let ls = lines(text);
    let mut out: Vec<Unit> = Vec::new();
    let mut i = 0usize;
    while i < ls.len() {
        let (s, e, ln) = ls[i];
        let raw = &text[s..e];
        let trimmed = raw.trim_start();

        // fenced code block: consume until the closing fence (or EOF)
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let fence: &str = &trimmed[..3];
            let bs = s;
            let mut j = i + 1;
            let mut be = e;
            while j < ls.len() {
                let (s2, e2, _) = ls[j];
                be = e2;
                if text[s2..e2].trim_start().starts_with(fence) {
                    j += 1;
                    break;
                }
                j += 1;
            }
            out.push(Unit { kind: UnitKind::Code, start: bs, end: be, line: ln, heading: String::new(), level: 0 });
            i = j;
            continue;
        }

        if let Some((level, after)) = heading_prefix(trimmed) {
            out.push(Unit {
                kind: UnitKind::Heading,
                start: s,
                end: e,
                line: ln,
                heading: trimmed[after..].trim().to_string(),
                level,
            });
            i += 1;
            continue;
        }

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        if trimmed.starts_with('|') {
            let bs = s;
            let mut j = i;
            let mut be = e;
            while j < ls.len() {
                let (s2, e2, _) = ls[j];
                if !text[s2..e2].trim_start().starts_with('|') {
                    break;
                }
                be = e2;
                j += 1;
            }
            out.push(Unit { kind: UnitKind::Table, start: bs, end: be, line: ln, heading: String::new(), level: 0 });
            i = j;
            continue;
        }

        if trimmed.starts_with('>') {
            let bs = s;
            let mut j = i;
            let mut be = e;
            while j < ls.len() {
                let (s2, e2, _) = ls[j];
                if !text[s2..e2].trim_start().starts_with('>') {
                    break;
                }
                be = e2;
                j += 1;
            }
            out.push(Unit { kind: UnitKind::Quote, start: bs, end: be, line: ln, heading: String::new(), level: 0 });
            i = j;
            continue;
        }

        let is_list = trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("+ ")
            || trimmed.starts_with(|c: char| c.is_ascii_digit()) && trimmed.contains(". ");
        if is_list {
            out.push(Unit { kind: UnitKind::List, start: s, end: e, line: ln, heading: String::new(), level: 0 });
            i += 1;
            continue;
        }

        // paragraph: consecutive plain lines until a blank line or a line that starts another unit
        let bs = s;
        let mut j = i;
        let mut be = e;
        while j < ls.len() {
            let (s2, e2, _) = ls[j];
            let t2 = text[s2..e2].trim_start();
            if t2.is_empty()
                || heading_prefix(t2).is_some()
                || t2.starts_with("```")
                || t2.starts_with("~~~")
                || t2.starts_with('|')
                || t2.starts_with('>')
                || t2.starts_with("- ")
                || t2.starts_with("* ")
            {
                break;
            }
            be = e2;
            j += 1;
        }
        out.push(Unit { kind: UnitKind::Paragraph, start: bs, end: be, line: ln, heading: String::new(), level: 0 });
        i = j.max(i + 1);
    }
    out
}

/// A sentence (or whatever the splitter was able to delimit) with its byte range in the source text.
#[derive(Debug, Clone)]
pub struct Sentence {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

fn is_abbrev_before(text: &str, dot: usize) -> bool {
    const ABBREV: [&str; 10] = ["e.g", "i.e", "Fig", "fig", "vs", "approx", "No", "Dr", "Mr", "et al"];
    for a in ABBREV {
        let alen = a.len();
        // `dot - alen` is a byte offset computed by subtraction, and the text before the period may be
        // multi-byte (≥, –, §). Slicing there panicked on 21 of 40 corpus files; a boundary check is required
        // before every backwards slice, not just at the start.
        if dot >= alen && text.is_char_boundary(dot - alen) {
            let tail = &text[dot - alen..dot];
            if tail.eq_ignore_ascii_case(a) {
                // must be a word boundary before the abbreviation
                if dot == alen || !text.as_bytes()[dot - alen - 1].is_ascii_alphanumeric() {
                    return true;
                }
            }
        }
    }
    false
}

/// Byte ranges covered by fenced code blocks, including the fence lines themselves.
fn fence_ranges(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut open: Option<usize> = None;
    for (ls, le, _) in lines(text) {
        let t = text[ls..le].trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            match open {
                None => open = Some(ls),
                Some(bs) => {
                    out.push((bs, le));
                    open = None;
                }
            }
        }
    }
    if let Some(bs) = open {
        out.push((bs, text.len()));
    }
    out
}

/// Emit trimmed sentences from `text[from..to]`, with offsets relative to the ORIGINAL text.
fn split_plain(text: &str, from: usize, to: usize, out: &mut Vec<Sentence>) {
    let bytes = text.as_bytes();
    let mut start = from;
    let mut i = from;

    let mut push = |s: usize, e: usize, out: &mut Vec<Sentence>| {
        // trim surrounding whitespace while keeping offsets pointing at the surviving text
        let mut a = s;
        let mut b = e;
        while a < b && (bytes[a] as char).is_ascii_whitespace() {
            a += 1;
        }
        while b > a && (bytes[b - 1] as char).is_ascii_whitespace() {
            b -= 1;
        }
        if b > a {
            out.push(Sentence { start: a, end: b, text: text[a..b].to_string() });
        }
    };

    while i < to {
        let b = bytes[i];
        if b == b'\n' {
            push(start, i, out);
            start = i + 1;
            i += 1;
            continue;
        }
        if b == b'.' || b == b'!' || b == b'?' {
            let next_is_break = i + 1 >= to || bytes[i + 1].is_ascii_whitespace();
            let between_digits = b == b'.'
                && i > start
                && bytes[i - 1].is_ascii_digit()
                && i + 1 < to
                && bytes[i + 1].is_ascii_digit();
            let after_single_cap = b == b'.'
                && i >= 2
                && bytes[i - 1].is_ascii_uppercase()
                && !bytes[i - 2].is_ascii_alphanumeric();
            let abbrev = b == b'.' && is_abbrev_before(text, i);
            if next_is_break && !between_digits && !abbrev && !after_single_cap {
                push(start, i + 1, out);
                start = i + 1;
            }
            i += 1;
            continue;
        }
        i += 1;
    }
    push(start, to, out);
}

/// Split text into sentences, preserving exact byte offsets.
///
/// Fenced code blocks are excluded wholesale — a splitter that turns `let x = 1;` into a sentence is inventing
/// prose. Inside fenced blocks nothing is emitted, not even the fence markers.
///
/// A sentence ends at `.`, `!` or `?` followed by whitespace or end-of-region, except when the period follows a
/// single capital letter (an initial), follows a known abbreviation, or sits between digits (a decimal).
pub fn split_sentences(text: &str) -> Vec<Sentence> {
    let mut out = Vec::new();
    let mut cursor = 0usize;
    for (fs, fe) in fence_ranges(text) {
        if fs > cursor {
            split_plain(text, cursor, fs, &mut out);
        }
        cursor = fe;
    }
    if cursor < text.len() {
        split_plain(text, cursor, text.len(), &mut out);
    }
    out
}

/// A number found in text, with the unit token that follows it and its byte range.
#[derive(Debug, Clone)]
pub struct Quantity {
    pub value: f64,
    pub text: String,
    pub start: usize,
    pub end: usize,
    /// The bare unit/word after the number (`mg`, `%`, `compounds`, `t/s`), empty when none follows.
    pub unit: String,
}

/// Extract numbers with byte offsets. Parsing is deliberately literal: it takes digits, one decimal point and
/// an optional sign, and it records the token that follows so a caller can decide whether `0.4633` is a rate, a
/// score or a p-value. It never rounds and never infers magnitude from context.
pub fn quantities(text: &str) -> Vec<Quantity> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_digit() {
            let start = i;
            let mut j = i;
            while j < bytes.len() && (bytes[j].is_ascii_digit()) {
                j += 1;
            }
            // thousands separators: 2,403 / 1,234,567 — a digit group of exactly three after each comma
            while j + 3 < bytes.len()
                && bytes[j] == b','
                && bytes[j + 1].is_ascii_digit()
                && bytes[j + 2].is_ascii_digit()
                && bytes[j + 3].is_ascii_digit()
                && (j + 4 >= bytes.len() || !bytes[j + 4].is_ascii_digit())
            {
                j += 4;
            }
            if j + 1 < bytes.len() && bytes[j] == b'.' && bytes[j + 1].is_ascii_digit() {
                j += 1;
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
            }
            let num_text = &text[start..j];
            let parsed: Option<f64> = num_text.replace(',', "").parse::<f64>().ok().or_else(|| num_text.parse::<f64>().ok());
            if let Some(value) = parsed {
                // unit: next token of letters/%/slash
                let mut k = j;
                while k < bytes.len() && (bytes[k] == b' ') {
                    k += 1;
                }
                let ustart = k;
                while k < bytes.len()
                    && (bytes[k].is_ascii_alphabetic() || bytes[k] == b'%' || bytes[k] == b'/')
                {
                    k += 1;
                }
                let unit = text[ustart..k].to_string();
                out.push(Quantity { value, text: num_text.to_string(), start, end: j, unit });
            }
            i = j;
            continue;
        }
        i += 1;
    }
    out
}

/// Lowercase, collapse whitespace, drop punctuation that does not change meaning. Used to compare a claim
/// against a sentence without pretending to do stemming or synonymy.
pub fn normalise(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_space = true;
    for c in text.chars() {
        if c.is_whitespace() {
            if !last_space {
                out.push(' ');
                last_space = true;
            }
        } else if c.is_alphanumeric() || c == '.' || c == '%' || c == '-' || c == '_' {
            for lc in c.to_lowercase() {
                out.push(lc);
            }
            last_space = false;
        } else if !last_space {
            out.push(' ');
            last_space = true;
        }
    }
    out.trim().to_string()
}

/// Does `haystack` contain `needle` (case-insensitive, whitespace-normalised)? Returns the byte offset in the
/// ORIGINAL haystack where the match starts, so the caller can cite the real span.
/// Normalise and keep a map from every byte of the normalised string back to its byte in the original.
///
/// The mapping is the whole point: `normalise()` throws away positions, so a search that only had the normalised
/// string could say "it matches" and not say where. Two of thirteen QA citations failed exactly because of that.
pub fn normalise_map(text: &str) -> (String, Vec<usize>) {
    let mut out = String::with_capacity(text.len());
    let mut map: Vec<usize> = Vec::with_capacity(text.len());
    let mut last_space = true;
    let mut push_char = |c: char, orig: usize, out: &mut String, map: &mut Vec<usize>| {
        let mut buf = [0u8; 4];
        for b in c.encode_utf8(&mut buf).bytes() {
            out.push(b as char);
            map.push(orig);
        }
    };
    for (i, c) in text.char_indices() {
        if c.is_whitespace() {
            if !last_space {
                out.push(' ');
                map.push(i);
                last_space = true;
            }
        } else if c.is_alphanumeric() || c == '.' || c == '%' || c == '-' || c == '_' {
            for lc in c.to_lowercase() {
                push_char(lc, i, &mut out, &mut map);
            }
            last_space = false;
        } else if !last_space {
            out.push(' ');
            map.push(i);
            last_space = true;
        }
    }
    // trailing space is not part of the normalised form
    while out.ends_with(' ') {
        out.pop();
        map.pop();
    }
    (out, map)
}

/// Locate `needle` in `haystack` by normalised match and return the byte span `(start, end)` covering the
/// matched region **in the original text**.
///
/// This is what a citation needs: a chunk's sentence may be line-wrapped in the file, so normalising-away
/// whitespace is the only way to find it, but the cited bytes must still be the file's own bytes. The end is
/// computed from the map too, so the returned `haystack[start..end]` is a real slice that normalises to the
/// needle.
pub fn span_normalised(haystack: &str, needle: &str) -> Option<(usize, usize)> {
    let n = normalise(needle);
    if n.is_empty() {
        return None;
    }
    let (h, map) = normalise_map(haystack);
    let pos = h.find(&n)?;
    let start_idx = *map.get(pos)?;
    let last_norm = pos + n.len() - 1;
    let last_orig = *map.get(last_norm)?;
    let ch_len = haystack[last_orig..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
    Some((start_idx, last_orig + ch_len))
}

/// Locate `needle` in `haystack` (case-insensitive, whitespace-collapsed) and return the **byte offset in the
/// original haystack** where it begins. Returns `None` when the needle is absent or normalises to nothing.
pub fn find_normalised(haystack: &str, needle: &str) -> Option<usize> {
    let n = normalise(needle);
    if n.is_empty() {
        return None;
    }
    let (h, map) = normalise_map(haystack);
    let pos = h.find(&n)?;
    map.get(pos).copied()
}
