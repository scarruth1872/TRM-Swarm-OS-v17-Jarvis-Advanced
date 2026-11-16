import struct
data = open(r'C:\WINDOWS\system32\AMDRyzenMasterDriver.sys', 'rb').read()
e_lfanew = struct.unpack_from('<I', data, 0x3C)[0]
num_sec = struct.unpack_from('<H', data, e_lfanew + 6)[0]
opt = e_lfanew + 24
sec_off = opt + struct.unpack_from('<H', data, e_lfanew + 20)[0]
secs = {}
for i in range(num_sec):
    o = sec_off + i * 40
    name = data[o:o+8].rstrip(b'\x00').decode()
    vsize = struct.unpack_from('<I', data, o + 8)[0]
    vaddr = struct.unpack_from('<I', data, o + 12)[0]
    rawsize = struct.unpack_from('<I', data, o + 16)[0]
    rawptr = struct.unpack_from('<I', data, o + 20)[0]
    secs[name] = (vaddr, max(vsize, rawsize), rawptr)
def rva_to_off(rva):
    for vaddr, size, rawptr in secs.values():
        if vaddr <= rva < vaddr + size:
            return rawptr + (rva - vaddr)
    return None
# import directory (data directory index 1) — PE32+ data dirs start at opt+112
imp_rva, imp_size = struct.unpack_from('<II', data, opt + 112 + 8)[0], struct.unpack_from('<II', data, opt + 112 + 8 + 8)[0]
off = rva_to_off(imp_rva)
print('=== imports (IAT name -> RVA of IAT slot) ===')
iat = {}
i = 0
while True:
    oft = struct.unpack_from('<I', data, off + i * 20)[0]  # OriginalFirstThunk
    if oft == 0:
        break
    name_off = rva_to_off(struct.unpack_from('<I', data, off + i * 20 + 12)[0])
    dll = data[name_off:data.index(b'\x00', name_off)].decode()
    first_thunk = struct.unpack_from('<I', data, off + i * 20 + 16)[0]
    to = rva_to_off(oft)
    slot = first_thunk
    j = 0
    while True:
        hint = struct.unpack_from('<Q', data, to + j * 8)[0]
        if hint == 0:
            break
        if hint & 0x8000000000000000:
            ord_ = hint & 0xFFFF
            fn = f'#{ord_}'
        else:
            fn_off = rva_to_off((hint & 0x7FFFFFFF) + 2)
            fn = data[fn_off:data.index(b'\x00', fn_off)].decode()
        iat[(dll, fn)] = slot
        slot += 8
        j += 1
    i += 1
for k, slot in iat.items():
    if any(x in k[1] for x in ('SymbolicLink', 'CreateDevice', 'OpenFile', 'CreateFile', 'DeviceIoControl', 'LoadDriver')):
        print(f'  {k[0]}!{k[1]}  -> IAT RVA {slot:#x} (VA {0x140000000+slot:#x})')
