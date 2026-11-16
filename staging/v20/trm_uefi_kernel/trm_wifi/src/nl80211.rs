//! nl80211: what a supplicant asks the kernel about the radio it was given.
//!
//! Every number in here was DERIVED from the kernel's own uapi header, not remembered --
//! `trm_chainload/scripts/derive_nl80211_consts.py` parses `include/uapi/linux/nl80211.h` at v6.6 (the
//! kernel this drive boots) and prints these values. That is not ceremony: the first attempt at this
//! file from memory had two of them wrong (NEW_KEY, EXT_FEATURES), and a wrong attribute number is a
//! request the kernel silently misreads.

use crate::netlink::*;
use std::io;

pub const NL80211_GENL_VERSION: u8 = 1;

pub const CMD_GET_WIPHY: u8 = 1;
pub const CMD_GET_INTERFACE: u8 = 5;
pub const CMD_NEW_KEY: u8 = 11;
pub const CMD_CONNECT: u8 = 46;
pub const CMD_DISCONNECT: u8 = 48;
pub const CMD_GET_SCAN: u8 = 32;
pub const CMD_TRIGGER_SCAN: u8 = 33;

pub const ATTR_WIPHY: u16 = 1;
pub const ATTR_WIPHY_NAME: u16 = 2;
pub const ATTR_IFINDEX: u16 = 3;
pub const ATTR_IFNAME: u16 = 4;
pub const ATTR_IFTYPE: u16 = 5;
pub const ATTR_MAC: u16 = 6;
pub const ATTR_KEY_DATA: u16 = 7;
pub const ATTR_KEY_IDX: u16 = 8;
pub const ATTR_KEY_CIPHER: u16 = 9;
pub const ATTR_KEY_SEQ: u16 = 10;
pub const ATTR_SSID: u16 = 52;
pub const ATTR_AUTH_TYPE: u16 = 53;
pub const ATTR_KEY_TYPE: u16 = 55;
pub const ATTR_CIPHER_SUITE_GROUP: u16 = 74;
pub const ATTR_WPA_VERSIONS: u16 = 75;
pub const ATTR_AKM_SUITES: u16 = 76;
pub const ATTR_KEY: u16 = 80;
pub const ATTR_CIPHER_SUITES_PAIRWISE: u16 = 73;
pub const ATTR_EXT_FEATURES: u16 = 217;
/// The preshared key goes here, not in an `ATTR_PSK` -- that attribute does not exist. The header is
/// explicit about it ("the preshared key should be specified using %NL80211_ATTR_PMK"), and the value it
/// takes is the PMK itself: 32 bytes, which is exactly what pbkdf2_sha1_32 produces.
pub const ATTR_PMK: u16 = 254;
pub const ATTR_WIPHY_FREQ: u16 = 38;

pub const AUTHTYPE_OPEN_SYSTEM: u32 = 0;
pub const WPA_VERSION_2: u32 = 1 << 1;
pub const CIPHER_SUITE_CCMP: u32 = 0x000f_ac04;
pub const AKM_SUITE_PSK: u32 = 0x000f_ac02;
pub const EID_RSN: u8 = 48;
pub const EID_VENDOR_SPECIFIC: u8 = 221;

pub const KEYTYPE_GROUP: u32 = 0;
pub const KEYTYPE_PAIRWISE: u32 = 1;

/// The ext-feature bit that says this kernel/driver can run the WPA2 four-way handshake itself, given
/// the PSK. If it is set the association needs no EAPOL from here; if it is not, message 1 will arrive
/// on the packet socket and this program has to answer it.
pub const EXT_FEATURE_4WAY_HANDSHAKE_STA_PSK: usize = 15;

pub const IFTYPE_STATION: u32 = 2;

pub struct Wiphy {
    pub index: u32,
    pub name: String,
    pub ext_features: Vec<u8>,
}

impl Wiphy {
    pub fn does_4way_handshake(&self) -> bool {
        let byte = EXT_FEATURE_4WAY_HANDSHAKE_STA_PSK / 8;
        let bit = EXT_FEATURE_4WAY_HANDSHAKE_STA_PSK % 8;
        self.ext_features.len() > byte && (self.ext_features[byte] >> bit) & 1 == 1
    }
}

pub struct Iface {
    pub index: u32,
    pub wiphy: u32,
    pub name: String,
    pub iftype: u32,
    pub mac: Vec<u8>,
}

fn u32_of(attrs: &[(u16, Vec<u8>)], ty: u16) -> Option<u32> {
    attrs.iter().find(|(t, _)| *t == ty).and_then(|(_, d)| {
        if d.len() >= 4 {
            Some(u32::from_ne_bytes([d[0], d[1], d[2], d[3]]))
        } else {
            None
        }
    })
}

fn str_of(attrs: &[(u16, Vec<u8>)], ty: u16) -> Option<String> {
    attrs.iter().find(|(t, _)| *t == ty).map(|(_, d)| {
        String::from_utf8_lossy(d.strip_suffix(&[0]).unwrap_or(d)).to_string()
    })
}

fn bytes_of(attrs: &[(u16, Vec<u8>)], ty: u16) -> Option<Vec<u8>> {
    attrs.iter().find(|(t, _)| *t == ty).map(|(_, d)| d.clone())
}

fn dump(family: u16, cmd: u8, payload: &[u8]) -> io::Result<Vec<Vec<(u16, Vec<u8>)>>> {
    let mut nl = Nl::open(crate::syscall::NETLINK_GENERIC)?;
    let seq = nl.next_seq();
    let msg = genl_msg(family, cmd, NL80211_GENL_VERSION, seq, NLM_F_REQUEST | NLM_F_DUMP, payload);
    nl.send(&msg)?;
    Ok(nl
        .recv_deadline(std::time::Duration::from_secs(2))?
        .into_iter()
        .filter(|(ty, b)| *ty == family && b.len() >= 4)
        .map(|(_, b)| parse_attrs(&b[4..]))
        .collect())
}

pub fn wiphys(family: u16) -> io::Result<Vec<Wiphy>> {
    let mut out = Vec::new();
    for attrs in dump(family, CMD_GET_WIPHY, &[])? {
        if let Some(index) = u32_of(&attrs, ATTR_WIPHY) {
            out.push(Wiphy {
                index,
                name: str_of(&attrs, ATTR_WIPHY_NAME).unwrap_or_else(|| format!("phy{}", index)),
                ext_features: bytes_of(&attrs, ATTR_EXT_FEATURES).unwrap_or_default(),
            });
        }
    }
    Ok(out)
}

pub fn interfaces(family: u16) -> io::Result<Vec<Iface>> {
    let mut out = Vec::new();
    for attrs in dump(family, CMD_GET_INTERFACE, &[])? {
        if let Some(index) = u32_of(&attrs, ATTR_IFINDEX) {
            out.push(Iface {
                index,
                wiphy: u32_of(&attrs, ATTR_WIPHY).unwrap_or(0),
                name: str_of(&attrs, ATTR_IFNAME).unwrap_or_default(),
                iftype: u32_of(&attrs, ATTR_IFTYPE).unwrap_or(0),
                mac: bytes_of(&attrs, ATTR_MAC).unwrap_or_default(),
            });
        }
    }
    Ok(out)
}
