# cppc_probe.py — read the CPPC MSRs (indices 15-18) idle vs load; find the live voltage/frequency
import ctypes, ctypes.wintypes as wt, threading, time
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
    out = (ctypes.c_uint64 * 2)(0, 0)
    ret = wintypes.DWORD(0)
    ok = kernel32.DeviceIoControl(h, 0x81112EE0, idx_b, 16, out, 16, ctypes.byref(ret), None)
    return out[0] if ok else None

h, st = nt_open(r'\Device\AMDRyzenMasterDriverV20')
assert h
labels = {0:'C001_0062', 1:'PSTATE_STAT', 2:'PSTATE_CTRL', 3:'PSTATE_0',
          15:'CPPC_CAP', 16:'CPPC_EN', 17:'CPPC_STATUS', 18:'CPPC_00102B4', 13:'RAPL_UNIT'}
def snap(label):
    print(f'--- {label} ---')
    for i in sorted(labels):
        v = read_msr(h, i)
        print(f'  [{i:2d}] {labels[i]:12s} = {v:#018x}' if v is not None else f'  [{i:2d}] read failed')

snap('idle')
spin = threading.Event()
def busy():
    x = 0
    while not spin.is_set():
        x = (x + 1) & 0xFFFFFFFF
ts = [threading.Thread(target=busy) for _ in range(6)]
for t in ts: t.start()
time.sleep(1.0)
snap('LOAD (6 threads busy)')
spin.set()
for t in ts: t.join()
