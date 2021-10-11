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

# marf_key = '/data/' + dir_name + '/mainnet/chainstate/vm/clarity/marf.sqlite'
marf_key = '/data/' + dir_name + '/mainnet/chainstate/vm/clarity/marf.sqlite'
marf_times = key_to_times[marf_key]
# print('marf_times', len(marf_times))

select_mod = 10000
break_max = 1000
break_count = 0

moving_average = marf_times[0]
alpha = 0.95

average_window = []

total = 0.0
components = []
replace_idx = None


for idx, time in enumerate(marf_times):
    if len(components) < 1000:
        components.append(time)
        total += time
    else:
        comp_idx = idx % len(components)
        total -= components[comp_idx]
        total += time
        components[comp_idx] = time

    average = total / len(components)

    if idx % select_mod == 0:
        parts = [idx, time, average]
        csv = ','.join([str(f) for f in parts])
        print(csv)

        if break_count == break_max:
            # break
            pass
        break_count += 1


