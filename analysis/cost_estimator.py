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

def load_reports(criterion_dir):
    paths = [f.path for f in os.scandir(criterion_dir) if f.is_dir()]
    try:
        paths.remove(criterion_dir + '/report')
    except:
        pass

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
    os.makedirs("analysis_target/graphs", exist_ok=True)
    plt.savefig("analysis_target/graphs/{}.svg".format(name))

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


def main():
    load_reports(sys.argv[1])

    pd.set_option('display.max_rows', 500)
    pd.set_option('display.max_columns', 500)
    df = pd.DataFrame(report)


    output = pd.DataFrame(columns=["a", "b"])

    estimate_plot(df, 'cost_analysis_type_annotate', output)
    estimate_plot(df, 'cost_analysis_type_check', output)
    estimate_plot(df, 'cost_analysis_type_lookup', output)
    estimate_plot(df, 'cost_analysis_visit', output)
    estimate_plot(df, 'cost_analysis_iterable_func', output)
    estimate_plot(df, 'cost_analysis_option_cons', output)
    estimate_plot(df, 'cost_analysis_option_check', output)
    estimate_plot(df, 'cost_analysis_bind_name', output)
    estimate_plot(df, 'cost_analysis_list_items_check', output)
    estimate_plot(df, 'cost_analysis_check_tuple_get', output, nlogn)
    # estimate_plot(df, 'cost_analysis_check_tuple_merge ', output)
    estimate_plot(df, 'cost_analysis_check_tuple_cons', output, nlogn)
    estimate_plot(df, 'cost_analysis_tuple_items_check', output)
    estimate_plot(df, 'cost_analysis_check_let', output)
    estimate_plot(df, 'cost_analysis_lookup_function', output)
    estimate_plot(df, 'cost_analysis_lookup_function_types', output)
    estimate_plot(df, 'cost_analysis_lookup_variable_const', output)
    estimate_plot(df, 'cost_analysis_lookup_variable_depth', output, nlogn)
    estimate_plot(df, 'cost_arithmetic_only_checker', output)
    estimate_plot(df, 'cost_read_only', output)
    estimate_plot(df, 'cost_trait_checker', output)
    estimate_plot(df, 'cost_type_checker', output)
    estimate_plot(df, 'cost_ast_parse', output)
    estimate_plot(df, 'cost_ast_cycle_detection', output)
    estimate_plot(df, 'cost_analysis_storage', output)
    estimate_plot(df, 'cost_analysis_use_trait_entry', output)
    estimate_plot(df, 'cost_analysis_get_function_entry', output)
    # do we have a benchmark for this?
    # estimate_plot(df, 'cost_analysis_fetch_contract_entry', output)
    estimate_plot(df, 'cost_lookup_variable_depth', output)
    estimate_plot(df, 'cost_lookup_variable_size', output)
    estimate_plot(df, 'cost_lookup_function', output)
    estimate_plot(df, 'cost_bind_name', output)
    estimate_plot(df, 'cost_inner_type_check_cost', output)
    estimate_plot(df, 'cost_user_function_application', output)
    estimate_plot(df, 'cost_let', output)
    estimate_plot(df, 'cost_if', output)
    estimate_plot(df, 'cost_asserts', output)
    estimate_plot(df, 'cost_map', output)
    estimate_plot(df, 'cost_filter', output)
    estimate_plot(df, 'cost_len', output)
    estimate_plot(df, 'cost_element_at', output)
    estimate_plot(df, 'cost_index_of', output)
    estimate_plot(df, 'cost_fold', output)
    estimate_plot(df, 'cost_list_cons', output)
    estimate_plot(df, 'cost_type_parse_step', output)
    estimate_plot(df, 'cost_tuple_get', output, nlogn)
    estimate_plot(df, 'cost_tuple_merge', output)
    estimate_plot(df, 'cost_tuple_cons', output, nlogn)
    estimate_plot(df, 'cost_add', output)
    estimate_plot(df, 'cost_sub', output)
    estimate_plot(df, 'cost_mul', output)
    estimate_plot(df, 'cost_div', output)
    estimate_plot(df, 'cost_geq', output)
    estimate_plot(df, 'cost_leq', output)
    estimate_plot(df, 'cost_le', output)
    estimate_plot(df, 'cost_ge', output)
    estimate_plot(df, 'cost_int_cast', output)
    estimate_plot(df, 'cost_mod', output)
    estimate_plot(df, 'cost_pow', output)
    estimate_plot(df, 'cost_sqrti', output)
    estimate_plot(df, 'cost_log2', output)
    estimate_plot(df, 'cost_xor', output)
    estimate_plot(df, 'cost_not', output)
    estimate_plot(df, 'cost_eq', output)
    estimate_plot(df, 'cost_begin', output)
    estimate_plot(df, 'cost_hash160', output)
    estimate_plot(df, 'cost_sha256', output)
    estimate_plot(df, 'cost_sha512', output)
    estimate_plot(df, 'cost_sha512t256', output)
    estimate_plot(df, 'cost_keccak256', output)
    estimate_plot(df, 'cost_secp256k1recover', output)
    estimate_plot(df, 'cost_secp256k1verify', output)
    estimate_plot(df, 'cost_print', output)
    estimate_plot(df, 'cost_some_cons', output)
    estimate_plot(df, 'cost_ok_cons', output)
    estimate_plot(df, 'cost_err_cons', output)
    estimate_plot(df, 'cost_default_to', output)
    estimate_plot(df, 'cost_unwrap_ret', output)
    estimate_plot(df, 'cost_unwrap_err_or_ret', output)
    estimate_plot(df, 'cost_is_okay', output)
    estimate_plot(df, 'cost_is_none', output)
    estimate_plot(df, 'cost_is_err', output)
    estimate_plot(df, 'cost_is_some', output)
    estimate_plot(df, 'cost_unwrap', output)
    estimate_plot(df, 'cost_unwrap_err', output)
    estimate_plot(df, 'cost_try_ret', output)
    estimate_plot(df, 'cost_match', output)
    estimate_plot(df, 'cost_or', output)
    estimate_plot(df, 'cost_and', output)
    estimate_plot(df, 'cost_append', output)
    estimate_plot(df, 'cost_concat', output)
    estimate_plot(df, 'cost_as_max_len', output)
    estimate_plot(df, 'cost_contract_call', output)
    estimate_plot(df, 'cost_contract_of', output)
    estimate_plot(df, 'cost_principal_of', output)
    estimate_plot(df, 'cost_at_block', output)
    estimate_plot(df, 'cost_load_contract', output)
    estimate_plot(df, 'cost_create_map', output)
    estimate_plot(df, 'cost_create_var', output)
    estimate_plot(df, 'cost_create_nft', output)
    estimate_plot(df, 'cost_create_ft', output)
    estimate_plot(df, 'cost_fetch_entry', output)
    estimate_plot(df, 'cost_set_entry', output)
    estimate_plot(df, 'cost_fetch_var', output)
    estimate_plot(df, 'cost_set_var', output)
    estimate_plot(df, 'cost_contract_storage', output)
    estimate_plot(df, 'cost_block_info', output)
    estimate_plot(df, 'cost_stx_balance', output)
    estimate_plot(df, 'cost_stx_transfer', output)
    estimate_plot(df, 'cost_ft_mint', output)
    estimate_plot(df, 'cost_ft_transfer', output)
    estimate_plot(df, 'cost_ft_balance', output)
    estimate_plot(df, 'cost_nft_mint', output)
    estimate_plot(df, 'cost_nft_transfer', output)
    estimate_plot(df, 'cost_nft_owner', output)
    estimate_plot(df, 'cost_ft_get_supply', output)
    estimate_plot(df, 'cost_ft_burn', output)
    estimate_plot(df, 'cost_nft_burn', output)
    estimate_plot(df, 'poison_microblock', output)
    estimate_plot(df, 'cost_buff_to_int_le', output)
    estimate_plot(df, 'cost_buff_to_uint_le', output)
    estimate_plot(df, 'cost_buff_to_int_be', output)
    estimate_plot(df, 'cost_buff_to_uint_be', output)
    estimate_plot(df, 'cost_is_standard', output)
    estimate_plot(df, 'cost_principal_destruct', output)
    estimate_plot(df, 'cost_principal_construct', output)
    estimate_plot(df, 'cost_string_to_int', output)
    estimate_plot(df, 'cost_string_to_uint', output)
    estimate_plot(df, 'cost_int_to_ascii', output)
    estimate_plot(df, 'cost_int_to_utf8', output)
    estimate_plot(df, 'cost_burn_block_info', output)
    estimate_plot(df, 'cost_stx_account', output)
    estimate_plot(df, 'cost_slice', output)
    estimate_plot(df, 'cost_to_consensus_buff', output)
    estimate_plot(df, 'cost_from_consensus_buff', output, nlogn)
    estimate_plot(df, 'cost_stx_transfer_memo', output)

    os.makedirs("analysis_target", exist_ok=True)
    output.to_csv("analysis_target/cost_constants.csv", index_label="function")


main()
