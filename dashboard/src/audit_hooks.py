"""Check for React hooks violations in App.jsx"""
import re

path = r"F:\Development sites\TRM-Swarm-OS-v2\dashboard\src\App.jsx"
with open(path, 'r', errors='ignore') as f:
    content = f.read()

lines = content.split('\n')
hook_pattern = re.compile(r'(useState|useEffect|useRef|useCallback|useMemo|useLayoutEffect|useReducer|useContext)\(')

print("=== ALL HOOK CALLS ===")
issues = []

for i, line in enumerate(lines, 1):
    stripped = line.strip()
    if stripped.startswith('//') or stripped.startswith('import') or stripped.startswith('*'):
        continue
    for match in hook_pattern.finditer(stripped):
        hook = match.group(1)
        before = stripped[:match.start()]
        
        flags = []
        if '=>' in before or '.map(' in before or '.then(' in before:
            flags.append("CALLBACK")
        if 'function(' in before or 'function (' in before:
            flags.append("FUNC-DEF")
        
        if flags:
            flag_str = " | ".join(flags)
            print(f"  Line {i:5d}: {hook:20s} [{flag_str}]")
            print(f"           {stripped[:100]}")
            issues.append((i, hook, flags))

if not issues:
    print("  No obvious hooks violations found")

print(f"\nTotal hook violations detected: {len(issues)}")
