#!/usr/bin/env python3
"""Post-boot capability test suite: exercises every command the booted
platform supports and compares against the full platform.

Structure:
  1. Native kernel commands (trm_swarm_boot.exe — shares kernel code)
  2. UEFI kernel command handlers (static verification of wiring)
  3. Full-platform equivalents (python) for parity check
  4. Multi-model capability assessment
"""
import subprocess, sys, os, json, re

EXE = r"F:\Development sites\TRM-Swarm-OS-v2\swarm_v2_rust_kernel\target\release\trm_swarm_boot.exe"
KERNEL_SRC = r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\src"
PY = r"C:/Users/shawn/AppData/Local/Programs/Python/Python311/python.exe"
ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"

def run_native(args, timeout=30):
    try:
        r = subprocess.run([EXE] + args, capture_output=True, text=True, timeout=timeout)
        return (r.stdout + r.stderr)[:600]
    except Exception as e:
        return f"ERR: {e}"

def run_py(code, timeout=60):
    try:
        r = subprocess.run([PY, "-c", code], capture_output=True, text=True, timeout=timeout,
                           cwd=ROOT)
        return (r.stdout + r.stderr)[:400]
    except Exception as e:
        return f"ERR: {e}"

results = []

# ── 1. Native kernel command tests ──
native_tests = [
    ("verify", ["verify"]),
    ("council", ["council", "--topic", "Test post-boot parity", "-c", "general", "-p", "OPERATOR"]),
    ("bench", ["bench"]),
    ("gui headless", ["gui", "--frames", "3", "--headless"]),
]
print("=== NATIVE KERNEL COMMANDS ===")
for name, args in native_tests:
    out = run_native(args)
    ok = "ERR" not in out and len(out) > 20
    results.append({"type": "native", "command": name, "status": "PASS" if ok else "FAIL", "output": out[:150]})
    print(f"[{'PASS' if ok else 'FAIL'}] {name}: {out[:80].strip()}")

# ── 2. UEFI kernel command wiring (static verification) ──
print("\n=== UEFI KERNEL COMMAND WIRING ===")
main_rs = open(os.path.join(KERNEL_SRC, "main.rs"), encoding="utf-8").read()
uefi_cmds = ["monad", "status", "agents", "council", "jarvis", "nand", "quic", "mesh", "hw",
             "radar", "setup", "clear", "help", "trm", "k3", "moe", "twin"]
for cmd in uefi_cmds:
    wired = f'cmd.starts_with(b"{cmd}")' in main_rs or f'cmd == b"{cmd}"' in main_rs
    results.append({"type": "uefi", "command": cmd, "status": "PASS" if wired else "FAIL"})
    print(f"[{'PASS' if wired else 'FAIL'}] uefi command: {cmd}")

# ── 3. Platform parity (python equivalents) ──
print("\n=== FULL PLATFORM PARITY ===")
py_tests = [
    ("council", "from swarm_v2.core.swarm_council import SwarmCouncil; c=SwarmCouncil(); p=c.propose(topic='parity test', category='general', proposer='t'); print('proposal ok')"),
    ("k3", "from swarm_v2.core.ternary_kleene_engine import KleeneTernaryEngine; e=KleeneTernaryEngine(); print('k3 ok')"),
    ("digital twin", "from swarm_v2.core.digital_twin_singleton import get_digital_twin; t=get_digital_twin(); print('twin ok, slots:', len(t.slots))"),
    ("voltage", "from swarm_v2.core.hardware_voltage_microkernel_binder import HardwareVoltageMicrokernelBinder; b=HardwareVoltageMicrokernelBinder(); print('phi_v:', round(b.compute_dynamic_voltage_multiplier(),4))"),
    ("gpu", "from swarm_v2.core.gpu_infinity_computer import GPUInfinityComputer; g=GPUInfinityComputer(); print('gpu ok')"),
    ("monad", "from swarm_v2.core.monad import Monad; m=Monad(); print('monad ok')"),
]
for name, code in py_tests:
    out = run_py(code)
    ok = "ok" in out.lower() and "error" not in out.lower() and "traceback" not in out.lower()
    results.append({"type": "platform", "command": name, "status": "PASS" if ok else "FAIL", "output": out[:100]})
    print(f"[{'PASS' if ok else 'FAIL'}] platform: {name}: {out[:70].strip()}")

# ── 4. Multi-model assessment ──
print("\n=== MULTI-MODEL CAPABILITY ===")
mm_tests = [
    ("TRM 7M (embedded)", "from swarm_v2.core.trm_7m_model import get_trm_7m; m=get_trm_7m(); print('trm7m ok')"),
    ("Instella MoE router", "from swarm_v2.core.instella_single_expert_router import get_instella_router; r=get_instella_router(); print('instella ok')"),
    ("TurboVec quant", "from swarm_v2.core.turbovec_16x_femto_engine import TurboVec16xQuantizer; q=TurboVec16xQuantizer(); print('turbovec ok')"),
    ("K3 ternary", "from swarm_v2.core.ternary_kleene_engine import get_kleene_engine; e=get_kleene_engine(); print('k3 ok')"),
    ("584-twin", "from swarm_v2.core.digital_twin_singleton import get_digital_twin; t=get_digital_twin(); s=t.stats(); print('twin ok', s.get('total_agents'))"),
]
for name, code in mm_tests:
    out = run_py(code)
    ok = "ok" in out.lower() and "error" not in out.lower()
    results.append({"type": "multimodel", "command": name, "status": "PASS" if ok else "FAIL", "output": out[:100]})
    print(f"[{'PASS' if ok else 'FAIL'}] model: {name}: {out[:70].strip()}")

# ── Summary ──
total = len(results)
passed = sum(1 for r in results if r["status"] == "PASS")
print(f"\n=== SUMMARY: {passed}/{total} PASS ===")
with open(os.path.join(KERNEL_SRC, "..", "capability_test_report.json"), "w") as f:
    json.dump(results, f, indent=1)
print(f"report: {KERNEL_SRC}/../capability_test_report.json")
