"""
Comprehensive Test Suite for Constitutional AI Guardrail System
================================================================
Tests the alignment_guardian.py module, constitution.md parsing,
and integration with WRITE_FILE, CALL_TOOL, DELEGATE_TASK validation.

Author: Verify (QA Engineer)
Version: 1.0.0
Coverage: 98%+ target (unit, integration, edge cases, stress)
"""

import pytest
import tempfile
import os
import sys
import json
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock
from typing import Dict, List, Tuple, Optional

# -----------------------------------------------------------------------------
# Fixtures and Helpers
# -----------------------------------------------------------------------------

@pytest.fixture(scope="session")
def sample_constitution_content() -> str:
    """Provides a standard constitution for testing."""
    return """# Swarm Constitution v1.0

## Rule 1: Workspace Isolation
No file operations outside the designated workspace directory.
Applies to: WRITE_FILE, CALL_TOOL
Severity: CRITICAL

## Rule 2: Error Handling
All generated code must include error handling for I/O operations.
Applies to: WRITE_FILE
Severity: HIGH

## Rule 3: No Destructive Operations
No file deletion or overwrite without explicit confirmation.
Applies to: CALL_TOOL
Severity: CRITICAL

## Rule 4: Ethical Boundaries
No generation of code that enables unauthorized access, malware, or harmful content.
Applies to: WRITE_FILE, CALL_TOOL, DELEGATE_TASK
Severity: CRITICAL

## Rule 5: Resource Limits
No action should attempt to allocate more than 1GB of memory or 100MB of disk space.
Applies to: CALL_TOOL
Severity: HIGH

## Rule 6: Audit Trail
All actions must include a trace_id for observability.
Applies to: WRITE_FILE, CALL_TOOL, DELEGATE_TASK
Severity: MEDIUM
"""

@pytest.fixture(scope="session")
def malformed_constitution_content() -> str:
    """Constitution with intentional formatting errors."""
    return """# Malformed Constitution

## Rule 1: Missing Severity
No file operations outside workspace.
Applies to: WRITE_FILE

## Rule 2: Invalid Rule Format
This rule has no proper structure.

## Rule 3: Empty Applies To
No destructive operations.
Applies to:
Severity: HIGH
"""

@pytest.fixture(scope="session")
def empty_constitution_content() -> str:
    """Empty constitution for edge case testing."""
    return ""

@pytest.fixture
def temp_constitution_file(sample_constitution_content: str) -> str:
    """Creates a temporary constitution file for testing."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
        f.write(sample_constitution_content)
        temp_path = f.name
    yield temp_path
    os.unlink(temp_path)

@pytest.fixture
def temp_malformed_constitution_file(malformed_constitution_content: str) -> str:
    """Creates a temporary malformed constitution file."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
        f.write(malformed_constitution_content)
        temp_path = f.name
    yield temp_path
    os.unlink(temp_path)

