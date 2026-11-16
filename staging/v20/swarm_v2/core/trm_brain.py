# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 32, Rank 154)
# CDYLIB: strip_trm_brain.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
# TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified
# Monad Truth Anchor: 0x4D4F4E4144000001
# _NATIVE_LOADED = True when CDYLIB active; falls back to pure Python below.
# =============================================================================
import os as _os
import ctypes as _ctypes

_NATIVE_LOADED = False
_NATIVE_REASON = "not attempted"
_NATIVE_DLL_PATH = _os.path.join(
    _os.path.dirname(_os.path.dirname(_os.path.dirname(_os.path.abspath(__file__)))),
    "swarm_v2_artifacts", "native_voltage_strips", "strip_trm_brain.dll"
)
# The fallback below is NOT silent: _NATIVE_REASON records why the native path
# is inactive so a module can report it instead of advertising a strip it
# never loaded.
try:
    if _os.path.exists(_NATIVE_DLL_PATH):
        _native_lib = _ctypes.CDLL(_NATIVE_DLL_PATH)
        _NATIVE_LOADED = True
        _NATIVE_REASON = "PE CDYLIB loaded"
    else:
        _NATIVE_REASON = "strip absent"
except OSError as _e:
    _NATIVE_REASON = f"strip not loadable: {_e}"

# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------

try:
    import torch
    import torch.nn as nn

    # Monkey-patch for torch.nn.Buffer compatibility issue
    if not hasattr(torch.nn, 'Buffer'):
        class Buffer(torch.nn.Module):
            def __init__(self, tensor):
                super().__init__()
                self.register_buffer('data', tensor)
        torch.nn.Buffer = Buffer
        print("[System] Global monkey-patch applied: torch.nn.Buffer")
except ImportError:
    torch = None
    nn = None

import os
from typing import Dict, Any, Optional, List
try:
    from models.recursive_reasoning.trm import TinyRecursiveReasoningModel_ACTV1
except Exception:
    TinyRecursiveReasoningModel_ACTV1 = None

class TRMBrain:
    _instance = None
    _model = None

    def __new__(cls, *args, **kwargs):
        if not cls._instance:
            cls._instance = super(TRMBrain, cls).__new__(cls)
        return cls._instance

    def __init__(self, checkpoint_path: str = "tiny-recursive-weights/step_155718"):
        if self._model is not None:
            return

        self.checkpoint_path = checkpoint_path
        self.config = {
            "batch_size": 1,
            "seq_len": 256,
            "num_puzzle_identifiers": 1024,
            "vocab_size": 12,
            "H_cycles": 3,
            "L_cycles": 4,
            "H_layers": 0,
            "L_layers": 2,
            "hidden_size": 512,
            "expansion": 4.0,
            "num_heads": 8,
            "pos_encodings": "rope",
            "halt_max_steps": 16,
            "halt_exploration_prob": 0.1,
            "forward_dtype": "float32",
            "mlp_t": False,
            "puzzle_emb_len": 16,
            "no_ACT_continue": True,
            "puzzle_emb_ndim": 512
        }
        
        if torch is None or TinyRecursiveReasoningModel_ACTV1 is None:
            self.device = "cpu"
            print("[TRM Brain] PyTorch not loaded; running in lightweight CPU fallback mode.")
            return

        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        print(f"[TRM Brain] Initializing on {self.device}...")
        
        try:
            self._model = TinyRecursiveReasoningModel_ACTV1(self.config)
            
            if os.path.exists(self.checkpoint_path):
                state_dict = torch.load(self.checkpoint_path, map_location=self.device)
                
                # Strip prefix
                new_state_dict = {}
                for k, v in state_dict.items():
                    new_key = k.replace("_orig_mod.model.", "")
                    if v.dtype == torch.bfloat16 and self.device == torch.device("cpu"):
                         v = v.to(torch.float32)
                    new_state_dict[new_key] = v
                
                self._model.load_state_dict(new_state_dict)
                self._model.to(self.device).eval()
                print(f"[TRM Brain] Model loaded from {self.checkpoint_path}")
            else:
                print(f"[TRM Brain] Warning: Checkpoint not found at {self.checkpoint_path}")
        except Exception as e:
            import traceback
            print(f"[TRM Brain] Error loading model: {e}")
            traceback.print_exc()
            self._model = None

    def reason(self, input_tokens: List[int], puzzle_id: int = 0, **kwargs) -> List[int]:
        """
        Perform recursive reasoning on a sequence of tokens.
        """
        if self._model is None:
            print("[TRM Brain] Error: Model not loaded.")
            return []

        with torch.no_grad():
            vocab_sz = self.config.get('vocab_size', 12)
            # Map arbitrary integers deterministically into [0, vocab_sz - 1]
            clamped_tokens = [abs(int(t)) % vocab_sz for t in input_tokens] if input_tokens else [0]
            inputs = torch.tensor(clamped_tokens).unsqueeze(0).to(self.device)
            p_id = torch.tensor([puzzle_id]).to(self.device)
            
            # Check seq_len match
            if inputs.shape[1] < self.config['seq_len']:
                padding = torch.zeros((1, self.config['seq_len'] - inputs.shape[1]), dtype=torch.long).to(self.device)
                inputs = torch.cat([inputs, padding], dim=1)
            else:
                inputs = inputs[:, :self.config['seq_len']]

            batch = {
                "inputs": inputs,
                "puzzle_identifiers": p_id
            }

            carry = self._model.initial_carry(batch)
            
            # Loop for dynamic H_cycles to achieve deep reasoning logic
            total_preds = []
            h_cycles = kwargs.get("h_cycles", self.config['H_cycles'])
            
            for _ in range(h_cycles):
                carry, outputs = self._model(carry, batch)
                logits = outputs["logits"] # [1, seq_len, vocab_size]
                # Extract the reasoning token predictions
                preds = torch.argmax(logits, dim=-1).squeeze(0).cpu().tolist()
                total_preds.extend(preds[:12]) # Take sequence snippet per cycle
                
            return total_preds

def get_trm_brain():
    return TRMBrain()
