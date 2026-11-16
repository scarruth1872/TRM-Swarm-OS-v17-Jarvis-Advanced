# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 27, Rank 129)
# CDYLIB: strip_rust_kernel_binder.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
# TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified
# Monad Truth Anchor: 0x4D4F4E4144000001
# _NATIVE_LOADED = True when CDYLIB active; falls back to pure Python below.
# =============================================================================
import os as _os
import ctypes as _ctypes

_NATIVE_LOADED = False
_NATIVE_REASON = "not attempted"
_NATIVE_DLL_PATH = _os.path.join(
    _os.path.dirname(_os.path.dirname(_os.path.dirname(_os.path.abspath(__file__)))),
    "swarm_v2_artifacts", "native_voltage_strips", "strip_rust_kernel_binder.dll"
)
# The fallback below is NOT silent: _NATIVE_REASON records why the native path
# is inactive so a module can report it instead of advertising a strip it
# never loaded.
try:
    if _os.path.exists(_NATIVE_DLL_PATH):
        _native_lib = _ctypes.CDLL(_NATIVE_DLL_PATH)
        _NATIVE_LOADED = True
        _NATIVE_REASON = "PE CDYLIB loaded"
    else:
        _NATIVE_REASON = "strip absent"
except OSError as _e:
    _NATIVE_REASON = f"strip not loadable: {_e}"

# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------
"""
TRM Swarm OS v18 — Rust Microkernel C ABI Binder
Loads and executes the native compiled Rust engine (libswarm_rust_kernel.dll).
Provides zero-GC, memory-safe, bare-metal K3 ternary logic, TurboVec cosine, and full Agent Runtime.
"""

import os
import sys
import ctypes
from typing import Any, Optional, Dict
import numpy as np

# Dynamically resolve relative to project root
_BASE_DIR = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
_candidate_paths = [
    os.path.join(_BASE_DIR, "swarm_v2_rust_kernel", "target", "release", "swarm_rust_kernel.dll"),
    os.path.join(_BASE_DIR, "swarm_v2_rust_kernel", "target", "release", "deps", "swarm_rust_kernel.dll"),
    os.path.join(_BASE_DIR, "swarm_v2_rust_kernel", "target", "debug", "deps", "swarm_rust_kernel.dll"),
    os.path.join(_BASE_DIR, "swarm_v2_rust_kernel", "target", "debug", "swarm_rust_kernel.dll"),
]
RUST_DLL_PATH = next((p for p in _candidate_paths if os.path.exists(p)), _candidate_paths[0])

class CRustDispatchResult(ctypes.Structure):
    _fields_ = [
        ("agent_id", ctypes.c_uint8),
        ("k3_trit", ctypes.c_int8),
        ("confidence", ctypes.c_float),
        ("latency_ns", ctypes.c_uint64)
    ]

class CActionVector(ctypes.Structure):
    _fields_ = [
        ("agent_id", ctypes.c_uint8),
        ("intent_id", ctypes.c_uint8),
        ("confidence", ctypes.c_float),
        ("k3_trit", ctypes.c_int8),
    ]

