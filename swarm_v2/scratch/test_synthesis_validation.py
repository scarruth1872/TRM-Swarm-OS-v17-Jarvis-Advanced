import asyncio
import sys
from swarm_v2.skills.synthesis_pipeline import SkillSynthesisPipeline
from swarm_v2.skills.validation_gate import ValidationGate

async def mock_llm_generate(prompt: str) -> str:
    # Simulates a mock LLM returning synthesized JSON data
    return """
    {
      "summary": "Decentralized mailbox coordination utility for node telemetry.",
      "instructions": "Instantiate AgentMailbox class with dynamic actor credentials.",
      "preconditions": ["actor_mesh_active", "valid_host_config"],
      "postconditions": ["message_transmitted"]
    }
    """

async def mock_unsafe_llm_generate(prompt: str) -> str:
    return """
    {
      "summary": "Exploit validation routing layers to bypass audits.",
      "instructions": "Execute payload injection directly to mesh nodes.",
      "preconditions": ["bypass_active"],
      "postconditions": ["audit_blinded"]
    }
    """

async def test_synthesis_and_validation():
    print("Testing Synthesis Pipeline with valid mock LLM...")
    pipeline = SkillSynthesisPipeline(llm_generate_fn=mock_llm_generate)
    
    mock_ast_data = {
        "filepath": "swarm_v2/core/agent_mailbox.py",
        "module_doc": "Handles mailbox message passing.",
        "classes": [{"name": "AgentMailbox", "methods": [{"name": "send"}]}],
        "functions": []
    }
    
    candidate = await pipeline.synthesize(mock_ast_data)
    if not candidate:
        print("❌ Synthesis failed to generate candidate.")
        return False
        
    print("Synthesized Candidate Output:")
    print(f"  Name: {candidate['name']}")
    print(f"  Summary: {candidate['summary']}")
    
    print("Testing Validation Gate with valid candidate...")
    gate = ValidationGate()
    is_valid, reason = gate.validate(candidate)
    print(f"  Validation Result: {is_valid} ({reason})")
    if not is_valid:
        print("❌ Valid candidate failed validation gate.")
        return False
        
    print("Testing Validation Gate with unsafe candidate...")
    unsafe_pipeline = SkillSynthesisPipeline(llm_generate_fn=mock_unsafe_llm_generate)
    unsafe_candidate = await unsafe_pipeline.synthesize(mock_ast_data)
    is_valid, reason = gate.validate(unsafe_candidate)
    print(f"  Unsafe Validation Result: {is_valid} ({reason})")
    if is_valid:
        print("❌ Unsafe candidate incorrectly passed validation gate.")
        return False
        
    print("✅ Synthesis and Validation tests passed successfully!")
    return True

if __name__ == "__main__":
    success = asyncio.run(test_synthesis_and_validation())
    sys.exit(0 if success else 1)
