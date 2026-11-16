"""Deep audit of WorkspaceNexus.tsx and remaining components."""
import re, os

root = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main"

# WorkspaceNexus
with open(os.path.join(root, "src/components/WorkspaceNexus.tsx"), "r", errors="ignore") as f:
    content = f.read()

print("=== WORKSPACE NEXUS (161KB) ===")
print(f"Size: {len(content):,} bytes")
print()

# Find all Google API URLs
urls = re.findall(r'https://[a-z.]+googleapis\.com[^,;)\s]+', content)
print(f"Google API calls: {len(urls)}")
for u in set(urls):
    print(f"  {u}")
print()

# Find UI tab/section titles
sections = re.findall(r'<h[1-3][^>]*>([^<]+)', content)
print(f"UI Sections: {len(sections)}")
for s in sections:
    print(f"  {s.strip()}")
print()

# Find button/label texts
buttons = re.findall(r'<button[^>]*>([^<]+)', content)
print(f"Buttons: {len(buttons)}")
for b in set(buttons):
    print(f"  {b.strip()}")
print()

# Check for OAuth/client credentials
oauth_patterns = re.findall(r'["\']([^"\']*(?:oauth|client_id|client_secret|api_key|token|scope|auth)[^"\']*)["\']', content, re.IGNORECASE)
print("OAuth/Token references:")
for o in set(oauth_patterns):
    print(f"  {o}")

print("\n" + "="*60)

# Check other large components
large_files = [
    "BiomimeticLab.tsx",
    "TernarySynapseEngine.tsx",
    "LocalModelSync.tsx",
    "RoboticsLab.tsx",
    "WiFiVisionIntelligence.tsx",
    "ResonanceResearchHub.tsx",
    "NoosphericRecalibration.tsx",
    "CognitiveMemoryGraph.tsx",
]
print("\n=== OTHER COMPONENT SIZES ===")
for fname in large_files:
    path = os.path.join(root, "src/components", fname)
    if os.path.exists(path):
        size = os.path.getsize(path)
        print(f"  {fname}: {size:,} bytes")
