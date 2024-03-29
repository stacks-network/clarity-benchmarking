"""
This script ingests the LOG output of the `stacks-node`, and outputs it as a
csv file, in which each row is a time.

Usage:
    head -n 1000000 ~/data/no_mmap.log | python3 extract_data.py no_mmap > data/no_mmap.txt

Output:
    rows of the form:
        idx,time
"""

import sys
import collections
import json

dir_name = sys.argv[1]

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

marf_key = '/data/' + dir_name + '/mainnet/chainstate/vm/clarity/marf.sqlite'
marf_times = key_to_times[marf_key]
for idx, time in enumerate(marf_times):
    parts = [idx, time]
    csv = ','.join([str(f) for f in parts])
    print(csv)
