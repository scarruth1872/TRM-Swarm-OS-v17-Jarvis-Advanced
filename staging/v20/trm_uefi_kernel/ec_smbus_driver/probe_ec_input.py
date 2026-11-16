# probe_ec_input.py — poll col02/col03 input reports (correct sizes) + characterize the 02 drift
import ctypes, ctypes.wintypes as wt, time, sys, threading

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
kernel32.ReadFile.restype = wt.BOOL
kernel32.ReadFile.argtypes = [ctypes.c_void_p, ctypes.c_void_p, wt.DWORD, ctypes.POINTER(wt.DWORD), ctypes.c_void_p]
hid.HidD_GetPreparsedData.restype = wt.BOOL
hid.HidD_GetPreparsedData.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p)]
hid.HidP_GetCaps.restype = ctypes.c_int
hid.HidP_GetCaps.argtypes = [ctypes.c_void_p, ctypes.POINTER(HIDP_CAPS)]
hid.HidD_FreePreparsedData.restype = wt.BOOL
hid.HidD_FreePreparsedData.argtypes = [ctypes.c_void_p]
hid.HidD_GetFeature.restype = wt.BOOL
hid.HidD_GetFeature.argtypes = [ctypes.c_void_p, ctypes.c_void_p, wt.DWORD]

dev = setupapi.SetupDiGetClassDevsW(ctypes.byref(GUID_HID), None, None, DIGCF_PRESENT | DIGCF_DEVICEINTERFACE)
idx = 0
cols = {}  # col -> (h, caps)
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
    col = path.split('&')[3].split('#')[0] if '&' in path else '?'
    h = kernel32.CreateFileW(path, GENERIC_READ | GENERIC_WRITE, FILE_SHARE_RW, None, OPEN_EXISTING, 0, None)
    if h == -1 or h == ctypes.c_void_p(-1).value:
        continue
    pp = ctypes.c_void_p(0)
    if hid.HidD_GetPreparsedData(h, ctypes.byref(pp)):
        c = HIDP_CAPS(); hid.HidP_GetCaps(pp, ctypes.byref(c)); hid.HidD_FreePreparsedData(pp)
        cols[col] = (h, c)
        print(f'{col}: usage {c.UsagePage:04X}:{c.Usage:04X} in {c.InputReportByteLength}B out {c.OutputReportByteLength}B feat {c.FeatureReportByteLength}B')

def read_timeout(h, size, timeout=1.0):
    res = []
    def go():
        buf = (ctypes.c_ubyte * size)()
        nread = wt.DWORD(0)
        ok = kernel32.ReadFile(h, buf, size, ctypes.byref(nread), None)
        res.append((ok, bytes(buf[:nread.value])))
    t = threading.Thread(target=go, daemon=True)
    t.start(); t.join(timeout)
    if t.is_alive():
        return None  # timed out, no report pending
    return res[0] if res else None

# 1) poll input reports on col02/col03
print('\n== input-report poll (col02/col03, correct sizes, 1s timeout) ==')
for col, (h, c) in sorted(cols.items()):
    if c.InputReportByteLength == 0:
        continue
    r = read_timeout(h, c.InputReportByteLength, 1.0)
    print(f'  {col} (in {c.InputReportByteLength}B): {"timeout (no report)" if r is None else f"ok={r[0]} data={r[1].hex()}"}')

# 2) characterize the 02 drift over 60s at 1Hz
vendor = None
for col, (h, c) in cols.items():
    if c.FeatureReportByteLength > 0:
        vendor = (col, h, c)
if vendor:
    _, vh, vc = vendor
    def get():
        buf = (ctypes.c_ubyte * vc.FeatureReportByteLength)(); buf[0] = 0xCC
        hid.HidD_GetFeature(vh, buf, vc.FeatureReportByteLength)
        return bytes(buf)
    print(f'\n== 02 drift: 60 samples, 1s apart ==')
    hist = {}
    for i in range(60):
        b = get()
        hist[b.hex()] = hist.get(b.hex(), 0) + 1
        if i == 0 or i % 15 == 14 or b.hex() != list(hist)[0]:
            print(f'  t={i:2d}s {b.hex()}')
        time.sleep(1)
    print('histogram:')
    for k, v in sorted(hist.items()):
        print(f'  {k}: {v}x')
print('done')
