"""Audit each component for real API wiring vs mock data."""
import os, re

root = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\src\components"

for fname in sorted(os.listdir(root)):
    fpath = os.path.join(root, fname)
    if not os.path.isfile(fpath) or not fname.endswith('.tsx'):
        continue
    
    with open(fpath, 'r', errors='ignore') as f:
        content = f.read()
    
    api_calls = len(re.findall(r'fetch\(|axios\.|\.post\(|\.get\(|\.put\(|\.delete\(', content))
    urls = re.findall(r'https?://[^,;)\s"\'`]+', content)
    mock_refs = len(re.findall(r'mock|placeholder|sample_data|dummy_data|faker', content, re.IGNORECASE))
    backend_routes = list(set(re.findall(r'(?:/api/[a-z0-9/_-]+|/config/[a-z0-9/_-]+)', content)))
    import_firebase = 'firebase' in content.lower()
    import_trm = '8021' in content or 'trm' in content.lower()
    
    verdict = "✅ REAL" if api_calls > 3 and mock_refs < 3 else \
              "⚠️ MIXED" if api_calls > 0 else \
              "❓ SUSPECT" if mock_refs > 5 else "❓ UNKNOWN"
    
    print(f"\n{'='*60}")
    print(f"  {fname} ({len(content):,} bytes)")
    print(f"{'='*60}")
    print(f"  API calls: {api_calls} | Mock refs: {mock_refs} | Firebase: {import_firebase} | TRM: {import_trm}")
    print(f"  VERDICT: {verdict}")
    if urls:
        print(f"  External URLs ({len(list(set(urls)))}):")
        for u in sorted(set(urls))[:5]:
            print(f"    {u[:100]}")
    if backend_routes:
        print(f"  Backend routes: {backend_routes[:5]}")

print("\n\n=== SUMMARY ===")
print(f"{'Component':40s} {'Size':>10s} {'API':>5s} {'Mock':>5s} {'Verdict':>10s}")
print("-"*75)
for fname in sorted(os.listdir(root)):
    fpath = os.path.join(root, fname)
    if not os.path.isfile(fpath) or not fname.endswith('.tsx'):
        continue
    with open(fpath, 'r', errors='ignore') as f:
        content = f.read()
    api = len(re.findall(r'fetch\(|axios\.|\.post\(|\.get\(|\.put\(|\.delete\(', content))
    mock = len(re.findall(r'mock|placeholder|sample_data|dummy_data|faker', content, re.IGNORECASE))
    v = "REAL" if api > 3 and mock < 3 else "MIXED" if api > 0 else "SUSPECT" if mock > 5 else "UNKNOWN"
    print(f"{fname:40s} {len(content):>10,} {api:>5d} {mock:>5d} {v:>10s}")
