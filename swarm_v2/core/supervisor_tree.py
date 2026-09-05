"""
supervisor_tree.py — TRM Swarm OS v19.001 "Sovereign"
======================================================
Erlang/OTP-Style Hierarchical Process Supervisor Engine:
- Manages lifecycles of autonomous subagents and worker tasks.
- Restart Strategies:
  * ONE_FOR_ONE: If a child crashes, only that child is restarted.
  * ONE_FOR_ALL: If any child crashes, all active sibling children are restarted.
  * REST_FOR_ONE: If a child crashes, children started after it in the sequence are restarted.
- Maximum failure intensity rate limiting (e.g. max 5 restarts in 60s).
- Heartbeat monitoring and state checkpointing prior to restart.
"""

import time
import uuid
import logging
from enum import Enum
from typing import Dict, List, Callable, Optional, Any
from dataclasses import dataclass, field

logger = logging.getLogger(__name__)

class RestartStrategy(Enum):
    ONE_FOR_ONE = "one_for_one"
    ONE_FOR_ALL = "one_for_all"
    REST_FOR_ONE = "rest_for_one"

class ChildStatus(Enum):
    STOPPED = "stopped"
    RUNNING = "running"
    FAILED = "failed"
    RESTARTING = "restarting"

@dataclass
class ChildSpec:
    child_id: str
    start_fn: Callable[[], Any]
    stop_fn: Optional[Callable[[Any], None]] = None
    max_retries: int = 3
    retry_delay_sec: float = 0.05
    state: Dict[str, Any] = field(default_factory=dict)
    instance: Optional[Any] = None
    status: ChildStatus = ChildStatus.STOPPED
    restart_count: int = 0
    last_heartbeat: float = 0.0
    failure_history: List[float] = field(default_factory=list)

