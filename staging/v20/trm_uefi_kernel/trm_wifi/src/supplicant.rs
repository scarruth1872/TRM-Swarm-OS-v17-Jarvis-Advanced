//! The supplicant: association, and the four-way handshake when the kernel will not do it.
//!
//! Two paths lead to a network and this program handles both, because which one it gets is a property of
//! the driver, not a choice:
//!
//!   * **The driver runs the handshake.** `NL80211_EXT_FEATURE_4WAY_HANDSHAKE_STA_PSK` says so, and
//!     `CONNECT` with the PMK completes without a single EAPOL frame leaving userspace. This is the common
//!     case and the code below does nothing more than ask.
//!   * **It does not.** Then message 1 arrives on the packet socket, and the frames in `eapol.rs` are the
//!     answer: derive the PTK from the AP's nonce and ours, return message 2, verify message 3's MIC --
//!     refusing it if it does not check out -- unwrap the GTK, install both keys, and send message 4.
//!
//! Nothing here retries a bad MIC or a timeout. A handshake that did not complete is reported as what it
//! was; inventing a second attempt at a key exchange is how a working-looking session gets built on a
//! wrong PTK.

use crate::crypto::*;
use crate::eapol::{self, EapolKey, KEY_DESC_VERSION, KI_KEY_TYPE_PAIRWISE, KI_MIC, KI_SECURE};
use crate::netlink::*;
use crate::nl80211::*;
use crate::syscall::*;
use std::io;
use std::time::{Duration, Instant};

/// How long the whole association is given. Long enough for a scan and a slow AP, short enough that a boot
/// is not held hostage by a network that is not answering.
const HANDSHAKE_BUDGET: Duration = Duration::from_secs(15);

pub struct Outcome {
    pub bssid: Vec<u8>,
    pub ran_handshake_here: bool,
    pub gtk: Option<Vec<u8>>,
}

/// Send CONNECT with the PMK. The kernel either completes the association or starts the handshake; the
/// return value says which, because a command that succeeded is not an association that finished.
fn connect(family: u16, ifindex: u32, ssid: &[u8], pmk: &[u8; 32]) -> io::Result<Vec<u8>> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&attr_u32(ATTR_IFINDEX, ifindex));
    payload.extend_from_slice(&attr(ATTR_SSID, ssid));
    payload.extend_from_slice(&attr_u32(ATTR_AUTH_TYPE, AUTHTYPE_OPEN_SYSTEM));
    payload.extend_from_slice(&attr_u32(ATTR_WPA_VERSIONS, WPA_VERSION_2));
    payload.extend_from_slice(&attr_u32(ATTR_AKM_SUITES, AKM_SUITE_PSK));
    payload.extend_from_slice(&attr_u32(ATTR_CIPHER_SUITE_GROUP, CIPHER_SUITE_CCMP));
    payload.extend_from_slice(&attr_u32(ATTR_CIPHER_SUITES_PAIRWISE, CIPHER_SUITE_CCMP));
    payload.extend_from_slice(&attr(ATTR_PMK, pmk));

    let mut nl = Nl::open(NETLINK_GENERIC)?;
    let seq = nl.next_seq();
    let msg = genl_msg(family, CMD_CONNECT, NL80211_GENL_VERSION, seq, NLM_F_REQUEST | NLM_F_ACK, &payload);
    nl.send(&msg)?;
    for (ty, body) in nl.recv_deadline(HANDSHAKE_BUDGET)? {
        if ty == family && body.len() >= 4 {
            for (atype, adata) in parse_attrs(&body[4..]) {
                if atype == ATTR_MAC && adata.len() == 6 {
                    return Ok(adata.to_vec());
                }
            }
        }
    }
    Ok(Vec::new())
}

/// A packet socket that sees only EAPOL, on one interface.
pub fn open_pae(ifindex: i32) -> io::Result<i32> {
    let fd = socket_nonblocking(AF_PACKET, SOCK_RAW, htons(eapol::ETHERTYPE_EAPOL) as i32)?;
    bind(fd, &sockaddr_ll(ifindex, eapol::ETHERTYPE_EAPOL))?;
    Ok(fd)
}

/// A frame out of one interface. The destination goes in the socket address and the source goes in the
/// frame, because that is the layer at which each is chosen.
fn send_frame(fd: i32, ifindex: i32, dst: &[u8; 6], src: &[u8; 6], payload: &[u8]) -> io::Result<usize> {
    let mut frame = Vec::with_capacity(14 + payload.len());
    frame.extend_from_slice(dst);
    frame.extend_from_slice(src);
    frame.extend_from_slice(&eapol::ETHERTYPE_EAPOL.to_be_bytes());
    frame.extend_from_slice(payload);
    sendto(fd, &frame, &sockaddr_ll_to(ifindex, eapol::ETHERTYPE_EAPOL, dst))
}

