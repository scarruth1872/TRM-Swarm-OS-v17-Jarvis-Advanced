import os
import json
import re
import time
import logging
import asyncio
from typing import Dict, List, Optional
from datetime import datetime

logger = logging.getLogger(__name__)

class SkillValidator:
    """
    SkillValidator implementing the 'audited_secv4' compliance protocol.
    Performs periodic scans on learned skills, audits their schemas,
    detects drift/corruption, and self-heals corrupted manifests.
    """
    def __init__(self, manifest_dir: str = "swarm_v2_learned_skills"):
        self.manifest_dir = manifest_dir
        self.manifest_path = os.path.join(self.manifest_dir, "manifest.json")
        self.backup_path = os.path.join(self.manifest_dir, "manifest.backup.json")
        self._spatial_broadcaster = None
        self._audit_log: List[dict] = []
        self._active: bool = False

    def bind_broadcaster(self, spatial_broadcaster):
        self._spatial_broadcaster = spatial_broadcaster

    def verify_skill_manifest(self) -> dict:
        """
        Audit the manifest file against audited_secv4 schema contract rules.
        """
        audit_result = {
            "status": "PASS",
            "errors": [],
            "healed": False,
            "timestamp": datetime.now().isoformat()
        }

        if not os.path.exists(self.manifest_path):
            # Attempt to self-heal using backup
            if os.path.exists(self.backup_path):
                try:
                    with open(self.backup_path, "r", encoding="utf-8") as bf:
                        data = bf.read()
                    with open(self.manifest_path, "w", encoding="utf-8") as mf:
                        mf.write(data)
                    audit_result["healed"] = True
                    logger.warning("[Secv4 Audit] Manifest was missing. Successfully self-healed from backup.")
                except Exception as e:
                    audit_result["status"] = "FAIL"
                    audit_result["errors"].append(f"Missing manifest and self-healing backup failed: {e}")
                    return audit_result
            else:
                # Recreate empty valid manifest
                try:
                    empty_manifest = {
                        "version": "1.0",
                        "updated_at": datetime.now().isoformat(),
                        "skills": []
                    }
                    with open(self.manifest_path, "w", encoding="utf-8") as f:
                        json.dump(empty_manifest, f, indent=2)
                    audit_result["healed"] = True
                    logger.warning("[Secv4 Audit] Manifest missing and no backup found. Created clean default state.")
                except Exception as e:
                    audit_result["status"] = "FAIL"
                    audit_result["errors"].append(f"Failed to create empty manifest: {e}")
                    return audit_result

        # Parse JSON and validate integrity
        try:
            with open(self.manifest_path, "r", encoding="utf-8") as f:
                data = json.load(f)
        except json.JSONDecodeError as jde:
            audit_result["errors"].append(f"JSON Syntax Corruption: {jde}")
            # Try to restore from backup
            if os.path.exists(self.backup_path):
                try:
                    with open(self.backup_path, "r", encoding="utf-8") as bf:
                        backup_data = json.load(bf)
                    with open(self.manifest_path, "w", encoding="utf-8") as mf:
                        json.dump(backup_data, mf, indent=2)
                    audit_result["healed"] = True
                    logger.warning("[Secv4 Audit] Manifest was corrupted. Successfully restored from backup.")
                    data = backup_data
                except Exception as e:
                    audit_result["status"] = "FAIL"
                    audit_result["errors"].append(f"Self-healing restore failed: {e}")
                    return audit_result
            else:
                audit_result["status"] = "FAIL"
                return audit_result
        except Exception as e:
            audit_result["status"] = "FAIL"
            audit_result["errors"].append(f"Read error: {e}")
            return audit_result

        # Validate internal skill records
        skills = data.get("skills", [])
        valid_skills = []
        drift_detected = False

        for skill in skills:
            skill_errors = []
            name = skill.get("skill_name")
            desc = skill.get("description")
            inst = skill.get("instructions")

            if not name:
                skill_errors.append("Missing skill_name")
            elif not re.match(r"^[a-zA-Z0-9_]+$", name):
                skill_errors.append(f"Invalid characters in name: {name}")

            if not desc:
                skill_errors.append("Missing description field")
            if not inst:
                skill_errors.append("Missing instructions usage guidelines")

            if skill_errors:
                drift_detected = True
                audit_result["errors"].append(f"Skill '{name or 'Unknown'}' contract drift: {', '.join(skill_errors)}")
                # Skip invalid skill to self-heal manifest list (remove corrupted records)
                logger.warning(f"[Secv4 Audit] Drift detected in skill '{name}'. Pruning record to maintain compliance.")
            else:
                valid_skills.append(skill)

        if drift_detected:
            # Self-heal manifest by saving only compliant records
            try:
                data["skills"] = valid_skills
                data["updated_at"] = datetime.now().isoformat()
                with open(self.manifest_path, "w", encoding="utf-8") as f:
                    json.dump(data, f, indent=2)
                audit_result["healed"] = True
                logger.info("[Secv4 Audit] Self-healed drift by writing compliant entries to disk.")
            except Exception as e:
                audit_result["errors"].append(f"Self-heal write failed: {e}")

        # Create periodic backups when valid
        if audit_result["status"] == "PASS" and not audit_result["errors"] and not audit_result["healed"]:
            try:
                with open(self.backup_path, "w", encoding="utf-8") as bf:
                    json.dump(data, bf, indent=2)
            except Exception:
                pass

        if audit_result["errors"]:
            audit_result["status"] = "WARNING" if audit_result["healed"] else "FAIL"
        elif audit_result["healed"]:
            audit_result["status"] = "WARNING"

        self._audit_log.append(audit_result)
        if len(self._audit_log) > 50:
            self._audit_log.pop(0)

        # Broadcast compliance result
        if self._spatial_broadcaster:
            try:
                self._spatial_broadcaster.push_event({
                    "event_type": "compliance_audit",
                    "status": audit_result["status"],
                    "healed": audit_result["healed"],
                    "errors_count": len(audit_result["errors"]),
                    "timestamp": time.time()
                })
            except Exception:
                pass

        return audit_result

    async def start_validation_loop(self, interval: int = 30):
        """Autonomic loop running periodic contract compliance audits."""
        self._active = True
        logger.info("[Secv4 Audit] Compliance background scanner activated.")
        while self._active:
            try:
                await asyncio.sleep(interval)
                self.verify_skill_manifest()
            except Exception as e:
                logger.error(f"[Secv4 Audit] Error in audit scan loop: {e}")

    async def stop(self):
        self._active = False

    def get_audit_log(self) -> List[dict]:
        return self._audit_log

_instance = None

def get_skill_validator() -> SkillValidator:
    global _instance
    if _instance is None:
        _instance = SkillValidator()
    return _instance