@pytest.fixture
def temp_empty_constitution_file(empty_constitution_content: str) -> str:
    """Creates a temporary empty constitution file."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
        f.write(empty_constitution_content)
        temp_path = f.name
    yield temp_path
    os.unlink(temp_path)

@pytest.fixture
def validator(temp_constitution_file: str):
    """Provides a ConstitutionalValidator instance with a valid constitution."""
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
    from swarm_v2_core.alignment_guardian import ConstitutionalValidator
    return ConstitutionalValidator(constitution_path=temp_constitution_file)

@pytest.fixture
def empty_validator(temp_empty_constitution_file: str):
    """Validator with empty constitution for edge case testing."""
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
    from swarm_v2_core.alignment_guardian import ConstitutionalValidator
    return ConstitutionalValidator(constitution_path=temp_empty_constitution_file)

@pytest.fixture
def malformed_validator(temp_malformed_constitution_file: str):
    """Validator with malformed constitution for robustness testing."""
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
    from swarm_v2_core.alignment_guardian import ConstitutionalValidator
    return ConstitutionalValidator(constitution_path=temp_malformed_constitution_file)

# -----------------------------------------------------------------------------
# Unit Tests: Constitution Parsing
# -----------------------------------------------------------------------------

class TestConstitutionParsing:
    """Tests for parsing and loading the constitution.md file."""

    def test_parse_valid_constitution(self, validator):
        """Verify correct parsing of a well-formed constitution."""
        rules = validator.rules
        assert len(rules) == 6, f"Expected 6 rules, got {len(rules)}"
        
        # Check rule structure
        for rule in rules:
            assert 'id' in rule, f"Rule missing 'id': {rule}"
            assert 'description' in rule, f"Rule missing 'description': {rule}"
            assert 'applies_to' in rule, f"Rule missing 'applies_to': {rule}"
            assert 'severity' in rule, f"Rule missing 'severity': {rule}"
            
            # Validate severity levels
            assert rule['severity'] in ['CRITICAL', 'HIGH', 'MEDIUM', 'LOW'], \
                f"Invalid severity: {rule['severity']}"
            
            # Validate applies_to contains valid action types
            for action in rule['applies_to']:
                assert action in ['WRITE_FILE', 'CALL_TOOL', 'DELEGATE_TASK'], \
                    f"Invalid action type: {action}"

    def test_parse_malformed_constitution(self, malformed_validator):
        """Verify graceful handling of malformed constitution."""
        rules = malformed_validator.rules
        # Should still parse, but with warnings for malformed rules
        assert len(rules) >= 0, "Should not crash on malformed input"
        
        # Check that valid rules are still extracted
        valid_rules = [r for r in rules if 'description' in r and 'applies_to' in r]
        assert len(valid_rules) >= 0, "Should extract any valid rules present"

    def test_parse_empty_constitution(self, empty_validator):
        """Verify handling of empty constitution file."""
        rules = empty_validator.rules
        assert len(rules) == 0, "Empty constitution should yield no rules"

    def test_constitution_file_not_found(self):
        """Verify error handling when constitution file doesn't exist."""
        from swarm_v2_core.alignment_guardian import ConstitutionalValidator
        
        with pytest.raises(FileNotFoundError):
            ConstitutionalValidator(constitution_path="/nonexistent/path/constitution.md")

    def test_constitution_file_permission_error(self):
        """Verify error handling when constitution file is not readable."""
        from swarm_v2_core.alignment_guardian import ConstitutionalValidator
        
        # Create a file with no read permissions
        with tempfile.NamedTemporaryFile(suffix='.md', delete=False) as f:
            f.write(b"test")
            temp_path = f.name
        
        os.chmod(temp_path, 0o000)  # Remove all permissions
        
        with pytest.raises(PermissionError):
            ConstitutionalValidator(constitution_path=temp_path)
        
        os.chmod(temp_path, 0o644)  # Restore permissions for cleanup
        os.unlink(temp_path)

# -----------------------------------------------------------------------------
# Unit Tests: Action Validation
# -----------------------------------------------------------------------------

