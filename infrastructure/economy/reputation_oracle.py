"""
Swarm OS v12 - Reputation Oracle
Phase 8 component for the decentralized economy.
Computes trust scores based on historical node reliability and successful trades.
"""

import logging
from typing import Dict

logger = logging.getLogger("ReputationOracle")

class ReputationRecord:
    def __init__(self, agent_id: str):
        self.agent_id = agent_id
        self.score = 1000 # Base score
        self.total_trades = 0
        self.successful_trades = 0
        self.failed_trades = 0

    @property
    def reliability(self) -> float:
        if self.total_trades == 0:
            return 50.0
        return (self.successful_trades / self.total_trades) * 100.0

class ReputationOracle:
    def __init__(self):
        self.records: Dict[str, ReputationRecord] = {}
        logger.info("[Reputation] Initialized Reputation Oracle.")

    def get_or_create_record(self, agent_id: str) -> ReputationRecord:
        if agent_id not in self.records:
            self.records[agent_id] = ReputationRecord(agent_id)
        return self.records[agent_id]

    def record_trade_result(self, agent_id: str, success: bool):
        record = self.get_or_create_record(agent_id)
        record.total_trades += 1
        
        if success:
            record.successful_trades += 1
            record.score = min(2000, record.score + 10)
            logger.debug(f"[Reputation] +10 Reputation for {agent_id}. New score: {record.score}")
        else:
            record.failed_trades += 1
            # Slash penalty
            slash_amount = int(record.score * 0.10)
            record.score = max(0, record.score - slash_amount)
            logger.warning(f"[Reputation] SLASHED {slash_amount} Reputation from {agent_id}. New score: {record.score}")

    def compute_reputation(self, agent_id: str) -> dict:
        record = self.get_or_create_record(agent_id)
        return {
            "agent_id": agent_id,
            "score": record.score,
            "reliability": round(record.reliability, 2),
            "total_trades": record.total_trades
        }

# Singleton
_reputation_oracle = ReputationOracle()

def get_reputation_oracle() -> ReputationOracle:
    return _reputation_oracle