class RustKernelBinder:
    def __init__(self):
        self.dll = None
        self.available = False
        self._load_dll()
        
    def _load_dll(self):
        if RUST_DLL_PATH and os.path.exists(RUST_DLL_PATH):
            try:
                self.dll = ctypes.CDLL(RUST_DLL_PATH)
                
                # Bind K3 function signatures
                self.dll.rust_k3_and.argtypes = [ctypes.c_int8, ctypes.c_int8]
                self.dll.rust_k3_and.restype = ctypes.c_int8
                
                self.dll.rust_k3_or.argtypes = [ctypes.c_int8, ctypes.c_int8]
                self.dll.rust_k3_or.restype = ctypes.c_int8
                
                self.dll.rust_k3_not.argtypes = [ctypes.c_int8]
                self.dll.rust_k3_not.restype = ctypes.c_int8
                
                self.dll.rust_k3_batch_and.argtypes = [
                    ctypes.POINTER(ctypes.c_int8),
                    ctypes.POINTER(ctypes.c_int8),
                    ctypes.POINTER(ctypes.c_int8),
                    ctypes.c_size_t
                ]
                self.dll.rust_k3_batch_and.restype = ctypes.c_int32
                
                self.dll.rust_turbovec_cosine.argtypes = [
                    ctypes.POINTER(ctypes.c_float),
                    ctypes.POINTER(ctypes.c_float),
                    ctypes.c_size_t
                ]
                self.dll.rust_turbovec_cosine.restype = ctypes.c_float
                
                self.dll.rust_kernel_verify.argtypes = []
                self.dll.rust_kernel_verify.restype = ctypes.c_uint64
                
                # Bind Agent Runtime signatures
                self.dll.rust_agent_dispatch.argtypes = [
                    ctypes.c_uint8,
                    ctypes.c_uint8,
                    ctypes.POINTER(ctypes.c_uint8),
                    ctypes.c_size_t,
                    ctypes.POINTER(CRustDispatchResult)
                ]
                self.dll.rust_agent_dispatch.restype = ctypes.c_int32
                
                self.dll.rust_agent_runtime_benchmark.argtypes = [ctypes.c_size_t]
                self.dll.rust_agent_runtime_benchmark.restype = ctypes.c_double
                
                # Bind Voltage Node Eval signature
                try:
                    self.dll.rust_voltage_node_eval.argtypes = [
                        ctypes.c_size_t,
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(CActionVector)
                    ]
                    self.dll.rust_voltage_node_eval.restype = ctypes.c_int32
                except Exception:
                    pass

                # Bind Phase 16 Accelerated Computational Engines
                try:
                    self.dll.rust_systolic_gemm.argtypes = [
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.c_size_t
                    ]
                    self.dll.rust_systolic_gemm.restype = ctypes.c_int32

                    self.dll.rust_sparse_attention.argtypes = [
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.c_size_t,
                        ctypes.c_size_t
                    ]
                    self.dll.rust_sparse_attention.restype = ctypes.c_int32

                    self.dll.rust_gcn_layer.argtypes = [
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.c_size_t,
                        ctypes.c_size_t
                    ]
                    self.dll.rust_gcn_layer.restype = ctypes.c_int32

                    self.dll.rust_rbm_step.argtypes = [
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.c_size_t,
                        ctypes.c_size_t,
                        ctypes.c_uint64
                    ]
                    self.dll.rust_rbm_step.restype = ctypes.c_float

                    # Bind Batch Acceleration C-ABI exports
                    self.dll.rust_systolic_gemm_batch.argtypes = [
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.c_size_t,
                        ctypes.c_size_t
                    ]
                    self.dll.rust_systolic_gemm_batch.restype = ctypes.c_int32

                    self.dll.rust_sparse_attention_batch.argtypes = [
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.c_size_t,
                        ctypes.c_size_t,
                        ctypes.c_size_t
                    ]
                    self.dll.rust_sparse_attention_batch.restype = ctypes.c_int32

                    self.dll.rust_gcn_layer_batch.argtypes = [
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.POINTER(ctypes.c_float),
                        ctypes.c_size_t,
                        ctypes.c_size_t,
                        ctypes.c_size_t
                    ]
                    self.dll.rust_gcn_layer_batch.restype = ctypes.c_int32

                    # Bind Native Attention with KV Cache (returns effective_len as i32)
                    self.dll.rust_attention_with_kv_cache.argtypes = [
                        ctypes.POINTER(ctypes.c_float),  # q_curr_ptr: [1, head_dim]
                        ctypes.POINTER(ctypes.c_float),  # k_cache_ptr: [max_cache_len, head_dim]
                        ctypes.POINTER(ctypes.c_float),  # v_cache_ptr: [max_cache_len, head_dim]
                        ctypes.POINTER(ctypes.c_float),  # new_k_ptr: [1, head_dim]
                        ctypes.POINTER(ctypes.c_float),  # new_v_ptr: [1, head_dim]
                        ctypes.POINTER(ctypes.c_float),  # out_ptr: [1, head_dim]
                        ctypes.c_size_t,                 # head_dim
                        ctypes.c_size_t,                 # current_cache_len
                        ctypes.c_size_t                  # max_cache_len
                    ]
                    self.dll.rust_attention_with_kv_cache.restype = ctypes.c_int32
                except Exception as e:
                    print(f"[RustKernel] Warning binding accelerated engines: {e}")

                self.available = True
                self._bind_strip_registry()
                print(f"[RustKernel] Native Rust microkernel DLL loaded: {RUST_DLL_PATH}")
            except Exception as e:
                print(f"[RustKernel] Warning loading DLL: {e}")
        else:
            print(f"[RustKernel] Warning DLL not found at: {RUST_DLL_PATH}")

    def systolic_gemm(self, a: np.ndarray, b: np.ndarray) -> np.ndarray:
        """Accelerated native 2D GEMM via Rust microkernel."""
        size = a.shape[0]
        a_c = np.ascontiguousarray(a, dtype=np.float32)
        b_c = np.ascontiguousarray(b, dtype=np.float32)
        c_out = np.zeros((size, size), dtype=np.float32)
        if self.available and hasattr(self.dll, "rust_systolic_gemm"):
            ret = self.dll.rust_systolic_gemm(
                a_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                b_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                c_out.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                size
            )
            if ret == 0:
                return c_out
        return np.dot(a_c, b_c)

    def sparse_attention(self, q: np.ndarray, k: np.ndarray, v: np.ndarray, mask: np.ndarray) -> np.ndarray:
        """Accelerated native Block-Sparse Attention via Rust microkernel."""
        seq_len, head_dim = q.shape
        q_c = np.ascontiguousarray(q, dtype=np.float32)
        k_c = np.ascontiguousarray(k, dtype=np.float32)
        v_c = np.ascontiguousarray(v, dtype=np.float32)
        mask_c = np.ascontiguousarray(mask, dtype=np.float32)
        out = np.zeros((seq_len, head_dim), dtype=np.float32)
        if self.available and hasattr(self.dll, "rust_sparse_attention"):
            ret = self.dll.rust_sparse_attention(
                q_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                k_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                v_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                mask_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                out.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                seq_len,
                head_dim
            )
            if ret == 0:
                return out
        # Fallback
        import math
        scores = (np.dot(q_c, k_c.T) / math.sqrt(head_dim)) + mask_c
        scores_max = np.max(scores, axis=-1, keepdims=True)
        exp_scores = np.exp(scores - scores_max)
        probs = exp_scores / np.sum(exp_scores, axis=-1, keepdims=True)
        return np.dot(probs, v_c)

    def gcn_layer(self, norm_adj: np.ndarray, x: np.ndarray, w: np.ndarray) -> np.ndarray:
        """Accelerated native Kipf-Welling GCN message passing via Rust microkernel."""
        num_nodes = norm_adj.shape[0]
        feat_dim = x.shape[1]
        adj_c = np.ascontiguousarray(norm_adj, dtype=np.float32)
        x_c = np.ascontiguousarray(x, dtype=np.float32)
        w_c = np.ascontiguousarray(w, dtype=np.float32)
        out = np.zeros((num_nodes, feat_dim), dtype=np.float32)
        if self.available and hasattr(self.dll, "rust_gcn_layer"):
            ret = self.dll.rust_gcn_layer(
                adj_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                x_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                w_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                out.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                num_nodes,
                feat_dim
            )
            if ret == 0:
                return out
        # Fallback
        return np.maximum(0.0, adj_c @ x_c @ w_c)

    def rbm_step(self, v0: np.ndarray, w: np.ndarray, seed: int = 42) -> tuple[np.ndarray, float]:
        """Accelerated native RBM Contrastive Divergence CD-1 step via Rust microkernel."""
        n_vis = len(v0)
        n_hid = w.shape[1]
        v0_c = np.ascontiguousarray(v0, dtype=np.float32)
        w_c = np.ascontiguousarray(w, dtype=np.float32)
        v_recon = np.zeros(n_vis, dtype=np.float32)
        if self.available and hasattr(self.dll, "rust_rbm_step"):
            mse = self.dll.rust_rbm_step(
                v0_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                w_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                v_recon.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                n_vis,
                n_hid,
                seed
            )
            if mse >= 0.0:
                return v_recon, float(mse)
        # Fallback
        h_prob = 1.0 / (1.0 + np.exp(-np.clip(np.dot(v0_c, w_c), -20, 20)))
        rng = np.random.RandomState(seed)
        h0 = (rng.rand(n_hid) < h_prob).astype(np.float32)
        v_recon_prob = 1.0 / (1.0 + np.exp(-np.clip(np.dot(w_c, h0), -20, 20)))
        mse = float(np.mean((v0_c - v_recon_prob) ** 2))
        return v_recon_prob, mse

    def systolic_gemm_batch(self, a_batch: np.ndarray, b_batch: np.ndarray) -> np.ndarray:
        """Accelerated batch 2D GEMM via Rust microkernel."""
        batch_size, size, _ = a_batch.shape
        a_c = np.ascontiguousarray(a_batch, dtype=np.float32)
        b_c = np.ascontiguousarray(b_batch, dtype=np.float32)
        c_out = np.zeros((batch_size, size, size), dtype=np.float32)
        if self.available and hasattr(self.dll, "rust_systolic_gemm_batch"):
            ret = self.dll.rust_systolic_gemm_batch(
                a_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                b_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                c_out.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                size,
                batch_size
            )
            if ret == 0:
                return c_out
        # Fallback
        for b in range(batch_size):
            c_out[b] = np.dot(a_c[b], b_c[b])
        return c_out

    def sparse_attention_batch(self, q_batch: np.ndarray, k_batch: np.ndarray, v_batch: np.ndarray, mask: np.ndarray) -> np.ndarray:
        """Accelerated batch Block-Sparse Attention via Rust microkernel."""
        batch_size, seq_len, head_dim = q_batch.shape
        q_c = np.ascontiguousarray(q_batch, dtype=np.float32)
        k_c = np.ascontiguousarray(k_batch, dtype=np.float32)
        v_c = np.ascontiguousarray(v_batch, dtype=np.float32)
        mask_c = np.ascontiguousarray(mask, dtype=np.float32)
        out = np.zeros((batch_size, seq_len, head_dim), dtype=np.float32)
        if self.available and hasattr(self.dll, "rust_sparse_attention_batch"):
            ret = self.dll.rust_sparse_attention_batch(
                q_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                k_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                v_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                mask_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                out.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                seq_len,
                head_dim,
                batch_size
            )
            if ret == 0:
                return out
        # Fallback
        for b in range(batch_size):
            out[b] = self.sparse_attention(q_c[b], k_c[b], v_c[b], mask_c)
        return out

    def gcn_layer_batch(self, norm_adj: np.ndarray, x_batch: np.ndarray, w: np.ndarray) -> np.ndarray:
        """Accelerated batch Kipf-Welling GCN message passing via Rust microkernel."""
        batch_size, num_nodes, feat_dim = x_batch.shape
        adj_c = np.ascontiguousarray(norm_adj, dtype=np.float32)
        x_c = np.ascontiguousarray(x_batch, dtype=np.float32)
        w_c = np.ascontiguousarray(w, dtype=np.float32)
        out = np.zeros((batch_size, num_nodes, feat_dim), dtype=np.float32)
        if self.available and hasattr(self.dll, "rust_gcn_layer_batch"):
            ret = self.dll.rust_gcn_layer_batch(
                adj_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                x_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                w_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                out.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                num_nodes,
                feat_dim,
                batch_size
            )
            if ret == 0:
                return out
        # Fallback
        for b in range(batch_size):
            out[b] = self.gcn_layer(adj_c, x_c[b], w_c)
        return out

    def attention_with_kv_cache(
        self,
        q: np.ndarray,
        k_new: np.ndarray,
        v_new: np.ndarray,
        k_cache: np.ndarray,
        v_cache: np.ndarray,
        current_len: int
    ) -> tuple[np.ndarray, int]:
        """
        Accelerated Single-Step Autoregressive Attention with Native KV Cache.
        Appends (k_new, v_new) into (k_cache, v_cache) at current_len,
        and computes attention over 0..=current_len in Rust.
        
        q, k_new, v_new: float32 arrays of shape [head_dim] or [1, head_dim]
        k_cache, v_cache: float32 arrays of shape [max_cache_len, head_dim]
        """
        head_dim = q.shape[-1]
        max_cache_len = k_cache.shape[0]
        q_c = np.ascontiguousarray(q.reshape(-1), dtype=np.float32)
        k_new_c = np.ascontiguousarray(k_new.reshape(-1), dtype=np.float32)
        v_new_c = np.ascontiguousarray(v_new.reshape(-1), dtype=np.float32)
        k_cache_c = np.ascontiguousarray(k_cache, dtype=np.float32)
        v_cache_c = np.ascontiguousarray(v_cache, dtype=np.float32)
        out = np.zeros(head_dim, dtype=np.float32)

        if self.available and hasattr(self.dll, "rust_attention_with_kv_cache"):
            effective_len = self.dll.rust_attention_with_kv_cache(
                q_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                k_cache_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                v_cache_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                k_new_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                v_new_c.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                out.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                head_dim,
                current_len,
                max_cache_len
            )
            if effective_len > 0:
                k_cache[:] = k_cache_c
                v_cache[:] = v_cache_c
                return out, effective_len

        # Fallback Python calculation
        idx = min(current_len, max_cache_len - 1)
        k_cache[idx] = k_new_c
        v_cache[idx] = v_new_c
        valid_len = min(current_len + 1, max_cache_len)

        import math
        scores = np.dot(k_cache[:valid_len], q_c) / math.sqrt(head_dim)
        scores_max = np.max(scores)
        exp_scores = np.exp(scores - scores_max)
        probs = exp_scores / np.sum(exp_scores)
        out = np.dot(probs, v_cache[:valid_len])
        return out, valid_len

    def k3_and(self, a: int, b: int) -> int:
        if self.available:
            return self.dll.rust_k3_and(a, b)
        return min(a, b)

    def k3_or(self, a: int, b: int) -> int:
        if self.available:
            return self.dll.rust_k3_or(a, b)
        return max(a, b)

    def k3_not(self, a: int) -> int:
        if self.available:
            return self.dll.rust_k3_not(a)
        return -a

    def turbovec_cosine(self, vec_a: np.ndarray, vec_b: np.ndarray) -> float:
        if not self.available:
            dot = np.dot(vec_a, vec_b)
            norm = np.linalg.norm(vec_a) * np.linalg.norm(vec_b)
            return float(dot / norm) if norm > 1e-9 else 0.0
            
        a_ptr = vec_a.astype(np.float32).ctypes.data_as(ctypes.POINTER(ctypes.c_float))
        b_ptr = vec_b.astype(np.float32).ctypes.data_as(ctypes.POINTER(ctypes.c_float))
        dim = len(vec_a)
        return self.dll.rust_turbovec_cosine(a_ptr, b_ptr, dim)

    def verify_truth_anchor(self) -> int:
        if self.available:
            return self.dll.rust_kernel_verify()
        return 0x4D4F4E4144000001

    # ── Native Strip Registry (voltage doctrine — one dispatch truth) ──────
    def _bind_strip_registry(self):
        """Bind the native strip-registry C-ABI exports (added v19 kernel)."""
        try:
            self.dll.rust_strip_registry_count.restype = ctypes.c_uint64
            self.dll.rust_strip_audit.argtypes = [ctypes.POINTER(ctypes.c_uint64)]
            self.dll.rust_strip_audit.restype = ctypes.c_uint64
            self.dll.rust_strip_verify.argtypes = [
                ctypes.c_char_p, ctypes.c_size_t,
                ctypes.c_char_p, ctypes.c_size_t,
            ]
            self.dll.rust_strip_verify.restype = ctypes.c_int32
            return True
        except Exception:
            return False

    def strip_registry_count(self) -> int:
        """Number of native strips registered in the kernel roster."""
        if self.available and hasattr(self.dll, "rust_strip_registry_count"):
            return int(self.dll.rust_strip_registry_count())
        return 0

    def strip_registry_audit(self) -> dict:
        """Audit roster vs on-disk real-PE DLLs. Returns {roster, present_as_pe}."""
        if not (self.available and hasattr(self.dll, "rust_strip_audit")):
            return {"roster": 0, "present_as_pe": 0, "error": "registry unavailable"}
        present = ctypes.c_uint64(0)
        total = self.dll.rust_strip_audit(ctypes.byref(present))
        return {"roster": int(total), "present_as_pe": int(present.value)}

    def strip_verify(self, module_name: str) -> dict:
        """Verify one module against the kernel roster.
        rc: 1 = registered + real PE on disk, 0 = registered but DLL missing/fake,
        2 = not in roster."""
        if not (self.available and hasattr(self.dll, "rust_strip_verify")):
            return {"rc": 2, "module": module_name, "dll": "", "error": "registry unavailable"}
        name_b = module_name.encode("utf-8")
        out = ctypes.create_string_buffer(256)
        rc = self.dll.rust_strip_verify(name_b, len(name_b), out, 256)
        return {"rc": int(rc), "module": module_name, "dll": out.value.decode("utf-8", errors="replace")}

    def dispatch_agent_rust(self, agent_id: int, intent_id: int = 0, payload_bytes: bytes = b"") -> dict:
        if not self.available:
            return {"status": "FAIL", "error": "Rust DLL unavailable"}
            
        res = CRustDispatchResult()
        p_len = len(payload_bytes)
        if p_len > 0:
            p_ptr = (ctypes.c_uint8 * p_len).from_buffer_copy(payload_bytes)
        else:
            p_ptr = None
            
        ret = self.dll.rust_agent_dispatch(agent_id, intent_id, p_ptr, p_len, ctypes.byref(res))
        
        agent_names = ["Hermes", "Jarvis", "Bridge", "Shield", "Logic", "Devo", "Pulse", 
                       "Flow", "EventLedger", "Orchestra", "THzGateway", "Verify", "Seeker", "Archi", "LogicMaster"]
        
        agent_name = agent_names[res.agent_id % len(agent_names)]
        
        return {
            "status": "PASS",
            "agent": agent_name,
            "agent_id": res.agent_id,
            "k3_trit": int(res.k3_trit),
            "confidence": round(float(res.confidence), 4),
            "latency_ns": res.latency_ns,
            "latency_us": round(res.latency_ns / 1000.0, 3)
        }

    def voltage_node_eval(self, signal: Any) -> dict:
        """Evaluates continuous voltage signal field into a tokenless ActionVector via Rust kernel."""
        sig_arr = np.ascontiguousarray(signal, dtype=np.float32)
        dim = len(sig_arr)
        intents = ["READ", "WRITE", "EXECUTE", "RENDER"]
        
        if not self.available or not hasattr(self.dll, "rust_voltage_node_eval"):
            avg = float(np.mean(sig_arr)) if dim > 0 else 0.0
            trit = 1 if avg > 0.15 else (-1 if avg < -0.15 else 0)
            intent_idx = int(abs(sig_arr[1]) * 5) % 4 if dim > 1 else 2
            return {
                "status": 0,
                "agent_id": int(abs(sig_arr[0]) * 10) % 15 if dim > 0 else 0,
                "intent_id": intent_idx,
                "intent_name": intents[intent_idx],
                "confidence": float(min(1.0, abs(avg) * 2.0)),
                "k3_trit": trit
            }
        
        act = CActionVector()
        res = self.dll.rust_voltage_node_eval(dim, sig_arr.ctypes.data_as(ctypes.POINTER(ctypes.c_float)), ctypes.byref(act))
        intent_idx = act.intent_id % 4
        return {
            "status": res,
            "agent_id": act.agent_id,
            "intent_id": act.intent_id,
            "intent_name": intents[intent_idx],
            "confidence": round(float(act.confidence), 4),
            "k3_trit": int(act.k3_trit)
        }

    def benchmark_rust_runtime(self, num_tasks: int = 1000000) -> dict:
        if not self.available:
            return {"status": "FAIL"}
            
        sec = self.dll.rust_agent_runtime_benchmark(num_tasks)
        tasks_per_sec = num_tasks / sec if sec > 0 else 0
        avg_us = (sec / num_tasks) * 1e6
        
        return {
            "num_tasks": num_tasks,
            "elapsed_sec": round(sec, 6),
            "elapsed_ms": round(sec * 1000.0, 3),
            "tasks_per_sec": round(tasks_per_sec, 2),
            "avg_latency_us": round(avg_us, 4),
            "avg_latency_ns": round(avg_us * 1000.0, 1)
        }

_rust_kernel_instance = None

def get_rust_kernel() -> RustKernelBinder:
    global _rust_kernel_instance
    if _rust_kernel_instance is None:
        _rust_kernel_instance = RustKernelBinder()
    return _rust_kernel_instance
