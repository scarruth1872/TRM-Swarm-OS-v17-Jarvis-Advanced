# decode_vid.py — read MSRs with the CORRECT output layout (low32@+4, high32@+8) and decode VID
import ctypes, ctypes.wintypes as wt
from ctypes import wintypes

ntdll = ctypes.WinDLL('ntdll', use_last_error=True)
kernel32 = ctypes.WinDLL('kernel32', use_last_error=True)

class UNICODE_STRING(ctypes.Structure):
    _fields_ = [('Length', wintypes.USHORT), ('MaximumLength', wintypes.USHORT), ('Buffer', ctypes.c_void_p)]
class OBJECT_ATTRIBUTES(ctypes.Structure):
    _fields_ = [('Length', wintypes.ULONG), ('RootDirectory', ctypes.c_void_p),
                ('ObjectName', ctypes.POINTER(UNICODE_STRING)), ('Attributes', wintypes.ULONG),
                ('SecurityDescriptor', ctypes.c_void_p), ('SecurityQualityOfService', ctypes.c_void_p)]
class IO_STATUS_BLOCK(ctypes.Structure):
    _fields_ = [('Status', ctypes.c_long), ('Information', ctypes.c_void_p)]

ntdll.NtCreateFile.restype = ctypes.c_long
ntdll.NtCreateFile.argtypes = [ctypes.POINTER(ctypes.c_void_p), wintypes.ULONG, ctypes.POINTER(OBJECT_ATTRIBUTES),
                               ctypes.POINTER(IO_STATUS_BLOCK), ctypes.c_void_p, wintypes.ULONG, wintypes.ULONG,
                               wintypes.ULONG, wintypes.ULONG, ctypes.c_void_p, wintypes.ULONG]
kernel32.DeviceIoControl.restype = wintypes.BOOL
kernel32.DeviceIoControl.argtypes = [ctypes.c_void_p, wintypes.DWORD, ctypes.c_void_p, wintypes.DWORD,
                                     ctypes.c_void_p, wintypes.DWORD, ctypes.POINTER(wintypes.DWORD), ctypes.c_void_p]

def nt_open(path):
    buf = ctypes.create_unicode_buffer(path)
    n = UNICODE_STRING(len(path)*2, (len(path)+1)*2, ctypes.cast(buf, ctypes.c_void_p))
    oa = OBJECT_ATTRIBUTES(); oa.Length = ctypes.sizeof(OBJECT_ATTRIBUTES); oa.ObjectName = ctypes.pointer(n); oa.Attributes = 0x40
    h = ctypes.c_void_p(); iosb = IO_STATUS_BLOCK()
    st = ntdll.NtCreateFile(ctypes.byref(h), 0x12019F, ctypes.byref(oa), ctypes.byref(iosb), None, 0, 0x3, 0x1, 0, None, 0)
    return h, st

def read_msr(h, idx):
    idx_b = (ctypes.c_uint32 * 4)(idx, 0, 0, 0)
    raw = (ctypes.c_ubyte * 16)()
    ret = wintypes.DWORD(0)
    ok = kernel32.DeviceIoControl(h, 0x81112EE0, idx_b, 16, raw, 16, ctypes.byref(ret), None)
    if not ok:
        return None
    # handler stores: low32 @ offset 4, high32 @ offset 8
    lo = int.from_bytes(bytes(raw[4:8]), 'little')
    hi = int.from_bytes(bytes(raw[8:12]), 'little')
    return (hi << 32) | lo

h, st = nt_open(r'\Device\AMDRyzenMasterDriverV20')
assert h
for idx, label in ((1, 'PSTATE_STAT'), (3, 'PSTATE_0'), (4, 'PSTATE_1'), (17, 'CPPC_STATUS'), (13, 'RAPL')):
    v = read_msr(h, idx)
    if v is None:
        print(f'[{idx:2d}] {label}: read failed'); continue
    # Family 17h/19h P-state layout: CpuFid bits8-13, CpuVid bits16-22, CpuDid bits30-32, PstateEn bit63
    fid = (v >> 8) & 0x3F
    vid = (v >> 16) & 0x7F
    did = (v >> 30) & 0x7
    en  = (v >> 63) & 1
    # VID -> volts (Zen 2/3): V = 1.55 - VID*0.00625 ; alt V = VID*0.00625+0.155
    v_lo = 1.55 - vid * 0.00625
    v_hi = 0.155 + vid * 0.00625
    print(f'[{idx:2d}] {label:12s} = {v:#018x}  fid={fid} vid={vid} did={did} en={en}  '
          f'V≈{v_lo:.3f} (or {v_hi:.3f})')
