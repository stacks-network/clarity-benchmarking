#! /usr/bin/env python3

import os
import json
import csv
import collections


report = {}
function_name_to_type = {}

def load_reports():
    paths = [f.path for f in os.scandir('target/criterion/') if f.is_dir()]
    paths.remove('target/criterion/report')

    name_to_sizes = collections.defaultdict(list)
    for path in paths:
        path_end = path.split('/')[-1].split(' ')
        function_name = path_end[0]

        size_paths = [f.path for f in os.scandir(path) if f.is_dir() and not f.path.endswith('report')]

        for size_path in size_paths:
            size = int(size_path.split('/')[-1])
            name_to_sizes[function_name].append(size)

    sorted_names = sorted(name_to_sizes.keys())
    for name in sorted_names:
        sizes = name_to_sizes[name]
        print(name, sorted(sizes))
def main():
    load_reports()

main()
