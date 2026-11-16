//! The protocol-level harness: an access point that exists only as bytes.
//!
//! `--vectors` proves the crypto against published vectors and the oracle checks the frames, but neither
//! exercises the *state machine* -- the part that has never run. No machine here has a radio, and
//! `CONFIG_MAC80211_HWSIM` is not set in the kernel this pathway boots, so no virtual one can be loaded.
//! This is the alternative the source document names: it plays message 1 and message 3 at the transitions
//! directly, so parsing, nonce handling, the replay rule and the MIC check all run before an access point
//! is ever involved. It is the association's equivalent of `--selftest`.
//!
//! Every case that must be REFUSED is run and reported too, because a handshake that accepts what it
//! should reject is the failure this program exists to not have. The values it derives are printed so an
//! independent implementation can recompute them rather than take this program's word for it.

use crate::crypto::*;
use crate::eapol::{self, KI_ACK, KI_ENCRYPTED, KI_INSTALL, KI_KEY_TYPE_PAIRWISE, KI_MIC, KI_SECURE, KEY_DESC_VERSION};
use crate::supplicant::{on_message_1, on_message_3};

/// Fixed inputs, fixed on purpose: the oracle recomputes every value below from these same bytes, and a
/// random nonce would make that comparison impossible.
const SSID: &[u8] = b"TRM-HARNESS";
const PASSPHRASE: &[u8] = b"correct-horse-battery";
const WRONG_PASSPHRASE: &[u8] = b"correct-horse-batteryx";
const AP_MAC: [u8; 6] = [0xb4, 0x45, 0x06, 0xd4, 0x2c, 0x51];
const STA_MAC: [u8; 6] = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
const GTK: [u8; 16] = [
    0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
];

