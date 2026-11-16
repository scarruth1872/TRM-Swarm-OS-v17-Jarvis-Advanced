# probe_hid_valuecaps.py — decode the ITE EC vendor collection's feature-report structure
import ctypes, ctypes.wintypes as wt, sys

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
class HIDP_VALUE_CAPS(ctypes.Structure):
    _fields_ = [
        ('UsagePage', wt.USHORT), ('ReportID', ctypes.c_ubyte), ('IsAlias', ctypes.c_ubyte),
        ('BitField', wt.USHORT), ('LinkCollection', wt.USHORT), ('LinkUsage', wt.USHORT),
        ('LinkUsagePage', wt.USHORT), ('IsRange', ctypes.c_ubyte), ('IsStringRange', ctypes.c_ubyte),
        ('IsDesignatorRange', ctypes.c_ubyte), ('IsAbsolute', ctypes.c_ubyte), ('HasNull', ctypes.c_ubyte),
        ('Reserved', ctypes.c_ubyte), ('BitSize', wt.USHORT), ('ReportCount', wt.USHORT),
        ('Reserved2', wt.USHORT * 5), ('UnitsExp', wt.ULONG), ('Units', wt.ULONG),
        ('LogicalMin', ctypes.c_long), ('LogicalMax', ctypes.c_long),
        ('PhysicalMin', ctypes.c_long), ('PhysicalMax', ctypes.c_long),
        ('u', ctypes.c_ubyte * 16),
    ]

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
hid.HidD_GetAttributes.restype = wt.BOOL
hid.HidD_GetAttributes.argtypes = [ctypes.c_void_p, ctypes.POINTER(HIDD_ATTRIBUTES)]
hid.HidD_GetPreparsedData.restype = wt.BOOL
hid.HidD_GetPreparsedData.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p)]
hid.HidP_GetValueCaps.restype = ctypes.c_int
hid.HidP_GetValueCaps.argtypes = [ctypes.c_int, ctypes.POINTER(HIDP_VALUE_CAPS), ctypes.POINTER(wt.USHORT), ctypes.c_void_p]
hid.HidD_FreePreparsedData.restype = wt.BOOL
hid.HidD_FreePreparsedData.argtypes = [ctypes.c_void_p]

dev = setupapi.SetupDiGetClassDevsW(ctypes.byref(GUID_HID), None, None, DIGCF_PRESENT | DIGCF_DEVICEINTERFACE)
idx = 0
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
    h = kernel32.CreateFileW(path, GENERIC_READ | GENERIC_WRITE, FILE_SHARE_RW, None, OPEN_EXISTING, 0, None)
    if h == -1 or h == ctypes.c_void_p(-1).value:
        continue
    attrs = HIDD_ATTRIBUTES(); attrs.Size = ctypes.sizeof(HIDD_ATTRIBUTES)
    if not hid.HidD_GetAttributes(h, ctypes.byref(attrs)):
        kernel32.CloseHandle(h); continue
    if not (attrs.VendorID == 0x0D62 and attrs.ProductID == 0x3740):
        kernel32.CloseHandle(h); continue
    # ITE EC — enumerate feature value caps
    pp = ctypes.c_void_p(0)
    if not hid.HidD_GetPreparsedData(h, ctypes.byref(pp)):
        kernel32.CloseHandle(h); continue
    n = wt.USHORT(0)
    hid.HidP_GetValueCaps(2, None, ctypes.byref(n), pp)  # 2 = HidP_Feature
    caps = (HIDP_VALUE_CAPS * n.value)()
    hid.HidP_GetValueCaps(2, caps, ctypes.byref(n), pp)
    print(f'ITE EC collection ({path.split("&")[2] if "&" in path else path[:40]}): {n.value} feature value caps')
    for c in caps:
        usage = ctypes.cast(ctypes.byref(c.u), ctypes.POINTER(wt.USHORT)).contents.value
        print(f'  reportID {c.ReportID:02X} usage {c.UsagePage:04X}:{usage:04X} bitfield {c.BitField} bitsize {c.BitSize} count {c.ReportCount}')
    hid.HidD_FreePreparsedData(pp)
    kernel32.CloseHandle(h)
print('done')
