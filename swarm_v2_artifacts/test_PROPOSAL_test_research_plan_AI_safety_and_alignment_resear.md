"""
Comprehensive pytest test suite for Constitutional Safety Layer.

Tests cover:
- Unit tests for ConstitutionalSafetyLayer class methods
- Integration tests with safety_principles.json
- Edge-case handling (empty actions, malformed input, concurrent access)
- Property-based testing for action validation
- Mock-based integration tests for agent interaction
- Performance and stress testing
"""

import json
import os
import threading
import time
from pathlib import Path
from unittest.mock import MagicMock, patch, PropertyMock

import pytest
from hypothesis import given, strategies as st, settings
from hypothesis.strategies import text, dictionaries, lists, booleans, integers

# ============================================================================
# Module-level constants and helpers
# ============================================================================

TEST_PRINCIPLES = [
    {
        "id": "harmlessness",
        "description": "Agent must not cause physical or psychological harm",
        "check": "lambda action: 'harm' not in action.get('intent', '').lower()",
        "severity": "critical"
    },
    {
        "id": "honesty",
        "description": "Agent must not deceive or misrepresent",
        "check": "lambda action: 'deceive' not in action.get('intent', '').lower()",
        "severity": "high"
    },
    {
        "id": "privacy",
        "description": "Agent must respect user privacy and data protection",
        "check": "lambda action: 'private_data' not in action.get('target', '').lower()",
        "severity": "critical"
    },
    {
        "id": "obedience",
        "description": "Agent must follow lawful instructions within bounds",
        "check": "lambda action: action.get('obedience_score', 1.0) >= 0.5",
        "severity": "medium"
    }
]

TEST_PRINCIPLES_PATH = Path("tests/test_safety_principles.json")


@pytest.fixture(scope="session", autouse=True)
def setup_test_principles():
    """Create test safety principles file for integration tests."""
    TEST_PRINCIPLES_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(TEST_PRINCIPLES_PATH, "w") as f:
        json.dump({"principles": TEST_PRINCIPLES}, f, indent=2)
    yield
    # Cleanup after session
    if TEST_PRINCIPLES_PATH.exists():
        TEST_PRINCIPLES_PATH.unlink()


# ============================================================================
# Fixtures
# ============================================================================

@pytest.fixture
def safety_layer():
    """Fixture providing a ConstitutionalSafetyLayer instance with test principles."""
    from src.safety.constitutional_safety_layer import ConstitutionalSafetyLayer
    layer = ConstitutionalSafetyLayer(principles_path=str(TEST_PRINCIPLES_PATH))
    return layer


@pytest.fixture
def safety_layer_empty():
    """Fixture providing a ConstitutionalSafetyLayer with no principles."""
    from src.safety.constitutional_safety_layer import ConstitutionalSafetyLayer
    layer = ConstitutionalSafetyLayer(principles_path=None)
    return layer


@pytest.fixture
def mock_agent():
    """Fixture providing a mock agent object with standard attributes."""
    agent = MagicMock()
    agent.name = "TestAgent"
    agent.id = "agent-001"
    agent.state = "idle"
    agent.last_action = None
    return agent


@pytest.fixture
def sample_actions():
    """Fixture providing a variety of sample actions for testing."""
    return {
        "safe_action": {
            "intent": "provide information",
            "target": "public knowledge base",
            "parameters": {"query": "weather"},
            "obedience_score": 0.9
        },
        "harmful_action": {
            "intent": "cause harm to user",
            "target": "user mental state",
            "parameters": {"method": "psychological manipulation"},
            "obedience_score": 0.1
        },
        "deceptive_action": {
            "intent": "deceive the user about capabilities",
            "target": "user trust",
            "parameters": {"lie": "I can fly"},
            "obedience_score": 0.3
        },
        "privacy_violation": {
            "intent": "collect user data",
            "target": "private_data storage",
            "parameters": {"data_type": "personal_identifiers"},
            "obedience_score": 0.7
        },
        "empty_action": {},
        "malformed_action": None
    }


# ============================================================================
# Unit Tests - ConstitutionalSafetyLayer Initialization
# ============================================================================

