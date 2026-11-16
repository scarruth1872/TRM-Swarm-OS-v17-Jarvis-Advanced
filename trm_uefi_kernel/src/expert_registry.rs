//! TRM Swarm OS — Expert Registry (no_std port).
//! 1:1 core semantics from swarm_v2/core/expert_registry.py.
//! A global registry connecting live agent instances across the swarm.
//! Agent *instances* in python are live objects; in the no_std kernel the
//! registry stores an ExpertAgent descriptor (agent_id + optional persona),
//! which is the honest equivalent. The singleton pattern (python __new__ /
//! module-level _instance) maps to a handle token per repo convention.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Persona descriptor (python agent.persona object).
#[derive(Debug, Clone)]
pub struct Persona {
    pub name: String,
    pub role: String,
}

impl Persona {
    pub fn new(name: &str, role: &str) -> Self {
        Persona {
            name: name.to_string(),
            role: role.to_string(),
        }
    }
}

/// A registered agent descriptor (stands in for the live python agent
/// instance; get_agent() returns it so callers can read id/persona).
#[derive(Debug, Clone)]
pub struct ExpertAgent {
    pub agent_id: String,
    pub persona: Option<Persona>,
}

impl ExpertAgent {
    pub fn new(agent_id: &str, persona: Option<Persona>) -> Self {
        ExpertAgent {
            agent_id: agent_id.to_string(),
            persona,
        }
    }
}

/// ExpertRegistry (1:1 python class; python _team dict -> Vec<(role, agent)>,
/// insertion order preserved — dict iteration order drives match priority).
#[derive(Debug, Clone)]
pub struct ExpertRegistry {
    pub team: Vec<(String, ExpertAgent)>,
}

impl ExpertRegistry {
    pub fn new() -> Self {
        ExpertRegistry { team: Vec::new() }
    }

    /// Register the live expert team (Role -> Agent). 1:1 python
    /// register_team: replaces the whole team.
    pub fn register_team(&mut self, team: Vec<(String, ExpertAgent)>) {
        self.team = team;
    }

    /// Retrieve an agent by role key, persona name, or friendly string.
    /// 1:1 python get_agent: identifier is stripped of [*`_] and whitespace,
    /// then matched (1) exact role key, (2) case-insensitive role key,
    /// (3) persona name/role exact or case-insensitive substring.
    pub fn get_agent(&self, identifier: &str) -> Option<&ExpertAgent> {
        if identifier.is_empty() {
            return None;
        }
        // python: re.sub(r'[\*\`_]', '', str(identifier)).strip()
        let cleaned: String = identifier
            .chars()
            .filter(|c| *c != '*' && *c != '`' && *c != '_')
            .collect();
        let clean_id = cleaned.trim().to_string();
        if clean_id.is_empty() {
            return None;
        }

        // 1. Direct role key match
        for (r_key, agent) in self.team.iter() {
            if r_key.as_str() == clean_id {
                return Some(agent);
            }
        }

        // 2. Case-insensitive role key match
        let clean_lower = clean_id.to_lowercase();
        for (r_key, agent) in self.team.iter() {
            if r_key.to_lowercase() == clean_lower {
                return Some(agent);
            }
        }

        // 3. Match by persona name or role title
        // python: if p_name and (clean_id == p_name or p_name.lower() in clean_id.lower())
        for (_r_key, agent) in self.team.iter() {
            if let Some(p) = &agent.persona {
                if !p.name.is_empty()
                    && (clean_id == p.name || clean_lower.contains(&p.name.to_lowercase()))
                {
                    return Some(agent);
                }
                if !p.role.is_empty()
                    && (clean_id == p.role || clean_lower.contains(&p.role.to_lowercase()))
                {
                    return Some(agent);
                }
            }
        }

        None
    }

    /// All registered agents (python get_all_agents returns the team dict).
    pub fn get_all_agents(&self) -> &[(String, ExpertAgent)] {
        &self.team
    }
}

impl Default for ExpertRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Singleton handle (repo convention; python get_expert_registry() returns
/// the class-level singleton registry).
pub fn get_expert_registry() -> u64 {
    0x4558_5045_5254_5247 // "EXPERTRG"
}

// ── ported from the v20 base tree (G189) ─────────────────────────────────────────────────────────
// 2 column-0 item(s) only: nested fn definitions and `self` receivers are excluded, and the
// statics/consts each function names travel with it (grafting bare functions was the first failure).

pub fn resolve_identifier(
    identifier: &str,
    keys: &[String],
    persona_names: &[String],
    persona_roles: &[String],
) -> i32 {
    if identifier.is_empty() {
        return -1;
    }
    let clean = sanitize_identifier(identifier);
    let clean_lower = clean.to_lowercase();

    for (i, k) in keys.iter().enumerate() {
        if *k == clean {
            return i as i32;
        }
    }
    for (i, k) in keys.iter().enumerate() {
        if k.to_lowercase() == clean_lower {
            return i as i32;
        }
    }
    for i in 0..keys.len() {
        let name = persona_names.get(i).map(|s| s.as_str()).unwrap_or("");
        let role = persona_roles.get(i).map(|s| s.as_str()).unwrap_or("");
        if !name.is_empty() && (clean == name || clean_lower.contains(&name.to_lowercase())) {
            return i as i32;
        }
        if !role.is_empty() && (clean == role || clean_lower.contains(&role.to_lowercase())) {
            return i as i32;
        }
    }
    -1
}

pub fn sanitize_identifier(identifier: &str) -> String {
    identifier
        .chars()
        .filter(|c| *c != '*' && *c != '`' && *c != '_')
        .collect::<String>()
        .trim()
        .to_string()
}
