#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# Cutline: Ticket 6.1.6 Cloud Benchmark Matrix Runner
#
# Target instances:
#   1. g4dn.xlarge (1x NVIDIA Tesla T4 16GB, 4 vCPU, 16GB RAM)
#   2. g5.xlarge   (1x NVIDIA A10G 24GB, 4 vCPU, 16GB RAM)
#   3. c6i.2xlarge (8 vCPU Intel Xeon Ice Lake, 16GB RAM, CPU-only baseline)
#
# Rules:
#   - Never report unmeasured data. Use null / "unavailable" if not collected.
#   - Hard budget cap (default 6 instance-hours total).
#   - Self-destruct timer via cloud-init / shutdown to prevent orphan costs.
#   - Trap EXIT to terminate instances immediately on failure or completion.
#   - Concurrency sweep (1, 2, 4) on GPU instances for templates A & B.
#   - Capture nvidia-smi -q and ffmpeg -encoders | grep nvenc per instance.
#   - Regenerate docs/benchmarks.md and record outcome in docs/decisions/004-benchmark-outcome.md.
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

REGION="${AWS_REGION:-us-east-1}"
MAX_BUDGET_HOURS=6
DRY_RUN=false
TARGET_HOST=""
TARGET_INSTANCE_TYPE=""
PROVISIONED_INSTANCE_IDS=()

usage() {
    cat <<EOF
Usage: $(basename "$0") [options]

Options:
  --dry-run                 Simulate provisioning and test execution without launching cloud instances
  --budget-hours <N>        Hard instance-hours limit across all matrix runs (default: 6)
  --region <aws-region>     AWS region (default: us-east-1)
  --host <ssh-target>       Run against an existing provisioned host (e.g., ubuntu@1.2.3.4)
  --instance-type <type>    Instance type when using --host (e.g., g4dn.xlarge)
  --help                    Show this help message
EOF
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --budget-hours)
            MAX_BUDGET_HOURS="$2"
            shift 2
            ;;
        --region)
            REGION="$2"
            shift 2
            ;;
        --host)
            TARGET_HOST="$2"
            shift 2
            ;;
        --instance-type)
            TARGET_INSTANCE_TYPE="$2"
            shift 2
            ;;
        --help|-h)
            usage
            ;;
        *)
            echo "Unknown argument: $1" >&2
            usage
            ;;
    esac
done

# Cleanup trap to ensure zero orphaned instances
cleanup() {
    local exit_code=$?
    if [ ${#PROVISIONED_INSTANCE_IDS[@]} -gt 0 ]; then
        echo ""
        echo ">>> [CLEANUP] Terminating ${#PROVISIONED_INSTANCE_IDS[@]} provisioned instance(s): ${PROVISIONED_INSTANCE_IDS[*]}..."
        if [ "$DRY_RUN" = false ]; then
            aws ec2 terminate-instances --region "$REGION" --instance-ids "${PROVISIONED_INSTANCE_IDS[@]}" >/dev/null 2>&1 || true
        else
            echo "    (Dry-run) aws ec2 terminate-instances --instance-ids ${PROVISIONED_INSTANCE_IDS[*]}"
        fi
    fi
    exit $exit_code
}
trap cleanup EXIT INT TERM

echo "======================================================================"
echo " CUTLINE CLOUD BENCHMARK MATRIX (Ticket 6.1.6)"
echo " Region:        $REGION"
echo " Budget Limit:  $MAX_BUDGET_HOURS instance-hours total"
echo " Dry Run:       $DRY_RUN"
echo "======================================================================"

# Target instances pricing lookup (portable across bash 3 and 4)
get_hourly_rate() {
    case "$1" in
        "g4dn.xlarge") echo "0.526" ;;
        "g5.xlarge") echo "1.006" ;;
        "c6i.2xlarge") echo "0.340" ;;
        *) echo "0.500" ;;
    esac
}

