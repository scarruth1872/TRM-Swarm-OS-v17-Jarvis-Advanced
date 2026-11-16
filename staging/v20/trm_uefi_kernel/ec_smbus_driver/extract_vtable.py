# extract_vtable.py — pull the IDeviceManager vtable (function pointers) from Device.dll
import struct

data = open(r'C:\Program Files\Alienware\Alienware Command Center\OCControlService\Device.dll', 'rb').read()
e_lfanew = struct.unpack_from('<I', data, 0x3C)[0]
num_sections = struct.unpack_from('<H', data, e_lfanew + 6)[0]
opt_hdr_size = struct.unpack_from('<H', data, e_lfanew + 20)[0]
opt_hdr = e_lfanew + 24
magic = struct.unpack_from('<H', data, opt_hdr)[0]
image_base = struct.unpack_from('<Q', data, opt_hdr + 24)[0] if magic == 0x20b else struct.unpack_from('<I', data, opt_hdr + 28)[0]
sections = opt_hdr + opt_hdr_size
secs = {}
for i in range(num_sections):
    off = sections + i * 40
    name = data[off:off+8].rstrip(b'\x00').decode()
    vsize = struct.unpack_from('<I', data, off + 8)[0]
    vaddr = struct.unpack_from('<I', data, off + 12)[0]
    rawsize = struct.unpack_from('<I', data, off + 16)[0]
    rawptr = struct.unpack_from('<I', data, off + 20)[0]
    secs[name] = (vaddr, max(vsize, rawsize), rawptr)

def rva_to_off(rva):
    for vaddr, size, rawptr in secs.values():
        if vaddr <= rva < vaddr + size:
            return rawptr + (rva - vaddr)
    return None

print(f'image_base: {image_base:#x}, sections: {list(secs)}')
vt_rva = 0xD2040
vt_off = rva_to_off(vt_rva)
print(f'vtable RVA {vt_rva:#x} -> file offset {vt_off:#x}')

ptrs = []
for i in range(80):
    p = struct.unpack_from('<Q', data, vt_off + i * 8)[0]
    if not (image_base + 0x1000 <= p < image_base + 0x200000):
        break
    ptrs.append(p)
print(f'IDeviceManager vtable: {len(ptrs)} virtual methods:')
for i, p in enumerate(ptrs):
    print(f'  [{i:2d}] RVA {p - image_base:#x}')
