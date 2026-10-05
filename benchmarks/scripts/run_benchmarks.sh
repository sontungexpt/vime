#!/usr/bin/env bash
# Run benchmarks and store results with commit info
# Usage: ./run_benchmarks.sh [commit_hash]

set -euo pipefail

REPO_ROOT="/home/stilux/Data/workspace/vime"
ENGINE_DIR="${REPO_ROOT}/engine"
BENCH_DIR="${REPO_ROOT}/benchmarks"
RESULTS_DIR="${BENCH_DIR}/results"
TIMESTAMP=$(date -u +"%Y%m%d_%H%M%S")
COMMIT=$(cd "${ENGINE_DIR}" && git rev-parse --short HEAD)
BRANCH=$(cd "${ENGINE_DIR}" && git rev-parse --abbrev-ref HEAD)
COMMIT_MSG=$(cd "${ENGINE_DIR}" && git log -1 --pretty=%B | head -1 | tr -d '"')

mkdir -p "${RESULTS_DIR}"

echo "Running benchmarks for ${COMMIT} (${BRANCH})"
echo "Commit: ${COMMIT_MSG}"

cd "${ENGINE_DIR}"

# Run benchmarks with JSON output
cargo bench --bench bench_small_vec -- --output-format json 2>&1 | tee /tmp/bench_small_vec.json
cargo bench --bench bench_array_vec -- --output-format json 2>&1 | tee /tmp/bench_array_vec.json

# Extract benchmark results using criterion's JSON output
# Also capture summary in human-readable format
cargo bench --bench bench_small_vec 2>&1 | grep -E "^──|push|extend|spill|memcpy" | tee "${RESULTS_DIR}/small_vec_${TIMESTAMP}_${COMMIT}.txt"
cargo bench --bench bench_array_vec 2>&1 | grep -E "^──|push|extend|memcpy|indexed|per-item" | tee "${RESULTS_DIR}/array_vec_${TIMESTAMP}_${COMMIT}.txt"

# Create JSON summary
cat > "${RESULTS_DIR}/summary_${TIMESTAMP}_${COMMIT}.json" <<EOJ
{
  "timestamp": "${TIMESTAMP}",
  "commit": "${COMMIT}",
  "branch": "${BRANCH}",
  "commit_message": "${COMMIT_MSG}",
  "small_vec_results_file": "small_vec_${TIMESTAMP}_${COMMIT}.txt",
  "array_vec_results_file": "array_vec_${TIMESTAMP}_${COMMIT}.txt"
}
EOJ

# Also append to history CSV
HISTORY_FILE="${RESULTS_DIR}/history.csv"
if [ ! -f "${HISTORY_FILE}" ]; then
  echo "timestamp,commit,branch,commit_msg,small_vec_onset_ns,small_vec_spill_onset_ns,small_vec_spill_raw_ns,array_vec_onset_ns,array_vec_push_1_ns,array_vec_push_3_ns" > "${HISTORY_FILE}"
fi

# Parse key metrics (simplified - in reality you'd use jq on criterion JSON)
# For now just log that results were saved
echo "Results saved to ${RESULTS_DIR}/"

