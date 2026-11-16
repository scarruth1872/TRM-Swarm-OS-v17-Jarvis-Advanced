p = r"F:\Development sites\TRM-Swarm-OS-v2\ARCHITECTURE.md"
lines = open(p, encoding="utf-8").read().split("\n")
out = []
for ln in lines:
    if "Dynamic Voltage Microkernel Binding" in ln and "\u03a6_V" in ln:
        width = len(ln)
        new_text = "  - Dynamic Voltage Microkernel Binding (\u03a6_V measured, not constant)"
        pad = width - 1 - len(new_text)
        ln = "\u2502" + new_text + (" " * pad) + "\u2502"
        print("width before:", width, "after:", len(ln))
        print("NEW:", ln)
    out.append(ln)
open(p, "w", encoding="utf-8").write("\n".join(out))