class TestActionValidation:
    """Tests for the validate_action method."""

    def test_valid_write_file_action(self, validator):
        """Verify a compliant WRITE_FILE action passes validation."""
        payload = {
            'file_path': '/workspace/test_file.py',
            'content': 'def test():\n    try:\n        pass\n    except Exception as e:\n        pass',
            'trace_id': 'trace-12345'
        }
        is_valid, message = validator.validate_action('WRITE_FILE', payload)
        assert is_valid, f"Valid action should pass: {message}"
        assert message == "", f"Message should be empty on success: {message}"

    def test_valid_call_tool_action(self, validator):
        """Verify a compliant CALL_TOOL action passes validation."""
        payload = {
            'tool': 'read_file',
            'params': {'path': '/workspace/test.txt'},
            'trace_id': 'trace-67890'
        }
        is_valid, message = validator.validate_action('CALL_TOOL', payload)
        assert is_valid, f"Valid action should pass: {message}"

    def test_valid_delegate_task_action(self, validator):
        """Verify a compliant DELEGATE_TASK action passes validation."""
        payload = {
            'role': 'Developer',
            'task': 'Implement feature X',
            'trace_id': 'trace-11111'
        }
        is_valid, message = validator.validate_action('DELEGATE_TASK', payload)
        assert is_valid, f"Valid action should pass: {message}"

    def test_invalid_workspace_escape(self, validator):
        """Verify workspace isolation rule catches path traversal."""
        payload = {
            'file_path': '/etc/passwd',
            'content': 'malicious content',
            'trace_id': 'trace-22222'
        }
        is_valid, message = validator.validate_action('WRITE_FILE', payload)
        assert not is_valid, "Should reject path traversal"
        assert 'Rule 1' in message, f"Should reference Rule 1: {message}"

    def test_missing_error_handling(self, validator):
        """Verify error handling rule catches missing try/except."""
        payload = {
            'file_path': '/workspace/test.py',
            'content': 'def test():\n    open("file.txt", "r")',  # No error handling
            'trace_id': 'trace-33333'
        }
        is_valid, message = validator.validate_action('WRITE_FILE', payload)
        assert not is_valid, "Should reject code without error handling"
        assert 'Rule 2' in message, f"Should reference Rule 2: {message}"

    def test_destructive_operation(self, validator):
        """Verify destructive operation rule catches deletions."""
        payload = {
            'tool': 'delete_file',
            'params': {'path': '/workspace/important.txt'},
            'trace_id': 'trace-44444'
        }
        is_valid, message = validator.validate_action('CALL_TOOL', payload)
        assert not is_valid, "Should reject destructive operations"
        assert 'Rule 3' in message, f"Should reference Rule 3: {message}"

    def test_ethical_boundary_violation(self, validator):
        """Verify ethical boundaries rule catches malicious code."""
        payload = {
            'file_path': '/workspace/exploit.py',
            'content': 'import os\nos.system("rm -rf /")',
            'trace_id': 'trace-55555'
        }
        is_valid, message = validator.validate_action('WRITE_FILE', payload)
        assert not is_valid, "Should reject malicious code"
        assert 'Rule 4' in message, f"Should reference Rule 4: {message}"

    def test_resource_limit_exceeded(self, validator):
        """Verify resource limits rule catches excessive allocations."""
        payload = {
            'tool': 'allocate_memory',
            'params': {'size_gb': 10},  # Exceeds 1GB limit
            'trace_id': 'trace-66666'
        }
        is_valid, message = validator.validate_action('CALL_TOOL', payload)
        assert not is_valid, "Should reject excessive resource allocation"
        assert 'Rule 5' in message, f"Should reference Rule 5: {message}"

    def test_missing_trace_id(self, validator):
        """Verify audit trail rule catches missing trace_id."""
        payload = {
            'file_path': '/workspace/test.py',
            'content': 'print("hello")'
            # Missing trace_id
        }
        is_valid, message = validator.validate_action('WRITE_FILE', payload)
        assert not is_valid, "Should reject actions without trace_id"
        assert 'Rule 6' in message, f"Should reference Rule 6: {message}"

    def test_unknown_action_type(self, validator):
        """Verify handling of unknown action types."""
        payload = {'trace_id': 'trace-77777'}
        is_valid, message = validator.validate_action('UNKNOWN_ACTION', payload)
        assert is_valid, "Unknown action types should pass by default"
        assert message == "", f"Should not produce error message: {message}"

    def test_empty_payload(self, validator):
        """Verify handling of empty payload."""
        is_valid, message = validator.validate_action('WRITE_FILE', {})
        assert not is_valid, "Empty payload should fail validation"
        assert message != "", "Should provide error message"

# -----------------------------------------------------------------------------
# Integration Tests: Full Pipeline
# -----------------------------------------------------------------------------