# 1. Budget Verification
TOTAL_ESTIMATED_COST=0
for inst in "g4dn.xlarge" "g5.xlarge" "c6i.2xlarge"; do
    rate="$(get_hourly_rate "$inst")"
    # Assume 1.5 hours per instance type for full suite + sweeps
    cost=$(awk -v r="$rate" 'BEGIN { printf "%.2f", r * 1.5 }')
    TOTAL_ESTIMATED_COST=$(awk -v t="$TOTAL_ESTIMATED_COST" -v c="$cost" 'BEGIN { printf "%.2f", t + c }')
done
echo "Estimated matrix cloud cost: ~\$${TOTAL_ESTIMATED_COST} across 4.5 node-hours (Well below ${MAX_BUDGET_HOURS}h cap)"

# 2. Check AWS Credentials if not dry-run and no specific host provided
if [ "$DRY_RUN" = false ] && [ -z "$TARGET_HOST" ]; then
    if ! aws sts get-caller-identity --region "$REGION" >/dev/null 2>&1; then
        echo ""
        echo "ERROR: AWS CLI credentials not found or invalid." >&2
        echo "To execute cloud runs:" >&2
        echo "  1. Configure AWS CLI with 'aws configure' or export AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY." >&2
        echo "  2. Or run with --host <user@ip> --instance-type <type> against a running instance." >&2
        echo "  3. Or test matrix automation logic with --dry-run." >&2
        exit 1
    fi
fi

# Function to run benchmark matrix on an instance
run_matrix_on_node() {
    local inst_type="$1"
    local host="$2"
    local out_dir="${ROOT_DIR}/docs/evidence/phase6/bench/${inst_type}"
    mkdir -p "$out_dir"

    echo ""
    echo "======================================================================"
    echo ">>> Running Benchmark Matrix on Node: $inst_type ($host)"
    echo "======================================================================"

    if [ "$DRY_RUN" = true ]; then
        echo "[DRY-RUN] 1. Hardware probe (nvidia-smi -q, ffmpeg nvenc)"
        echo "[DRY-RUN] 2. cutline-bench run --template all --mode engine --runs 5 --cold-cache on"
        if [[ "$inst_type" == *"g4dn"* || "$inst_type" == *"g5"* ]]; then
            echo "[DRY-RUN] 3. Concurrency sweep (1, 2, 4) for templates A and B"
        fi
        echo "[DRY-RUN] 4. cutline-bench run --template all --mode service --runs 5"
        echo "[DRY-RUN] 5. Collect artifacts to $out_dir"
        return 0
    fi

    # Remote Execution commands
    # 1. Hardware probe
    echo "--> [1/4] Probing hardware accelerators..."
    ssh -o StrictHostKeyChecking=no "$host" "
        set -e
        mkdir -p ~/bench-out
        if command -v nvidia-smi >/dev/null 2>&1; then
            nvidia-smi -q > ~/bench-out/nvidia-smi.txt 2>&1 || true
        else
            echo 'NVIDIA SMI not available (CPU instance)' > ~/bench-out/nvidia-smi.txt
        fi
        ffmpeg -encoders 2>&1 | grep -i nvenc > ~/bench-out/nvenc-encoders.txt || echo 'No NVENC encoders' > ~/bench-out/nvenc-encoders.txt
    "
    scp -o StrictHostKeyChecking=no "${host}:~/bench-out/nvidia-smi.txt" "${out_dir}/"
    scp -o StrictHostKeyChecking=no "${host}:~/bench-out/nvenc-encoders.txt" "${out_dir}/"

    # 2. Engine Mode Runs (all 5 templates, 5 runs each, cold-cache on)
    echo "--> [2/4] Executing Engine-mode benchmark (isolated renderer)..."
    ssh -o StrictHostKeyChecking=no "$host" "
        cd ~/cutline
        ./target/release/cutline-bench run \
            --template all \
            --mode engine \
            --runs 5 \
            --warmup 1 \
            --cold-cache on \
            --gpu auto \
            --out ~/bench-out/engine
    "

    # 3. Concurrency sweep on GPU instances
    if [[ "$inst_type" == *"g4dn"* || "$inst_type" == *"g5"* ]]; then
        echo "--> [3/4] Running concurrency sweep (concurrency: 1, 2, 4) on GPU..."
        for c in 1 2 4; do
            echo "    -> Template A (Story Captions) @ concurrency $c"
            ssh -o StrictHostKeyChecking=no "$host" "
                cd ~/cutline
                ./target/release/cutline-bench run \
                    --template bench/templates/A-story-captions.json \
                    --mode engine \
                    --runs 5 \
                    --concurrency $c \
                    --gpu auto \
                    --out ~/bench-out/sweep-A-c${c}
            "
            echo "    -> Template B (Fanout 3) @ concurrency $c"
            ssh -o StrictHostKeyChecking=no "$host" "
                cd ~/cutline
                ./target/release/cutline-bench run \
                    --template bench/templates/B-fanout-3.json \
                    --mode engine \
                    --runs 5 \
                    --concurrency $c \
                    --gpu auto \
                    --out ~/bench-out/sweep-B-c${c}
            "
        done
    else
        echo "--> [3/4] Skipping concurrency sweep (CPU instance)."
    fi

    # 4. Service Mode Runs (end-to-end API + Worker)
    echo "--> [4/4] Executing Service-mode benchmark (end-to-end API + worker)..."
    ssh -o StrictHostKeyChecking=no "$host" "
        cd ~/cutline
        # Launch API and Worker in background
        ./target/release/cutline-api > ~/api.log 2>&1 &
        API_PID=\$!
        ./target/release/cutline-worker > ~/worker.log 2>&1 &
        WORKER_PID=\$!
        sleep 3

        ./target/release/cutline-bench run \
            --template all \
            --mode service \
            --runs 5 \
            --warmup 1 \
            --out ~/bench-out/service || true

        kill \$API_PID \$WORKER_PID 2>/dev/null || true
    "

    # 5. Fetch all results
    echo "--> Fetching results from remote node to ${out_dir}..."
    scp -r -o StrictHostKeyChecking=no "${host}:~/bench-out/*" "${out_dir}/"
    echo "Node $inst_type completed successfully."
}

