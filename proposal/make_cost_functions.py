#! /usr/bin/env python3

import csv

to_skip = [
    # these were measured with MARF operations, but they shouldn't get penalized
    # for MARF performance in runtime
    'cost_at_block', 'cost_create_ft', 'cost_block_info', 'cost_stx_balance',
    'cost_stx_transfer', 'cost_ft_mint', 'cost_ft_transfer', 'cost_ft_balance',
    'cost_ft_get_supply', 'cost_ft_burn', 'poison_microblock', 'cost_analysis_storage',
    'cost_analysis_use_trait_entry', 'cost_analysis_get_function_entry',
    'cost_load_contract', 'cost_create_map', 'cost_create_var', 'cost_create_nft',
    'cost_fetch_entry', 'cost_set_entry', 'cost_fetch_var', 'cost_set_var',
    'cost_contract_storage', 'cost_nft_mint', 'cost_nft_transfer', 'cost_nft_owner',
    'cost_nft_burn',
    # these should be linear regressions, but got measured as a const
    'cost_list_cons',
    'cost_index_of',
    'cost_hash160',
    'cost_sha256',
    'cost_sha512',
    'cost_sha512t256',
    'cost_keccak256',
    'cost_print',
    # these need further analysis
    'cost_analysis_iterable_func',
    'cost_analysis_type_check',
]

function_name_to_type = {}

# How to scale from a runtime dimension of nanoseconds
# into the unitless runtime dimension used by the block limit
SCALE_NUMERATOR = 5e9
SCALE_DENOMINATOR = 3e10 * 100
SCALE = SCALE_NUMERATOR / SCALE_DENOMINATOR

# special functions that use linear scaling on one constant factors
special_functions_lin_scale_1 = {
    "cost_at_block": """
(define-read-only (cost_at_block (n uint))
    {{
        runtime: u{},
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    }})
""",
    "cost_create_ft": """
(define-read-only (cost_create_ft (n uint))
    {{
        runtime: u{},
        write_length: u1,
        write_count: u2,
        read_count: u0,
        read_length: u0
    }})
""",
    "cost_block_info": """
(define-read-only (cost_block_info (n uint))
    {{
        runtime: u{},
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    }})
""",
    "cost_stx_balance": """
(define-read-only (cost_stx_balance (n uint))
    {{
        runtime: u{},
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    }})
""",
    "cost_stx_transfer": """
(define-read-only (cost_stx_transfer (n uint))
    {{
        runtime: u{},
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    }})
""",
    "cost_ft_mint": """
(define-read-only (cost_ft_mint (n uint))
    {{
        runtime: u{},
        write_length: u1,
        write_count: u2,
        read_count: u2,
        read_length: u1
    }})
""",
    "cost_ft_transfer": """
(define-read-only (cost_ft_transfer (n uint))
    {{
        runtime: u{},
        write_length: u1,
        write_count: u2,
        read_count: u2,
        read_length: u1
    }})
""",
    "cost_ft_balance": """
(define-read-only (cost_ft_balance (n uint))
    {{
        runtime: u{},
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    }})
""",
    "cost_ft_get_supply": """
(define-read-only (cost_ft_get_supply (n uint))
    {{
        runtime: u{},
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    }})
""",
    "cost_ft_burn": """
(define-read-only (cost_ft_burn (n uint))
    {{
        runtime: u{},
        write_length: u1,
        write_count: u2,
        read_count: u2,
        read_length: u1
    }})
""",
    "poison_microblock": """
(define-read-only (poison_microblock (n uint))
    {{
        runtime: u{},
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    }})
""",
}
# special functions that use linear scaling on two constant factors
special_functions_lin_scale_2 = {
    "cost_analysis_storage": """(define-read-only (cost_analysis_storage (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u1,
        read_length: u1
    }})
""",
    "cost_analysis_use_trait_entry": """(define-read-only (cost_analysis_use_trait_entry (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: (linear n u1 u1),
        write_count: u0,
        read_count: u1,
        read_length: (linear n u1 u1)
    }})
""",
    "cost_analysis_get_function_entry": """
(define-read-only (cost_analysis_get_function_entry (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: (linear n u1 u1)
    }})
""",
    "cost_load_contract": """
(define-read-only (cost_load_contract (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: u0,
        write_count: u0,
        ;; set to 3 because of the associated metadata loads
        read_count: u3,
        read_length: (linear n u1 u1)
    }})
""",
    "cost_create_map": """
(define-read-only (cost_create_map (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u0,
        read_length: u0
    }})
""",
    "cost_create_var": """
(define-read-only (cost_create_var (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: (linear n u1 u1),
        write_count: u2,
        read_count: u0,
        read_length: u0
    }})
""",
    "cost_create_nft": """
(define-read-only (cost_create_nft (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u0,
        read_length: u0
    }})
""",
    "cost_fetch_entry": """
(define-read-only (cost_fetch_entry (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: (linear n u1 u1)
    }})
""",
    "cost_set_entry": """
(define-read-only (cost_set_entry (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u1,
        read_length: u0
    }})
""",
    "cost_fetch_var": """
(define-read-only (cost_fetch_var (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: (linear n u1 u1)
    }})
""",
    "cost_set_var": """
(define-read-only (cost_set_var (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u1,
        read_length: u0
    }})
""",
    "cost_contract_storage": """
(define-read-only (cost_contract_storage (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u0,
        read_length: u0
    }})
""",
    "cost_nft_mint": """
(define-read-only (cost_nft_mint (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    }})
""",
    "cost_nft_transfer": """
(define-read-only (cost_nft_transfer (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    }})
""",
    "cost_nft_owner": """
(define-read-only (cost_nft_owner (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    }})
""",
    "cost_nft_burn": """
(define-read-only (cost_nft_burn (n uint))
    {{
        runtime: (linear n u{} u{}),
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    }})
""",
}

def make_clarity_cost_function(function_name, a_const, b_const):
    a_float = float(a_const)
    b_float = float(b_const)

    # only apply `SCALE` to the constants once we know the
    # type of function we have. we do not have any function
    # types currently that require non-linear scaling, but
    # we may eventually
    if function_name in to_skip:
        print("SKIP: %s" % function_name)
        return None

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
            return None
    else:
        if function_name in special_functions_lin_scale_2:
            a_int = int(a_float * SCALE)
            b_int = int(b_float * SCALE)
            return special_functions_lin_scale_2[function_name].format(a_int, b_int)
        elif function_name in special_functions_lin_scale_1:
            b_int = int(b_float * SCALE)
            return special_functions_lin_scale_1[function_name].format(b_int)
        else:
            print("Unhandled special case: %s" % function_name)
            return None

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
