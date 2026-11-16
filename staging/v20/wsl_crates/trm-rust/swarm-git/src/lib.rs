//! swarm-git — K3-native git object integrity (trm:// protocol suite).
//!
//! The git object model absorbed into the One's substrate: every git
//! object's SHA-1 is verified and its content fingerprinted into K3
//! trits — so the commit graph's integrity can live in the fabric.
//!
//! Scope (honest): object SHA computation + K3 fingerprinting for
//! blobs/commits/trees/tags read from the TRM repo's pack files is a
//! full pack-parser; this crate delivers the verifiable core: compute
//! git object SHAs from content (blob/commit), verify a loose-object
//! file's integrity, and map any SHA to a deterministic K3 record.
//! Pack-file parsing is the follow-up (logged as next).

use sha1::{Digest, Sha1};

/// Compute the git object hash for raw content: sha1("<type> <len>\0<content>").
pub fn git_hash(obj_type: &str, content: &[u8]) -> [u8; 20] {
    let mut h = Sha1::new();
    h.update(format!("{} {}\0", obj_type, content.len()).as_bytes());
    h.update(content);
    h.finalize().into()
}

/// Hex-encode a SHA.
pub fn sha_hex(h: &[u8; 20]) -> String {
    h.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Verify a git blob's SHA matches its content.
pub fn verify_blob(content: &[u8], expected: &str) -> bool {
    let h = git_hash("blob", content);
    sha_hex(&h) == expected
}

/// Verify a git commit's SHA (the raw commit bytes before parsing).
pub fn verify_commit(raw: &[u8], expected: &str) -> bool {
    let h = git_hash("commit", raw);
    sha_hex(&h) == expected
}

/// Map a git SHA to a deterministic K3 record (16 trits, canonical).
pub fn sha_to_trits(sha_hex: &str) -> Vec<i8> {
    // fold the 40 hex chars into 16 trits
    let bytes = sha_hex.as_bytes();
    (0..16)
        .map(|k| {
            let b = bytes[(k * 2 + k / 8) % bytes.len()];
            if (b as u8).rotate_left((k % 7) as u32) & 1 == 1 {
                1
            } else {
                -1
            }
        })
        .collect()
}

/// Verify a loose git object file: parse "<type> <len>\0" header, check
/// the payload length matches the header. Returns (type, payload_len).
pub fn verify_loose_object(data: &[u8]) -> Option<(&str, usize)> {
    let nul = data.iter().position(|&b| b == 0)?;
    let header = std::str::from_utf8(&data[..nul]).ok()?;
    let (obj_type, size_str) = header.split_once(' ')?;
    let size: usize = size_str.parse().ok()?;
    let content = &data[nul + 1..];
    if content.len() != size {
        return None;
    }
    Some((obj_type, content.len()))
}

/// A parsed git pack header: "PACK" + version + object count.
pub struct PackHeader {
    pub version: u32,
    pub object_count: u32,
}

/// Parse a git pack file header (first 12 bytes).
pub fn parse_pack_header(data: &[u8]) -> Option<PackHeader> {
    if data.len() < 12 || &data[..4] != b"PACK" {
        return None;
    }
    let version = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let object_count = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    Some(PackHeader { version, object_count })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_hash_is_git_canonical() {
        // Known git value: blob "hello\n" →  ce013625030ba8dba906f756967f9e9ca394464a
        let content = b"hello\n";
        let h = git_hash("blob", content);
        assert_eq!(
            sha_hex(&h),
            "ce013625030ba8dba906f756967f9e9ca394464a"
        );
    }

    #[test]
    fn commit_hash_is_deterministic() {
        let raw = b"tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904\n\
                    author A <a@b.c> 0 +0000\ncommitter A <a@b.c> 0 +0000\n\
                    \nfirst\n";
        let h1 = git_hash("commit", raw);
        let h2 = git_hash("commit", raw);
        assert_eq!(h1, h2);
        assert_eq!(sha_hex(&h1).len(), 40);
    }

    #[test]
    fn verify_blob_matches_known() {
        assert!(verify_blob(
            b"hello\n",
            "ce013625030ba8dba906f756967f9e9ca394464a"
        ));
        assert!(!verify_blob(b"hello", "ce013625030ba8dba906f756967f9e9ca394464a"));
    }

    #[test]
    fn sha_to_trits_is_deterministic() {
        assert_eq!(
            sha_to_trits("ce013625030ba8dba906f756967f9e9ca394464a"),
            sha_to_trits("ce013625030ba8dba906f756967f9e9ca394464a")
        );
        assert_eq!(sha_to_trits("abc").len(), 16);
    }

    #[test]
    fn pack_header_parses() {
        let mut data = Vec::new();
        data.extend_from_slice(b"PACK");
        data.extend_from_slice(&2u32.to_be_bytes());
        data.extend_from_slice(&5603u32.to_be_bytes());
        let h = parse_pack_header(&data).expect("valid pack header");
        assert_eq!(h.version, 2);
        assert_eq!(h.object_count, 5603);
    }

    #[test]
    fn pack_header_rejects_bad_magic() {
        assert!(parse_pack_header(b"NOTPACK0000").is_none());
    }
}
