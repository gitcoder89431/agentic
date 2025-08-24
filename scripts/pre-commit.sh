#!/bin/bash
# Pre-commit quality checks for Agentic

echo "🔍 Running pre-commit checks..."

echo "📦 Building project..."
cargo build || exit 1

echo "🎨 Checking formatting..."
cargo fmt --check || exit 1

echo "📋 Running clippy lints..."
cargo clippy -- -D warnings || exit 1

echo "🧪 Running tests..."
cargo test || exit 1

echo "✅ All checks passed!"