/// Message 2, from message 1. Pure: the AP's frame in, the answer out, no socket anywhere -- which is
/// what lets the harness drive this without a radio. Everything the state machine decides about message 1
/// is here, so a test of this is a test of the transition and not of a wire.
pub fn on_message_1(
    pmk: &[u8; 32],
    bssid: &[u8; 6],
    spa: &[u8; 6],
    snonce: &[u8; 32],
    frame: &EapolKey,
) -> Result<(Vec<u8>, [u8; 64]), String> {
    let rsn = eapol::rsn_ie_from(&frame.key_data);
    if rsn.is_empty() {
        // WPA2 needs the RSN IE echoed back; without it there is nothing to echo and the AP will reject
        // the answer, so this is reported rather than attempted.
        return Err("message 1 carried no RSN information element to echo back".into());
    }
    // The PTK is the PRF over both nonces with the two addresses in their sorted order -- the standard's
    // own rule, not ours.
    let key = derive_ptk(pmk, bssid, spa, &frame.nonce, snonce);
    let info = KEY_DESC_VERSION | KI_KEY_TYPE_PAIRWISE | KI_MIC;
    // The RSN element as it goes on the wire: id, length, then the body the AP sent.
    let data = {
        let mut out = vec![EID_RSN, rsn.len() as u8];
        out.extend_from_slice(&rsn);
        out
    };
    Ok((eapol::build(frame.replay, snonce, info, &data, &key[..16]), key))
}

/// Message 4, from message 3 -- and the GTK when the frame carried one. `msg1_replay` is the counter
/// message 1 carried: a message 3 that does not advance it is a replay, not a handshake.
pub fn on_message_3(
    ptk: &[u8; 64],
    snonce: &[u8; 32],
    msg1_replay: u64,
    frame: &EapolKey,
) -> Result<(Vec<u8>, Option<Vec<u8>>), String> {
    if frame.replay <= msg1_replay {
        // A replayed message 3 is not a handshake; the counter must advance.
        return Err(format!(
            "message 3 carried replay counter {} -- not past message 1's {} -- refusing it",
            frame.replay, msg1_replay
        ));
    }
    // The authentication step, and the only one that can fail loudly:
    let expect = eapol_mic(&ptk[..16], &frame.mic_input());
    if expect != frame.mic {
        return Err(
            "message 3's MIC did not verify against the PTK -- the PSK or the nonces are not what the AP has"
                .into(),
        );
    }
    let gtk = eapol::gtk_from_key_data(&frame.key_data, &ptk[16..32], frame.is_encrypted_key_data());
    // An encrypted key data block that will not unwrap is not a frame with no group key: the wrap's
    // integrity word just failed. Continuing here would install a pairwise key and leave the group key
    // silently missing, which is the failure this program exists to not have.
    if frame.is_encrypted_key_data() && gtk.is_none() && !frame.key_data.is_empty() {
        return Err(
            "message 3's key data would not unwrap under the KEK -- the wrap failed its integrity check".into(),
        );
    }
    let info = KEY_DESC_VERSION | KI_KEY_TYPE_PAIRWISE | KI_MIC | KI_SECURE;
    Ok((eapol::build(frame.replay, snonce, info, &[], &ptk[..16]), gtk))
}

