| Runtime cost | New function | Old function |
| ----------- | ----------- | ----------- |
| cost_analysis_type_annotate | f(x) := 2*x + 10 | f(x) := 1000*x + 1000 |
| cost_analysis_type_check | f(x) := 128*x + 0 | f(x) := 1000*x + 1000 |
| cost_analysis_type_lookup | f(x) := 1*x + 5 | f(x) := 1000*x + 1000 |
| cost_analysis_visit | f(x) := 1 | f(x) := 1000 |
| cost_analysis_iterable_func | f(x) := 2*x + 16 | f(x) := 1000*x + 1000 |
| cost_analysis_option_cons | f(x) := 6 | f(x) := 1000 |
| cost_analysis_option_check | f(x) := 4 | f(x) := 1000 |
| cost_analysis_bind_name | f(x) := 2*x + 69 | f(x) := 1000*x + 1000 |
| cost_analysis_list_items_check | f(x) := 2*x + 2 | f(x) := 1000*x + 1000 |
| cost_analysis_check_tuple_get | f(x) := 1*log(x) + 2 | f(x) := 1000*log(x) + 1000 |
| cost_analysis_check_tuple_merge | f(x) := 45*x*log(x) + 49 | f(x) := 1000*x*log(x) + 1000 |
| cost_analysis_check_tuple_cons | f(x) := 3*x*log(x) + 144 | f(x) := 1000*x*log(x) + 1000 |
| cost_analysis_tuple_items_check | f(x) := 2*x + 66 | f(x) := 1000*x + 1000 |
| cost_analysis_check_let | f(x) := 2*x + 11 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_function | f(x) := 21 | f(x) := 1000 |
| cost_analysis_lookup_function_types | f(x) := 1*x + 31 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_variable_const | f(x) := 17 | f(x) := 1000 |
| cost_analysis_lookup_variable_depth | f(x) := 1*x*log(x) + 10 | f(x) := 1000*x*log(x) + 1000 |
| cost_ast_parse | f(x) := 29*x + 101 | f(x) := 1000*x + 1000 |
| cost_ast_cycle_detection | f(x) := 168*x + 175 | f(x) := 1000*x + 1000 |
| cost_analysis_storage | f(x) := 2*x + 119 | f(x) := 1000*x + 1000 |
| cost_analysis_use_trait_entry | f(x) := 10*x + 774 | f(x) := 1000*x + 1000 |
| cost_analysis_get_function_entry | f(x) := 87*x + 1372 | f(x) := 1000*x + 1000 |
| cost_analysis_fetch_contract_entry | f(x) := 1*x + 1516 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_depth | f(x) := 2*x + 6 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_size | f(x) := 2*x + 6 | f(x) := 1000*x + 1000 |
| cost_lookup_function | f(x) := 21 | f(x) := 1000 |
| cost_bind_name | f(x) := 230 | f(x) := 1000 |
| cost_inner_type_check_cost | f(x) := 3*x + 10 | f(x) := 1000*x + 1000 |
| cost_user_function_application | f(x) := 30*x + 5 | f(x) := 1000*x + 1000 |
| cost_let | f(x) := 136*x + 231 | f(x) := 1000*x + 1000 |
| cost_if | f(x) := 167 | f(x) := 1000 |
| cost_asserts | f(x) := 135 | f(x) := 1000 |
| cost_map | f(x) := 1374*x + 2883 | f(x) := 1000*x + 1000 |
| cost_filter | f(x) := 476 | f(x) := 1000 |
| cost_len | f(x) := 498 | f(x) := 1000 |
| cost_element_at | f(x) := 493 | f(x) := 1000 |
| cost_index_of | f(x) := 1*x + 231 | f(x) := 1000*x + 1000 |
| cost_fold | f(x) := 487 | f(x) := 1000 |
| cost_list_cons | f(x) := 16*x + 178 | f(x) := 1000*x + 1000 |
| cost_type_parse_step | f(x) := 5 | f(x) := 1000 |
| cost_tuple_get | f(x) := 4*x*log(x) + 2047 | f(x) := 1000*x*log(x) + 1000 |
| cost_tuple_merge | f(x) := 5*x + 620 | f(x) := 1000*x + 1000 |
| cost_tuple_cons | f(x) := 11*x*log(x) + 2040 | f(x) := 1000*x*log(x) + 1000 |
| cost_add | f(x) := 15*x + 132 | f(x) := 1000*x + 1000 |
| cost_sub | f(x) := 15*x + 132 | f(x) := 1000*x + 1000 |
| cost_mul | f(x) := 18*x + 129 | f(x) := 1000*x + 1000 |
| cost_div | f(x) := 18*x + 129 | f(x) := 1000*x + 1000 |
| cost_geq | f(x) := 7*x + 128 | f(x) := 1000*x + 1000 |
| cost_leq | f(x) := 7*x + 128 | f(x) := 1000*x + 1000 |
| cost_le | f(x) := 7*x + 128 | f(x) := 1000*x + 1000 |
| cost_ge | f(x) := 7*x + 128 | f(x) := 1000*x + 1000 |
| cost_int_cast | f(x) := 141 | f(x) := 1000 |
| cost_mod | f(x) := 155 | f(x) := 1000 |
| cost_pow | f(x) := 151 | f(x) := 1000 |
| cost_sqrti | f(x) := 147 | f(x) := 1000 |
| cost_log2 | f(x) := 138 | f(x) := 1000 |
| cost_xor | f(x) := 15*x + 132 | f(x) := 1000*x + 1000 |
| cost_not | f(x) := 148 | f(x) := 1000 |
| cost_eq | f(x) := 12*x + 150 | f(x) := 1000*x + 1000 |
| cost_begin | f(x) := 180 | f(x) := 1000 |
| cost_hash160 | f(x) := 1*x + 105 | f(x) := 1000*x + 1000 |
| cost_sha256 | f(x) := 1*x + 20 | f(x) := 1000*x + 1000 |
| cost_sha512 | f(x) := 1*x + 72 | f(x) := 1000*x + 1000 |
| cost_sha512t256 | f(x) := 1*x + 35 | f(x) := 1000*x + 1000 |
| cost_keccak256 | f(x) := 1*x + 102 | f(x) := 1000*x + 1000 |
| cost_secp256k1recover | f(x) := 9093 | f(x) := 1000 |
| cost_secp256k1verify | f(x) := 8798 | f(x) := 1000 |
| cost_some_cons | f(x) := 211 | f(x) := 1000 |
| cost_ok_cons | f(x) := 211 | f(x) := 1000 |
| cost_err_cons | f(x) := 211 | f(x) := 1000 |
| cost_default_to | f(x) := 306 | f(x) := 1000 |
| cost_unwrap_ret | f(x) := 327 | f(x) := 1000 |
| cost_unwrap_err_or_ret | f(x) := 313 | f(x) := 1000 |
| cost_is_okay | f(x) := 280 | f(x) := 1000 |
| cost_is_none | f(x) := 213 | f(x) := 1000 |
| cost_is_err | f(x) := 261 | f(x) := 1000 |
| cost_is_some | f(x) := 226 | f(x) := 1000 |
| cost_unwrap | f(x) := 292 | f(x) := 1000 |
| cost_unwrap_err | f(x) := 257 | f(x) := 1000 |
| cost_try_ret | f(x) := 276 | f(x) := 1000 |
| cost_match | f(x) := 313 | f(x) := 1000 |
| cost_or | f(x) := 6*x + 123 | f(x) := 1000*x + 1000 |
| cost_and | f(x) := 6*x + 123 | f(x) := 1000*x + 1000 |
| cost_append | f(x) := 80*x + 336 | f(x) := 1000*x + 1000 |
| cost_concat | f(x) := 14*x + 517 | f(x) := 1000*x + 1000 |
| cost_as_max_len | f(x) := 412 | f(x) := 1000 |
| cost_contract_call | f(x) := 141 | f(x) := 1000 |
| cost_contract_of | f(x) := 20271 | f(x) := 1000 |
| cost_principal_of | f(x) := 990 | f(x) := 1000 |
| cost_at_block | f(x) := 1327 | f(x) := 1000 |
| cost_load_contract | f(x) := 1*x + 1284 | f(x) := 1000*x + 1000 |
| cost_create_map | f(x) := 1*x + 1152 | f(x) := 1000*x + 1000 |
| cost_create_var | f(x) := 8*x + 1556 | f(x) := 1000*x + 1000 |
| cost_create_nft | f(x) := 1*x + 1146 | f(x) := 1000*x + 1000 |
| cost_create_ft | f(x) := 1439 | f(x) := 1000 |
| cost_fetch_entry | f(x) := 1*x + 1483 | f(x) := 1000*x + 1000 |
| cost_set_entry | f(x) := 4*x + 1725 | f(x) := 1000*x + 1000 |
| cost_fetch_var | f(x) := 1*x + 493 | f(x) := 1000*x + 1000 |
| cost_set_var | f(x) := 5*x + 661 | f(x) := 1000*x + 1000 |
| cost_contract_storage | f(x) := 11*x + 6460 | f(x) := 1000*x + 1000 |
| cost_block_info | f(x) := 8189 | f(x) := 1000 |
| cost_stx_balance | f(x) := 4294 | f(x) := 1000 |
| cost_stx_transfer | f(x) := 4640 | f(x) := 1000 |
| cost_ft_mint | f(x) := 1561 | f(x) := 1000 |
| cost_ft_transfer | f(x) := 573 | f(x) := 1000 |
| cost_ft_balance | f(x) := 504 | f(x) := 1000 |
| cost_nft_mint | f(x) := 10*x + 568 | f(x) := 1000*x + 1000 |
| cost_nft_transfer | f(x) := 10*x + 589 | f(x) := 1000*x + 1000 |
| cost_nft_owner | f(x) := 10*x + 829 | f(x) := 1000*x + 1000 |
| cost_ft_get_supply | f(x) := 429 | f(x) := 1000 |
| cost_ft_burn | f(x) := 573 | f(x) := 1000 |
| cost_nft_burn | f(x) := 10*x + 589 | f(x) := 1000*x + 1000 |
| poison_microblock | f(x) := 18418 | f(x) := 1000 |
| cost_buff_to_int_le | f(x) := 147 | f(x) := 1000 |
| cost_buff_to_uint_le | f(x) := 147 | f(x) := 1000 |
| cost_buff_to_int_be | f(x) := 147 | f(x) := 1000 |
| cost_buff_to_uint_be | f(x) := 147 | f(x) := 1000 |
| cost_is_standard | f(x) := 128 | f(x) := 1000 |
| cost_principal_destruct | f(x) := 369 | f(x) := 1000 |
| cost_principal_construct | f(x) := 403 | f(x) := 1000 |
| cost_string_to_int | f(x) := 180 | f(x) := 1000 |
| cost_string_to_uint | f(x) := 180 | f(x) := 1000 |
| cost_int_to_ascii | f(x) := 166 | f(x) := 1000 |
| cost_int_to_utf8 | f(x) := 198 | f(x) := 1000 |
| cost_burn_block_info | f(x) := 96479 | f(x) := 1000 |
| cost_stx_account | f(x) := 4654 | f(x) := 1000 |
| cost_slice | f(x) := 448 | f(x) := 1000 |
| cost_to_consensus_buff | f(x) := 1*x + 713 | f(x) := 1000*x + 1000 |
| cost_from_consensus_buff | f(x) := 3*x*log(x) + 174 | f(x) := 1000*x*log(x) + 1000 |
| cost_stx_transfer_memo | f(x) := 4709 | f(x) := 1000 |
| cost_replace_at | f(x) := 1*x + 561 | f(x) := 1000*x + 1000 |
| cost_as_contract | f(x) := 139 | f(x) := 1000 |
| cost_bitwise_and | f(x) := 16*x + 129 | f(x) := 1000*x + 1000 |
| cost_bitwise_or | f(x) := 15*x + 130 | f(x) := 1000*x + 1000 |
| cost_bitwise_not | f(x) := 147 | f(x) := 1000 |
| cost_bitwise_left_shift | f(x) := 167 | f(x) := 1000 |
| cost_bitwise_right_shift | f(x) := 169 | f(x) := 1000 |
