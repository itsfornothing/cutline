#!/usr/bin/env bash
set -euo pipefail

# Ticket 6.1.5: Local Baseline Benchmark Runner
# Runs all 5 benchmark templates (E, D, B, A, C) with 5 runs each + 1 warmup.
# Executes under --gpu auto and --gpu off.

CHIP="apple-m1"
OUT_BASE="docs/evidence/phase6/bench/local-${CHIP}"
BIN="target/release/cutline-bench"

if [ ! -f "$BIN" ]; then
    echo "Error: $BIN not found. Compile with 'cargo build --release -p cutline-bench'." >&2
    exit 1
fi

TEMPLATES=(
    "E-baseline-10s"
    "D-60fps-text"
    "B-fanout-3"
    "A-story-captions"
    "C-4k-slideshow"
)

echo "=== CUTLINE LOCAL BASELINE BENCHMARK SUITE ==="
echo "Host: $(uname -m) / $(sysctl -n machdep.cpu.brand_string 2>/dev/null || uname -s)"
echo "Power: $(pmset -g batt 2>/dev/null | head -n 1 || echo 'unknown')"
echo "============================================="

# 1. GPU AUTO Mode
echo ""
echo ">>> [1/2] Running benchmark matrix with --gpu auto..."
for tmpl in "${TEMPLATES[@]}"; do
    echo "--- Running $tmpl (--gpu auto) ---"
    "$BIN" run \
        --template "bench/templates/${tmpl}.json" \
        --mode engine \
        --runs 5 \
        --warmup 1 \
        --gpu auto \
        --out "${OUT_BASE}/gpu-auto"
done

# 2. GPU OFF Mode
echo ""
echo ">>> [2/2] Running benchmark matrix with --gpu off..."
for tmpl in "${TEMPLATES[@]}"; do
    echo "--- Running $tmpl (--gpu off) ---"
    "$BIN" run \
        --template "bench/templates/${tmpl}.json" \
        --mode engine \
        --runs 5 \
        --warmup 1 \
        --gpu off \
        --out "${OUT_BASE}/gpu-off"
done

# 3. Generate Report
echo ""
echo ">>> Generating docs/benchmarks.md..."
"$BIN" report \
    --in "${OUT_BASE}/gpu-auto" "${OUT_BASE}/gpu-off" \
    --out docs/benchmarks.md

echo "=== Local Baseline Benchmark Complete ==="
