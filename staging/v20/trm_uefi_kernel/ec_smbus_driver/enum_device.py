# enum_device.py — enumerate \Device\ to check whether AMDRyzenMasterDriverV20/V22 exist
import ctypes
from ctypes import wintypes

ntdll = ctypes.WinDLL('ntdll', use_last_error=True)

class UNICODE_STRING(ctypes.Structure):
    _fields_ = [('Length', wintypes.USHORT), ('MaximumLength', wintypes.USHORT), ('Buffer', ctypes.c_void_p)]

class OBJECT_ATTRIBUTES(ctypes.Structure):
    _fields_ = [('Length', wintypes.ULONG), ('RootDirectory', ctypes.c_void_p),
                ('ObjectName', ctypes.POINTER(UNICODE_STRING)), ('Attributes', wintypes.ULONG),
                ('SecurityDescriptor', ctypes.c_void_p), ('SecurityQualityOfService', ctypes.c_void_p)]

class OBJECT_DIRECTORY_INFORMATION(ctypes.Structure):
    _fields_ = [('Name', UNICODE_STRING), ('TypeName', UNICODE_STRING)]

def us(name):
    b = ctypes.create_unicode_buffer(name)
    return UNICODE_STRING(len(name) * 2, (len(name) + 1) * 2, ctypes.cast(b, ctypes.c_void_p))

def open_dir(name):
    buf = ctypes.create_unicode_buffer(name)
    n = UNICODE_STRING(len(name) * 2, (len(name) + 1) * 2, ctypes.cast(buf, ctypes.c_void_p))
    oa = OBJECT_ATTRIBUTES()
    oa.Length = ctypes.sizeof(OBJECT_ATTRIBUTES)
    oa.ObjectName = ctypes.pointer(n)
    oa.Attributes = 0x40  # OBJ_CASE_INSENSITIVE
    h = ctypes.c_void_p()
    st = ntdll.NtOpenDirectoryObject(ctypes.byref(h), 0x1 | 0x2, ctypes.byref(oa))
    return h, st

def query_dir(h):
    entries = []
    ctx = wintypes.ULONG(0)
    while True:
        buf = (ctypes.c_ubyte * 0x1000)()
        ret = wintypes.ULONG(0)
        st = ntdll.NtQueryDirectoryObject(h, buf, 0x1000, True, False, ctypes.byref(ctx), ctypes.byref(ret))
        if st != 0:
            break
        info = ctypes.cast(buf, ctypes.POINTER(OBJECT_DIRECTORY_INFORMATION)).contents
        nm = ctypes.wstring_at(info.Name.Buffer, info.Name.Length // 2)
        entries.append(nm)
    return entries

for d in (r'\Device', r'\DosDevices', r'\??'):
    h, st = open_dir(d)
    if not h or st != 0:
        print(f'{d}: open failed st={st:#x}')
        continue
    entries = query_dir(h)
    ntdll.NtClose(h)
    print(f'{d}: {len(entries)} entries')
    for e in entries:
        if 'RyzenMaster' in e or 'AMDRyzen' in e or 'Ryzen' in e:
            print(f'    MATCH: {e}')
