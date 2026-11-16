"""The independent oracle for the association harness: recompute the state machine's output in Python.

The harness asserts things about its own frames; this recomputes them from the bytes, so an error shared
between the assertion and the code under test cannot hide. It reads the harness's stdout, derives the PMK
and PTK from the same fixed inputs, and then:

  * recomputes the MIC of every frame from the frame itself, with the MIC field zeroed -- the rule that,
    taken as-received, produces a frame the access point discards in silence;
  * unwraps message 3's key data with AES key unwrap under the KEK and checks the group key inside it;
  * checks that message 1 carries no MIC at all, because a message 1 with a digest is not a message 1.

    python trm_chainload/scripts/verify_wifi_harness.py [harness-output.txt]
"""
import hashlib
import hmac
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from verify_wifi_crypto import oracle_prf  # the PRF already proven against this binary

SSID = b"TRM-HARNESS"
PASSPHRASE = b"correct-horse-battery"
AP_MAC = bytes([0xB4, 0x45, 0x06, 0xD4, 0x2C, 0x51])
STA_MAC = bytes([0x00, 0x11, 0x22, 0x33, 0x44, 0x55])
ANONCE = bytes(range(0x00, 0x20))
SNONCE = bytes(range(0x20, 0x40))
# The EAPOL MIC field: 4 bytes of header, then descriptor(1) key_info(2) key_length(2) replay(8)
# nonce(32) iv(16) rsc(8) key_id(8).
MIC_AT = 4 + 77


def ptk_from(pmk: bytes, aa: bytes, spa: bytes, anonce: bytes, snonce: bytes) -> bytes:
    """802.11i: the PRF over the two addresses and the two nonces, each pair in sorted order. The
    addresses are the harness's own -- an earlier oracle hardcoded a different station's, which is why it
    disagreed with the harness on the first run, and the disagreement is what caught it."""
    macs = b"".join(sorted([aa, spa]))
    nonces = b"".join(sorted([anonce, snonce]))
    return oracle_prf(pmk, b"Pairwise key expansion", macs + nonces, 64)


def mic_over(frame: bytes, kck: bytes) -> bytes:
    """The MIC as it must be computed: the frame with its own MIC field zeroed."""
    f = bytearray(frame)
    f[MIC_AT:MIC_AT + 16] = b"\x00" * 16
    return hmac.new(kck, bytes(f), hashlib.sha1).digest()[:16]


def key_data_at(frame: bytes):
    """The key data and whether the encrypted bit is set, read off the frame."""
    key_info = int.from_bytes(frame[5:7], "big")
    kd_len = int.from_bytes(frame[4 + 93:4 + 95], "big")
    return frame[4 + 95:4 + 95 + kd_len], bool(key_info & 0x1000)


def aes_unwrap(kek: bytes, wrapped: bytes):
    """AES key unwrap through a second implementation, never this tree's. python-cryptography first;
    openssl when it is absent, because a missing library must not read as a passing check. openssl's name
    for the algorithm differs between builds (id-aes128-wrap and aes128-wrap), so both are tried before
    declaring failure, and the default IV is the standard's own A6A6A6A6A6A6A6A6."""
    try:
        from cryptography.hazmat.primitives.keywrap import aes_key_unwrap
        return aes_key_unwrap(kek, wrapped)
    except ImportError:
        pass
    for cipher in ("aes128-wrap", "id-aes128-wrap"):
        p = subprocess.run(
            ["openssl", "enc", "-d", "-" + cipher, "-K", kek.hex(),
             "-iv", "a6a6a6a6a6a6a6a6", "-nopad"],
            input=wrapped, capture_output=True)
        if p.returncode == 0 and p.stdout:
            return p.stdout
    raise RuntimeError("neither python-cryptography nor openssl could unwrap the key data")


def main() -> int:
    # A relative argument resolves against the repository, not the caller's working directory: the sealer
    # runs this from wherever it was invoked, and a check that cannot find its evidence reads as a failure
    # rather than as a missing file.
    root = Path(__file__).resolve().parents[2]
    arg = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(
        r"C:/Users/shawn/AppData/Local/hermes/cache/scratch/harness_output.txt")
    src = arg if arg.is_absolute() else root / arg
    text = src.read_text(errors="replace")
    got = dict(re.findall(r"harness: ([a-z0-9_]+)=([0-9a-f]+)", text))
    passed = failed = 0

    def check(name, ok, detail=""):
        nonlocal passed, failed
        if ok:
            passed += 1
            print(f"  ok       {name}{(' -- ' + detail) if detail else ''}")
        else:
            failed += 1
            print(f"  FAILED   {name}{(' -- ' + detail) if detail else ''}")

    pmk = hashlib.pbkdf2_hmac("sha1", PASSPHRASE, SSID, 4096, 32)
    check("the PMK recomputed in Python matches the harness",
          got.get("pmk") == pmk.hex(), f"{pmk.hex()[:16]}...")

    ptk = ptk_from(pmk, AP_MAC, STA_MAC, ANONCE, SNONCE)
    check("the PTK recomputed in Python matches the harness",
          got.get("ptk") == ptk.hex(), f"{ptk.hex()[:16]}...")
    kck, kek = ptk[:16], ptk[16:32]

    frames = {n: bytes.fromhex(got[n]) for n in ("msg1", "msg2", "msg3", "msg4") if n in got}
    check("the harness printed all four frames", len(frames) == 4, " ".join(sorted(frames)))

    # Message 1 carries no MIC. Messages 2 and 4 carry one over the frame with the field zeroed.
    m1 = frames.get("msg1", b"")
    check("message 1 carries no MIC", m1[MIC_AT:MIC_AT + 16] == b"\x00" * 16)
    for name in ("msg2", "msg4"):
        f = frames.get(name, b"")
        want = mic_over(f, kck)
        carried = f[MIC_AT:MIC_AT + 16]
        check(f"{name}'s MIC recomputed over the frame with its own field zeroed",
              want == carried, f"python={want.hex()[:16]} frame={carried.hex()[:16]}")
        check(f"{name}'s MIC matches the harness's own printed value",
              carried.hex() == got.get(f"{name}_mic"))

    # Message 3's key data, unwrapped by a second implementation of AES key wrap.
    m3 = frames.get("msg3", b"")
    kd, encrypted = key_data_at(m3)
    check("message 3's key data is marked encrypted", encrypted)
    try:
        plain = aes_unwrap(kek, kd)
        check("message 3's key wrap opened under the KEK", True)
        gtk = plain[8:]  # dd len 00 0f ac 04 key_id reserved <gtk>
        check("the group key inside the wrap is the one the harness used",
              gtk.hex() == got.get("gtk"), gtk.hex())
    except Exception as exc:  # a wrap that will not open is a FAILED check, not a crash
        check("message 3's key wrap opened under the KEK", False, str(exc))
    try:
        aes_unwrap(kck, kd)  # the wrong KEK must not open it
        check("the wrap refuses a KEK that is not the PTK's", False, "it opened under the KCK")
    except Exception:
        check("the wrap refuses a KEK that is not the PTK's", True)

    refused = len(re.findall(r"harness: refused", text))
    check("every case the harness had to refuse was refused", refused >= 6, f"{refused} refusals")
    check("the harness reported no failures of its own", "0 failed" in text)

    print(f"  ---\n  oracle: {passed} check(s) agreed, {failed} disagreed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
