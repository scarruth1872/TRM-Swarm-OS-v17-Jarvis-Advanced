#!/usr/bin/env python3
"""Remove all {false ? null : ( blocks — Oxc rejects these."""

with open('dashboard/src/App.jsx', 'r') as f:
    content = f.read()

marker = '{false ? null : ('
count = 0

while marker in content:
    idx = content.find(marker)
    if idx == -1:
        break
    
    # Find matching closing )} by counting braces
    pos = idx + len(marker)
    brace_depth = 1
    in_string = False
    string_char = None
    
    while pos < len(content) and brace_depth > 0:
        ch = content[pos]
        
        if ch in "\"'`":
            if in_string and ch == string_char:
                in_string = False
            elif not in_string:
                in_string = True
                string_char = ch
        elif not in_string:
            if ch == '{':
                brace_depth += 1
            elif ch == '}':
                brace_depth -= 1
                if brace_depth == 0:
                    if pos + 1 < len(content) and content[pos + 1] == ')':
                        # Found )} — delete everything from idx to pos+2
                        # Clean up: remove preceding whitespace/newline too
                        start = idx
                        while start > 0 and content[start - 1] in ' \t':
                            start -= 1
                        if start > 0 and content[start - 1] == '\n':
                            start -= 1
                        
                        content = content[:start] + '\n' + content[pos + 2:]
                        count += 1
                        break
        pos += 1
    else:
        print(f'WARNING: Could not find matching }} for block at {idx}')
        break

with open('dashboard/src/App.jsx', 'w') as f:
    f.write(content)

# Verify
braces = sum(1 for ch in content if ch == '{') - sum(1 for ch in content if ch == '}')
parens = sum(1 for ch in content if ch == '(') - sum(1 for ch in content if ch == ')')
print(f'Removed {count} hidden blocks')
print(f'Balance: Braces={braces} Parens={parens}')
print(f'Remaining hidden: {"{false ? null : (" in content}')
