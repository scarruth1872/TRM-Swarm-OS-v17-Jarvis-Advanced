//! TRM Swarm OS - expert_registry (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/expert_registry.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


use alloc::string::ToString;
//  ExpertRegistry 
#[derive(Debug, Clone)]
pub struct ExpertRegistry {
    pub _team: Vec<(String, f64)>,
}

impl ExpertRegistry {
    pub fn new() -> Self {
        ExpertRegistry {
            _team: vec![],
        }
    }
}

//  get_expert_registry() 
static SINGLETON_EXPERT_REGISTRY: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

/// python `get_expert_registry()` returns the class's singleton `ExpertRegistry()` instance.
/// The no_std handle convention is 1 once initialized.
///
/// NOT portable here, by design: the registry's payload is live agent INSTANCES. What this module owns is
/// the RESOLUTION RULE -- how an identifier maps to a row -- which is what `resolve_identifier` below ports
/// and what a parity ruler can check. `_team: Vec<(String, f64)>` above is the mechanical extraction of the
/// dict's shape, not a substitute for it.
pub fn get_expert_registry() -> u64 {
    let v = SINGLETON_EXPERT_REGISTRY.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_EXPERT_REGISTRY.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

/// python: `re.sub(r'[*`_]', '', str(identifier)).strip()` -- the markdown characters are removed, then the
/// result is trimmed. `str()` on a non-string arrives here already converted by the caller.
pub fn sanitize_identifier(identifier: &str) -> String {
    identifier
        .chars()
        .filter(|c| *c != '*' && *c != '`' && *c != '_')
        .collect::<String>()
        .trim()
        .to_string()
}

/// The oracle's three-stage resolution rule from `ExpertRegistry.get_agent`, in its order:
///
///   0. an empty RAW identifier resolves to nothing (the oracle tests truthiness BEFORE cleaning)
///   1. exact match against a role key
///   2. case-insensitive match against a role key
///   3. containment against a row's persona name, then its persona role -- the NAME/ROLE must appear inside
///      the identifier (`p_name.lower() in clean_id.lower()`), not the other way round
///
/// First row wins, so the row ORDER is part of the answer and is preserved rather than sorted. Returns the
/// row index, or -1 for the oracle's `None`.
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
