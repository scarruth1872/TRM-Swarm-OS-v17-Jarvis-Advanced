#!/usr/bin/env python3
"""Cron entry point: heartbeat report from the native council server. No LLM, no network but loopback."""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import platform_report as pr

sys.argv = [sys.argv[0], "heartbeat"] + sys.argv[1:]
sys.exit(pr.main())
