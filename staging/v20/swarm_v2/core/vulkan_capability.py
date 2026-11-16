"""Vulkan capability and a mapped device buffer, MEASURED on this host.

Vulkan is the platform's only path that touches GPU memory: the Infinity Cache pattern write in
`gpu_infinity_computer.load_architecture` and the VRAM figure `hardware_voltage_microkernel_binder` documents
in its header. Both need the same facts, so they live here ONCE -- the physical devices the loader reports
(name, type, vendor, api/driver version), the DEVICE-LOCAL heap sizes the driver reports, and a minimal
host-visible buffer that can be written, read back and verified.

Nothing here is declared. If the loader is absent, no device enumerates, or an allocation fails, the call
returns `ok: False` **with the reason and no number**. `ctypes` on the system loader -- no new dependency, and
`vulkaninfo.exe` stays the independent cross-check.

What this does NOT claim: a mapped buffer is device-VISIBLE memory, not the GPU's Infinity Cache. The 96MB IC in
`GPU_SPECS` is a design target the driver does not expose as a memory type; `device_local_bytes` is what the
driver actually reports for this machine.
"""
import ctypes
import hashlib
import json
import os
import time

VK_SUCCESS = 0
VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU = 2
VK_MEMORY_HEAP_DEVICE_LOCAL_BIT = 0x00000001
VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT = 0x00000002
VK_MEMORY_PROPERTY_HOST_COHERENT_BIT = 0x00000004
VK_BUFFER_USAGE_TRANSFER_DST_BIT = 0x00000002
VK_BUFFER_USAGE_STORAGE_BUFFER_BIT = 0x00000020
MAX_DEVICE_NAME = 256
MAX_MEMORY_HEAPS = 16
MAX_MEMORY_TYPES = 32
TYPE_NAMES = {0: "OTHER", 1: "INTEGRATED_GPU", 2: "DISCRETE_GPU", 3: "VIRTUAL_GPU", 4: "CPU"}


class VkApplicationInfo(ctypes.Structure):
    _fields_ = [("sType", ctypes.c_uint32), ("pNext", ctypes.c_void_p), ("pApplicationName", ctypes.c_char_p),
                ("applicationVersion", ctypes.c_uint32), ("pEngineName", ctypes.c_char_p),
                ("engineVersion", ctypes.c_uint32), ("apiVersion", ctypes.c_uint32)]


class VkInstanceCreateInfo(ctypes.Structure):
    _fields_ = [("sType", ctypes.c_uint32), ("pNext", ctypes.c_void_p), ("flags", ctypes.c_uint32),
                ("pApplicationInfo", ctypes.POINTER(VkApplicationInfo)), ("enabledLayerCount", ctypes.c_uint32),
                ("ppEnabledLayerNames", ctypes.c_void_p), ("enabledExtensionCount", ctypes.c_uint32),
                ("ppEnabledExtensionNames", ctypes.c_void_p)]


class VkPhysicalDeviceProperties(ctypes.Structure):
    """The leading fields are read; `_rest` covers the tail so the driver's full write fits.

    The struct is 824 bytes on x64 (limits + sparse properties follow `pipelineCacheUUID`) and the driver writes
    ALL of it. Declaring only the leading 276 bytes does not truncate the write -- it overruns the Python
    allocation and corrupts the heap, which showed up as a segfault in a LATER call rather than here. Oversizing
    is the safe direction; `vulkan_capability.probe_properties_write_extent()` measures how much is actually
    written so the padding is a fact, not a guess.
    """
    _fields_ = [("apiVersion", ctypes.c_uint32), ("driverVersion", ctypes.c_uint32),
                ("vendorID", ctypes.c_uint32), ("deviceID", ctypes.c_uint32), ("deviceType", ctypes.c_uint32),
                ("deviceName", ctypes.c_char * MAX_DEVICE_NAME), ("pipelineCacheUUID", ctypes.c_ubyte * 16),
                ("_rest", ctypes.c_ubyte * 2048)]


