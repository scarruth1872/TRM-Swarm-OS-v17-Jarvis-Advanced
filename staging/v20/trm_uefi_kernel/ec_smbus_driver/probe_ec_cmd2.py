# probe_ec_cmd2.py — report-ID/collection variation, async write delay, and observe the byte drift over time
import ctypes, ctypes.wintypes as wt, time, sys

setupapi = ctypes.WinDLL('setupapi', use_last_error=True)
hid = ctypes.WinDLL('hid', use_last_error=True)
kernel32 = ctypes.WinDLL('kernel32', use_last_error=True)

class GUID(ctypes.Structure):
    _fields_ = [('d1', wt.DWORD), ('d2', wt.WORD), ('d3', wt.WORD), ('d4', ctypes.c_ubyte * 8)]
GUID_HID = GUID(0x4D1E55B2, 0xF16F, 0x11CF, (0x88, 0xCB, 0x00, 0x11, 0x11, 0x00, 0x00, 0x30))
class SP_DEVICE_INTERFACE_DATA(ctypes.Structure):
    _fields_ = [('cbSize', wt.DWORD), ('InterfaceClassGuid', GUID), ('Flags', wt.DWORD), ('Reserved', ctypes.c_void_p)]
class SP_DEVICE_INTERFACE_DETAIL_DATA_W(ctypes.Structure):
    _fields_ = [('cbSize', wt.DWORD), ('DevicePath', wt.WCHAR * 1)]
class HIDP_CAPS(ctypes.Structure):
    _fields_ = [('Usage', wt.USHORT), ('UsagePage', wt.USHORT), ('InputReportByteLength', wt.USHORT),
                ('OutputReportByteLength', wt.USHORT), ('FeatureReportByteLength', wt.USHORT),
                ('Reserved', wt.USHORT * 17), ('NumberLinkCollectionNodes', wt.USHORT),
                ('NumberInputButtonCaps', wt.USHORT), ('NumberInputValueCaps', wt.USHORT),
                ('NumberInputDataIndices', wt.USHORT), ('NumberOutputButtonCaps', wt.USHORT),
                ('NumberOutputValueCaps', wt.USHORT), ('NumberOutputDataIndices', wt.USHORT),
                ('NumberFeatureButtonCaps', wt.USHORT), ('NumberFeatureValueCaps', wt.USHORT),
                ('NumberFeatureDataIndices', wt.USHORT)]

DIGCF_PRESENT = 0x02; DIGCF_DEVICEINTERFACE = 0x10
GENERIC_READ = 0x80000000; GENERIC_WRITE = 0x40000000
FILE_SHARE_RW = 0x3; OPEN_EXISTING = 3

setupapi.SetupDiGetClassDevsW.restype = ctypes.c_void_p
setupapi.SetupDiGetClassDevsW.argtypes = [ctypes.POINTER(GUID), wt.LPCWSTR, wt.HWND, wt.DWORD]
setupapi.SetupDiEnumDeviceInterfaces.restype = wt.BOOL
setupapi.SetupDiEnumDeviceInterfaces.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.POINTER(GUID), wt.DWORD, ctypes.POINTER(SP_DEVICE_INTERFACE_DATA)]
setupapi.SetupDiGetDeviceInterfaceDetailW.restype = wt.BOOL
setupapi.SetupDiGetDeviceInterfaceDetailW.argtypes = [ctypes.c_void_p, ctypes.POINTER(SP_DEVICE_INTERFACE_DATA), ctypes.c_void_p, wt.DWORD, ctypes.POINTER(wt.DWORD), ctypes.c_void_p]
kernel32.CreateFileW.restype = ctypes.c_void_p
kernel32.CreateFileW.argtypes = [wt.LPCWSTR, wt.DWORD, wt.DWORD, ctypes.c_void_p, wt.DWORD, wt.DWORD, ctypes.c_void_p]
kernel32.WriteFile.restype = wt.BOOL
kernel32.WriteFile.argtypes = [ctypes.c_void_p, ctypes.c_void_p, wt.DWORD, ctypes.POINTER(wt.DWORD), ctypes.c_void_p]
hid.HidD_GetPreparsedData.restype = wt.BOOL
hid.HidD_GetPreparsedData.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p)]
hid.HidP_GetCaps.restype = ctypes.c_int
hid.HidP_GetCaps.argtypes = [ctypes.c_void_p, ctypes.POINTER(HIDP_CAPS)]
hid.HidD_FreePreparsedData.restype = wt.BOOL
hid.HidD_FreePreparsedData.argtypes = [ctypes.c_void_p]
hid.HidD_GetFeature.restype = wt.BOOL
hid.HidD_GetFeature.argtypes = [ctypes.c_void_p, ctypes.c_void_p, wt.DWORD]
hid.HidD_SetFeature.restype = wt.BOOL
hid.HidD_SetFeature.argtypes = [ctypes.c_void_p, ctypes.c_void_p, wt.DWORD]

