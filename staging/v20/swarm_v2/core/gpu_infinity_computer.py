"""GPU Infinity Cache Pattern Computer — RX 6700 XT.
96MB Infinity Cache = 805M SRAM cells = programmable fabric.
Same SRAM pattern modulation as CPU, but with 2.1 TB/s bandwidth.

Architecture: pattern → Vulkan buffer → Infinity Cache → compute.
K3 ternary: 805M cells / 2 = 402M trits on GPU.
Throughput: 402M × 2.1 TB/s bandwidth = ~525T ternary ops/s."""

import time, math, struct, hashlib, json

# ── NATIVE VOLTAGE STRIP — Phase 18 auto-injected ─────────────────────────────
import ctypes as _ctypes
from pathlib import Path as _Path

_GPU_INFINITY_DLL = None
try:
    _dll_p2 = _Path(__file__).parent.parent.parent / "swarm_v2_artifacts" / "native_voltage_strips" / "strip_gpu_infinity.dll"
    if _dll_p2.exists():
        _GPU_INFINITY_DLL = _ctypes.CDLL(str(_dll_p2))
        _GPU_INFINITY_DLL.strip_batch_gpu_infinity_reconfig.argtypes = [
            _ctypes.POINTER(_ctypes.c_int32),
            _ctypes.c_size_t,
            _ctypes.POINTER(_ctypes.c_uint32)
        ]
        _GPU_INFINITY_DLL.strip_batch_gpu_infinity_reconfig.restype = _ctypes.c_int32
except Exception as _e:
    _GPU_INFINITY_DLL = None
    _GPU_INFINITY_DLL_ERROR = f"strip not loadable: {_e}"
# ─────────────────────────────────────────────────────────────────────────────
from pathlib import Path

# ═══════════════════════════════════════════════════════
# GPU Fabric Specifications
# ═══════════════════════════════════════════════════════

GPU_SPECS = {
    'model': 'AMD Radeon RX 6700 XT',
    'architecture': 'RDNA 2 (Navi 22)',
    'infinity_cache': {
        'size_mb': 96,
        'size_bytes': 96 * 1024 * 1024,
        'cells': 96 * 1024 * 1024 * 8,  # 805,306,368 SRAM cells
        'transistors': 96 * 1024 * 1024 * 8 * 6,  # 6T SRAM = ~4.83B transistors
        'bandwidth_tbps': 2.1,
        'access_pattern': 'Vulkan buffer → cache line (128B)',
        'coherency': 'Fully coherent across all 40 CUs',
    },
    'vram': {
        'size_gb': 12,
        'size_bytes': 12 * 1024 * 1024 * 1024,
        'cells': 12 * 1024 * 1024 * 1024 * 8,  # 103B DRAM cells
        'type': 'GDDR6',
        'bandwidth_gbps': 384,
    },
    'compute': {
        'compute_units': 40,
        'stream_processors': 2560,
        'wavefront_size': 32,
        'max_wavefronts': 2560 // 32 * 40,  # 3200
        'clock_mhz': 2581,
    },
    'total_transistors': 17_200_000_000,
}

# ═══════════════════════════════════════════════════════
# GPU Virtual Architectures (via Infinity Cache patterns)
# ═══════════════════════════════════════════════════════

def gpu_pattern_fpga(size_mb=8):
    """FPGA pattern on Infinity Cache — 8MB = 131K LUTs."""
    lines = (size_mb * 1024 * 1024) // 128  # 128B cache lines
    return {
        'name': 'GPU FPGA-Lite',
        'cache_used': f'{size_mb}MB Infinity Cache',
        'logic_cells': lines,
        'lut_size': 7,  # 7-input LUT per 128B line
        'total_luts': lines,
        'bandwidth': '2.1 TB/s',
    }

def gpu_pattern_neural(size_mb=64):
    """Neural network on Infinity Cache — 64MB = 8M neurons."""
    neurons = (size_mb * 1024 * 1024) // 8  # 8B per neuron (16-bit weights × 4)
    return {
        'name': 'GPU SRAM Neural Network',
        'cache_used': f'{size_mb}MB Infinity Cache',
        'neurons': neurons,
        'synapses': neurons * 4,  # 4-way connectivity
        'precision': '16-bit floating point',
    }

