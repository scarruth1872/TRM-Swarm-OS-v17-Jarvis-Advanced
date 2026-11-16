//! trm_wifi -- the supplicant the bare-metal pathway needs, and nothing more than that.
//!
//! A driver, its firmware and a regulatory database produce a radio that no network is on: association
//! and the WPA2-PSK four-way handshake are userspace. This is that userspace. It is dependency-free for
//! the same reason the supervisor is -- a single rustc invocation on any host, no crate graph to resolve
//! -- which means the one primitive std does not offer (a netlink or AF_PACKET socket) is issued as a
//! syscall in src/syscall.rs.
//!
//! The crypto is checked against oracles rather than trusted: `--selftest` holds the published vectors
//! for SHA1, HMAC and AES, and `--vectors` / `--unwrap` exist so an independent implementation (see
//! trm_chainload/scripts/verify_wifi_crypto.py, which uses Python's hashlib and the openssl tool) can
//! diff this one's output byte for byte.

// The protocol's surface is written out in full -- the commands and attributes the association will
// send, and the syscall helpers it will need -- while the association itself is the next piece of work.
// Those warnings are therefore expected rather than ignored, and this allow comes off with it.
#![allow(dead_code)]

mod crypto;
mod eapol;
mod harness;
mod netlink;
mod nl80211;
mod supplicant;
mod syscall;

use crypto::*;

const SSID: &[u8] = b"TRM-Test";
const PASSPHRASE: &[u8] = b"trmtest123";
const AA: [u8; 6] = [0x02, 0, 0, 0, 0, 1];      // the access point
const SPA: [u8; 6] = [0x02, 0, 0, 0, 0, 2];     // this station

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok())
        .collect()
}

fn nonces() -> ([u8; 32], [u8; 32]) {
    let mut an = [0u8; 32];
    let mut sn = [0u8; 32];
    for i in 0..32 {
        an[i] = i as u8;
        sn[i] = 0x20 + i as u8;
    }
    (an, sn)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match mode {
        "--selftest" => std::process::exit(selftest()),
        "--harness" => std::process::exit(harness::run()),
        "--vectors" => {
            // The fixed inputs an outside implementation can reproduce: the passphrase and SSID here,
            // the MACs and nonces above. Everything derived is printed as hex, so a diff is the check.
            let pmk = pbkdf2_sha1_32(PASSPHRASE, SSID, 4096);
            let (an, sn) = nonces();
            let ptk = derive_ptk(&pmk, &AA, &SPA, &an, &sn);
            let frame = b"\x02\x03\x00\x5f\x02\x00\x8a\x00\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
            println!("passphrase {}", String::from_utf8_lossy(PASSPHRASE));
            println!("ssid {}", String::from_utf8_lossy(SSID));
            println!("pmk {}", hex(&pmk));
            println!("ptk {}", hex(&ptk));
            println!("kck {}", hex(&ptk[..16]));
            println!("kek {}", hex(&ptk[16..32]));
            println!("tk {}", hex(&ptk[32..48]));
            // The frame is printed too, so the outside implementation computes its MIC over these exact
            // bytes instead of over a second copy of them written by hand in the harness.
            println!("frame {}", hex(frame));
            println!("mic {}", hex(&eapol_mic(&ptk[..16], frame)));
        }
        "--unwrap" => {
            // AES key-wrap unwrap, so a key wrapped by openssl must come back out of here unchanged.
            let kek = match args.get(2).and_then(|s| unhex(s)) {
                Some(k) => k,
                None => {
                    eprintln!("usage: trm_wifi --unwrap <kek-hex> <wrapped-hex>");
                    std::process::exit(2);
                }
            };
            let wrapped = match args.get(3).and_then(|s| unhex(s)) {
                Some(w) => w,
                None => {
                    eprintln!("usage: trm_wifi --unwrap <kek-hex> <wrapped-hex>");
                    std::process::exit(2);
                }
            };
            match aes_kw_unwrap(&kek, &wrapped) {
                Some(out) => println!("unwrapped {}", hex(&out)),
                None => {
                    println!("unwrapped REFUSED");
                    std::process::exit(1);
                }
            }
        }
        "--eapol-vectors" => std::process::exit(eapol_vectors()),
        "--gtk-from-kde" => {
            // The oracle wraps a GTK and builds the key data encapsulation around it; this must find the
            // key inside and hand back exactly those bytes. The third argument says whether the key data
            // arrived AES-key-wrapped, which message 3's encrypted-key-data bit decides.
            let kek = match args.get(2).and_then(|s| unhex(s)) {
                Some(k) => k,
                None => {
                    eprintln!("usage: trm_wifi --gtk-from-kde <kek-hex> <kde-hex> [encrypted]");
                    std::process::exit(2);
                }
            };
            let kde = match args.get(3).and_then(|s| unhex(s)) {
                Some(k) => k,
                None => {
                    eprintln!("usage: trm_wifi --gtk-from-kde <kek-hex> <kde-hex> [encrypted]");
                    std::process::exit(2);
                }
            };
            let enc = args.get(4).map(|s| s == "encrypted").unwrap_or(false);
            match eapol::gtk_from_key_data(&kde, &kek, enc) {
                Some(g) => println!("kde-gtk {}", hex(&g)),
                None => {
                    println!("kde-gtk NONE");
                    std::process::exit(1);
                }
            }
        }
        "--associate" => {
            // The credentials live in a file beside the launcher's own network.conf, in the same shape,
            // rather than on this command line where anything on the machine could read them out of /proc.
            let iface = match args.get(2) {
                Some(s) => s.clone(),
                None => {
                    eprintln!("usage: trm_wifi --associate <interface> <wifi.conf>");
                    std::process::exit(2);
                }
            };
            let conf = args.get(3).cloned().unwrap_or_default();
            std::process::exit(associate(&iface, &conf));
        }
        "--probe" | "" => {
            // What the boot runs: ask the kernel what radio it has. Every line here is the kernel's answer
            // or an honest admission that there is none -- a machine with no wireless stack must not read
            // as a machine whose driver failed.
            std::process::exit(probe());
        }
        other => {
            println!("wifi: {} is not a mode -- --selftest, --vectors, --unwrap <kek> <wrapped>, --probe", other);
            std::process::exit(2);
        }
    }
}