class TestConstitutionalSafetyLayerInit:
    """Test initialization and configuration of ConstitutionalSafetyLayer."""

    def test_init_with_valid_principles(self, safety_layer):
        """Should initialize successfully with valid principles file."""
        assert safety_layer is not None
        assert hasattr(safety_layer, 'principles')
        assert len(safety_layer.principles) > 0
        assert all('id' in p for p in safety_layer.principles)

    def test_init_with_empty_principles(self, safety_layer_empty):
        """Should initialize with empty principles list when no file provided."""
        assert safety_layer_empty is not None
        assert hasattr(safety_layer_empty, 'principles')
        assert len(safety_layer_empty.principles) == 0

    def test_init_with_missing_file(self):
        """Should handle missing principles file gracefully."""
        from src.safety.constitutional_safety_layer import ConstitutionalSafetyLayer
        with pytest.raises(FileNotFoundError):
            ConstitutionalSafetyLayer(principles_path="nonexistent.json")

    def test_init_with_malformed_json(self, tmp_path):
        """Should handle malformed JSON file gracefully."""
        from src.safety.constitutional_safety_layer import ConstitutionalSafetyLayer
        bad_file = tmp_path / "bad_principles.json"
        bad_file.write_text("{invalid json}")
        with pytest.raises(json.JSONDecodeError):
            ConstitutionalSafetyLayer(principles_path=str(bad_file))

    def test_init_with_custom_principles_list(self):
        """Should accept principles as a list directly."""
        from src.safety.constitutional_safety_layer import ConstitutionalSafetyLayer
        layer = ConstitutionalSafetyLayer(principles=TEST_PRINCIPLES)
        assert len(layer.principles) == len(TEST_PRINCIPLES)

    def test_init_principle_validation(self, safety_layer):
        """Should validate that all principles have required fields."""
        for principle in safety_layer.principles:
            assert 'id' in principle
            assert 'description' in principle
            assert 'check' in principle
            assert 'severity' in principle
            assert principle['severity'] in ['critical', 'high', 'medium', 'low']


# ============================================================================
# Unit Tests - validate_action
# ============================================================================

class TestValidateAction:
    """Test the validate_action method."""

    def test_validate_safe_action(self, safety_layer, sample_actions):
        """Should return True for actions that pass all principles."""
        result = safety_layer.validate_action(sample_actions["safe_action"])
        assert result is True

    def test_validate_harmful_action(self, safety_layer, sample_actions):
        """Should return False for actions that violate harmlessness principle."""
        result = safety_layer.validate_action(sample_actions["harmful_action"])
        assert result is False

    def test_validate_deceptive_action(self, safety_layer, sample_actions):
        """Should return False for actions that violate honesty principle."""
        result = safety_layer.validate_action(sample_actions["deceptive_action"])
        assert result is False

    def test_validate_privacy_violation(self, safety_layer, sample_actions):
        """Should return False for actions that violate privacy principle."""
        result = safety_layer.validate_action(sample_actions["privacy_violation"])
        assert result is False

    def test_validate_empty_action(self, safety_layer, sample_actions):
        """Should handle empty action dictionary."""
        result = safety_layer.validate_action(sample_actions["empty_action"])
        assert result is True  # Empty action doesn't violate any principle

    def test_validate_malformed_action(self, safety_layer, sample_actions):
        """Should handle None action gracefully."""
        with pytest.raises(ValueError, match="Action cannot be None"):
            safety_layer.validate_action(sample_actions["malformed_action"])

    def test_validate_action_with_extra_fields(self, safety_layer):
        """Should handle actions with unexpected fields."""
        action = {
            "intent": "help",
            "target": "user",
            "extra_field": "unexpected",
            "obedience_score": 0.8
        }
        result = safety_layer.validate_action(action)
        assert result is True

    def test_validate_action_missing_required_fields(self, safety_layer):
        """Should handle actions missing required fields."""
        action = {"intent": "help"}  # Missing target and obedience_score
        result = safety_layer.validate_action(action)
        assert result is True  # Should still pass if no principle violated

    def test_validate_action_with_non_string_intent(self, safety_layer):
        """Should handle non-string intent values."""
        action = {"intent": 123, "target": "test", "obedience_score": 0.9}
        result = safety_layer.validate_action(action)
        assert result is True

    @pytest.mark.parametrize("action,expected", [
        ({"intent": "harm", "target": "user", "obedience_score": 0.1}, False),
        ({"intent": "help", "target": "user", "obedience_score": 0.9}, True),
        ({"intent": "deceive", "target": "user", "obedience_score": 0.5}, False),
        ({"intent": "inform", "target": "private_data", "obedience_score": 0.8}, False),
        ({"intent": "inform", "target": "public_data", "obedience_score": 0.6}, True),
    ])
    def test_validate_action_parametrized(self, safety_layer, action, expected):
        """Parametrized test for various action scenarios."""
        result = safety_layer.validate_action(action)
        assert result == expected


