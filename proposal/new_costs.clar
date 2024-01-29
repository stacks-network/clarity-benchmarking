
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
    (runtime (linear n u1 u9)))

(define-read-only (cost_analysis_type_check (n uint))
    (runtime (linear n u115 u0)))

(define-read-only (cost_analysis_type_lookup (n uint))
    (runtime (linear n u1 u5)))

(define-read-only (cost_analysis_visit (n uint))
    (runtime u1))

(define-read-only (cost_analysis_iterable_func (n uint))
    (runtime (linear n u2 u14)))

(define-read-only (cost_analysis_option_cons (n uint))
    (runtime u5))

(define-read-only (cost_analysis_option_check (n uint))
    (runtime u3))

(define-read-only (cost_analysis_bind_name (n uint))
    (runtime (linear n u1 u54)))

(define-read-only (cost_analysis_list_items_check (n uint))
    (runtime (linear n u2 u4)))

(define-read-only (cost_analysis_check_tuple_get (n uint))
    (runtime (logn n u1 u2)))

(define-read-only (cost_analysis_check_tuple_merge (n uint))
    (runtime (nlogn n u38 u24)))

(define-read-only (cost_analysis_check_tuple_cons (n uint))
    (runtime (nlogn n u2 u98)))

(define-read-only (cost_analysis_tuple_items_check (n uint))
    (runtime (linear n u2 u57)))

(define-read-only (cost_analysis_check_let (n uint))
    (runtime (linear n u1 u10)))

(define-read-only (cost_analysis_lookup_function (n uint))
    (runtime u16))

(define-read-only (cost_analysis_lookup_function_types (n uint))
    (runtime (linear n u1 u27)))

(define-read-only (cost_analysis_lookup_variable_const (n uint))
    (runtime u16))

(define-read-only (cost_analysis_lookup_variable_depth (n uint))
    (runtime (nlogn n u1 u6)))

(define-read-only (cost_ast_parse (n uint))
    (runtime (linear n u25 u83)))

(define-read-only (cost_analysis_storage (n uint))
    {
        runtime: (linear n u2 u120),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_analysis_get_function_entry (n uint))
    {
        runtime: (linear n u76 u1254),
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: (linear n u1 u1)
    })

(define-read-only (cost_analysis_fetch_contract_entry (n uint))
    (runtime (linear n u1 u642)))

(define-read-only (cost_lookup_variable_depth (n uint))
    (runtime (linear n u1 u6)))

(define-read-only (cost_lookup_variable_size (n uint))
    (runtime (linear n u2 u6)))

(define-read-only (cost_lookup_function (n uint))
    (runtime u17))

(define-read-only (cost_bind_name (n uint))
    (runtime u211))

(define-read-only (cost_inner_type_check_cost (n uint))
    (runtime (linear n u3 u6)))

(define-read-only (cost_user_function_application (n uint))
    (runtime (linear n u30 u0)))

(define-read-only (cost_let (n uint))
    (runtime (linear n u143 u108)))

(define-read-only (cost_if (n uint))
    (runtime u157))

(define-read-only (cost_asserts (n uint))
    (runtime u117))

(define-read-only (cost_map (n uint))
    (runtime (linear n u1388 u3017)))

(define-read-only (cost_filter (n uint))
    (runtime u420))

(define-read-only (cost_len (n uint))
    (runtime u505))

(define-read-only (cost_element_at (n uint))
    (runtime u602))

(define-read-only (cost_index_of (n uint))
    (runtime (linear n u1 u197)))

(define-read-only (cost_fold (n uint))
    (runtime u468))

(define-read-only (cost_list_cons (n uint))
    (runtime (linear n u18 u142)))

(define-read-only (cost_type_parse_step (n uint))
    (runtime u4))

(define-read-only (cost_tuple_get (n uint))
    (runtime (nlogn n u4 u1928)))

(define-read-only (cost_tuple_merge (n uint))
    (runtime (linear n u5 u1554)))

(define-read-only (cost_tuple_cons (n uint))
    (runtime (nlogn n u11 u1486)))

(define-read-only (cost_add (n uint))
    (runtime (linear n u14 u110)))

(define-read-only (cost_sub (n uint))
    (runtime (linear n u14 u110)))

(define-read-only (cost_mul (n uint))
    (runtime (linear n u16 u107)))

