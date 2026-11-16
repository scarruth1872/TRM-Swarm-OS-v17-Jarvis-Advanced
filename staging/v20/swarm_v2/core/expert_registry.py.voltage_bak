
from typing import Dict, Any, Optional

class ExpertRegistry:
    """A global registry to connect live agent instances across the swarm."""
    _instance = None
    _team: Dict[str, Any] = {}

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super(ExpertRegistry, cls).__new__(cls)
        return cls._instance

    def register_team(self, team: Dict[str, Any]):
        """Register the live expert team (Role -> Agent Instance)."""
        self._team = team

    def get_agent(self, identifier: str) -> Optional[Any]:
        """Retrieve a live agent instance by role key, persona name, or friendly string."""
        if not identifier:
            return None
        import re
        clean_id = re.sub(r'[\*\`_]', '', str(identifier)).strip()
        
        # 1. Direct role key match
        if clean_id in self._team:
            return self._team[clean_id]

        # 2. Case-insensitive role key match
        for r_key, agent in self._team.items():
            if r_key.lower() == clean_id.lower():
                return agent

        # 3. Match by persona name or role title (e.g., "Logic", "** Logic (Reasoning Engine)")
        for r_key, agent in self._team.items():
            p_name = getattr(getattr(agent, "persona", None), "name", "")
            p_role = getattr(getattr(agent, "persona", None), "role", "")
            
            if p_name and (clean_id == p_name or p_name.lower() in clean_id.lower()):
                return agent
            if p_role and (clean_id == p_role or p_role.lower() in clean_id.lower()):
                return agent

        return None

    def get_all_agents(self) -> Dict[str, Any]:
        return self._team

def get_expert_registry() -> ExpertRegistry:
    return ExpertRegistry()
