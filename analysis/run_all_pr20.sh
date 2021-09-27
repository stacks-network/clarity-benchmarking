for MODEL_NAME in cost_fetch_entry cost_set_entry cost_fetch_var cost_set_var cost_list_cons cost_hash160 cost_sha256 cost_sha512 cost_sha512t256 cost_keccak256 cost_print
do
echo $MODEL_NAME

python3 analysis/draw_single_curve.py ${MODEL_NAME}

done