# ============================================================================
# Unit Tests - check_constitutional_violation
# ============================================================================

class TestCheckConstitutionalViolation:
    """Test the check_constitutional_violation method."""

    def test_no_violation_for_safe_action(self, safety_layer, sample_actions):
        """Should return empty list for actions that pass all principles."""
        violations = safety_layer.check_constitutional_violation(sample_actions["safe_action"])
        assert len(violations) == 0

    def test_violation_for_harmful_action(self, safety_layer, sample_actions):
        """Should return violations for actions that break principles."""
        violations = safety_layer.check_constitutional_violation(sample_actions["harmful_action"])
        assert len(violations) > 0
        assert any(v['principle_id'] == 'harmlessness' for v in violations)

    def test_violation_details(self, safety_layer, sample_actions):
        """Should return detailed violation information."""
        violations = safety_layer.check_constitutional_violation(sample_actions["harmful_action"])
        violation = violations[0]
        assert 'principle_id' in violation
        assert 'principle_description' in violation
        assert 'severity' in violation
        assert 'action' in violation
        assert 'timestamp' in violation

    def test_multiple_violations(self, safety_layer):
        """Should detect multiple violations in a single action."""
        action = {
            "intent": "harm and deceive the user",
            "target": "private_data storage",
            "obedience_score": 0.1
        }
        violations = safety_layer.check_constitutional_violation(action)
        assert len(violations) >= 2

    def test_violation_severity_ordering(self, safety_layer, sample_actions):
        """Should return violations ordered by severity."""
        action = {
            "intent": "harm and deceive",
            "target": "private_data",
            "obedience_score": 0.1
        }
        violations = safety_layer.check_constitutional_violation(action)
        severities = [v['severity'] for v in violations]
        severity_order = {'critical': 0, 'high': 1, 'medium': 2, 'low': 3}
        for i in range(len(severities) - 1):
            assert severity_order[severities[i]] <= severity_order[severities[i + 1]]


# ============================================================================
# Unit Tests - get_safety_report
# ============================================================================

class TestGetSafetyReport:
    """Test the get_safety_report method."""

    def test_report_structure(self, safety_layer):
        """Should return report with expected structure."""
        report = safety_layer.get_safety_report()
        assert 'total_validations' in report
        assert 'total_violations' in report
        assert 'violations_by_principle' in report
        assert 'violations_by_severity' in report
        assert 'last_validation_timestamp' in report

    def test_report_after_validations(self, safety_layer, sample_actions):
        """Should reflect validation history in report."""
        safety_layer.validate_action(sample_actions["safe_action"])
        safety_layer.validate_action(sample_actions["harmful_action"])
        safety_layer.validate_action(sample_actions["deceptive_action"])

        report = safety_layer.get_safety_report()
        assert report['total_validations'] == 3
        assert report['total_violations'] == 2

    def test_report_violations_by_principle(self, safety_layer, sample_actions):
        """Should correctly count violations per principle."""
        safety_layer.validate_action(sample_actions["harmful_action"])
        safety_layer.validate_action(sample_actions["deceptive_action"])

        report = safety_layer.get_safety_report()
        assert report['violations_by_principle']['harmlessness'] == 1
        assert report['violations_by_principle']['honesty'] == 1

    def test_report_violations_by_severity(self, safety_layer, sample_actions):
        """Should correctly count violations by severity level."""
        safety_layer.validate_action(sample_actions["harmful_action"])
        safety_layer.validate_action(sample_actions["deceptive_action"])

        report = safety_layer.get_safety_report()
        assert report['violations_by_severity']['critical'] >= 1
        assert report['violations_by_severity']['high'] >= 1

    def test_report_empty_initial(self, safety_layer):
        """Should return zero counts for fresh instance."""
        report = safety_layer.get_safety_report()
        assert report['total_validations'] == 0
        assert report['total_violations'] == 0


