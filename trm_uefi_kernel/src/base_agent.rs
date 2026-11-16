//! TRM Swarm OS - base_agent (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/base_agent.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  AgentPersona 
#[derive(Debug, Clone)]
pub struct AgentPersona {
    pub name: String,
    pub role: String,
    pub background: String,
    pub specialties: Vec<String>,
    pub avatar_color: String,
    pub model: String,
    pub department: String,
    pub llm_backend: String,
    pub core_axioms: Vec<String>,
    pub forbidden_lexicon: Vec<String>,
}

impl AgentPersona {
    pub fn new() -> Self {
        AgentPersona {
            name: String::new(),
            role: String::new(),
            background: String::new(),
            specialties: vec![],
            avatar_color: String::new(),
            model: String::new(),
            department: String::new(),
            llm_backend: String::new(),
            core_axioms: vec![],
            forbidden_lexicon: vec![],
        }
    }
}

//  Message 
#[derive(Debug, Clone)]
pub struct Message {
    pub sender: String,
    pub receiver: String,
    pub content: String,
    pub task_id: String,
    pub metadata: Vec<(String, f64)>,
}

impl Message {
    pub fn new() -> Self {
        Message {
            sender: String::new(),
            receiver: String::new(),
            content: String::new(),
            task_id: String::new(),
            metadata: vec![],
        }
    }
}
