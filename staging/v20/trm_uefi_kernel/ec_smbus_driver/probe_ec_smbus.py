# probe_ec_smbus.py — post-reboot: load the driver, read the raw EC RAM + SMBus window.
# Run from the driver dir after `bcdedit /set testsigning on` + reboot:
#   sc create trm_ec_smbus type= kernel binPath= <abs path to trm_ec_smbus.sys>
#   sc start trm_ec_smbus
#   python probe_ec_smbus.py
import ctypes, struct, sys
from ctypes import wintypes

k32 = ctypes.WinDLL('kernel32', use_last_error=True)

DEVICE_TYPE = 0x22  # FILE_DEVICE_UNKNOWN
def CTL_CODE(func):  # METHOD_BUFFERED, FILE_ANY_ACCESS
    return (DEVICE_TYPE << 16) | (func << 2)
IOCTL_EC_READ    = CTL_CODE(0x800)
IOCTL_EC_SCAN    = CTL_CODE(0x801)
IOCTL_SMBUS_READ = CTL_CODE(0x802)

h = k32.CreateFileW(r'\\.\trm_ec_smbus', 0x80000000 | 0x40000000,
                    0x1 | 0x2, None, 3, 0, None)  # OPEN_EXISTING
if h == -1:
    print(f'open \\\\.\\trm_ec_smbus failed, err {ctypes.get_last_error()} '
          '(driver loaded? `sc query trm_ec_smbus`)')
    sys.exit(1)

# 1) EC RAM scan — 256 bytes from offset 0
out = (ctypes.c_ubyte * 256)()
ret = wintypes.DWORD(0)
inbuf = struct.pack('BB', 0, 256)
k32.DeviceIoControl(h, IOCTL_EC_SCAN, inbuf, 2, out, 256, ctypes.byref(ret), None)
print(f'EC RAM scan: {ret.value} bytes')
nz = [(i, b) for i, b in enumerate(out[:ret.value]) if b != 0]
print(f'non-zero bytes: {len(nz)}')
print(' '.join(f'{i:02x}:{b:02x}' for i, b in nz))

# 2) one EC byte read (offset 0) as a sanity check
b = (ctypes.c_ubyte * 1)()
r = wintypes.DWORD(0)
k32.DeviceIoControl(h, IOCTL_EC_READ, struct.pack('B', 0), 1, b, 1, ctypes.byref(r), None)
print(f'EC[0] single read: {b[0]:02x}')

k32.CloseHandle(h)
print('done — correlate non-zero offsets with a load change to identify the rails')
