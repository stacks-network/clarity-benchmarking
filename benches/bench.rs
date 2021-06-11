use std::fs;

use benchmarking_lib::generators::gen;
use blockstack_lib::clarity_vm::database::{MemoryBackingStore, marf::MarfedKV};
use blockstack_lib::clarity_vm::clarity::ClarityInstance;
use blockstack_lib::types::proof::ClarityMarfTrieId;
use blockstack_lib::vm::contexts::{GlobalContext, ContractContext};
use blockstack_lib::vm::database::{NULL_BURN_STATE_DB, NULL_HEADER_DB, HeadersDB, ClarityDatabase};
use blockstack_lib::vm::{ast, eval_all, Value};
use blockstack_lib::vm::costs::cost_functions::ClarityCostFunction;
use blockstack_lib::vm::costs::{LimitedCostTracker, ExecutionCost};
use blockstack_lib::vm::types::QualifiedContractIdentifier;
use blockstack_lib::types::chainstate::{BlockHeaderHash, BurnchainHeaderHash, StacksAddress, StacksBlockId, VRFSeed};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

const INPUT_SIZES: [u16; 8] = [1, 2, 8, 16, 32, 64, 128, 256];
const MORE_INPUT_SIZES: [u16; 12] = [1, 2, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];
const SCALE: u16 = 100;

struct TestHeadersDB;

impl HeadersDB for TestHeadersDB {
    fn get_stacks_block_header_hash_for_block(
        &self,
        id_bhh: &StacksBlockId,
    ) -> Option<BlockHeaderHash> {
        Some(BlockHeaderHash(id_bhh.0.clone()))
    }

    fn get_burn_header_hash_for_block(
        &self,
        id_bhh: &StacksBlockId,
    ) -> Option<BurnchainHeaderHash> {
        Some(BurnchainHeaderHash(id_bhh.0.clone()))
    }

    fn get_vrf_seed_for_block(&self, _id_bhh: &StacksBlockId) -> Option<VRFSeed> {
        Some(VRFSeed([0; 32]))
    }

    fn get_burn_block_time_for_block(&self, _id_bhh: &StacksBlockId) -> Option<u64> {
        Some(1)
    }

    fn get_burn_block_height_for_block(&self, id_bhh: &StacksBlockId) -> Option<u32> {
        if id_bhh == &StacksBlockId::sentinel() {
            Some(0)
        } else {
            let mut bytes = [0; 4];
            bytes.copy_from_slice(&id_bhh.0[0..4]);
            let height = u32::from_le_bytes(bytes);
            Some(height)
        }
    }

    fn get_miner_address(&self, _id_bhh: &StacksBlockId) -> Option<StacksAddress> {
        None
    }
}


fn height_to_hash(burn_height: u64, fork: u64) -> [u8; 32] {
    let mut out = [0; 32];
    out[0..8].copy_from_slice(&burn_height.to_le_bytes());
    out[8..16].copy_from_slice(&fork.to_le_bytes());
    out
}

fn as_hash(inp: u32) -> [u8; 32] {
    let mut out = [0; 32];
    out[0..4].copy_from_slice(&inp.to_le_bytes());
    out
}

fn as_hash160(inp: u32) -> [u8; 20] {
    let mut out = [0; 20];
    out[0..4].copy_from_slice(&inp.to_le_bytes());
    out
}

fn setup_chain_state(scaling: u32) -> MarfedKV {
    let pre_initialized_path = format!("/tmp/clarity_bench_{}.marf", scaling);
    let out_path = "/tmp/clarity_bench_last.marf";

    if fs::metadata(&pre_initialized_path).is_err() {
        let marf = MarfedKV::open(&pre_initialized_path, None).unwrap();
        let mut clarity_instance = ClarityInstance::new(false, marf, ExecutionCost::max_value());
        let mut conn = clarity_instance.begin_test_genesis_block(
            &StacksBlockId::sentinel(),
            &StacksBlockId(as_hash(0)),
            &TestHeadersDB,
            &NULL_BURN_STATE_DB,
        );

        conn.as_transaction(|tx| {
            for j in 0..scaling {
                tx.with_clarity_db(|db| {
                    db.put(format!("key{}", j).as_str(), &Value::none());
                    Ok(())
                })
                .unwrap();
            }
        });

        conn.commit_to_block(&StacksBlockId(as_hash(0)));
    };

    if fs::metadata(&out_path).is_err() {
        fs::create_dir(out_path).unwrap();
    }

    fs::copy(
        &format!("{}/marf.sqlite", pre_initialized_path),
        &format!("{}/marf.sqlite", out_path),
    )
    .unwrap();

    return MarfedKV::open(out_path, None).unwrap();
}

fn bench_with_input_sizes(
    c: &mut Criterion,
    function: ClarityCostFunction,
    scale: u16,
    input_sizes: Vec<u16>,
    use_marf: bool,
) {
    let mut group = c.benchmark_group(function.to_string());

    for input_size in input_sizes.iter() {
        let mut memory_backing_store = MemoryBackingStore::new();

        let mut marf = setup_chain_state(1000);

        let mut marf_store = marf.begin(
            &StacksBlockId(as_hash(0)),
            &StacksBlockId(as_hash(1)),
        );

        let clarity_db = if use_marf {
            marf_store.as_clarity_db(&TestHeadersDB, &NULL_BURN_STATE_DB)
        } else {
            memory_backing_store.as_clarity_db()
        };

        let mut global_context = GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
        global_context.begin();

        let contract_identifier =
            QualifiedContractIdentifier::local(&*format!("c{}", input_size)).unwrap();
        let mut contract_context = ContractContext::new(contract_identifier.clone());

        let contract = gen(function, scale, *input_size);

        let contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };

        group.throughput(Throughput::Bytes(input_size.clone() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(input_size),
            input_size,
            |b, &_| {
                b.iter(|| {
                    global_context.execute(|g| eval_all(&contract_ast.expressions, &mut contract_context, g)).unwrap();
                })
            },
        );
    }
}

