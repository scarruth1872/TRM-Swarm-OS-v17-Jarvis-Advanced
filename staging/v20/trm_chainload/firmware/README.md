# Firmware this image stages into /lib/firmware

A driver that requests firmware when it binds needs the file in the root filesystem *at that moment* --
and on this pathway the RAM image is the root filesystem then. The board's RTL8125B asked and got
`Direct firmware load for rtl_nic/rtl8125b-2.fw failed with error -2`.

| file (as the kernel asks for it) | bytes | sha256 | source |
|---|---|---|---|
| `rtl_nic/rtl8125b-2.fw` | 3,328 | `529bf1c25c97ff52b401090d00ff89cc22351012336e5a0c9662728a3ee909ef` | linux-firmware, `rtl_nic/rtl8125b-2.fw` |
| `iwlwifi-cc-a0-77.ucode` | 1,368,100 | `1866a58a1daee56b120362122d5f7548b443b53d33f2558a7e4407d58fd27dab` | linux-firmware, `intel/iwlwifi/iwlwifi-cc-a0-77.ucode` |
| `regulatory.db` | 6,348 | `7e236caecd939c8ec98be4870bf30422f28ffef2565a38aaaa2d9ddabd0c2641` | wireless-regdb 2026.09.03 |
| `regulatory.db.p7s` | 1,085 | `50332f0db09b8bcc719235ec4985c27377f2cc2f42abb7b5dcf951866c5a888a` | wireless-regdb 2026.09.03 |

All four are for this machine's own hardware -- Realtek RTL8125B (`10ec:8125`) and Intel AX200
(`8086:2723`, whose `iwlwifi.ko` names `iwlwifi-cc-a0-77` itself). All fetched from kernel.org, unmodified,
carrying their publishers' own redistribution terms. Staged by `make_rootfs.py --firmware
trm_chainload/firmware`, which walks this directory, so a second device is a second file rather than a
second mechanism.

Two things learned here by measuring rather than by following a howto:

- **The Wi-Fi firmware is not at the tree's root.** `WHENCE`, the tree's own manifest, lists both
  `File: intel/iwlwifi/iwlwifi-cc-a0-77.ucode` and `Link: iwlwifi-cc-a0-77.ucode -> intel/iwlwifi/...`: the
  root name the kernel asks for is a symlink, so fetching the root path returns 404 while the real path
  returns the blob. It is staged here under the root name, which is the name the kernel will use.
- **`regulatory.db` is load-bearing, not decorative**, because the kernel that boots this drive is built
  with `CONFIG_CFG80211_REQUIRE_SIGNED_REGDB=y` -- an absent or unsigned database is not a warning the
  system can shrug off. The pair comes from the separate wireless-regdb project, not from linux-firmware,
  and its signature was checked before staging (`openssl smime -verify -noverify` over the database:
  *Verification successful*). That check proves the signature is genuine; whether the kernel trusts the
  signer is its own built-in key, and the boot log is where that answer appears.
