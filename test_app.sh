#!/bin/bash

echo "=== Testing Render Sandbox Application ==="
echo
echo "1. Testing headless mode with default settings:"
cargo run --quiet -- --headless
echo
echo "2. Testing headless mode with verbose output:"
cargo run --quiet -- --headless --verbose
echo
echo "3. Testing headless mode with debug logging:"
cargo run --quiet -- --headless --log-level debug
echo
echo "4. Testing windowed mode (will fail in headless environment):"
cargo run --quiet -- --verbose || echo "Expected failure in headless environment"
echo
echo "5. Running all tests:"
cargo test --quiet
echo
echo "=== All tests completed ==="