def probe_properties_write_extent(device_index=0, sentinel=0xAB, span=4096):
    """MEASURE how many bytes the driver writes into VkPhysicalDeviceProperties.

    Fills a buffer with a sentinel byte, hands it to `vkGetPhysicalDeviceProperties`, and reports the last byte
    that changed -- the driver's own struct size for the header it was built against. This is what makes the
    padding above safe to choose rather than lucky.
    """
    lib, why = _loader()
    if lib is None:
        return {"ok": False, "reason": why}
    inst, why = _open_instance(lib)
    if inst is None:
        return {"ok": False, "reason": why}
    try:
        n = ctypes.c_uint32(0)
        lib.vkEnumeratePhysicalDevices(inst, ctypes.byref(n), None)
        if n.value == 0:
            return {"ok": False, "reason": "no physical device"}
        arr = (ctypes.c_void_p * n.value)()
        lib.vkEnumeratePhysicalDevices(inst, ctypes.byref(n), arr)
        if not (0 <= device_index < n.value):
            return {"ok": False, "reason": "device_index out of range"}
        p = VkPhysicalDeviceProperties()
        ctypes.memset(ctypes.byref(p), sentinel, ctypes.sizeof(p))
        lib.vkGetPhysicalDeviceProperties(arr[device_index], ctypes.byref(p))
        wrote = ctypes.string_at(ctypes.byref(p), ctypes.sizeof(p))
        written = 0
        for i, b in enumerate(wrote):
            if b != sentinel:
                written = i + 1
        return {"ok": True, "device_index": device_index, "write_extent_bytes": written,
                "struct_bytes": ctypes.sizeof(VkPhysicalDeviceProperties), "sentinel": hex(sentinel),
                "basis": "sentinel-fill of the typed struct + vkGetPhysicalDeviceProperties + last changed byte"}
    finally:
        lib.vkDestroyInstance(inst, None)


class VkMemoryType(ctypes.Structure):
    _fields_ = [("propertyFlags", ctypes.c_uint32), ("heapIndex", ctypes.c_uint32)]


class VkMemoryHeap(ctypes.Structure):
    _fields_ = [("size", ctypes.c_uint64), ("flags", ctypes.c_uint32)]


class VkPhysicalDeviceMemoryProperties(ctypes.Structure):
    _fields_ = [("memoryTypeCount", ctypes.c_uint32), ("memoryTypes", VkMemoryType * MAX_MEMORY_TYPES),
                ("memoryHeapCount", ctypes.c_uint32), ("memoryHeaps", VkMemoryHeap * MAX_MEMORY_HEAPS)]


class VkBufferCreateInfo(ctypes.Structure):
    _fields_ = [("sType", ctypes.c_uint32), ("pNext", ctypes.c_void_p), ("flags", ctypes.c_uint32),
                ("size", ctypes.c_uint64), ("usage", ctypes.c_uint32), ("sharingMode", ctypes.c_uint32),
                ("queueFamilyIndexCount", ctypes.c_uint32), ("pQueueFamilyIndices", ctypes.c_void_p)]


class VkMemoryRequirements(ctypes.Structure):
    _fields_ = [("size", ctypes.c_uint64), ("alignment", ctypes.c_uint64), ("memoryTypeBits", ctypes.c_uint32)]


class VkMemoryAllocateInfo(ctypes.Structure):
    _fields_ = [("sType", ctypes.c_uint32), ("pNext", ctypes.c_void_p), ("allocationSize", ctypes.c_uint64),
                ("memoryTypeIndex", ctypes.c_uint32)]


class VkQueueFamilyProperties(ctypes.Structure):
    _fields_ = [("queueFlags", ctypes.c_uint32), ("queueCount", ctypes.c_uint32),
                ("timestampValidBits", ctypes.c_uint32), ("minImageTransferGranularity", ctypes.c_uint32 * 3)]


class VkDeviceQueueCreateInfo(ctypes.Structure):
    _fields_ = [("sType", ctypes.c_uint32), ("pNext", ctypes.c_void_p), ("flags", ctypes.c_uint32),
                ("queueFamilyIndex", ctypes.c_uint32), ("queueCount", ctypes.c_uint32),
                ("pQueuePriorities", ctypes.POINTER(ctypes.c_float))]


class VkPhysicalDeviceFeatures(ctypes.Structure):
    """An INPUT struct the driver only reads, so oversized zeroed storage is safe (55 VkBool32 = 220 bytes)."""
    _fields_ = [("_raw", ctypes.c_ubyte * 256)]