dev = setupapi.SetupDiGetClassDevsW(ctypes.byref(GUID_HID), None, None, DIGCF_PRESENT | DIGCF_DEVICEINTERFACE)
idx = 0
handles = {}  # col -> (h, caps)
while True:
    idata = SP_DEVICE_INTERFACE_DATA(); idata.cbSize = ctypes.sizeof(SP_DEVICE_INTERFACE_DATA)
    if not setupapi.SetupDiEnumDeviceInterfaces(dev, None, ctypes.byref(GUID_HID), idx, ctypes.byref(idata)):
        break
    idx += 1
    need = wt.DWORD(0)
    setupapi.SetupDiGetDeviceInterfaceDetailW(dev, ctypes.byref(idata), None, 0, ctypes.byref(need), None)
    detail = (ctypes.c_ubyte * need.value)()
    d = ctypes.cast(detail, ctypes.POINTER(SP_DEVICE_INTERFACE_DETAIL_DATA_W))
    d.contents.cbSize = ctypes.sizeof(SP_DEVICE_INTERFACE_DETAIL_DATA_W)
    if not setupapi.SetupDiGetDeviceInterfaceDetailW(dev, ctypes.byref(idata), detail, need.value, None, None):
        continue
    path = ctypes.wstring_at(ctypes.addressof(d.contents) + SP_DEVICE_INTERFACE_DETAIL_DATA_W.DevicePath.offset)
    if 'vid_0d62' not in path.lower():
        continue
    col = path.split('&')[2] if '&' in path else '?'
    h = kernel32.CreateFileW(path, GENERIC_READ | GENERIC_WRITE, FILE_SHARE_RW, None, OPEN_EXISTING, 0, None)
    if h == -1 or h == ctypes.c_void_p(-1).value:
        continue
    pp = ctypes.c_void_p(0)
    if hid.HidD_GetPreparsedData(h, ctypes.byref(pp)):
        c = HIDP_CAPS(); hid.HidP_GetCaps(pp, ctypes.byref(c)); hid.HidD_FreePreparsedData(pp)
        handles[col] = (h, c)
        print(f'{col}: usage {c.UsagePage:04X}:{c.Usage:04X} in {c.InputReportByteLength}B out {c.OutputReportByteLength}B feat {c.FeatureReportByteLength}B')

# vendor collection = the one with FeatureReportByteLength > 0
vendor = None
for col, (h, c) in handles.items():
    if c.FeatureReportByteLength > 0:
        vendor = (col, h, c)
if not vendor:
    print('no vendor collection'); sys.exit(1)
vcol, vh, vc = vendor
print(f'vendor = {vcol} (feat {vc.FeatureReportByteLength}B)')

def get():
    buf = (ctypes.c_ubyte * vc.FeatureReportByteLength)(); buf[0] = 0xCC
    hid.HidD_GetFeature(vh, buf, vc.FeatureReportByteLength)
    return bytes(buf)

def setf(data7):
    buf = (ctypes.c_ubyte * vc.FeatureReportByteLength)(); buf[0] = 0xCC
    for i, b in enumerate(data7): buf[i + 1] = b
    return hid.HidD_SetFeature(vh, buf, vc.FeatureReportByteLength)

# 1) async write test: write candidate, DELAY, read
base = get()
print(f'\nbaseline: {base.hex()}')
print('write + 100ms delay + read (opcode 0x00..0x1F, reg 0):')
found = []
for op in range(0x20):
    setf([op, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    time.sleep(0.1)
    r = get()
    if r != base:
        found.append((op, r.hex()))
        print(f'  opcode {op:02X} -> {r.hex()}')
if not found:
    print('  (no opcode changed the response even with delay)')

# 2) report-ID variation on the SAME vendor collection: try write report IDs 0x00..0x1F
print('\nwrite-report-ID sweep (feature report ID byte 0x00..0x1F):')
found = []
for rid in range(0x20):
    buf = (ctypes.c_ubyte * vc.FeatureReportByteLength)(); buf[0] = rid
    for i in range(7): buf[i + 1] = 0
    ok = hid.HidD_SetFeature(vh, buf, vc.FeatureReportByteLength)
    time.sleep(0.05)
    r = get()
    if r != base:
        found.append((rid, r.hex(), ok))
        print(f'  reportID {rid:02X} -> {r.hex()} (write_ok={ok})')
if not found:
    print('  (no alternate report ID changed the response)')

# 3) observe the byte drift over ~30s
print('\nobserving drift (30 samples, 1s apart):')
seen = {}
for i in range(30):
    r = get()
    h = r.hex()
    seen[h] = seen.get(h, 0) + 1
    if i < 5 or r.hex() != base:
        print(f'  t={i:2d}s {h}')
    time.sleep(1)
print('payload histogram over 30s:')
for k, v in sorted(seen.items()):
    print(f'  {k}: {v}x')
print('done')
