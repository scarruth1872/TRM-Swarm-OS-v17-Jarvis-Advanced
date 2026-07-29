#!/bin/bash
# Run all non-slow, non-integration tests
cd "$(dirname "$0")"
python -m pytest -v --tb=short -m "not slow and not integration" "$@"
