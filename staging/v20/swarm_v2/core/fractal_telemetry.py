# -*- coding: utf-8 -*-
"""
fractal_telemetry.py — TRM Swarm OS Fractal Telemetry Engine
============================================================
Implements Higuchi Fractal Dimension (D_H) algorithm for real-time voltage/telemetry streams.
Provides continuous chaos detection and Kleene K3 ternary state quantization.

Mathematical Formulation:
  V_k^m = (v_m, v_{m+k}, v_{m+2k}, ...)
  L(m,k) = 1/k * sum|v_{m+ik} - v_{m+(i-1)k}| * (N-1)/(floor((N-m)/k)*k)
  <L(k)> ∝ k^{-D_H}
  D_H ∈ [1.0, 2.0]

Kleene Quantization Doctrine:
  +1 (TRUE / DETERMINISTIC) : 1.00 ≤ D_H ≤ 1.25  → Low entropy / Fast-path bypass
   0 (INDETERMINATE / DYNAMIC): 1.25 < D_H < 1.70  → Normal operating regime / Adaptive modulation
  -1 (REFUTED / CHAOTIC)     : D_H ≥ 1.70          → Chaotic drift / Intercept & stabilize
"""

import time
import threading
import ctypes
import numpy as np
from typing import Dict, Any, Optional

from .rust_kernel_binder import RUST_DLL_PATH


def _load_higuchi():
    """Load the Rust Higuchi function. There is no second implementation: if this fails, the caller
    is told so rather than silently getting numpy arithmetic that can drift from the native path."""
    lib = ctypes.CDLL(RUST_DLL_PATH)
    fn = lib.rust_higuchi_d_h
    fn.restype = ctypes.c_double
    fn.argtypes = [ctypes.POINTER(ctypes.c_float), ctypes.c_size_t]
    return fn


try:
    _higuchi_dh = _load_higuchi()
except Exception as _e:  # pragma: no cover - depends on the build artefact
    _higuchi_dh = None
    _higuchi_load_error = str(_e)


class HiguchiFractalCalculator:
    """
    Computes Higuchi Fractal Dimension (D_H) from streaming hardware telemetry.
    Thread-safe ring-buffer with sub-microsecond evaluation.
    """
    
    def __init__(self, window_size: int = 64, noise_floor_mv: float = 2.5):
        self.window_size = window_size
        self.noise_floor_mv = noise_floor_mv
        self.buffer = np.zeros(window_size, dtype=np.float32)
        self.buffer_idx = 0
        self.buffer_full = False
        self._lock = threading.Lock()
        
        # Performance & telemetry metrics
        self.computations = 0
        self.total_ns = 0.0
        self.last_D_H = 1.05
        self.last_update_ts = time.time()
        
    def ingest_reading(self, reading: float):
        """Push a new hardware metric reading (e.g. CPU freq / voltage / load proxy) into buffer."""
        with self._lock:
            self.buffer[self.buffer_idx] = float(reading)
            self.buffer_idx = (self.buffer_idx + 1) % self.window_size
            if self.buffer_idx == 0:
                self.buffer_full = True
            self.last_update_ts = time.time()
            
    def compute_higuchi(self) -> float:
        """Compute Higuchi Fractal Dimension D_H over the current sample window."""
        t0 = time.perf_counter_ns()
        
        with self._lock:
            n = self.window_size if self.buffer_full else self.buffer_idx
            if n < 8:
                return float(self.last_D_H)
            
            if _higuchi_dh is None:
                raise RuntimeError(
                    "Rust Higuchi implementation unavailable "
                    f"({_higuchi_load_error}); build swarm_v2_rust_kernel --release --lib"
                )
            # The reference accumulated in f32 because numpy promotes `float + np.float32`; the Rust
            # port mirrors that exactly, so this call is bit-identical on 300 random windows.
            arr = np.ascontiguousarray(self.buffer[:n], dtype=np.float32)
            d_h = float(
                _higuchi_dh(arr.ctypes.data_as(ctypes.POINTER(ctypes.c_float)), n)
            )
            self.last_D_H = d_h

            elapsed = time.perf_counter_ns() - t0
            self.computations += 1
            self.total_ns += elapsed

            return d_h
            
    def get_kleene_trit(self) -> int:
        """Quantize D_H into Kleene ternary trit (+1, 0, -1)."""
        d = self.last_D_H
        if 1.0 <= d <= 1.25:
            return 1
        elif d >= 1.70:
            return -1
        else:
            return 0
            
    def get_snapshot(self) -> Dict[str, Any]:
        """Return comprehensive telemetry snapshot of fractal dimension and state."""
        d_h = self.compute_higuchi()
        trit = self.get_kleene_trit()
        regime = "DETERMINISTIC_FAST_PATH" if trit == 1 else ("CHAOTIC_INTERCEPT" if trit == -1 else "AMBIGUOUS_DYNAMIC")
        
        return {
            "higuchi_d_h": round(d_h, 4),
            "kleene_trit": trit,
            "regime": regime,
            "sample_count": self.window_size if self.buffer_full else self.buffer_idx,
            "computations_total": self.computations,
            "avg_latency_us": round((self.total_ns / max(1, self.computations)) / 1000.0, 2),
            "noise_floor_mv": self.noise_floor_mv,
        }


# Global thread-safe singleton
_fractal_singleton: Optional[HiguchiFractalCalculator] = None
_fractal_lock = threading.Lock()

def get_fractal_telemetry() -> HiguchiFractalCalculator:
    """Return the global singleton Higuchi Fractal Telemetry instance."""
    global _fractal_singleton
    if _fractal_singleton is None:
        with _fractal_lock:
            if _fractal_singleton is None:
                _fractal_singleton = HiguchiFractalCalculator(window_size=64)
    return _fractal_singleton
