# More Accurate Cost Functions for Clarity Native Functions

This document is a first draft proposal for more accurate runtime cost
assessment in the Stacks blockchain. Runtime costs are assessed in the
Stacks blockchain through a dynamic cost tracker in the Clarity
VM. Updates to these costs requires either execution of a cost vote or
a hard fork in the network consensus. The initial costs for the Stacks
blockchain were proposed and merged with the launch of Stacks 2.0, and
these costs were intentionally pessimistic--- erring on the side of
caution prevents Stacks nodes from falling behind the rest of the
network. However, these pessimistic cost estimates unnecessarily limit
the throughput of the Stacks network, when even without further
optimization, Stacks nodes would be able to process more transactions
in each block.

These assessed costs interact with the total block limit for the
Stacks chain: each Stacks block has an upper bound on the total
runtime of its transactions (as well as the total number of state
reads, writes, total amount of data written, and total amount of data
read). Reducing the runtime cost assessed for functions will allow
those functions to be invoked more times per block, meaning that a
block could fit more transactions (without otherwise changing the
block limit).

## Determining accurate runtime costs

This proposal sets out to update the _runtime_ cost assessed for
various Clarity native functions. The other dimensions of Clarity
execution costs are unchanged: the number of MARF reads, the number of
MARF writes, the length of any state writes, and the length of any
state reads. This is because those 4 other dimensions *are* measured
accurately.

The goal of this proposal is to make the total real runtime of a full
block less than 30 seconds. 30 seconds is a short enough period of time
that prospective miners should be able to process a new block before
the next Bitcoin block 95% of the time (`exp( -1/20 ) ~= 95%`).

