# probe_hid_ec.py — enumerate + read the EC over HID (ITE VID_0D62 + DELL0A6F I2C HID)
import ctypes, ctypes.wintypes as wt, struct, sys

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
class HIDD_ATTRIBUTES(ctypes.Structure):
    _fields_ = [('Size', wt.DWORD), ('VendorID', wt.WORD), ('ProductID', wt.WORD), ('VersionNumber', wt.WORD)]

DIGCF_PRESENT = 0x02; DIGCF_DEVICEINTERFACE = 0x10
GENERIC_READ = 0x80000000; GENERIC_WRITE = 0x40000000
FILE_SHARE_RW = 0x3; OPEN_EXISTING = 3; FILE_FLAG_OVERLAPPED = 0x40000000

setupapi.SetupDiGetClassDevsW.restype = ctypes.c_void_p
setupapi.SetupDiGetClassDevsW.argtypes = [ctypes.POINTER(GUID), wt.LPCWSTR, wt.HWND, wt.DWORD]
setupapi.SetupDiEnumDeviceInterfaces.restype = wt.BOOL
setupapi.SetupDiEnumDeviceInterfaces.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.POINTER(GUID), wt.DWORD, ctypes.POINTER(SP_DEVICE_INTERFACE_DATA)]
setupapi.SetupDiGetDeviceInterfaceDetailW.restype = wt.BOOL
setupapi.SetupDiGetDeviceInterfaceDetailW.argtypes = [ctypes.c_void_p, ctypes.POINTER(SP_DEVICE_INTERFACE_DATA), ctypes.c_void_p, wt.DWORD, ctypes.POINTER(wt.DWORD), ctypes.c_void_p]
kernel32.CreateFileW.restype = ctypes.c_void_p
kernel32.CreateFileW.argtypes = [wt.LPCWSTR, wt.DWORD, wt.DWORD, ctypes.c_void_p, wt.DWORD, wt.DWORD, ctypes.c_void_p]
hid.HidD_GetAttributes.restype = wt.BOOL
hid.HidD_GetAttributes.argtypes = [ctypes.c_void_p, ctypes.POINTER(HIDD_ATTRIBUTES)]
hid.HidD_GetProductString.restype = wt.BOOL
hid.HidD_GetProductString.argtypes = [ctypes.c_void_p, ctypes.c_void_p, wt.DWORD]
class HIDP_CAPS(ctypes.Structure):
    _fields_ = [('Usage', wt.USHORT), ('UsagePage', wt.USHORT), ('InputReportByteLength', wt.USHORT),
                ('OutputReportByteLength', wt.USHORT), ('FeatureReportByteLength', wt.USHORT),
                ('Reserved', wt.USHORT * 17), ('NumberLinkCollectionNodes', wt.USHORT),
                ('NumberInputButtonCaps', wt.USHORT), ('NumberInputValueCaps', wt.USHORT),
                ('NumberInputDataIndices', wt.USHORT), ('NumberOutputButtonCaps', wt.USHORT),
                ('NumberOutputValueCaps', wt.USHORT), ('NumberOutputDataIndices', wt.USHORT),
                ('NumberFeatureButtonCaps', wt.USHORT), ('NumberFeatureValueCaps', wt.USHORT),
                ('NumberFeatureDataIndices', wt.USHORT)]
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

def caps_of(h):
    pp = ctypes.c_void_p(0)
    if not hid.HidD_GetPreparsedData(h, ctypes.byref(pp)):
        return None
    c = HIDP_CAPS()
    hid.HidP_GetCaps(pp, ctypes.byref(c))
    hid.HidD_FreePreparsedData(pp)
    return c

kernel32.ReadFile.restype = wt.BOOL
kernel32.ReadFile.argtypes = [ctypes.c_void_p, ctypes.c_void_p, wt.DWORD, ctypes.POINTER(wt.DWORD), ctypes.c_void_p]

dev = setupapi.SetupDiGetClassDevsW(ctypes.byref(GUID_HID), None, None, DIGCF_PRESENT | DIGCF_DEVICEINTERFACE)
if dev == -1:
    print('SetupDiGetClassDevsW failed'); sys.exit(1)

idx = 0
targets = [(0x0D62, 0x3740), (0x0A6F, 0x0000)]  # ITE EC USB, Dell EC (approx PID)
found = []
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
    h = kernel32.CreateFileW(path, GENERIC_READ | GENERIC_WRITE, FILE_SHARE_RW, None, OPEN_EXISTING, FILE_FLAG_OVERLAPPED, None)
    if h == -1 or h == ctypes.c_void_p(-1).value:
        continue
    attrs = HIDD_ATTRIBUTES(); attrs.Size = ctypes.sizeof(HIDD_ATTRIBUTES)
    if not hid.HidD_GetAttributes(h, ctypes.byref(attrs)):
        kernel32.CloseHandle(h); continue
    prod = ctypes.create_unicode_buffer(128)
    hid.HidD_GetProductString(h, prod, 128)
    print(f'[{idx}] VID {attrs.VendorID:04X} PID {attrs.ProductID:04X}  "{prod.value}"  path={path[:60]}...')
    if (attrs.VendorID, attrs.ProductID) in targets or attrs.VendorID in (0x0D62, 0x0A6F):
        found.append((attrs.VendorID, attrs.ProductID, h, path, prod.value))
    else:
        kernel32.CloseHandle(h)

print(f'\n== {len(found)} EC-candidate device(s) ==')
for vid, pid, h, path, prod in found:
    print(f'\n--- VID {vid:04X} PID {pid:04X} "{prod}"  path={path[:80]} ---')
    c = caps_of(h)
    if c:
        print(f'  caps: usage {c.UsagePage:04X}:{c.Usage:04X}  in {c.InputReportByteLength}B  '
              f'out {c.OutputReportByteLength}B  feat {c.FeatureReportByteLength}B')
    # feature reports (vendor-defined EC data lives here)
    flen = c.FeatureReportByteLength if c else 0
    if flen:
        for report_id in (0, 1, 2, 3, 4):
            buf = (ctypes.c_ubyte * flen)()
            buf[0] = report_id
            ok = hid.HidD_GetFeature(h, buf, flen)
            if ok:
                print(f'  feature report {report_id}: {bytes(buf).hex()}')
            else:
                print(f'  feature report {report_id}: GetFeature failed err={ctypes.get_last_error()}')
    # input report with correct length
    ilen = c.InputReportByteLength if c else 0
    if ilen:
        buf = (ctypes.c_ubyte * ilen)()
        nread = wt.DWORD(0)
        ok = kernel32.ReadFile(h, buf, ilen, ctypes.byref(nread), None)
        print(f'  input report ({ilen}B): ok={ok} nread={nread.value}' + (f' {bytes(buf[:nread.value]).hex()}' if nread.value else f' err={ctypes.get_last_error()}'))
    kernel32.CloseHandle(h)

print('\ndone')
