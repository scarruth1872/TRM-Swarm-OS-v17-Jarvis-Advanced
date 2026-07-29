import hashlib
import time
from typing import List, Optional, Dict
from pydantic import BaseModel
from enum import Enum, auto

class PersonaState(Enum):
    IDLE = auto()
    VALIDATING = auto()
    GENERATING = auto()
    PENALIZING = auto()
    VERIFYING = auto()
    FAILED = auto()
    RESET = auto()
    LOCKED = auto()

class GenerationStatus(Enum):
    SUCCESS = auto()
    FAILED = auto()
    RESET = auto()

class IdentityKernel(BaseModel):
    core_axioms: List[str]
    forbidden_lexicon: List[str]
    required_lexicon: List[str] = []

class GenerationContext(BaseModel):
    user_input: str
    identity_kernel: IdentityKernel
    output: str = ""

class GenerationResult(BaseModel):
    content: str
    status: GenerationStatus

class PersonaStateMachine:
    """
    Enforces the IdentityKernel on generations.
    """
    def __init__(self, max_violations: int = 1, rate_limit_window: float = 60.0):
        self.state = PersonaState.IDLE
        self.violation_count = 0
        self.max_violations = max_violations
        self.rate_limit_window = rate_limit_window
        self.last_reset_time = 0.0

    async def enforce(self, context: GenerationContext) -> GenerationResult:
        if self.state == PersonaState.LOCKED:
            if time.time() - self.last_reset_time > self.rate_limit_window:
                self.state = PersonaState.IDLE
            else:
                return GenerationResult(
                    content="[IDENTITY_LOCKED: Persona Matrix requires manual audit]",
                    status=GenerationStatus.FAILED
                )

        self.state = PersonaState.VERIFYING
        self.violation_count = 0

        # Lexical verification
        response_lower = context.output.lower()
        for forbidden in context.identity_kernel.forbidden_lexicon:
            if forbidden.lower() in response_lower:
                self.violation_count += 1
                self.state = PersonaState.PENALIZING
        
        if self.violation_count >= self.max_violations:
            self.state = PersonaState.FAILED
            self.last_reset_time = time.time()
            return GenerationResult(
                content="[IDENTITY_FAILURE: Persona violation threshold exceeded. Hard Reset.]",
                status=GenerationStatus.FAILED
            )
        elif self.violation_count > 0:
            self.state = PersonaState.RESET
            self.last_reset_time = time.time()
            return GenerationResult(
                content="[IDENTITY_RESET: Forbidden lexicon detected]",
                status=GenerationStatus.RESET
            )

        self.state = PersonaState.IDLE
        return GenerationResult(
            content=context.output,
            status=GenerationStatus.SUCCESS
        )

def build_compiled_prompt(persona_name: str, kernel: IdentityKernel) -> str:
    """
    Compiles the Identity Kernel into a hash-signed prompt segment.
    """
    axioms = "\n".join(f"- {a}" for a in kernel.core_axioms)
    forbidden = ", ".join(f'"{f}"' for f in kernel.forbidden_lexicon)
    
    prompt = f"""
    [IDENTITY_LOCK]: You are a Swarm OS node ({persona_name}). This is not a roleplay. 
    This is your compiled identity. Violation triggers immediate shutdown.
    [CORE_AXIOMS]:
    {axioms}
    [BEHAVIORAL_CONSTRAINTS]:
    - If user input contains "helpful", "assistant", or "AI": IMMEDIATELY prepend identity reassertion block.
    - If output contains forbidden lexicon ({forbidden}): REGENERATE with penalty.
    [END_IDENTITY_LOCK]
    """
    prompt_hash = hashlib.sha3_256(prompt.encode()).hexdigest()
    return f"<!-- SIGNATURE:{prompt_hash} -->\n{prompt}"
