# probe_ec_reg.py — map the register field: write [opcode][register] + delay + read, find where values appear
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
idx = 0; vh = None; vlen = 0
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
    if 'vid_0d62' not in path.lower() or 'col04' not in path.lower():
        continue
    h = kernel32.CreateFileW(path, GENERIC_READ | GENERIC_WRITE, FILE_SHARE_RW, None, OPEN_EXISTING, 0, None)
    if h == -1 or h == ctypes.c_void_p(-1).value:
        continue
    pp = ctypes.c_void_p(0)
    if hid.HidD_GetPreparsedData(h, ctypes.byref(pp)):
        c = HIDP_CAPS(); hid.HidP_GetCaps(pp, ctypes.byref(c)); hid.HidD_FreePreparsedData(pp)
        if c.UsagePage == 0xFF89:
            vh = h; vlen = c.FeatureReportByteLength; break
    kernel32.CloseHandle(h)
if not vh:
    print('vendor collection not found'); sys.exit(1)

def get():
    buf = (ctypes.c_ubyte * vlen)(); buf[0] = 0xCC
    hid.HidD_GetFeature(vh, buf, vlen)
    return bytes(buf)

def cmd(d7):
    buf = (ctypes.c_ubyte * vlen)(); buf[0] = 0xCC
    for i, b in enumerate(d7): buf[i + 1] = b
    hid.HidD_SetFeature(vh, buf, vlen)
    time.sleep(0.12)
    return get()

# sweep register addresses for opcode 0x01 (hypothetical "read")
print('register sweep: opcode 0x01, register byte2 0x00..0x7F (non-zero responses shown)')
interesting = []
for reg in range(0x80):
    r = cmd([0x01, reg, 0x00, 0x00, 0x00, 0x00, 0x00])
    data = r[1:]  # drop report id
    if any(data):
        interesting.append((reg, r.hex()))
        print(f'  reg {reg:02X}: {r.hex()}')
if not interesting:
    print('  (all-zero responses for opcode 0x01)')

# sweep opcodes 0x00..0x1F with register 0x00, show the full response
print('\nopcode x reg=0 (full response, delay 120ms):')
for op in range(0x20):
    r = cmd([op, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    print(f'  op {op:02X}: {r.hex()}')
print('done')
