import struct, re
data = open(r'C:\WINDOWS\system32\AMDRyzenMasterDriver.sys', 'rb').read()
msrs = {0xC0010063: 'PSTATE_STAT', 0xC0010064: 'PSTATE_0', 0xC0010065: 'PSTATE_1',
        0xC0010299: 'PWR_UNIT', 0xC001029A: 'CORE_ENERGY', 0xC001029B: 'PKG_ENERGY',
        0xC0010293: 'RAPL_PWR_UNIT', 0xC0010058: 'C001_0058', 0xC0010292: 'C001_0292',
        0xC0010061: 'PSTATE_CTRL'}
print('=== MSR values present in the driver binary ===')
for val, name in msrs.items():
    pat = struct.pack('<I', val)
    n = data.count(pat)
    if n:
        print(f'  MSR {val:#x} ({name}): {n}x')

print('=== device/symbolic-link strings (UTF-16) ===')
ws = data.decode('utf-16le', 'ignore')
for m in set(re.findall(r'[ -~]{4,40}', ws)):
    if any(k in m for k in ('Device', 'Master', 'Ryzen', 'RMD', 'SMU', 'VID')):
        print('  ', repr(m))
