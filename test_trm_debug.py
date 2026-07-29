
import os
import sys
import torch

# Set PYTHONPATH to root
sys.path.append(os.getcwd())

from swarm_v2.core.trm_brain import TRMBrain

print("Testing TRM Brain initialization...")
try:
    brain = TRMBrain()
    if brain._model:
        print("SUCCESS: TRM Brain loaded.")
    else:
        print("FAILURE: TRM Brain model is None.")
except Exception as e:
    print(f"CRASH: {e}")
    import traceback
    traceback.print_exc()
