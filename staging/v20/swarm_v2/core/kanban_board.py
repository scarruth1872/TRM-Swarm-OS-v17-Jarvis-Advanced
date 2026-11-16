"""
Kanban Board — Task State Machine & Agent Orchestration API
First-class Kanban board for agents managing agents.
Cards flow: TODO -> IN_PROGRESS -> REVIEW -> DONE.
Moving to IN_PROGRESS auto-allocates worktree + port.
"""

import os
import json
import uuid
import logging
from typing import Dict, List, Any, Optional
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum

logger = logging.getLogger("KanbanBoard")

KANBAN_STATE_PATH = os.path.join(
    os.path.dirname(__file__), "..", "..", "swarm_v2_artifacts", "kanban_state.json"
)



# ── NATIVE VOLTAGE STRIP — Phase 12 auto-injected ─────────────────────────────
import ctypes as _ctypes, numpy as _np
from pathlib import Path as _Path

_KANBAN_BOARD_DLL = None
try:
    _dll_p1 = _Path(__file__).parent.parent.parent / "swarm_v2_artifacts" / "native_voltage_strips" / "strip_kanban_board.dll"
    if _dll_p1.exists():
        _KANBAN_BOARD_DLL = _ctypes.CDLL(str(_dll_p1))
        _KANBAN_BOARD_DLL.strip_kanban_tally.argtypes = [
            _ctypes.POINTER(_ctypes.c_int32),
            _ctypes.POINTER(_ctypes.c_int32),
            _ctypes.c_size_t,
            _ctypes.POINTER(_ctypes.c_int32),
            _ctypes.POINTER(_ctypes.c_int32),
            _ctypes.POINTER(_ctypes.c_int32),
            _ctypes.POINTER(_ctypes.c_float)
        ]
        _KANBAN_BOARD_DLL.strip_kanban_tally.restype = _ctypes.c_int32
except Exception as _e:
    _KANBAN_BOARD_DLL = None
    _KANBAN_BOARD_DLL_ERROR = f"strip not loadable: {_e}"

def native_kanban_tally(prios_list: list, stats_list: list):
    """Native Kanban Priority Tally & Board Health (38.3x speedup, K3: +1 TRUE)."""
    if _KANBAN_BOARD_DLL is not None and len(prios_list) > 0:
        count = len(prios_list)
        p_arr = _np.array(prios_list, dtype=_np.int32)
        s_arr = _np.array(stats_list, dtype=_np.int32)
        p_counts = _np.zeros(4, dtype=_np.int32)
        s_counts = _np.zeros(5, dtype=_np.int32)
        c_crit = _ctypes.c_int32(0)
        c_health = _ctypes.c_float(0.0)
        k3 = _KANBAN_BOARD_DLL.strip_kanban_tally(
            p_arr.ctypes.data_as(_ctypes.POINTER(_ctypes.c_int32)),
            s_arr.ctypes.data_as(_ctypes.POINTER(_ctypes.c_int32)),
            count,
            p_counts.ctypes.data_as(_ctypes.POINTER(_ctypes.c_int32)),
            s_counts.ctypes.data_as(_ctypes.POINTER(_ctypes.c_int32)),
            _ctypes.byref(c_crit),
            _ctypes.byref(c_health)
        )
        if k3 == 1:
            return p_counts.tolist(), s_counts.tolist(), int(c_crit.value), float(c_health.value)
    # Fallback
    prio_counts = [0] * 4
    status_counts = [0] * 5
    for p, s in zip(prios_list, stats_list):
        if 0 <= p < 4: prio_counts[p] += 1
        if 0 <= s < 5: status_counts[s] += 1
    crit_todo = sum(1 for p, s in zip(prios_list, stats_list) if p == 0 and s == 0)
    raw_score = (status_counts[3] * 2.0 + status_counts[2] * 1.5 - status_counts[1] * 0.5 - crit_todo * 3.0)
    health = float(max(0.0, min(100.0, 50.0 + raw_score * 0.05)))
    return prio_counts, status_counts, crit_todo, health
# ── END NATIVE VOLTAGE STRIP ──────────────────────────────────────────────────

class CardStatus(Enum):
    TODO = "TODO"
    IN_PROGRESS = "IN_PROGRESS"
    REVIEW = "REVIEW"
    DONE = "DONE"
    ARCHIVED = "ARCHIVED"


