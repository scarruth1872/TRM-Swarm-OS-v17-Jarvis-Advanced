#!/usr/bin/env python3
"""Resident native federation bridge launcher (UDP 8022, HMAC trust).

Replaces the ad-hoc `python -c` one-liner. Ops served:
  drive_list, file_list, file_read, council, jarvis_chat,
  quantize_task, holograde, ping, verify_build, wetlab_dispatch
"""
import os
import sys
import time
import threading

ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
os.chdir(ROOT)
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

from swarm_v2.core.native_federation_bridge import NativeBridgeServer  # noqa: E402

s = NativeBridgeServer()
threading.Thread(target=s.serve_loop, daemon=True).start()
print("BRIDGE RUNNING on UDP :8022 (ops: drives/ls/cat/council/jarvis/"
      "quantize/holograde/ping/verify_build/wetlab_dispatch)", flush=True)
while True:
    time.sleep(10)
