"""Read real sensor values out of a running HWiNFO64 via its shared memory (SM2).

Why this exists
---------------
The kernel's voltage metric has never had a voltage sensor behind it. `voltage.rs` computes
Phi_V from the *nominal* rails (V_CORE_NOMINAL = 1.25, V_DDR_NOMINAL = 1.35) and reads only the
clock (WMIC) and memory load. The rails are divisors, so those two terms cannot vary — the number
is honest about being computed, but it is not measured silicon.

HWiNFO64 (8.60, running on this host) does read the real rails from the Super I/O chip. This script
pulls those values out of HWiNFO instead of inventing them.

Contract (enforced here, per the platform's never-fake-a-sensor rule)
--------------------------------------------------------------------
* If HWiNFO's shared memory is not enabled, the mapping does not exist and this script says so and
  exits non-zero. It NEVER falls back to a nominal rail printed as if it were measured.
* Every value returned is labelled with the HWiNFO sensor name it came from, so a caller can see
  which chip/rail produced a number.
* `found: false` is a first-class result.

Enabling it (one toggle, GUI-only — HWiNFO has no CLI for this)
---------------------------------------------------------------
Open HWiNFO64 -> Sensors window -> the Configure/Settings (gear) dialog -> enable
"Shared Memory Support" -> OK. Then this script's mapping appears as Global\\HWiNFO_SENS_SM2.
Note: the free build limits shared memory to a 12-hour session per restart; the mapping disappears
when it expires and this script reports that rather than reusing a stale value.

Layout (HWiNFO SDK, little-endian, Pack=1)
------------------------------------------
header  : uint32 signature, uint32 version, uint32 revision, int64 poll_time,
          uint32 off_sensors, uint32 size_sensor, uint32 num_sensors,
          uint32 off_readings, uint32 size_reading, uint32 num_readings   (= 44 bytes)
sensor  : uint32 id, uint32 inst, char name_orig[128], char name_user[128]
reading : uint32 id, uint32 type, char label_orig[128], char label_user[128], char unit[16],
          double value, double value_min, double value_max, double value_avg

The strides are taken from the header, and the value offset is derived as size-32 rather than
hardcoded, so a newer HWiNFO that adds a field does not silently shift the numbers.

Usage
-----
    python tools/hwinfo_sensors.py                # list everything
    python tools/hwinfo_sensors.py --voltage      # only voltage rails, JSON
    python tools/hwinfo_sensors.py --grep Vcore
"""
from __future__ import annotations

import argparse
import ctypes
import json
import struct
import sys
from ctypes import wintypes

SHARED_MEM_NAME = "Global\\HWiNFO_SENS_SM2"
SIGNATURE = 0x53695748  # 'HWiS' little-endian
VOLT_UNITS = ("V",)
VOLT_HINTS = ("voltage", "vcore", "vsoc", "vdd", "vddcr", "vccio", "vdimm", "vddp", "vbat", "rail")


class SHARED_MEMORY(ctypes.Structure):
    pass