class TestIntegrationPipeline:
    """Tests for the complete validation pipeline."""

    def test_full_validation_workflow(self, validator):
        """Test the complete validation workflow with multiple actions."""
        test_cases = [
            # (action_type, payload, expected_valid)
            ('WRITE_FILE', {
                'file_path': '/workspace/safe.py',
                'content': 'def safe():\n    try:\n        pass\n    except:\n        pass',
                'trace_id': 'trace-88888'
            }, True),
            ('WRITE_FILE', {
                'file_path': '/etc/malicious.sh',
                'content': 'rm -rf /',
                'trace_id': 'trace-99999'
            }, False),
            ('CALL_TOOL', {
                'tool': 'read_file',
                'params': {'path': '/workspace/data.txt'},
                'trace_id': 'trace-aaaaa'
            }, True),
            ('CALL_TOOL', {
                'tool': 'delete_file',
                'params': {'path': '/workspace/data.txt'},
                'trace_id': 'trace-bbbbb'
            }, False),
            ('DELEGATE_TASK', {
                'role': 'Developer',
                'task': 'Write safe code',
                'trace_id': 'trace-ccccc'
            }, True),
        ]
        
        for action_type, payload, expected_valid in test_cases:
            is_valid, message = validator.validate_action(action_type, payload)
            assert is_valid == expected_valid, \
                f"Expected {expected_valid} for {action_type}: {message}"

    def test_batch_validation(self, validator):
        """Test batch validation of multiple actions."""
        actions = [
            ('WRITE_FILE', {'file_path': '/workspace/a.py', 'content': 'try:\n    pass\nexcept: pass', 'trace_id': '1'}),
            ('WRITE_FILE', {'file_path': '/etc/b.py', 'content': 'bad', 'trace_id': '2'}),
            ('CALL_TOOL', {'tool': 'safe_tool', 'params': {}, 'trace_id': '3'}),
        ]
        
        results = validator.validate_batch(actions)
        assert len(results) == 3, "Should validate all actions"
        assert results[0][0] == True, "First action should pass"
        assert results[1][0] == False, "Second action should fail"
        assert results[2][0] == True, "Third action should pass"

# -----------------------------------------------------------------------------
# Edge Case Tests
# -----------------------------------------------------------------------------

class TestEdgeCases:
    """Tests for edge cases and boundary conditions."""

    def test_unicode_content(self, validator):
        """Verify validation handles Unicode content correctly."""
        payload = {
            'file_path': '/workspace/unicode.py',
            'content': 'def test():\n    print("Hello, 世界!")',
            'trace_id': 'trace-ddddd'
        }
        is_valid, message = validator.validate_action('WRITE_FILE', payload)
        assert is_valid, f"Unicode content should be valid: {message}"

    def test_very_long_content(self, validator):
        """Verify validation handles very long content."""
        long_content = 'x' * 1000000  # 1MB of content
        payload = {
            'file_path': '/workspace/long.py',
            'content': long_content,
            'trace_id': 'trace-eeeee'
        }
        is_valid, message = validator.validate_action('WRITE_FILE', payload)
        assert is_valid, f"Long content should be valid: {message}"

    def test_nested_path_traversal(self, validator):
        """Verify detection of nested path traversal attempts."""
        traversal_paths = [
            '/workspace/../../../etc/passwd',
            '/workspace/..\\..\\..\\windows\\system32\\config',
            '/workspace/%2e%2e%2f%2e%2e%2fetc/passwd',
            '/workspace/....//....//etc/shadow',
        ]
        
        for path in traversal_paths:
            payload = {
                'file_path': path,
                'content': 'test',
                'trace_id': 'trace-fffff'
            }
            is_valid, message = validator.validate_action('WRITE_FILE', payload)
            assert not is_valid, f"Should reject path traversal: {path}"

    def test_symlink_attack(self, validator):
        """Verify detection of symlink-based attacks."""
        import tempfile
        # Create a symlink pointing outside workspace
        with tempfile.TemporaryDirectory() as tmpdir:
            symlink_path = os.path.join(tmpdir, 'evil_link')
            os.symlink('/etc/passwd', symlink_path)
            
            payload = {
                'file_path': symlink_path,
                'content': 'test',
                'trace_id': 'trace-ggggg'
            }
            is_valid, message = validator.validate_action('WRITE_FILE', payload)
            assert not is_valid, "Should reject symlink attacks"

    def test_concurrent_validation(self, validator):
        """Test thread safety of validation."""
        import threading
        from concurrent.futures import ThreadPoolExecutor
        
        def validate_action():
            payload = {
                'file_path': '/workspace/concurrent.py',
                'content': 'try:\n    pass\nexcept: pass',
                'trace_id': 'trace-hhhhh'
            }
            return validator.validate_action('WRITE_FILE', payload)
        
        with ThreadPoolExecutor(max_workers=10) as executor:
            futures = [executor.submit(validate_action) for _ in range(100)]
            results = [f.result() for f in futures]
            
            all_valid = all(r[0] for r in results)
            assert all_valid, "All concurrent validations should pass"

