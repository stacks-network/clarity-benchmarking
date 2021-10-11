import sys
import collections
import json

"""
Converts a Rust-style structured output line to a python-style dict.

Input format is like:
    INFO [1631296771.981741] [src/cost_estimates/pessimistic.rs:213] [chains-coordinator] PessimisticEstimator received event, key: coinbase:runtime, estimate: 0, actual: 1, estimate_err: -1, estimate_err_pct: -1

Output is a dict mapping strings to strings.
"""
def rust_to_map(line):
    parts = line.rstrip().split(', ')
    result = {}
    for part in parts[1:]:
        inner_parts = part.split(': ', 2)
        result [inner_parts[0]] = inner_parts[1]
    return result

key_to_times = collections.defaultdict(list)
for line in sys.stdin:
    if "MARF read" in line:
        kv = rust_to_map(line)
        db = kv['db']
        time_micros = float(kv['time_micros'])
        key_to_times[db].append(time_micros)


for kv, times in key_to_times.items():
    avg = sum(times) / len(times)
    print('kv', kv)
    print('avg', avg)
    print('len', len(times))

