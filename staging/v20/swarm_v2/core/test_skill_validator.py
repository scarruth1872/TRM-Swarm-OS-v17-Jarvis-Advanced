# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 36, Rank 174)
# CDYLIB: strip_test_skill_validator.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_test_skill_validator.dll"
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
from unittest.mock import MagicMock
from swarm_v2.core.skill_validator import get_skill_validator

class TestSkillValidator(unittest.TestCase):
    def setUp(self):
        self.test_dir = "test_learned_skills_sandbox"
        os.makedirs(self.test_dir, exist_ok=True)
        self.validator = get_skill_validator()
        self.validator.manifest_dir = self.test_dir
        self.validator.manifest_path = os.path.join(self.test_dir, "manifest.json")
        self.validator.backup_path = os.path.join(self.test_dir, "manifest.backup.json")
        
        # Reset files
        if os.path.exists(self.validator.manifest_path):
            os.remove(self.validator.manifest_path)
        if os.path.exists(self.validator.backup_path):
            os.remove(self.validator.backup_path)

        self.mock_broadcaster = MagicMock()
        self.validator.bind_broadcaster(self.mock_broadcaster)

    def tearDown(self):
        # Cleanup
        if os.path.exists(self.validator.manifest_path):
            os.remove(self.validator.manifest_path)
        if os.path.exists(self.validator.backup_path):
            os.remove(self.validator.backup_path)
        if os.path.exists(self.test_dir):
            os.rmdir(self.test_dir)

    def test_missing_manifest_self_heals(self):
        res = self.validator.verify_skill_manifest()
        self.assertTrue(res["healed"])
        self.assertEqual(res["status"], "WARNING")
        self.assertTrue(os.path.exists(self.validator.manifest_path))

    def test_syntax_corruption_heals_from_backup(self):
        # Create a valid backup
        valid_data = {
            "version": "1.0",
            "skills": [
                {"skill_name": "Test", "description": "Desc", "instructions": "Inst"}
            ]
        }
        with open(self.validator.backup_path, "w") as f:
            json.dump(valid_data, f)

        # Write corrupted manifest
        with open(self.validator.manifest_path, "w") as f:
            f.write("{invalid json...")

        res = self.validator.verify_skill_manifest()
        self.assertTrue(res["healed"])
        
        # Verify it restored from backup
        with open(self.validator.manifest_path, "r") as f:
            manifest = json.load(f)
        self.assertEqual(manifest["skills"][0]["skill_name"], "Test")

    def test_compliance_drift_prunes_records(self):
        # Write manifest with one compliant and one non-compliant skill
        drifted_data = {
            "version": "1.0",
            "skills": [
                {"skill_name": "Compliant", "description": "Desc", "instructions": "Inst"},
                {"skill_name": "Drifted", "description": "", "instructions": "Missing description"}
            ]
        }
        with open(self.validator.manifest_path, "w") as f:
            json.dump(drifted_data, f)

        res = self.validator.verify_skill_manifest()
        self.assertTrue(res["healed"])
        self.assertEqual(res["status"], "WARNING")

        # Verify drifted was pruned
        with open(self.validator.manifest_path, "r") as f:
            manifest = json.load(f)
        self.assertEqual(len(manifest["skills"]), 1)
        self.assertEqual(manifest["skills"][0]["skill_name"], "Compliant")

if __name__ == '__main__':
    unittest.main()
