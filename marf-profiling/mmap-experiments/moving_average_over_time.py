"""
Ingests the data and outputs a moving average. THe average uses the last NUM_AVERAGE_COMPONENTS
times, and outputs a (idx, time, moving_average) point every REPORT_EVERY_N_ROWS.

Usage:
    python3 speed_over_time.py /path/to/raw/data.csv

Output:
    rows of the form:
        idx,time,moving_average
"""

import sys
import collections
import json

file_name = sys.argv[1]
in_file = open(file_name, 'r')
lines = in_file.readlines()

marf_times = []
for line in lines:
    parts = line.strip().split(',')
    idx = int(parts[0])
    value = float(parts[1])
    marf_times.append(value)

REPORT_EVERY_N_ROWS = 10000
moving_average = marf_times[0]

## Compute a rolling average.
NUM_AVERAGE_COMPONENTS = 1000
average_total = 0.0
average_components = []

for idx, time in enumerate(marf_times):
    if len(average_components) < NUM_AVERAGE_COMPONENTS:
        average_components.append(time)
        average_total += time
    else:
        comp_idx = idx % len(average_components)
        average_total -= average_components[comp_idx]
        average_total += time
        average_components[comp_idx] = time

    average = average_total / len(average_components)

    if idx % REPORT_EVERY_N_ROWS == 0:
        parts = [idx, time, average]
        csv = ','.join([str(f) for f in parts])
        print(csv)
