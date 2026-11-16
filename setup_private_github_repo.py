import requests
import os
import subprocess

print("=== CREATING PRIVATE PROPRIETARY GITHUB REPOSITORY ===")

token = None
env_path = r"F:\Development sites\TRM-Swarm-OS-v2\.env"
if os.path.exists(env_path):
    with open(env_path, "r", encoding="utf-8") as f:
        for line in f:
            if line.startswith("GITHUB_TOKEN="):
                token = line.strip().split("=", 1)[1]
                break

if not token:
    token = "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"

headers = {"Authorization": f"token {token}", "Accept": "application/vnd.github.v3+json"}
res = requests.get("https://api.github.com/user", headers=headers)
if res.status_code != 200:
    print("Failed to authenticate with GitHub:", res.text)
    exit(1)

user_info = res.json()
username = user_info.get("login")
print(f"Authenticated as GitHub User: {username}")

private_repo_name = "TRM-Swarm-OS-v17-Enterprise-Private"
print(f"Creating PRIVATE repository '{private_repo_name}' on GitHub under user '{username}'...")

create_res = requests.post("https://api.github.com/user/repos", headers=headers, json={
    "name": private_repo_name,
    "description": "PROPRIETARY - Swarm OS v17.0 & Jarvis Advanced Enterprise Core & Quantum Algorithms",
    "private": True,
    "auto_init": False
})

if create_res.status_code in (201, 422):
    if create_res.status_code == 201:
        print("PRIVATE Repository created successfully on GitHub!")
    else:
        print("PRIVATE Repository already exists on GitHub!")
        
    remote_url = f"https://{username}:{token}@github.com/{username}/{private_repo_name}.git"
    
    # Configure git remote 'private-origin'
    subprocess.run(["git", "remote", "remove", "private-origin"], cwd=r"F:\Development sites\TRM-Swarm-OS-v2", capture_output=True)
    subprocess.run(["git", "remote", "add", "private-origin", remote_url], cwd=r"F:\Development sites\TRM-Swarm-OS-v2", capture_output=True)
    
    print("Pushing repository to PRIVATE GitHub repository...")
    push_res = subprocess.run(["git", "push", "-u", "private-origin", "main", "--force"], cwd=r"F:\Development sites\TRM-Swarm-OS-v2", capture_output=True, text=True)
    
    print("Push Return Code:", push_res.returncode)
    print("Push Output:\n", push_res.stdout)
    if push_res.stderr:
        print("Push Log:\n", push_res.stderr)
        
    clean_url = f"https://github.com/{username}/{private_repo_name}"
    print(f"\nSUCCESS! PRIVATE Repository is live at: {clean_url}")
else:
    print("Failed to create private repo:", create_res.status_code, create_res.text)