class VkDeviceCreateInfo(ctypes.Structure):
    """sType = 3. `pEnabledFeatures` is read-only to the driver; zeroed means "everything unsupported"."""
    _fields_ = [("sType", ctypes.c_uint32), ("pNext", ctypes.c_void_p), ("flags", ctypes.c_uint32),
                ("queueCreateInfoCount", ctypes.c_uint32),
                ("pQueueCreateInfos", ctypes.POINTER(VkDeviceQueueCreateInfo)), ("enabledLayerCount",
                                                                                 ctypes.c_uint32),
                ("ppEnabledLayerNames", ctypes.c_void_p), ("enabledExtensionCount", ctypes.c_uint32),
                ("ppEnabledExtensionNames", ctypes.c_void_p),
                ("pEnabledFeatures", ctypes.POINTER(VkPhysicalDeviceFeatures))]


def _loader():
    """The system Vulkan loader, or a reason. No SDK path, no vendored copy."""
    path = os.path.join(os.environ.get("SystemRoot", "C:\\Windows"), "System32", "vulkan-1.dll")
    if not os.path.exists(path):
        return None, "no Vulkan loader at %s" % path
    try:
        lib = ctypes.CDLL(path)
    except OSError as e:
        return None, "loader present but not loadable: %s" % e
    for fn, argt, rest in (
            ("vkCreateInstance", [ctypes.POINTER(VkInstanceCreateInfo), ctypes.c_void_p,
                                  ctypes.POINTER(ctypes.c_void_p)], ctypes.c_int32),
            ("vkDestroyInstance", [ctypes.c_void_p, ctypes.c_void_p], None),
            ("vkEnumeratePhysicalDevices", [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint32),
                                            ctypes.POINTER(ctypes.c_void_p)], ctypes.c_int32),
            ("vkGetPhysicalDeviceProperties", [ctypes.c_void_p, ctypes.POINTER(VkPhysicalDeviceProperties)], None),
            ("vkGetPhysicalDeviceMemoryProperties",
             [ctypes.c_void_p, ctypes.POINTER(VkPhysicalDeviceMemoryProperties)], None)):
        try:
            f = getattr(lib, fn)
            f.argtypes = argt
            if rest is not None:
                f.restype = rest
        except AttributeError as e:
            return None, "loader is missing %s: %s" % (fn, e)
    return lib, path


def enumerate_devices():
    """Every physical device the loader reports, with the device-local memory the driver reports for it."""
    lib, why = _loader()
    if lib is None:
        return {"ok": False, "reason": why, "devices": [], "device_count": 0}
    app = VkApplicationInfo(sType=0, pNext=None, pApplicationName=b"trm-swarm-os", applicationVersion=1,
                            pEngineName=b"trm", engineVersion=1, apiVersion=(1 << 22))  # 1.0.0 always accepted
    ci = VkInstanceCreateInfo(sType=1, pNext=None, flags=0, pApplicationInfo=ctypes.pointer(app),
                              enabledLayerCount=0, ppEnabledLayerNames=None, enabledExtensionCount=0,
                              ppEnabledExtensionNames=None)
    inst = ctypes.c_void_p()
    rc = lib.vkCreateInstance(ctypes.byref(ci), None, ctypes.byref(inst))
    if rc != VK_SUCCESS:
        return {"ok": False, "reason": "vkCreateInstance returned %d" % rc, "devices": [], "device_count": 0}
    try:
        n = ctypes.c_uint32(0)
        rc = lib.vkEnumeratePhysicalDevices(inst, ctypes.byref(n), None)
        if rc != VK_SUCCESS or n.value == 0:
            return {"ok": False, "reason": "vkEnumeratePhysicalDevices rc=%d count=%d" % (rc, n.value),
                    "devices": [], "device_count": 0}
        arr = (ctypes.c_void_p * n.value)()
        rc = lib.vkEnumeratePhysicalDevices(inst, ctypes.byref(n), arr)
        if rc != VK_SUCCESS:
            return {"ok": False, "reason": "second enumerate rc=%d" % rc, "devices": [], "device_count": 0}
        out = []
        for i in range(n.value):
            p = VkPhysicalDeviceProperties()
            lib.vkGetPhysicalDeviceProperties(arr[i], ctypes.byref(p))
            mp = VkPhysicalDeviceMemoryProperties()
            lib.vkGetPhysicalDeviceMemoryProperties(arr[i], ctypes.byref(mp))
            heaps = [{"size_bytes": mp.memoryHeaps[h].size, "device_local": bool(
                mp.memoryHeaps[h].flags & VK_MEMORY_HEAP_DEVICE_LOCAL_BIT)} for h in range(mp.memoryHeapCount)]
            out.append({
                "index": i,
                "name": p.deviceName.decode("utf-8", "replace").strip("\x00"),
                "type": TYPE_NAMES.get(p.deviceType, "UNKNOWN(%d)" % p.deviceType),
                "device_type_raw": p.deviceType,
                "vendor_id": "0x%04x" % p.vendorID,
                "device_id": "0x%04x" % p.deviceID,
                "api_version": "%d.%d.%d" % (p.apiVersion >> 22, (p.apiVersion >> 12) & 0x3FF, p.apiVersion & 0xFFF),
                "driver_version_raw": p.driverVersion,
                "device_local_bytes": sum(h["size_bytes"] for h in heaps if h["device_local"]),
                "heaps": heaps,
            })
        return {"ok": True, "device_count": n.value, "devices": out,
                "basis": "ctypes on the system vulkan-1.dll: vkEnumeratePhysicalDevices + "
                         "vkGetPhysicalDeviceProperties + vkGetPhysicalDeviceMemoryProperties"}
    finally:
        lib.vkDestroyInstance(inst, None)


