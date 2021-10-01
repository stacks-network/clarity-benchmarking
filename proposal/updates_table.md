| Runtime cost | New function | Old function |
| ----------- | ----------- | ----------- |
| cost_analysis_type_annotate | f(x) := 3*x + 12 | f(x) := 1000*x + 1000 |
| cost_analysis_type_lookup | f(x) := 1*x + 5 | f(x) := 1000*x + 1000 |
| cost_analysis_visit | f(x) := 17 | f(x) := 1000 |
| cost_analysis_option_cons | f(x) := 51 | f(x) := 1000 |
| cost_analysis_option_check | f(x) := 131 | f(x) := 1000 |
| cost_analysis_bind_name | f(x) := 14*x + 144 | f(x) := 1000*x + 1000 |
| cost_analysis_list_items_check | f(x) := 25*x + 5 | f(x) := 1000*x + 1000 |
| cost_analysis_check_tuple_get | f(x) := 1*log(x) + 1 | f(x) := 1000*log(x) + 1000 |
| cost_analysis_check_tuple_cons | f(x) := 12*x*log(x) + 64 | f(x) := 1000*x*log(x) + 1000 |
| cost_analysis_tuple_items_check | f(x) := 13*x + 50 | f(x) := 1000*x + 1000 |
| cost_analysis_check_let | f(x) := 51*x + 87 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_function | f(x) := 21 | f(x) := 1000 |
| cost_analysis_lookup_function_types | f(x) := 1*x + 27 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_variable_const | f(x) := 15 | f(x) := 1000 |
| cost_analysis_lookup_variable_depth | f(x) := 1*x*log(x) + 65 | f(x) := 1000*x*log(x) + 1000 |
| cost_ast_parse | f(x) := 171*x + 282923 | f(x) := 1000*x + 1000 |
| cost_ast_cycle_detection | f(x) := 141*x + 26 | f(x) := 1000*x + 1000 |
| cost_analysis_storage | f(x) := 1*x + 5 | f(x) := 1000*x + 1000 |
| cost_analysis_use_trait_entry | f(x) := 9*x + 736 | f(x) := 1000*x + 1000 |
| cost_analysis_get_function_entry | f(x) := 82*x + 1345 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_depth | f(x) := 2*x + 14 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_size | f(x) := 2*x + 1 | f(x) := 1000*x + 1000 |
| cost_lookup_function | f(x) := 26 | f(x) := 1000 |
| cost_bind_name | f(x) := 273 | f(x) := 1000 |
| cost_inner_type_check_cost | f(x) := 2*x + 9 | f(x) := 1000*x + 1000 |
| cost_user_function_application | f(x) := 26*x + 0 | f(x) := 1000*x + 1000 |
| cost_let | f(x) := 1*x + 270 | f(x) := 1000*x + 1000 |
| cost_if | f(x) := 191 | f(x) := 1000 |
| cost_asserts | f(x) := 151 | f(x) := 1000 |
| cost_map | f(x) := 1186*x + 3325 | f(x) := 1000*x + 1000 |
| cost_filter | f(x) := 437 | f(x) := 1000 |
| cost_len | f(x) := 444 | f(x) := 1000 |
| cost_element_at | f(x) := 548 | f(x) := 1000 |
| cost_fold | f(x) := 489 | f(x) := 1000 |
| cost_type_parse_step | f(x) := 5 | f(x) := 1000 |
| cost_tuple_get | f(x) := 4*x*log(x) + 1780 | f(x) := 1000*x*log(x) + 1000 |
| cost_tuple_merge | f(x) := 208*x + 185 | f(x) := 1000*x + 1000 |
| cost_tuple_cons | f(x) := 11*x*log(x) + 1481 | f(x) := 1000*x*log(x) + 1000 |
| cost_add | f(x) := 11*x + 152 | f(x) := 1000*x + 1000 |
| cost_sub | f(x) := 11*x + 152 | f(x) := 1000*x + 1000 |
| cost_mul | f(x) := 12*x + 151 | f(x) := 1000*x + 1000 |
| cost_div | f(x) := 13*x + 151 | f(x) := 1000*x + 1000 |
| cost_geq | f(x) := 162 | f(x) := 1000 |
| cost_leq | f(x) := 164 | f(x) := 1000 |
| cost_le | f(x) := 152 | f(x) := 1000 |
| cost_ge | f(x) := 152 | f(x) := 1000 |
| cost_int_cast | f(x) := 157 | f(x) := 1000 |
| cost_mod | f(x) := 166 | f(x) := 1000 |
| cost_pow | f(x) := 166 | f(x) := 1000 |
| cost_sqrti | f(x) := 165 | f(x) := 1000 |
| cost_log2 | f(x) := 156 | f(x) := 1000 |
| cost_xor | f(x) := 163 | f(x) := 1000 |
| cost_not | f(x) := 158 | f(x) := 1000 |
| cost_eq | f(x) := 8*x + 155 | f(x) := 1000*x + 1000 |
| cost_begin | f(x) := 189 | f(x) := 1000 |
| cost_secp256k1recover | f(x) := 14312 | f(x) := 1000 |
| cost_secp256k1verify | f(x) := 13488 | f(x) := 1000 |
| cost_some_cons | f(x) := 217 | f(x) := 1000 |
| cost_ok_cons | f(x) := 209 | f(x) := 1000 |
| cost_err_cons | f(x) := 205 | f(x) := 1000 |
| cost_default_to | f(x) := 255 | f(x) := 1000 |
| cost_unwrap_ret | f(x) := 330 | f(x) := 1000 |
| cost_unwrap_err_or_ret | f(x) := 319 | f(x) := 1000 |
| cost_is_okay | f(x) := 275 | f(x) := 1000 |
| cost_is_none | f(x) := 229 | f(x) := 1000 |
| cost_is_err | f(x) := 268 | f(x) := 1000 |
| cost_is_some | f(x) := 217 | f(x) := 1000 |
| cost_unwrap | f(x) := 281 | f(x) := 1000 |
| cost_unwrap_err | f(x) := 273 | f(x) := 1000 |
| cost_try_ret | f(x) := 275 | f(x) := 1000 |
| cost_match | f(x) := 316 | f(x) := 1000 |
| cost_or | f(x) := 3*x + 147 | f(x) := 1000*x + 1000 |
| cost_and | f(x) := 3*x + 146 | f(x) := 1000*x + 1000 |
| cost_append | f(x) := 1*x + 1024 | f(x) := 1000*x + 1000 |
| cost_concat | f(x) := 1*x + 1004 | f(x) := 1000*x + 1000 |
| cost_as_max_len | f(x) := 482 | f(x) := 1000 |
| cost_contract_call | f(x) := 154 | f(x) := 1000 |
| cost_contract_of | f(x) := 13391 | f(x) := 1000 |
| cost_principal_of | f(x) := 15 | f(x) := 1000 |
| cost_at_block | f(x) := 205 | f(x) := 1000 |
| cost_load_contract | f(x) := 1*x + 10 | f(x) := 1000*x + 1000 |
| cost_create_map | f(x) := 3*x + 1650 | f(x) := 1000*x + 1000 |
| cost_create_var | f(x) := 24*x + 2170 | f(x) := 1000*x + 1000 |
| cost_create_nft | f(x) := 4*x + 1624 | f(x) := 1000*x + 1000 |
| cost_create_ft | f(x) := 2025 | f(x) := 1000 |
| cost_fetch_entry | f(x) := 1*x + 1466 | f(x) := 1000*x + 1000 |
| cost_set_entry | f(x) := 1*x + 1574 | f(x) := 1000*x + 1000 |
| cost_fetch_var | f(x) := 1*x + 679 | f(x) := 1000*x + 1000 |
| cost_set_var | f(x) := 1*x + 723 | f(x) := 1000*x + 1000 |
| cost_contract_storage | f(x) := 13*x + 8043 | f(x) := 1000*x + 1000 |
| cost_block_info | f(x) := 5886 | f(x) := 1000 |
| cost_stx_balance | f(x) := 1386 | f(x) := 1000 |
| cost_stx_transfer | f(x) := 1444 | f(x) := 1000 |
| cost_ft_mint | f(x) := 1624 | f(x) := 1000 |
| cost_ft_transfer | f(x) := 563 | f(x) := 1000 |
| cost_ft_balance | f(x) := 543 | f(x) := 1000 |
| cost_nft_mint | f(x) := 1*x + 724 | f(x) := 1000*x + 1000 |
| cost_nft_transfer | f(x) := 1*x + 787 | f(x) := 1000*x + 1000 |
| cost_nft_owner | f(x) := 1*x + 680 | f(x) := 1000*x + 1000 |
| cost_ft_get_supply | f(x) := 474 | f(x) := 1000 |
| cost_ft_burn | f(x) := 599 | f(x) := 1000 |
| cost_nft_burn | f(x) := 1*x + 644 | f(x) := 1000*x + 1000 |
| poison_microblock | f(x) := 29374 | f(x) := 1000 |
