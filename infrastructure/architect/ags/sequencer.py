"""
Swarm OS v12 - Architecture Genome Sequencer (AGS)
Phase 9 component for self-evolving architecture.
Sequences the current system topology and code structure into a blueprint.
"""

import ast
import json
import logging
import os
from pathlib import Path
from typing import Dict, List

logger = logging.getLogger("AGS")

class GenomeSequencer:
    def __init__(self, root_dir: str):
        self.root_dir = Path(root_dir)
        self.genome = {"modules": {}, "dependencies": {}}

    def sequence(self) -> dict:
        logger.info(f"[AGS] Starting architecture sequencing from {self.root_dir}...")
        for filepath in self.root_dir.rglob("*.py"):
            if "venv" in str(filepath) or ".git" in str(filepath):
                continue
            
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    content = f.read()
                    
                tree = ast.parse(content)
                module_name = filepath.relative_to(self.root_dir).with_suffix('').as_posix().replace('/', '.')
                
                classes = [node.name for node in ast.walk(tree) if isinstance(node, ast.ClassDef)]
                functions = [node.name for node in ast.walk(tree) if isinstance(node, ast.FunctionDef)]
                imports = [node.names[0].name for node in ast.walk(tree) if isinstance(node, ast.Import)]
                from_imports = [node.module for node in ast.walk(tree) if isinstance(node, ast.ImportFrom) and node.module]
                
                self.genome["modules"][module_name] = {
                    "classes": classes,
                    "functions": functions
                }
                
                # Build dependency graph
                deps = imports + from_imports
                self.genome["dependencies"][module_name] = deps
                
            except Exception as e:
                logger.warning(f"[AGS] Failed to sequence {filepath}: {e}")
                
        logger.info(f"[AGS] Sequencing complete. Discovered {len(self.genome['modules'])} modules.")
        return self.genome

    def export(self, output_file: str = "system_genome.json"):
        with open(output_file, "w") as f:
            json.dump(self.genome, f, indent=2)
        logger.info(f"[AGS] Genome exported to {output_file}")

# Usage stub
if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    ags = GenomeSequencer(str(Path(__file__).parent.parent.parent.parent))
    genome = ags.sequence()
    ags.export()
