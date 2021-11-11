| Runtime cost | New function | Old function |
| ----------- | ----------- | ----------- |
| cost_analysis_type_annotate | f(x) := 1*x + 7 | f(x) := 1000*x + 1000 |
| cost_analysis_type_lookup | f(x) := 1*x + 4 | f(x) := 1000*x + 1000 |
| cost_analysis_visit | f(x) := 0 | f(x) := 1000 |
| cost_analysis_option_cons | f(x) := 4 | f(x) := 1000 |
| cost_analysis_option_check | f(x) := 2 | f(x) := 1000 |
| cost_analysis_bind_name | f(x) := 1*x + 132 | f(x) := 1000*x + 1000 |
| cost_analysis_list_items_check | f(x) := 1*x + 3 | f(x) := 1000*x + 1000 |
| cost_analysis_check_tuple_get | f(x) := 1*log(x) + 1 | f(x) := 1000*log(x) + 1000 |
| cost_analysis_check_tuple_cons | f(x) := 2*x*log(x) + 3 | f(x) := 1000*x*log(x) + 1000 |
| cost_analysis_tuple_items_check | f(x) := 1*x + 44 | f(x) := 1000*x + 1000 |
| cost_analysis_check_let | f(x) := 1*x + 9 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_function | f(x) := 15 | f(x) := 1000 |
| cost_analysis_lookup_function_types | f(x) := 1*x + 21 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_variable_const | f(x) := 11 | f(x) := 1000 |
| cost_analysis_lookup_variable_depth | f(x) := 1*x*log(x) + 26 | f(x) := 1000*x*log(x) + 1000 |
| cost_ast_parse | f(x) := 129*x + 215581 | f(x) := 1000*x + 1000 |
| cost_ast_cycle_detection | f(x) := 106*x + 54 | f(x) := 1000*x + 1000 |
| cost_analysis_storage | f(x) := 1*x + 75 | f(x) := 1000*x + 1000 |
| cost_analysis_use_trait_entry | f(x) := 7*x + 542 | f(x) := 1000*x + 1000 |
| cost_analysis_get_function_entry | f(x) := 61*x + 977 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_depth | f(x) := 2*x + 11 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_size | f(x) := 1*x + 0 | f(x) := 1000*x + 1000 |
| cost_lookup_function | f(x) := 12 | f(x) := 1000 |
| cost_bind_name | f(x) := 192 | f(x) := 1000 |
| cost_inner_type_check_cost | f(x) := 1*x + 7 | f(x) := 1000*x + 1000 |
| cost_user_function_application | f(x) := 20*x + 105 | f(x) := 1000*x + 1000 |
| cost_let | f(x) := 110*x + 646 | f(x) := 1000*x + 1000 |
| cost_if | f(x) := 150 | f(x) := 1000 |
| cost_asserts | f(x) := 118 | f(x) := 1000 |
| cost_map | f(x) := 908*x + 2486 | f(x) := 1000*x + 1000 |
| cost_filter | f(x) := 345 | f(x) := 1000 |
| cost_len | f(x) := 365 | f(x) := 1000 |
| cost_element_at | f(x) := 464 | f(x) := 1000 |
| cost_fold | f(x) := 362 | f(x) := 1000 |
| cost_type_parse_step | f(x) := 3 | f(x) := 1000 |
| cost_tuple_get | f(x) := 3*x*log(x) + 1335 | f(x) := 1000*x*log(x) + 1000 |
| cost_tuple_merge | f(x) := 3*x + 484 | f(x) := 1000*x + 1000 |
| cost_tuple_cons | f(x) := 8*x*log(x) + 826 | f(x) := 1000*x*log(x) + 1000 |
| cost_add | f(x) := 9*x + 117 | f(x) := 1000*x + 1000 |
| cost_sub | f(x) := 9*x + 117 | f(x) := 1000*x + 1000 |
| cost_mul | f(x) := 10*x + 117 | f(x) := 1000*x + 1000 |
| cost_div | f(x) := 10*x + 117 | f(x) := 1000*x + 1000 |
| cost_geq | f(x) := 124 | f(x) := 1000 |
| cost_leq | f(x) := 124 | f(x) := 1000 |
| cost_le | f(x) := 124 | f(x) := 1000 |
| cost_ge | f(x) := 124 | f(x) := 1000 |
| cost_int_cast | f(x) := 123 | f(x) := 1000 |
| cost_mod | f(x) := 126 | f(x) := 1000 |
| cost_pow | f(x) := 127 | f(x) := 1000 |
| cost_sqrti | f(x) := 125 | f(x) := 1000 |
| cost_log2 | f(x) := 120 | f(x) := 1000 |
| cost_xor | f(x) := 125 | f(x) := 1000 |
| cost_not | f(x) := 121 | f(x) := 1000 |
| cost_eq | f(x) := 5*x + 129 | f(x) := 1000*x + 1000 |
| cost_begin | f(x) := 151 | f(x) := 1000 |
| cost_secp256k1recover | f(x) := 10758 | f(x) := 1000 |
| cost_secp256k1verify | f(x) := 10155 | f(x) := 1000 |
| cost_some_cons | f(x) := 164 | f(x) := 1000 |
| cost_ok_cons | f(x) := 164 | f(x) := 1000 |
| cost_err_cons | f(x) := 164 | f(x) := 1000 |
| cost_default_to | f(x) := 187 | f(x) := 1000 |
| cost_unwrap_ret | f(x) := 224 | f(x) := 1000 |
| cost_unwrap_err_or_ret | f(x) := 254 | f(x) := 1000 |
| cost_is_okay | f(x) := 215 | f(x) := 1000 |
| cost_is_none | f(x) := 159 | f(x) := 1000 |
| cost_is_err | f(x) := 211 | f(x) := 1000 |
| cost_is_some | f(x) := 167 | f(x) := 1000 |
| cost_unwrap | f(x) := 213 | f(x) := 1000 |
| cost_unwrap_err | f(x) := 198 | f(x) := 1000 |
| cost_try_ret | f(x) := 192 | f(x) := 1000 |
| cost_match | f(x) := 214 | f(x) := 1000 |
| cost_or | f(x) := 2*x + 111 | f(x) := 1000*x + 1000 |
| cost_and | f(x) := 2*x + 111 | f(x) := 1000*x + 1000 |
| cost_append | f(x) := 53*x + 132 | f(x) := 1000*x + 1000 |
| cost_concat | f(x) := 56*x + 183 | f(x) := 1000*x + 1000 |
| cost_as_max_len | f(x) := 356 | f(x) := 1000 |
| cost_contract_call | f(x) := 115 | f(x) := 1000 |
| cost_contract_of | f(x) := 10050 | f(x) := 1000 |
| cost_principal_of | f(x) := 29 | f(x) := 1000 |
| cost_at_block | f(x) := 157 | f(x) := 1000 |
| cost_load_contract | f(x) := 1*x + 118 | f(x) := 1000*x + 1000 |
| cost_create_map | f(x) := 1*x + 1223 | f(x) := 1000*x + 1000 |
| cost_create_var | f(x) := 5*x + 1614 | f(x) := 1000*x + 1000 |
| cost_create_nft | f(x) := 1*x + 1207 | f(x) := 1000*x + 1000 |
| cost_create_ft | f(x) := 1479 | f(x) := 1000 |
| cost_fetch_entry | f(x) := 1*x + 1154 | f(x) := 1000*x + 1000 |
| cost_set_entry | f(x) := 3*x + 1653 | f(x) := 1000*x + 1000 |
| cost_fetch_var | f(x) := 1*x + 407 | f(x) := 1000*x + 1000 |
| cost_set_var | f(x) := 3*x + 518 | f(x) := 1000*x + 1000 |
| cost_contract_storage | f(x) := 10*x + 5987 | f(x) := 1000*x + 1000 |
| cost_block_info | f(x) := 4741 | f(x) := 1000 |
| cost_stx_balance | f(x) := 1039 | f(x) := 1000 |
| cost_stx_transfer | f(x) := 1072 | f(x) := 1000 |
| cost_ft_mint | f(x) := 1234 | f(x) := 1000 |
| cost_ft_transfer | f(x) := 459 | f(x) := 1000 |
| cost_ft_balance | f(x) := 410 | f(x) := 1000 |
| cost_nft_mint | f(x) := 7*x + 477 | f(x) := 1000*x + 1000 |
| cost_nft_transfer | f(x) := 6*x + 485 | f(x) := 1000*x + 1000 |
| cost_nft_owner | f(x) := 7*x + 596 | f(x) := 1000*x + 1000 |
| cost_ft_get_supply | f(x) := 362 | f(x) := 1000 |
| cost_ft_burn | f(x) := 459 | f(x) := 1000 |
| cost_nft_burn | f(x) := 6*x + 485 | f(x) := 1000*x + 1000 |
| poison_microblock | f(x) := 22176 | f(x) := 1000 |
