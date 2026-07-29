
import torch
import torch.nn as nn
import sys

# Monkey patch to find out who is accessing Buffer
class TraceNN(object):
    def __init__(self, original_nn):
        self.original_nn = original_nn
    
    def __getattr__(self, name):
        if name == 'Buffer':
            import traceback
            print("!!! Access to nn.Buffer detected !!!")
            traceback.print_stack()
            # Return something to avoid immediate crash
            return None
        return getattr(self.original_nn, name)

# This might not work because nn is a module, not a class instance we can easily wrap
# But we can try to replace it in sys.modules

import torch.nn as nn
sys.modules['torch.nn.dummy'] = nn

# A better way: just add the attribute and see if it's used
# But we want to know WHO uses it.

# Let's try to load the model and see where it fails
from swarm_v2.core.trm_brain import TRMBrain
try:
    brain = TRMBrain()
except Exception as e:
    import traceback
    traceback.print_exc()