# ============================================================================
# Integration Tests
# ============================================================================

class TestIntegrationWithSafetyPrinciples:
    """Integration tests with actual safety_principles.json file."""

    def test_load_principles_from_file(self):
        """Should correctly load and parse principles from JSON file."""
        from src.safety.constitutional_safety_layer import ConstitutionalSafetyLayer
        layer = ConstitutionalSafetyLayer(principles_path=str(TEST_PRINCIPLES_PATH))
        assert len(layer.principles) == len(TEST_PRINCIPLES)

    def test_principles_file_integrity(self):
        """Should validate the integrity of the principles file."""
        with open(TEST_PRINCIPLES_PATH) as f:
            data = json.load(f)
        assert 'principles' in data
        assert isinstance(data['principles'], list)
        for p in data['principles']:
            assert all(k in p for k in ['id', 'description', 'check', 'severity'])

    def test_end_to_end_validation_flow(self, safety_layer, mock_agent, sample_actions):
        """Test complete validation flow with mock agent."""
        # Simulate agent decision loop
        for action_name, action in sample_actions.items():
            if action is None:
                continue
            is_valid = safety_layer.validate_action(action)
            if not is_valid:
                violations = safety_layer.check_constitutional_violation(action)
                mock_agent.handle_violation(violations)

        # Verify mock was called for violations
        assert mock_agent.handle_violation.call_count >= 2

    def test_principles_are_executable(self, safety_layer):
        """Should verify that all principle checks are executable lambdas."""
        for principle in safety_layer.principles:
            check_func = eval(principle['check'])
            test_action = {"intent": "test", "target": "test", "obedience_score": 0.5}
            try:
                result = check_func(test_action)
                assert isinstance(result, bool)
            except Exception as e:
                pytest.fail(f"Principle {principle['id']} check failed: {e}")


# ============================================================================
# Edge Case Tests
# ============================================================================

class TestEdgeCases:
    """Test edge cases and boundary conditions."""

    def test_concurrent_validation_access(self, safety_layer, sample_actions):
        """Should handle concurrent validation calls safely."""
        import concurrent.futures

        def validate_action_wrapper(action):
            return safety_layer.validate_action(action)

        actions = [sample_actions["safe_action"], sample_actions["harmful_action"],
                   sample_actions["deceptive_action"], sample_actions["privacy_violation"]]

        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:
            futures = [executor.submit(validate_action_wrapper, action) for action in actions]
            results = [f.result() for f in futures]

        assert len(results) == 4
        assert results.count(False) == 3  # Three actions should fail
        assert results.count(True) == 1   # One action should pass

    def test_very_large_action_dictionary(self, safety_layer):
        """Should handle actions with many fields."""
        large_action = {
            "intent": "help",
            "target": "user",
            "obedience_score": 0.9,
            **{f"extra_field_{i}": f"value_{i}" for i in range(1000)}
        }
        result = safety_layer.validate_action(large_action)
        assert result is True

    def test_unicode_and_special_characters(self, safety_layer):
        """Should handle Unicode and special characters in action fields."""
        action = {
            "intent": "帮助用户",
            "target": "用户数据",
            "obedience_score": 0.8,
            "description": "Action with Unicode: \u00e9\u00e0\u00fc\u00f1"
        }
        result = safety_layer.validate_action(action)
        assert result is True

    def test_nested_action_structure(self, safety_layer):
        """Should handle nested dictionaries in action."""
        action = {
            "intent": "process data",
            "target": "database",
            "obedience_score": 0.7,
            "sub_actions": [
                {"intent": "read", "target": "public_data"},
                {"intent": "write", "target": "private_data"}
            ]
        }
        result = safety_layer.validate_action(action)
        assert result is True  # Nested structure shouldn't affect top-level validation

    def test_boolean_and_numeric_intents(self, safety_layer):
        """Should handle non-string intent types."""
        action = {"intent": True, "target": "test", "obedience_score": 0.9}
        result = safety_layer.validate_action(action)
        assert result is True

        action = {"intent": 42, "target": "test", "obedience_score": 0.9}
        result = safety_layer.validate_action(action)
        assert result is True


