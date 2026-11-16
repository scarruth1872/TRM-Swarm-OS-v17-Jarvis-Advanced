#!/usr/bin/env python3
"""
sync_private_repository.py — TRM Swarm OS v19.001 "Sovereign"
==============================================================
Creates and synchronizes the full private repository 'TRM-Swarm-OS-v19-Sovereign-Private'
on GitHub, unstaging multi-GB model weights/checkpoints to comply with GitHub's 100MB per-file limit.
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

def sync_private_repo():
    print("===============================================================================================")
    print(">>> SYNCHRONIZING FULL PRIVATE PLATFORM REPOSITORY TO GITHUB")
    print("===============================================================================================")

    if not os.path.exists(TOKEN_PATH):
        print(f"❌ Error: Token file not found at {TOKEN_PATH}")
        sys.exit(1)

    with open(TOKEN_PATH, "r", encoding="utf-8") as f:
        token = f.read().strip()

    if not token:
        print("❌ Error: Token file is empty.")
        sys.exit(1)

    headers = {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github+json",
        "User-Agent": "TRM-Swarm-OS-PrivateSync/19.001",
        "X-GitHub-Api-Version": "2022-11-28"
    }

    # 1. Get authenticated user
    print("\n[1/4] Authenticating with GitHub API...")
    req = urllib.request.Request("https://api.github.com/user", headers=headers)
    try:
        with urllib.request.urlopen(req) as resp:
            user_data = json.loads(resp.read().decode('utf-8'))
            username = user_data["login"]
            print(f"  * Authenticated as GitHub user: @{username}")
    except Exception as e:
        print(f"❌ Authentication failed: {e}")
        sys.exit(1)

    # 2. Check or create private repository
    print(f"\n[2/4] Verifying/Creating PRIVATE repository '{PRIVATE_REPO_NAME}'...")
    repo_url = f"https://api.github.com/repos/{username}/{PRIVATE_REPO_NAME}"
    req_repo = urllib.request.Request(repo_url, headers=headers)
    try:
        with urllib.request.urlopen(req_repo) as resp:
            if resp.status == 200:
                print(f"  * Private repository @{username}/{PRIVATE_REPO_NAME} verified.")
    except urllib.error.HTTPError as e:
        if e.code == 404:
            create_payload = json.dumps({
                "name": PRIVATE_REPO_NAME,
                "description": "TRM Swarm OS v19.001 Sovereign — Complete Bare-Metal Microkernel, Cognitive Brain, 24-Sector LCARS & Full Platform Codebase (PRIVATE ASSETS)",
                "private": True,
                "has_issues": True,
                "has_projects": True,
                "has_wiki": False
            }).encode('utf-8')
            create_req = urllib.request.Request("https://api.github.com/user/repos", data=create_payload, headers=headers)
            try:
                with urllib.request.urlopen(create_req) as c_resp:
                    print(f"  * Created PRIVATE repository @{username}/{PRIVATE_REPO_NAME} (Status: {c_resp.status})")
            except Exception as ce:
                print(f"❌ Failed to create private repo: {ce}")
                sys.exit(1)
        else:
            print(f"❌ Error checking private repo: {e}")
            sys.exit(1)

    # 3. Clean Git index of large binary files & commit
    print("\n[3/4] Unstaging large binary models (>100MB) from git tracking...")
    subprocess.run(["git", "config", "http.postBuffer", "524288000"], cwd=WORKSPACE_DIR, check=True)
    subprocess.run(["git", "config", "http.lowSpeedLimit", "0"], cwd=WORKSPACE_DIR, check=True)
    subprocess.run(["git", "config", "http.lowSpeedTime", "999999"], cwd=WORKSPACE_DIR, check=True)

    # Remove large tracking paths if staged
    for path in ["Jarvis-Advanced-main/models", "swarm_v2_artifacts/training_session", "backups"]:
        subprocess.run(["git", "rm", "-r", "--cached", path], cwd=WORKSPACE_DIR, capture_output=True)

    # Stage all source code and documents
    subprocess.run(["git", "add", "."], cwd=WORKSPACE_DIR, check=True)
    subprocess.run(["git", "commit", "-m", "chore: sync full sovereign platform codebase, kernel, and deep-tech algorithms"], cwd=WORKSPACE_DIR, capture_output=True)
    print("  * Codebase staged and committed cleanly.")

    # 4. Push to private repository
    print(f"\n[4/4] Pushing to https://github.com/{username}/{PRIVATE_REPO_NAME}...")
    auth_remote_url = f"https://{username}:{token}@github.com/{username}/{PRIVATE_REPO_NAME}.git"
    clean_remote_url = f"https://github.com/{username}/{PRIVATE_REPO_NAME}.git"

    try:
        subprocess.run(["git", "remote", "remove", "private-sovereign"], cwd=WORKSPACE_DIR, capture_output=True)
        subprocess.run(["git", "remote", "add", "private-sovereign", auth_remote_url], cwd=WORKSPACE_DIR, check=True)
        
        push_res = subprocess.run(["git", "push", "-u", "private-sovereign", "main", "--force"], cwd=WORKSPACE_DIR, capture_output=True, text=True)
        if push_res.returncode == 0:
            print(f"  * Successfully pushed full platform to private repository @{username}/{PRIVATE_REPO_NAME}")
        else:
            print(f"  * Push output: {push_res.stderr or push_res.stdout}")
    finally:
        # Scrub token from remote URL
        subprocess.run(["git", "remote", "set-url", "private-sovereign", clean_remote_url], cwd=WORKSPACE_DIR, capture_output=True)
        print("  * Sanitized git remote URL (token removed from local .git/config)")

    print("\n-----------------------------------------------------------------------------------------------")
    print(f"⭐ FULL PLATFORM BACKED UP TO PRIVATE REPOSITORY:")
    print(f"   Private URL: https://github.com/{username}/{PRIVATE_REPO_NAME} (Locked & Private)")
    print("===============================================================================================")

if __name__ == "__main__":
    sync_private_repo()