_BUFFER_FNS = (
    ("vkGetPhysicalDeviceQueueFamilyProperties", [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint32),
                                                  ctypes.POINTER(VkQueueFamilyProperties)], None),
    ("vkCreateDevice", [ctypes.c_void_p, ctypes.POINTER(VkDeviceCreateInfo), ctypes.c_void_p,
                        ctypes.POINTER(ctypes.c_void_p)], ctypes.c_int32),
    ("vkDestroyDevice", [ctypes.c_void_p, ctypes.c_void_p], None),
    ("vkCreateBuffer", [ctypes.c_void_p, ctypes.POINTER(VkBufferCreateInfo), ctypes.c_void_p,
                        ctypes.POINTER(ctypes.c_void_p)], ctypes.c_int32),
    ("vkDestroyBuffer", [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p], None),
    ("vkGetBufferMemoryRequirements", [ctypes.c_void_p, ctypes.c_void_p, ctypes.POINTER(VkMemoryRequirements)],
     None),
    ("vkAllocateMemory", [ctypes.c_void_p, ctypes.POINTER(VkMemoryAllocateInfo), ctypes.c_void_p,
                          ctypes.POINTER(ctypes.c_void_p)], ctypes.c_int32),
    ("vkFreeMemory", [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p], None),
    ("vkBindBufferMemory", [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint64], ctypes.c_int32),
    ("vkMapMemory", [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint64, ctypes.c_uint64, ctypes.c_uint32,
                     ctypes.POINTER(ctypes.c_void_p)], ctypes.c_int32),
    ("vkUnmapMemory", [ctypes.c_void_p, ctypes.c_void_p], None),
)


def _bind_buffer_fns(lib):
    for fn, argt, rest in _BUFFER_FNS:
        try:
            f = getattr(lib, fn)
            f.argtypes = argt
            if rest is not None:
                f.restype = rest
        except AttributeError as e:
            return "loader is missing %s: %s" % (fn, e)
    return None


def _open_instance(lib):
    app = VkApplicationInfo(sType=0, pNext=None, pApplicationName=b"trm-swarm-os", applicationVersion=1,
                            pEngineName=b"trm", engineVersion=1, apiVersion=(1 << 22))
    ci = VkInstanceCreateInfo(sType=1, pNext=None, flags=0, pApplicationInfo=ctypes.pointer(app),
                              enabledLayerCount=0, ppEnabledLayerNames=None, enabledExtensionCount=0,
                              ppEnabledExtensionNames=None)
    inst = ctypes.c_void_p()
    rc = lib.vkCreateInstance(ctypes.byref(ci), None, ctypes.byref(inst))
    return (inst, None) if rc == VK_SUCCESS else (None, "vkCreateInstance returned %d" % rc)


