#!/usr/bin/env python3
"""Parse criterion JSON output and store in structured format."""
import json
import sys
import subprocess
import os
from datetime import datetime
from pathlib import Path

REPO_ROOT = Path("/home/stilux/Data/workspace/vime")
ENGINE_DIR = REPO_ROOT / "engine"
BENCH_DIR = REPO_ROOT / "benchmarks"
RESULTS_DIR = BENCH_DIR / "results"

def run_cmd(cmd, cwd=None):
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True, cwd=cwd)
    return result.stdout, result.stderr, result.returncode

def get_git_info():
    stdout, _, _ = run_cmd("git rev-parse --short HEAD", ENGINE_DIR)
    commit = stdout.strip()
    stdout, _, _ = run_cmd("git rev-parse --abbrev-ref HEAD", ENGINE_DIR)
    branch = stdout.strip()
    stdout, _, _ = run_cmd("git log -1 --pretty=%B", ENGINE_DIR)
    msg = stdout.strip().split('\n')[0]
    return commit, branch, msg

def run_benchmark(bench_name):
    """Run cargo bench and return criterion JSON output."""
    cmd = f"cargo bench --bench {bench_name} -- --output-format json"
    stdout, stderr, code = run_cmd(cmd, ENGINE_DIR)
    if code != 0:
        print(f"Warning: {bench_name} failed: {stderr}", file=sys.stderr)
        return None
    return stdout

def parse_criterion_json(json_output):
    """Parse criterion JSON lines into structured data."""
    results = {}
    for line in json_output.strip().split('\n'):
        try:
            data = json.loads(line)
            if data.get('type') == 'BenchmarkCompleted':
                name = data['id']
                # Extract mean and std dev
                mean_ns = data['mean']['point_estimate'] / 1_000_000  # ns
                results[name] = {
                    'mean_ns': mean_ns,
                    'std_dev_ns': data['mean']['std_dev'] / 1_000_000,
                }
        except json.JSONDecodeError:
            continue
    return results

def extract_key_metrics(results, prefix=""):
    """Extract key metrics from benchmark results."""
    metrics = {}
    for name, data in results.items():
        full_name = f"{prefix}{name}" if prefix else name
        # Simplify name for CSV columns
        simple = name.lower().replace(' ', '_').replace('(', '').replace(')', '').replace('/', '_')
        metrics[simple] = data['mean_ns']
    return metrics

def save_results(commit, branch, msg, small_vec_results, array_vec_results):
    timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
    commit_short = commit[:8]
    
    # Save raw JSON
    (RESULTS_DIR / f"small_vec_{timestamp}_{commit_short}.json").write_text(json.dumps(small_vec_results, indent=2))
    (RESULTS_DIR / f"array_vec_{timestamp}_{commit_short}.json").write_text(json.dumps(array_vec_results, indent=2))
    
    # Save summary
    summary = {
        "timestamp": timestamp,
        "commit": commit,
        "branch": branch,
        "commit_message": msg,
    }
    (RESULTS_DIR / f"summary_{timestamp}_{commit_short}.json").write_text(json.dumps(summary, indent=2))
    
    # Append to CSV history
    history_file = RESULTS_DIR / "history.csv"
    small_metrics = extract_key_metrics(small_vec_results, "small_vec_")
    array_metrics = extract_key_metrics(array_vec_results, "array_vec_")
    
    # Build row
    row = {
        'timestamp': timestamp,
        'commit': commit,
        'branch': branch,
        'commit_message': msg[:100],  # truncate
        **small_metrics,
        **array_metrics,
    }
    
    # Write CSV
    import csv
    file_exists = history_file.exists()
    with open(history_file, 'a', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=sorted(row.keys()))
        if not file_exists:
            writer.writeheader()
        writer.writerow(row)
    
    print(f"Saved results to {RESULTS_DIR}/")

def main():
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    
    commit, branch, msg = get_git_info()
    print(f"Commit: {commit} ({branch}) - {msg[:60]}")
    
    print("Running benchmarks...")
    small_vec_json = run_benchmark("bench_small_vec")
    array_vec_json = run_benchmark("bench_array_vec")
    
    if small_vec_json:
        small_vec_results = parse_criterion_json(small_vec_json)
        print(f"SmallVec: {len(small_vec_results)} benchmarks")
    else:
        small_vec_results = {}
        
    if array_vec_json:
        array_vec_results = parse_criterion_json(array_vec_json)
        print(f"ArrayVec: {len(array_vec_results)} benchmarks")
    else:
        array_vec_results = {}
    
    save_results(commit, branch, msg, small_vec_results, array_vec_results)
    print("Done!")

if __name__ == "__main__":
    main()
