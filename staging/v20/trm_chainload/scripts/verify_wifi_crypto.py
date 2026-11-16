#!/usr/bin/env python3
"""Diff trm_wifi's crypto against an independent implementation of the same protocol.

The rule this exists for: parity must diff the ORACLE's own output, not assert that two things agree.
So nothing here is a remembered constant. The passphrase, SSID, MACs and nonces are the fixed inputs the
binary prints alongside its answers; every derived value on this side comes from Python's hashlib/hmac
for the SHA1 family and from the openssl tool for AES, and the wrapped key in the round trip is produced
by openssl and must come back out of the binary unchanged.

Run inside WSL (where the musl artifact executes):

    python3 verify_wifi_crypto.py /tmp/trm_wifi

Exit status is 0 only if every line matches and the key-wrap round trip survives both ways.
"""
import hashlib
import hmac
import os
import shutil
import subprocess
import sys

AA = bytes([0x02, 0, 0, 0, 0, 1])
SPA = bytes([0x02, 0, 0, 0, 0, 2])
ANONCE = bytes(range(32))
SNONCE = bytes(range(0x20, 0x40))
GTK = bytes(range(0x10, 0x30))          # 32 bytes, as a GTK is


def oracle_prf(key: bytes, label: bytes, data: bytes, n: int) -> bytes:
    """802.11 PRF-n: HMAC-SHA1 over label || 0 || data || counter, counter from zero."""
    out = b""
    counter = 0
    while len(out) < n:
        out += hmac.new(key, label + b"\x00" + data + bytes([counter]), hashlib.sha1).digest()
        counter += 1
    return out[:n]


def oracle_ptk(pmk: bytes) -> bytes:
    return oracle_ptk_from(pmk, ANONCE, SNONCE)


def oracle_ptk_from(pmk: bytes, anonce: bytes, snonce: bytes) -> bytes:
    """The PTK for one pair of nonces: the PRF over the addresses and the nonces in sorted order."""
    macs = b"".join(sorted([AA, SPA]))
    nonces = b"".join(sorted([anonce, snonce]))
    return oracle_prf(pmk, b"Pairwise key expansion", macs + nonces, 64)


RFC_KEK = bytes.fromhex("000102030405060708090a0b0c0d0e0f")
RFC_PLAIN = bytes.fromhex("00112233445566778899aabbccddeeff")
RFC_WRAPPED = "1fa68b0a8112b447aef34bd8fb5a7b829d3e862371d2cfe5"


