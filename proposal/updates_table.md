| Runtime cost | New function | Old function |
| ----------- | ----------- | ----------- |
| cost_analysis_type_annotate | f(x) := 1*x + 9 | f(x) := 1000*x + 1000 |
| cost_analysis_type_check | f(x) := 115*x + 0 | f(x) := 1000*x + 1000 |
| cost_analysis_type_lookup | f(x) := 1*x + 5 | f(x) := 1000*x + 1000 |
| cost_analysis_visit | f(x) := 1 | f(x) := 1000 |
| cost_analysis_iterable_func | f(x) := 2*x + 14 | f(x) := 1000*x + 1000 |
| cost_analysis_option_cons | f(x) := 5 | f(x) := 1000 |
| cost_analysis_option_check | f(x) := 3 | f(x) := 1000 |
| cost_analysis_bind_name | f(x) := 1*x + 54 | f(x) := 1000*x + 1000 |
| cost_analysis_list_items_check | f(x) := 2*x + 4 | f(x) := 1000*x + 1000 |
| cost_analysis_check_tuple_get | f(x) := 1*log(x) + 2 | f(x) := 1000*log(x) + 1000 |
| cost_analysis_check_tuple_merge | f(x) := 38*x*log(x) + 24 | f(x) := 1000*x*log(x) + 1000 |
| cost_analysis_check_tuple_cons | f(x) := 2*x*log(x) + 98 | f(x) := 1000*x*log(x) + 1000 |
| cost_analysis_tuple_items_check | f(x) := 2*x + 57 | f(x) := 1000*x + 1000 |
| cost_analysis_check_let | f(x) := 1*x + 10 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_function | f(x) := 16 | f(x) := 1000 |
| cost_analysis_lookup_function_types | f(x) := 1*x + 27 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_variable_const | f(x) := 16 | f(x) := 1000 |
| cost_analysis_lookup_variable_depth | f(x) := 1*x*log(x) + 6 | f(x) := 1000*x*log(x) + 1000 |
| cost_ast_parse | f(x) := 25*x + 83 | f(x) := 1000*x + 1000 |
| cost_analysis_storage | f(x) := 2*x + 120 | f(x) := 1000*x + 1000 |
| cost_analysis_get_function_entry | f(x) := 76*x + 1254 | f(x) := 1000*x + 1000 |
| cost_analysis_fetch_contract_entry | f(x) := 1*x + 642 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_depth | f(x) := 1*x + 6 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_size | f(x) := 2*x + 6 | f(x) := 1000*x + 1000 |
| cost_lookup_function | f(x) := 17 | f(x) := 1000 |
| cost_bind_name | f(x) := 211 | f(x) := 1000 |
| cost_inner_type_check_cost | f(x) := 3*x + 6 | f(x) := 1000*x + 1000 |
| cost_user_function_application | f(x) := 30*x + 0 | f(x) := 1000*x + 1000 |
| cost_let | f(x) := 143*x + 108 | f(x) := 1000*x + 1000 |
| cost_if | f(x) := 157 | f(x) := 1000 |
| cost_asserts | f(x) := 117 | f(x) := 1000 |
| cost_map | f(x) := 1388*x + 3017 | f(x) := 1000*x + 1000 |
| cost_filter | f(x) := 420 | f(x) := 1000 |
| cost_len | f(x) := 505 | f(x) := 1000 |
| cost_element_at | f(x) := 602 | f(x) := 1000 |
| cost_index_of | f(x) := 1*x + 197 | f(x) := 1000*x + 1000 |
| cost_fold | f(x) := 468 | f(x) := 1000 |
| cost_list_cons | f(x) := 18*x + 142 | f(x) := 1000*x + 1000 |
| cost_type_parse_step | f(x) := 4 | f(x) := 1000 |
| cost_tuple_get | f(x) := 4*x*log(x) + 1928 | f(x) := 1000*x*log(x) + 1000 |
| cost_tuple_merge | f(x) := 5*x + 1554 | f(x) := 1000*x + 1000 |
| cost_tuple_cons | f(x) := 11*x*log(x) + 1486 | f(x) := 1000*x*log(x) + 1000 |
| cost_add | f(x) := 14*x + 110 | f(x) := 1000*x + 1000 |
| cost_sub | f(x) := 14*x + 110 | f(x) := 1000*x + 1000 |
| cost_mul | f(x) := 16*x + 107 | f(x) := 1000*x + 1000 |
| cost_div | f(x) := 16*x + 107 | f(x) := 1000*x + 1000 |
| cost_geq | f(x) := 5*x + 19 | f(x) := 1000*x + 1000 |
| cost_leq | f(x) := 5*x + 19 | f(x) := 1000*x + 1000 |
| cost_le | f(x) := 5*x + 19 | f(x) := 1000*x + 1000 |
| cost_ge | f(x) := 5*x + 19 | f(x) := 1000*x + 1000 |
| cost_int_cast | f(x) := 125 | f(x) := 1000 |
| cost_mod | f(x) := 131 | f(x) := 1000 |
| cost_pow | f(x) := 132 | f(x) := 1000 |
| cost_sqrti | f(x) := 128 | f(x) := 1000 |
| cost_log2 | f(x) := 119 | f(x) := 1000 |
| cost_xor | f(x) := 13*x + 119 | f(x) := 1000*x + 1000 |
| cost_not | f(x) := 124 | f(x) := 1000 |
| cost_eq | f(x) := 10*x + 134 | f(x) := 1000*x + 1000 |
| cost_begin | f(x) := 170 | f(x) := 1000 |
| cost_hash160 | f(x) := 1*x + 209 | f(x) := 1000*x + 1000 |
| cost_sha256 | f(x) := 1*x + 84 | f(x) := 1000*x + 1000 |
| cost_sha512 | f(x) := 1*x + 44 | f(x) := 1000*x + 1000 |
| cost_sha512t256 | f(x) := 1*x + 48 | f(x) := 1000*x + 1000 |
| cost_keccak256 | f(x) := 1*x + 146 | f(x) := 1000*x + 1000 |
| cost_secp256k1recover | f(x) := 7580 | f(x) := 1000 |
| cost_secp256k1verify | f(x) := 7335 | f(x) := 1000 |
| cost_some_cons | f(x) := 176 | f(x) := 1000 |
| cost_ok_cons | f(x) := 176 | f(x) := 1000 |
| cost_err_cons | f(x) := 176 | f(x) := 1000 |
| cost_default_to | f(x) := 235 | f(x) := 1000 |
| cost_unwrap_ret | f(x) := 286 | f(x) := 1000 |
| cost_unwrap_err_or_ret | f(x) := 291 | f(x) := 1000 |
| cost_is_okay | f(x) := 263 | f(x) := 1000 |
| cost_is_none | f(x) := 178 | f(x) := 1000 |
| cost_is_err | f(x) := 250 | f(x) := 1000 |
| cost_is_some | f(x) := 181 | f(x) := 1000 |
| cost_unwrap | f(x) := 242 | f(x) := 1000 |
| cost_unwrap_err | f(x) := 229 | f(x) := 1000 |
| cost_try_ret | f(x) := 250 | f(x) := 1000 |
| cost_match | f(x) := 283 | f(x) := 1000 |
| cost_or | f(x) := 6*x + 107 | f(x) := 1000*x + 1000 |
| cost_and | f(x) := 6*x + 107 | f(x) := 1000*x + 1000 |
| cost_append | f(x) := 75*x + 125 | f(x) := 1000*x + 1000 |
| cost_concat | f(x) := 16*x + 472 | f(x) := 1000*x + 1000 |
| cost_as_max_len | f(x) := 474 | f(x) := 1000 |
| cost_contract_call | f(x) := 125 | f(x) := 1000 |
| cost_contract_of | f(x) := 20006 | f(x) := 1000 |
| cost_principal_of | f(x) := 862 | f(x) := 1000 |
| cost_at_block | f(x) := 1266 | f(x) := 1000 |
| cost_load_contract | f(x) := 1*x + 9 | f(x) := 1000*x + 1000 |
| cost_create_map | f(x) := 1*x + 2071 | f(x) := 1000*x + 1000 |
| cost_create_var | f(x) := 8*x + 3123 | f(x) := 1000*x + 1000 |
| cost_create_nft | f(x) := 1*x + 2038 | f(x) := 1000*x + 1000 |
| cost_create_ft | f(x) := 2299 | f(x) := 1000 |
| cost_fetch_entry | f(x) := 1*x + 2776 | f(x) := 1000*x + 1000 |
| cost_set_entry | f(x) := 4*x + 3194 | f(x) := 1000*x + 1000 |
| cost_fetch_var | f(x) := 3*x + 54097 | f(x) := 1000*x + 1000 |
| cost_set_var | f(x) := 5*x + 1347 | f(x) := 1000*x + 1000 |
| cost_contract_storage | f(x) := 10*x + 7380 | f(x) := 1000*x + 1000 |
| cost_block_info | f(x) := 3828 | f(x) := 1000 |
| cost_stx_balance | f(x) := 5527 | f(x) := 1000 |
| cost_stx_transfer | f(x) := 8607 | f(x) := 1000 |
| cost_ft_mint | f(x) := 1346 | f(x) := 1000 |
| cost_ft_transfer | f(x) := 485 | f(x) := 1000 |
| cost_ft_balance | f(x) := 435 | f(x) := 1000 |
| cost_nft_mint | f(x) := 10*x + 1936 | f(x) := 1000*x + 1000 |
| cost_nft_transfer | f(x) := 10*x + 1970 | f(x) := 1000*x + 1000 |
| cost_nft_owner | f(x) := 11*x + 2028 | f(x) := 1000*x + 1000 |
| cost_ft_get_supply | f(x) := 377 | f(x) := 1000 |
| cost_ft_burn | f(x) := 485 | f(x) := 1000 |
| cost_nft_burn | f(x) := 10*x + 1970 | f(x) := 1000*x + 1000 |
| poison_microblock | f(x) := 15441 | f(x) := 1000 |
| cost_buff_to_int_le | f(x) := 128 | f(x) := 1000 |
| cost_buff_to_uint_le | f(x) := 128 | f(x) := 1000 |
| cost_buff_to_int_be | f(x) := 128 | f(x) := 1000 |
| cost_buff_to_uint_be | f(x) := 128 | f(x) := 1000 |
| cost_is_standard | f(x) := 113 | f(x) := 1000 |
| cost_principal_destruct | f(x) := 323 | f(x) := 1000 |
| cost_principal_construct | f(x) := 369 | f(x) := 1000 |
| cost_string_to_int | f(x) := 160 | f(x) := 1000 |
| cost_string_to_uint | f(x) := 160 | f(x) := 1000 |
| cost_int_to_ascii | f(x) := 137 | f(x) := 1000 |
| cost_int_to_utf8 | f(x) := 173 | f(x) := 1000 |
| cost_burn_block_info | f(x) := 106694 | f(x) := 1000 |
| cost_stx_account | f(x) := 7138 | f(x) := 1000 |
| cost_slice | f(x) := 530 | f(x) := 1000 |
| cost_to_consensus_buff | f(x) := 1*x + 226 | f(x) := 1000*x + 1000 |
| cost_from_consensus_buff | f(x) := 4*x*log(x) + 207 | f(x) := 1000*x*log(x) + 1000 |
| cost_stx_transfer_memo | f(x) := 8688 | f(x) := 1000 |
| cost_replace_at | f(x) := 1*x + 454 | f(x) := 1000*x + 1000 |
| cost_as_contract | f(x) := 124 | f(x) := 1000 |
| cost_bitwise_and | f(x) := 13*x + 115 | f(x) := 1000*x + 1000 |
| cost_bitwise_or | f(x) := 13*x + 115 | f(x) := 1000*x + 1000 |
| cost_bitwise_not | f(x) := 126 | f(x) := 1000 |
| cost_bitwise_left_shift | f(x) := 139 | f(x) := 1000 |
| cost_bitwise_right_shift | f(x) := 140 | f(x) := 1000 |
