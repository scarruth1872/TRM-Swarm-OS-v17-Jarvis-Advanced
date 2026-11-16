//! Kanban board - task state machine with orchestration hooks.
//!
//! Port of `swarm_v2/core/kanban_board.py` (311 lines).
//!
//! SWEEP CONTEXT: only a `KanbanCard` field-struct and a stub accessor had been
//! ported. The board itself - create_card, move_card (a validated state
//! machine), assign_card, archive_card, get_board/get_column/get_stats - was
//! absent. `PRIORITY_ORDER` was a placeholder `u64 = 0` where the Python has a
//! dict.
//!
//! TRANSITIONS (the Python state machine, reproduced exactly):
//!   TODO        -> IN_PROGRESS
//!   IN_PROGRESS -> REVIEW, TODO
//!   REVIEW      -> DONE, IN_PROGRESS
//!   DONE        -> ARCHIVED
//!   ARCHIVED    -> (terminal)
//!
//! no_std ADAPTATIONS:
//!  * Python's `move_card` side effects call WorktreeManager (git worktrees) and
//!    PortManager (host ports). The kernel can do neither, so instead of
//!    silently skipping them the move RETURNS the hooks the host must perform.
//!    Nothing is pretended to have happened.
//!  * `datetime.now().isoformat()` -> `at_ns` u64 on the card and each history
//!    entry. The String timestamp fields are gone rather than left as fake ISO
//!    strings.
//!  * No file persistence (`_load`/`_save` wrote kanban_state.json). The board
//!    is in-memory; the host owns durability.

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// CardStatus, in the Python's declaration order.
pub const CARD_STATUSES: [&str; 5] = ["TODO", "IN_PROGRESS", "REVIEW", "DONE", "ARCHIVED"];

/// `TRANSITIONS` - valid targets for a status. Unknown status -> empty.
pub fn allowed_targets(status: &str) -> Vec<&'static str> {
    match status {
        "TODO" => vec!["IN_PROGRESS"],
        "IN_PROGRESS" => vec!["REVIEW", "TODO"],
        "REVIEW" => vec!["DONE", "IN_PROGRESS"],
        "DONE" => vec!["ARCHIVED"],
        _ => Vec::new(),
    }
}

/// `PRIORITY_ORDER` - lower is more urgent. Unknown defaults to 2 ("medium"),
/// matching Python's `.get(priority, 2)`.
pub fn priority_rank(priority: &str) -> u32 {
    match priority {
        "critical" => 0,
        "high" => 1,
        "medium" => 2,
        "low" => 3,
        _ => 2,
    }
}

#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub action: String,
    pub at_ns: u64,
    pub details: String,
}

#[derive(Clone, Debug)]
pub struct KanbanCard {
    pub card_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub assignee: String,
    pub priority: String,
    pub tags: Vec<String>,
    pub worktree_path: String,
    pub allocated_port: u64,
    pub created_ns: u64,
    pub updated_ns: u64,
    pub completed_ns: u64,
    pub history: Vec<HistoryEntry>,
}

#[derive(Clone, Debug)]
pub struct BoardStats {
    pub total_cards: usize,
    pub todo: usize,
    pub in_progress: usize,
    pub review: usize,
    pub done: usize,
    pub archived: usize,
    pub active_worktrees: usize,
    pub active_ports: usize,
}

/// Work the HOST must do as a result of a move. Returned, never assumed done.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoardHook {
    AllocateWorktree { card_id: String, branch: String },
    AllocatePort { assignee: String, card_id: String },
    ReleasePort { port: u64 },
    RemoveWorktree { path: String },
}

#[derive(Clone, Debug)]
pub struct MoveResult {
    pub success: bool,
    pub message: String,
    pub card: Option<KanbanCard>,
    pub hooks: Vec<BoardHook>,
}

pub struct KanbanBoard {
    pub cards: Vec<KanbanCard>,
    /// The Python used uuid.uuid4().hex[:8]. A seeded xorshift keeps ids unique
    /// and reproducible - the kernel has no uuid source, and determinism is an
    /// improvement over the original's randomness (same reasoning as the
    /// seeded weight init in kernel_instructions).
    rng: crate::float_math::Rng,
}

impl KanbanBoard {
    pub fn new() -> Self {
        KanbanBoard { cards: Vec::new(), rng: crate::float_math::Rng::new(0x4B42_0000_0001) }
    }

    fn new_card_id(&mut self) -> String {
        let v = self.rng.next_u64() & 0xFFFF_FFFF;
        format!("card-{:08x}", v)
    }

    fn find_mut(&mut self, card_id: &str) -> Option<usize> {
        self.cards.iter().position(|c| c.card_id == card_id)
    }

    /// Create a card in TODO. Returns the generated card_id.
    /// Mirrors `create_card` (including the "created" history entry).
    pub fn create_card(&mut self, title: &str, description: &str, assignee: &str,
                       priority: &str, tags: &[String], now_ns: u64) -> String {
        let card_id = self.new_card_id();
        let mut card = KanbanCard {
            card_id: card_id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            status: "TODO".to_string(),
            assignee: assignee.to_string(),
            priority: priority.to_string(),
            tags: tags.to_vec(),
            worktree_path: String::new(),
            allocated_port: 0,
            created_ns: now_ns,
            updated_ns: now_ns,
            completed_ns: 0,
            history: Vec::new(),
        };
        card.history.push(HistoryEntry {
            action: "created".to_string(),
            at_ns: now_ns,
            details: format!("Card created: {}", title),
        });
        self.cards.push(card);
        card_id
    }

