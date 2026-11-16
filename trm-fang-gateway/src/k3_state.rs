// k3_state.rs — TRM Swarm OS v19.001 "Sovereign"
// Kleene K3 Balanced Ternary State for TRM-Fang Gateway

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum K3State {
    True,    // +1 — Established, active, verified
    Unknown, //  0 — Potential, held in superposition, pending consensus
    False,   // -1 — Rejected, blocked, invalid
}

impl K3State {
    pub fn from_bool(b: bool) -> Self {
        if b { K3State::True } else { K3State::False }
    }

    pub fn to_bool(&self) -> Option<bool> {
        match self {
            K3State::True => Some(true),
            K3State::False => Some(false),
            K3State::Unknown => None,
        }
    }

    pub fn is_actionable(&self) -> bool {
        *self == K3State::True
    }
}