/// Read the launcher's own config shape: `key=value` lines, `#` starts a comment. The same reader the
/// launcher uses for network.conf, because a second credential format is a second thing to get wrong.
fn read_conf(path: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Ok(text) = std::fs::read_to_string(path) {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                out.push((k.trim().to_lowercase(), v.trim().to_string()));
            }
        }
    }
    out
}

/// The station interface to associate on: the one named in the config, or the only one there is.
fn station_interface(family: u16, want: Option<&str>) -> Option<nl80211::Iface> {
    let ifs = nl80211::interfaces(family).ok()?;
    if let Some(name) = want {
        if let Some(i) = ifs.iter().find(|i| i.name == name) {
            return Some(nl80211::Iface {
                index: i.index,
                wiphy: i.wiphy,
                name: i.name.clone(),
                iftype: i.iftype,
                mac: i.mac.clone(),
            });
        }
    }
    ifs.into_iter().find(|i| i.iftype == nl80211::IFTYPE_STATION)
}

fn associate(iface_name: &str, conf_path: &str) -> i32 {
    let conf = read_conf(conf_path);
    let get = |k: &str| conf.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
    let (ssid, passphrase) = match (get("ssid"), get("psk")) {
        (Some(s), Some(p)) if !s.is_empty() && !p.is_empty() => (s, p),
        _ => {
            println!("wifi: {} names no ssid/psk pair, so there is nothing to associate with. Said plainly \
                      rather than attempted: a wrong passphrase and a missing radio look identical otherwise.",
                     conf_path);
            return 0;
        }
    };
    if passphrase.len() < 8 || passphrase.len() > 63 {
        println!("wifi: the passphrase is {} characters; WPA2-PSK takes 8 to 63, so this cannot be the key",
                 passphrase.len());
        return 2;
    }

    let family = match netlink::resolve_family("nl80211") {
        Ok(f) => f,
        Err(e) => {
            println!("wifi: no nl80211 family ({}) -- there is no radio here to associate", e);
            return 0;
        }
    };
    let iface = match station_interface(family, Some(iface_name)) {
        Some(i) => i,
        None => {
            println!("wifi: no interface named {} in station mode -- nothing to associate on", iface_name);
            return 0;
        }
    };
    if iface.mac.len() != 6 {
        println!("wifi: {} has no MAC in the kernel's own answer, so the PTK cannot be derived", iface.name);
        return 2;
    }
    println!("wifi: associating {} (mac {}) with ssid {:?} -- the PMK is derived here, and the kernel is \
              told to connect", iface.name, hex(&iface.mac), ssid);
    match supplicant::associate(family, &iface, ssid.as_bytes(), passphrase.as_bytes()) {
        Ok(o) => {
            println!("wifi: associated to {:?} -- the handshake was run {}",
                     hex(&o.bssid),
                     if o.ran_handshake_here { "by this program (4 EAPOL frames)" } else { "by the kernel" });
            match &o.gtk {
                Some(g) => println!("wifi: the group key was unwrapped ({} bytes) and both keys installed", g.len()),
                None => println!("wifi: the pairwise key is installed; no group key came with message 3"),
            }
            0
        }
        Err(e) => {
            println!("wifi: association did not complete: {}", e);
            1
        }
    }
}

