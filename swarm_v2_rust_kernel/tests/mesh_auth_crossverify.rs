//! Cross-verification: the no_std kernel HMAC and the host sha2/hmac HMAC must
//! produce byte-identical tags for identical input. If they diverge, a signed
//! beacon would verify on one side and be rejected on the other.

#[path = "../../trm_uefi_kernel/src/hmac_sha256.rs"]
mod kernel_hmac;

use swarm_rust_kernel::mesh_discovery::{sign_beacon_bytes, verify_beacon_bytes, BEACON_BODY, BEACON_SIGNED};

const KEY: &[u8] = &[0x11u8; 32];
const NODE: u64 = 0x5452_4D00_0000_0001;

#[test]
fn kernel_and_host_hmac_agree_on_raw_input() {
    for msg in [&b"abcdef"[..], &b""[..], &[0u8; 200][..]] {
        let k = kernel_hmac::hmac_sha256(KEY, msg);
        let h = {
            use hmac::{Hmac, Mac};
            let mut m = Hmac::<sha2::Sha256>::new_from_slice(KEY).unwrap();
            m.update(msg);
            let out = m.finalize().into_bytes();
            let mut a = [0u8; 32];
            a.copy_from_slice(&out);
            a
        };
        assert_eq!(k, h, "HMAC disagreement on {}-byte message", msg.len());
    }
}

#[test]
fn kernel_verifies_host_signed_beacon() {
    // Host signs
    let signed = sign_beacon_bytes(NODE, KEY);
    // Kernel verifies the same bytes
    let r = kernel_hmac::verify_beacon(&signed, KEY);
    assert!(r.is_ok(), "kernel rejected a host-signed beacon: {:?}", r.err());
    let (node, agents, trits) = r.unwrap();
    assert_eq!(node, NODE);
    assert_eq!(agents, 14);
    assert_eq!(trits, 153523393);
}

#[test]
fn kernel_produced_bytes_match_host_bytes() {
    // Kernel builds + signs; host verifies the raw bytes.
    let body = kernel_hmac::build_beacon_body(NODE, 14, 153523393, 1_790_147_837_001);
    let ksigned = kernel_hmac::sign_beacon(&body, KEY);
    let host_ok = verify_beacon_bytes(&ksigned, KEY);
    assert!(host_ok.is_ok(), "host rejected kernel-signed beacon: {:?}", host_ok.err());
    println!("CROSSVERIFY_BEACON_HEX={}", ksigned.iter().map(|b| format!("{:02x}", b)).collect::<String>());
    assert_eq!(ksigned.len(), BEACON_SIGNED);
}

#[test]
fn both_reject_forgery() {
    let mut signed = sign_beacon_bytes(NODE, KEY);
    signed[9] ^= 0x01; // tamper node_id inside the signed body
    assert!(verify_beacon_bytes(&signed, KEY).is_err(), "host accepted forgery");
    assert!(kernel_hmac::verify_beacon(&signed, KEY).is_err(), "kernel accepted forgery");
}

#[test]
fn both_reject_unsigned_legacy_beacon() {
    let body = kernel_hmac::build_beacon_body(NODE, 14, 153523393, 1);
    let legacy = &body[..]; // 36 bytes, no tag
    assert_eq!(legacy.len(), BEACON_BODY);
    assert!(verify_beacon_bytes(legacy, KEY).is_err(), "host accepted unsigned beacon");
    assert!(kernel_hmac::verify_beacon(legacy, KEY).is_err(), "kernel accepted unsigned beacon");
}
