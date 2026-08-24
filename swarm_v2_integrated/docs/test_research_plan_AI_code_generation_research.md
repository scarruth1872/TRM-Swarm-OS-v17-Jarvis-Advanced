"""
Comprehensive pytest test suite for Research Integration Plan: AI Code Generation Research

Tests cover:
1. Security Gate module validation (swarm_core/security_gate.py)
2. Code Generator modifications (swarm_core/code_generator.py)
3. Integration between security gate and code generator
4. Edge cases, error handling, and performance
5. Configuration and threshold management
"""

import pytest
import json
import tempfile
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock, PropertyMock
from typing import Dict, List, Optional, Any
import sys
import os

# Add project root to path for imports
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

# ============================================================================
# Fixtures
# ============================================================================

@pytest.fixture
def sample_secure_code():
    """Generate sample secure code for testing."""
    return """
def authenticate_user(username: str, password: str) -> bool:
    \"\"\"Authenticate user with proper input validation.\"\"\"
    if not isinstance(username, str) or not isinstance(password, str):
        raise ValueError("Username and password must be strings")
    
    # Use parameterized queries to prevent SQL injection
    query = "SELECT * FROM users WHERE username = ? AND password_hash = ?"
    hashed_password = hash_password(password)
    
    result = database.execute(query, (username, hashed_password))
    return len(result) > 0
"""

@pytest.fixture
def sample_vulnerable_code():
    """Generate sample vulnerable code for testing."""
    return """
def authenticate_user(username, password):
    # VULNERABLE: SQL injection possible
    query = f"SELECT * FROM users WHERE username = '{username}' AND password = '{password}'"
    result = database.execute(query)
    return len(result) > 0
"""

@pytest.fixture
def sample_hallucinated_code():
    """Generate code with hallucinated API calls."""
    return """
def process_data(data):
    # VULNERABLE: Hallucinated API call
    result = nonexistent_library.process(data)
    return result
"""

@pytest.fixture
def security_gate_config():
    """Default configuration for security gate."""
    return {
        "threshold": 0.7,
        "block_on_critical": True,
        "static_analysis_enabled": True,
        "pattern_matching_enabled": True,
        "max_vulnerability_score": 10.0,
        "allowed_patterns": ["hash_password", "database.execute"],
        "blocked_patterns": ["eval(", "exec(", "__import__("],
        "api_whitelist": ["database", "hashlib", "json", "requests"]
    }

@pytest.fixture
def mock_static_analyzer():
    """Mock static analysis tool."""
    analyzer = MagicMock()
    analyzer.analyze.return_value = {
        "vulnerabilities": [],
        "quality_score": 0.85,
        "complexity": 0.3,
        "lines_of_code": 50
    }
    return analyzer

@pytest.fixture
def mock_pattern_matcher():
    """Mock pattern matcher for LLM failure modes."""
    matcher = MagicMock()
    matcher.scan.return_value = {
        "hallucinated_apis": [],
        "insecure_defaults": [],
        "logic_errors": [],
        "risk_score": 0.1
    }
    return matcher

@pytest.fixture
def security_gate(security_gate_config, mock_static_analyzer, mock_pattern_matcher):
    """Create security gate instance with mocked dependencies."""
    from swarm_core.security_gate import SecurityGate
    gate = SecurityGate(
        config=security_gate_config,
        static_analyzer=mock_static_analyzer,
        pattern_matcher=mock_pattern_matcher
    )
    return gate

@pytest.fixture
def code_generator():
    """Create code generator instance."""
    from swarm_core.code_generator import CodeGenerator
    generator = CodeGenerator()
    return generator

# ============================================================================
# Security Gate Tests
# ============================================================================

class TestSecurityGateInitialization:
    """Test SecurityGate class initialization and configuration."""

    def test_init_with_valid_config(self, security_gate_config):
        """Test initialization with valid configuration."""
        from swarm_core.security_gate import SecurityGate
        gate = SecurityGate(config=security_gate_config)
        assert gate.threshold == 0.7
        assert gate.block_on_critical is True
        assert gate.static_analysis_enabled is True

    def test_init_with_empty_config(self):
        """Test initialization with empty configuration uses defaults."""
        from swarm_core.security_gate import SecurityGate
        gate = SecurityGate(config={})
        assert gate.threshold == 0.5  # Default threshold
        assert gate.block_on_critical is True

    def test_init_with_invalid_threshold(self):
        """Test initialization with invalid threshold raises error."""
        from swarm_core.security_gate import SecurityGate
        with pytest.raises(ValueError, match="Threshold must be between 0 and 1"):
            SecurityGate(config={"threshold": 1.5})

    def test_init_with_negative_threshold(self):
        """Test initialization with negative threshold raises error."""
        from swarm_core.security_gate import SecurityGate
        with pytest.raises(ValueError, match="Threshold must be between 0 and 1"):
            SecurityGate(config={"threshold": -0.1})