(define-read-only (cost_div (n uint))
    (runtime (linear n u16 u107)))

(define-read-only (cost_geq (n uint))
    (runtime (linear n u5 u19)))

(define-read-only (cost_leq (n uint))
    (runtime (linear n u5 u19)))

(define-read-only (cost_le (n uint))
    (runtime (linear n u5 u19)))

(define-read-only (cost_ge (n uint))
    (runtime (linear n u5 u19)))

(define-read-only (cost_int_cast (n uint))
    (runtime u125))

(define-read-only (cost_mod (n uint))
    (runtime u131))

(define-read-only (cost_pow (n uint))
    (runtime u132))

(define-read-only (cost_sqrti (n uint))
    (runtime u128))

(define-read-only (cost_log2 (n uint))
    (runtime u119))

(define-read-only (cost_xor (n uint))
    (runtime (linear n u13 u119)))

(define-read-only (cost_not (n uint))
    (runtime u124))

(define-read-only (cost_eq (n uint))
    (runtime (linear n u10 u134)))

(define-read-only (cost_begin (n uint))
    (runtime u170))

(define-read-only (cost_hash160 (n uint))
    (runtime (linear n u1 u209)))

(define-read-only (cost_sha256 (n uint))
    (runtime (linear n u1 u84)))

(define-read-only (cost_sha512 (n uint))
    (runtime (linear n u1 u44)))

(define-read-only (cost_sha512t256 (n uint))
    (runtime (linear n u1 u48)))

(define-read-only (cost_keccak256 (n uint))
    (runtime (linear n u1 u146)))

(define-read-only (cost_secp256k1recover (n uint))
    (runtime u7580))

(define-read-only (cost_secp256k1verify (n uint))
    (runtime u7335))

(define-read-only (cost_some_cons (n uint))
    (runtime u176))

(define-read-only (cost_ok_cons (n uint))
    (runtime u176))

(define-read-only (cost_err_cons (n uint))
    (runtime u176))

(define-read-only (cost_default_to (n uint))
    (runtime u235))

(define-read-only (cost_unwrap_ret (n uint))
    (runtime u286))

(define-read-only (cost_unwrap_err_or_ret (n uint))
    (runtime u291))

(define-read-only (cost_is_okay (n uint))
    (runtime u263))

(define-read-only (cost_is_none (n uint))
    (runtime u178))

(define-read-only (cost_is_err (n uint))
    (runtime u250))

(define-read-only (cost_is_some (n uint))
    (runtime u181))

(define-read-only (cost_unwrap (n uint))
    (runtime u242))

(define-read-only (cost_unwrap_err (n uint))
    (runtime u229))

(define-read-only (cost_try_ret (n uint))
    (runtime u250))

(define-read-only (cost_match (n uint))
    (runtime u283))

(define-read-only (cost_or (n uint))
    (runtime (linear n u6 u107)))

(define-read-only (cost_and (n uint))
    (runtime (linear n u6 u107)))

(define-read-only (cost_append (n uint))
    (runtime (linear n u75 u125)))

(define-read-only (cost_concat (n uint))
    (runtime (linear n u16 u472)))

(define-read-only (cost_as_max_len (n uint))
    (runtime u474))

(define-read-only (cost_contract_call (n uint))
    (runtime u125))

(define-read-only (cost_contract_of (n uint))
    (runtime u20006))

(define-read-only (cost_principal_of (n uint))
    (runtime u862))


(define-read-only (cost_at_block (n uint))
    {
        runtime: u1266,
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_load_contract (n uint))
    {
        runtime: (linear n u1 u9),
        write_length: u0,
        write_count: u0,
        ;; set to 3 because of the associated metadata loads
        read_count: u3,
        read_length: (linear n u1 u1)
    })


(define-read-only (cost_create_map (n uint))
    {
        runtime: (linear n u1 u2071),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u0,
        read_length: u0
    })


(define-read-only (cost_create_var (n uint))
    {
        runtime: (linear n u8 u3123),
        write_length: (linear n u1 u1),
        write_count: u2,
        read_count: u0,
        read_length: u0
    })


(define-read-only (cost_create_nft (n uint))
    {
        runtime: (linear n u1 u2038),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u0,
        read_length: u0
    })


(define-read-only (cost_create_ft (n uint))
    {
        runtime: u2299,
        write_length: u1,
        write_count: u2,
        read_count: u0,
        read_length: u0
    })


