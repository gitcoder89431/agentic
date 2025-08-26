#!/bin/bash

# Verification script for Agentic project compliance

echo "=== Agentic Project Compliance Verification ==="
echo

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

echo "1. Checking code formatting..."
if cargo fmt -- --check; then
    echo "✅ Code formatting is correct"
else
    echo "❌ Code formatting issues found"
    echo "Run 'cargo fmt' to fix formatting"
fi
echo

echo "2. Running Clippy for code quality checks..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo "✅ Clippy checks passed"
else
    echo "❌ Clippy found issues"
fi
echo

echo "3. Running tests..."
if cargo test; then
    echo "✅ All tests passed"
else
    echo "❌ Some tests failed"
fi
echo

echo "4. Checking compilation..."
if cargo build; then
    echo "✅ Project compiles successfully"
else
    echo "❌ Compilation failed"
fi
echo

echo "=== Verification Complete ==="
echo "Please address any issues above to ensure compliance with project standards."