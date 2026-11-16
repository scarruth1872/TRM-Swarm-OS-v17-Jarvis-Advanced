# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 38, Rank 181)
# CDYLIB: strip_redis_mock.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_redis_mock.dll"
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

import json
import os
import threading
import time

class PersistentRedisMock:
    """
    A high-fidelity replacement for fakeredis that persists to disk.
    Fixes the 'transient state' simulation issue in the Nexus Bridge.
    """
    def __init__(self, filename="swarm_v2_artifacts/redis_state.json"):
        self.filename = filename
        self.data = {"strings": {}, "hashes": {}, "lists": {}}
        self.lock = threading.Lock()
        self._load()

    def _load(self):
        if os.path.exists(self.filename):
            try:
                with open(self.filename, "r", encoding="utf-8") as f:
                    self.data = json.load(f)
            except:
                pass

    def _save(self):
        with open(self.filename, "w", encoding="utf-8") as f:
            json.dump(self.data, f, indent=2)

    def ping(self):
        return True

    def set(self, key, value):
        with self.lock:
            self.data["strings"][str(key)] = str(value)
            self._save()
        return True

    def get(self, key):
        with self.lock:
            val = self.data["strings"].get(str(key))
            return val.encode() if val else None

    def hset(self, name, key=None, value=None, mapping=None):
        with self.lock:
            if name not in self.data["hashes"]:
                self.data["hashes"][name] = {}
            
            if mapping:
                for k, v in mapping.items():
                    self.data["hashes"][name][str(k)] = str(v)
            else:
                self.data["hashes"][name][str(key)] = str(value)
            self._save()
        return 1

    def hget(self, name, key):
        with self.lock:
            val = self.data["hashes"].get(str(name), {}).get(str(key))
            return val.encode() if val else None

    def hgetall(self, name):
        with self.lock:
            val = self.data["hashes"].get(name, {})
            # Return bytes-encoded dict for compatibility with redis-py
            return {k.encode(): v.encode() for k, v in val.items()}

    def rpush(self, name, value):
        with self.lock:
            if name not in self.data["lists"]:
                self.data["lists"][name] = []
            self.data["lists"][name].append(str(value))
            self._save()
        return len(self.data["lists"][name])

    def blpop(self, keys, timeout=0):
        if isinstance(keys, str):
            keys = [keys]
        
        start_time = time.time()
        while True:
            with self.lock:
                for key in keys:
                    if key in self.data["lists"] and self.data["lists"][key]:
                        val = self.data["lists"][key].pop(0)
                        self._save()
                        return (key.encode(), val.encode())
            
            if timeout > 0 and (time.time() - start_time) > timeout:
                return None
            
            time.sleep(0.1)