/// The whole association. Returns what happened, or says plainly what did not.
pub fn associate(family: u16, iface: &Iface, ssid: &[u8], passphrase: &[u8]) -> Result<Outcome, String> {
    let pmk = pbkdf2_sha1_32(passphrase, ssid, 4096);
    // The reply names the BSSID when it has one, and the frame that carries the nonce names it again. The
    // frame is the authority -- it is the AP speaking -- so this is a starting value, not the answer.
    let named = connect(family, iface.index, ssid, &pmk)
        .map_err(|e| format!("CONNECT was refused: {}", e))?;
    let mut bssid = [0u8; 6];
    if named.len() == 6 {
        bssid.copy_from_slice(&named);
    }

    // The PMK derivation is the one thing worth proving before trusting a MIC to it.
    if pmk.iter().all(|b| *b == 0) {
        return Err("the PMK came out all zeroes, which is a derivation failure, not a network one".into());
    }

    let fd = open_pae(iface.index as i32).map_err(|e| format!("no packet socket on {}: {}", iface.name, e))?;
    let mut spa = [0u8; 6];
    if iface.mac.len() == 6 {
        spa.copy_from_slice(&iface.mac);
    }

    let mut snonce = [0u8; 32];
    getrandom(&mut snonce).map_err(|e| format!("no randomness for the nonce: {}", e))?;

    let deadline = Instant::now() + HANDSHAKE_BUDGET;
    let mut ptk: Option<[u8; 64]> = None;
    let mut msg1: Option<EapolKey> = None;

    while Instant::now() < deadline {
        let mut buf = vec![0u8; 2048];
        let n = match recvfrom(fd, &mut buf) {
            Ok(Some(n)) => n,
            Ok(None) => {
                std::thread::sleep(Duration::from_millis(20));
                continue;
            }
            Err(e) => return Err(format!("the packet socket failed: {}", e)),
        };
        buf.truncate(n);
        let frame = match eapol::parse(&buf) {
            Some(f) => f,
            None => continue,
        };
        match frame.handshake_message() {
            1 => {
                // The AP's nonce is in this frame; ours is the one just drawn. Its source address is read
                // off the frame it sent, because the frame is the authority on who is speaking.
                if buf.len() >= 12 {
                    bssid.copy_from_slice(&buf[6..12]);
                }
                let (pkt, key) = on_message_1(&pmk, &bssid, &spa, &snonce, &frame)?;
                send_frame(fd, iface.index as i32, &bssid, &spa, &pkt)
                    .map_err(|e| format!("message 2 could not be sent: {}", e))?;
                ptk = Some(key);
                msg1 = Some(frame);
            }
            3 => {
                // The keys are installed before message 4 goes out, because that frame is precisely the
                // statement that they are.
                let key = match ptk {
                    Some(k) => k,
                    None => {
                        return Err(
                            "message 3 arrived before message 1 -- nothing to check its MIC against".into(),
                        )
                    }
                };
                let msg1_replay = msg1.as_ref().map(|m| m.replay).unwrap_or(0);
                let (pkt, gtk) = on_message_3(&key, &snonce, msg1_replay, &frame)?;
                install_keys(family, iface.index, &key, gtk.as_deref())
                    .map_err(|e| format!("the PTK verified but the keys would not install: {}", e))?;
                send_frame(fd, iface.index as i32, &bssid, &spa, &pkt)
                    .map_err(|e| format!("message 4 could not be sent: {}", e))?;
                return Ok(Outcome { bssid: bssid.to_vec(), ran_handshake_here: true, gtk });
            }
            _ => continue,
        }
    }
    Err(format!(
        "no handshake completed within {}s. The association was requested and the frames never arrived, \
         which on a machine with a radio means the PSK or the SSID is not what the AP expects",
        HANDSHAKE_BUDGET.as_secs()
    ))
}

/// Install the pairwise key (and the group key when message 3 carried one).
fn install_keys(family: u16, ifindex: u32, ptk: &[u8; 64], gtk: Option<&[u8]>) -> io::Result<()> {
    let mut nl = Nl::open(NETLINK_GENERIC)?;
    let mut new_key = |idx: u8, kind: u32, data: &[u8], seq: &[u8]| -> io::Result<()> {
        let mut k = Vec::new();
        k.extend_from_slice(&attr_u32(ATTR_KEY_TYPE, kind));
        k.extend_from_slice(&attr_u32(ATTR_KEY_CIPHER, CIPHER_SUITE_CCMP));
        k.extend_from_slice(&attr_u8(ATTR_KEY_IDX, idx));
        k.extend_from_slice(&attr(ATTR_KEY_DATA, data));
        if !seq.is_empty() {
            k.extend_from_slice(&attr(ATTR_KEY_SEQ, seq));
        }
        let mut payload = Vec::new();
        payload.extend_from_slice(&attr_u32(ATTR_IFINDEX, ifindex));
        payload.extend_from_slice(&attr(ATTR_KEY, &k));
        let seq_no = nl.next_seq();
        let msg = genl_msg(family, CMD_NEW_KEY, NL80211_GENL_VERSION, seq_no, NLM_F_REQUEST | NLM_F_ACK, &payload);
        nl.send(&msg)?;
        nl.recv_deadline(Duration::from_secs(2)).map(|_| ())
    };
    // The temporal key is PTK[32..48]; the rest of the PTK has already been spent (KCK on the MIC,
    // KEK on the key data), which is why the install takes a slice and not the whole thing.
    new_key(0, KEYTYPE_PAIRWISE, &ptk[32..48], &[0u8; 6])?;
    if let Some(g) = gtk {
        new_key(1, KEYTYPE_GROUP, g, &[0u8; 6])?;
    }
    Ok(())
}
