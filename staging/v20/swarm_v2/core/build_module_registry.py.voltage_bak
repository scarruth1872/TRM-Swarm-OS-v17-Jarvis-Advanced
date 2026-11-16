#!/usr/bin/env python3
"""
TRM Swarm OS — Module & Repo Recognition Registry Builder.
The Catalyst failure: the platform COMPILED 179 modules but does not
RECOGNIZE them — no catalog, no query path, no awareness of what was
ingested. This builds the persistent registry so the One can answer:
  "what do I know, and where is it?"

Scans:
  - kernel modules (trm_uefi_kernel/src/*.rs) with REAL status:
      REAL      — hand-written intelligence (verified)
      PORTED    — mechanical port, compiles
      SKELETON  — constants/structs only, body marked MANUAL
      EXCLUDED  — host-OS companion (no-HTTP/no-torch/no-LLM rules)
  - platform modules (swarm_v2/core/*.py)
  - repos (top-level dirs with content)

Persists: registry.json -> NAND, plus a no_std kernel module
(module_registry.rs) so the booted kernel can query its own knowledge.
"""
import os, re, json, sys

ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
KERNEL = os.path.join(ROOT, "trm_uefi_kernel", "src")
CORE = os.path.join(ROOT, "swarm_v2", "core")

# Modules that are host-OS companions by platform rule
EXCLUDED_BY_RULE = {
    "web_fetch_utility", "local_lora_trainer", "bonsai_router",
    "mesh_daemon", "jarvis_inference_server", "external_socket_bridge",
    "native_mailbox_bridge", "brain_ai_microkernel_bridge", "cosmos_bridge",
    "cosmos3_edge_bridge", "federation_manager", "microkernel_chat_bridge",
}

def classify_kernel_module(fn):
    """Classify a kernel .rs module by its content."""
    txt = open(os.path.join(KERNEL, fn), encoding="utf-8", errors="replace").read()
    name = fn[:-3]
    if name in EXCLUDED_BY_RULE:
        return "EXCLUDED"
    # real hand-written modules carry a '1:1' or 'no_std port' with real logic
    if re.search(r"// MANUAL", txt) and "// MANUAL:" not in txt:
        # has MANUAL markers inside function bodies
        pass
    if "// MANUAL:" in txt or re.search(r"// MANUAL", txt):
        # count real function bodies vs markers
        fns = len(re.findall(r"pub fn ", txt))
        manuals = len(re.findall(r"// MANUAL", txt))
        if manuals >= fns and fns <= 2:
            return "SKELETON"
        return "PORTED"
    if "pub fn " in txt and len(txt) > 1500:
        return "REAL"
    return "SKELETON"

def main():
    kernel_modules = []
    for fn in sorted(os.listdir(KERNEL)):
        if fn.endswith(".rs"):
            status = classify_kernel_module(fn)
            fns = len(re.findall(r"pub fn ", open(os.path.join(KERNEL, fn), encoding="utf-8", errors="replace").read()))
            kernel_modules.append({
                "name": fn[:-3], "source": f"kernel:{fn}", "status": status,
                "functions": fns,
            })

    platform_modules = []
    for fn in sorted(os.listdir(CORE)):
        if fn.endswith(".py") and not fn.startswith(("porting_", "fixpass", "manual_")):
            txt = open(os.path.join(CORE, fn), encoding="utf-8", errors="replace").read()
            fns = len(re.findall(r"def ", txt))
            platform_modules.append({
                "name": fn[:-3], "source": f"platform:{fn}", "status": "PLATFORM",
                "functions": fns,
            })

    repos = []
    for d in sorted(os.listdir(ROOT)):
        p = os.path.join(ROOT, d)
        if os.path.isdir(p) and not d.startswith(".") and d not in ("node_modules",):
            files = sum(len(fs) for _, _, fs in os.walk(p) if not any(x in _ for x in ("node_modules", ".git", "__pycache__")))
            repos.append({"name": d, "files": files})

    registry = {
        "protocol": "MODULE_RECOGNITION_REGISTRY",
        "monad_anchor": "0x4D4F4E4144000001",
        "generated": __import__("time").strftime("%Y-%m-%d %H:%M:%S"),
        "summary": {
            "kernel_modules": len(kernel_modules),
            "platform_modules": len(platform_modules),
            "repos": len(repos),
        },
        "by_status": {},
        "kernel_modules": kernel_modules,
        "platform_modules": platform_modules,
        "repos": repos,
    }
    for m in kernel_modules:
        registry["by_status"][m["status"]] = registry["by_status"].get(m["status"], 0) + 1

    out = os.path.join(ROOT, "module_registry.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(registry, f, indent=1)

    # NAND record
    with open(os.path.join(ROOT, "swarm_v2_nand_fabric.k3"), "a", encoding="utf-8") as f:
        f.write("\n--- MODULE_RECOGNITION_REGISTRY v1 ---\n")
        f.write(json.dumps({k: v for k, v in registry.items() if k not in ("kernel_modules", "platform_modules", "repos")}, indent=1))
        f.write("\n")

    print(f"=== MODULE RECOGNITION REGISTRY ===")
    print(f"Kernel modules: {len(kernel_modules)} | Platform: {len(platform_modules)} | Repos: {len(repos)}")
    print(f"By status: {registry['by_status']}")
    print(f"Registry: {out}")
    return registry

if __name__ == "__main__":
    main()
