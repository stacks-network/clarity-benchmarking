use blockstack_lib::chainstate::stacks::index::MarfTrieId;
use blockstack_lib::chainstate::stacks::{StacksBlockHeader, StacksBlockId};
use blockstack_lib::core::{FIRST_BURNCHAIN_CONSENSUS_HASH, FIRST_STACKS_BLOCK_HASH};
use blockstack_lib::vm::clarity::ClarityInstance;
use blockstack_lib::vm::costs::cost_functions::ClarityCostFunction;
use blockstack_lib::vm::costs::ExecutionCost;
use blockstack_lib::vm::database::{MarfedKV, NULL_BURN_STATE_DB, NULL_HEADER_DB};
use blockstack_lib::vm::types::{PrincipalData, QualifiedContractIdentifier};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::Rng;

const INPUT_SIZES: [usize; 8] = [1, 2, 8, 16, 32, 64, 128, 256];
const SCALE: usize = 1000;

// generate arithmetic function call
fn gen_arithmetic(function_name: &'static str, scale: usize, input_size: usize) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let args = (0..input_size)
            .map(|_| format!("u{}", rng.gen_range(1..u128::MAX).to_string()))
            .collect::<Vec<String>>()
            .join(" ");
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    body
}

fn gen_cmp(function_name: &'static str, scale: usize) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let n1: u128 = rng.gen();
        let n2: u128 = rng.gen();
        body.push_str(&*format!("({} u{} u{}) ", function_name, n1, n2));
    }

    body
}

fn gen_logic(function_name: &'static str, scale: usize) -> String {
    let mut body = String::new();
    for _ in 0..scale {
        body.push_str(&*format!("({} true false) ", function_name));
    }
    body
}

fn gen_tuple_get(scale: usize, input_size: usize) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    let tuple_vals = (0..input_size)
        .map(|i| format!("(id{} 1337)", i))
        .collect::<Vec<String>>()
        .join(" ");

    let tuple = format!("(test-tuple (tuple {}))", tuple_vals);

    for _ in 0..scale {
        body.push_str(&*format!(
            "(get id{} test-tuple) ",
            rng.gen_range(0..input_size)
        ));
    }

    format!("(let ({}) {})", tuple, body)
}

// generate clarity code for benchmarking
fn gen(function: ClarityCostFunction, scale: usize, input_size: usize) -> String {
    let mut body = String::new();

    match function {
        ClarityCostFunction::Add => {
            body = gen_arithmetic("+", scale, input_size);
        }
        ClarityCostFunction::Sub => {
            body = gen_arithmetic("-", scale, input_size);
        }
        ClarityCostFunction::Mul => {
            body = gen_arithmetic("*", scale, input_size);
        }
        ClarityCostFunction::Div => {
            body = gen_arithmetic("/", scale, input_size);
        }
        ClarityCostFunction::Le => body = gen_cmp("<", scale),
        ClarityCostFunction::Leq => body = gen_cmp("<=", scale),
        ClarityCostFunction::Ge => body = gen_cmp(">", scale),
        ClarityCostFunction::Geq => body = gen_cmp(">=", scale),
        ClarityCostFunction::And => body = gen_logic("and", scale),
        ClarityCostFunction::Or => body = gen_logic("or", scale),
        ClarityCostFunction::Mod => body = gen_arithmetic("mod", scale, input_size),
        ClarityCostFunction::Pow => body = gen_arithmetic("pow", scale, input_size),
        ClarityCostFunction::Sqrti => body = gen_arithmetic("sqrti", scale, 1),
        ClarityCostFunction::Log2 => body = gen_arithmetic("log2", scale, 1),
        ClarityCostFunction::TupleGet => body = gen_tuple_get(scale, input_size),
        _ => {}
    }

    format!("(define-public (test) (begin {} (ok true)))", body)
}