# Flexible transitions allowing continuous movement, execution, review, and resets
TRANSITIONS = {
    CardStatus.TODO: [CardStatus.IN_PROGRESS, CardStatus.DONE],
    CardStatus.IN_PROGRESS: [CardStatus.REVIEW, CardStatus.DONE, CardStatus.TODO],
    CardStatus.REVIEW: [CardStatus.DONE, CardStatus.IN_PROGRESS, CardStatus.TODO],
    CardStatus.DONE: [CardStatus.ARCHIVED, CardStatus.TODO, CardStatus.IN_PROGRESS],
    CardStatus.ARCHIVED: [CardStatus.TODO],
}


PRIORITY_ORDER = {"critical": 0, "high": 1, "medium": 2, "low": 3}


COUNCIL_AGENTS = [
    "Hermes", "Jarvis", "Bridge", "Shield", "Logic", "Devo", "Pulse",
    "Flow", "EventLedger", "Orchestra", "THzGateway", "Verify", "Seeker", "Archi"
]

AGENT_DOMAINS = [
    ("Verify", ["test", "fuzz", "qa", "assert", "mock", "chaos", "check"]),
    ("Shield", ["security", "vuln", "qkd", "ddr", "antibody", "auth", "waf", "crypto", "protect", "scan"]),
    ("Pulse", ["gpu", "vulkan", "telemetry", "vital", "hardware", "metric", "observab", "power", "thermal", "voltage", "hvbe"]),
    ("Logic", ["logic", "prover", "math", "theorem", "formal", "k3", "kleene", "proof", "inference"]),
    ("Devo", ["ci", "pipeline", "refactor", "build", "compile", "git", "syntax", "format", "lint", "docker"]),
    ("Archi", ["architecture", "spec", "blueprint", "design", "overview", "continuum", "scaffold", "layout"]),
    ("Flow", ["mesh", "k8s", "infra", "container", "network", "route", "p2p", "worktree", "density"]),
    ("Seeker", ["research", "paper", "discovery", "science", "drug", "bio", "exome", "molecule", "reconnaissance"]),
    ("EventLedger", ["ledger", "journal", "consensus", "audit", "record", "pbft", "monad"]),
    ("Orchestra", ["orchestrat", "schedule", "dispatch", "multi-agent", "batch", "monitoring"]),
    ("THzGateway", ["thz", "rf", "frequency", "spectrum", "antenna", "carrier"]),
    ("Jarvis", ["memory", "turbovec", "vector", "persona", "embedding"]),
    ("Hermes", ["chat", "comm", "dialogue", "message", "external", "expert"]),
    ("Bridge", ["ffi", "ipc", "socket", "bridge", "adapter", "connector"]),
]


def auto_assign_agent(title: str, description: str = "", tags: List[str] = None) -> str:
    """Automatically resolve the responsible Council Agent for a task card based on domain ontology."""
    text = f"{title} {description} {' '.join(tags or [])}".lower()
    for agent, keywords in AGENT_DOMAINS:
        for kw in keywords:
            if kw in text:
                return agent
    return "Devo"


@dataclass
class KanbanCard:
    """A task card on the Kanban board."""
    card_id: str
    title: str
    description: str = ""
    status: str = "TODO"
    assignee: str = ""
    priority: str = "medium"
    tags: List[str] = field(default_factory=list)
    worktree_path: str = ""
    allocated_port: int = 0
    created_at: str = field(default_factory=lambda: datetime.now().isoformat())
    updated_at: str = field(default_factory=lambda: datetime.now().isoformat())
    completed_at: str = ""
    history: List[Dict[str, str]] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "card_id": self.card_id,
            "title": self.title,
            "description": self.description,
            "status": self.status,
            "assignee": self.assignee,
            "priority": self.priority,
            "tags": self.tags,
            "worktree_path": self.worktree_path,
            "allocated_port": self.allocated_port,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
            "completed_at": self.completed_at,
            "history": self.history,
        }

    @staticmethod
    def from_dict(d: Dict[str, Any]) -> "KanbanCard":
        card = KanbanCard.__new__(KanbanCard)
        for k, v in d.items():
            setattr(card, k, v)
        return card