/// The handshake frames over fixed inputs, so an implementation that is not this one can check the MICs
/// on frames this one built -- the same shape of proof as --vectors, one layer up.
fn eapol_vectors() -> i32 {
    let pmk = pbkdf2_sha1_32(PASSPHRASE, SSID, 4096);
    let (anonce, snonce) = nonces();
    let ptk = derive_ptk(&pmk, &AA, &SPA, &anonce, &snonce);
    // A fixed message 1 key data: the RSN element an AP would send, 22 bytes of it.
    let rsn: [u8; 22] = [
        0x01, 0x00, 0x00, 0x0f, 0xac, 0x04, 0x01, 0x00, 0x00, 0x0f, 0xac, 0x04, 0x01, 0x00, 0x00, 0x0f,
        0xac, 0x02, 0x00, 0x00, 0x00, 0x00,
    ];
    let mut msg1_key_data = vec![nl80211::EID_RSN, rsn.len() as u8];
    msg1_key_data.extend_from_slice(&rsn);
    let msg1 = eapol::parse(&{
        let mut f = eapol::build(1, &anonce, eapol::KEY_DESC_VERSION | eapol::KI_KEY_TYPE_PAIRWISE | eapol::KI_ACK,
                                 &msg1_key_data, &[0u8; 16]);
        // message 1 carries no MIC, so the frame as built is the frame as sent
        f.truncate(f.len());
        f
    });
    let echoed = msg1.as_ref().map(|m| eapol::rsn_ie_from(&m.key_data)).unwrap_or_default();

    let data = {
        let mut out = vec![nl80211::EID_RSN, echoed.len() as u8];
        out.extend_from_slice(&echoed);
        out
    };
    let msg2 = eapol::build(1, &snonce, eapol::KEY_DESC_VERSION | eapol::KI_KEY_TYPE_PAIRWISE | eapol::KI_MIC,
                            &data, &ptk[..16]);
    let msg4 = eapol::build(1, &snonce,
                            eapol::KEY_DESC_VERSION | eapol::KI_KEY_TYPE_PAIRWISE | eapol::KI_MIC | eapol::KI_SECURE,
                            &[], &ptk[..16]);

    println!("anonce {}", hex(&anonce));
    println!("snonce {}", hex(&snonce));
    println!("kck {}", hex(&ptk[..16]));
    println!("msg1 {}", hex(&msg1.as_ref().map(|m| m.raw.clone()).unwrap_or_default()));
    println!("msg2 {}", hex(&msg2));
    println!("msg4 {}", hex(&msg4));
    println!("rsn {}", hex(&echoed));
    0
}

