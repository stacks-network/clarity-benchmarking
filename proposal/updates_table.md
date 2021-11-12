| Runtime cost | New function | Old function |
| ----------- | ----------- | ----------- |
| cost_analysis_type_annotate | f(x) := 1*x + 9 | f(x) := 1000*x + 1000 |
| cost_analysis_type_check | f(x) := 113*x + 0 | f(x) := 1000*x + 1000 |
| cost_analysis_type_lookup | f(x) := 1*x + 6 | f(x) := 1000*x + 1000 |
| cost_analysis_visit | f(x) := 1 | f(x) := 1000 |
| cost_analysis_iterable_func | f(x) := 2*x + 14 | f(x) := 1000*x + 1000 |
| cost_analysis_option_cons | f(x) := 6 | f(x) := 1000 |
| cost_analysis_option_check | f(x) := 3 | f(x) := 1000 |
| cost_analysis_bind_name | f(x) := 2*x + 176 | f(x) := 1000*x + 1000 |
| cost_analysis_list_items_check | f(x) := 2*x + 4 | f(x) := 1000*x + 1000 |
| cost_analysis_check_tuple_get | f(x) := 1*log(x) + 2 | f(x) := 1000*log(x) + 1000 |
| cost_analysis_check_tuple_cons | f(x) := 3*x*log(x) + 5 | f(x) := 1000*x*log(x) + 1000 |
| cost_analysis_tuple_items_check | f(x) := 1*x + 59 | f(x) := 1000*x + 1000 |
| cost_analysis_check_let | f(x) := 1*x + 12 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_function | f(x) := 20 | f(x) := 1000 |
| cost_analysis_lookup_function_types | f(x) := 1*x + 28 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_variable_const | f(x) := 15 | f(x) := 1000 |
| cost_analysis_lookup_variable_depth | f(x) := 1*x*log(x) + 34 | f(x) := 1000*x*log(x) + 1000 |
| cost_ast_parse | f(x) := 172*x + 287441 | f(x) := 1000*x + 1000 |
| cost_ast_cycle_detection | f(x) := 141*x + 72 | f(x) := 1000*x + 1000 |
| cost_analysis_storage | f(x) := 2*x + 100 | f(x) := 1000*x + 1000 |
| cost_analysis_use_trait_entry | f(x) := 9*x + 723 | f(x) := 1000*x + 1000 |
| cost_analysis_get_function_entry | f(x) := 81*x + 1303 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_depth | f(x) := 2*x + 14 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_size | f(x) := 2*x + 0 | f(x) := 1000*x + 1000 |
| cost_lookup_function | f(x) := 16 | f(x) := 1000 |
| cost_bind_name | f(x) := 256 | f(x) := 1000 |
| cost_inner_type_check_cost | f(x) := 2*x + 9 | f(x) := 1000*x + 1000 |
| cost_user_function_application | f(x) := 26*x + 140 | f(x) := 1000*x + 1000 |
| cost_let | f(x) := 146*x + 862 | f(x) := 1000*x + 1000 |
| cost_if | f(x) := 200 | f(x) := 1000 |
| cost_asserts | f(x) := 158 | f(x) := 1000 |
| cost_map | f(x) := 1210*x + 3314 | f(x) := 1000*x + 1000 |
| cost_filter | f(x) := 460 | f(x) := 1000 |
| cost_len | f(x) := 486 | f(x) := 1000 |
| cost_element_at | f(x) := 619 | f(x) := 1000 |
| cost_index_of | f(x) := 1*x + 243 | f(x) := 1000*x + 1000 |
| cost_fold | f(x) := 483 | f(x) := 1000 |
| cost_list_cons | f(x) := 14*x + 198 | f(x) := 1000*x + 1000 |
| cost_type_parse_step | f(x) := 5 | f(x) := 1000 |
| cost_tuple_get | f(x) := 4*x*log(x) + 1780 | f(x) := 1000*x*log(x) + 1000 |
| cost_tuple_merge | f(x) := 4*x + 646 | f(x) := 1000*x + 1000 |
| cost_tuple_cons | f(x) := 11*x*log(x) + 1101 | f(x) := 1000*x*log(x) + 1000 |
| cost_add | f(x) := 12*x + 156 | f(x) := 1000*x + 1000 |
| cost_sub | f(x) := 12*x + 156 | f(x) := 1000*x + 1000 |
| cost_mul | f(x) := 14*x + 157 | f(x) := 1000*x + 1000 |
| cost_div | f(x) := 14*x + 157 | f(x) := 1000*x + 1000 |
| cost_geq | f(x) := 166 | f(x) := 1000 |
| cost_leq | f(x) := 166 | f(x) := 1000 |
| cost_le | f(x) := 166 | f(x) := 1000 |
| cost_ge | f(x) := 166 | f(x) := 1000 |
| cost_int_cast | f(x) := 164 | f(x) := 1000 |
| cost_mod | f(x) := 168 | f(x) := 1000 |
| cost_pow | f(x) := 170 | f(x) := 1000 |
| cost_sqrti | f(x) := 167 | f(x) := 1000 |
| cost_log2 | f(x) := 161 | f(x) := 1000 |
| cost_xor | f(x) := 167 | f(x) := 1000 |
| cost_not | f(x) := 162 | f(x) := 1000 |
| cost_eq | f(x) := 7*x + 172 | f(x) := 1000*x + 1000 |
| cost_begin | f(x) := 202 | f(x) := 1000 |
| cost_hash160 | f(x) := 1*x + 201 | f(x) := 1000*x + 1000 |
| cost_sha256 | f(x) := 1*x + 100 | f(x) := 1000*x + 1000 |
| cost_sha512 | f(x) := 1*x + 176 | f(x) := 1000*x + 1000 |
| cost_sha512t256 | f(x) := 1*x + 188 | f(x) := 1000*x + 1000 |
| cost_keccak256 | f(x) := 1*x + 221 | f(x) := 1000*x + 1000 |
| cost_secp256k1recover | f(x) := 14344 | f(x) := 1000 |
| cost_secp256k1verify | f(x) := 13540 | f(x) := 1000 |
| cost_print | f(x) := 3*x + 1413 | f(x) := 1000*x + 1000 |
| cost_some_cons | f(x) := 219 | f(x) := 1000 |
| cost_ok_cons | f(x) := 219 | f(x) := 1000 |
| cost_err_cons | f(x) := 219 | f(x) := 1000 |
| cost_default_to | f(x) := 249 | f(x) := 1000 |
| cost_unwrap_ret | f(x) := 299 | f(x) := 1000 |
| cost_unwrap_err_or_ret | f(x) := 339 | f(x) := 1000 |
| cost_is_okay | f(x) := 287 | f(x) := 1000 |
| cost_is_none | f(x) := 212 | f(x) := 1000 |
| cost_is_err | f(x) := 282 | f(x) := 1000 |
| cost_is_some | f(x) := 223 | f(x) := 1000 |
| cost_unwrap | f(x) := 284 | f(x) := 1000 |
| cost_unwrap_err | f(x) := 264 | f(x) := 1000 |
| cost_try_ret | f(x) := 256 | f(x) := 1000 |
| cost_match | f(x) := 286 | f(x) := 1000 |
| cost_or | f(x) := 3*x + 149 | f(x) := 1000*x + 1000 |
| cost_and | f(x) := 3*x + 149 | f(x) := 1000*x + 1000 |
| cost_append | f(x) := 71*x + 176 | f(x) := 1000*x + 1000 |
| cost_concat | f(x) := 75*x + 244 | f(x) := 1000*x + 1000 |
| cost_as_max_len | f(x) := 475 | f(x) := 1000 |
| cost_contract_call | f(x) := 153 | f(x) := 1000 |
| cost_contract_of | f(x) := 13400 | f(x) := 1000 |
| cost_principal_of | f(x) := 39 | f(x) := 1000 |
| cost_at_block | f(x) := 210 | f(x) := 1000 |
| cost_load_contract | f(x) := 1*x + 157 | f(x) := 1000*x + 1000 |
| cost_create_map | f(x) := 1*x + 1631 | f(x) := 1000*x + 1000 |
| cost_create_var | f(x) := 7*x + 2152 | f(x) := 1000*x + 1000 |
| cost_create_nft | f(x) := 1*x + 1610 | f(x) := 1000*x + 1000 |
| cost_create_ft | f(x) := 1972 | f(x) := 1000 |
| cost_fetch_entry | f(x) := 1*x + 1539 | f(x) := 1000*x + 1000 |
| cost_set_entry | f(x) := 4*x + 2204 | f(x) := 1000*x + 1000 |
| cost_fetch_var | f(x) := 1*x + 543 | f(x) := 1000*x + 1000 |
| cost_set_var | f(x) := 5*x + 691 | f(x) := 1000*x + 1000 |
| cost_contract_storage | f(x) := 13*x + 7982 | f(x) := 1000*x + 1000 |
| cost_block_info | f(x) := 6321 | f(x) := 1000 |
| cost_stx_balance | f(x) := 1385 | f(x) := 1000 |
| cost_stx_transfer | f(x) := 1430 | f(x) := 1000 |
| cost_ft_mint | f(x) := 1645 | f(x) := 1000 |
| cost_ft_transfer | f(x) := 612 | f(x) := 1000 |
| cost_ft_balance | f(x) := 547 | f(x) := 1000 |
| cost_nft_mint | f(x) := 9*x + 636 | f(x) := 1000*x + 1000 |
| cost_nft_transfer | f(x) := 9*x + 647 | f(x) := 1000*x + 1000 |
| cost_nft_owner | f(x) := 9*x + 795 | f(x) := 1000*x + 1000 |
| cost_ft_get_supply | f(x) := 483 | f(x) := 1000 |
| cost_ft_burn | f(x) := 612 | f(x) := 1000 |
| cost_nft_burn | f(x) := 9*x + 647 | f(x) := 1000*x + 1000 |
| poison_microblock | f(x) := 29568 | f(x) := 1000 |
