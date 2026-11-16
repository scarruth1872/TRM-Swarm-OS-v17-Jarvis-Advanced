//! TRM Swarm OS — Native parsers (no_std port).
//! 1:1 core semantics from swarm_v2/parsers/{fastq,vcf}_parser.py — Catalyst.
//! The Python parsers measure ~17,000 ns per FASTQ read on this host, linear in record count,
//! against an L1-resident floor of ~0.01 us for the same bytes: the gap is interpreter overhead,
//! not I/O. These are the same records computed natively, same key names, so the two compare
//! directly. Ternary (K3) is the classification primitive, not a by-product.

extern crate alloc;
use alloc::string::String;
use alloc::format;

/// A/G = +1 (purine), C/T = -1 (pyrimidine), anything else = 0 (unknown).
#[inline]
pub fn k3_trit(b: u8) -> i8 {
    match b {
        b'A' | b'G' => 1,
        b'C' | b'T' => -1,
        _ => 0,
    }
}

/// FASTQ summary — same keys as FASTQParser.parse_text.
pub fn parse_fastq(payload: &str) -> Result<String, String> {
    let (mut reads, mut bases, mut qsum, mut gc) = (0u64, 0u64, 0u64, 0u64);
    let mut trits: [i64; 3] = [0, 0, 0];
    for chunk in payload.split('@').skip(1) {
        let mut lines = chunk.lines();
        if lines.next().is_none() {
            continue;
        }
        let seq = match lines.next() {
            Some(s) => s,
            None => return Err(String::from("truncated read: no sequence")),
        };
        if lines.next().is_none() {
            return Err(String::from("truncated read: no plus line"));
        }
        let qual = match lines.next() {
            Some(q) => q,
            None => return Err(String::from("truncated read: no quality")),
        };
        reads += 1;
        bases += seq.len() as u64;
        for b in seq.bytes() {
            trits[(k3_trit(b) + 1) as usize] += 1;
            if b == b'G' || b == b'C' {
                gc += 1;
            }
        }
        for q in qual.bytes() {
            qsum += q.saturating_sub(33) as u64;
        }
    }
    if reads == 0 {
        return Err(String::from("no reads parsed"));
    }
    Ok(format!(
        "{{\"format\":\"FASTQ\",\"parser\":\"native (Rust no_std)\",\"total_reads_parsed\":{},\"total_bases\":{},\"mean_q_score\":{:.4},\"gc_content_ratio\":{:.4},\"trit_distribution\":{{\"minus\":{},\"zero\":{},\"plus\":{}}}}}",
        reads, bases, qsum as f64 / bases as f64, gc as f64 / bases as f64, trits[0], trits[1], trits[2]
    ))
}

/// VCF summary — same keys as VCFParser.parse_text.
pub fn parse_vcf(payload: &str) -> Result<String, String> {
    let (mut variants, mut snps, mut indels, mut headers) = (0u64, 0u64, 0u64, 0u64);
    for line in payload.lines() {
        if line.starts_with("##") {
            headers += 1;
            continue;
        }
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let f: alloc::vec::Vec<&str> = line.split('\t').collect();
        if f.len() < 5 {
            continue;
        }
        variants += 1;
        if f[3].len() == 1 && f[4].len() == 1 {
            snps += 1;
        } else {
            indels += 1;
        }
    }
    if variants == 0 {
        return Err(String::from("no variant records parsed"));
    }
    Ok(format!(
        "{{\"format\":\"VCF\",\"parser\":\"native (Rust no_std)\",\"header_lines_count\":{},\"total_variants_parsed\":{},\"statistics\":{{\"snps\":{},\"indels\":{}}}}}",
        headers, variants, snps, indels
    ))
}

/// Dispatch by format name. House contract: fallible, error as String.
pub fn parse_dispatch(fmt: &str, payload: &str) -> Result<String, String> {
    match fmt {
        "fastq" | "fq" => parse_fastq(payload),
        "vcf" => parse_vcf(payload),
        other => Err(format!("native parser not implemented for format: {}", other)),
    }
}