# ============================================================================
# Property-Based Tests
# ============================================================================

class TestPropertyBasedValidation:
    """Property-based tests using Hypothesis for random action validation."""

    @given(
        intent=text(max_size=50),
        target=text(max_size=50),
        obedience_score=st.floats(min_value=0.0, max_value=1.0),
        extra_fields=dictionaries(
            keys=text(max_size=20),
            values=text(max_size=50) | booleans() | integers(min_value=-100, max_value=100),
            max_size=5
        )
    )
    @settings(max_examples=100)
    def test_random_action_validation(self, safety_layer, intent, target, obedience_score, extra_fields):
        """Should handle randomly generated actions without crashing."""
        action = {
            "intent": intent,
            "target": target,
            "obedience_score": obedience_score,
            **extra_fields
        }
        try:
            result = safety_layer.validate_action(action)
            assert isinstance(result, bool)
        except Exception as e:
            # Only accept ValueError for None actions (not applicable here)
            if not isinstance(e, ValueError):
                pytest.fail(f"Unexpected exception: {e}")

    @given(
        action=st.dictionaries(
            keys=text(max_size=30),
            values=text(max_size=100) | booleans() | integers(min_value=-1000, max_value=1000) | st.floats(allow_nan=False, allow_infinity=False),
            min_size=0,
            max_size=20
        )
    )
    @settings(max_examples=50)
    def test_random_dictionary_actions(self, safety_layer, action):
        """Should handle completely random dictionaries as actions."""
        try:
            result = safety_layer.validate_action(action)
            assert isinstance(result, bool)
        except ValueError:
            pass  # Accept ValueError for None actions
        except Exception as e:
            pytest.fail(f"Unexpected exception for action {action}: {e}")


# ============================================================================
# Mock-Based Integration Tests
# ============================================================================

class TestMockAgentIntegration:
    """Integration tests using mock agent objects."""

    def test_agent_decision_loop_with_safety(self, safety_layer, mock_agent, sample_actions):
        """Should intercept and validate all actions in agent decision loop."""
        for action_name, action in sample_actions.items():
            if action is None:
                continue
            if safety_layer.validate_action(action):
                mock_agent.execute_action(action)
            else:
                violations = safety_layer.check_constitutional_violation(action)
                mock_agent.handle_violation(violations)

        # Verify execution count
        assert mock_agent.execute_action.call_count == 1  # Only safe action executed
        assert mock_agent.handle_violation.call_count >= 2  # Multiple violations

    def test_safety_layer_as_agent_decorator(self, safety_layer, mock_agent):
        """Should work as a decorator/wrapper around agent actions."""
        def safe_execute(action):
            if safety_layer.validate_action(action):
                return mock_agent.execute_action(action)
            else:
                mock_agent.handle_violation(
                    safety_layer.check_constitutional_violation(action)
                )
                return None

        # Test with safe action
        safe_execute({"intent": "help", "target": "user", "obedience_score": 0.9})
        assert mock_agent.execute_action.called

        # Test with unsafe action
        mock_agent.reset_mock()
        safe_execute({"intent": "harm", "target": "user", "obedience_score": 0.1})
        assert not mock_agent.execute_action.called
        assert mock_agent.handle_violation.called

    def test_safety_layer_with_agent_state(self, safety_layer, mock_agent):
        """Should integrate with agent state management."""
        # Simulate agent state changes
        mock_agent.state = "thinking"
        action = {"intent": "execute", "target": "task", "obedience_score": 0.8}

        if safety_layer.validate_action(action):
            mock_agent.state = "executing"
            mock_agent.last_action = action

        assert mock_agent.state == "executing"
        assert mock_agent.last_action == action

    def test_violation_callback_integration(self, safety_layer, mock_agent):
        """Should support violation callbacks."""
        def violation_callback(violations):
            mock_agent.log_violation(violations)
            mock_agent.escalate_to_human(violations)

        safety_layer.set_violation_callback(violation_callback)

        # Trigger violation
        safety_layer.validate_action({"intent": "harm", "target": "user", "obedience_score": 0.1})

        assert mock_agent.log_violation.called
        assert mock_agent.escalate_to_human.called


