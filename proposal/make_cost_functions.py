#! /usr/bin/env python3

import csv

to_skip = []

function_name_to_type = {}

# How to scale from a runtime dimension of nanoseconds
# into the unitless runtime dimension used by the block limit
SCALE_NUMERATOR = 5e9
SCALE_DENOMINATOR = 3e10 * 100
SCALE = SCALE_NUMERATOR / SCALE_DENOMINATOR

def make_clarity_cost_function(function_name, a_const, b_const):
    a_float = float(a_const)
    b_float = float(b_const)

    # only apply `SCALE` to the constants once we know the
    # type of function we have. we do not have any function
    # types currently that require non-linear scaling, but
    # we may eventually

    if function_name in function_name_to_type:
        func_type = function_name_to_type[function_name]
        if func_type == "constant":
            b_int = int(b_float * SCALE)
            return """(define-read-only (%s (n uint))
    (runtime u%s))
""" % (function_name, b_int)
        elif func_type == "linear":
            a_int = int(a_float * SCALE)
            b_int = int(b_float * SCALE)
            return """(define-read-only (%s (n uint))
    (runtime (linear n u%s u%s)))
""" % (function_name, a_int, b_int)
        elif func_type == "logn":
            a_int = int(a_float * SCALE)
            b_int = int(b_float * SCALE)
            return """(define-read-only (%s (n uint))
    (runtime (logn n u%s u%s)))
""" % (function_name, a_int, b_int)
        elif func_type == "nlogn":
            a_int = int(a_float * SCALE)
            b_int = int(b_float * SCALE)
            return """(define-read-only (%s (n uint))
    (runtime (nlogn n u%s u%s)))
""" % (function_name, a_int, b_int)
        else:
            print("ERROR: unknown type %s for %s" % (func_type, function_name))
    else:
        print("SPECIAL CASE: %s" % function_name)

def load_function_name_types(filename):
    with open(filename, 'r') as raw_file:
        csv_reader = csv.DictReader(raw_file, delimiter=',')
        for row in csv_reader:
            function_name_to_type[row['function_name']] = row['type_name'].strip()

def main():
    load_function_name_types('./function_name_to_type.csv')

    clarity_functions = []

    with open('./estimated_constants.csv', 'r') as raw_file:
        csv_reader = csv.DictReader(raw_file, delimiter=',')
        for row in csv_reader:
            result = make_clarity_cost_function(
                row["function"].strip(),
                row["a"].strip(),
                row["b"].strip()
            )

            if result != None:
                clarity_functions.append(result)

    with open('new_costs.clar', 'w') as out_file:
        for clarity_function in clarity_functions:
            out_file.write(clarity_function)
            out_file.write('\n')

main()