class SupervisorTree:
    """
    Hierarchical process supervisor for resilient, deterministic autonomous task management.
    """
    def __init__(
        self,
        name: str = "RootSupervisor",
        strategy: RestartStrategy = RestartStrategy.ONE_FOR_ONE,
        max_restarts: int = 5,
        restart_window_sec: float = 60.0
    ):
        self.name = name
        self.strategy = strategy
        self.max_restarts = max_restarts
        self.restart_window_sec = restart_window_sec
        self.children: Dict[str, ChildSpec] = {}
        self.child_order: List[str] = []
        self.supervisor_failures: List[float] = []
        self.is_active: bool = False

    def add_child(
        self,
        child_id: str,
        start_fn: Callable[[], Any],
        stop_fn: Optional[Callable[[Any], None]] = None,
        initial_state: Optional[Dict[str, Any]] = None,
        max_retries: int = 3
    ) -> ChildSpec:
        """Register a new child worker specification under this supervisor."""
        if child_id in self.children:
            raise ValueError(f"Child {child_id} already registered in supervisor {self.name}")

        spec = ChildSpec(
            child_id=child_id,
            start_fn=start_fn,
            stop_fn=stop_fn,
            state=initial_state or {},
            max_retries=max_retries
        )
        self.children[child_id] = spec
        self.child_order.append(child_id)
        return spec

    def start_all(self) -> Dict[str, bool]:
        """Starts all registered children in registration order."""
        self.is_active = True
        results = {}
        for child_id in self.child_order:
            results[child_id] = self._start_child(child_id)
        return results

    def _start_child(self, child_id: str) -> bool:
        spec = self.children.get(child_id)
        if not spec:
            return False

        try:
            spec.instance = spec.start_fn()
            spec.status = ChildStatus.RUNNING
            spec.last_heartbeat = time.time()
            logger.info(f"[{self.name}] Child {child_id} started successfully.")
            return True
        except Exception as e:
            spec.status = ChildStatus.FAILED
            spec.failure_history.append(time.time())
            logger.error(f"[{self.name}] Failed to start child {child_id}: {e}")
            return False

    def _stop_child(self, child_id: str) -> bool:
        spec = self.children.get(child_id)
        if not spec or spec.status == ChildStatus.STOPPED:
            return True

        if spec.stop_fn and spec.instance:
            try:
                spec.stop_fn(spec.instance)
            except Exception as e:
                logger.warning(f"[{self.name}] Error during child {child_id} stop: {e}")

        spec.status = ChildStatus.STOPPED
        spec.instance = None
        return True

    def record_heartbeat(self, child_id: str, state_update: Optional[Dict[str, Any]] = None):
        """Records a successful heartbeat and optional state checkpoint."""
        spec = self.children.get(child_id)
        if spec and spec.status == ChildStatus.RUNNING:
            spec.last_heartbeat = time.time()
            if state_update:
                spec.state.update(state_update)

    def handle_child_failure(self, child_id: str, error_context: Optional[str] = None) -> bool:
        """
        Handles worker crash according to the supervisor's restart strategy.
        Enforces maximum failure rate limits.
        """
        now = time.time()
        self.supervisor_failures.append(now)
        # Purge failures outside window
        self.supervisor_failures = [t for t in self.supervisor_failures if now - t <= self.restart_window_sec]

        if len(self.supervisor_failures) > self.max_restarts:
            logger.critical(
                f"[{self.name}] Exceeded max restarts ({self.max_restarts} in {self.restart_window_sec}s). "
                "Escalating supervisor shutdown."
            )
            self.stop_all()
            return False

        spec = self.children.get(child_id)
        if not spec:
            return False

        spec.status = ChildStatus.FAILED
        spec.restart_count += 1
        spec.failure_history.append(now)

        if self.strategy == RestartStrategy.ONE_FOR_ONE:
            # Restart only the failed child
            logger.info(f"[{self.name}] Strategy ONE_FOR_ONE: Restarting child {child_id}")
            self._stop_child(child_id)
            time.sleep(spec.retry_delay_sec)
            return self._start_child(child_id)

        elif self.strategy == RestartStrategy.ONE_FOR_ALL:
            # Restart all sibling children
            logger.info(f"[{self.name}] Strategy ONE_FOR_ALL: Restarting all children due to {child_id} failure")
            for cid in self.child_order:
                self._stop_child(cid)
            time.sleep(spec.retry_delay_sec)
            all_ok = True
            for cid in self.child_order:
                if not self._start_child(cid):
                    all_ok = False
            return all_ok

        elif self.strategy == RestartStrategy.REST_FOR_ONE:
            # Restart the failed child and all children registered after it
            idx = self.child_order.index(child_id)
            targets = self.child_order[idx:]
            logger.info(f"[{self.name}] Strategy REST_FOR_ONE: Restarting {targets}")
            for cid in targets:
                self._stop_child(cid)
            time.sleep(spec.retry_delay_sec)
            all_ok = True
            for cid in targets:
                if not self._start_child(cid):
                    all_ok = False
            return all_ok

        return False

    def stop_all(self) -> Dict[str, bool]:
        """Stops all children in reverse registration order."""
        results = {}
        for child_id in reversed(self.child_order):
            results[child_id] = self._stop_child(child_id)
        self.is_active = False
        return results

    def get_health_report(self) -> Dict[str, Any]:
        """Returns health status of the supervisor and all child workers."""
        now = time.time()
        child_reports = {}
        for cid, spec in self.children.items():
            child_reports[cid] = {
                "status": spec.status.value,
                "restart_count": spec.restart_count,
                "seconds_since_last_heartbeat": round(now - spec.last_heartbeat, 3) if spec.last_heartbeat > 0 else None,
                "state_keys": list(spec.state.keys())
            }

        return {
            "supervisor_name": self.name,
            "strategy": self.strategy.value,
            "is_active": self.is_active,
            "total_children": len(self.children),
            "recent_supervisor_failures": len(self.supervisor_failures),
            "children": child_reports
        }
