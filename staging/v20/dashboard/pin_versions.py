from pathlib import Path as _Path
# Repo root derived from this file's location, not pinned to one host's
# drive letter (swarm_v2/core/derive_repo_root.py).
_TRM_ROOT = str(_Path(__file__).resolve().parents[1])
import json

path = str(_Path(_TRM_ROOT) / "dashboard" / "package.json")
with open(path, 'r') as f:
    pkg = json.load(f)

pkg['dependencies']['react'] = '19.2.0'
pkg['dependencies']['react-dom'] = '19.2.0'
pkg['dependencies']['framer-motion'] = '12.34.1'
pkg['dependencies']['lucide-react'] = '0.574.0'
pkg['dependencies']['axios'] = '1.13.5'

pkg['devDependencies']['@types/react'] = '19.2.7'
pkg['devDependencies']['@types/react-dom'] = '19.2.3'
pkg['devDependencies']['@vitejs/plugin-react'] = '5.1.1'
pkg['devDependencies']['vite'] = '8.0.0-beta.13'

with open(path, 'w') as f:
    json.dump(pkg, f, indent=2)

print("Pinned exact versions")
