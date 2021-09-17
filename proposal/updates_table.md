| Runtime cost | New function | Old function |
| ----------- | ----------- | ----------- |
| cost_analysis_type_annotate | f(x) := 3*x + 13 | f(x) := 1000*x + 1000 |
| cost_analysis_type_lookup | f(x) := 1*x + 6 | f(x) := 1000*x + 1000 |
| cost_analysis_visit | f(x) := 16 | f(x) := 1000 |
| cost_analysis_option_cons | f(x) := 60 | f(x) := 1000 |
| cost_analysis_option_check | f(x) := 118 | f(x) := 1000 |
| cost_analysis_bind_name | f(x) := 14*x + 174 | f(x) := 1000*x + 1000 |
| cost_analysis_list_items_check | f(x) := 25*x + 5 | f(x) := 1000*x + 1000 |
| cost_analysis_check_tuple_get | f(x) := 1*log(x) + 2 | f(x) := 1000*log(x) + 1000 |
| cost_analysis_check_tuple_cons | f(x) := 11*x*log(x) + 155 | f(x) := 1000*x*log(x) + 1000 |
| cost_analysis_tuple_items_check | f(x) := 13*x + 56 | f(x) := 1000*x + 1000 |
| cost_analysis_check_let | f(x) := 46*x + 93 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_function | f(x) := 19 | f(x) := 1000 |
| cost_analysis_lookup_function_types | f(x) := 1*x + 28 | f(x) := 1000*x + 1000 |
| cost_analysis_lookup_variable_const | f(x) := 15 | f(x) := 1000 |
| cost_analysis_lookup_variable_depth | f(x) := 1*x*log(x) + 65 | f(x) := 1000*x*log(x) + 1000 |
| cost_ast_parse | f(x) := 173*x + 285411 | f(x) := 1000*x + 1000 |
| cost_ast_cycle_detection | f(x) := 142*x + 25 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_depth | f(x) := 2*x + 14 | f(x) := 1000*x + 1000 |
| cost_lookup_variable_size | f(x) := 2*x + 0 | f(x) := 1000*x + 1000 |
| cost_lookup_function | f(x) := 25 | f(x) := 1000 |
| cost_bind_name | f(x) := 270 | f(x) := 1000 |
| cost_inner_type_check_cost | f(x) := 2*x + 10 | f(x) := 1000*x + 1000 |
| cost_user_function_application | f(x) := 26*x + 0 | f(x) := 1000*x + 1000 |
| cost_let | f(x) := 1*x + 290 | f(x) := 1000*x + 1000 |
| cost_if | f(x) := 199 | f(x) := 1000 |
| cost_asserts | f(x) := 158 | f(x) := 1000 |
| cost_map | f(x) := 1181*x + 3113 | f(x) := 1000*x + 1000 |
| cost_filter | f(x) := 464 | f(x) := 1000 |
| cost_len | f(x) := 566 | f(x) := 1000 |
| cost_element_at | f(x) := 489 | f(x) := 1000 |
| cost_fold | f(x) := 521 | f(x) := 1000 |
| cost_type_parse_step | f(x) := 5 | f(x) := 1000 |
| cost_tuple_get | f(x) := 4*x*log(x) + 1697 | f(x) := 1000*x*log(x) + 1000 |
| cost_tuple_merge | f(x) := 212*x + 248 | f(x) := 1000*x + 1000 |
| cost_tuple_cons | f(x) := 10*x*log(x) + 2166 | f(x) := 1000*x*log(x) + 1000 |
| cost_add | f(x) := 10*x + 171 | f(x) := 1000*x + 1000 |
| cost_sub | f(x) := 11*x + 170 | f(x) := 1000*x + 1000 |
| cost_mul | f(x) := 12*x + 169 | f(x) := 1000*x + 1000 |
| cost_div | f(x) := 13*x + 166 | f(x) := 1000*x + 1000 |
| cost_geq | f(x) := 179 | f(x) := 1000 |
| cost_leq | f(x) := 178 | f(x) := 1000 |
| cost_le | f(x) := 169 | f(x) := 1000 |
| cost_ge | f(x) := 170 | f(x) := 1000 |
| cost_int_cast | f(x) := 174 | f(x) := 1000 |
| cost_mod | f(x) := 183 | f(x) := 1000 |
| cost_pow | f(x) := 185 | f(x) := 1000 |
| cost_sqrti | f(x) := 179 | f(x) := 1000 |
| cost_log2 | f(x) := 172 | f(x) := 1000 |
| cost_xor | f(x) := 178 | f(x) := 1000 |
| cost_not | f(x) := 177 | f(x) := 1000 |
| cost_eq | f(x) := 7*x + 182 | f(x) := 1000*x + 1000 |
| cost_begin | f(x) := 212 | f(x) := 1000 |
| cost_secp256k1recover | f(x) := 14346 | f(x) := 1000 |
| cost_secp256k1verify | f(x) := 13544 | f(x) := 1000 |
| cost_some_cons | f(x) := 232 | f(x) := 1000 |
| cost_ok_cons | f(x) := 231 | f(x) := 1000 |
| cost_err_cons | f(x) := 219 | f(x) := 1000 |
| cost_default_to | f(x) := 260 | f(x) := 1000 |
| cost_unwrap_ret | f(x) := 350 | f(x) := 1000 |
| cost_unwrap_err_or_ret | f(x) := 333 | f(x) := 1000 |
| cost_is_okay | f(x) := 288 | f(x) := 1000 |
| cost_is_none | f(x) := 246 | f(x) := 1000 |
| cost_is_err | f(x) := 299 | f(x) := 1000 |
| cost_is_some | f(x) := 246 | f(x) := 1000 |
| cost_unwrap | f(x) := 309 | f(x) := 1000 |
| cost_unwrap_err | f(x) := 275 | f(x) := 1000 |
| cost_try_ret | f(x) := 294 | f(x) := 1000 |
| cost_match | f(x) := 306 | f(x) := 1000 |
| cost_or | f(x) := 3*x + 152 | f(x) := 1000*x + 1000 |
| cost_and | f(x) := 3*x + 153 | f(x) := 1000*x + 1000 |
| cost_append | f(x) := 1*x + 1026 | f(x) := 1000*x + 1000 |
| cost_concat | f(x) := 1*x + 867 | f(x) := 1000*x + 1000 |
| cost_as_max_len | f(x) := 508 | f(x) := 1000 |
| cost_contract_call | f(x) := 164 | f(x) := 1000 |
| cost_contract_of | f(x) := 45847 | f(x) := 1000 |
| cost_principal_of | f(x) := 35 | f(x) := 1000 |