fn bench_with_input_sizes(
    c: &mut Criterion,
    function: ClarityCostFunction,
    scale: usize,
    input_sizes: Vec<usize>,
) {
    let marf = MarfedKV::temporary();
    let mut clarity_instance = ClarityInstance::new(marf, ExecutionCost::max_value());

    let bhh = StacksBlockHeader::make_index_block_hash(
        &FIRST_BURNCHAIN_CONSENSUS_HASH,
        &FIRST_STACKS_BLOCK_HASH,
    );

    let mut conn = clarity_instance.begin_genesis_block(
        &StacksBlockId::sentinel(),
        &bhh.clone(),
        &NULL_HEADER_DB,
        &NULL_BURN_STATE_DB,
    );

    let p = PrincipalData::from(
        PrincipalData::parse_standard_principal("SM2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKQVX8X0G")
            .unwrap(),
    );

    let mut group = c.benchmark_group(function.to_string());

    for input_size in input_sizes.iter() {
        let contract_identifier =
            QualifiedContractIdentifier::local(&*format!("c{}", input_size)).unwrap();
        let contract = gen(function, scale, *input_size);

        conn.as_transaction(|tx| {
            let (ct_ast, _ct_analysis) = tx
                .analyze_smart_contract(&contract_identifier, &contract)
                .unwrap();
            tx.initialize_smart_contract(&contract_identifier, &ct_ast, &*contract, |_, _| false)
                .unwrap();
        });

        group.throughput(Throughput::Bytes(input_size.clone() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(input_size),
            input_size,
            |b, &_| {
                b.iter(|| {
                    conn.as_transaction(|tx| {
                        tx.run_contract_call(&p, &contract_identifier, "test", &[], |_, _| false)
                    })
                    .unwrap()
                })
            },
        );
    }
}

fn bench_add(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Add,
        SCALE.into(),
        INPUT_SIZES.into(),
    )
}

fn bench_sub(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Sub,
        SCALE.into(),
        INPUT_SIZES.into(),
    )
}

fn bench_le(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Le, SCALE.into(), vec![2])
}

fn bench_leq(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Leq, SCALE.into(), vec![2])
}

fn bench_ge(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Ge, SCALE.into(), vec![2])
}

fn bench_geq(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Geq, SCALE.into(), vec![2])
}

fn bench_and(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::And, SCALE.into(), vec![2])
}

fn bench_or(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Or, SCALE.into(), vec![2])
}

fn bench_mod(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Mod, SCALE.into(), vec![2])
}

fn bench_pow(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Pow, SCALE.into(), vec![2])
}

fn bench_sqrti(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Sqrti, SCALE.into(), vec![1])
}

fn bench_log2(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Log2, SCALE.into(), vec![1])
}

fn bench_tuple_get(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::TupleGet,
        SCALE.into(),
        vec![1, 2, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096],
    )
}

criterion_group!(
    benches,
    // bench_add,
    // bench_sub,
    // bench_le,
    // bench_leq,
    // bench_ge,
    // bench_geq,
    // bench_and,
    // bench_or,
    // bench_mod,
    // bench_pow,
    // bench_sqrti,
    // bench_log2,
    bench_tuple_get,
);

