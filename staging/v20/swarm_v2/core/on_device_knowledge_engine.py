import os
import sys
import json
import time
import ctypes
from typing import List, Dict, Any, Optional

_BASE_DIR = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DLL_PATH = os.path.join(_BASE_DIR, "swarm_v2_rust_kernel", "target", "release", "swarm_rust_kernel.dll")
INDEX_PATH = os.path.join(_BASE_DIR, "swarm_v2_artifacts", "kernel_knowledge_index.json")

DIM = 768
MONAD_TRUTH_ANCHOR = 0x4D4F4E4144000001

class CActionVector(ctypes.Structure):
    _fields_ = [
        ("agent_id", ctypes.c_uint8),
        ("intent_id", ctypes.c_uint8),
        ("confidence", ctypes.c_float),
        ("k3_trit", ctypes.c_int8)
    ]

_INSTANCE = None

class OnDeviceKnowledgeEngine:
    '''Kernel-resident knowledge and voltage doctrine engine.'''

    def __init__(self):
        self.dll_loaded = False
        self.corpus_loaded = False
        self.keys: List[str] = []
        self.corpus_buf = None
        self.num_docs = 0

        self._bind_dll()
        self._load_index()

    def _bind_dll(self):
        if not os.path.exists(DLL_PATH):
            return

        try:
            self.lib = ctypes.CDLL(DLL_PATH)

            # 1. Monad Truth Anchor
            self.lib.rust_kernel_verify.restype = ctypes.c_uint64

            # 2. Native Model-Free Text Ternary Encoding
            self.lib.rust_native_encode_text_ternary.argtypes = [
                ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
                ctypes.POINTER(ctypes.c_int8), ctypes.c_size_t,
                ctypes.c_float, ctypes.POINTER(ctypes.c_float)
            ]
            self.lib.rust_native_encode_text_ternary.restype = ctypes.c_int32

            # 3. Native Ternary Index Search
            self.lib.rust_ternary_index_search.argtypes = [
                ctypes.POINTER(ctypes.c_int8), ctypes.c_size_t,
                ctypes.POINTER(ctypes.c_int8), ctypes.c_size_t, ctypes.c_size_t,
                ctypes.POINTER(ctypes.c_size_t), ctypes.POINTER(ctypes.c_int32),
                ctypes.POINTER(ctypes.c_size_t)
            ]
            self.lib.rust_ternary_index_search.restype = ctypes.c_int32

            # 4. Voltage Node Eval (Tokenless ActionVector)
            self.lib.rust_voltage_node_eval.argtypes = [
                ctypes.c_size_t,
                ctypes.POINTER(ctypes.c_float),
                ctypes.POINTER(CActionVector)
            ]
            self.lib.rust_voltage_node_eval.restype = ctypes.c_int32

            # 5. Council Evaluate
            self.lib.rust_council_evaluate.argtypes = [
                ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t,
                ctypes.POINTER(ctypes.c_float), ctypes.POINTER(ctypes.c_uint64)
            ]
            self.lib.rust_council_evaluate.restype = ctypes.c_int32

            # 6. K3 Logic
            self.lib.rust_k3_and.argtypes = [ctypes.c_int8, ctypes.c_int8]
            self.lib.rust_k3_and.restype = ctypes.c_int8
            self.lib.rust_k3_or.argtypes = [ctypes.c_int8, ctypes.c_int8]
            self.lib.rust_k3_or.restype = ctypes.c_int8
            self.lib.rust_k3_not.argtypes = [ctypes.c_int8]
            self.lib.rust_k3_not.restype = ctypes.c_int8

            self.dll_loaded = True
        except Exception as e:
            self.dll_loaded = False
            self.dll_error = str(e)

    def _load_index(self):
        if not os.path.exists(INDEX_PATH):
            return

        try:
            with open(INDEX_PATH, "r", encoding="utf-8") as f:
                raw = json.load(f)
            idx_map = raw.get("index", raw)
            self.keys = list(idx_map.keys())
            self.num_docs = len(self.keys)

            corpus_trits = []
            for k in self.keys:
                emb = idx_map[k].get("embedding", [])
                if len(emb) != DIM:
                    emb = (emb + [0.0] * DIM)[:DIM]
                c_trits = [(1 if x > 0.02 else (-1 if x < -0.02 else 0)) for x in emb]
                corpus_trits.extend(c_trits)

            self.corpus_buf = (ctypes.c_int8 * len(corpus_trits))(*corpus_trits)
            self.corpus_loaded = True
        except Exception as e:
            self.corpus_loaded = False
            self.index_error = str(e)

    def search_on_device_knowledge(self, query_text: str, top_k: int = 3) -> Dict[str, Any]:
        t0 = time.perf_counter_ns()
        if not self.dll_loaded or not self.corpus_loaded or not query_text:
            return {
                "status": "UNAVAILABLE",
                "hits": [],
                "latency_us": 0.0,
                "model_free": True
            }

        q_bytes = query_text.encode("utf-8", errors="replace")
        q_buf = (ctypes.c_uint8 * len(q_bytes))(*q_bytes)
        out_trits = (ctypes.c_int8 * DIM)()
        scale = ctypes.c_float(0.0)

        t_enc_start = time.perf_counter_ns()
        self.lib.rust_native_encode_text_ternary(
            q_buf, len(q_bytes), out_trits, DIM, 1.0, ctypes.byref(scale)
        )
        enc_us = (time.perf_counter_ns() - t_enc_start) / 1000.0

        k_val = min(top_k, self.num_docs)
        out_idx = (ctypes.c_size_t * k_val)()
        out_score = (ctypes.c_int32 * k_val)()
        comps = ctypes.c_size_t(0)

        t_search_start = time.perf_counter_ns()
        rc = self.lib.rust_ternary_index_search(
            out_trits, DIM, self.corpus_buf, self.num_docs, k_val,
            out_idx, out_score, ctypes.byref(comps)
        )
        search_us = (time.perf_counter_ns() - t_search_start) / 1000.0
        total_us = (time.perf_counter_ns() - t0) / 1000.0

        hits = []
        for i in range(max(0, rc)):
            hit_i = out_idx[i]
            if hit_i < len(self.keys):
                hits.append({
                    "document": self.keys[hit_i],
                    "rank": i + 1,
                    "agreement_score": int(out_score[i]),
                    "dimension": DIM
                })

        return {
            "status": "SUCCESS",
            "hits": hits,
            "top_match": hits[0]["document"] if hits else None,
            "encode_latency_us": round(enc_us, 2),
            "search_latency_us": round(search_us, 2),
            "total_latency_us": round(total_us, 2),
            "comparisons": comps.value,
            "on_device_corpus_size": self.num_docs,
            "model_free": True
        }

    def dispatch_voltage_action(self, signal_text_or_vector: Any) -> Dict[str, Any]:
        t0 = time.perf_counter_ns()
        if not self.dll_loaded:
            return {"status": "UNAVAILABLE", "error": "DLL unavailable"}

        import numpy as np
        if isinstance(signal_text_or_vector, str):
            seed = abs(hash(signal_text_or_vector)) % (2**32)
            rng = np.random.RandomState(seed)
            signal = rng.randn(DIM).astype(np.float32)
        elif hasattr(signal_text_or_vector, "__len__"):
            signal = np.array(signal_text_or_vector, dtype=np.float32)
            if len(signal) < DIM:
                signal = np.pad(signal, (0, DIM - len(signal)))
            elif len(signal) > DIM:
                signal = signal[:DIM]
        else:
            signal = np.zeros(DIM, dtype=np.float32)

        signal_ptr = signal.ctypes.data_as(ctypes.POINTER(ctypes.c_float))
        action = CActionVector()

        ret = self.lib.rust_voltage_node_eval(DIM, signal_ptr, ctypes.byref(action))
        elapsed_us = (time.perf_counter_ns() - t0) / 1000.0

        agent_names = [
            "Hermes", "Jarvis", "Bridge", "Shield", "Logic", "Devo", "Pulse",
            "Flow", "EventLedger", "Orchestra", "THzGateway", "Verify", "Seeker", "Archi"
        ]
        intent_names = ["READ", "WRITE", "EXECUTE", "RENDER"]

        agent_name = agent_names[action.agent_id % len(agent_names)]
        intent_name = intent_names[action.intent_id % len(intent_names)]

        return {
            "status": "PASS",
            "agent": agent_name,
            "agent_id": action.agent_id,
            "intent": intent_name,
            "intent_id": action.intent_id,
            "confidence": round(float(action.confidence), 4),
            "k3_trit": action.k3_trit,
            "k3_label": {1: "TRUE", 0: "UNKNOWN", -1: "FALSE"}.get(action.k3_trit, "UNKNOWN"),
            "dispatch_latency_us": round(elapsed_us, 2),
            "mode": "TOKENLESS_VOLTAGE_TRANSFORM",
            "phi_v_carrier_thz": 15.53
        }

    def evaluate_council(self, topic: str) -> Dict[str, Any]:
        t0 = time.perf_counter_ns()
        if not self.dll_loaded:
            return {"status": "UNAVAILABLE", "consensus": False}

        topic_bytes = topic.encode("utf-8")
        buf = (ctypes.c_ubyte * len(topic_bytes))(*topic_bytes)
        score = ctypes.c_float(0.0)
        seal = ctypes.c_uint64(0)

        rc = self.lib.rust_council_evaluate(
            buf, len(topic_bytes), ctypes.byref(score), ctypes.byref(seal)
        )
        elapsed_us = (time.perf_counter_ns() - t0) / 1000.0

        return {
            "status": "SUCCESS",
            "consensus_reached": rc == 1,
            "weighted_score": round(score.value, 4),
            "monad_seal": hex(seal.value),
            "latency_us": round(elapsed_us, 2)
        }

def get_on_device_knowledge_engine() -> OnDeviceKnowledgeEngine:
    global _INSTANCE
    if _INSTANCE is None:
        _INSTANCE = OnDeviceKnowledgeEngine()
    return _INSTANCE
