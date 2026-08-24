import os
import shutil

artifacts_source_dir = r"C:\Users\shawn\.gemini\antigravity\brain\807168cd-2585-4a4e-b18f-383a6bdf794d"
target_dir = r"F:\Development sites\TRM-Swarm-OS-v2\swarm_v2_artifacts"

os.makedirs(target_dir, exist_ok=True)

docs = [
    "project_continuum_v53_spec_ledger.md",
    "self_hosting_setup_guide.md",
    "master_complex_task_report.md",
    "system_architecture_overview.md",
    "swarm_os_v17_pitch_deck.md",
    "microkernel_subagent_investigation.md",
    "jarvis_complex_task_report.md",
    "walkthrough.md"
]

print(f"Copying master documentation files to: {target_dir}")

for d in docs:
    src = os.path.join(artifacts_source_dir, d)
    dst = os.path.join(target_dir, d)
    if os.path.exists(src):
        shutil.copy2(src, dst)
        print(f"  [+] Copied: {d}")
    else:
        print(f"  [-] Not found in artifacts source: {d}")

print("\nDocumentation sync complete!")
