# More Accurate Cost Functions for Clarity Native Functions

This document is a first draft proposal for more accurate runtime cost
assessment in the Stacks blockchain. Runtime costs are assessed in the
Stacks blockchain through a dynamic cost tracker in the Clarity
VM. Updates to these costs requires either execution of a cost vote or
a hard fork in the network consensus. The initial costs for the Stacks
blockchain were proposed and merged with the launch of Stacks 2.0, and
these costs were intentionally pessimistic--- erring on the side of
caution prevents Stacks nodes from falling behind the rest of the
network.

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
[clarity-benchmarking](https://github.com/blockstack/clarity-benchmarking)
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

```
(define-read-only (cost_analysis_type_annotate (n uint))
    (runtime (linear n u186 u680)))

(define-read-only (cost_analysis_type_check (n uint))
    (runtime (linear n u6414 u0)))

(define-read-only (cost_analysis_type_lookup (n uint))
    (runtime (linear n u79 u313)))

(define-read-only (cost_analysis_visit (n uint))
    (runtime u830))

(define-read-only (cost_analysis_iterable_func (n uint))
    (runtime (linear n u13176 u0)))

(define-read-only (cost_analysis_option_cons (n uint))
    (runtime u3018))

(define-read-only (cost_analysis_option_check (n uint))
    (runtime u5908))

(define-read-only (cost_analysis_bind_name (n uint))
    (runtime (linear n u742 u8734)))

(define-read-only (cost_analysis_list_items_check (n uint))
    (runtime (linear n u1273 u296)))

(define-read-only (cost_analysis_check_tuple_get (n uint))
    (runtime (logn n u1 u100)))

(define-read-only (cost_analysis_check_tuple_cons (n uint))
    (runtime (nlogn n u571 u7789)))

(define-read-only (cost_analysis_tuple_items_check (n uint))
    (runtime (linear n u692 u2803)))

(define-read-only (cost_analysis_check_let (n uint))
    (runtime (linear n u2318 u4662)))

(define-read-only (cost_analysis_lookup_function (n uint))
    (runtime u997))

(define-read-only (cost_analysis_lookup_function_types (n uint))
    (runtime (linear n u81 u1402)))

(define-read-only (cost_analysis_lookup_variable_const (n uint))
    (runtime u762))

(define-read-only (cost_analysis_lookup_variable_depth (n uint))
    (runtime (nlogn n u37 u3270)))

(define-read-only (cost_ast_parse (n uint))
    (runtime (linear n u8657 u14270569)))

(define-read-only (cost_ast_cycle_detection (n uint))
    (runtime (linear n u7116 u1271)))

(define-read-only (cost_lookup_variable_depth (n uint))
    (runtime (linear n u139 u738)))

(define-read-only (cost_lookup_variable_size (n uint))
    (runtime (linear n u120 u28)))

(define-read-only (cost_lookup_function (n uint))
    (runtime u1286))

(define-read-only (cost_bind_name (n uint))
    (runtime u13508))

(define-read-only (cost_inner_type_check_cost (n uint))
    (runtime (linear n u139 u509)))

(define-read-only (cost_user_function_application (n uint))
    (runtime (linear n u1336 u0)))

(define-read-only (cost_let (n uint))
    (runtime (linear n u0 u14528)))

(define-read-only (cost_if (n uint))
    (runtime u9962))

(define-read-only (cost_asserts (n uint))
    (runtime u7937))

(define-read-only (cost_map (n uint))
    (runtime (linear n u59086 u155667)))

(define-read-only (cost_filter (n uint))
    (runtime u23203))

(define-read-only (cost_len (n uint))
    (runtime u28318))

(define-read-only (cost_element_at (n uint))
    (runtime u24468))

(define-read-only (cost_index_of (n uint))
    (runtime (linear n u0 u27155)))

(define-read-only (cost_fold (n uint))
    (runtime u26058))

(define-read-only (cost_list_cons (n uint))
    (runtime (linear n u0 u13485)))

(define-read-only (cost_type_parse_step (n uint))
    (runtime u263))

(define-read-only (cost_tuple_get (n uint))
    (runtime (nlogn n u218 u84887)))

(define-read-only (cost_tuple_merge (n uint))
    (runtime (linear n u10604 u12411)))

(define-read-only (cost_tuple_cons (n uint))
    (runtime (nlogn n u534 u108305)))

(define-read-only (cost_add (n uint))
    (runtime (linear n u547 u8571)))

(define-read-only (cost_sub (n uint))
    (runtime (linear n u557 u8506)))

(define-read-only (cost_mul (n uint))
    (runtime (linear n u628 u8460)))

(define-read-only (cost_div (n uint))
    (runtime (linear n u666 u8329)))

(define-read-only (cost_geq (n uint))
    (runtime u8973))

(define-read-only (cost_leq (n uint))
    (runtime u8936))

(define-read-only (cost_le (n uint))
    (runtime u8493))

(define-read-only (cost_ge (n uint))
    (runtime u8507))

(define-read-only (cost_int_cast (n uint))
    (runtime u8714))

(define-read-only (cost_mod (n uint))
    (runtime u9199))

(define-read-only (cost_pow (n uint))
    (runtime u9261))

(define-read-only (cost_sqrti (n uint))
    (runtime u8961))

(define-read-only (cost_log2 (n uint))
    (runtime u8641))

(define-read-only (cost_xor (n uint))
    (runtime u8938))

(define-read-only (cost_not (n uint))
    (runtime u8865))

(define-read-only (cost_eq (n uint))
    (runtime (linear n u380 u9126)))

(define-read-only (cost_begin (n uint))
    (runtime u10648))

(define-read-only (cost_hash160 (n uint))
    (runtime (linear n u0 u14718)))

(define-read-only (cost_sha256 (n uint))
    (runtime (linear n u0 u12751)))

(define-read-only (cost_sha512 (n uint))
    (runtime (linear n u0 u12554)))

(define-read-only (cost_sha512t256 (n uint))
    (runtime (linear n u0 u12646)))

(define-read-only (cost_keccak256 (n uint))
    (runtime (linear n u0 u13509)))

(define-read-only (cost_secp256k1recover (n uint))
    (runtime u717341))

(define-read-only (cost_secp256k1verify (n uint))
    (runtime u677234))

(define-read-only (cost_print (n uint))
    (runtime (linear n u0 u86726)))

(define-read-only (cost_some_cons (n uint))
    (runtime u11641))

(define-read-only (cost_ok_cons (n uint))
    (runtime u11581))

(define-read-only (cost_err_cons (n uint))
    (runtime u10980))

(define-read-only (cost_default_to (n uint))
    (runtime u13018))

(define-read-only (cost_unwrap_ret (n uint))
    (runtime u17500))

(define-read-only (cost_unwrap_err_or_ret (n uint))
    (runtime u16695))

(define-read-only (cost_is_okay (n uint))
    (runtime u14404))

(define-read-only (cost_is_none (n uint))
    (runtime u12300))

(define-read-only (cost_is_err (n uint))
    (runtime u14989))

(define-read-only (cost_is_some (n uint))
    (runtime u12316))

(define-read-only (cost_unwrap (n uint))
    (runtime u15452))

(define-read-only (cost_unwrap_err (n uint))
    (runtime u13776))

(define-read-only (cost_try_ret (n uint))
    (runtime u14746))

(define-read-only (cost_match (n uint))
    (runtime u15327))

(define-read-only (cost_or (n uint))
    (runtime (linear n u172 u7639)))

(define-read-only (cost_and (n uint))
    (runtime (linear n u173 u7674)))

(define-read-only (cost_append (n uint))
    (runtime (linear n u0 u51333)))

(define-read-only (cost_concat (n uint))
    (runtime (linear n u0 u43361)))

(define-read-only (cost_as_max_len (n uint))
    (runtime u25432))

(define-read-only (cost_contract_call (n uint))
    (runtime u8223))

(define-read-only (cost_contract_of (n uint))
    (runtime u2292392))

(define-read-only (cost_principal_of (n uint))
    (runtime u1793))
```
