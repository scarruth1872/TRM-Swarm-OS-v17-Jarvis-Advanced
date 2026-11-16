# The Vulkan path — from a placeholder to a measurement

**Block 149 companion.** `gpu_infinity_computer.load_architecture` reported its write as

```
'write_time': '(simulated — Vulkan path pending)'
```

a timing that measured nothing on a path that did nothing — and `hardware_voltage_microkernel_binder`'s header
promised "GPU VRAM (via Vulkan API)" while reading only nvidia-smi counters, with no VRAM and no Vulkan. Both are
now measured, on this host, through the loader that was already installed.

## What exists now

`swarm_v2/core/vulkan_capability.py` — one owner for Vulkan facts, `ctypes` on the system `vulkan-1.dll` (no new
dependency; `vulkaninfo.exe` stays the independent cross-check):

| call | what it establishes |
|---|---|
| `enumerate_devices()` | every physical device: name, type, vendor, api/driver version, and the **device-local** heap bytes the driver reports |
| `probe_properties_write_extent()` | how many bytes the driver actually writes into `VkPhysicalDeviceProperties` (measured 824) |
| `write_to_device(payload)` | create buffer → driver's own memory requirements → host-visible+coherent allocation → bind → map → write → **read back and compare** → timing distribution |
| `capability_report()` | the above, persisted to `swarm_v2_artifacts/vulkan_capability.json` |

## Measured on this host

```
[0] NVIDIA GeForce RTX 3070 Laptop GPU  DISCRETE_GPU   api 1.4.329  device-local 8.41 GB
[1] AMD Radeon(TM) Graphics             INTEGRATED_GPU api 1.3.260  device-local 0.54 GB

write: 128 B into a mapped device buffer on the RTX 3070, verified=True,
       payload sha == readback sha, 0.8 / 0.9 / 7.2 µs (min/median/max over 5 writes)
```

`load_architecture` now reports `write_path: 'vulkan_mapped_device_buffer'`, `write_time_us` (median), the range,
`write_verified`, the read-back hash, and the device it went to — for all four architectures. With the path
disabled it reports `write_path: 'unavailable'`, a reason, and **`None` for the timing**: no number is invented
where a measurement cannot be taken.

`hardware_voltage_microkernel_binder` gained `get_gpu_vram()` (delegated to the telemetry source, one owner):
this machine's **8.41 GB** on the RTX 3070 — not the header's declared "AMD RX 6700 XT (12GB VRAM)", which is now
labelled as the design target, as are its Ryzen 3900X rails and 4000 MHz denominator.

## Three ABI bugs, each found by measurement

1. **The truncated struct.** Declaring only the leading 276 bytes of `VkPhysicalDeviceProperties` does not
   truncate the driver's write — it **overruns the Python allocation**. It survived the enumerate call and
   segfaulted later in the write path, which is what made it look like a Vulkan problem. Oversizing the struct
   fixed it; the sentinel probe then measured the real extent at exactly **824 bytes**, so the padding is a fact
   rather than luck.
2. **A physical device is not a logical one.** `vkCreateBuffer` needs a `VkDevice`; passing the
   `VkPhysicalDevice` handle is the loader's `vkCreateBuffer: Invalid device`. The path now creates a logical
   device (queue family discovered from `vkGetPhysicalDeviceQueueFamilyProperties`) and destroys it.
3. **Two functions take the physical device, not the logical one** — `vkGetPhysicalDeviceMemoryProperties` is one
   of them, and the loader says so plainly once the call is dispatched correctly.

## Verified: 17 checks, 0 failed

`scratch/verify_vulkan_path.py` → `swarm_v2_artifacts/vulkan_path_ruler.json`. The parts that matter:

- **Two independent tools agree**: the ctypes device names equal `vulkaninfo --summary`'s, and the Vulkan
  device-local figure sits within 1% of `nvidia-smi`'s `memory.total` (8.41 GB vs 8.00 GB, ratio 0.979).
- The write's read-back hash equals the payload's, and the allocation comes from the driver's requirements.
- Timing is a distribution (`min ≤ median ≤ max`, all > 0), never a single figure.
- Refusals carry a reason and no number: empty payload, out-of-range device index, path disabled.
- The retired `'simulated — Vulkan path pending'` string is gone from the oracle's output.

## What this does NOT claim

- A mapped host-coherent buffer is **device-visible memory, not Infinity Cache**. The 96 MB IC in `GPU_SPECS` is
  a design target the driver does not expose as a memory type; the read-back goes through the CPU's own mapping,
  so it proves the bytes reached the buffer the driver returned, not that a shader consumed them.
- `GPU_SPECS` remains the RX 6700 XT target. The measured devices are this machine's two adapters.
- The kernel route reports these figures; **the kernel cannot run Vulkan** (a `no_std` image has no loader or
  driver), and `GET /native/gpu/architectures` says exactly that beside the numbers it relays.
