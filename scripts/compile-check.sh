#!/usr/bin/env bash
# Validate that every component's example code compiles.
#
# Generates a wrapped `fn main` program for each component (via `tuiframe code <name>`),
# writes each as a separate cargo example, and compiles them all with `cargo build --examples`.
# Reports any component whose generated code fails to compile.
#
# Usage:
#   ./scripts/compile-check.sh              # check all components
#   ./scripts/compile-check.sh block table  # check specific components
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/debug/tuiframe-cli"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cd "$ROOT"
cargo build --quiet --bin tuiframe-cli

if [ $# -gt 0 ]; then
    NAMES=("$@")
else
    mapfile -t NAMES < <("$BIN" list --json | python3 -c '
import json,sys
for cat in json.load(sys.stdin):
    for c in cat.get("components", []):
        print(c["name"])
')
fi

mkdir -p "$WORK/src" "$WORK/examples"
touch "$WORK/src/lib.rs"
cat > "$WORK/Cargo.toml" <<EOF
[package]
name = "tuiframe-compile-check"
version = "0.1.0"
edition = "2024"

[dependencies]
ratatui = "0.30"
crossterm = "0.29"
tuiframe-viz = { path = "$ROOT/tuiframe-viz" }
EOF

fail=0
for name in "${NAMES[@]}"; do
    if ! "$BIN" code "$name" > "$WORK/examples/$name.rs" 2>/dev/null; then
        echo "SKIP (no code): $name"
        continue
    fi
done

# Batch build first: if it succeeds, every example compiled. Cargo fails fast and
# cancels pending jobs, so on failure it only reports a random subset of broken
# examples. In that case fall back to checking every example individually so the
# reported failure list is complete and deterministic.
if (cd "$WORK" && cargo build --examples --quiet 2> "$WORK/errors.log"); then
    echo "OK: all ${#NAMES[@]} components compile."
    exit 0
fi

echo
echo "Batch build failed; enumerating every failing example individually..."
fail=0
: > "$WORK/fails.txt"
for name in "${NAMES[@]}"; do
    if [ ! -f "$WORK/examples/$name.rs" ]; then
        echo "SKIP (no code): $name"
        continue
    fi
    if ! (cd "$WORK" && cargo build --example "$name" --quiet 2>> "$WORK/errors.log"); then
        echo "$name" >> "$WORK/fails.txt"
        echo "  FAIL: $name"
        fail=$((fail + 1))
    fi
done

echo
echo "Total: $fail of ${#NAMES[@]} components fail to compile."
echo "Failing list: $WORK/fails.txt"
echo "Error log: $WORK/errors.log"
exit 1