fn nonce(seed: u8) -> [u8; 32] {
    let mut n = [0u8; 32];
    for (i, b) in n.iter_mut().enumerate() {
        *b = seed.wrapping_add(i as u8);
    }
    n
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// The RSN information element an access point puts in message 1: version 1, CCMP group, one CCMP
/// pairwise, one PSK AKM. The body is what the station has to echo, so it has to be a real one.
fn rsn_element() -> Vec<u8> {
    let mut body = vec![0x01, 0x00];
    body.extend_from_slice(&[0x00, 0x0f, 0xac, 0x04]);
    body.extend_from_slice(&[0x01, 0x00]);
    body.extend_from_slice(&[0x00, 0x0f, 0xac, 0x04]);
    body.extend_from_slice(&[0x01, 0x00]);
    body.extend_from_slice(&[0x00, 0x0f, 0xac, 0x02]);
    body.extend_from_slice(&[0x00, 0x00]);
    let mut ie = vec![0x30, body.len() as u8];
    ie.extend_from_slice(&body);
    ie
}

/// Message 1 as the access point sends it: ACK set, no MIC. `build` always writes a digest, so the field
/// is zeroed afterwards -- a message 1 that carries a MIC is not a message 1.
fn build_msg1(replay: u64, anonce: &[u8; 32], key_data: &[u8]) -> Vec<u8> {
    let info = KEY_DESC_VERSION | KI_KEY_TYPE_PAIRWISE | KI_ACK;
    let mut f = eapol::build(replay, anonce, info, key_data, &[0u8; 16]);
    f[4 + 77..4 + 77 + 16].copy_from_slice(&[0u8; 16]);
    f
}

/// Message 3 as the access point sends it: ACK, MIC, SECURE, INSTALL, with the key data encrypted under
/// the KEK. The MIC is taken over the frame with the MIC field zeroed, which is what `build` does.
fn build_msg3(replay: u64, anonce: &[u8; 32], ptk: &[u8; 64], wrapped: &[u8]) -> Vec<u8> {
    let info = KEY_DESC_VERSION
        | KI_KEY_TYPE_PAIRWISE
        | KI_ACK
        | KI_MIC
        | KI_SECURE
        | KI_INSTALL
        | KI_ENCRYPTED;
    eapol::build(replay, anonce, info, wrapped, &ptk[..16])
}

/// The group key as a key data encapsulation and then wrapped: `dd len 00 0f ac 04 key_id reserved gtk`.
fn wrapped_gtk(ptk: &[u8; 64]) -> Vec<u8> {
    let mut kde = vec![0xdd, (6 + GTK.len()) as u8];
    kde.extend_from_slice(&[0x00, 0x0f, 0xac, 0x04, 0x01, 0x00]);
    kde.extend_from_slice(&GTK);
    aes_kw_wrap(&ptk[16..32], &kde).unwrap_or_default()
}

/// What a case must do, so a failure names the transition that broke rather than the file.
struct Report {
    passed: usize,
    failed: usize,
}

impl Report {
    fn must_hold(&mut self, name: &str, ok: bool) {
        if ok {
            self.passed += 1;
            println!("harness: ok       {}", name);
        } else {
            self.failed += 1;
            println!("harness: FAILED   {} -- this should have held, and did not", name);
        }
    }

    fn must_refuse(&mut self, name: &str, err: Option<String>) {
        match err {
            Some(why) => {
                self.passed += 1;
                println!("harness: refused  {} -- {}", name, why);
            }
            None => {
                self.failed += 1;
                println!("harness: FAILED   {} -- it was accepted, and it must not be", name);
            }
        }
    }
}

/// Play an access point at the transitions. Returns 0 only if every case held and every case that had to
/// be refused was refused.
pub fn run() -> i32 {
    let mut r = Report { passed: 0, failed: 0 };
    let pmk = pbkdf2_sha1_32(PASSPHRASE, SSID, 4096);
    let anonce = nonce(0x00);
    let snonce = nonce(0x20);
    // The access point's own derivation, which has to agree with the station's. The Python oracle
    // recomputes it a third way from the same bytes, so agreement here is not the only check on it.
    let ptk = derive_ptk(&pmk, &AP_MAC, &STA_MAC, &anonce, &snonce);

    println!(
        "harness: ssid={} ap={} sta={}",
        String::from_utf8_lossy(SSID),
        hex(&AP_MAC),
        hex(&STA_MAC)
    );
    println!("harness: pmk={}", hex(&pmk));
    println!("harness: ptk={}", hex(&ptk));

    // --- message 1 in, message 2 out -------------------------------------------------------------
    let m1 = build_msg1(1, &anonce, &rsn_element());
    let m1p = eapol::parse(&m1).expect("the harness built a message 1 that does not parse");
    r.must_hold("message 1 classifies as message 1", m1p.handshake_message() == 1);
    let (m2, sta_ptk) = on_message_1(&pmk, &AP_MAC, &STA_MAC, &snonce, &m1p)
        .expect("a message 1 with a real RSN element must be answerable");
    r.must_hold("the two sides derived the same PTK", sta_ptk == ptk);
    let m2p = eapol::parse(&m2).expect("message 2 does not parse");
    r.must_hold("message 2 classifies as message 2", m2p.handshake_message() == 2);
    r.must_hold("message 2 carries message 1's replay counter", m2p.replay == 1);
    r.must_hold("message 2 carries the station's nonce", m2p.nonce == snonce);
    r.must_hold(
        "message 2's MIC verifies under the PTK",
        eapol_mic(&ptk[..16], &m2p.mic_input()) == m2p.mic,
    );
    r.must_hold(
        "message 2 echoes the RSN element byte for byte",
        m2p.key_data == rsn_element(),
    );
    // The frames themselves go out, so an independent implementation recomputes their MICs from the bytes
    // rather than from this program's own claim about them.
    println!("harness: msg1={}", hex(&m1));
    println!("harness: msg2={}", hex(&m2));
    println!("harness: msg2_mic={}", hex(&m2p.mic));

    // --- message 3 in, message 4 and the group key out -------------------------------------------
    let wrapped = wrapped_gtk(&ptk);
    let m3 = build_msg3(2, &anonce, &ptk, &wrapped);
    let m3p = eapol::parse(&m3).expect("the harness built a message 3 that does not parse");
    r.must_hold("message 3 classifies as message 3", m3p.handshake_message() == 3);
    let (m4, gtk) = on_message_3(&sta_ptk, &snonce, 1, &m3p)
        .expect("a message 3 with a good MIC and a good wrap must be accepted");
    r.must_hold("the group key came back out of the wrap", gtk.as_deref() == Some(&GTK[..]));
    let m4p = eapol::parse(&m4).expect("message 4 does not parse");
    r.must_hold("message 4 classifies as message 4", m4p.handshake_message() == 4);
    r.must_hold("message 4 carries message 3's replay counter", m4p.replay == 2);
    r.must_hold(
        "message 4's MIC verifies under the PTK",
        eapol_mic(&ptk[..16], &m4p.mic_input()) == m4p.mic,
    );
    r.must_hold("message 4 carries no key data", m4p.key_data.is_empty());
    r.must_hold("message 4 sets SECURE and does not ACK", m4p.key_info & KI_SECURE != 0 && m4p.key_info & KI_ACK == 0);
    println!("harness: msg3={}", hex(&m3));
    println!("harness: msg4={}", hex(&m4));
    println!("harness: gtk={}", hex(&GTK));
    println!("harness: msg4_mic={}", hex(&m4p.mic));

    // --- every case that must be REFUSED ---------------------------------------------------------
    // A message 3 whose MIC does not verify: the PTK is not the one the access point used. On real
    // hardware this is what a wrong passphrase produces, and it must not install a key.
    let mut bad_mic = m3p.clone();
    bad_mic.mic[0] ^= 0xff;
    r.must_refuse(
        "message 3 with a corrupted MIC",
        on_message_3(&sta_ptk, &snonce, 1, &bad_mic).err(),
    );
    let wrong_ptk = derive_ptk(
        &pbkdf2_sha1_32(WRONG_PASSPHRASE, SSID, 4096),
        &AP_MAC,
        &STA_MAC,
        &anonce,
        &snonce,
    );
    r.must_refuse(
        "message 3 checked against a PTK from a different passphrase",
        on_message_3(&wrong_ptk, &snonce, 1, &m3p).err(),
    );

    // A replayed message 3, and one that goes backwards. The counter has to advance.
    let mut replayed = m3p.clone();
    replayed.replay = 1;
    r.must_refuse(
        "message 3 that does not advance the replay counter",
        on_message_3(&sta_ptk, &snonce, 1, &replayed).err(),
    );
    let mut backwards = m3p.clone();
    backwards.replay = 0;
    r.must_refuse(
        "message 3 with a counter behind message 1",
        on_message_3(&sta_ptk, &snonce, 1, &backwards).err(),
    );

    // A corrupted key wrap behind a valid MIC: the wrap's own integrity word has to catch it, rather
    // than a wrong group key being installed as though it were the right one.
    let mut bad_wrap = wrapped.clone();
    let n = bad_wrap.len();
    bad_wrap[n - 4] ^= 0x01;
    let m3_bad_wrap = eapol::parse(&build_msg3(2, &anonce, &ptk, &bad_wrap)).expect("parses");
    r.must_refuse(
        "message 3 whose key wrap fails its integrity check",
        on_message_3(&sta_ptk, &snonce, 1, &m3_bad_wrap).err(),
    );

    // A message 1 with nothing to echo back.
    let m1_no_rsn = eapol::parse(&build_msg1(1, &anonce, &[])).expect("parses");
    r.must_refuse(
        "message 1 with no RSN element to echo",
        on_message_1(&pmk, &AP_MAC, &STA_MAC, &snonce, &m1_no_rsn).err(),
    );

    println!(
        "harness: {} case(s) held or refused as required, {} failed",
        r.passed, r.failed
    );
    if r.failed == 0 {
        0
    } else {
        1
    }
}
