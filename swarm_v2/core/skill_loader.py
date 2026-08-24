"""
SKILL.md Loader — Portable Expertise Format
Parses SKILL.md files with YAML frontmatter and registers them as
executable skill definitions. Compatible with Anti Gravity, Agent Zero,
and Claude Code SKILL.md standards.
"""

import os
import re
import logging
from typing import Dict, List, Any, Optional
from dataclasses import dataclass, field
from datetime import datetime

logger = logging.getLogger("SkillLoader")

SKILL_SEARCH_PATHS = [
    ".agent/skills",
    ".agents/skills",
    "_agent/skills",
    "swarm_v2/skills",
]


@dataclass
class SkillDefinition:
    """A parsed SKILL.md skill definition."""
    name: str
    description: str
    version: str = "1.0.0"
    instructions: str = ""
    prerequisites: List[str] = field(default_factory=list)
    tags: List[str] = field(default_factory=list)
    source_path: str = ""
    source_type: str = "skill_md"  # "skill_md" or "python_class"
    loaded_at: str = field(default_factory=lambda: datetime.now().isoformat())


class SkillLoader:
    """
    Loads and manages SKILL.md files and Python skill classes.

    Supports two formats:
    1. SKILL.md with YAML frontmatter (portable, standard)
    2. Python skill classes (existing swarm skills, wrapped with metadata)
    """

    def __init__(self, project_root: str = None):
        self.project_root = project_root or os.path.abspath(
            os.path.join(os.path.dirname(__file__), "..", "..")
        )
        self._skills: Dict[str, SkillDefinition] = {}
        self._auto_discover()

    def _auto_discover(self):
        """Auto-discover skills from search paths and Python modules."""
        # 1. Load SKILL.md files
        for rel_path in SKILL_SEARCH_PATHS:
            full_path = os.path.join(self.project_root, rel_path)
            if os.path.isdir(full_path):
                self._scan_directory(full_path)

        # 2. Load existing Python skill classes
        self._load_python_skills()

    def _scan_directory(self, directory: str):
        """Scan a directory for SKILL.md files."""
        for root, dirs, files in os.walk(directory):
            for fname in files:
                if fname.upper() == "SKILL.MD" or fname.endswith(".skill.md"):
                    path = os.path.join(root, fname)
                    self._parse_skill_md(path)

    def _parse_skill_md(self, path: str) -> Optional[SkillDefinition]:
        """Parse a SKILL.md file with YAML frontmatter."""
        try:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
        except Exception as e:
            logger.warning(f"Could not read {path}: {e}")
            return None

        # Extract YAML frontmatter between --- markers
        fm_match = re.match(r'^---\s*\n(.*?)\n---\s*\n(.*)', content, re.DOTALL)
        if fm_match:
            frontmatter_text = fm_match.group(1)
            instructions = fm_match.group(2).strip()
        else:
            # No frontmatter — treat entire file as instructions
            frontmatter_text = ""
            instructions = content.strip()

        # Parse YAML-like frontmatter (lightweight, no PyYAML dependency)
        fm = {}
        for line in frontmatter_text.split("\n"):
            line = line.strip()
            if ":" in line:
                key, _, value = line.partition(":")
                fm[key.strip().lower()] = value.strip().strip('"').strip("'")

        name = fm.get("name", os.path.basename(os.path.dirname(path)))
        skill = SkillDefinition(
            name=name,
            description=fm.get("description", ""),
            version=fm.get("version", "1.0.0"),
            instructions=instructions,
            prerequisites=[p.strip() for p in fm.get("prerequisites", "").split(",") if p.strip()],
            tags=[t.strip() for t in fm.get("tags", "").split(",") if t.strip()],
            source_path=path,
            source_type="skill_md",
        )

        self._skills[name] = skill
        logger.info(f"[SkillLoader] Loaded SKILL.md: {name} from {path}")
        return skill

    def _load_python_skills(self):
        """Dynamic discovery of Python skill classes in the skills directory."""
        skills_dir = os.path.join(self.project_root, "swarm_v2", "skills")
        if not os.path.isdir(skills_dir):
            return

        import importlib.util
        import inspect

        for fname in os.listdir(skills_dir):
            if fname.endswith("_skill.py") and fname != "base_skill.py":
                module_name = f"swarm_v2.skills.{fname[:-3]}"
                try:
                    # Import the module
                    spec = importlib.util.spec_from_file_location(module_name, os.path.join(skills_dir, fname))
                    if not spec or not spec.loader:
                        continue
                    module = importlib.util.module_from_spec(spec)
                    spec.loader.exec_module(module)

                    # Find all classes in the module
                    for name, obj in inspect.getmembers(module):
                        if inspect.isclass(obj) and name.endswith("Skill") and name != "BaseSkill":
                            # Check if it has the required metadata
                            skill_name = getattr(obj, "skill_name", name)
                            if skill_name not in self._skills:
                                self._skills[skill_name] = SkillDefinition(
                                    name=skill_name,
                                    description=getattr(obj, "description", ""),
                                    source_type="python_class",
                                    source_path=module_name,
                                    tags=["python", "builtin", "autonomous"],
                                )
                                logger.info(f"[SkillLoader] Dynamically loaded Python skill: {skill_name}")
                except Exception as e:
                    logger.debug(f"[SkillLoader] Failed to load module {module_name}: {e}")

    def load_skill(self, path: str) -> Optional[SkillDefinition]:
        """Load a specific SKILL.md file."""
        return self._parse_skill_md(path)

    def discover_skills(self) -> List[SkillDefinition]:
        """Return all discovered skills."""
        return list(self._skills.values())

    def register_skill(self, skill: SkillDefinition):
        """Manually register a skill definition."""
        self._skills[skill.name] = skill

    def get_skill(self, name: str) -> Optional[SkillDefinition]:
        """Retrieve a skill by name."""
        return self._skills.get(name)

    def get_skill_instructions(self, name: str) -> str:
        """Get the instructions text for a skill (for injection into agent prompts)."""
        skill = self._skills.get(name)
        if not skill:
            return ""
        return skill.instructions

    def list_skills(self) -> List[Dict[str, Any]]:
        """List all skills with metadata (for API/dashboard)."""
        return [{
            "name": s.name,
            "description": s.description,
            "version": s.version,
            "source_type": s.source_type,
            "tags": s.tags,
        } for s in self._skills.values()]

    def get_stats(self) -> Dict[str, Any]:
        """Get skill loader statistics."""
        return {
            "total_skills": len(self._skills),
            "skill_md_count": sum(1 for s in self._skills.values() if s.source_type == "skill_md"),
            "python_class_count": sum(1 for s in self._skills.values() if s.source_type == "python_class"),
            "search_paths": SKILL_SEARCH_PATHS,
        }


# Singleton
_loader: Optional[SkillLoader] = None

def get_skill_loader() -> SkillLoader:
    global _loader
    if _loader is None:
        _loader = SkillLoader()
    return _loader
