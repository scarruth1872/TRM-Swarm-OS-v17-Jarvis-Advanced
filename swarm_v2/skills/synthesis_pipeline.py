import logging
import json
from typing import Dict, Any, Optional

logger = logging.getLogger(__name__)

class SkillSynthesisPipeline:
    """
    Transforms raw codebase structural analysis (AST data) into a well-formed
    skill candidate using LLM template generation.
    """
    def __init__(self, llm_generate_fn=None):
        """
        llm_generate_fn: A function that takes a prompt and returns an LLM response string.
        """
        self.llm_generate = llm_generate_fn

    def _determine_template(self, parsed_data: Dict[str, Any]) -> str:
        """Determines the broad category of the codebase file."""
        name = parsed_data.get("filepath", "").lower()
        if "api" in name or "client" in name:
            return "api_integration"
        elif "helper" in name or "utils" in name:
            return "utility_helper"
        elif "db" in name or "storage" in name or "model" in name:
            return "data_model"
        return "general_capability"

    async def synthesize(self, parsed_data: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """
        Compiles the AST structure into a Skill Candidate dictionary containing
        instructions, examples, and preconditions.
        """
        if not self.llm_generate:
            logger.warning("[SynthesisPipeline] No LLM function provided for synthesis.")
            return None

        filepath = parsed_data.get("filepath", "unknown_file")
        template_type = self._determine_template(parsed_data)
        
        prompt = (
            f"You are the Swarm's Skill Synthesis Pipeline.\n"
            f"Analyze the following code structure and output a JSON skill template.\n\n"
            f"File: {filepath}\n"
            f"Template Type: {template_type}\n"
            f"Classes: {json.dumps(parsed_data.get('classes', []))}\n"
            f"Functions: {json.dumps(parsed_data.get('functions', []))}\n\n"
            f"Output strictly valid JSON with this schema:\n"
            f"{{\n"
            f"  \"summary\": \"1-sentence description\",\n"
            f"  \"instructions\": \"Step-by-step how to use this code\",\n"
            f"  \"preconditions\": [\"condition 1\", \"condition 2\"],\n"
            f"  \"postconditions\": [\"result 1\", \"result 2\"]\n"
            f"}}"
        )
        
        try:
            response = await self.llm_generate(prompt)
            if isinstance(response, tuple):
                response = response[0]
            
            # Simple JSON extraction in case the LLM wraps it in markdown blocks
            if "```json" in response:
                response = response.split("```json")[1].split("```")[0]
            elif "```" in response:
                response = response.split("```")[1].split("```")[0]
                
            skill_dict = json.loads(response.strip())
            
            # Format into the final payload structure
            clean_filepath = filepath.replace("/", "_").replace("\\", "_").replace(".", "_")
            skill_name = f"Learned_{clean_filepath}"
            
            return {
                "name": skill_name,
                "source": f"codebase://{filepath}",
                "template_type": template_type,
                "summary": skill_dict.get("summary", ""),
                "instructions": skill_dict.get("instructions", ""),
                "preconditions": skill_dict.get("preconditions", []),
                "postconditions": skill_dict.get("postconditions", [])
            }
        except Exception as e:
            logger.error(f"[SynthesisPipeline] Failed to synthesize {filepath}: {e}")
            return None
