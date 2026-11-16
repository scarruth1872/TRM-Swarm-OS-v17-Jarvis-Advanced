# extract_methods.py — pull the 8 IDeviceManager virtual methods out of the disassembly
import re

lines = open('device_disasm.txt', encoding='utf-8', errors='replace').read().splitlines()
# parse: address + bytes + mnemonic
methods = [0xa54c0, 0xa55f0, 0xa5770, 0xa5af0, 0xa5b80, 0xa77f0, 0xa7830, 0xa79a0]
# map from the base image 0x180000000
base = 0x180000000

idx = {}
for i, ln in enumerate(lines):
    m = re.match(r'\s*([0-9A-F]{16}):', ln)
    if m:
        idx[int(m.group(1), 16)] = i

for slot, rva in enumerate(methods):
    addr = base + rva
    start = idx.get(addr)
    if start is None:
        print(f'[method {slot}] RVA {rva:#x}: NOT FOUND'); continue
    print(f'\n===== method [{slot}] RVA {rva:#x} =====')
    # print until ret (or ~60 lines)
    count = 0
    for ln in lines[start:start+80]:
        if count > 60: break
        # stop after the first 'ret' following the first instruction
        print(ln)
        count += 1
        if count > 1 and (' ret' in ln or 'ret ' in ln.split('  ')[-1] if False else False):
            pass
