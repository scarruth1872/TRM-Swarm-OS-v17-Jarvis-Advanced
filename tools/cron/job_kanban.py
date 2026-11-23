"""Cron entry point: kanban board check (see kanban_check.py for what it refuses to do)."""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from kanban_check import main  # noqa: E402

if __name__ == "__main__":
    sys.exit(main())
