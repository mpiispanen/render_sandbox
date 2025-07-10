#!/bin/bash

# Script to check and optionally fix code formatting
# Usage: ./check_format.sh [--fix]

set -e

echo "Checking code formatting..."

if [[ "$1" == "--fix" ]]; then
    echo "Running cargo fmt to fix formatting..."
    cargo fmt
    echo "✅ Formatting applied successfully"
else
    echo "Running cargo fmt --check..."
    if cargo fmt --check; then
        echo "✅ Code formatting is correct"
    else
        echo "❌ Code formatting issues found"
        echo "Run './check_format.sh --fix' to automatically fix formatting"
        exit 1
    fi
fi

echo "Running clippy for additional code quality checks..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo "✅ Clippy checks passed"
else
    echo "❌ Clippy found issues that need to be fixed"
    exit 1
fi

echo "✅ All formatting and linting checks passed!"