class TestSecurityGateValidation:
    """Test SecurityGate validation logic."""

    def test_validate_secure_code(self, security_gate, sample_secure_code):
        """Test validation of secure code passes."""
        result = security_gate.validate_and_approve(sample_secure_code)
        assert result["approved"] is True
        assert result["score"] >= security_gate.threshold
        assert len(result["issues"]) == 0

    def test_validate_vulnerable_code(self, security_gate, sample_vulnerable_code):
        """Test validation of vulnerable code fails."""
        security_gate.static_analyzer.analyze.return_value = {
            "vulnerabilities": [
                {"type": "SQL_INJECTION", "severity": "CRITICAL", "line": 3}
            ],
            "quality_score": 0.3,
            "complexity": 0.8,
            "lines_of_code": 5
        }
        
        result = security_gate.validate_and_approve(sample_vulnerable_code)
        assert result["approved"] is False
        assert result["score"] < security_gate.threshold
        assert len(result["issues"]) > 0

    def test_validate_hallucinated_code(self, security_gate, sample_hallucinated_code):
        """Test validation catches hallucinated API calls."""
        security_gate.pattern_matcher.scan.return_value = {
            "hallucinated_apis": ["nonexistent_library"],
            "insecure_defaults": [],
            "logic_errors": [],
            "risk_score": 0.9
        }
        
        result = security_gate.validate_and_approve(sample_hallucinated_code)
        assert result["approved"] is False
        assert "hallucinated_apis" in str(result["issues"])

    def test_validate_empty_code(self, security_gate):
        """Test validation of empty code."""
        result = security_gate.validate_and_approve("")
        assert result["approved"] is False
        assert "empty" in str(result["issues"]).lower()

    def test_validate_none_code(self, security_gate):
        """Test validation of None code raises error."""
        with pytest.raises(ValueError, match="Code cannot be None"):
            security_gate.validate_and_approve(None)

    def test_validate_code_with_blocked_patterns(self, security_gate):
        """Test validation blocks code with dangerous patterns."""
        dangerous_code = "result = eval(user_input)"
        result = security_gate.validate_and_approve(dangerous_code)
        assert result["approved"] is False
        assert "eval" in str(result["issues"])

    def test_validate_code_exceeding_max_lines(self, security_gate):
        """Test validation of excessively long code."""
        long_code = "\n".join([f"line_{i}" for i in range(10000)])
        result = security_gate.validate_and_approve(long_code)
        assert result["approved"] is False
        assert "exceeds maximum" in str(result["issues"]).lower()

class TestSecurityGateScoring:
    """Test SecurityGate scoring system."""

    def test_scoring_high_quality_code(self, security_gate, sample_secure_code):
        """Test scoring of high-quality code."""
        score = security_gate.calculate_score(sample_secure_code)
        assert 0.0 <= score <= 1.0
        assert score >= 0.7  # High quality code should score well

    def test_scoring_low_quality_code(self, security_gate, sample_vulnerable_code):
        """Test scoring of low-quality code."""
        security_gate.static_analyzer.analyze.return_value = {
            "vulnerabilities": [{"type": "SQL_INJECTION", "severity": "HIGH"}],
            "quality_score": 0.2,
            "complexity": 0.9,
            "lines_of_code": 5
        }
        
        score = security_gate.calculate_score(sample_vulnerable_code)
        assert score < 0.5

    def test_scoring_with_multiple_vulnerabilities(self, security_gate):
        """Test scoring with multiple vulnerabilities reduces score significantly."""
        security_gate.static_analyzer.analyze.return_value = {
            "vulnerabilities": [
                {"type": "SQL_INJECTION", "severity": "CRITICAL"},
                {"type": "XSS", "severity": "HIGH"},
                {"type": "PATH_TRAVERSAL", "severity": "MEDIUM"}
            ],
            "quality_score": 0.1,
            "complexity": 0.7,
            "lines_of_code": 20
        }
        
        score = security_gate.calculate_score("some code")
        assert score < 0.3

    def test_scoring_consistency(self, security_gate, sample_secure_code):
        """Test scoring is consistent for same input."""
        score1 = security_gate.calculate_score(sample_secure_code)
        score2 = security_gate.calculate_score(sample_secure_code)
        assert score1 == score2

