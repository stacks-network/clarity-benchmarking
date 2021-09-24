#! /usr/bin/env python3

import os
import json
import seaborn as sns
import csv

sns.set_theme()

report = {}
function_name_to_type = {}

def load_reports():
    paths = [f.path for f in os.scandir('target/criterion/') if f.is_dir()]
    paths.remove('target/criterion/report')

    for path in paths:
        print('path:', path)
        path_end = path.split('/')[-1].split(' ')
        function_name = path_end[0]

        size_paths = [f.path for f in os.scandir(path) if f.is_dir() and not f.path.endswith('report')]

        for size_path in size_paths:
            print('size_path:', size_path)
            size = int(size_path.split('/')[-1])
            with open(os.path.join(size_path, 'base', 'estimates.json'), 'r') as f:
                data = json.load(f)
                if function_name not in report:
                    report[function_name] = {}
                report[function_name][size] = data['median']['point_estimate']

def main():
    load_reports()

main()
