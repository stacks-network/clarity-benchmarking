| Runtime cost | New function | Old function |
| ----------- | ----------- | ----------- |
| cost_analysis_type_annotate | f(x) := 1*x + 10 | f(x) := 1000*x + 1000 |
| cost_analysis_type_check | f(x) := 115*x + 0 | f(x) := 1000*x + 1000 |
| cost_analysis_type_lookup | f(x) := 1*x + 4 | f(x) := 1000*x + 1000 |
| cost_analysis_visit | f(x) := 1 | f(x) := 1000 |
| cost_analysis_iterable_func | f(x) := 2*x + 14 | f(x) := 1000*x + 1000 |
| cost_analysis_option_cons | f(x) := 5 | f(x) := 1000 |
| cost_analysis_option_check | f(x) := 4 | f(x) := 1000 |
| cost_analysis_bind_name | f(x) := 1*x + 59 | f(x) := 1000*x + 1000 |
| cost_analysis_list_items_check | f(x) := 2*x + 8 | f(x) := 1000*x + 1000 |
| cost_analysis_check_tuple_get | f(x) := 1*log(x) + 2 | f(x) := 1000*log(x) + 1000 |
| cost_analysis_check_tuple_cons | f(x) := 3*x*log(x) + 142 | f(x) := 1000*x*log(x) + 1000 |
| cost_analysis_tuple_items_check | f(x) := 2*x + 52 | f(x) := 1000*x + 1000 |
| cost_analysis_check_let | f(x) := 1*x + 10 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_function | f(x) := 18 | f(x) := 1000 |
| cost_analysis_lookup_function_types | f(x) := 1*x + 26 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_variable_const | f(x) := 16 | f(x) := 1000 |
| cost_analysis_lookup_variable_depth | f(x) := 1*x*log(x) + 12 | f(x) := 1000*x*log(x) + 1000 |
| cost_ast_parse | f(x) := 27*x + 81 | f(x) := 1000*x + 1000 |
| cost_ast_cycle_detection | f(x) := 156*x + 23 | f(x) := 1000*x + 1000 |
| cost_analysis_storage | f(x) := 2*x + 94 | f(x) := 1000*x + 1000 |
| cost_analysis_use_trait_entry | f(x) := 9*x + 698 | f(x) := 1000*x + 1000 |
| cost_analysis_get_function_entry | f(x) := 78*x + 1307 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_depth | f(x) := 1*x + 1 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_size | f(x) := 2*x + 18 | f(x) := 1000*x + 1000 |
| cost_lookup_function | f(x) := 16 | f(x) := 1000 |
| cost_bind_name | f(x) := 216 | f(x) := 1000 |
| cost_inner_type_check_cost | f(x) := 2*x + 5 | f(x) := 1000*x + 1000 |
| cost_user_function_application | f(x) := 27*x + 5 | f(x) := 1000*x + 1000 |
| cost_let | f(x) := 117*x + 178 | f(x) := 1000*x + 1000 |
| cost_if | f(x) := 168 | f(x) := 1000 |
| cost_asserts | f(x) := 128 | f(x) := 1000 |
| cost_map | f(x) := 1198*x + 3067 | f(x) := 1000*x + 1000 |
| cost_filter | f(x) := 407 | f(x) := 1000 |
| cost_len | f(x) := 429 | f(x) := 1000 |
| cost_element_at | f(x) := 498 | f(x) := 1000 |
| cost_index_of | f(x) := 1*x + 211 | f(x) := 1000*x + 1000 |
| cost_fold | f(x) := 460 | f(x) := 1000 |
| cost_list_cons | f(x) := 14*x + 164 | f(x) := 1000*x + 1000 |
| cost_type_parse_step | f(x) := 4 | f(x) := 1000 |
| cost_tuple_get | f(x) := 4*x*log(x) + 1736 | f(x) := 1000*x*log(x) + 1000 |
| cost_tuple_merge | f(x) := 5*x + 408 | f(x) := 1000*x + 1000 |
| cost_tuple_cons | f(x) := 10*x*log(x) + 1876 | f(x) := 1000*x*log(x) + 1000 |
| cost_add | f(x) := 11*x + 125 | f(x) := 1000*x + 1000 |
| cost_sub | f(x) := 11*x + 125 | f(x) := 1000*x + 1000 |
| cost_mul | f(x) := 13*x + 125 | f(x) := 1000*x + 1000 |
| cost_div | f(x) := 13*x + 125 | f(x) := 1000*x + 1000 |
| cost_geq | f(x) := 126 | f(x) := 1000 |
| cost_leq | f(x) := 126 | f(x) := 1000 |
| cost_le | f(x) := 126 | f(x) := 1000 |
| cost_ge | f(x) := 126 | f(x) := 1000 |
| cost_int_cast | f(x) := 135 | f(x) := 1000 |
| cost_mod | f(x) := 141 | f(x) := 1000 |
| cost_pow | f(x) := 143 | f(x) := 1000 |
| cost_sqrti | f(x) := 142 | f(x) := 1000 |
| cost_log2 | f(x) := 133 | f(x) := 1000 |
| cost_xor | f(x) := 139 | f(x) := 1000 |
| cost_not | f(x) := 138 | f(x) := 1000 |
| cost_eq | f(x) := 8*x + 151 | f(x) := 1000*x + 1000 |
| cost_begin | f(x) := 151 | f(x) := 1000 |
| cost_hash160 | f(x) := 1*x + 188 | f(x) := 1000*x + 1000 |
| cost_sha256 | f(x) := 1*x + 180 | f(x) := 1000*x + 1000 |
| cost_sha512 | f(x) := 1*x + 216 | f(x) := 1000*x + 1000 |
| cost_sha512t256 | f(x) := 1*x + 56 | f(x) := 1000*x + 1000 |
| cost_keccak256 | f(x) := 1*x + 127 | f(x) := 1000*x + 1000 |
| cost_secp256k1recover | f(x) := 8655 | f(x) := 1000 |
| cost_secp256k1verify | f(x) := 8349 | f(x) := 1000 |
| cost_print | f(x) := 15*x + 1458 | f(x) := 1000*x + 1000 |
| cost_some_cons | f(x) := 199 | f(x) := 1000 |
| cost_ok_cons | f(x) := 199 | f(x) := 1000 |
| cost_err_cons | f(x) := 199 | f(x) := 1000 |
| cost_default_to | f(x) := 268 | f(x) := 1000 |
| cost_unwrap_ret | f(x) := 274 | f(x) := 1000 |
| cost_unwrap_err_or_ret | f(x) := 302 | f(x) := 1000 |
| cost_is_okay | f(x) := 258 | f(x) := 1000 |
| cost_is_none | f(x) := 214 | f(x) := 1000 |
| cost_is_err | f(x) := 245 | f(x) := 1000 |
| cost_is_some | f(x) := 195 | f(x) := 1000 |
| cost_unwrap | f(x) := 252 | f(x) := 1000 |
| cost_unwrap_err | f(x) := 248 | f(x) := 1000 |
| cost_try_ret | f(x) := 240 | f(x) := 1000 |
| cost_match | f(x) := 264 | f(x) := 1000 |
| cost_or | f(x) := 3*x + 120 | f(x) := 1000*x + 1000 |
| cost_and | f(x) := 3*x + 120 | f(x) := 1000*x + 1000 |
| cost_append | f(x) := 73*x + 285 | f(x) := 1000*x + 1000 |
| cost_concat | f(x) := 37*x + 220 | f(x) := 1000*x + 1000 |
| cost_as_max_len | f(x) := 518 | f(x) := 1000 |
| cost_contract_call | f(x) := 134 | f(x) := 1000 |
| cost_contract_of | f(x) := 19002 | f(x) := 1000 |
| cost_principal_of | f(x) := 984 | f(x) := 1000 |
| cost_at_block | f(x) := 2821 | f(x) := 1000 |
| cost_load_contract | f(x) := 1*x + 80 | f(x) := 1000*x + 1000 |
| cost_create_map | f(x) := 1*x + 1564 | f(x) := 1000*x + 1000 |
| cost_create_var | f(x) := 7*x + 2025 | f(x) := 1000*x + 1000 |
| cost_create_nft | f(x) := 1*x + 1570 | f(x) := 1000*x + 1000 |
| cost_create_ft | f(x) := 1831 | f(x) := 1000 |
| cost_fetch_entry | f(x) := 1*x + 1025 | f(x) := 1000*x + 1000 |
| cost_set_entry | f(x) := 4*x + 1899 | f(x) := 1000*x + 1000 |
| cost_fetch_var | f(x) := 1*x + 468 | f(x) := 1000*x + 1000 |
| cost_set_var | f(x) := 5*x + 655 | f(x) := 1000*x + 1000 |
| cost_contract_storage | f(x) := 11*x + 7165 | f(x) := 1000*x + 1000 |
| cost_block_info | f(x) := 7841 | f(x) := 1000 |
| cost_stx_balance | f(x) := 9626 | f(x) := 1000 |
| cost_stx_transfer | f(x) := 9983 | f(x) := 1000 |
| cost_ft_mint | f(x) := 1479 | f(x) := 1000 |
| cost_ft_transfer | f(x) := 549 | f(x) := 1000 |
| cost_ft_balance | f(x) := 479 | f(x) := 1000 |
| cost_nft_mint | f(x) := 9*x + 575 | f(x) := 1000*x + 1000 |
| cost_nft_transfer | f(x) := 9*x + 572 | f(x) := 1000*x + 1000 |
| cost_nft_owner | f(x) := 10*x + 741 | f(x) := 1000*x + 1000 |
| cost_ft_get_supply | f(x) := 420 | f(x) := 1000 |
| cost_ft_burn | f(x) := 549 | f(x) := 1000 |
| cost_nft_burn | f(x) := 9*x + 572 | f(x) := 1000*x + 1000 |
| poison_microblock | f(x) := 17485 | f(x) := 1000 |
| cost_buff_to_int_le | f(x) := 141 | f(x) := 1000 |
| cost_buff_to_uint_le | f(x) := 141 | f(x) := 1000 |
| cost_buff_to_int_be | f(x) := 141 | f(x) := 1000 |
| cost_buff_to_uint_be | f(x) := 141 | f(x) := 1000 |
| cost_is_standard | f(x) := 127 | f(x) := 1000 |
| cost_principal_destruct | f(x) := 314 | f(x) := 1000 |
| cost_principal_construct | f(x) := 398 | f(x) := 1000 |
| cost_string_to_int | f(x) := 168 | f(x) := 1000 |
| cost_string_to_uint | f(x) := 168 | f(x) := 1000 |
| cost_int_to_ascii | f(x) := 147 | f(x) := 1000 |
| cost_int_to_utf8 | f(x) := 181 | f(x) := 1000 |
| cost_burn_block_info | f(x) := 105877 | f(x) := 1000 |
| cost_stx_account | f(x) := 10028 | f(x) := 1000 |
| cost_slice | f(x) := 523 | f(x) := 1000 |
| cost_to_consensus_buff | f(x) := 1*x + 233 | f(x) := 1000*x + 1000 |
| cost_from_consensus_buff | f(x) := 3*x*log(x) + 185 | f(x) := 1000*x*log(x) + 1000 |
| cost_stx_transfer_memo | f(x) := 11281 | f(x) := 1000 |