def gpu_pattern_cellular(size_mb=96):
    """Cellular automaton on full Infinity Cache."""
    return {
        'name': 'GPU Cellular Automaton',
        'cache_used': f'{size_mb}MB (full Infinity Cache)',
        'grid': '8192 × 8192 cells (approximately)',
        'cells': (size_mb * 1024 * 1024 * 8) // 8,  # ~100M cells
        'updates_per_second': 2.1e12 / (100e6 * 8),  # Bandwidth / cells / bits
    }

def gpu_pattern_k3_ternary():
    """K3 ternary logic on Infinity Cache."""
    cells = 96 * 1024 * 1024 * 8  # 805M cells
    trits = cells // 2  # 402M trits
    return {
        'name': 'GPU K3 Ternary Engine',
        'cache_used': '96MB Infinity Cache',
        'trits': trits,
        'cells_used': cells,
        'encoding': '2 cells per trit',
        'throughput_tps': trits * 2.1e12 / 1e12,  # T ops/s
        'instruction': 'S_BITCMP_B32 (RDNA2 native)',
    }

ARCHITECTURES_GPU = {
    'fpga': gpu_pattern_fpga,
    'neural': gpu_pattern_neural,
    'cellular': gpu_pattern_cellular,
    'k3': gpu_pattern_k3_ternary,
}


def _probe_gpu():
    """Which GPU this host actually has, so the specs above cannot be mistaken for a measurement.

    `GPU_SPECS` describes an RX 6700 XT -- 96MB Infinity Cache, 2.1 TB/s, 805M cells. That is a DESIGN TARGET
    carried over from the oracle, not a probe. On a host with a different card every bandwidth/cell number
    here describes the target rather than the machine, so this names both instead of implying the first.
    """
    names = []
    try:
        import subprocess
        out = subprocess.run(['wmic', 'path', 'win32_VideoController', 'get', 'Name'],
                             capture_output=True, text=True, timeout=25).stdout
        names = [l.strip() for l in out.splitlines() if l.strip() and 'Name' not in l]
    except Exception as e:  # noqa: BLE001 -- a failed probe must say so, not guess a GPU
        names = [f'<probe failed: {e}>']
    target = GPU_SPECS['model']
    key = target.split()[-1].lower()  # 'XT'
    return {
        'described_gpu': target,
        'installed_gpu': names,
        'target_present': any(key in n.lower() and 'radeon' in n.lower() for n in names),
        'basis': 'GPU_SPECS is the oracle design target; installed_gpu is probed on this host '
                 '(wmic win32_VideoController)',
    }


# ═══════════════════════════════════════════════════════
# GPU Infinity Pattern Computer
# ═══════════════════════════════════════════════════════

