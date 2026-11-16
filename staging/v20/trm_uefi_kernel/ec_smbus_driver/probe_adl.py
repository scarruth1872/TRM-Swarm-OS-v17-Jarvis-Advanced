# probe_adl.py — query AMD ADL for the Radeon iGPU's real telemetry (temp/clocks/voltage/power)
import ctypes
from ctypes import wintypes, c_void_p, c_int, c_uint, c_char_p, c_size_t

adl = ctypes.WinDLL('atiadlxx.dll')

# ADL_MAIN_MALLOC_CALLBACK: void* (*)(int)
MALLOC_CB = ctypes.CFUNCTYPE(c_void_p, c_int)
msvcrt = ctypes.CDLL('msvcrt')
msvcrt.malloc.restype = c_void_p
msvcrt.malloc.argtypes = [c_size_t]
@MALLOC_CB
def _malloc(sz):
    return msvcrt.malloc(sz)

adl.ADL2_Main_Control_Create.restype = c_int
adl.ADL2_Main_Control_Create.argtypes = [MALLOC_CB, c_int]
adl.ADL2_Main_Control_Destroy.restype = c_int

adl.ADL2_Adapter_NumberOfAdapters_Get.restype = c_int
adl.ADL2_Adapter_NumberOfAdapters_Get.argtypes = [ctypes.POINTER(c_int)]

class AdapterInfo(ctypes.Structure):
    _fields_ = [
        ('iSize', c_int),
        ('iAdapterIndex', c_int),
        ('strUDID', ctypes.c_char * 256),
        ('iBusNumber', c_int),
        ('iDeviceNumber', c_int),
        ('iFunctionNumber', c_int),
        ('iVendorID', c_int),
        ('strAdapterName', ctypes.c_char * 256),
        ('strDisplayName', ctypes.c_char * 256),
        ('iPresent', c_int),
        ('iExist', c_int),
        ('strDriverPath', ctypes.c_char * 256),
        ('strDriverPathExt', ctypes.c_char * 256),
        ('strPNPString', ctypes.c_char * 256),
        ('iOSDisplayIndex', c_int),
    ]
adl.ADL2_Adapter_AdapterInfo_Get.restype = c_int
adl.ADL2_Adapter_AdapterInfo_Get.argtypes = [ctypes.POINTER(AdapterInfo), c_int]

class ADLPMActivity(ctypes.Structure):
    _fields_ = [
        ('iSize', c_int),
        ('iEngineClock', c_int),   # MHz
        ('iMemoryClock', c_int),   # MHz
        ('iVddc', c_int),          # mV  <- the voltage field
        ('iActivityPercent', c_int),
        ('iCurrentPerformanceLevel', c_int),
        ('iCurrentBusSpeed', c_int),
        ('iCurrentBusLanes', c_int),
        ('iMaximumBusLanes', c_int),
        ('iReserved', c_int),
    ]
adl.ADL2_Overdrive5_CurrentActivity_Get.restype = c_int
adl.ADL2_Overdrive5_CurrentActivity_Get.argtypes = [c_int, ctypes.POINTER(ADLPMActivity)]

print('== ADL init ==')
for mode in (1, 0):
    r = adl.ADL2_Main_Control_Create(_malloc, mode)
    print(f'ADL2_Main_Control_Create(mode={mode}): {r}')

    num = c_int(0)
    r = adl.ADL2_Adapter_NumberOfAdapters_Get(ctypes.byref(num))
    print(f'  adapters: {num.value} (rc {r})')

    for i in range(num.value):
        info = AdapterInfo()
        info.iSize = ctypes.sizeof(AdapterInfo)
        r = adl.ADL2_Adapter_AdapterInfo_Get(ctypes.byref(info), ctypes.sizeof(AdapterInfo))
        name = info.strAdapterName.decode(errors='replace')
        vendor = hex(info.iVendorID)
        print(f'  adapter {i}: {name} (vendor {vendor}, bus {info.iBusNumber}:{info.iDeviceNumber}.{info.iFunctionNumber})')

        act = ADLPMActivity()
        act.iSize = ctypes.sizeof(ADLPMActivity)
        r = adl.ADL2_Overdrive5_CurrentActivity_Get(i, ctypes.byref(act))
        print(f'    Overdrive5 rc={r}: engine {act.iEngineClock} MHz, mem {act.iMemoryClock} MHz, '
              f'VDDC {act.iVddc} mV, activity {act.iActivityPercent}%')

    try:
        adl.ADL2_Main_Control_Destroy()
    except OSError as e:
        print(f'  destroy skipped: {e}')

print('\ndone')
