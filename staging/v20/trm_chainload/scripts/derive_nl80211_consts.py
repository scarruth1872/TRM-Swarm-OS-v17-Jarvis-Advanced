#!/usr/bin/env python3
"""Derive the nl80211 constants trm_wifi uses from the kernel's own uapi header -- and check the Rust.

These are wire numbers: a wrong attribute value is a request the kernel misreads rather than refuses, so
it fails quietly and looks like a driver problem. They are stable ABI, which is exactly what makes them
easy to write from memory and hard to notice being wrong.

    curl -o nl80211.h      'https://git.kernel.org/.../include/uapi/linux/nl80211.h?h=v6.6'
    curl -o ieee80211.h    'https://git.kernel.org/.../include/linux/ieee80211.h?h=v6.6'

    python3 derive_nl80211_consts.py nl80211.h ieee80211.h                  # print the values
    python3 derive_nl80211_consts.py nl80211.h ieee80211.h --check <nl80211.rs>   # diff the Rust against them

The check mode exists because writing them by hand already produced one wrong value
(`NL80211_ATTR_CIPHER_SUITES_PAIRWISE` as 81 instead of 73) that no test in the crate could catch --
nothing in this program can tell a wrong attribute number from a driver that did not answer.
"""
import re
import sys

# (rust constant name, kind, header name) -- the kinds decide how the value is computed
WANT = [
    ("CMD_GET_WIPHY", "cmd", "NL80211_CMD_GET_WIPHY"),
    ("CMD_GET_INTERFACE", "cmd", "NL80211_CMD_GET_INTERFACE"),
    ("CMD_NEW_KEY", "cmd", "NL80211_CMD_NEW_KEY"),
    ("CMD_CONNECT", "cmd", "NL80211_CMD_CONNECT"),
    ("CMD_DISCONNECT", "cmd", "NL80211_CMD_DISCONNECT"),
    ("CMD_GET_SCAN", "cmd", "NL80211_CMD_GET_SCAN"),
    ("CMD_TRIGGER_SCAN", "cmd", "NL80211_CMD_TRIGGER_SCAN"),
    ("ATTR_WIPHY", "attr", "NL80211_ATTR_WIPHY"),
    ("ATTR_WIPHY_NAME", "attr", "NL80211_ATTR_WIPHY_NAME"),
    ("ATTR_IFINDEX", "attr", "NL80211_ATTR_IFINDEX"),
    ("ATTR_IFNAME", "attr", "NL80211_ATTR_IFNAME"),
    ("ATTR_IFTYPE", "attr", "NL80211_ATTR_IFTYPE"),
    ("ATTR_MAC", "attr", "NL80211_ATTR_MAC"),
    ("ATTR_KEY_DATA", "attr", "NL80211_ATTR_KEY_DATA"),
    ("ATTR_KEY_IDX", "attr", "NL80211_ATTR_KEY_IDX"),
    ("ATTR_KEY_CIPHER", "attr", "NL80211_ATTR_KEY_CIPHER"),
    ("ATTR_KEY_SEQ", "attr", "NL80211_ATTR_KEY_SEQ"),
    ("ATTR_SSID", "attr", "NL80211_ATTR_SSID"),
    ("ATTR_AUTH_TYPE", "attr", "NL80211_ATTR_AUTH_TYPE"),
    ("ATTR_KEY_TYPE", "attr", "NL80211_ATTR_KEY_TYPE"),
    ("ATTR_CIPHER_SUITE_GROUP", "attr", "NL80211_ATTR_CIPHER_SUITE_GROUP"),
    ("ATTR_WPA_VERSIONS", "attr", "NL80211_ATTR_WPA_VERSIONS"),
    ("ATTR_AKM_SUITES", "attr", "NL80211_ATTR_AKM_SUITES"),
    ("ATTR_KEY", "attr", "NL80211_ATTR_KEY"),
    ("ATTR_CIPHER_SUITES_PAIRWISE", "attr", "NL80211_ATTR_CIPHER_SUITES_PAIRWISE"),
    ("ATTR_EXT_FEATURES", "attr", "NL80211_ATTR_EXT_FEATURES"),
    ("ATTR_PMK", "attr", "NL80211_ATTR_PMK"),
    ("ATTR_WIPHY_FREQ", "attr", "NL80211_ATTR_WIPHY_FREQ"),
]


