"""Replace all framer-motion imports and usage with plain React equivalents"""
import re

path = r"F:\Development sites\TRM-Swarm-OS-v2\dashboard\src\App.jsx"
with open(path, 'r', errors='ignore') as f:
    content = f.read()

# 1. Remove framer-motion import
content = content.replace(
    "import { motion, AnimatePresence } from 'framer-motion';",
    "// framer-motion removed"
)

# 2. Replace <motion.XXX ...> with <div ...>
# Match <motion.XXX ... props ...> and replace with <div ... props ...>
content = re.sub(r'<motion\.(\w+)([^>]*)>', r'<\1\2>', content)
content = re.sub(r'</motion\.(\w+)>', r'</\1>', content)

# 3. Replace <AnimatePresence> wrappers - just remove the tags
content = content.replace('<AnimatePresence>', '')
content = content.replace('</AnimatePresence>', '')

# 4. Replace motion.div, motion.span, etc. in className/other attributes
content = content.replace('as motion.', 'as ')

# 5. Remove any framer-motion specific props
content = re.sub(r'\s+initial=\{.*?\}', '', content)
content = re.sub(r'\s+animate=\{.*?\}', '', content)
content = re.sub(r'\s+exit=\{.*?\}', '', content)
content = re.sub(r'\s+transition=\{.*?\}', '', content)
content = re.sub(r'\s+whileHover=\{.*?\}', '', content)
content = re.sub(r'\s+whileTap=\{.*?\}', '', content)
content = re.sub(r'\s+layout\b', '', content)
content = re.sub(r'\s+layoutId=\{.*?\}', '', content)

with open(path, 'w') as f:
    f.write(content)

print("framer-motion removed from App.jsx")
# Count motion references
print(f"Remaining motion references: {content.count('motion.')}")
print(f"Remaining AnimatePresence: {content.count('AnimatePresence')}")