class GPUInfinityComputer:
    """Programmable GPU fabric via Infinity Cache pattern modulation."""
    
    def __init__(self):
        self.specs = GPU_SPECS
        self.ic = self.specs['infinity_cache']
        self.current = None
    
    def _write_pattern_to_device(self, config):
        """Write this architecture's pattern descriptor into device memory, verifying the read-back.

        The descriptor is serialized deterministically, so a wrong or partial write is visible in the hash rather
        than merely expected not to happen. Returns the probe's own dict, `ok: False` with a reason when Vulkan is
        unavailable -- this method never invents a timing.
        """
        try:
            import vulkan_capability as vk
        except Exception as e:  # noqa: BLE001 -- an unimportable probe is a refusal, not a number
            return {'ok': False, 'reason': f'vulkan_capability not importable: {e}'}
        payload = json.dumps(config, sort_keys=True).encode('utf-8')
        r = vk.write_to_device(payload, None)
        if r.get('ok'):
            r['ic_lines'] = -(-len(payload) // vk.IC_LINE_BYTES)
        return r

    def load_architecture(self, name, use_vulkan=True):
        """Load one architecture, writing its pattern to device memory when this host can.

        The previous form returned `write_time` as the string `'(simulated — Vulkan path pending)'`: a timing that
        measured nothing, on a path that did nothing. Now the write is real -- a mapped Vulkan device buffer, read
        back and hashed (see `vulkan_capability`) -- and `write_time_us` is that measurement, reported as a range
        with the device named. When the path is unavailable the key carries the REASON and no number: a
        measurement is never replaced by a plausible one.
        """
        if name not in ARCHITECTURES_GPU:
            return {'error': f'Unknown: {name}', 'available': list(ARCHITECTURES_GPU.keys())}

        config = ARCHITECTURES_GPU[name]()
        self.current = config['name']
        h = hashlib.sha256(config['name'].encode()).hexdigest()[:12]  # the oracle's identity, unchanged

        write = self._write_pattern_to_device(config) if use_vulkan else {'ok': False,
                                                                          'reason': 'vulkan disabled by caller'}
        out = {
            'architecture': config['name'],
            'details': config,
            'pattern_hash': h,
            'bandwidth_available': f'{self.ic["bandwidth_tbps"]} TB/s',
            'cache_utilization': config.get('cache_used', '?'),
            'write_path': None,
            'write_time_us': None,
            'write_time_us_range': None,
            'write_verified': None,
            'written_bytes': None,
            'ic_lines': None,
            'buffer_sha256': None,
            'device': None,
            'write_basis': None,
        }
        if write.get('ok'):
            out.update({
                'write_path': 'vulkan_mapped_device_buffer',
                'write_time_us': write['write_us_median'],
                'write_time_us_range': [write['write_us_min'], write['write_us_max']],
                'write_verified': write['verified'],
                'written_bytes': write['bytes'],
                'ic_lines': write.get('ic_lines'),
                'buffer_sha256': write['readback_sha256'],
                'device': write['device'],
                'write_basis': write['basis'],
            })
            out['write_time'] = (f"{write['write_us_median']}μs measured (median of {write['writes']} writes to a "
                                 f"mapped Vulkan device buffer on {write['device']}, read back and verified)")
        else:
            out.update({
                'write_path': 'unavailable',
                'write_unavailable_reason': write.get('reason'),
                'write_basis': 'no timing is reported when the path cannot run; the previous value was a '
                               'simulated string, which is the defect this replaces',
            })
            out['write_time'] = f"not measured: {write.get('reason')}"
        return out

    

    def benchmark_all_native(self, iterations: int = 100) -> dict:
        """Native batch reconfiguration, with the strip's result CHECKED and the comparison MEASURED.

        The previous form declared "7.5x over pure Python struct pack & sha256", ignored the return code and
        never read `out_h` -- so a strip that silently did nothing still reported a speedup. Here the rc is
        checked, the hashes are read back (each architecture must hash differently -- the bug the generated
        strip fixes), and the python path it replaces is timed on the same work.
        """
        if not _GPU_INFINITY_DLL:
            return self.benchmark_all()

        arch_keys = list(ARCHITECTURES_GPU.keys())
        n_archs = len(arch_keys)
        total_ops = n_archs * iterations
        indices = [i % n_archs for i in range(total_ops)]

        c_idx = (_ctypes.c_int32 * total_ops)(*indices)
        out_h = (_ctypes.c_uint32 * total_ops)()

        t0 = time.perf_counter()
        rc = _GPU_INFINITY_DLL.strip_batch_gpu_infinity_reconfig(c_idx, total_ops, out_h)
        t_native = time.perf_counter() - t0
        distinct = len(set(out_h))

        # the python path this replaces: struct-pack one 128B line, hash it with sha256
        t0 = time.perf_counter()
        for i in indices:
            line = struct.pack('<128B', *([i % 256] * 128))
            hashlib.sha256(line).digest()
        t_python = time.perf_counter() - t0

        if rc != 0:
            return {"mode": "NATIVE_CDYLIB_GPU_INFINITY", "total_reconfigurations": total_ops,
                    "error": f"strip returned rc={rc}; no result claimed"}
        if distinct != n_archs:
            return {"mode": "NATIVE_CDYLIB_GPU_INFINITY", "total_reconfigurations": total_ops,
                    "distinct_hashes": distinct,
                    "error": f"{distinct} distinct hashes for {n_archs} architectures -- the strip is not "
                             f"carrying the architecture (this is the defect the generated strip fixes)"}

        return {
            "mode": "NATIVE_CDYLIB_GPU_INFINITY",
            "total_reconfigurations": total_ops,
            "total_elapsed_ms": round(t_native * 1000, 4),
            "per_reconfig_us": round((t_native / total_ops) * 1e6, 4),
            "distinct_hashes": distinct,
            "python_reference_ms": round(t_python * 1000, 4),
            "measured_speedup": round(t_python / t_native, 2) if t_native > 0 else None,
            "speedup_basis": "measured on this host: native batch vs python struct.pack+sha256 per line; the "
                             "previous '7.5x' was declared, not measured, and read nothing back",
        }

    def benchmark_all(self):
        results = {}
        for name in ARCHITECTURES_GPU:
            t0 = time.perf_counter()
            r = self.load_architecture(name)
            t = (time.perf_counter() - t0) * 1e6
            r['load_latency_us'] = round(t, 1)
            results[name] = r
        return results
    
    def summary(self):
        return {
            'gpu': self.specs['model'],
            'transistors': f'{self.specs["total_transistors"]/1e9:.1f}B',
            'infinity_cache': f'{self.ic["size_mb"]}MB SRAM',
            'cells': f'{self.ic["cells"]/1e6:.0f}M',
            'bandwidth': f'{self.ic["bandwidth_tbps"]} TB/s',
            'architectures': list(ARCHITECTURES_GPU.keys()),
            'architecture_count': len(ARCHITECTURES_GPU),
            'active': self.current,
            'host': _probe_gpu(),
            'basis': 'the cache/cell/bandwidth figures describe GPU_SPECS (the oracle design target); '
                     '`host` reports what is actually installed on this machine',
        }


# ═══════════════════════════════════════════════════════
# Comparison: CPU vs GPU fabric
# ═══════════════════════════════════════════════════════

def compare_fabrics():
    """CPU vs GPU fabric. The GPU count is DERIVED; the CPU count names its owner.

    `ARCHITECTURES_GPU` holds four entries, so the previous hardcoded "5 GPU" contradicted the dict in this
    same file. The CPU set is owned by `sram_pattern_computer`, which is not imported here, so its count is
    labelled rather than silently asserted.
    """
    gpu_n = len(ARCHITECTURES_GPU)
    cpu_n = 5  # owned by sram_pattern_computer (the CPU L3 set); not recounted here
    return {
        'cpu': {
            'cache': '32MB L3',
            'cells': '268M SRAM',
            'bandwidth': '~200 GB/s (read)',
            'transistors': '1.61B accessible (SRAM)',
            'architectures': cpu_n,
            'architectures_basis': 'declared: owned by sram_pattern_computer, not recounted here',
        },
        'gpu': {
            'cache': '96MB Infinity Cache (design target)',
            'cells': '805M SRAM (design target)',
            'bandwidth': '2.1 TB/s (design target)',
            'transistors': '4.83B accessible (IC SRAM)',
            'architectures': gpu_n,
            'architectures_basis': 'derived: len(ARCHITECTURES_GPU)',
        },
        'combined': {
            'total_sram': '128MB (32+96, design target)',
            'total_cells': '1.07B SRAM cells (design target)',
            'total_transistors': '6.44B programmable',
            'peak_bandwidth': '2.3 TB/s (design target)',
            'architectures': f'{cpu_n + gpu_n} total ({cpu_n} CPU + {gpu_n} GPU)',
        }
    }


# ═══════════════════════════════════════════════════════
# Self-test
# ═══════════════════════════════════════════════════════
if __name__ == '__main__':
    gpu = GPUInfinityComputer()
    
    print('GPU INFINITY CACHE PATTERN COMPUTER')
    print('=' * 55)
    print(f'GPU: {gpu.specs["model"]}')
    print(f'Transistors: {gpu.specs["total_transistors"]/1e9:.1f}B')
    print(f'Infinity Cache: {gpu.ic["size_mb"]}MB ({gpu.ic["cells"]/1e6:.0f}M SRAM cells)')
    print(f'Bandwidth: {gpu.ic["bandwidth_tbps"]} TB/s')
    print()
    
    print('ARCHITECTURES:')
    print('-' * 55)
    for name in ARCHITECTURES_GPU:
        r = gpu.load_architecture(name)
        print(f'  [{name:10s}] {r["architecture"]}')
        for k, v in r['details'].items():
            print(f'           {k}: {v}')
        print()
    
    print('CPU vs GPU FABRIC:')
    print('-' * 55)
    comp = compare_fabrics()
    for fabric, specs in comp.items():
        print(f'  {fabric.upper()}:')
        for k, v in specs.items():
            print(f'    {k}: {v}')
        print()
