"""
Usage:
    python3 summarize_data.py /path/to/raw/data.csv

Output:
    to screen, looks like:

```
avg 929.2762000832141
len 747469
```
"""
import sys
import collections
import json

file_name = sys.argv[1]
in_file = open(file_name, 'r')
lines = in_file.readlines()

values = []
for line in lines:
    parts = line.strip().split(',')
    idx = int(parts[0])
    value = float(parts[1])
    values.append(value)

avg = sum(values) / len(values)
print('avg', avg)
print('len', len(values))

