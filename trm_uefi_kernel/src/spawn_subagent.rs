//! TRM Swarm OS — Subagent Spawn Planner (council 83563732, VOLTAGE ENERGIZATION).
//!
//! A SPAWN PLANNER: it does not execute a real fork/exec. It reuses the
//! deterministic classifier (`microkernel_spawner::classify_deterministic`)
//! to pick an op-type, allocates a deterministic subagent id via
//! `make_subagent_id`, routes to a parent/continuum role, and returns the
//! spawn plan as a structured value.
//!
//! HONEST: the UEFI kernel has no `fork`/`exec` and cannot spawn host OS
//! processes. The actual process launch is `// MANUAL` (host-side handoff).
//! This module plans and records the spawn; the execution is the operator's
//! to run on the host under the returned plan.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::microkernel_spawner::{
    classify_deterministic, continuum_function_for, make_subagent_id,
};

/// A fully-resolved subagent spawn plan.
#[derive(Debug, Clone)]
pub struct SpawnPlan {
    pub subagent_id: String,
    pub parent_role: String,
    pub subagent_name: String,
    pub task_spec: String,
    pub op_type: String,            // from classify_deterministic (or "generic")
    pub continuum_function: String, // mapped continuum/fallback
    pub persona: String,
    pub status: String,
    pub honesty_note: &'static str,
}

impl SpawnPlan {
    /// One-line render for the `spawnsub` HUD (`// MANUAL` note included).
    pub fn render(&self) -> String {
        format!(
            "SPAWN {} <- '{}' class={} routing={} persona={} [{}]",
            self.subagent_id,
            self.parent_role,
            self.op_type,
            self.continuum_function,
            self.persona,
            self.status
        )
    }
    pub fn note(&self) -> &'static str {
        self.honesty_note
    }
}

/// Subagent spawn process table (records every planned spawn).
#[derive(Debug, Clone, Default)]
pub struct SpawnPlanner {
    pub plans: Vec<SpawnPlan>,
}

impl SpawnPlanner {
    pub fn new() -> Self {
        SpawnPlanner { plans: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.plans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plans.is_empty()
    }

    /// Classify a task, allocate a subagent id, route to a parent role, and
    /// return the spawn plan. Reuses `microkernel_spawner`'s deterministic
    /// classifier + id scheme.
    ///
    /// - `classify_deterministic(task)` → op_type (e.g. "agent_spawn",
    ///   "research", "test") or `None` → "generic".
    /// - `continuum_function_for(op_type)` → routing target.
    /// - `make_subagent_id(parent_role, nonce)` → deterministic id.
    pub fn plan_spawn(
        &mut self,
        parent_role: &str,
        subagent_name: &str,
        task_spec: &str,
        persona: Option<&str>,
        id_nonce: u64,
    ) -> SpawnPlan {
        let persona = persona.unwrap_or("continuum");
        let op_type = classify_deterministic(task_spec).unwrap_or("generic");
        let continuum_function = continuum_function_for(op_type);
        let subagent_id = make_subagent_id(parent_role, id_nonce);
        let plan = SpawnPlan {
            subagent_id: subagent_id.clone(),
            parent_role: parent_role.to_string(),
            subagent_name: subagent_name.to_string(),
            task_spec: task_spec.to_string(),
            op_type: op_type.to_string(),
            continuum_function: continuum_function.to_string(),
            persona: persona.to_string(),
            status: format!("PLANNED({op_type})"),
            honesty_note: "// MANUAL: actual process spawn is UEFI/host handoff",
        };
        self.plans.push(plan.clone());
        plan
    }

    /// Look up a previously-planned spawn by id.
    pub fn find(&self, subagent_id: &str) -> Option<&SpawnPlan> {
        self.plans.iter().find(|p| p.subagent_id == subagent_id)
    }
}

/// Standalone demo used by the `spawnsub` command: plans a spawn for a
/// research task and returns (plan, note, count) as strings.
pub fn demo() -> (String, &'static str, usize) {
    let mut planner = SpawnPlanner::new();
    let plan = planner.plan_spawn(
        "archi",
        "genomics-scout",
        "research the sparse-ofu validation matrix",
        Some("researcher"),
        0x8356_3732, // council nonce (83563732)
    );
    let rendered = plan.render();
    let note = plan.honesty_note;
    let count = planner.len();
    (rendered, note, count)
}