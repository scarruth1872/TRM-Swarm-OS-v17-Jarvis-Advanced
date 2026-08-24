import os
import ast
import logging
from typing import Dict, List, Any

logger = logging.getLogger(__name__)

class CodebaseIndexer:
    """
    Crawls local codebase files and extracts class structures, function definitions,
    and docstrings using Python's abstract syntax trees (AST).
    """

    def __init__(self, root_dir: str):
        self.root_dir = root_dir

    def scan_files(self, limit: int = 20) -> List[Dict[str, Any]]:
        """Crawls directories recursively and gathers target files."""
        discovered = []
        target_dirs = ["swarm_v2/core", "swarm_v2/skills"]
        
        count = 0
        for tdir in target_dirs:
            full_path = os.path.join(self.root_dir, tdir)
            if not os.path.exists(full_path):
                continue
            
            for root, _, files in os.walk(full_path):
                for file in files:
                    if file.endswith(".py") and not file.startswith("test_"):
                        filepath = os.path.join(root, file)
                        discovered.append(filepath)
                        count += 1
                        if count >= limit:
                            break
                if count >= limit:
                    break
        return [self.parse_source_file(f) for f in discovered if os.path.exists(f)]

    def parse_source_file(self, filepath: str) -> Dict[str, Any]:
        """Parses a Python file using AST to extract functions, classes, and docs."""
        try:
            with open(filepath, "r", encoding="utf-8") as f:
                source = f.read()
            
            tree = ast.parse(source, filename=filepath)
            
            classes = []
            functions = []
            module_doc = ast.get_docstring(tree) or ""

            for node in tree.body:
                if isinstance(node, ast.ClassDef):
                    class_doc = ast.get_docstring(node) or ""
                    methods = []
                    for subnode in node.body:
                        if isinstance(subnode, ast.FunctionDef):
                            methods.append({
                                "name": subnode.name,
                                "args": [arg.arg for arg in subnode.args.args],
                                "docstring": ast.get_docstring(subnode) or ""
                            })
                    classes.append({
                        "name": node.name,
                        "docstring": class_doc,
                        "methods": methods
                    })
                elif isinstance(node, ast.FunctionDef):
                    functions.append({
                        "name": node.name,
                        "args": [arg.arg for arg in node.args.args],
                        "docstring": ast.get_docstring(node) or ""
                    })

            relative_path = os.path.relpath(filepath, self.root_dir)
            return {
                "filepath": relative_path,
                "module_doc": module_doc,
                "classes": classes,
                "functions": functions
            }

        except Exception as e:
            logger.error(f"Failed to parse source file AST {filepath}: {e}")
            return {
                "filepath": filepath,
                "error": str(e),
                "classes": [],
                "functions": []
            }