fn bench_add(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Add,
        SCALE,
        INPUT_SIZES.into(),
        false,
    )
}

fn bench_sub(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Sub,
        SCALE,
        INPUT_SIZES.into(),
        false,
    )
}

fn bench_le(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Le, SCALE, vec![2], false)
}

fn bench_leq(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Leq, SCALE, vec![2], false)
}

fn bench_ge(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Ge, SCALE, vec![2], false)
}

fn bench_geq(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Geq, SCALE, vec![2], false)
}

// boolean functions
fn bench_and(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::And, SCALE, INPUT_SIZES.into(), false)
}

fn bench_or(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Or, SCALE, INPUT_SIZES.into(), false)
}

fn bench_xor(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Xor, SCALE, vec![2], false)
}

fn bench_not(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Not, SCALE, vec![1], false)
}

// note: only testing is-eq when the values are bools; could try doing it with ints?
fn bench_eq(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Eq, SCALE, INPUT_SIZES.into(), false)
}

fn bench_mod(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Mod, SCALE, vec![2], false)
}

fn bench_pow(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Pow, SCALE, vec![2], false)
}

fn bench_sqrti(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Sqrti, SCALE, vec![1], false)
}

fn bench_log2(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Log2, SCALE, vec![1], false)
}

fn bench_tuple_get(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::TupleGet,
        SCALE,
        MORE_INPUT_SIZES.into(),
        false,
    )
}

fn bench_tuple_merge(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::TupleMerge,
        SCALE,
        INPUT_SIZES.into(),
        false,
    )
}

fn bench_tuple_cons(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::TupleCons,
        SCALE,
        MORE_INPUT_SIZES.into(),
        false,
    )
}

// hash functions
fn bench_hash160(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Hash160, SCALE, vec![1], false)
}

fn bench_sha256(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Sha256, SCALE, vec![1], false)
}

fn bench_sha512(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Sha512, SCALE, vec![1], false)
}

fn bench_sha512t256(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Sha512t256, SCALE, vec![1], false)
}

fn bench_keccak256(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Keccak256, SCALE, vec![1], false)
}

fn bench_secp256k1recover(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Secp256k1recover, SCALE, vec![1], false)
}

fn bench_secp256k1verify(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Secp256k1verify, SCALE, vec![1], false)
}

fn bench_create_ft(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::CreateFt, SCALE.into(), vec![1], false)
}

fn bench_mint_ft(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FtMint, SCALE.into(), vec![1], false)
}

fn bench_ft_transfer(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FtTransfer, SCALE.into(), vec![1])
}

fn bench_ft_balance(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FtBalance, SCALE.into(), vec![1])
}

fn bench_ft_supply(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FtSupply, SCALE.into(), vec![1])
}

fn bench_ft_burn(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FtBurn, SCALE.into(), vec![1])
}

fn bench_create_nft(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::CreateNft, SCALE.into(), vec![1])
}

fn bench_nft_mint(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::NftMint, SCALE.into(), vec![1])
}

fn bench_nft_transfer(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::NftTransfer, SCALE.into(), vec![1])
}

fn bench_nft_owner(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::NftOwner, SCALE.into(), vec![1])
}

fn bench_nft_burn(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::NftBurn, SCALE.into(), vec![1])
}

fn bench_is_none(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IsNone, SCALE.into(), vec![1])
}

fn bench_is_some(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IsSome, SCALE.into(), vec![1])
}

fn bench_is_ok(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IsOkay, SCALE.into(), vec![1])
}

fn bench_is_err(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IsErr, SCALE.into(), vec![1])
}

fn bench_unwrap(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Unwrap, SCALE.into(), vec![1])
}

fn bench_unwrap_ret(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::UnwrapRet, SCALE.into(), vec![1])
}

fn bench_unwrap_err(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::UnwrapErr, SCALE.into(), vec![1])
}

fn bench_unwrap_err_or_ret(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::UnwrapErrOrRet, SCALE.into(), vec![1])
}

fn bench_create_map(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::CreateMap, SCALE.into(), vec![1])
}

fn bench_create_var(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::CreateVar, SCALE.into(), vec![1])
}

criterion_group!(
    benches,
    bench_add,
    bench_sub,
    bench_le,
    bench_leq,
    bench_ge,
    bench_geq,
    bench_and,
    bench_or,
    bench_xor,
    bench_not,
    bench_eq,
    bench_mod,
    bench_pow,
    bench_sqrti,
    bench_log2,
    bench_tuple_get,
    bench_tuple_merge,
    bench_tuple_cons,
    bench_hash160,
    bench_sha256,
    bench_sha512,
    bench_sha512t256,
    bench_keccak256,
    bench_secp256k1recover,
    bench_secp256k1verify,
    // bench_create_ft,
    // bench_ft_mint,
    // bench_ft_transfer,
    // bench_ft_balance,
    // bench_ft_supply,
    // bench_ft_burn,
    // bench_create_nft,
    // bench_nft_mint,
    // bench_nft_transfer,
    // bench_nft_owner,
    // bench_nft_burn,
    bench_is_none,
    bench_is_some,
    bench_is_ok,
    bench_is_err,
    bench_unwrap,
    bench_unwrap_ret,
    bench_unwrap_err,
    bench_unwrap_err_or_ret
    bench_create_map,
    bench_create_var,
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
