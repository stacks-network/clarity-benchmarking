#! /usr/bin/env python3

import sys
import os
import json
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
from sklearn.linear_model import LinearRegression

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

def estimate_params(df, name, transform):
    X = df.index.values.reshape(-1, 1)
    Y = df[name].values.reshape(-1, 1)
    X = X[np.logical_not(np.isnan(Y))].reshape(-1, 1)
    Y = Y[np.logical_not(np.isnan(Y))]
    X = transform(X)
#     Y = transform(Y)

    linear_regressor = LinearRegression()
    linear_regressor.fit(X, Y)
    Y_pred = linear_regressor.predict(X)

    b = linear_regressor.intercept_
    if b < 0:
        b = max(Y[0] - linear_regressor.coef_, 0)
    a = linear_regressor.coef_

    return (a, b)

def logn(n):
    return np.log2(n)

def nlogn(n):
    return n * np.log2(n)

def plot(df, name, a, b, transform):
    Y = df[name].values.reshape(-1, 1)
    X = df.index.values.reshape(-1, 1)
    X = X[np.logical_not(np.isnan(Y))]
    X = transform(X)
    Y = Y[np.logical_not(np.isnan(Y))]

    y_pred = a*X + b

    plt.scatter(X, Y, color='orange')
    plt.suptitle(name)
    plt.plot(X, y_pred, color='blue')
    os.makedirs("analysis_output/graphs", exist_ok=True)
    plt.savefig("analysis_output/graphs/{}.svg".format(name))

def estimate_plot(df, fun_name, output, transform = lambda x: x):
    if fun_name not in df:
        print("Function not found in criterion result set: {}".format(fun_name))
        return

    a, b = estimate_params(df, fun_name, transform)
    print(a, b)
#     output.loc[fun_name] = [a.squeeze(), b.squeeze()]
    if not isinstance(a, int):
        a = a.squeeze()
    if not isinstance(b, int):
        b = b.squeeze()
    output.loc[fun_name] = [a, b]
    plot(df, fun_name, a, b, transform)


def main(PLOT_NAME):
    load_reports()

    pd.set_option('display.max_rows', 500)
    pd.set_option('display.max_columns', 500)
    df = pd.DataFrame(report)


    output = pd.DataFrame(columns=["a", "b"])

    estimate_plot(df, PLOT_NAME, output)

    os.makedirs("analysis_output", exist_ok=True)
    output.to_csv("analysis_output/cost_constants.csv")


main(PLOT_NAME = sys.argv[1])
