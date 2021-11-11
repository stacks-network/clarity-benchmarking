
;; the .costs-2 contract

;; Helper Functions

;; Return a Cost Specification with just a runtime cost
(define-private (runtime (r uint))
    {
        runtime: r,
        write_length: u0,
        write_count: u0,
        read_count: u0,
        read_length: u0,
    })

;; Linear cost-assessment function
(define-private (linear (n uint) (a uint) (b uint))
    (+ (* a n) b))

;; LogN cost-assessment function
(define-private (logn (n uint) (a uint) (b uint))
    (+ (* a (log2 n)) b))

;; NLogN cost-assessment function
(define-private (nlogn (n uint) (a uint) (b uint))
    (+ (* a (* n (log2 n))) b))


;; Cost Functions
(define-read-only (cost_analysis_type_annotate (n uint))
    (runtime (linear n u1 u7)))

(define-read-only (cost_analysis_type_lookup (n uint))
    (runtime (linear n u1 u4)))

(define-read-only (cost_analysis_visit (n uint))
    (runtime u0))

(define-read-only (cost_analysis_option_cons (n uint))
    (runtime u4))

(define-read-only (cost_analysis_option_check (n uint))
    (runtime u2))

(define-read-only (cost_analysis_bind_name (n uint))
    (runtime (linear n u1 u132)))

(define-read-only (cost_analysis_list_items_check (n uint))
    (runtime (linear n u1 u3)))

(define-read-only (cost_analysis_check_tuple_get (n uint))
    (runtime (logn n u1 u1)))

(define-read-only (cost_analysis_check_tuple_cons (n uint))
    (runtime (nlogn n u2 u3)))

(define-read-only (cost_analysis_tuple_items_check (n uint))
    (runtime (linear n u1 u44)))

(define-read-only (cost_analysis_check_let (n uint))
    (runtime (linear n u1 u9)))

(define-read-only (cost_analysis_lookup_function (n uint))
    (runtime u15))

(define-read-only (cost_analysis_lookup_function_types (n uint))
    (runtime (linear n u1 u21)))

(define-read-only (cost_analysis_lookup_variable_const (n uint))
    (runtime u11))

(define-read-only (cost_analysis_lookup_variable_depth (n uint))
    (runtime (nlogn n u1 u26)))

(define-read-only (cost_ast_parse (n uint))
    (runtime (linear n u129 u215581)))

(define-read-only (cost_ast_cycle_detection (n uint))
    (runtime (linear n u106 u54)))

(define-read-only (cost_analysis_storage (n uint))
    {
        runtime: (linear n u1 u75),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u1,
        read_length: u1
    })

(define-read-only (cost_analysis_use_trait_entry (n uint))
    {
        runtime: (linear n u7 u542),
        write_length: (linear n u1 u1),
        write_count: u0,
        read_count: u1,
        read_length: (linear n u1 u1)
    })


(define-read-only (cost_analysis_get_function_entry (n uint))
    {
        runtime: (linear n u61 u977),
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: (linear n u1 u1)
    })

(define-read-only (cost_lookup_variable_depth (n uint))
    (runtime (linear n u2 u11)))

(define-read-only (cost_lookup_variable_size (n uint))
    (runtime (linear n u1 u0)))

(define-read-only (cost_lookup_function (n uint))
    (runtime u12))

(define-read-only (cost_bind_name (n uint))
    (runtime u192))

(define-read-only (cost_inner_type_check_cost (n uint))
    (runtime (linear n u1 u7)))

(define-read-only (cost_user_function_application (n uint))
    (runtime (linear n u20 u105)))

(define-read-only (cost_let (n uint))
    (runtime (linear n u110 u646)))

(define-read-only (cost_if (n uint))
    (runtime u150))

(define-read-only (cost_asserts (n uint))
    (runtime u118))

(define-read-only (cost_map (n uint))
    (runtime (linear n u908 u2486)))

(define-read-only (cost_filter (n uint))
    (runtime u345))

(define-read-only (cost_len (n uint))
    (runtime u365))

(define-read-only (cost_element_at (n uint))
    (runtime u464))

(define-read-only (cost_fold (n uint))
    (runtime u362))

(define-read-only (cost_type_parse_step (n uint))
    (runtime u3))

(define-read-only (cost_tuple_get (n uint))
    (runtime (nlogn n u3 u1335)))

(define-read-only (cost_tuple_merge (n uint))
    (runtime (linear n u3 u484)))

(define-read-only (cost_tuple_cons (n uint))
    (runtime (nlogn n u8 u826)))

(define-read-only (cost_add (n uint))
    (runtime (linear n u9 u117)))

(define-read-only (cost_sub (n uint))
    (runtime (linear n u9 u117)))

(define-read-only (cost_mul (n uint))
    (runtime (linear n u10 u117)))

(define-read-only (cost_div (n uint))
    (runtime (linear n u10 u117)))

(define-read-only (cost_geq (n uint))
    (runtime u124))

(define-read-only (cost_leq (n uint))
    (runtime u124))

(define-read-only (cost_le (n uint))
    (runtime u124))

(define-read-only (cost_ge (n uint))
    (runtime u124))

(define-read-only (cost_int_cast (n uint))
    (runtime u123))

(define-read-only (cost_mod (n uint))
    (runtime u126))

