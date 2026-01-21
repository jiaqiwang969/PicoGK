#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ ! -f "$ROOT_DIR/PicoGK.csproj" ]]; then
  echo "error: expected PicoGK.csproj in repo root: $ROOT_DIR" >&2
  exit 1
fi

shopt -s nullglob

echo "Cleaning build artifacts..."

# C# build outputs
rm -rf "$ROOT_DIR/bin" "$ROOT_DIR/obj" "$ROOT_DIR/nupkg_out"
rm -rf "$ROOT_DIR/PicoGK_Test/bin" "$ROOT_DIR/PicoGK_Test/obj"
rm -rf "$ROOT_DIR/tools/api-dump/bin" "$ROOT_DIR/tools/api-dump/obj"

# Rust build outputs
rm -rf "$ROOT_DIR/picogk-rs/target"

# Parity outputs (also live under target, but keep explicit for clarity)
rm -rf "$ROOT_DIR/picogk-rs/target/csharp_advanced_examples_out"
rm -rf "$ROOT_DIR/picogk-rs/target/csharp_advanced_examples_baseline"

# Example/demo outputs (examples default to writing into the crate root)
rm -f "$ROOT_DIR/picogk-rs/"*.stl "$ROOT_DIR/picogk-rs/"*.vdb

if [[ "${1:-}" == "--all" ]]; then
  # Large generated outputs under PicoGK_Test (ignored by git, safe to remove).
  rm -f "$ROOT_DIR/PicoGK_Test/"*.stl
  rm -f "$ROOT_DIR/PicoGK_Test/"*.aux "$ROOT_DIR/PicoGK_Test/"*.toc \
    "$ROOT_DIR/PicoGK_Test/"*.synctex.gz "$ROOT_DIR/PicoGK_Test/"*.fls \
    "$ROOT_DIR/PicoGK_Test/"*.fdb_latexmk
fi

echo "Done."