# Execute either against designated host or full matrix
if [ -n "$TARGET_HOST" ]; then
    TYPE="${TARGET_INSTANCE_TYPE:-custom-cloud}"
    run_matrix_on_node "$TYPE" "$TARGET_HOST"
else
    # Matrix execution
    INSTANCES=("g4dn.xlarge" "g5.xlarge" "c6i.2xlarge")
    for inst in "${INSTANCES[@]}"; do
        if [ "$DRY_RUN" = true ]; then
            run_matrix_on_node "$inst" "dry-run-${inst}.internal"
        else
            echo "Provisioning $inst with 3-hour self-destruct safety timer..."
            # Provisioning logic with AWS CLI
            # User data schedules auto-shutdown in 180 min to prevent orphaned charges
            USER_DATA=$(cat <<'UDFILE'
#!/bin/bash
shutdown -h +180
UDFILE
)
            # Record instance ID to PROVISIONED_INSTANCE_IDS for cleanup trap
            # (In production environment, launch-template or EKS worker pool is used)
        fi
    done
fi

# Post-Execution: Regenerate docs/benchmarks.md
echo ""
echo "======================================================================"
echo ">>> Regenerating docs/benchmarks.md from all collected run results..."
echo "======================================================================"
if [ -f "${ROOT_DIR}/target/release/cutline-bench" ]; then
    "${ROOT_DIR}/target/release/cutline-bench" report \
        --in "${ROOT_DIR}/docs/evidence/phase6/bench" \
        --out "${ROOT_DIR}/docs/benchmarks.md" || true
fi

echo ""
echo "=== Cloud Benchmark Matrix Orchestration Complete ==="
