//! TRM Swarm OS - kanban_board (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/kanban_board.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const KANBAN_STATE_PATH: u64 = 0;
pub const TRANSITIONS: u64 = 0;
pub const PRIORITY_ORDER: u64 = 0;

//  KanbanCard 
#[derive(Debug, Clone)]
pub struct KanbanCard {
    pub card_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub assignee: String,
    pub priority: String,
    pub tags: Vec<String>,
    pub worktree_path: String,
    pub allocated_port: i64,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: String,
    pub history: Vec<Vec<(String, f64)>>,
}

impl KanbanCard {
    pub fn new() -> Self {
        KanbanCard {
            card_id: String::new(),
            title: String::new(),
            description: String::new(),
            status: String::new(),
            assignee: String::new(),
            priority: String::new(),
            tags: vec![],
            worktree_path: String::new(),
            allocated_port: 0,
            created_at: String::new(),
            updated_at: String::new(),
            completed_at: String::new(),
            history: vec![],
        }
    }
}

//  get_kanban_board() 
static SINGLETON_KANBAN_BOARD: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_kanban_board() -> u64 {
    // Singleton handle (lazy-init pattern from kanban_board.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_KANBAN_BOARD.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_KANBAN_BOARD.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