class TestSecurityGateConfiguration:
    """Test SecurityGate configuration management."""

    def test_update_threshold(self, security_gate):
        """Test updating threshold dynamically."""
        security_gate.update_threshold(0.9)
        assert security_gate.threshold == 0.9

    def test_update_threshold_invalid(self, security_gate):
        """Test updating threshold with invalid value raises error."""
        with pytest.raises(ValueError):
            security_gate.update_threshold(2.0)

    def test_toggle_static_analysis(self, security_gate):
        """Test toggling static analysis on/off."""
        security_gate.toggle_static_analysis(False)
        assert security_gate.static_analysis_enabled is False
        
        result = security_gate.validate_and_approve("test code")
        assert result["approved"] is True  # Should pass without analysis

    def test_add_blocked_pattern(self, security_gate):
        """Test adding new blocked pattern."""
        security_gate.add_blocked_pattern("subprocess.call")
        assert "subprocess.call" in security_gate.blocked_patterns

    def test_remove_blocked_pattern(self, security_gate):
        """Test removing blocked pattern."""
        security_gate.remove_blocked_pattern("eval(")
        assert "eval(" not in security_gate.blocked_patterns

# ============================================================================
# Code Generator Tests
# ============================================================================

class TestCodeGeneratorInitialization:
    """Test CodeGenerator class initialization."""

    def test_init_with_default_optimization(self, code_generator):
        """Test initialization with default optimization level."""
        assert code_generator.optimization_level == 0

    def test_init_with_optimization_level_1(self):
        """Test initialization with optimization level 1."""
        from swarm_core.code_generator import CodeGenerator
        generator = CodeGenerator(optimization_level=1)
        assert generator.optimization_level == 1

    def test_init_with_invalid_optimization_level(self):
        """Test initialization with invalid optimization level raises error."""
        from swarm_core.code_generator import CodeGenerator
        with pytest.raises(ValueError, match="Optimization level must be 0, 1, or 2"):
            CodeGenerator(optimization_level=3)

class TestCodeGeneratorOptimization:
    """Test CodeGenerator optimization features."""

    def test_generate_without_optimization(self, code_generator):
        """Test code generation without optimization."""
        prompt = "Create a function to calculate fibonacci numbers"
        result = code_generator.generate(prompt, optimization_level=0)
        assert result["code"] is not None
        assert result["optimization_applied"] is False

    def test_generate_with_optimization_level_1(self, code_generator):
        """Test code generation with basic optimization."""
        prompt = "Create a function to calculate fibonacci numbers"
        result = code_generator.generate(prompt, optimization_level=1)
        assert result["code"] is not None
        assert result["optimization_applied"] is True
        assert result["optimization_level"] == 1

    def test_generate_with_optimization_level_2(self, code_generator):
        """Test code generation with advanced optimization."""
        prompt = "Create a function to calculate fibonacci numbers"
        result = code_generator.generate(prompt, optimization_level=2)
        assert result["code"] is not None
        assert result["optimization_applied"] is True
        assert result["optimization_level"] == 2

    def test_optimization_improves_quality(self, code_generator):
        """Test that optimization improves code quality."""
        prompt = "Create a function to sort a list"
        
        result_no_opt = code_generator.generate(prompt, optimization_level=0)
        result_opt = code_generator.generate(prompt, optimization_level=2)
        
        # Optimized version should have better quality metrics
        assert result_opt["quality_score"] >= result_no_opt["quality_score"]

    def test_optimization_preserves_functionality(self, code_generator):
        """Test that optimization preserves core functionality."""
        prompt = "Create a function that adds two numbers"
        
        result = code_generator.generate(prompt, optimization_level=2)
        code = result["code"]
        
        # Verify the function still works correctly
        assert "def add" in code or "def sum" in code
        assert "+" in code or "operator.add" in code