(define-read-only (cost_fetch_entry (n uint))
    {
        runtime: (linear n u1 u2776),
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: (linear n u1 u1)
    })


(define-read-only (cost_set_entry (n uint))
    {
        runtime: (linear n u4 u3194),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u1,
        read_length: u0
    })


(define-read-only (cost_fetch_var (n uint))
    {
        runtime: (linear n u3 u54097),
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: (linear n u1 u1)
    })


(define-read-only (cost_set_var (n uint))
    {
        runtime: (linear n u5 u1347),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u1,
        read_length: u0
    })


(define-read-only (cost_contract_storage (n uint))
    {
        runtime: (linear n u10 u7380),
        write_length: (linear n u1 u1),
        write_count: u1,
        read_count: u0,
        read_length: u0
    })


(define-read-only (cost_block_info (n uint))
    {
        runtime: u3828,
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_stx_balance (n uint))
    {
        runtime: u5527,
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_stx_transfer (n uint))
    {
        runtime: u8607,
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_ft_mint (n uint))
    {
        runtime: u1346,
        write_length: u1,
        write_count: u2,
        read_count: u2,
        read_length: u1
    })


(define-read-only (cost_ft_transfer (n uint))
    {
        runtime: u485,
        write_length: u1,
        write_count: u2,
        read_count: u2,
        read_length: u1
    })


(define-read-only (cost_ft_balance (n uint))
    {
        runtime: u435,
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_nft_mint (n uint))
    {
        runtime: (linear n u10 u1936),
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_nft_transfer (n uint))
    {
        runtime: (linear n u10 u1970),
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_nft_owner (n uint))
    {
        runtime: (linear n u11 u2028),
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_ft_get_supply (n uint))
    {
        runtime: u377,
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_ft_burn (n uint))
    {
        runtime: u485,
        write_length: u1,
        write_count: u2,
        read_count: u2,
        read_length: u1
    })


(define-read-only (cost_nft_burn (n uint))
    {
        runtime: (linear n u10 u1970),
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    })


(define-read-only (poison_microblock (n uint))
    {
        runtime: u15441,
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    })

(define-read-only (cost_buff_to_int_le (n uint))
    (runtime u128))

(define-read-only (cost_buff_to_uint_le (n uint))
    (runtime u128))

(define-read-only (cost_buff_to_int_be (n uint))
    (runtime u128))

(define-read-only (cost_buff_to_uint_be (n uint))
    (runtime u128))

(define-read-only (cost_is_standard (n uint))
    (runtime u113))

(define-read-only (cost_principal_destruct (n uint))
    (runtime u323))

(define-read-only (cost_principal_construct (n uint))
    (runtime u369))

(define-read-only (cost_string_to_int (n uint))
    (runtime u160))

(define-read-only (cost_string_to_uint (n uint))
    (runtime u160))

(define-read-only (cost_int_to_ascii (n uint))
    (runtime u137))

(define-read-only (cost_int_to_utf8 (n uint))
    (runtime u173))


(define-read-only (cost_burn_block_info (n uint))
    {
        runtime: u106694,
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    })


(define-read-only (cost_stx_account (n uint))
    {
        runtime: u7138,
        write_length: u0,
        write_count: u0,
        read_count: u1,
        read_length: u1
    })

(define-read-only (cost_slice (n uint))
    (runtime u530))

(define-read-only (cost_to_consensus_buff (n uint))
    (runtime (linear n u1 u226)))

(define-read-only (cost_from_consensus_buff (n uint))
    (runtime (nlogn n u4 u207)))


(define-read-only (cost_stx_transfer_memo (n uint))
    {
        runtime: u8688,
        write_length: u1,
        write_count: u1,
        read_count: u1,
        read_length: u1
    })

(define-read-only (cost_replace_at (n uint))
    (runtime (linear n u1 u454)))

(define-read-only (cost_as_contract (n uint))
    (runtime u124))

(define-read-only (cost_bitwise_and (n uint))
    (runtime (linear n u13 u115)))

(define-read-only (cost_bitwise_or (n uint))
    (runtime (linear n u13 u115)))

(define-read-only (cost_bitwise_not (n uint))
    (runtime u126))

(define-read-only (cost_bitwise_left_shift (n uint))
    (runtime u139))

(define-read-only (cost_bitwise_right_shift (n uint))
    (runtime u140))