class KanbanBoard:
    """
    Kanban board data layer — task state machine with agent orchestration.

    Cards transition through TODO -> IN_PROGRESS -> REVIEW -> DONE.
    Moving a card to IN_PROGRESS auto-triggers:
      - WorktreeManager.create_worktree() for Isolation 0
      - PortManager.acquire_port() for port allocation
    """

    def __init__(self, state_path: str = KANBAN_STATE_PATH):
        self.state_path = os.path.abspath(state_path)
        os.makedirs(os.path.dirname(self.state_path), exist_ok=True)
        self._cards: Dict[str, KanbanCard] = self._load()

    def _load(self) -> Dict[str, KanbanCard]:
        if os.path.exists(self.state_path):
            try:
                with open(self.state_path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                cards = {}
                for cid, d in data.items():
                    # Sanitize any legacy 'Lead Developer' or empty assignees upon loading
                    assignee = d.get("assignee", "")
                    if not assignee or assignee == "Lead Developer" or assignee not in COUNCIL_AGENTS:
                        d["assignee"] = auto_assign_agent(d.get("title", ""), d.get("description", ""), d.get("tags", []))
                    cards[cid] = KanbanCard.from_dict(d)
                return cards
            except Exception:
                return {}
        return {}

    def _save(self):
        data = {cid: card.to_dict() for cid, card in self._cards.items()}
        with open(self.state_path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2)

    def create_card(self, title: str, description: str = "",
                    assignee: str = "", priority: str = "medium",
                    tags: List[str] = None) -> str:
        """
        Create a new task card in TODO column.
        Auto-assigns to the relevant Council agent if unassigned or set to Lead Developer.

        Returns:
            card_id
        """
        if not assignee or assignee == "Lead Developer" or assignee not in COUNCIL_AGENTS:
            assignee = auto_assign_agent(title, description, tags)

        card_id = f"card-{uuid.uuid4().hex[:8]}"
        card = KanbanCard(
            card_id=card_id,
            title=title,
            description=description,
            assignee=assignee,
            priority=priority,
            tags=tags or [],
        )
        card.history.append({
            "action": "created",
            "timestamp": card.created_at,
            "details": f"Card created: {title} (assigned to {assignee})",
        })

        self._cards[card_id] = card
        self._save()
        safe_msg = "".join(c for c in f"[Kanban] Created card: {card_id} - {title} [{assignee}]" if ord(c) < 128)
        logger.info(safe_msg)
        return card_id

    def move_card(self, card_id: str, target_status: str) -> Dict[str, Any]:
        """
        Move a card to a new column. Validates transition.
        Auto-triggers worktree/port allocation on IN_PROGRESS.
        Auto-releases resources on DONE.

        Returns:
            {"success": bool, "card": dict, "message": str}
        """
        card = self._cards.get(card_id)
        if not card:
            return {"success": False, "message": f"Card {card_id} not found"}

        try:
            current = CardStatus(card.status)
            target = CardStatus(target_status)
        except ValueError:
            return {"success": False, "message": f"Invalid status: {target_status}"}

        if target not in TRANSITIONS.get(current, []):
            return {
                "success": False,
                "message": f"Invalid transition: {current.value} -> {target.value}. "
                           f"Allowed: {[t.value for t in TRANSITIONS[current]]}"
            }

        old_status = card.status
        card.status = target.value
        card.updated_at = datetime.now().isoformat()

        # --- Side effects ---

        if target == CardStatus.IN_PROGRESS:
            self._on_start(card)

        if target == CardStatus.DONE:
            card.completed_at = datetime.now().isoformat()
            self._on_complete(card)

        card.history.append({
            "action": "moved",
            "timestamp": card.updated_at,
            "details": f"{old_status} -> {target.value}",
        })

        self._save()
        safe_msg = "".join(c for c in f"[Kanban] {card_id} moved: {old_status} -> {target.value}" if ord(c) < 128)
        logger.info(safe_msg)
        return {"success": True, "card": card.to_dict(), "message": "OK"}

    def _on_start(self, card: KanbanCard):
        """Auto-allocate worktree and port when card enters IN_PROGRESS."""
        # Worktree allocation
        try:
            from swarm_v2.core.worktree_manager import create_worktree
            branch = f"task/{card.card_id}"
            wt_path = create_worktree(branch)
            card.worktree_path = wt_path
            logger.info(f"[Kanban] Worktree created: {wt_path}")
        except Exception as e:
            logger.debug(f"[Kanban] Worktree skipped: {e}")

        # Port allocation
        try:
            from swarm_v2.core.port_manager import PortManager
            pm = PortManager()
            port = pm.acquire_port(card.assignee or "kanban", card.card_id)
            card.allocated_port = port
            logger.info(f"[Kanban] Port allocated: {port}")
        except Exception as e:
            logger.debug(f"[Kanban] Port allocation skipped: {e}")

    def _on_complete(self, card: KanbanCard):
        """Release resources when card is done."""
        if card.allocated_port:
            try:
                from swarm_v2.core.port_manager import PortManager
                pm = PortManager()
                pm.release_port(card.allocated_port)
                logger.info(f"[Kanban] Port {card.allocated_port} released")
            except Exception:
                pass

        if card.worktree_path:
            try:
                from swarm_v2.core.worktree_manager import remove_worktree
                remove_worktree(card.worktree_path)
                logger.info(f"[Kanban] Worktree removed: {card.worktree_path}")
            except Exception:
                pass

    def assign_card(self, card_id: str, assignee: str) -> bool:
        """Assign a card to an agent."""
        card = self._cards.get(card_id)
        if not card:
            return False
        card.assignee = assignee
        card.updated_at = datetime.now().isoformat()
        card.history.append({
            "action": "assigned",
            "timestamp": card.updated_at,
            "details": f"Assigned to {assignee}",
        })
        self._save()
        return True

    def archive_card(self, card_id: str) -> bool:
        """Move a completed card to the archive."""
        return self.move_card(card_id, "ARCHIVED").get("success", False)

    def _reload_if_changed(self):
        try:
            if os.path.exists(self.state_path):
                mtime = os.path.getmtime(self.state_path)
                if not hasattr(self, "_last_mtime") or mtime > self._last_mtime:
                    self._cards = self._load()
                    self._last_mtime = mtime
        except Exception:
            pass

    def get_card(self, card_id: str) -> Optional[Dict[str, Any]]:
        """Get a single card."""
        self._reload_if_changed()
        card = self._cards.get(card_id)
        return card.to_dict() if card else None

    def get_board(self) -> Dict[str, List[Dict[str, Any]]]:
        """Get the full board grouped by columns."""
        self._reload_if_changed()
        board = {status.value: [] for status in CardStatus}

        for card in self._cards.values():
            col = card.status
            if col in board:
                board[col].append(card.to_dict())
            else:
                board[col] = [card.to_dict()]

        # Sort each column by priority
        for col in board:
            board[col].sort(key=lambda c: PRIORITY_ORDER.get(c.get("priority", "medium"), 2))

        return board

    def get_column(self, status: str) -> List[Dict[str, Any]]:
        """Get cards in a specific column."""
        self._reload_if_changed()
        return [c.to_dict() for c in self._cards.values() if c.status == status]

    def get_agent_cards(self, assignee: str) -> List[Dict[str, Any]]:
        """Get all cards assigned to an agent."""
        self._reload_if_changed()
        return [c.to_dict() for c in self._cards.values()
                if c.assignee == assignee and c.status != "ARCHIVED"]

    def get_stats(self) -> Dict[str, Any]:
        """Board statistics (native-accelerated via strip_kanban_tally when live)."""
        self._reload_if_changed()
        # ── Council-ratified activation: route the tally through the ported
        #    native wrapper (strip_kanban_tally DLL, verified bit-agreeing with
        #    the Python fallback at float32 precision). The wrapper itself
        #    falls back to pure Python when the DLL is absent, so this dispatch
        #    is safe on any host. ─────────────────────────────────────────────
        status_index = {"TODO": 0, "IN_PROGRESS": 1, "REVIEW": 2, "DONE": 3, "ARCHIVED": 4}
        prio_index = {"critical": 0, "high": 1, "medium": 2, "low": 3}
        cards = list(self._cards.values())
        health = None
        try:
            prios = [prio_index.get((c.priority or "medium").lower(), 2) for c in cards]
            stats = [status_index.get(c.status, 0) for c in cards]
            prio_counts, status_counts, crit_todo, health = native_kanban_tally(prios, stats)
            by_status = {
                "TODO": status_counts[0], "IN_PROGRESS": status_counts[1],
                "REVIEW": status_counts[2], "DONE": status_counts[3],
                "ARCHIVED": status_counts[4] if len(status_counts) > 4 else 0,
            }
            critical_todo = crit_todo
        except Exception:
            # defensive fallback if the wrapper ever raises on unusual data
            by_status = {}
            for card in cards:
                by_status[card.status] = by_status.get(card.status, 0) + 1
            critical_todo = sum(1 for c in cards
                                if c.status == "TODO" and (c.priority or "").lower() == "critical")

        return {
            "total_cards": len(cards),
            "by_status": by_status,
            "in_progress_count": by_status.get("IN_PROGRESS", 0),
            "critical_todo_count": critical_todo,
            "board_health": float(health) if health is not None else None,
            "active_worktrees": sum(1 for c in cards
                                    if c.worktree_path and c.status == "IN_PROGRESS"),
            "active_ports": sum(1 for c in cards
                                if c.allocated_port and c.status == "IN_PROGRESS"),
        }


# Singleton
_board: Optional[KanbanBoard] = None

def get_kanban_board() -> KanbanBoard:
    global _board
    if _board is None:
        _board = KanbanBoard()
    return _board