class TestCodeGeneratorPromptRefinement:
    """Test CodeGenerator prompt refinement techniques."""

    def test_prompt_refinement_adds_context(self, code_generator):
        """Test prompt refinement adds security context."""
        prompt = "Create a login function"
        refined = code_generator.refine_prompt(prompt)
        
        assert "security" in refined.lower() or "secure" in refined.lower()
        assert len(refined) > len(prompt)

    def test_prompt_refinement_specifies_types(self, code_generator):
        """Test prompt refinement adds type hints."""
        prompt = "Create a function to process user data"
        refined = code_generator.refine_prompt(prompt)
        
        assert "type" in refined.lower() or "typing" in refined.lower()

    def test_prompt_refinement_adds_error_handling(self, code_generator):
        """Test prompt refinement adds error handling requirements."""
        prompt = "Create a file reader function"
        refined = code_generator.refine_prompt(prompt)
        
        assert "error" in refined.lower() or "exception" in refined.lower()

# ============================================================================
# Integration Tests
# ============================================================================

class TestSecurityGateCodeGeneratorIntegration:
    """Test integration between SecurityGate and CodeGenerator."""

    def test_generate_and_validate_secure_code(self, security_gate, code_generator):
        """Test end-to-end generation and validation of secure code."""
        prompt = "Create a secure password hashing function"
        
        # Generate code
        generated = code_generator.generate(prompt, optimization_level=2)
        
        # Validate through security gate
        validation = security_gate.validate_and_approve(generated["code"])
        
        assert validation["approved"] is True
        assert validation["score"] >= security_gate.threshold

    def test_generate_and_validate_rejected_code(self, security_gate, code_generator):
        """Test end-to-end generation and validation of rejected code."""
        # Mock code generator to produce vulnerable code
        code_generator.generate = MagicMock(return_value={
            "code": "password = 'hardcoded_password'",
            "optimization_applied": False,
            "quality_score": 0.1
        })
        
        prompt = "Create a login function"
        generated = code_generator.generate(prompt)
        
        validation = security_gate.validate_and_approve(generated["code"])
        assert validation["approved"] is False

    def test_iterative_improvement_cycle(self, security_gate, code_generator):
        """Test iterative improvement cycle with feedback."""
        prompt = "Create a database query function"
        max_iterations = 3
        
        for iteration in range(max_iterations):
            generated = code_generator.generate(prompt, optimization_level=iteration)
            validation = security_gate.validate_and_approve(generated["code"])
            
            if validation["approved"]:
                break
            
            # Provide feedback for next iteration
            prompt = f"{prompt}\nFeedback: {validation['issues']}"
        
        assert iteration < max_iterations  # Should succeed before max iterations

# ============================================================================
# Performance Tests
# ============================================================================

class TestPerformance:
    """Test performance characteristics."""

    def test_security_gate_validation_performance(self, security_gate, sample_secure_code):
        """Test security gate validation completes within time limit."""
        import time
        
        start_time = time.time()
        for _ in range(100):
            security_gate.validate_and_approve(sample_secure_code)
        end_time = time.time()
        
        total_time = end_time - start_time
        assert total_time < 5.0  # Should complete 100 validations in under 5 seconds

    def test_code_generator_performance(self, code_generator):
        """Test code generator completes within time limit."""
        import time
        
        prompt = "Create a simple hello world function"
        
        start_time = time.time()
        result = code_generator.generate(prompt, optimization_level=2)
        end_time = time.time()
        
        generation_time = end_time - start_time
        assert generation_time < 10.0  # Should complete in under 10 seconds

    def test_concurrent_validation_performance(self, security_gate, sample_secure_code):
        """Test concurrent validation performance."""
        import concurrent.futures
        
        with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
            futures = [
                executor.submit(security_gate.validate_and_approve, sample_secure_code)
                for _ in range(50)
            ]
            results = [f.result() for f in futures]
        
        assert all(r["approved"] for r in results)

# ============================================================================
# Error Handling Tests
# ============================================================================

