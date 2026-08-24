"""
Triggers the live Swarm OS v14 server to run a spatial mesh simulation.
This ensures the live React dashboard populates with telemetry data.
"""

import urllib.request
import logging

logging.basicConfig(level=logging.INFO, format="%(message)s")

try:
    req = urllib.request.Request("http://localhost:8021/swarm/spatial/simulate", method="POST")
    with urllib.request.urlopen(req) as response:
        result = response.read().decode('utf-8')
        logging.info(f"✅ Simulation successfully triggered on live server: {result}")
except Exception as e:
    logging.error(f"❌ Failed to trigger simulation. Is the Swarm Server (app_v2.py) running? Error: {e}")
