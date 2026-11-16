#!/usr/bin/env python3
"""Resident native federation bridge launcher (UDP 8022, HMAC trust).

The served op list comes from `native_federation_bridge.OP_LIST` — the single
source of truth — so this banner cannot advertise a stale dispatch surface.

NOTE: the bridge binds the port with a plain bind() (no SO_REUSEADDR). Only ONE
instance can hold UDP 8022; a second launch dies with WinError 10048 while the
old instance keeps serving. Kill every run_resident_bridge python process and
confirm the port is free BEFORE launching. Run `bridge_status.py` to check.
"""
import os
import sys
import time
import threading

ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
os.chdir(ROOT)
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

from swarm_v2.core.native_federation_bridge import NativeBridgeServer, OP_LIST  # noqa: E402

s = NativeBridgeServer()
threading.Thread(target=s.serve_loop, daemon=True).start()
print(f"BRIDGE RUNNING on UDP :8022 (ops: {'/'.join(OP_LIST)})", flush=True)
while True:
    time.sleep(10)
