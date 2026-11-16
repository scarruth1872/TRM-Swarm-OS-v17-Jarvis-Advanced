# Test Suite: Machine Learning Optimization Research Plan

## Overview
This test suite validates the core components and hypotheses outlined in the `research_plan_machine_learning_optimization_.md` document. It covers unit tests for optimization algorithms, integration tests for training loops, and benchmark tests for convergence properties.

## Dependencies
- Python 3.10+
- `pytest==7.4.0`
- `numpy==1.24.0`
- `torch==2.0.0` (or `jax==0.4.0` for alternative backend)
- `hypothesis==6.70.0` (for property-based testing)

## Test Structure

```python
"""
test_research_plan_machine_learning_optimization_.md

Production-grade test suite for ML optimization research plan.
Validates optimizer correctness, convergence properties, and edge-case robustness.
"""

import pytest
import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim
from typing import Callable, List, Tuple, Optional
from hypothesis import given, strategies as st
from hypothesis.extra.numpy import arrays

# =============================================================================
# Mock Components (Isolate optimization logic from full model)
# =============================================================================

class SimpleQuadraticModel(nn.Module):
    """A simple quadratic model for testing optimizer convergence."""
    def __init__(self, a: float = 1.0, b: float = 0.0):
        super().__init__()
        self.a = nn.Parameter(torch.tensor(a, dtype=torch.float32))
        self.b = nn.Parameter(torch.tensor(b, dtype=torch.float32))

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.a * x**2 + self.b * x

def quadratic_loss(y_pred: torch.Tensor, y_true: torch.Tensor) -> torch.Tensor:
    """Mean squared error loss."""
    return torch.mean((y_pred - y_true)**2)

# =============================================================================
# Unit Tests: Optimizer Initialization and Configuration
# =============================================================================

class TestOptimizerInitialization:
    """Verify optimizers are correctly instantiated with given hyperparameters."""

    def test_sgd_initialization(self):
        """Test SGD optimizer with momentum and weight decay."""
        model = SimpleQuadraticModel()
        optimizer = optim.SGD(model.parameters(), lr=0.01, momentum=0.9, weight_decay=1e-4)
        assert optimizer.param_groups[0]['lr'] == 0.01
        assert optimizer.param_groups[0]['momentum'] == 0.9
        assert optimizer.param_groups[0]['weight_decay'] == 1e-4

    def test_adam_initialization(self):
        """Test Adam optimizer with default and custom betas."""
        model = SimpleQuadraticModel()
        optimizer = optim.Adam(model.parameters(), lr=0.001, betas=(0.9, 0.999))
        assert optimizer.param_groups[0]['lr'] == 0.001
        assert optimizer.param_groups[0]['betas'] == (0.9, 0.999)

    def test_invalid_learning_rate_raises_error(self):
        """Negative or zero learning rates should raise ValueError."""
        model = SimpleQuadraticModel()
        with pytest.raises(ValueError):
            optim.SGD(model.parameters(), lr=-0.01)

# =============================================================================
# Unit Tests: Optimizer Step Correctness
# =============================================================================

class TestOptimizerStep:
    """Verify that optimizer.step() correctly updates model parameters."""

    @pytest.mark.parametrize("optimizer_class,lr", [
        (optim.SGD, 0.01),
        (optim.Adam, 0.001),
        (optim.AdamW, 0.001),
    ])
    def test_parameter_update_direction(self, optimizer_class: Callable, lr: float):
        """After one step, parameters should move in the direction of the negative gradient."""
        model = SimpleQuadraticModel(a=2.0, b=1.0)
        optimizer = optimizer_class(model.parameters(), lr=lr)

        x = torch.tensor([1.0, 2.0, 3.0])
        y_true = torch.tensor([0.0, 0.0, 0.0])

        # Forward pass
        y_pred = model(x)
        loss = quadratic_loss(y_pred, y_true)

        # Backward pass
        loss.backward()

        # Record parameters before step
        a_before = model.a.clone().detach()
        b_before = model.b.clone().detach()

        # Step
        optimizer.step()

        # Verify parameters changed
        assert not torch.equal(model.a, a_before), "Parameter 'a' did not update"
        assert not torch.equal(model.b, b_before), "Parameter 'b' did not update"

        # Verify direction: loss should decrease (or at least not increase for small lr)
        with torch.no_grad():
            y_pred_new = model(x)
            loss_new = quadratic_loss(y_pred_new, y_true)
        assert loss_new <= loss + 1e-6, f"Loss increased from {loss.item()} to {loss_new.item()}"

    def test_zero_grad_clears_gradients(self):
        """After zero_grad(), all parameter gradients should be None or zero."""
        model = SimpleQuadraticModel()
        optimizer = optim.SGD(model.parameters(), lr=0.01)

        x = torch.tensor([1.0])
        y_true = torch.tensor([0.0])
        y_pred = model(x)
        loss = quadratic_loss(y_pred, y_true)
        loss.backward()

        optimizer.zero_grad()

        for param in model.parameters():
            assert param.grad is None or torch.all(param.grad == 0), "Gradients not zeroed"

# =============================================================================
# Integration Tests: Full Training Loop Convergence
# =============================================================================

class TestTrainingLoopConvergence:
    """Verify that a full training loop converges to the expected minimum."""

    @pytest.mark.parametrize("optimizer_class,lr,expected_loss", [
        (optim.SGD, 0.1, 1e-3),
        (optim.Adam, 0.01, 1e-4),
    ])
    def test_convergence_to_global_minimum(
        self, optimizer_class: Callable, lr: float, expected_loss: float
    ):
        """Train on a simple quadratic and verify loss drops below threshold."""
        model = SimpleQuadraticModel(a=3.0, b=-2.0)
        optimizer = optimizer_class(model.parameters(), lr=lr)

        x = torch.linspace(-5, 5, 100)
        y_true = torch.zeros_like(x)  # Target is zero everywhere

        for epoch in range(500):
            optimizer.zero_grad()
            y_pred = model(x)
            loss = quadratic_loss(y_pred, y_true)
            loss.backward()
            optimizer.step()

        final_loss = loss.item()
        assert final_loss < expected_loss, (
            f"Optimizer {optimizer_class.__name__} failed to converge. "
            f"Final loss: {final_loss}, expected < {expected_loss}"
        )

    def test_learning_rate_scheduler_integration(self):
        """Test that StepLR scheduler reduces learning rate over epochs."""
        model = SimpleQuadraticModel()
        optimizer = optim.SGD(model.parameters(), lr=0.1)
        scheduler = optim.lr_scheduler.StepLR(optimizer, step_size=10, gamma=0.5)

        initial_lr = optimizer.param_groups[0]['lr']
        for epoch in range(15):
            scheduler.step()

        final_lr = optimizer.param_groups[0]['lr']
        assert final_lr < initial_lr, "Learning rate did not decrease"
        assert final_lr == pytest.approx(initial_lr * (0.5 ** 1), rel=1e-6), (
            f"Expected LR {initial_lr * 0.5}, got {final_lr}"
        )

# =============================================================================
# Edge Case Tests: Numerical Stability and Robustness
# =============================================================================

class TestNumericalStability:
    """Verify optimizers handle edge cases without NaN or Inf."""

    def test_nan_gradient_handling(self):
        """Optimizer should not crash when gradients contain NaN."""
        model = SimpleQuadraticModel()
        optimizer = optim.SGD(model.parameters(), lr=0.01)

        # Manually set NaN gradient
        model.a.grad = torch.tensor(float('nan'))
        model.b.grad = torch.tensor(1.0)

        # Step should not raise an error (though parameters may become NaN)
        try:
            optimizer.step()
        except Exception as e:
            pytest.fail(f"Optimizer raised an exception on NaN gradient: {e}")

    def test_inf_gradient_handling(self):
        """Optimizer should not crash when gradients contain Inf."""
        model = SimpleQuadraticModel()
        optimizer = optim.SGD(model.parameters(), lr=0.01)

        model.a.grad = torch.tensor(float('inf'))
        model.b.grad = torch.tensor(1.0)

        try:
            optimizer.step()
        except Exception as e:
            pytest.fail(f"Optimizer raised an exception on Inf gradient: {e}")

    def test_zero_gradient_no_update(self):
        """If gradient is zero, parameters should remain unchanged."""
        model = SimpleQuadraticModel(a=1.0, b=2.0)
        optimizer = optim.SGD(model.parameters(), lr=0.1)

        # Set gradients to zero
        model.a.grad = torch.tensor(0.0)
        model.b.grad = torch.tensor(0.0)

        a_before = model.a.clone().detach()
        b_before = model.b.clone().detach()

        optimizer.step()

        assert torch.equal(model.a, a_before), "Parameter 'a' changed despite zero gradient"
        assert torch.equal(model.b, b_before), "Parameter 'b' changed despite zero gradient"

# =============================================================================
# Property-Based Tests (Hypothesis)
# =============================================================================

class TestOptimizerProperties:
    """Use property-based testing to verify optimizer invariants."""

    @given(
        lr=st.floats(min_value=1e-5, max_value=1.0),
        momentum=st.floats(min_value=0.0, max_value=0.99),
        weight_decay=st.floats(min_value=0.0, max_value=0.1),
    )
    def test_sgd_hyperparameter_range(self, lr: float, momentum: float, weight_decay: float):
        """SGD should accept a wide range of valid hyperparameters."""
        model = SimpleQuadraticModel()
        try:
            optimizer = optim.SGD(model.parameters(), lr=lr, momentum=momentum, weight_decay=weight_decay)
            assert optimizer.param_groups[0]['lr'] == lr
        except Exception as e:
            pytest.fail(f"SGD rejected valid hyperparameters: {e}")

    @given(
        lr=st.floats(min_value=1e-5, max_value=1.0),
        betas=st.tuples(
            st.floats(min_value=0.8, max_value=0.99),
            st.floats(min_value=0.9, max_value=0.999),
        ),
    )
    def test_adam_hyperparameter_range(self, lr: float, betas: Tuple[float, float]):
        """Adam should accept a wide range of valid hyperparameters."""
        model = SimpleQuadraticModel()
        try:
            optimizer = optim.Adam(model.parameters(), lr=lr, betas=betas)
            assert optimizer.param_groups[0]['lr'] == lr
            assert optimizer.param_groups[0]['betas'] == betas
        except Exception as e:
            pytest.fail(f"Adam rejected valid hyperparameters: {e}")

# =============================================================================
# Benchmark Tests (Performance and Convergence Speed)
# =============================================================================

class TestConvergenceSpeed:
    """Measure and assert on convergence speed for different optimizers."""

    @pytest.mark.benchmark
    def test_adam_converges_faster_than_sgd(self):
        """Adam should reach a target loss in fewer epochs than SGD on a simple problem."""
        model_sgd = SimpleQuadraticModel(a=5.0, b=-3.0)
        model_adam = SimpleQuadraticModel(a=5.0, b=-3.0)

        optimizer_sgd = optim.SGD(model_sgd.parameters(), lr=0.01)
        optimizer_adam = optim.Adam(model_adam.parameters(), lr=0.01)

        x = torch.linspace(-10, 10, 200)
        y_true = torch.sin(x)  # Non-trivial target

        target_loss = 0.5
        epochs_sgd = 0
        epochs_adam = 0

        # Train SGD
        for epoch in range(1000):
            optimizer_sgd.zero_grad()
            y_pred = model_sgd(x)
            loss = quadratic_loss(y_pred, y_true)
            loss.backward()
            optimizer_sgd.step()
            epochs_sgd = epoch + 1
            if loss.item() < target_loss:
                break

        # Train Adam
        for epoch in range(1000):
            optimizer_adam.zero_grad()
            y_pred = model_adam(x)
            loss = quadratic_loss(y_pred, y_true)
            loss.backward()
            optimizer_adam.step()
            epochs_adam = epoch + 1
            if loss.item() < target_loss:
                break

        assert epochs_adam < epochs_sgd, (
            f"Adam took {epochs_adam} epochs, SGD took {epochs_sgd} epochs. "
            f"Expected Adam to converge faster."
        )

# =============================================================================
# Execution
# =============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
```

## Running the Tests

```bash
# Install dependencies
pip install pytest numpy torch hypothesis

# Run all tests
pytest test_research_plan_machine_learning_optimization_.md -v

# Run only unit tests
pytest test_research_plan_machine_learning_optimization_.md -v -k "TestOptimizer"

# Run benchmark tests (may be slower)
pytest test_research_plan_machine_learning_optimization_.md -v -k "benchmark"
```

## Expected Output
- All unit and integration tests should pass.
- Edge case tests should demonstrate robustness (no crashes).
- Benchmark tests should show Adam converging faster than SGD on the given problem.
- Property-based tests should verify hyperparameter range acceptance.

## Notes
- This test suite assumes the research plan proposes standard optimization algorithms (SGD, Adam, AdamW) with common hyperparameters.
- If the research plan includes novel optimizers (e.g., LAMB, RAdam, Lookahead), additional test classes should be added.
- The `SimpleQuadraticModel` is a minimal surrogate; replace with actual model architecture from the research plan for full validation.