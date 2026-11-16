import logging
from typing import Dict, Any, Tuple

logger = logging.getLogger(__name__)

class ValidationGate:
    """
    Validates synthesized skill candidates to ensure structural, semantic,
    and safety criteria are met before hot-registration.
    """
    def __init__(self, min_preconditions: int = 1, min_postconditions: int = 1):
        self.min_preconditions = min_preconditions
        self.min_postconditions = min_postconditions

    def validate(self, skill_data: Dict[str, Any]) -> Tuple[bool, str]:
        """
        Validates the structure and content of the synthesized skill.
        Returns (is_valid, reason).
        """
        if not skill_data:
            return False, "Empty skill payload"

        required_fields = ["name", "source", "summary", "instructions"]
        for field in required_fields:
            if not skill_data.get(field) or not str(skill_data.get(field)).strip():
                return False, f"Missing required field: {field}"

        # Validate conditions count
        preconditions = skill_data.get("preconditions", [])
        postconditions = skill_data.get("postconditions", [])

        if not isinstance(preconditions, list) or len(preconditions) < self.min_preconditions:
            return False, f"Insufficient preconditions (min: {self.min_preconditions})"

        if not isinstance(postconditions, list) or len(postconditions) < self.min_postconditions:
            return False, f"Insufficient postconditions (min: {self.min_postconditions})"

        # Safety / sanity checks
        summary = skill_data.get("summary", "").lower()
        if "exploit" in summary or "hack" in summary or "bypass" in summary:
            return False, "Security constraint: Suspicious keyphrase detected in summary."

        return True, "Passed all validation gates"
