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
    (runtime (logn n u0 u2)))

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
    (runtime (nlogn n u0 u65)))

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
    (runtime (linear n u0 u290)))

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
    (runtime (linear n u0 u1026)))

(define-read-only (cost_concat (n uint))
    (runtime (linear n u0 u867)))

(define-read-only (cost_as_max_len (n uint))
    (runtime u508))

(define-read-only (cost_contract_call (n uint))
    (runtime u164))

(define-read-only (cost_contract_of (n uint))
    (runtime u45847))

(define-read-only (cost_principal_of (n uint))
    (runtime u35))

