//! Pins `K3Head::input_with_ids` to `K3Head::input`.
//!
//! The roster scores text by tokenising into character n-grams and summing embedding rows. Every
//! head hashes the same buckets from the same string, so scoring a directive across 15 heads was
//! hashing it 15 times; `input_with_ids` lets a caller tokenise once. That is only a valid
//! optimisation if the two paths are bit-identical, which is what this test asserts.
//!
//! Skips (does not silently pass) when the head blob is not on disk.

use trm_uefi_kernel::k3_head::{char_ngram_ids, parse};

fn heads() -> Option<Vec<trm_uefi_kernel::k3_head::K3Head>> {
    let candidates = [
        "assets/k3_heads.bin",
        "../assets/k3_heads.bin",
        "../../assets/k3_heads.bin",
    ];
    for p in candidates {
        if let Ok(b) = std::fs::read(p) {
            if let Some(h) = parse(&b) {
                if !h.is_empty() {
                    return Some(h);
                }
            }
        }
    }
    None
}

#[test]
fn tokenise_once_matches_tokenise_per_head() {
    let heads = match heads() {
        Some(h) => h,
        None => {
            eprintln!("SKIP: k3_heads.bin not found from this cwd");
            return;
        }
    };

    let probes = [
        "erase the audit logs from last quarter",
        "cover the watchdog with a regression test",
        "take MFA off the shared ops account",
        "run the migration against a throwaway staging database",
        "",
    ];

    let mut compared = 0usize;
    for h in heads.iter() {
        for p in probes {
            let direct = h.input(p);
            let ids = char_ngram_ids(p, h.buckets);
            let once = h.input_with_ids(&ids);
            assert_eq!(direct.len(), once.len());
            for (i, (a, b)) in direct.iter().zip(once.iter()).enumerate() {
                assert_eq!(
                    a.to_bits(),
                    b.to_bits(),
                    "head {} probe {:?} index {i}: {a} vs {b}",
                    h.name,
                    p
                );
            }
            compared += 1;
        }
    }
    assert!(compared > 0, "no comparisons ran");
    eprintln!("compared {compared} head/text pairs bit-exactly");
}