To determine a new proposed cost for a Clarity function, we executed a
set of benchmarks for each Clarity cost function in the
[clarity-benchmarking](https://github.com/stacks-network/clarity-benchmarking)
Github repository. After running these benchmarks, constant factors in
the runtime functions were fitted using linear regression (given a
transform). These benchmarks produced regression fitted functions
for each Clarity cost function, for example:

```
runtime_ns(cost_secp256k1verify) = 8126809.571429
runtime_ns(cost_or) = 2064.4713444648587 * input_len + 91676.397154
```

The runtime block limit in the Stacks network is `5e9` (unitless), and
the goal of this proposal is that this should correspond to 30 seconds
or `3e10` nanoseconds. So, to convert the `runtime_ns` functions into
runtimes for the Stacks network, we have the simple conversion:

```
runtime_stacks = runtime_ns * 5e9 / 3e10ns
```

## Proposed costs

### Old and New Cost Functions Table

| Runtime cost | New function | Old function |
| ----------- | ----------- | ----------- |
| cost_analysis_type_annotate | f(x) := 3*x + 13 | f(x) := 1000*x + 1000 |
| cost_analysis_type_lookup | f(x) := 1*x + 6 | f(x) := 1000*x + 1000 |
| cost_analysis_visit | f(x) := 16 | f(x) := 1000 |
| cost_analysis_option_cons | f(x) := 60 | f(x) := 1000 |
| cost_analysis_option_check | f(x) := 118 | f(x) := 1000 |
| cost_analysis_bind_name | f(x) := 14*x + 174 | f(x) := 1000*x + 1000 |
| cost_analysis_list_items_check | f(x) := 25*x + 5 | f(x) := 1000*x + 1000 |
| cost_analysis_check_tuple_get | f(x) := 1*log(x) + 2 | f(x) := 1000*log(x) + 1000 |
| cost_analysis_check_tuple_cons | f(x) := 11*x*log(x) + 155 | f(x) := 1000*x*log(x) + 1000 |
| cost_analysis_tuple_items_check | f(x) := 13*x + 56 | f(x) := 1000*x + 1000 |
| cost_analysis_check_let | f(x) := 46*x + 93 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_function | f(x) := 19 | f(x) := 1000 |
| cost_analysis_lookup_function_types | f(x) := 1*x + 28 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_variable_const | f(x) := 15 | f(x) := 1000 |
| cost_analysis_lookup_variable_depth | f(x) := 1*x*log(x) + 65 | f(x) := 1000*x*log(x) + 1000 |
| cost_ast_parse | f(x) := 173*x + 285411 | f(x) := 1000*x + 1000 |
| cost_ast_cycle_detection | f(x) := 142*x + 25 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_depth | f(x) := 2*x + 14 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_size | f(x) := 2*x + 0 | f(x) := 1000*x + 1000 |
| cost_lookup_function | f(x) := 25 | f(x) := 1000 |
| cost_bind_name | f(x) := 270 | f(x) := 1000 |
| cost_inner_type_check_cost | f(x) := 2*x + 10 | f(x) := 1000*x + 1000 |
| cost_user_function_application | f(x) := 26*x + 0 | f(x) := 1000*x + 1000 |
| cost_let | f(x) := 1*x + 290 | f(x) := 1000*x + 1000 |
| cost_if | f(x) := 199 | f(x) := 1000 |
| cost_asserts | f(x) := 158 | f(x) := 1000 |
| cost_map | f(x) := 1181*x + 3113 | f(x) := 1000*x + 1000 |
| cost_filter | f(x) := 464 | f(x) := 1000 |
| cost_len | f(x) := 566 | f(x) := 1000 |
| cost_element_at | f(x) := 489 | f(x) := 1000 |
| cost_fold | f(x) := 521 | f(x) := 1000 |
| cost_type_parse_step | f(x) := 5 | f(x) := 1000 |
| cost_tuple_get | f(x) := 4*x*log(x) + 1697 | f(x) := 1000*x*log(x) + 1000 |
| cost_tuple_merge | f(x) := 212*x + 248 | f(x) := 1000*x + 1000 |
| cost_tuple_cons | f(x) := 10*x*log(x) + 2166 | f(x) := 1000*x*log(x) + 1000 |
| cost_add | f(x) := 10*x + 171 | f(x) := 1000*x + 1000 |
| cost_sub | f(x) := 11*x + 170 | f(x) := 1000*x + 1000 |
| cost_mul | f(x) := 12*x + 169 | f(x) := 1000*x + 1000 |
| cost_div | f(x) := 13*x + 166 | f(x) := 1000*x + 1000 |
| cost_geq | f(x) := 179 | f(x) := 1000 |
| cost_leq | f(x) := 178 | f(x) := 1000 |
| cost_le | f(x) := 169 | f(x) := 1000 |
| cost_ge | f(x) := 170 | f(x) := 1000 |
| cost_int_cast | f(x) := 174 | f(x) := 1000 |
| cost_mod | f(x) := 183 | f(x) := 1000 |
| cost_pow | f(x) := 185 | f(x) := 1000 |
| cost_sqrti | f(x) := 179 | f(x) := 1000 |
| cost_log2 | f(x) := 172 | f(x) := 1000 |
| cost_xor | f(x) := 178 | f(x) := 1000 |
| cost_not | f(x) := 177 | f(x) := 1000 |
| cost_eq | f(x) := 7*x + 182 | f(x) := 1000*x + 1000 |
| cost_begin | f(x) := 212 | f(x) := 1000 |
| cost_secp256k1recover | f(x) := 14346 | f(x) := 1000 |
| cost_secp256k1verify | f(x) := 13544 | f(x) := 1000 |
| cost_some_cons | f(x) := 232 | f(x) := 1000 |
| cost_ok_cons | f(x) := 231 | f(x) := 1000 |
| cost_err_cons | f(x) := 219 | f(x) := 1000 |
| cost_default_to | f(x) := 260 | f(x) := 1000 |
| cost_unwrap_ret | f(x) := 350 | f(x) := 1000 |
| cost_unwrap_err_or_ret | f(x) := 333 | f(x) := 1000 |
| cost_is_okay | f(x) := 288 | f(x) := 1000 |
| cost_is_none | f(x) := 246 | f(x) := 1000 |
| cost_is_err | f(x) := 299 | f(x) := 1000 |
| cost_is_some | f(x) := 246 | f(x) := 1000 |
| cost_unwrap | f(x) := 309 | f(x) := 1000 |
| cost_unwrap_err | f(x) := 275 | f(x) := 1000 |
| cost_try_ret | f(x) := 294 | f(x) := 1000 |
| cost_match | f(x) := 306 | f(x) := 1000 |
| cost_or | f(x) := 3*x + 152 | f(x) := 1000*x + 1000 |
| cost_and | f(x) := 3*x + 153 | f(x) := 1000*x + 1000 |
| cost_append | f(x) := 1*x + 1026 | f(x) := 1000*x + 1000 |
| cost_concat | f(x) := 1*x + 867 | f(x) := 1000*x + 1000 |
| cost_as_max_len | f(x) := 508 | f(x) := 1000 |
| cost_contract_call | f(x) := 164 | f(x) := 1000 |
| cost_contract_of | f(x) := 45847 | f(x) := 1000 |
| cost_principal_of | f(x) := 35 | f(x) := 1000 |


### New Costs as Clarity Functions

```
(define-read-only (cost_analysis_type_annotate (n uint))
    (runtime (linear n u3 u13)))

(define-read-only (cost_analysis_type_lookup (n uint))
    (runtime (linear n u1 u6)))

(define-read-only (cost_analysis_visit (n uint))
    (runtime u16))

(define-read-only (cost_analysis_option_cons (n uint))
    (runtime u60))

(define-read-only (cost_analysis_option_check (n uint))
    (runtime u118))

(define-read-only (cost_analysis_bind_name (n uint))
    (runtime (linear n u14 u174)))

(define-read-only (cost_analysis_list_items_check (n uint))
    (runtime (linear n u25 u5)))

(define-read-only (cost_analysis_check_tuple_get (n uint))
    (runtime (logn n u1 u2)))

(define-read-only (cost_analysis_check_tuple_cons (n uint))
    (runtime (nlogn n u11 u155)))

(define-read-only (cost_analysis_tuple_items_check (n uint))
    (runtime (linear n u13 u56)))

(define-read-only (cost_analysis_check_let (n uint))
    (runtime (linear n u46 u93)))

(define-read-only (cost_analysis_lookup_function (n uint))
    (runtime u19))

(define-read-only (cost_analysis_lookup_function_types (n uint))
    (runtime (linear n u1 u28)))

(define-read-only (cost_analysis_lookup_variable_const (n uint))
    (runtime u15))

(define-read-only (cost_analysis_lookup_variable_depth (n uint))
    (runtime (nlogn n u1 u65)))

(define-read-only (cost_ast_parse (n uint))
    (runtime (linear n u173 u285411)))

(define-read-only (cost_ast_cycle_detection (n uint))
    (runtime (linear n u142 u25)))

(define-read-only (cost_lookup_variable_depth (n uint))
    (runtime (linear n u2 u14)))

(define-read-only (cost_lookup_variable_size (n uint))
    (runtime (linear n u2 u0)))

(define-read-only (cost_lookup_function (n uint))
    (runtime u25))

(define-read-only (cost_bind_name (n uint))
    (runtime u270))

(define-read-only (cost_inner_type_check_cost (n uint))
    (runtime (linear n u2 u10)))

(define-read-only (cost_user_function_application (n uint))
    (runtime (linear n u26 u0)))

(define-read-only (cost_let (n uint))
    (runtime (linear n u1 u290)))

(define-read-only (cost_if (n uint))
    (runtime u199))

(define-read-only (cost_asserts (n uint))
    (runtime u158))

(define-read-only (cost_map (n uint))
    (runtime (linear n u1181 u3113)))

(define-read-only (cost_filter (n uint))
    (runtime u464))

(define-read-only (cost_len (n uint))
    (runtime u566))

(define-read-only (cost_element_at (n uint))
    (runtime u489))

(define-read-only (cost_fold (n uint))
    (runtime u521))

(define-read-only (cost_type_parse_step (n uint))
    (runtime u5))

(define-read-only (cost_tuple_get (n uint))
    (runtime (nlogn n u4 u1697)))

(define-read-only (cost_tuple_merge (n uint))
    (runtime (linear n u212 u248)))

(define-read-only (cost_tuple_cons (n uint))
    (runtime (nlogn n u10 u2166)))

(define-read-only (cost_add (n uint))
    (runtime (linear n u10 u171)))

(define-read-only (cost_sub (n uint))
    (runtime (linear n u11 u170)))

(define-read-only (cost_mul (n uint))
    (runtime (linear n u12 u169)))

(define-read-only (cost_div (n uint))
    (runtime (linear n u13 u166)))

(define-read-only (cost_geq (n uint))
    (runtime u179))

(define-read-only (cost_leq (n uint))
    (runtime u178))

(define-read-only (cost_le (n uint))
    (runtime u169))

(define-read-only (cost_ge (n uint))
    (runtime u170))

(define-read-only (cost_int_cast (n uint))
    (runtime u174))

(define-read-only (cost_mod (n uint))
    (runtime u183))

(define-read-only (cost_pow (n uint))
    (runtime u185))

(define-read-only (cost_sqrti (n uint))
    (runtime u179))

(define-read-only (cost_log2 (n uint))
    (runtime u172))

(define-read-only (cost_xor (n uint))
    (runtime u178))

(define-read-only (cost_not (n uint))
    (runtime u177))

(define-read-only (cost_eq (n uint))
    (runtime (linear n u7 u182)))

(define-read-only (cost_begin (n uint))
    (runtime u212))

(define-read-only (cost_secp256k1recover (n uint))
    (runtime u14346))

(define-read-only (cost_secp256k1verify (n uint))
    (runtime u13544))

(define-read-only (cost_some_cons (n uint))
    (runtime u232))

(define-read-only (cost_ok_cons (n uint))
    (runtime u231))

(define-read-only (cost_err_cons (n uint))
    (runtime u219))

(define-read-only (cost_default_to (n uint))
    (runtime u260))

(define-read-only (cost_unwrap_ret (n uint))
    (runtime u350))

(define-read-only (cost_unwrap_err_or_ret (n uint))
    (runtime u333))

(define-read-only (cost_is_okay (n uint))
    (runtime u288))

(define-read-only (cost_is_none (n uint))
    (runtime u246))

(define-read-only (cost_is_err (n uint))
    (runtime u299))

(define-read-only (cost_is_some (n uint))
    (runtime u246))

(define-read-only (cost_unwrap (n uint))
    (runtime u309))

(define-read-only (cost_unwrap_err (n uint))
    (runtime u275))

(define-read-only (cost_try_ret (n uint))
    (runtime u294))

(define-read-only (cost_match (n uint))
    (runtime u306))

(define-read-only (cost_or (n uint))
    (runtime (linear n u3 u152)))

(define-read-only (cost_and (n uint))
    (runtime (linear n u3 u153)))

(define-read-only (cost_append (n uint))
    (runtime (linear n u1 u1026)))

(define-read-only (cost_concat (n uint))
    (runtime (linear n u1 u867)))

(define-read-only (cost_as_max_len (n uint))
    (runtime u508))

(define-read-only (cost_contract_call (n uint))
    (runtime u164))

(define-read-only (cost_contract_of (n uint))
    (runtime u45847))

(define-read-only (cost_principal_of (n uint))
    (runtime u35))
```

## Requires more analysis

Several cost functions require more benchmarking analysis. For many of
these, the runtime costs of many of these functions need to be
separated from the MARF operations that they use. For others, more
analysis is needed to determine the appropriate asymptotic function
and constants for capturing the runtime of the operation.

```
    cost_at_block
    cost_create_ft
    cost_block_info
    cost_stx_balance
    cost_stx_transfer
    cost_ft_mint
    cost_ft_transfer
    cost_ft_balance
    cost_ft_get_supply
    cost_ft_burn
    poison_microblock
    cost_analysis_storage
    cost_analysis_use_trait_entry
    cost_analysis_get_function_entry
    cost_load_contract
    cost_create_map
    cost_create_var
    cost_create_nft
    cost_fetch_entry
    cost_set_entry
    cost_fetch_var
    cost_set_var
    cost_contract_storage
    cost_nft_mint
    cost_nft_transfer
    cost_nft_owner
    cost_nft_burn
    cost_list_cons
    cost_index_of
    cost_hash160
    cost_sha256
    cost_sha512
    cost_sha512t256
    cost_keccak256
    cost_print
    cost_analysis_iterable_func
    cost_analysis_type_check
```
