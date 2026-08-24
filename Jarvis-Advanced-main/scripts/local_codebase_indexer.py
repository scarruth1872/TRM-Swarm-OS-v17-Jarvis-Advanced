import ast
import os
import json
from pathlib import Path

class CodebaseIndexer:
    def __init__(self, root_dir):
        self.root_dir = Path(root_dir)
        self.index = {"classes": [], "functions": []}

    def scan(self):
        """Crawls the directory for .py and .ts files."""
        for path in self.root_dir.rglob('*'):
            # Basic filter to skip hidden dirs and node_modules
            if any(part.startswith(('.', 'node_modules')) for part in path.parts):
                continue
            if path.suffix in ['.py', '.ts']:
                self._parse_file(path)
        return self.index

    def _parse_file(self, file_path):
        """Uses AST to extract structural metadata."""
        try:
            content = file_path.read_text(encoding='utf-8')
            if file_path.suffix == '.py':
                tree = ast.parse(content)
                for node in ast.walk(tree):
                    if isinstance(node, ast.ClassDef):
                        self.index["classes"].append({
                            "name": node.name,
                            "file": str(file_path.name),
                            "methods": [m.name for m in node.body if isinstance(m, ast.FunctionDef)]
                        })
                    elif isinstance(node, ast.FunctionDef):
                        self.index["functions"].append({
                            "name": node.name,
                            "file": str(file_path.name),
                            "args": [arg.arg for arg in node.args.args]
                        })
        except Exception:
            pass

# Initialize indexer for local execution
indexer = CodebaseIndexer("./")
print(json.dumps(indexer.scan(), indent=2))