def enum_values(text: str, enum_name: str) -> dict:
    """Walk an enum, honouring explicit assignments -- including the header's own expressions."""
    m = re.search(r"enum\s+" + enum_name + r"\s*\{(.*?)\n\};", text, re.S)
    if not m:
        return {}
    body = re.sub(r"/\*.*?\*/", "", m.group(1), flags=re.S)
    body = re.sub(r"//[^\n]*", "", body)
    vals, running = {}, -1
    for part in body.split(","):
        mm = re.match(r"([A-Za-z_][A-Za-z0-9_]*)\s*(?:=\s*(.+?))?$", part.strip())
        if not mm:
            continue
        name, explicit = mm.group(1), mm.group(2)
        if explicit is None:
            running += 1
        else:
            try:
                running = int(explicit, 0)
            except ValueError:
                try:
                    running = eval(explicit, {}, vals)   # e.g. "1 << 1"
                except Exception:
                    running += 1
        vals[name] = running
    return vals


def suite_macro(text: str, macro: str):
    m = re.search(r"#define\s+" + macro + r"\s+SUITE\((0x[0-9A-Fa-f]+),\s*(\d+)\)", text)
    return (int(m.group(1), 16) << 8) | int(m.group(2)) if m else None


def truth_table(nl_h: str, ieee_h: str) -> dict:
    cmds = enum_values(nl_h, "nl80211_commands")
    attrs = enum_values(nl_h, "nl80211_attrs")
    feats = enum_values(nl_h, "nl80211_ext_feature_index")
    eids = enum_values(ieee_h, "ieee80211_eid")
    out = {}
    for rust, kind, hdr in WANT:
        value = cmds.get(hdr) if kind == "cmd" else attrs.get(hdr)
        if value is not None:
            out[rust] = value
        else:
            print(f"#  {rust}: {hdr} is ABSENT from this header -- do not write a number for it")
    out["CIPHER_SUITE_CCMP"] = suite_macro(ieee_h, "WLAN_CIPHER_SUITE_CCMP")
    out["AKM_SUITE_PSK"] = suite_macro(ieee_h, "WLAN_AKM_SUITE_PSK")
    out["KEYTYPE_GROUP"] = enum_values(nl_h, "nl80211_key_type").get("NL80211_KEYTYPE_GROUP")
    out["KEYTYPE_PAIRWISE"] = enum_values(nl_h, "nl80211_key_type").get("NL80211_KEYTYPE_PAIRWISE")
    out["EID_RSN"] = eids.get("WLAN_EID_RSN")
    out["EID_VENDOR_SPECIFIC"] = eids.get("WLAN_EID_VENDOR_SPECIFIC")
    return {k: v for k, v in out.items() if v is not None}


def check(rust_path: str, truth: dict) -> int:
    src = open(rust_path, encoding="utf-8", errors="replace").read()
    mine = {m.group(1): int(m.group(2), 0)
            for m in re.finditer(r"pub const ([A-Z0-9_]+): u(?:8|16|32) = ([0-9x_a-fA-F]+);", src)}
    bad, checked = [], 0
    for name, want in sorted(truth.items()):
        if name not in mine:
            print(f"  --   {name:30} not declared in the Rust (header says {want})")
            continue
        checked += 1
        if mine[name] == want:
            print(f"  ok   {name:30} {mine[name]}")
        else:
            print(f"  WRONG {name:30} rust={mine[name]}  header={want}")
            bad.append(name)
    print(f"  ---\n  {checked} constants checked, {len(bad)} wrong")
    return 1 if bad else 0


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    nl_h = open(sys.argv[1], encoding="utf-8", errors="replace").read()
    ieee_h = open(sys.argv[2], encoding="utf-8", errors="replace").read() if len(sys.argv) > 2 and not sys.argv[2].startswith("--") else ""
    truth = truth_table(nl_h, ieee_h)
    if "--check" in sys.argv:
        target = sys.argv[sys.argv.index("--check") + 1]
        print(f"  checking {target} against the headers")
        return check(target, truth)
    for name, value in sorted(truth.items()):
        print(f"  {name:30} {value}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