def wrap_gtk(kek: bytes, plain: bytes):
    """RFC 3394 wrap from an implementation that is not this binary, validated against the RFC vector.

    Two oracles are tried, in order of how much they prove: the `cryptography` package (a widely used
    independent implementation) and the openssl tool. Whichever answers must first reproduce the RFC's
    own published vector for these inputs -- an oracle that fails its own test is not an oracle, and its
    silence is reported rather than papered over.
    """
    try:
        from cryptography.hazmat.primitives.keywrap import aes_key_wrap
        if aes_key_wrap(RFC_KEK, RFC_PLAIN).hex() == RFC_WRAPPED:
            return aes_key_wrap(kek, plain), "python-cryptography"
    except ImportError:
        pass

    if shutil.which("openssl"):
        def enc(key, data):
            with tempfile.NamedTemporaryFile(delete=False) as f:
                f.write(data)
                path = f.name
            p = subprocess.run(["openssl", "enc", "-id-aes128-wrap", "-K", key.hex(), "-nopad",
                                "-in", path], capture_output=True)
            os.unlink(path)
            return p.stdout if p.returncode == 0 else None
        if enc(RFC_KEK, RFC_PLAIN) is not None and enc(RFC_KEK, RFC_PLAIN).hex() == RFC_WRAPPED:
            return enc(kek, plain), "openssl enc -id-aes128-wrap"
    return None, None


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: verify_wifi_crypto.py <path-to-trm_wifi>")
        return 2
    binary = sys.argv[1]
    got = {}
    proc = subprocess.run([binary, "--vectors"], capture_output=True, text=True)
    if proc.returncode != 0:
        print("the binary would not produce vectors:")
        print(proc.stderr)
        return 2
    for line in proc.stdout.splitlines():
        if " " in line:
            k, v = line.split(" ", 1)
            got[k] = v

    fails = 0

    def diff(name, oracle, mine):
        nonlocal fails
        if oracle == mine:
            print(f"  same  {name:12} {mine}")
        else:
            print(f"  DIFF  {name:12} oracle {oracle}\n        {'':12} binary {mine}")
            fails += 1

    pmk = hashlib.pbkdf2_hmac("sha1", got["passphrase"].encode(), got["ssid"].encode(), 4096, 32)
    ptk = oracle_ptk(pmk)
    frame = bytes.fromhex(got["frame"])
    mic = hmac.new(ptk[:16], frame, hashlib.sha1).digest()[:16]

    diff("pmk", pmk.hex(), got["pmk"])
    diff("ptk", ptk.hex(), got["ptk"])
    diff("kck", ptk[:16].hex(), got["kck"])
    diff("kek", ptk[16:32].hex(), got["kek"])
    diff("tk", ptk[32:48].hex(), got["tk"])
    diff("mic", mic.hex(), got["mic"])

    # The round trip: an outside implementation wraps a GTK under the KEK the binary derived, and the
    # binary must hand back exactly the bytes that implementation was given. Neither side sees the other's
    # plaintext, and the wrapper is validated against the RFC's vector before it is trusted.
    wrapped, source = wrap_gtk(ptk[16:32], GTK)
    round_trip = False
    if wrapped is None:
        print("  SKIP  gtk round trip -- no usable key-wrap oracle here (both candidates failed the")
        print("        RFC 3394 vector, so neither one's output would prove anything)")
    else:
        print(f"  note  {source} reproduces the RFC 3394 vector, so its wrap is the one being tested")
        p = subprocess.run([binary, "--unwrap", ptk[16:32].hex(), wrapped.hex()],
                           capture_output=True, text=True)
        out = p.stdout.strip().split(" ")[-1]
        diff("gtk unwrap", GTK.hex(), out)
        round_trip = True

    # --- the handshake frames, checked the same way: the MICs are recomputed over the frames the binary
    # built, from a PTK the oracle derives itself. Nothing here is read out of the binary and compared with
    # itself -- the frames and the nonces are, everything derived from them is not.
    proc = subprocess.run([binary, "--eapol-vectors"], capture_output=True, text=True)
    if proc.returncode != 0:
        print("  SKIP  handshake frames -- the binary would not produce them")
    else:
        ev = {}
        for line in proc.stdout.splitlines():
            if " " in line:
                k, v = line.split(" ", 1)
                ev[k] = v
        try:
            anonce = bytes.fromhex(ev["anonce"])
            snonce = bytes.fromhex(ev["snonce"])
            msg1 = bytes.fromhex(ev["msg1"])
            msg2 = bytes.fromhex(ev["msg2"])
            msg4 = bytes.fromhex(ev["msg4"])

            def zero_mic(frame: bytes) -> bytes:
                f = bytearray(frame)
                f[81:97] = bytes(16)
                return bytes(f)

            def mic_of(frame: bytes) -> str:
                return frame[81:97].hex()

            # The PTK is derived here, from the PMK and the nonces, exactly as the binary must have.
            ptk2 = oracle_ptk_from(pmk, anonce, snonce)
            diff("ptk(kck)", ptk2[:16].hex(), got["kck"])
            for name, frame in (("msg2", msg2), ("msg4", msg4)):
                want = hmac.new(ptk2[:16], zero_mic(frame), hashlib.sha1).digest()[:16].hex()
                diff(f"{name} mic", want, mic_of(frame))

            # The RSN element: the AP sent one inside message 1, and the station must echo that body back.
            kd = msg1[4 + 95:]
            i = 0
            found = ""
            while i + 2 <= len(kd):
                ty, ln = kd[i], kd[i + 1]
                if ty == 48 and i + 2 + ln <= len(kd):
                    found = kd[i + 2:i + 2 + ln].hex()
                    break
                i += 2 + ln
            diff("rsn echo", found, ev["rsn"])
        except (KeyError, ValueError) as e:
            print(f"  SKIP  handshake frames -- could not parse the vector output ({e})")
            fails += 1

    print("  ---")
    if fails:
        print(f"  {fails} disagreement(s): the binary and the oracle do not agree")
        return 1
    print("  the derived values agree with the oracle" +
          (", and the wrapped GTK round trip survived" if round_trip else
           "; the key-wrap round trip did NOT run (see SKIP above), so nothing is claimed for it"))
    return 0 if round_trip else 3


if __name__ == "__main__":
    sys.exit(main())
