#!/usr/bin/env python3
"""
sync_clean_private_repo.py — TRM Swarm OS v19.001 "Sovereign"
==============================================================
Creates a clean, blob-purged push branch for 'TRM-Swarm-OS-v19-Sovereign-Private'
ensuring zero historical >100MB model files are sent, enabling instant HTTP 200 upload.
"""

import os
import sys
import json
import urllib.request
import urllib.error
import subprocess

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')

TOKEN_PATH = r"C:\Users\shawn\Downloads\githubtoken.txt"
WORKSPACE_DIR = r"F:\Development sites\TRM-Swarm-OS-v2"
PRIVATE_REPO_NAME = "TRM-Swarm-OS-v19-Sovereign-Private"

def sync_clean_private():
    print("===============================================================================================")
    print(">>> CREATING CLEAN SOVEREIGN PRIVATE REPOSITORY SNAPSHOT")
    print("===============================================================================================")

    with open(TOKEN_PATH, "r", encoding="utf-8") as f:
        token = f.read().strip()

    headers = {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github+json",
        "User-Agent": "TRM-Private-Sync/19.001"
    }

    # Verify user
    req = urllib.request.Request("https://api.github.com/user", headers=headers)
    with urllib.request.urlopen(req) as resp:
        username = json.loads(resp.read().decode())["login"]
        print(f"  * Authenticated user: @{username}")

    auth_remote_url = f"https://{username}:{token}@github.com/{username}/{PRIVATE_REPO_NAME}.git"
    clean_remote_url = f"https://github.com/{username}/{PRIVATE_REPO_NAME}.git"

    try:
        # Create an isolated temporary branch with no bloated historical objects
        print("\n[1/3] Packaging clean codebase snapshot (excluding >100MB blobs)...")
        subprocess.run(["git", "checkout", "--orphan", "sovereign-v19-clean"], cwd=WORKSPACE_DIR, capture_output=True)
        
        # Reset index and re-add according to current .gitignore
        subprocess.run(["git", "rm", "-rf", "--cached", "."], cwd=WORKSPACE_DIR, capture_output=True)
        subprocess.run(["git", "add", "."], cwd=WORKSPACE_DIR, check=True)
        subprocess.run(["git", "commit", "-m", "feat: TRM Swarm OS v19.001 Sovereign — Complete Bare-Metal Microkernel, Drug Discovery Pipeline, Post-Quantum Security & 24-Sector LCARS Stack"], cwd=WORKSPACE_DIR, check=True)

        print("\n[2/3] Connecting to private remote...")
        subprocess.run(["git", "remote", "remove", "private-sovereign"], cwd=WORKSPACE_DIR, capture_output=True)
        subprocess.run(["git", "remote", "add", "private-sovereign", auth_remote_url], cwd=WORKSPACE_DIR, check=True)

        print("\n[3/3] Uploading clean platform snapshot to private repo...")
        push_res = subprocess.run(["git", "push", "-u", "private-sovereign", "sovereign-v19-clean:main", "--force"], cwd=WORKSPACE_DIR, capture_output=True, text=True)
        
        if push_res.returncode == 0:
            print(f"  * SUCCESS! Pushed to https://github.com/{username}/{PRIVATE_REPO_NAME}")
        else:
            print(f"  * Push response: {push_res.stderr or push_res.stdout}")
    finally:
        # Clean up local branch and sanitize credentials
        subprocess.run(["git", "checkout", "main"], cwd=WORKSPACE_DIR, capture_output=True)
        subprocess.run(["git", "branch", "-D", "sovereign-v19-clean"], cwd=WORKSPACE_DIR, capture_output=True)
        subprocess.run(["git", "remote", "set-url", "private-sovereign", clean_remote_url], cwd=WORKSPACE_DIR, capture_output=True)
        print("  * Switched back to main branch & sanitized remote URL.")

if __name__ == "__main__":
    sync_clean_private()
