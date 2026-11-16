import os
import json
import re
from pathlib import Path

class CodebaseIndexer:
    def __init__(self, root_dir):
        self.root_dir = Path(root_dir)
        self.index = {"classes": [], "functions": [], "interfaces": [], "types": []}

    def scan(self):
        """Crawls the directory for .py and .ts files."""
        for path in self.root_dir.rglob('*'):
            if path.suffix in ['.py', '.ts', '.tsx']:
                self._parse_file(path)
        return self.index

    def _parse_file(self, file_path):
        """Parses files for structural definitions."""
        try:
            content = file_path.read_text(encoding='utf-8')
            
            # Python parsing (simplified regex for broad discovery)
            if file_path.suffix == '.py':
                for match in re.finditer(r'class\s+(\w+)', content):
                    self.index["classes"].append({"name": match.group(1), "file": file_path.name})
                for match in re.finditer(r'def\s+(\w+)', content):
                    self.index["functions"].append({"name": match.group(1), "file": file_path.name})
            
            # TypeScript/TSX parsing
            elif file_path.suffix in ['.ts', '.tsx']:
                for match in re.finditer(r'interface\s+(\w+)', content):
                    self.index["interfaces"].append({"name": match.group(1), "file": file_path.name})
                for match in re.finditer(r'type\s+(\w+)', content):
                    self.index["types"].append({"name": match.group(1), "file": file_path.name})
                for match in re.finditer(r'function\s+(\w+)', content):
                    self.index["functions"].append({"name": match.group(1), "file": file_path.name})
                for match in re.finditer(r'class\s+(\w+)', content):
                    self.index["classes"].append({"name": match.group(1), "file": file_path.name})
                    
        except Exception as e:
            pass

if __name__ == "__main__":
    indexer = CodebaseIndexer(".")
    results = indexer.scan()
    print(json.dumps(results, indent=2))
