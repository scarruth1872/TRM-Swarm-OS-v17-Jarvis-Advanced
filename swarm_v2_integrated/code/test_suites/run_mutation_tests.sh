#!/bin/bash
# Mutation testing runner script
# Usage: ./run_mutation_tests.sh [--quick]

set -euo pipefail

echo "=== Swarm OS Mutation Testing Suite ==="
echo "Starting at $(date)"

# Configuration
MUTMUT_BIN="mutmut"
COVERAGE_THRESHOLD=80
MUTATION_THRESHOLD=70

# Check if mutmut is installed
if ! command -v $MUTMUT_BIN &> /dev/null; then
    echo "ERROR: mutmut is not installed. Install with: pip install mutmut"
    exit 1
fi

# Run mutation testing
echo "Running mutation tests..."
if [[ "${1:-}" == "--quick" ]]; then
    echo "Quick mode: limiting mutations for faster execution"
    $MUTMUT_BIN run --paths-to-mutate swarm_components/ --tests-dir test_suites/ --runner "pytest -x" --timeout 30
else
    $MUTMUT_BIN run --paths-to-mutate swarm_components/ --tests-dir test_suites/ --timeout 60
fi

# Generate report
echo "Generating mutation report..."
$MUTMUT_BIN results

# Parse results
TOTAL_MUTANTS=$($MUTMUT_BIN results --format json | python3 -c "import sys, json; data = json.load(sys.stdin); print(data['total'])")
KILLED_MUTANTS=$($MUTMUT_BIN results --format json | python3 -c "import sys, json; data = json.load(sys.stdin); print(data['killed'])")
SURVIVED_MUTANTS=$((TOTAL_MUTANTS - KILLED_MUTANTS))

if [ $TOTAL_MUTANTS -eq 0 ]; then
    echo "ERROR: No mutants were generated. Check test discovery."
    exit 1
fi

MUTATION_SCORE=$((KILLED_MUTANTS * 100 / TOTAL_MUTANTS))
echo ""
echo "=== Mutation Testing Results ==="
echo "Total Mutants: $TOTAL_MUTANTS"
echo "Killed: $KILLED_MUTANTS"
echo "Survived: $SURVIVED_MUTANTS"
echo "Mutation Score: $MUTATION_SCORE%"

# Check against threshold
if [ $MUTATION_SCORE -lt $MUTATION_THRESHOLD ]; then
    echo "FAIL: Mutation score below threshold of $MUTATION_THRESHOLD%"
    echo "Review surviving mutants to identify test gaps."
    exit 1
else
    echo "PASS: Mutation score meets threshold of $MUTATION_THRESHOLD%"
fi

# Check coverage
COVERAGE=$($MUTMUT_BIN results --format json | python3 -c "import sys, json; data = json.load(sys.stdin); print(data.get('coverage', 0))")
if [ $COVERAGE -lt $COVERAGE_THRESHOLD ]; then
    echo "WARNING: Coverage below threshold of $COVERAGE_THRESHOLD%"
    echo "Current coverage: $COVERAGE%"
fi

echo ""
echo "Mutation testing completed at $(date)"
echo "=== End of Mutation Testing Suite ==="