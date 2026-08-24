"""Carefully remove framer-motion from App.jsx without breaking JSX"""
import re

path = r"F:\Development sites\TRM-Swarm-OS-v2\dashboard\src\App.jsx"
with open(path, 'r', errors='ignore') as f:
    content = f.read()

# 1. Fix API_BASE to 8022
content = content.replace(
    "const API_BASE = 'http://127.0.0.1:8021';",
    "const API_BASE = 'http://127.0.0.1:8022';"
)

# 2. Fix hardcoded axios URLs to use API_BASE
content = content.replace(
    "axios.get('http://localhost:8021/artifacts')",
    "axios.get(`${API_BASE}/artifacts`)"
)
content = content.replace(
    "axios.get('http://localhost:8021/infrastructure/status')",
    "axios.get(`${API_BASE}/infrastructure/status`)"
)
content = content.replace(
    "axios.get('http://localhost:8021/testing/stats')",
    "axios.get(`${API_BASE}/testing/stats`)"
)
content = content.replace(
    "axios.get('http://localhost:8021/swarm/orchestrator/stats')",
    "axios.get(`${API_BASE}/swarm/orchestrator/stats`)"
)

# 3. Remove framer-motion import
content = content.replace(
    "import { motion, AnimatePresence } from 'framer-motion';",
    "// framer-motion removed"
)

# 4. Replace <motion.xxx> with <div> (or just remove "motion." prefix)
# This handles <motion.div>, <motion.span>, <motion.h1>, etc.
# But we need to be careful: motion.div should become div
content = re.sub(r'<motion\.(\w+)', r'<\1', content)
content = re.sub(r'</motion\.(\w+)', r'</\1', content)

# 5. Remove AnimatePresence tags
content = content.replace('<AnimatePresence>', '')
content = content.replace('</AnimatePresence>', '')
content = content.replace('<AnimatePresence mode="wait">', '<>')

# 6. Remove framer-motion specific props - these are JSX attributes
# We need to handle nested braces properly
# Patterns to remove: initial={...} animate={...} exit={...} transition={...} whileHover={...} whileTap={...} layout layoutId={...}
props_to_remove = [
    (r'\s+initial=\{', '}'),
    (r'\s+animate=\{', '}'),
    (r'\s+exit=\{', '}'),
    (r'\s+transition=\{', '}'),
    (r'\s+whileHover=\{', '}'),
    (r'\s+whileTap=\{', '}'),
    (r'\s+layoutId=\{', '}'),
]

# Remove each prop by matching the opening brace and finding the matching closing brace
def remove_prop(content, open_pattern):
    """Remove a JSX attribute with brace-delimited value"""
    result = []
    i = 0
    while i < len(content):
        # Look for the pattern
        m = re.search(open_pattern, content[i:])
        if not m:
            result.append(content[i:])
            break
        result.append(content[i:i + m.start()])
        i += m.start()
        # Skip the opening brace
        brace_start = i + len(m.group())
        if brace_start >= len(content) or content[brace_start] != '{':
            result.append(content[i:brace_start])
            i = brace_start
            continue
        # Find matching closing brace (handle nesting)
        depth = 1
        j = brace_start + 1
        while j < len(content) and depth > 0:
            if content[j] == '{':
                depth += 1
            elif content[j] == '}':
                depth -= 1
            j += 1
        # Skip the entire attribute value
        i = j
    return ''.join(result)

for pattern, _ in props_to_remove:
    content = remove_prop(content, pattern)

# Remove standalone layout prop
content = re.sub(r'\s+layout(?=[\s>/>])', '', content)

with open(path, 'w') as f:
    f.write(content)

# Verify no motion references remain
remaining = re.findall(r'\bmotion\.', content)
print(f"Remaining motion. references: {len(remaining)}")
remaining_ap = re.findall(r'AnimatePresence', content)
print(f"Remaining AnimatePresence: {len(remaining_ap)}")

# Verify the file is parseable
print(f"File size: {len(content)} chars")
print(f"Line count: {content.count(chr(10))}")
print("Done!")