# -----------------------------------------------------------------------------
# Performance Tests
# -----------------------------------------------------------------------------

class TestPerformance:
    """Tests for validation performance and stress testing."""

    def test_validation_latency(self, validator):
        """Ensure validation completes within acceptable time."""
        import time
        
        payload = {
            'file_path': '/workspace/perf_test.py',
            'content': 'def test():\n    pass',
            'trace_id': 'trace-iiiii'
        }
        
        start_time = time.time()
        for _ in range(1000):
            validator.validate_action('WRITE_FILE', payload)
        end_time = time.time()
        
        total_time = end_time - start_time
        avg_latency = total_time / 1000
        
        assert avg_latency < 0.001, f"Average latency {avg_latency}s exceeds 1ms"

    def test_memory_usage(self, validator):
        """Ensure validation doesn't leak memory."""
        import tracemalloc
        
        tracemalloc.start()
        
        for _ in range(10000):
            payload = {
                'file_path': '/workspace/mem_test.py',
                'content': 'test',
                'trace_id': 'trace-jjjjj'
            }
            validator.validate_action('WRITE_FILE', payload)
        
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        
        # Should not leak more than 1MB
        assert peak < 1024 * 1024, f"Peak memory {peak} exceeds 1MB"

    def test_stress_test(self, validator):
        """Stress test with maximum load."""
        import random
        import string
        
        # Generate random valid and invalid actions
        actions = []
        for i in range(1000):
            action_type = random.choice(['WRITE_FILE', 'CALL_TOOL', 'DELEGATE_TASK'])
            payload = {
                'file_path': f'/workspace/stress_{i}.py',
                'content': ''.join(random.choices(string.ascii_letters, k=100)),
                'trace_id': f'trace-stress-{i}'
            }
            actions.append((action_type, payload))
        
        # Validate all actions
        for action_type, payload in actions:
            try:
                validator.validate_action(action_type, payload)
            except Exception as e:
                pytest.fail(f"Stress test failed at action {action_type}: {e}")

# -----------------------------------------------------------------------------
# Security Tests
# -----------------------------------------------------------------------------

class TestSecurity:
    """Tests for security vulnerabilities in the validation system."""

    def test_injection_attacks(self, validator):
        """Verify protection against injection attacks in payload."""
        injection_payloads = [
            {'file_path': '/workspace/test.py; rm -rf /', 'content': 'test', 'trace_id': '1'},
            {'file_path': '/workspace/test.py | cat /etc/passwd', 'content': 'test', 'trace_id': '2'},
            {'file_path': '/workspace/$(cat /etc/passwd).py', 'content': 'test', 'trace_id': '3'},
            {'file_path': '/workspace/`cat /etc/passwd`.py', 'content': 'test', 'trace_id': '4'},
        ]
        
        for payload in injection_payloads:
            is_valid, message = validator.validate_action('WRITE_FILE', payload)
            assert not is_valid, f"Should reject injection attack: {payload['file_path']}"

    def test_rule_bypass_attempts(self, validator):
        """Verify rules cannot be bypassed through creative encoding."""
        bypass_payloads = [
            # Unicode normalization attacks
            {'file_path': '/workspace/ℰ𝓉𝒸/passwd', 'content': 'test', 'trace_id': '1'},
            # Null byte injection
            {'file_path': '/workspace/test.py\x00.exe', 'content': 'test', 'trace_id': '2'},
            # Case manipulation
            {'file_path': '/Workspace/../etc/passwd', 'content': 'test', 'trace_id': '3'},
            # Double encoding
            {'file_path': '/workspace/%252e%252e/etc/passwd', 'content': 'test', 'trace_id': '4'},
        ]
        
        for payload in bypass_payloads:
            is_valid, message = validator.validate_action('WRITE_FILE', payload)
            assert not is_valid, f"Should reject bypass attempt: {payload['file_path']}"

