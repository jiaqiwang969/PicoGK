#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "[1/4] Rust fmt"
(cd "$ROOT_DIR/picogk-rs" && cargo fmt --check)

echo "[2/4] Rust clippy"
(cd "$ROOT_DIR/picogk-rs" && cargo clippy --all-targets --all-features -- -D warnings)

echo "[3/4] Rust tests"
(cd "$ROOT_DIR/picogk-rs" && cargo test --all-features)

echo "[4/4] C# ↔ Rust parity (AdvancedExamples)"
if command -v dotnet >/dev/null 2>&1; then
  (cd "$ROOT_DIR/picogk-rs" && cargo test --test csharp_advanced_examples_parity -- --ignored)
else
  echo "dotnet not found; skipping parity test"
fi

echo "OK"

