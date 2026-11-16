"""
Swarm OS v12 - Resource Marketplace Engine
Phase 8 component for the tokenized micro-economy.
Simulates a smart contract matching resource demands with offers using SWRM tokens.
"""

import hashlib
import logging
import time
from typing import Dict, Optional

logger = logging.getLogger("MarketEngine")

class ResourceOffer:
    def __init__(self, provider: str, cpu_units: int, price_per_unit: int, available_units: int):
        self.provider = provider
        self.cpu_units = cpu_units
        self.price_per_unit = price_per_unit
        self.available_units = available_units
        self.active = True

class ResourceDemand:
    def __init__(self, consumer: str, cpu_units: int, max_budget: int):
        self.consumer = consumer
        self.cpu_units = cpu_units
        self.max_budget = max_budget
        self.active = True

class MarketEngine:
    def __init__(self):
        self.offers: Dict[str, ResourceOffer] = {}
        self.demands: Dict[str, ResourceDemand] = {}
        self.balances: Dict[str, int] = {} # Mock token balances
        
        # Initial liquidity
        self.balances["system_faucet"] = 1_000_000
        logger.info("[Market] Initialized Swarm Resource Market (SWRM)")

    def fund_account(self, address: str, amount: int):
        self.balances[address] = self.balances.get(address, 0) + amount

    def create_offer(self, provider: str, cpu_units: int, price_per_unit: int, available_units: int) -> str:
        offer_id = hashlib.sha256(f"{provider}:{time.time()}".encode()).hexdigest()[:12]
        self.offers[offer_id] = ResourceOffer(provider, cpu_units, price_per_unit, available_units)
        logger.info(f"[Market] Offer {offer_id} created by {provider}")
        return offer_id

    def create_demand(self, consumer: str, cpu_units: int, max_budget: int) -> str:
        if self.balances.get(consumer, 0) < max_budget:
            logger.error(f"[Market] Demand creation failed for {consumer}: Insufficient SWRM balance.")
            return ""
            
        demand_id = hashlib.sha256(f"{consumer}:{time.time()}".encode()).hexdigest()[:12]
        self.demands[demand_id] = ResourceDemand(consumer, cpu_units, max_budget)
        logger.info(f"[Market] Demand {demand_id} created by {consumer}")
        return demand_id

    def execute_trade(self, offer_id: str, demand_id: str, units: int) -> bool:
        if offer_id not in self.offers or demand_id not in self.demands:
            return False
            
        offer = self.offers[offer_id]
        demand = self.demands[demand_id]
        
        if not offer.active or not demand.active:
            return False
            
        if units > offer.available_units:
            logger.warning(f"[Market] Insufficient offer units for trade.")
            return False
            
        total_price = units * offer.price_per_unit
        if total_price > demand.max_budget:
            logger.warning(f"[Market] Trade exceeds demand budget.")
            return False
            
        # Execute Token Transfer
        self.balances[demand.consumer] -= total_price
        self.balances[offer.provider] = self.balances.get(offer.provider, 0) + total_price
        
        # Update state
        offer.available_units -= units
        demand.max_budget -= total_price
        
        if offer.available_units == 0:
            offer.active = False
        if demand.max_budget == 0:
            demand.active = False
            
        logger.info(f"[Market] Trade Executed! {units} units exchanged for {total_price} SWRM.")
        return True

# Singleton
_market_engine = MarketEngine()

def get_market_engine() -> MarketEngine:
    return _market_engine
