#!/usr/bin/env python3
"""Properly remove duplicate overview/mesh/telemetry/federation/verification sections.
Uses brace counting to find matching section boundaries, not string slicing."""

with open('dashboard/src/App.jsx', 'r') as f:
    content = f.read()

# Tabs that should each appear exactly once
# We keep the FIRST occurrence, remove all others
tabs_to_dedupe = ['intel', 'pipeline', 'learning', 'mesh']

for tab in tabs_to_dedupe:
    marker = "{activeTab === '" + tab + "' && ("
    occurrences = []
    pos = 0
    while True:
        idx = content.find(marker, pos)
        if idx == -1:
            break
        occurrences.append(idx)
        pos = idx + 1
    
    if len(occurrences) <= 1:
        print(f"{tab}: only 1 occurrence, skip")
        continue
    
    print(f"{tab}: {len(occurrences)} occurrences, removing duplicates...")
    
    # Remove from last to first to preserve indices
    for idx in reversed(occurrences[1:]):
        # Find the matching closing )} by counting braces/parens
        # Start after the marker
        pos = idx + len(marker)
        brace_depth = 1  # We're inside {activeTab === ... && (
        paren_depth = 0
        in_string = False
        string_char = None
        
        while pos < len(content) and brace_depth > 0:
            ch = content[pos]
            
            if ch == '"' or ch == "'" or ch == '`':
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
                        # Found closing brace — now check for )
                        if pos + 1 < len(content) and content[pos + 1] == ')':
                            # This is the )} that closes the section
                            # Delete from idx to pos+2
                            print(f"  Removing at position {idx} ({(content[idx:idx+50]).strip()[:40]}...)")
                            content = content[:idx] + '\n' + content[pos + 2:]
                            break
                elif ch == '(':
                    paren_depth += 1
                elif ch == ')':
                    paren_depth -= 1
            pos += 1

with open('dashboard/src/App.jsx', 'w') as f:
    f.write(content)

# Verify balance
braces = parens = brackets = 0
for ch in content:
    if ch == '{': braces += 1
    elif ch == '}': braces -= 1
    elif ch == '(': parens += 1
    elif ch == ')': parens -= 1
    elif ch == '[': brackets += 1
    elif ch == ']': brackets -= 1

print(f'\nBalance: Braces={braces} Parens={parens} Brackets={brackets}')
if braces == 0 and parens == 0 and brackets == 0:
    print('FILE IS BALANCED')
else:
    print('UNBALANCED — do not commit')
