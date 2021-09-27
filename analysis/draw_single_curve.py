#! /usr/bin/env python3

import sys
import os
import json
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
from sklearn.linear_model import LinearRegression
from sklearn.metrics import r2_score

sns.set_theme()

report = {}
function_name_to_type = {}

def load_function_name_types(filename):
    with open(filename, 'r') as raw_file:
        csv_reader = csv.DictReader(raw_file, delimiter=',')
        for row in csv_reader:
            function_name_to_type[row['function_name']] = row['type_name'].strip()

def load_reports():
    paths = [f.path for f in os.scandir('target/criterion/') if f.is_dir()]
    paths.remove('target/criterion/report')

    for path in paths:
        path_end = path.split('/')[-1].split(' ')
        function_name = path_end[0]

        size_paths = [f.path for f in os.scandir(path) if f.is_dir() and not f.path.endswith('report')]

        for size_path in size_paths:
            size = int(size_path.split('/')[-1])
            with open(os.path.join(size_path, 'base', 'estimates.json'), 'r') as f:
                data = json.load(f)
                if function_name not in report:
                    report[function_name] = {}
                report[function_name][size] = data['median']['point_estimate']

def estimate_params(df, name):
    Y = df.values.reshape(-1, 1)
    X = df.index.values.reshape(-1, 1)
    X = X[np.logical_not(np.isnan(Y))].reshape(-1, 1)
    Y = Y[np.logical_not(np.isnan(Y))].reshape(-1, 1)

    linear_regressor = LinearRegression()
    linear_regressor.fit(X, Y)
    return linear_regressor


def print_metrics(Y, y_pred, plot_name):
    r2 = r2_score(Y, y_pred)
    print(','.join([plot_name, str(r2)]))

def plot(df, plot_name, linear_model):
    Y = df.values.reshape(-1, 1)
    X = df.index.values.reshape(-1, 1)
    X = X[np.logical_not(np.isnan(Y))].reshape(-1, 1)
    Y = Y[np.logical_not(np.isnan(Y))].reshape(-1, 1)

    y_pred = linear_model.predict(X)

    plt.scatter(X, Y, color='orange')
    plt.suptitle(plot_name)
    plt.plot(X, y_pred, color='blue')
    os.makedirs("analysis_output/graphs", exist_ok=True)
    plt.savefig("analysis_output/graphs/{}.svg".format(plot_name))

    print_metrics(Y, y_pred, plot_name)

def estimate_plot(df, function_name):
    linear_model = estimate_params(df, function_name)
    plot(df, function_name, linear_model)


def main(PLOT_NAME):
    load_reports()

    pd.set_option('display.max_rows', 500)
    pd.set_option('display.max_columns', 500)
    df = pd.DataFrame(report)
    df.dropna()

    estimate_plot(df[PLOT_NAME], PLOT_NAME)


main(PLOT_NAME = sys.argv[1])
