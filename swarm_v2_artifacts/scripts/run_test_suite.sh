#!/bin/bash
set -euo pipefail

# Swarm OS Test Suite Runner
# Usage: ./scripts/run_test_suite.sh [--fast] [--coverage] [--stress]

MODE="full"
COVERAGE=false
STRESS=false

for arg in "$@"; do
    case $arg in
        --fast) MODE="fast" ;;
        --coverage) COVERAGE=true ;;
        --stress) STRESS=true ;;
    esac
done

echo "🔍 Validating components.json..."
python -c "
import json
with open('components.json') as f:
    data = json.load(f)
assert 'components' in data
print(f'✅ Found {len(data[\"components\"])} components')
"

echo "🧪 Running test suite in ${MODE} mode..."

PYTEST_ARGS=(
    "--strict-markers"
    "--tb=short"
)

if [ "$MODE" == "fast" ]; then
    PYTEST_ARGS+=("-m" "not slow and not flaky")
    PYTEST_ARGS+=("-x")  # Stop on first failure
else
    PYTEST_ARGS+=("-m" "not flaky")
    PYTEST_ARGS+=("--reruns" "2")
    PYTEST_ARGS+=("--reruns-delay" "1")
fi

if [ "$COVERAGE" == true ]; then
    PYTEST_ARGS+=(
        "--cov=src"
        "--cov-report=term-missing"
        "--cov-report=html:coverage_html"
        "--cov-fail-under=85"
    )
fi

if [ "$STRESS" == true ]; then
    PYTEST_ARGS+=("--count" "3")  # Requires pytest-repeat
fi

# Run tests
pytest "${PYTEST_ARGS[@]}"

echo "✅ Test suite completed successfully"