def write_to_device(payload, device_index=None, writes=5):
    """Write `payload` into a mapped device buffer, read it back, verify, and time the writes.

    The buffer is a host-visible, host-coherent device buffer: the same class of object the Infinity Cache
    pattern write needs (a 128-byte line per cache line). Sizes are taken from the driver's own
    `VkMemoryRequirements`, never assumed. The returned timing is a DISTRIBUTION over `writes` writes -- a single
    figure would be the declared-number defect with a measurement's clothes on.
    """
    if not payload:
        return {"ok": False, "reason": "empty payload"}
    lib, why = _loader()
    if lib is None:
        return {"ok": False, "reason": why}
    why = _bind_buffer_fns(lib)
    if why:
        return {"ok": False, "reason": why}
    inst, why = _open_instance(lib)
    if inst is None:
        return {"ok": False, "reason": why}
    buf = mem = None
    dev = None
    ptr = ctypes.c_void_p()
    try:
        n = ctypes.c_uint32(0)
        lib.vkEnumeratePhysicalDevices(inst, ctypes.byref(n), None)
        if n.value == 0:
            return {"ok": False, "reason": "no physical device"}
        arr = (ctypes.c_void_p * n.value)()
        lib.vkEnumeratePhysicalDevices(inst, ctypes.byref(n), arr)
        idx = device_index
        if idx is None:
            idx = 0
            for i in range(n.value):
                p = VkPhysicalDeviceProperties()
                lib.vkGetPhysicalDeviceProperties(arr[i], ctypes.byref(p))
                if p.deviceType == VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU:
                    idx = i
                    break
        if not (0 <= idx < n.value):
            return {"ok": False, "reason": "device_index %s out of range (%d devices)" % (idx, n.value)}
        phys = arr[idx]
        props = VkPhysicalDeviceProperties()
        lib.vkGetPhysicalDeviceProperties(phys, ctypes.byref(props))

        # Device-level functions need a LOGICAL device. Handing a VkPhysicalDevice to vkCreateBuffer is the error
        # the loader reports as "Invalid device" -- the physical handle is not a dispatchable device object.
        qn = ctypes.c_uint32(0)
        lib.vkGetPhysicalDeviceQueueFamilyProperties(phys, ctypes.byref(qn), None)
        if qn.value == 0:
            return {"ok": False, "reason": "no queue family on this device"}
        qs = (VkQueueFamilyProperties * qn.value)()
        lib.vkGetPhysicalDeviceQueueFamilyProperties(phys, ctypes.byref(qn), qs)
        fam = next((i for i in range(qn.value) if qs[i].queueCount > 0), None)
        if fam is None:
            return {"ok": False, "reason": "no queue family with a queue available"}
        prio = (ctypes.c_float * 1)(1.0)
        qci = VkDeviceQueueCreateInfo(sType=2, pNext=None, flags=0, queueFamilyIndex=fam, queueCount=1,
                                      pQueuePriorities=prio)
        feats = VkPhysicalDeviceFeatures()
        dci = VkDeviceCreateInfo(sType=3, pNext=None, flags=0, queueCreateInfoCount=1,
                                 pQueueCreateInfos=ctypes.pointer(qci), enabledLayerCount=0,
                                 ppEnabledLayerNames=None, enabledExtensionCount=0, ppEnabledExtensionNames=None,
                                 pEnabledFeatures=ctypes.pointer(feats))
        ldev = ctypes.c_void_p()
        rc = lib.vkCreateDevice(phys, ctypes.byref(dci), None, ctypes.byref(ldev))
        if rc != VK_SUCCESS:
            return {"ok": False, "reason": "vkCreateDevice rc=%d (queue family %d)" % (rc, fam)}
        dev = ldev

        bi = VkBufferCreateInfo(sType=12, pNext=None, flags=0, size=len(payload),
                                usage=VK_BUFFER_USAGE_STORAGE_BUFFER_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT,
                                sharingMode=0, queueFamilyIndexCount=0, pQueueFamilyIndices=None)
        vkbuf = ctypes.c_void_p()
        rc = lib.vkCreateBuffer(dev, ctypes.byref(bi), None, ctypes.byref(vkbuf))
        if rc != VK_SUCCESS:
            return {"ok": False, "reason": "vkCreateBuffer rc=%d" % rc, "device": props.deviceName.decode(
                "utf-8", "replace").strip("\x00")}
        buf = vkbuf
        req = VkMemoryRequirements()
        lib.vkGetBufferMemoryRequirements(dev, buf, ctypes.byref(req))
        mp = VkPhysicalDeviceMemoryProperties()
        lib.vkGetPhysicalDeviceMemoryProperties(phys, ctypes.byref(mp))
        want = VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT
        chosen = None
        for t in range(mp.memoryTypeCount):
            mt = mp.memoryTypes[t]
            if (req.memoryTypeBits & (1 << t)) and (mt.propertyFlags & want) == want:
                chosen = (t, mt)
                break
        if chosen is None:
            return {"ok": False, "reason": "no host-visible+coherent memory type in the buffer's %d-bit mask"
                                           % req.memoryTypeBits}
        t_index, mt = chosen
        ai = VkMemoryAllocateInfo(sType=5, pNext=None, allocationSize=req.size, memoryTypeIndex=t_index)
        mem = ctypes.c_void_p()
        rc = lib.vkAllocateMemory(dev, ctypes.byref(ai), None, ctypes.byref(mem))
        if rc != VK_SUCCESS:
            return {"ok": False, "reason": "vkAllocateMemory rc=%d (%d bytes)" % (rc, req.size)}
        rc = lib.vkBindBufferMemory(dev, buf, mem, 0)
        if rc != VK_SUCCESS:
            return {"ok": False, "reason": "vkBindBufferMemory rc=%d" % rc}
        rc = lib.vkMapMemory(dev, mem, 0, req.size, 0, ctypes.byref(ptr))
        if rc != VK_SUCCESS:
            return {"ok": False, "reason": "vkMapMemory rc=%d" % rc}

        times = []
        for _ in range(max(1, writes)):
            t0 = time.perf_counter()
            ctypes.memmove(ptr, payload, len(payload))
            times.append((time.perf_counter() - t0) * 1e6)
        back = ctypes.string_at(ptr, len(payload))
        times.sort()
        lib.vkUnmapMemory(dev, mem)
        return {
            "ok": True,
            "device": props.deviceName.decode("utf-8", "replace").strip("\x00"),
            "device_index": idx,
            "device_type": TYPE_NAMES.get(props.deviceType, "?"),
            "queue_family_index": fam,
            "bytes": len(payload),
            "allocation_bytes": req.size,
            "memory_type_index": t_index,
            "memory_type_properties": "HOST_VISIBLE|HOST_COHERENT (0x%x)" % mt.propertyFlags,
            "heap_index": mt.heapIndex,
            "host_heap_bytes": mp.memoryHeaps[mt.heapIndex].size,
            "writes": max(1, writes),
            "write_us_min": round(times[0], 4),
            "write_us_median": round(times[len(times) // 2], 4),
            "write_us_max": round(times[-1], 4),
            "verified": back == payload,
            "payload_sha256": hashlib.sha256(payload).hexdigest()[:16],
            "readback_sha256": hashlib.sha256(back).hexdigest()[:16],
            "basis": "ctypes on vulkan-1.dll: create buffer -> driver memory requirements -> host-visible+"
                     "coherent allocation -> bind -> map -> memmove -> read back the SAME mapping",
            "limits": ["a mapped host-coherent buffer is device-VISIBLE memory, not the GPU's Infinity Cache, "
                       "and this does not claim a shader consumed the bytes",
                       "the read-back goes through the CPU's own mapping; it proves the write reached the buffer "
                       "the driver returned, not that the GPU read it"],
        }
    finally:
        if mem:
            lib.vkFreeMemory(dev, mem, None)
        if buf:
            lib.vkDestroyBuffer(dev, buf, None)
        if dev:
            lib.vkDestroyDevice(dev, None)
        lib.vkDestroyInstance(inst, None)


IC_LINE_BYTES = 128


def default_payload():
    """One Infinity Cache line, self-describing: byte i = i, so a wrong or partial write is visible."""
    return bytes(i % 256 for i in range(IC_LINE_BYTES))


def capability_report(payload=None, persist=True, artifact="swarm_v2_artifacts/vulkan_capability.json"):
    """The whole measured picture: devices, the device chosen, and a verified write with its timing range."""
    devs = enumerate_devices()
    rep = {"protocol": "VULKAN_CAPABILITY", "devices": devs, "chosen_device": None, "write": None,
           "basis": "measured on this host at call time; vulkaninfo.exe is the independent cross-check",
           "limits": ["device_local_bytes is the driver's report for THIS machine, not the 96MB design target "
                      "in GPU_SPECS"]}
    if devs.get("ok"):
        chosen = next((d for d in devs["devices"] if d["type"] == "DISCRETE_GPU"),
                      devs["devices"][0] if devs["devices"] else None)
        rep["chosen_device"] = chosen
        rep["write"] = write_to_device(payload or default_payload(),
                                       None if chosen is None else chosen["index"])
    if persist:
        try:
            with open(artifact, "w") as f:
                json.dump(rep, f, indent=1)
        except OSError as e:
            rep["persist_error"] = str(e)
    return rep


if __name__ == "__main__":
    r = capability_report()
    print(json.dumps(r, indent=1)[:2000])