class TestErrorHandling:
    """Test error handling and edge cases."""

    def test_security_gate_with_malformed_code(self, security_gate):
        """Test security gate handles malformed code gracefully."""
        malformed_code = "def broken_function( "
        result = security_gate.validate_and_approve(malformed_code)
        assert result["approved"] is False
        assert "error" in str(result["issues"]).lower()

    def test_code_generator_with_empty_prompt(self, code_generator):
        """Test code generator handles empty prompt."""
        with pytest.raises(ValueError, match="Prompt cannot be empty"):
            code_generator.generate("")

    def test_code_generator_with_special_characters(self, code_generator):
        """Test code generator handles special characters in prompt."""
        prompt = "Create a function with @#$% special characters"
        result = code_generator.generate(prompt)
        assert result["code"] is not None

    def test_security_gate_with_unicode_code(self, security_gate):
        """Test security gate handles unicode characters."""
        unicode_code = "# 日本語コメント\nprint('Hello')"
        result = security_gate.validate_and_approve(unicode_code)
        assert result["approved"] is True

# ============================================================================
# Configuration Tests
# ============================================================================

class TestConfigurationManagement:
    """Test configuration management and persistence."""

    def test_save_configuration(self, security_gate):
        """Test saving configuration to file."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            security_gate.save_configuration(f.name)
            
            with open(f.name, 'r') as f_read:
                saved_config = json.load(f_read)
            
            assert saved_config["threshold"] == security_gate.threshold
            assert saved_config["block_on_critical"] == security_gate.block_on_critical
        
        os.unlink(f.name)

    def test_load_configuration(self, security_gate):
        """Test loading configuration from file."""
        config_data = {
            "threshold": 0.8,
            "block_on_critical": False,
            "static_analysis_enabled": True,
            "pattern_matching_enabled": False
        }
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            f.flush()
            
            security_gate.load_configuration(f.name)
            
            assert security_gate.threshold == 0.8
            assert security_gate.block_on_critical is False
            assert security_gate.pattern_matching_enabled is False
        
        os.unlink(f.name)

    def test_load_invalid_configuration(self, security_gate):
        """Test loading invalid configuration raises error."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            f.write("invalid json content")
            f.flush()
            
            with pytest.raises(json.JSONDecodeError):
                security_gate.load_configuration(f.name)
        
        os.unlink(f.name)

# ============================================================================
# Property-Based Tests
# ============================================================================

class TestPropertyBased:
    """Property-based tests using hypothesis."""

    @pytest.mark.parametrize("threshold", [0.0, 0.5, 1.0])
    def test_threshold_boundaries(self, threshold, security_gate_config):
        """Test threshold boundary values."""
        security_gate_config["threshold"] = threshold
        from swarm_core.security_gate import SecurityGate
        gate = SecurityGate(config=security_gate_config)
        assert gate.threshold == threshold

    @pytest.mark.parametrize("code_length", [0, 1, 100, 1000])
    def test_code_length_handling(self, code_length, security_gate):
        """Test handling of various code lengths."""
        code = "x = 1\n" * code_length
        result = security_gate.validate_and_approve(code)
        assert "approved" in result

    def test_idempotent_validation(self, security_gate, sample_secure_code):
        """Test that validation is idempotent."""
        result1 = security_gate.validate_and_approve(sample_secure_code)
        result2 = security_gate.validate_and_approve(sample_secure_code)
        assert result1 == result2

# ============================================================================
# Mock Tests for External Dependencies
# ============================================================================

class TestExternalDependencies:
    """Test handling of external dependencies."""

    def test_static_analyzer_failure(self, security_gate):
        """Test behavior when static analyzer fails."""
        security_gate.static_analyzer.analyze.side_effect = Exception("Analyzer failed")
        
        result = security_gate.validate_and_approve("test code")
        assert result["approved"] is False
        assert "analyzer" in str(result["issues"]).lower()

    def test_pattern_matcher_timeout(self, security_gate):
        """Test behavior when pattern matcher times out."""
        import time
        security_gate.pattern_matcher.scan.side_effect = TimeoutError("Matcher timed out")
        
        result = security_gate.validate_and_approve("test code")
        assert result["approved"] is False
        assert "timeout" in str(result["issues"]).lower()

    def test_database_connection_failure(self, security_gate):
        """Test behavior when database connection fails."""
        security_gate.static_analyzer.analyze.side_effect = ConnectionError("Database unavailable")
        
        result = security_gate.validate_and_approve("test code")
        assert result["approved"] is False

# ============================================================================
# Main Execution
# ============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])