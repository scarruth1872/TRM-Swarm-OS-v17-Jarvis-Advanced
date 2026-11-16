# open_and_read_vid.py — open \Device\AMDRyzenMasterDriverV20 via NtCreateFile, read PSTATE_STAT (VID)
import ctypes
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

def nt_open(path, access=0x12019F):  # GENERIC_READ|GENERIC_WRITE = 0x12019F for files; use FILE_READ|WRITE attrs
    buf = ctypes.create_unicode_buffer(path)
    n = UNICODE_STRING(len(path) * 2, (len(path) + 1) * 2, ctypes.cast(buf, ctypes.c_void_p))
    oa = OBJECT_ATTRIBUTES()
    oa.Length = ctypes.sizeof(OBJECT_ATTRIBUTES)
    oa.ObjectName = ctypes.pointer(n)
    oa.Attributes = 0x40
    h = ctypes.c_void_p()
    iosb = IO_STATUS_BLOCK()
    st = ntdll.NtCreateFile(ctypes.byref(h), 0x12019F, ctypes.byref(oa), ctypes.byref(iosb),
                            None, 0, 0x1 | 0x2, 0x1, 0, None, 0)  # OPEN_EXISTING
    return h, st

IOCTL_MSR_READ = 0x81112EE0
for devname in (r'\Device\AMDRyzenMasterDriverV20', r'\Device\AMDRyzenMasterDriverV22'):
    h, st = nt_open(devname)
    if not h or st != 0:
        print(f'{devname}: NtCreateFile st={st:#x} (err {ctypes.get_last_error()})')
        continue
    print(f'{devname}: OPENED (st={st:#x})')
    # read several MSRs: 1=PSTATE_STAT, 3=PSTATE_0, 13=RAPL_PWR_UNIT
    for idx_val, label in ((1, 'PSTATE_STAT'), (3, 'PSTATE_0'), (13, 'RAPL_PWR_UNIT')):
        idx = (ctypes.c_uint32 * 4)(idx_val, 0, 0, 0)
        out = (ctypes.c_uint64 * 2)(0, 0)
        ret = wintypes.DWORD(0)
        ok = kernel32.DeviceIoControl(h, IOCTL_MSR_READ, idx, 16, out, 16, ctypes.byref(ret), None)
        print(f'  [{idx_val:2d}] {label:14s}: ok={ok} ret={ret.value} out={out[0]:#018x} {out[1]:#018x}')
    kernel32.CloseHandle(h)