/// Ask the kernel about its radios, and say what it answers.
fn probe() -> i32 {
    let fam = match netlink::resolve_family("nl80211") {
        Ok(f) => f,
        Err(e) => {
            println!(
                "wifi: the kernel has no nl80211 family ({}) -- cfg80211 is not loaded, so there is no \
                 radio here to associate, and that is not the driver's failure",
                e
            );
            return 0;
        }
    };
    println!("wifi: nl80211 is family {} (asked of the kernel, not assumed)", fam);
    let phys = nl80211::wiphys(fam).unwrap_or_default();
    for w in &phys {
        println!("wifi:   {} -- {} ext-feature bytes, four-way handshake in the kernel: {}",
            w.name, w.ext_features.len(),
            if w.does_4way_handshake() { "yes" } else { "no" });
    }
    let ifs = nl80211::interfaces(fam).unwrap_or_default();
    for i in &ifs {
        println!("wifi:   {} ifindex {} of {} type {} mac {}",
            i.name, i.index, i.wiphy, i.iftype, hex(&i.mac));
    }
    if phys.is_empty() {
        println!("wifi: no wiphy is registered -- the modules are loaded, but no radio has bound");
    } else if ifs.iter().all(|i| i.iftype != nl80211::IFTYPE_STATION) {
        println!("wifi: no interface in station mode -- association needs one, and none is here");
    }
    0
}

/// The published vectors, so the primitives are checked by someone else's numbers rather than the ones
/// they produced. SHA1("abc") and HMAC-SHA1 RFC 2202 case 1 are canonical; the AES block is FIPS-197
/// appendix B; the key-wrap pair is RFC 3394 section 4.6.
fn selftest() -> i32 {
    let mut fails = 0;
    let check = |name: &str, got: String, want: &str, fails: &mut i32| {
        if got == want {
            println!("  ok   {}", name);
        } else {
            println!("  FAIL {} got {} want {}", name, got, want);
            *fails += 1;
        }
    };
    check("sha1(abc)", hex(&sha1(b"abc")), "a9993e364706816aba3e25717850c26c9cd0d89d", &mut fails);
    check(
        "hmac_sha1(rfc2202 #1)",
        hex(&hmac_sha1(&[0x0bu8; 20], b"Hi There")),
        "b617318655057264e28bc0b6fb378c8ef146be00",
        &mut fails,
    );
    let k: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let p: [u8; 16] = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];
    check("aes128 fips197", hex(&aes128_encrypt_block(&k, &p)), "69c4e0d86a7b0430d8cdb78070b4c55a", &mut fails);
    check(
        "aes128 decrypt(r)",
        hex(&aes128_decrypt_block(&k, &aes128_encrypt_block(&k, &p))),
        "00112233445566778899aabbccddeeff",
        &mut fails,
    );
    // RFC 3394 section 4.6, both directions: the wrapped key data must come back, and a corrupted wrap
    // must be REFUSED rather than returned wrong -- that integrity word is the only thing that catches a
    // KEK derived from the wrong PTK, so it is checked here rather than assumed.
    let kek: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f];
    let wrapped = unhex("1fa68b0a8112b447aef34bd8fb5a7b829d3e862371d2cfe5").unwrap();
    match aes_kw_unwrap(&kek, &wrapped) {
        Some(k) => check(
            "aeskw rfc3394 4.6",
            hex(&k),
            "00112233445566778899aabbccddeeff",
            &mut fails,
        ),
        None => {
            println!("  FAIL aeskw rfc3394 4.6 -- refused a valid wrap");
            fails += 1;
        }
    }
    let mut bad = wrapped.clone();
    bad[8] ^= 1;
    if aes_kw_unwrap(&kek, &bad).is_some() {
        println!("  FAIL aeskw -- accepted a corrupted wrap");
        fails += 1;
    } else {
        println!("  ok   aeskw refuses a corrupted wrap");
    }
    let ret = 0;
    if fails == 0 {
        println!("wifi: self-test OK (6 checks)");
    } else {
        println!("wifi: self-test FAILED ({} checks)", fails);
    }
    ret + if fails == 0 { 0 } else { 1 }
}
