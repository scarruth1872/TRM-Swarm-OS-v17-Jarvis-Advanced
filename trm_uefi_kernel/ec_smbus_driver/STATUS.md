# trm_ec_smbus — status, measured 2026-11-12

Source here is the driver that was written on 2026-10-06 (`trm_ec_smbus.c`, ~6 KB). The compiled
`trm_ec_smbus.sys` stays in `staging/v20/trm_uefi_kernel/ec_smbus_driver/` and is **deliberately not
copied here**, because a test-signed binary in the live tree invites the assumption that it runs.

## What is true right now

| fact | measured value |
|---|---|
| driver binary exists | yes, `staging/v20/.../trm_ec_smbus.sys`, 6,016 bytes |
| its signature | self-signed `CN=TRM EC SMBus Test`, thumbprint `CFCB178B28452219207EFF34753C6BC22EEDDEB0`, **Status = NotTrusted** |
| test-signing mode | **OFF** (`bcdedit` shows no `testsigning` entry) |
| WDK installed | **NO** — `Windows Kits/10/Include/*/km` does not exist (SDK only) |
| service registered | no (`sc query trm_ec_smbus` → 1060, does not exist) |
| shell elevation | yes (`IsInRole(Administrator)` = True) |

So the driver cannot be rebuilt here (no `km` headers, no `ntddk.h`) and cannot be loaded as-is
(NotTrusted + test-signing off). Both are fixable, but neither is a code change:

1. install the WDK (matching the installed SDK 10.0.26100 or 22621) to compile;
2. `bcdedit /set testsigning on` **plus a reboot**, then `sc create trm_ec_smbus type= kernel` and
   load; the self-signed cert must also be in `Trusted Root`/`Trusted Publishers`.

## Why this is not the frequency answer

The frequency gap is not an EC/SMBus problem. Measured on this host (2026-11-12):

* `Win32_Processor.CurrentClockSpeed` = `MaxClockSpeed` = 4000 (static);
* PDH `\Processor Information(_Total)\Processor Frequency` = 4000, `% Processor Performance`
  jittering 99–100, `% of Maximum Frequency` = 100;
* `CallNtPowerInformation(ProcessorInformation)` → every core `cur 4000 / max 4000`, idle state 1/1;
* LibreHardwareMonitor 0.9.6 (downloaded, `LibreHardwareMonitorLib.dll`) reports every Clock sensor
  as null/zero, and the package ships **no** `.sys` to read MSRs with.

A frequency signal needs APERF/MPERF (MSR `0xC00000E7` / `0xC00000E8`) or P-state status
(`0xC0010063`), which is ring-0 only. EC/SMBus gives **voltages**, which is a different gap and the one
this driver was written for.

## What would make the clock term real, in order of cost

1. **Change the policy** — the active plan sets minimum processor state to 100 % on AC. Tested: setting
   it to 5 % on a duplicated plan did **not** move any reading, so this is not the explanation.
2. **Signed third-party agent** (HWiNFO, AMD uProf) — installs its own ring-0 driver; needs a download
   and an install decision.
3. **Our own MSR driver** — WDK install + test-signing reboot, ~150 lines for
   `TRM_IOCTL_MSR_READ`, then APERF/MPERF over a timed interval gives real MHz.