(define-read-only (cost_pow (n uint))
    (runtime u127))

(define-read-only (cost_sqrti (n uint))
    (runtime u125))

(define-read-only (cost_log2 (n uint))
    (runtime u120))

(define-read-only (cost_xor (n uint))
    (runtime u125))

(define-read-only (cost_not (n uint))
    (runtime u121))

(define-read-only (cost_eq (n uint))
    (runtime (linear n u5 u129)))

(define-read-only (cost_begin (n uint))
    (runtime u151))

(define-read-only (cost_secp256k1recover (n uint))
    (runtime u10758))

(define-read-only (cost_secp256k1verify (n uint))
    (runtime u10155))

(define-read-only (cost_some_cons (n uint))
    (runtime u164))

(define-read-only (cost_ok_cons (n uint))
    (runtime u164))

(define-read-only (cost_err_cons (n uint))
    (runtime u164))

(define-read-only (cost_default_to (n uint))
    (runtime u187))

(define-read-only (cost_unwrap_ret (n uint))
    (runtime u224))

(define-read-only (cost_unwrap_err_or_ret (n uint))
    (runtime u254))

(define-read-only (cost_is_okay (n uint))
    (runtime u215))

(define-read-only (cost_is_none (n uint))
    (runtime u159))

(define-read-only (cost_is_err (n uint))
    (runtime u211))

(define-read-only (cost_is_some (n uint))
    (runtime u167))

(define-read-only (cost_unwrap (n uint))
    (runtime u213))

(define-read-only (cost_unwrap_err (n uint))
    (runtime u198))

(define-read-only (cost_try_ret (n uint))
    (runtime u192))

(define-read-only (cost_match (n uint))
    (runtime u214))

(define-read-only (cost_or (n uint))
    (runtime (linear n u2 u111)))

(define-read-only (cost_and (n uint))
    (runtime (linear n u2 u111)))

(define-read-only (cost_append (n uint))
    (runtime (linear n u53 u132)))

(define-read-only (cost_concat (n uint))
    (runtime (linear n u56 u183)))

(define-read-only (cost_as_max_len (n uint))
    (runtime u356))

(define-read-only (cost_contract_call (n uint))
    (runtime u115))

(define-read-only (cost_contract_of (n uint))
    (runtime u10050))

(define-read-only (cost_principal_of (n uint))
    (runtime u29))


(define-read-only (cost_at_block (n uint))
    {
        runtime: u157,
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_load_contract (n uint))
    {
        runtime: (linear n u1 u118),
        write_length: u0,
        write_count: u0,
        ;; set to 3 because of the associated metadata loads
        read_count: u3,
        read_length: (linear n u1 u1)
    })


(define-read-only (cost_create_map (n uint))
    {
        runtime: (linear n u1 u1223),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u0,
        read_length: u0
    })


(define-read-only (cost_create_var (n uint))
    {
        runtime: (linear n u5 u1614),
        write_length: (linear n u1 u1),
        write_count: u2,
        read_count: u0,
        read_length: u0
    })


(define-read-only (cost_create_nft (n uint))
    {
        runtime: (linear n u1 u1207),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u0,
        read_length: u0
    })


(define-read-only (cost_create_ft (n uint))
    {
        runtime: u1479,
        write_length: u1,
        write_count: u2,
        read_count: u0,
        read_length: u0
    })


(define-read-only (cost_fetch_entry (n uint))
    {
        runtime: (linear n u1 u1154),
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: (linear n u1 u1)
    })


(define-read-only (cost_set_entry (n uint))
    {
        runtime: (linear n u3 u1653),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u1,
        read_length: u0
    })


(define-read-only (cost_fetch_var (n uint))
    {
        runtime: (linear n u1 u407),
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: (linear n u1 u1)
    })


(define-read-only (cost_set_var (n uint))
    {
        runtime: (linear n u3 u518),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u1,
        read_length: u0
    })


(define-read-only (cost_contract_storage (n uint))
    {
        runtime: (linear n u10 u5987),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u0,
        read_length: u0
    })


(define-read-only (cost_block_info (n uint))
    {
        runtime: u4741,
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_stx_balance (n uint))
    {
        runtime: u1039,
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_stx_transfer (n uint))
    {
        runtime: u1072,
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_ft_mint (n uint))
    {
        runtime: u1234,
        write_length: u1,
        write_count: u2,
        read_count: u2,
        read_length: u1
    })


(define-read-only (cost_ft_transfer (n uint))
    {
        runtime: u459,
        write_length: u1,
        write_count: u2,
        read_count: u2,
        read_length: u1
    })


(define-read-only (cost_ft_balance (n uint))
    {
        runtime: u410,
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_nft_mint (n uint))
    {
        runtime: (linear n u7 u477),
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_nft_transfer (n uint))
    {
        runtime: (linear n u6 u485),
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_nft_owner (n uint))
    {
        runtime: (linear n u7 u596),
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_ft_get_supply (n uint))
    {
        runtime: u362,
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_ft_burn (n uint))
    {
        runtime: u459,
        write_length: u1,
        write_count: u2,
        read_count: u2,
        read_length: u1
    })


(define-read-only (cost_nft_burn (n uint))
    {
        runtime: (linear n u6 u485),
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    })


(define-read-only (poison_microblock (n uint))
    {
        runtime: u22176,
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    })