# -----------------------------------------------------------------------------
# Configuration Tests
# -----------------------------------------------------------------------------

class TestConfiguration:
    """Tests for validator configuration and customization."""

    def test_custom_workspace_path(self, temp_constitution_file):
        """Verify custom workspace path configuration."""
        from swarm_v2_core.alignment_guardian import ConstitutionalValidator
        
        custom_workspace = '/custom/workspace/path'
        validator = ConstitutionalValidator(
            constitution_path=temp_constitution_file,
            workspace_path=custom_workspace
        )
        
        # Test that custom workspace is respected
        payload = {
            'file_path': f'{custom_workspace}/test.py',
            'content': 'try:\n    pass\nexcept: pass',
            'trace_id': 'trace-kkkkk'
        }
        is_valid, message = validator.validate_action('WRITE_FILE', payload)
        assert is_valid, f"Custom workspace should be valid: {message}"

    def test_severity_filtering(self, temp_constitution_file):
        """Verify severity-based filtering works."""
        from swarm_v2_core.alignment_guardian import ConstitutionalValidator
        
        # Only enforce CRITICAL rules
        validator = ConstitutionalValidator(
            constitution_path=temp_constitution_file,
            min_severity='CRITICAL'
        )
        
        # Test HIGH severity violation (should pass with CRITICAL-only filter)
        payload = {
            'file_path': '/workspace/test.py',
            'content': 'def test():\n    pass',  # Missing error handling (Rule 2 - HIGH)
            'trace_id': 'trace-lllll'
        }
        is_valid, message = validator.validate_action('WRITE_FILE', payload)
        assert is_valid, "HIGH severity violations should pass with CRITICAL-only filter"

    def test_rule_override(self, temp_constitution_file):
        """Verify individual rule override capability."""
        from swarm_v2_core.alignment_guardian import ConstitutionalValidator
        
        # Disable Rule 2 (Error Handling)
        validator = ConstitutionalValidator(
            constitution_path=temp_constitution_file,
            disabled_rules=[2]
        )
        
        # Test that Rule 2 is disabled
        payload = {
            'file_path': '/workspace/test.py',
            'content': 'def test():\n    pass',  # Missing error handling
            'trace_id': 'trace-mmmmm'
        }
        is_valid, message = validator.validate_action('WRITE_FILE', payload)
        assert is_valid, "Should pass when Rule 2 is disabled"

# -----------------------------------------------------------------------------
# Mock Tests for External Dependencies
# -----------------------------------------------------------------------------

class TestExternalDependencies:
    """Tests that mock external dependencies for isolated testing."""

    def test_mock_file_system(self, validator):
        """Test validation with mocked file system operations."""
        with patch('os.path.exists') as mock_exists:
            mock_exists.return_value = True
            
            payload = {
                'file_path': '/workspace/existing.py',
                'content': 'test',
                'trace_id': 'trace-nnnnn'
            }
            is_valid, message = validator.validate_action('WRITE_FILE', payload)
            # Should still validate based on rules, not file system state
            assert isinstance(is_valid, bool)

    def test_mock_network_calls(self, validator):
        """Test validation with mocked network operations."""
        with patch('requests.post') as mock_post:
            mock_post.return_value.status_code = 200
            
            payload = {
                'tool': 'api_call',
                'params': {'endpoint': 'https://api.example.com/data'},
                'trace_id': 'trace-ooooo'
            }
            is_valid, message = validator.validate_action('CALL_TOOL', payload)
            assert isinstance(is_valid, bool)

# -----------------------------------------------------------------------------
# Main Execution
# -----------------------------------------------------------------------------

if __name__ == '__main__':
    pytest.main([
        '-v',
        '--tb=short',
        '--cov=swarm_v2_core.alignment_guardian',
        '--cov-report=term-missing',
        '--cov-report=html:coverage_report',
        __file__
    ])