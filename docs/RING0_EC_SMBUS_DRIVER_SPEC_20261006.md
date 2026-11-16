# Ring-0 EC/SMBus Voltage Driver — Specification (2026-10-06)

Goal: read the real voltage rails the BIOS/EC knows, that userspace cannot — the prerequisite the
council ratified as "null until it exists, never fabricated."

## Verified hardware (measured, this host)

| piece | identity | access |
|---|---|---|
| Embedded Controller (EC) | `ACPI\PNP0C09` | I/O ports **0x62 (data) / 0x66 (command-status)** |
| SMBus controller | AMD `PCI\VEN_1022&DEV_790B` (FCH) | PCI config → BAR → I/O base + 0x0C/0x0E offset |
| privilege | admin shell (elevated) | can load a signed driver |
| existing helper | none (no inpout32/WinRing0) | clean slate |

## EC command protocol (PNP0C09)

The EC is the ACPI "embedded controller" — a mailbox. Read EC RAM:

1. write the EC RAM index to port **0x66** (command = 0x80 read / 0x81 write)
2. poll port **0x66** bit 1 (IBF, input-buffer-full) until clear
3. read/write the data byte at port **0x62**

The voltage/temperature offsets in EC RAM are **vendor-specific** (Alienware/Dell layout). The
driver must therefore ship a probe pass: read a candidate offset range, and report the raw bytes
plus which offsets changed under load, so the mapping is *discovered*, not hardcoded from a
marketing sheet. No offset is trusted until it correlates with a load change.

## SMBus protocol (AMD 790B)

The AMD FCH SMBus has two fixed I/O ranges decoded from the PCI BAR (typically the SMBus base at
BAR offset, the ASF controller at base+0x20). Reading a PMBus VR:

1. SMBus address + command → SMBus Host Status/Control registers (base+0x00..0x0E)
2. poll Host Status (base+0x00) for transaction complete
3. read Host Data (base+0x05) for the returned voltage word

The VR addresses (VID/VRM devices, typically 0x60–0x7F range for the CPU VR, DRAM VR, SoC VR) are
enumerated by a 7-bit address scan — again discovered, not asserted.

## Driver design

A WDM/`no_std` kernel driver (`trm_ec_smbus.sys`) exposing a device interface with three IOCTLs:

| IOCTL | returns |
|---|---|
| `TRM_IOCTL_EC_READ` (index) | one EC RAM byte |
| `TRM_IOCTL_EC_SCAN` (start, len) | raw EC RAM block for the probe pass |
| `TRM_IOCTL_SMBUS_READ` (addr, cmd) | one SMBus transaction word |

Implementation uses only the HAL port primitives (`READ_PORT_UCHAR`, `WRITE_PORT_UCHAR`,
`KeStallExecutionProcessor`) — no IRP queue, no PnP, the minimal surface.

## Rust caller (the kernel side)

`host_council_server` opens `\\.\trm_ec_smbus` and `DeviceIoControl`s the three codes. The result
is folded into `/native/shadow-bios` under `voltages`, replacing the current `null` — with the
basis `"measured: EC RAM / SMBus PMBus, via trm_ec_smbus.sys"`.

## Build / sign / load plan

1. Build `trm_ec_smbus.sys` with the WDK (or `windows-drivers-rs`); target x64.
2. **Test-sign** (self-signed) + load on this admin shell with test-signing enabled — enough to
   prove the reads, not for fleet distribution.
3. Probe pass: EC RAM scan + SMBus address scan; record which offsets/addresses respond.
4. Correlate the discovered offsets with a load change (spin CPU / draw GPU) to confirm they are
   the rails, not noise.
5. Promote to `/native/shadow-bios` voltages with the measured values + the discovered mapping.

## Honest bounds

- Step 3's "discovered" mapping is the gate: until an offset correlates with load, it is a byte,
  not a voltage. The driver reports raw bytes and the correlation status — it never asserts a
  rail it has not tied to a load change.
- Test-signed drivers require boot-time test-signing; this is for the dev platform, not a
  production signing story.
- This driver is the *one* way to close the voltage gap without fabricating. Until it loads, the
  honest value stays `null`.

---

## Status (2026-10-07) — outcome, not just plan

The ring-0 `trm_ec_smbus.sys` path is **correct but blocked on this host**: C: is BitLocker-encrypted
(100% TPM), so `bcdedit /set testsigning on` changes a PCR and forces recovery. Test-signing is
permanently off the table here; the production path needs an EV cert + Microsoft attestation (2–6 weeks).

The voltage gap was instead closed from **userspace** through the AMD-signed `AMDRyzenMasterDriverV20`:
`NtCreateFile` on `\Device\AMDRyzenMasterDriverV20` + `DeviceIoControl(0x81112EE0)` reads table-indexed
MSRs. The P-state MSRs (`PSTATE_0/1` = `0xC0010064/65`) hold the VID (voltage ID), decoded
`V = 1.55 − VID × 6.25 mV` (Renesas/Intersil ISL62771 SVI2 VRM datasheet).

**Live result:** P0 VID 77 → **1.06875 V**, P1 VID 88 → **1.0 V** — the CPU's *requested* P-state
targets, folded into `/native/shadow-bios` as `reference_voltage_v_p0/p1` (honest: a target, not the
measured rail).

**Voltage doctrine applied:** `Φ_V = (V_core/1.25)² × (f/4000) × (V_ddr/1.35) × max(0.5, 1−mem%/200)`
now uses the live V_core instead of the collapsing rail term 1.0. Live `voltage_phi_v = 0.37833`
(host clock 3301 MHz, mem 74.6%) — below the 0.82 resonance-lock threshold, so the substrate honestly
reports *not locked*.

**Still unreachable:** the *measured* rail. The P-state definitions are targets, not the live rail;
that needs VRM SVI2 telemetry or a motherboard monitor IC (both unconfirmed on this board).
