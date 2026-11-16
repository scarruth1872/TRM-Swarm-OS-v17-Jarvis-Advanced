# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 37, Rank 176)
# CDYLIB: strip_test_secrets_vault_api.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_test_secrets_vault_api.dll"
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
import unittest
import os
import json
from fastapi.testclient import TestClient
from swarm_v2.app_v2 import app
from swarm_v2.core.secrets_vault import get_secrets_vault
from swarm_v2.core.ddr_antibody import get_ddr

import pytest


@pytest.mark.integration
class TestSecretsVaultAPI(unittest.TestCase):
    def setUp(self):
        self.client = TestClient(app)
        self.vault = get_secrets_vault()
        self.test_vault_path = "test_secrets.vault"
        self.test_key_path = "test_secrets.key"
        self.orig_vault_path = self.vault.vault_path
        self.orig_key_path = self.vault.key_path
        
        self.vault.vault_path = self.test_vault_path
        self.vault.key_path = self.test_key_path
        self.vault._secrets = {}
        if os.path.exists(self.test_vault_path):
            os.remove(self.test_vault_path)
        if os.path.exists(self.test_key_path):
            os.remove(self.test_key_path)

    def tearDown(self):
        self.vault.vault_path = self.orig_vault_path
        self.vault.key_path = self.orig_key_path
        if os.path.exists(self.test_vault_path):
            os.remove(self.test_vault_path)
        if os.path.exists(self.test_key_path):
            os.remove(self.test_key_path)

    def test_set_and_get_secrets_api(self):
        res = self.client.post("/secrets/set", json={"key": "API_VAL", "value": "supersecret"})
        self.assertEqual(res.status_code, 200)
        self.assertEqual(res.json()["key"], "API_VAL")

        res_keys = self.client.get("/secrets/keys")
        self.assertEqual(res_keys.status_code, 200)
        self.assertIn("API_VAL", res_keys.json()["keys"])

        res_dec = self.client.post("/secrets/decrypt", json={"key": "API_VAL"})
        self.assertEqual(res_dec.status_code, 200)
        self.assertEqual(res_dec.json()["value"], "supersecret")

        res_del = self.client.delete("/secrets/delete/API_VAL")
        self.assertEqual(res_del.status_code, 200)

        res_keys_after = self.client.get("/secrets/keys")
        self.assertNotIn("API_VAL", res_keys_after.json()["keys"])

    def test_key_rotation_api(self):
        self.client.post("/secrets/set", json={"key": "API_VAL", "value": "supersecret"})
        
        res_rotate = self.client.post("/secrets/rotate")
        self.assertEqual(res_rotate.status_code, 200)

        res_dec = self.client.post("/secrets/decrypt", json={"key": "API_VAL"})
        self.assertEqual(res_dec.status_code, 200)
        self.assertEqual(res_dec.json()["value"], "supersecret")

    def test_add_antibody_api(self):
        ddr = get_ddr()
        orig_antibodies = list(ddr._antibodies)
        ddr._antibodies = []

        res = self.client.post("/ddr/antibodies/add", json={
            "error_type": "api_leak",
            "file_pattern": "*.py",
            "line_pattern": "leak_key",
            "fix_description": "Use env",
            "severity": "high"
        })
        self.assertEqual(res.status_code, 200)
        self.assertTrue(res.json()["antibody_id"].startswith("ab-"))

        self.assertEqual(len(ddr._antibodies), 1)
        self.assertEqual(ddr._antibodies[0].error_type, "api_leak")

        ddr._antibodies = orig_antibodies
        ddr._save()

if __name__ == '__main__':
    unittest.main()
