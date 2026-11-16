"""Find all useEffect/useState/useMemo calls with non-function first args"""
import re

path = r"F:\Development sites\TRM-Swarm-OS-v2\dashboard\src\App.jsx"
with open(path, 'r', errors='ignore') as f:
    lines = f.readlines()

hook_calls = re.compile(r'(useEffect|useState|useCallback|useMemo|useLayoutEffect)\s*\(')

for i, line in enumerate(lines, 1):
    stripped = line.strip()
    if stripped.startswith('//'):
        continue
    for match in hook_calls.finditer(stripped):
        hook = match.group(1)
        after_paren = stripped[match.end():]  # content after the (
        
        # Get the first argument
        depth = 1
        arg_start = 0
        for j, ch in enumerate(after_paren):
            if ch == '(':
                depth += 1
            elif ch == ')':
                depth -= 1
                if depth == 0:
                    first_arg = after_paren[:j].strip()
                    break
        else:
            continue
        
        # For useState: check if the initial value looks like a function call result
        if hook == 'useState':
            if first_arg == '':
                print(f"Line {i}: useState() with NO argument")
            elif first_arg.startswith('[') or first_arg.startswith('{') or first_arg.startswith('"') or first_arg.startswith("'"):
                print(f"Line {i}: useState({first_arg[:40]}...) — static value (OK)")
            elif first_arg.startswith('(') or first_arg.startswith('function') or first_arg == '()' or first_arg == 'null':
                pass  # lazy initializer or null
            else:
                # Could be a variable or expression
                pass
                
        # For useEffect: check first arg is a function
        if hook == 'useEffect':
            if not first_arg.startswith('(') and not first_arg.startswith('async') and not first_arg.startswith('function'):
                if first_arg.startswith('[') or first_arg.startswith('null'):
                    print(f"Line {i}: useEffect with non-function first arg: {first_arg[:60]}")