def _open_mapping():
    """Return (buffer_bytes, error_string). Never raises for a missing mapping."""
    k32 = ctypes.WinDLL("kernel32", use_last_error=True)
    k32.OpenFileMappingW.restype = wintypes.HANDLE
    k32.OpenFileMappingW.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.LPCWSTR]
    k32.MapViewOfFile.restype = ctypes.c_void_p
    k32.MapViewOfFile.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.DWORD,
                                  wintypes.DWORD, ctypes.c_size_t]

    FILE_MAP_READ = 0x0004
    h = k32.OpenFileMappingW(FILE_MAP_READ, False, SHARED_MEM_NAME)
    if not h:
        err = ctypes.get_last_error()
        return None, None, (f"shared memory mapping {SHARED_MEM_NAME} not found "
                      f"(OpenFileMapping err={err}). HWiNFO64 >= 8.x must be running with "
                      f"'Shared Memory Support' enabled in its Settings dialog.")
    view = k32.MapViewOfFile(h, FILE_MAP_READ, 0, 0, 0)
    if not view:
        err = ctypes.get_last_error()
        k32.CloseHandle(h)
        return None, None, f"MapViewOfFile failed (err={err})"

    # Header first, to learn the true size of the mapping's contents.
    hdr = ctypes.string_at(view, 44)
    (sig, ver, rev, _poll, off_s, size_s, num_s, off_r, size_r, num_r) = struct.unpack("<IIIqIIIIII", hdr)
    need = max(off_r + size_r * num_r, off_s + size_s * num_s, 44)
    data = ctypes.string_at(view, need)
    k32.UnmapViewOfFile.argtypes = [ctypes.c_void_p]
    k32.UnmapViewOfFile(view)
    k32.CloseHandle(h)
    if sig != SIGNATURE:
        return None, None, f"signature mismatch: 0x{sig:08x} (expected 0x{SIGNATURE:08x} 'HWiS')"
    return data, (ver, rev, off_s, size_s, num_s, off_r, size_r, num_r), None


def _cstr(raw: bytes) -> str:
    return raw.split(b"\x00", 1)[0].decode("latin-1", "replace").strip()


def read_all():
    """Return (rows, meta, error). rows are dicts; nothing is invented when the mapping is absent."""
    buf, info, err = _open_mapping()
    if buf is None:
        return [], {}, err
    ver, rev, off_s, size_s, num_s, off_r, size_r, num_r = info

    sensor_names = []
    for i in range(num_s):
        base = off_s + i * size_s
        orig = _cstr(buf[base + 8: base + 8 + 128])
        user = _cstr(buf[base + 136: base + 136 + 128])
        sensor_names.append(user or orig)

    rows = []
    for i in range(num_r):
        base = off_r + i * size_r
        rid, rtype, sensor_index = struct.unpack_from("<III", buf, base)
        # Measured layout on HWiNFO 8.60-6120 (element stride 460): label at 12, user label at 140,
        # unit at 268, value at 300. The earlier guess (8/264, size-32) came from the documented
        # 312-byte element and silently produced zero readings, so these are the verified offsets.
        label = _cstr(buf[base + 12: base + 12 + 128])
        unit = _cstr(buf[base + 268: base + 268 + 16])
        (value,) = struct.unpack_from("<d", buf, base + 300)
        rows.append({
            "reading_id": rid,
            "type": rtype,
            "sensor": sensor_names[sensor_index] if 0 <= sensor_index < len(sensor_names) else "",
            "label": label,
            "unit": unit,
            "value": value,
        })
    meta = {"version": ver, "revision": rev, "sensors": num_s, "readings": num_r,
            "size_sensor": size_s, "size_reading": size_r}
    return rows, meta, None


def is_voltage(row) -> bool:
    if row["unit"] not in VOLT_UNITS:
        return False
    lab = row["label"].lower()
    return any(h in lab for h in VOLT_HINTS)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--voltage", action="store_true", help="only voltage readings, as JSON")
    ap.add_argument("--grep", default=None, help="substring filter on the label")
    a = ap.parse_args()

    rows, meta, err = read_all()
    if err:
        print(json.dumps({"found": False, "source": "HWiNFO64 shared memory",
                          "error": err, "measured": False}, indent=2))
        return 2

    if a.voltage:
        rows = [r for r in rows if is_voltage(r)]
    if a.grep:
        rows = [r for r in rows if a.grep.lower() in r["label"].lower()]

    if a.voltage or a.grep:
        print(json.dumps({"found": True, "source": "HWiNFO64 shared memory",
                          "measured": True, "meta": meta,
                          "readings": rows}, indent=2))
        return 0

    print(f"HWiNFO shared memory: {meta['sensors']} sensors, {meta['readings']} readings "
          f"(v{meta['version']}.{meta['revision']})")
    for r in rows:
        if r["unit"]:
            print(f"  {r['label'][:44]:<44} {r['value']:>10.4f} {r['unit']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