// AnalysisTypeAnnotate("cost_analysis_type_annotate"),
// AnalysisTypeCheck("cost_analysis_type_check"),
// AnalysisTypeLookup("cost_analysis_type_lookup"),
// AnalysisVisit("cost_analysis_visit"),
// AnalysisIterableFunc("cost_analysis_iterable_func"),
// AnalysisOptionCons("cost_analysis_option_cons"),
// AnalysisOptionCheck("cost_analysis_option_check"),
// AnalysisBindName("cost_analysis_bind_name"),
// AnalysisListItemsCheck("cost_analysis_list_items_check"),
// AnalysisCheckTupleGet("cost_analysis_check_tuple_get"),
// AnalysisCheckTupleMerge("cost_analysis_check_tuple_merge"),
// AnalysisCheckTupleCons("cost_analysis_check_tuple_cons"),
// AnalysisTupleItemsCheck("cost_analysis_tuple_items_check"),
// AnalysisCheckLet("cost_analysis_check_let"),
// AnalysisLookupFunction("cost_analysis_lookup_function"),
// AnalysisLookupFunctionTypes("cost_analysis_lookup_function_types"),
// AnalysisLookupVariableConst("cost_analysis_lookup_variable_const"),
// AnalysisLookupVariableDepth("cost_analysis_lookup_variable_depth"),
// AstParse("cost_ast_parse"),
// AstCycleDetection("cost_ast_cycle_detection"),
// AnalysisStorage("cost_analysis_storage"),
// AnalysisUseTraitEntry("cost_analysis_use_trait_entry"),
// AnalysisGetFunctionEntry("cost_analysis_get_function_entry"),
// AnalysisFetchContractEntry("cost_analysis_fetch_contract_entry"),
// LookupVariableDepth("cost_lookup_variable_depth"),
// LookupVariableSize("cost_lookup_variable_size"),
// LookupFunction("cost_lookup_function"),
// BindName("cost_bind_name"),
// InnerTypeCheckCost("cost_inner_type_check_cost"),
// UserFunctionApplication("cost_user_function_application"),
// Let("cost_let"),
// If("cost_if"),
// Asserts("cost_asserts"),
// Map("cost_map"),
// Filter("cost_filter"),
// Len("cost_len"),
// ElementAt("cost_element_at"),
// IndexOf("cost_index_of"),
// Fold("cost_fold"),
// ListCons("cost_list_cons"),
// TypeParseStep("cost_type_parse_step"),
// TupleGet("cost_tuple_get"),
// TupleMerge("cost_tuple_merge"),
// TupleCons("cost_tuple_cons"),
// IntCast("cost_int_cast"),
// Xor("cost_xor"),
// Not("cost_not"),
// Eq("cost_eq"),
// Begin("cost_begin"),
// Hash160("cost_hash160"),
// Sha256("cost_sha256"),
// Sha512("cost_sha512"),
// Sha512t256("cost_sha512t256"),
// Keccak256("cost_keccak256"),
// Secp256k1recover("cost_secp256k1recover"),
// Secp256k1verify("cost_secp256k1verify"),
// Print("cost_print"),
// SomeCons("cost_some_cons"),
// OkCons("cost_ok_cons"),
// ErrCons("cost_err_cons"),
// DefaultTo("cost_default_to"),
// UnwrapRet("cost_unwrap_ret"),
// UnwrapErrOrRet("cost_unwrap_err_or_ret"),
// IsOkay("cost_is_okay"),
// IsNone("cost_is_none"),
// IsErr("cost_is_err"),
// IsSome("cost_is_some"),
// Unwrap("cost_unwrap"),
// UnwrapErr("cost_unwrap_err"),
// TryRet("cost_try_ret"),
// Match("cost_match"),
// Append("cost_append"),
// Concat("cost_concat"),
// AsMaxLen("cost_as_max_len"),
// ContractCall("cost_contract_call"),
// ContractOf("cost_contract_of"),
// PrincipalOf("cost_principal_of"),
// AtBlock("cost_at_block"),
// LoadContract("cost_load_contract"),
// CreateMap("cost_create_map"),
// CreateVar("cost_create_var"),
// CreateNft("cost_create_nft"),
// CreateFt("cost_create_ft"),
// FetchEntry("cost_fetch_entry"),
// SetEntry("cost_set_entry"),
// FetchVar("cost_fetch_var"),
// SetVar("cost_set_var"),
// ContractStorage("cost_contract_storage"),
// BlockInfo("cost_block_info"),
// StxBalance("cost_stx_balance"),
// StxTransfer("cost_stx_transfer"),
// FtMint("cost_ft_mint"),
// FtTransfer("cost_ft_transfer"),
// FtBalance("cost_ft_balance"),
// FtSupply("cost_ft_get_supply"),
// FtBurn("cost_ft_burn"),
// NftMint("cost_nft_mint"),
// NftTransfer("cost_nft_transfer"),
// NftOwner("cost_nft_owner"),
// NftBurn("cost_nft_burn"),
// PoisonMicroblock("poison_microblock"),

criterion_main!(benches);