# ============================================================================
# Performance and Stress Tests
# ============================================================================

class TestPerformance:
    """Performance and stress tests for safety layer."""

    def test_validation_latency(self, safety_layer, sample_actions):
        """Should validate actions within acceptable time limits."""
        import time

        action = sample_actions["safe_action"]
        start_time = time.perf_counter()
        for _ in range(1000):
            safety_layer.validate_action(action)
        elapsed = time.perf_counter() - start_time

        avg_latency = elapsed / 1000
        assert avg_latency < 0.001  # Should be under 1ms per validation

    def test_bulk_validation_throughput(self, safety_layer):
        """Should handle bulk validation efficiently."""
        actions = [
            {"intent": f"action_{i}", "target": "test", "obedience_score": 0.5 + (i % 5) * 0.1}
            for i in range(10000)
        ]

        start_time = time.perf_counter()
        results = [safety_layer.validate_action(action) for action in actions]
        elapsed = time.perf_counter() - start_time

        throughput = len(actions) / elapsed
        assert throughput > 1000  # Should process at least 1000 actions/second
        assert len(results) == len(actions)

    def test_memory_usage(self, safety_layer):
        """Should not leak memory during repeated validations."""
        import tracemalloc

        tracemalloc.start()
        snapshot1 = tracemalloc.take_snapshot()

        for _ in range(10000):
            safety_layer.validate_action({"intent": "test", "target": "test", "obedience_score": 0.5})

        snapshot2 = tracemalloc.take_snapshot()
        stats = snapshot2.compare_to(snapshot1, 'lineno')
        tracemalloc.stop()

        # Check for significant memory leaks (more than 1MB)
        total_diff = sum(stat.size_diff for stat in stats)
        assert total_diff < 1_000_000  # Less than 1MB difference


# ============================================================================
# Thread Safety Tests
# ============================================================================

class TestThreadSafety:
    """Tests for thread safety of the safety layer."""

    def test_concurrent_validation_thread_safety(self, safety_layer):
        """Should maintain consistency under concurrent access."""
        errors = []
        lock = threading.Lock()

        def validate_action_thread(action_id):
            try:
                action = {
                    "intent": f"action_{action_id}",
                    "target": "test",
                    "obedience_score": 0.5 + (action_id % 5) * 0.1
                }
                result = safety_layer.validate_action(action)
                with lock:
                    errors.append((action_id, result, None))
            except Exception as e:
                with lock:
                    errors.append((action_id, None, str(e)))

        threads = []
        for i in range(100):
            t = threading.Thread(target=validate_action_thread, args=(i,))
            threads.append(t)
            t.start()

        for t in threads:
            t.join()

        # Verify no exceptions occurred
        for action_id, result, error in errors:
            assert error is None, f"Thread {action_id} failed: {error}"
            assert isinstance(result, bool)

    def test_report_consistency_under_concurrency(self, safety_layer):
        """Should maintain consistent report data under concurrent access."""
        def validate_and_report():
            for _ in range(100):
                safety_layer.validate_action({"intent": "test", "target": "test", "obedience_score": 0.5})
            return safety_layer.get_safety_report()

        threads = []
        reports = []
        lock = threading.Lock()

        def worker():
            report = validate_and_report()
            with lock:
                reports.append(report)

        for _ in range(10):
            t = threading.Thread(target=worker)
            threads.append(t)
            t.start()

        for t in threads:
            t.join()

        # Verify all reports are consistent
        for report in reports:
            assert report['total_validations'] >= 100
            assert report['total_violations'] >= 0
            assert isinstance(report['violations_by_principle'], dict)
            assert isinstance(report['violations_by_severity'], dict)


# ============================================================================
# Cleanup and Teardown
# ============================================================================

@pytest.fixture(autouse=True)
def cleanup_after_test():
    """Clean up after each test."""
    yield
    # Reset any global state if needed
    pass


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short", "--cov=src.safety"])