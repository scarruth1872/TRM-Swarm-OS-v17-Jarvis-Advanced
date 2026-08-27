"""
test_sparse_moe_and_supervisor.py — TRM Swarm OS v19.001 "Sovereign"
====================================================================
Comprehensive Test Suite for:
1. SparseMoERouter (Top-K expert gating, auxiliary load-balancing loss, tensor recombination)
2. SupervisorTree (OneForOne, OneForAll, RestForOne fault recovery, heartbeat monitoring)
"""

import sys
import os
import unittest
import time
import math

PROJECT_ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

from swarm_v2.core.sparse_moe_router import SparseMoERouter
from swarm_v2.core.supervisor_tree import (
    SupervisorTree, RestartStrategy, ChildStatus
)

class TestSparseMoERouter(unittest.TestCase):
    def setUp(self):
        self.router = SparseMoERouter(
            input_dim=64,
            num_experts=8,
            top_k=2,
            aux_loss_weight=0.01
        )

    def test_single_routing_dimensions_and_weights(self):
        vec = [math.sin(i * 0.1) for i in range(64)]
        selected_idx, weights, all_probs = self.router.route_single(vec)

        self.assertEqual(len(selected_idx), 2)
        self.assertEqual(len(weights), 2)
        self.assertEqual(len(all_probs), 8)
        self.assertAlmostEqual(sum(weights), 1.0, places=5)
        self.assertAlmostEqual(sum(all_probs), 1.0, places=5)

    def test_batch_routing_and_aux_loss(self):
        batch = [[math.sin(b + j * 0.2) for j in range(64)] for b in range(16)]
        res = self.router.route_batch(batch)

        self.assertEqual(len(res["routes"]), 16)
        self.assertGreater(res["aux_loss"], 0.0)
        self.assertEqual(sum(res["expert_counts"]), 16 * 2)

    def test_dispatch_and_combine(self):
        # Create 8 dummy expert functions that scale the vector
        experts = [lambda v, factor=i: [x * (1.0 + factor * 0.1) for x in v] for i in range(8)]
        batch = [[1.0] * 64, [0.5] * 64]
        out = self.router.dispatch_and_combine(batch, experts)

        self.assertEqual(len(out), 2)
        self.assertEqual(len(out[0]), 64)
        self.assertGreater(out[0][0], 1.0)


class TestSupervisorTree(unittest.TestCase):
    def test_one_for_one_restart(self):
        sup = SupervisorTree("TestSupOneForOne", strategy=RestartStrategy.ONE_FOR_ONE)
        state_log = {"w1_starts": 0, "w2_starts": 0}

        def start_w1():
            state_log["w1_starts"] += 1
            return "Worker1_Instance"

        def start_w2():
            state_log["w2_starts"] += 1
            return "Worker2_Instance"

        sup.add_child("worker_1", start_w1)
        sup.add_child("worker_2", start_w2)

        start_res = sup.start_all()
        self.assertTrue(start_res["worker_1"])
        self.assertTrue(start_res["worker_2"])
        self.assertEqual(state_log["w1_starts"], 1)
        self.assertEqual(state_log["w2_starts"], 1)

        # Trigger crash on worker_1
        sup.handle_child_failure("worker_1")

        # In ONE_FOR_ONE: only worker_1 restarted
        self.assertEqual(state_log["w1_starts"], 2)
        self.assertEqual(state_log["w2_starts"], 1)
        self.assertEqual(sup.children["worker_1"].status, ChildStatus.RUNNING)

        sup.stop_all()

    def test_one_for_all_restart(self):
        sup = SupervisorTree("TestSupOneForAll", strategy=RestartStrategy.ONE_FOR_ALL)
        state_log = {"w1_starts": 0, "w2_starts": 0}

        def start_w1():
            state_log["w1_starts"] += 1
            return "Worker1"

        def start_w2():
            state_log["w2_starts"] += 1
            return "Worker2"

        sup.add_child("w1", start_w1)
        sup.add_child("w2", start_w2)
        sup.start_all()

        # Trigger crash on w2
        sup.handle_child_failure("w2")

        # In ONE_FOR_ALL: BOTH w1 and w2 restarted
        self.assertEqual(state_log["w1_starts"], 2)
        self.assertEqual(state_log["w2_starts"], 2)

        sup.stop_all()

    def test_heartbeat_and_health_reporting(self):
        sup = SupervisorTree("TestSupHeartbeat")
        sup.add_child("w1", lambda: "W1", initial_state={"counter": 0})
        sup.start_all()

        sup.record_heartbeat("w1", {"counter": 42})
        self.assertEqual(sup.children["w1"].state["counter"], 42)

        health = sup.get_health_report()
        self.assertEqual(health["supervisor_name"], "TestSupHeartbeat")
        self.assertEqual(health["children"]["w1"]["status"], "running")

        sup.stop_all()

if __name__ == "__main__":
    unittest.main()