    /// Move a card, validating the transition. Returns the hooks the host must
    /// run (worktree + port allocation on IN_PROGRESS, release on DONE).
    /// Mirrors `move_card`, message strings included.
    pub fn move_card(&mut self, card_id: &str, target_status: &str, now_ns: u64) -> MoveResult {
        let idx = match self.find_mut(card_id) {
            Some(i) => i,
            None => return MoveResult {
                success: false,
                message: format!("Card {} not found", card_id),
                card: None, hooks: Vec::new(),
            },
        };
        if !CARD_STATUSES.contains(&target_status) {
            return MoveResult {
                success: false,
                message: format!("Invalid status: {}", target_status),
                card: None, hooks: Vec::new(),
            };
        }
        let current = self.cards[idx].status.clone();
        let allowed = allowed_targets(&current);
        if !allowed.contains(&target_status) {
            let al: Vec<&str> = allowed.clone();
            return MoveResult {
                success: false,
                message: format!("Invalid transition: {} -> {}. Allowed: {:?}",
                                 current, target_status, al),
                card: None, hooks: Vec::new(),
            };
        }
        let old_status = current.clone();
        self.cards[idx].status = target_status.to_string();
        self.cards[idx].updated_ns = now_ns;

        let mut hooks: Vec<BoardHook> = Vec::new();
        if target_status == "IN_PROGRESS" {
            let assignee = self.cards[idx].assignee.clone();
            if assignee.is_empty() {
                hooks.push(BoardHook::AllocatePort {
                    assignee: "kanban".to_string(), card_id: card_id.to_string() });
            } else {
                hooks.push(BoardHook::AllocatePort {
                    assignee, card_id: card_id.to_string() });
            }
            hooks.push(BoardHook::AllocateWorktree {
                card_id: card_id.to_string(),
                branch: format!("task/{}", card_id),
            });
        }
        if target_status == "DONE" {
            self.cards[idx].completed_ns = now_ns;
            if self.cards[idx].allocated_port != 0 {
                hooks.push(BoardHook::ReleasePort { port: self.cards[idx].allocated_port });
            }
            if !self.cards[idx].worktree_path.is_empty() {
                hooks.push(BoardHook::RemoveWorktree {
                    path: self.cards[idx].worktree_path.clone() });
            }
        }
        self.cards[idx].history.push(HistoryEntry {
            action: "moved".to_string(),
            at_ns: now_ns,
            details: format!("{} -> {}", old_status, target_status),
        });
        MoveResult {
            success: true,
            message: "OK".to_string(),
            card: Some(self.cards[idx].clone()),
            hooks,
        }
    }

    /// Assign a card. Mirrors `assign_card` (false when the card is unknown).
    pub fn assign_card(&mut self, card_id: &str, assignee: &str, now_ns: u64) -> bool {
        let idx = match self.find_mut(card_id) { Some(i) => i, None => return false };
        self.cards[idx].assignee = assignee.to_string();
        self.cards[idx].updated_ns = now_ns;
        self.cards[idx].history.push(HistoryEntry {
            action: "assigned".to_string(),
            at_ns: now_ns,
            details: format!("Assigned to {}", assignee),
        });
        true
    }

    /// Archive a DONE card. Mirrors `archive_card` (delegates to move_card).
    pub fn archive_card(&mut self, card_id: &str, now_ns: u64) -> bool {
        self.move_card(card_id, "ARCHIVED", now_ns).success
    }

    pub fn get_card(&self, card_id: &str) -> Option<&KanbanCard> {
        self.cards.iter().find(|c| c.card_id == card_id)
    }

    /// Cards in one column, in insertion order.
    pub fn get_column(&self, status: &str) -> Vec<&KanbanCard> {
        self.cards.iter().filter(|c| c.status == status).collect()
    }

    /// A card's column sorted by priority (stable, like Python's sort).
    pub fn get_column_sorted(&self, status: &str) -> Vec<&KanbanCard> {
        let mut v = self.get_column(status);
        v.sort_by_key(|c| priority_rank(&c.priority));
        v
    }

    /// An agent's non-archived cards.
    pub fn get_agent_cards(&self, assignee: &str) -> Vec<&KanbanCard> {
        self.cards.iter()
            .filter(|c| c.assignee == assignee && c.status != "ARCHIVED")
            .collect()
    }

    pub fn get_stats(&self) -> BoardStats {
        let n = |s: &str| self.cards.iter().filter(|c| c.status == s).count();
        BoardStats {
            total_cards: self.cards.len(),
            todo: n("TODO"),
            in_progress: n("IN_PROGRESS"),
            review: n("REVIEW"),
            done: n("DONE"),
            archived: n("ARCHIVED"),
            active_worktrees: self.cards.iter()
                .filter(|c| !c.worktree_path.is_empty() && c.status == "IN_PROGRESS").count(),
            active_ports: self.cards.iter()
                .filter(|c| c.allocated_port != 0 && c.status == "IN_PROGRESS").count(),
        }
    }
}
