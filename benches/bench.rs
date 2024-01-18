use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
#[allow(unused_imports)]
#[allow(unused_variables)]
use std::fs::{self, read, File};
use std::io::Write;
use std::num::ParseIntError;
use std::time::Duration;

use benchmarking_lib::generators::{
    define_dummy_trait, gen, gen_analysis_fetch_contract_entry, gen_analysis_pass,
    gen_read_only_func, helper_gen_clarity_list_type, helper_generate_rand_char_string,
    helper_make_value_for_sized_type_sig, make_sized_contracts_map, make_sized_tuple_sigs_map,
    make_sized_type_sig_map, make_sized_values_map, make_type_sig_list_of_size, GenOutput,
    READ_TIP,
};
use benchmarking_lib::headers_db::{SimHeadersDB, TestHeadersDB};
use criterion::measurement::WallTime;
use criterion::{
    criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion, Throughput,
};
use lazy_static::lazy_static;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use stackslib::address::AddressHashMode;
use stackslib::burnchains::PoxConstants;
use stackslib::chainstate::burn::db::sortdb::SortitionDB;
use stackslib::chainstate::stacks::db::StacksChainState;
use stackslib::chainstate::stacks::{
    CoinbasePayload, StacksBlock, StacksBlockHeader, StacksMicroblock, StacksMicroblockHeader,
    StacksPrivateKey, StacksPublicKey, StacksTransaction, StacksTransactionSigner,
    TransactionAnchorMode, TransactionAuth, TransactionPayload, TransactionVersion,
    C32_ADDRESS_VERSION_TESTNET_SINGLESIG, MINER_BLOCK_CONSENSUS_HASH, MINER_BLOCK_HEADER_HASH,
};
use stackslib::clarity::types::{Address, StacksEpochId};
use stackslib::clarity::vm::analysis::arithmetic_checker::ArithmeticOnlyChecker;
use stackslib::clarity::vm::analysis::read_only_checker::ReadOnlyChecker;
use stackslib::clarity::vm::analysis::trait_checker::TraitChecker;
use stackslib::clarity::vm::analysis::type_checker::contexts::TypingContext;
use stackslib::clarity::vm::analysis::type_checker::v2_1::natives::assets::bench_check_special_mint_asset;
use stackslib::clarity::vm::analysis::type_checker::v2_1::natives::options::{
    bench_analysis_option_check_helper, bench_analysis_option_cons_helper,
    check_special_is_response, check_special_some,
};
use stackslib::clarity::vm::analysis::type_checker::v2_1::natives::sequences::{
    bench_analysis_iterable_function_helper, check_special_map, get_simple_native_or_user_define,
};
use stackslib::clarity::vm::analysis::type_checker::v2_1::natives::{
    bench_analysis_check_tuple_merge_helper, bench_analysis_get_function_entry_in_context,
    bench_analysis_list_items_check_helper, bench_analysis_tuple_cons_helper,
    bench_analysis_tuple_items_check_helper, bench_check_contract_call, check_special_get,
    check_special_let, check_special_list_cons, check_special_merge, check_special_tuple_cons,
    inner_handle_tuple_get,
};
use stackslib::clarity::vm::analysis::type_checker::v2_1::{TypeChecker, _trait_type_size};
use stackslib::clarity::vm::analysis::{
    AnalysisDatabase, AnalysisPass, CheckResult, ContractAnalysis,
};
use stackslib::clarity::vm::ast::definition_sorter::DefinitionSorter;
use stackslib::clarity::vm::ast::expression_identifier::ExpressionIdentifier;
use stackslib::clarity::vm::ast::{build_ast, parser, ASTRules, ContractAST};
use stackslib::clarity::vm::contexts::{ContractContext, GlobalContext, OwnedEnvironment};
use stackslib::clarity::vm::contracts::Contract;
use stackslib::clarity::vm::costs::cost_functions::{AnalysisCostFunction, ClarityCostFunction};
use stackslib::clarity::vm::costs::{CostTracker, ExecutionCost, LimitedCostTracker};
use stackslib::clarity::vm::database::clarity_store::NullBackingStore;
use stackslib::clarity::vm::database::{
    ClarityBackingStore, ClarityDatabase, ClaritySerializable, HeadersDB, NULL_BURN_STATE_DB,
    NULL_HEADER_DB,
};
use stackslib::clarity::vm::functions::crypto::special_principal_of;
use stackslib::clarity::vm::representations::depth_traverse;
use stackslib::clarity::vm::tests::BurnStateDB;
use stackslib::clarity::vm::types::signatures::TypeSignature::{
    BoolType, IntType, NoType, PrincipalType, TupleType, UIntType,
};
use stackslib::clarity::vm::types::signatures::{TupleTypeSignature, TypeSignature};
use stackslib::clarity::vm::types::{
    BufferLength, FunctionSignature, FunctionType, PrincipalData, QualifiedContractIdentifier,
    SequenceSubtype, StandardPrincipalData, TraitIdentifier,
};
use stackslib::clarity::vm::{
    analysis, apply, ast, bench_create_ft_in_context, bench_create_map_in_context,
    bench_create_nft_in_context, bench_create_var_in_context, eval_all, lookup_function,
    lookup_variable, CallStack, ClarityName, ClarityVersion, Environment, LocalContext,
    SymbolicExpression, Value,
};
use stackslib::clarity_vm::clarity::ClarityInstance;
use stackslib::clarity_vm::database::marf::MarfedKV;
use stackslib::clarity_vm::database::MemoryBackingStore;
use stackslib::core::{FIRST_BURNCHAIN_CONSENSUS_HASH, FIRST_STACKS_BLOCK_HASH};
use stackslib::types::chainstate::{
    BlockHeaderHash, BurnchainHeaderHash, StacksAddress, StacksBlockId, StacksWorkScore, VRFSeed,
};
use stackslib::util::hash::{hex_bytes, to_hex, Hash160, MerkleTree, Sha512Trunc256Sum};
use stackslib::util::secp256k1::MessageSignature;
use stackslib::util::vrf::VRFProof;

// for when input size is the number of elements
const INPUT_SIZES: [u64; 8] = [1, 2, 8, 16, 32, 64, 128, 256];
const MORE_INPUT_SIZES: [u64; 12] = [1, 2, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

// for when input size is the size of the data
const INPUT_SIZES_DATA: [u64; 8] = [22, 1000, 40000, 160000, 360000, 640000, 1000000, 1100000];

// for when input size is the size of the data, but with a smaller max value
const INPUT_SIZES_DATA_SMALL: [u64; 8] = [17, 100, 500, 1000, 5000, 10000, 50000, 500000];

// for comparators, which can compare any data of any size
const CMP_INPUT_SIZES: [u64; 16] = [
    1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768,
];

// input sizes for arithmetic functions
const INPUT_SIZES_ARITHMETIC: [u64; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

const INPUT_SIZES_ANALYSIS_PASS: [u64; 6] = [1, 2, 8, 16, 32, 64];

// scaling factor for code generators
const SCALE: u16 = 75;

const SORTITION_MARF_PATH: &str = "./db/epoch3/sortition/";
const CLARITY_MARF_PATH: &str = "./db/epoch3/vm/clarity/";
const CHAINSTATE_PATH: &str = "./db/epoch3/vm/index.sqlite";

lazy_static! {
    pub static ref SIZED_VALUES: HashMap<u64, Value> = make_sized_values_map(INPUT_SIZES.to_vec());
    pub static ref SIZED_CONTRACTS: HashMap<u64, String> =
        make_sized_contracts_map(INPUT_SIZES.to_vec());
    // The size of the TupleTypeSignature is measured by the length of its type map
    pub static ref SIZED_TUPLE_SIG: HashMap<u64, TupleTypeSignature> =
        make_sized_tuple_sigs_map(INPUT_SIZES.to_vec());
    pub static ref SIZED_TYPE_SIG: HashMap<u64, TypeSignature> =
        make_sized_type_sig_map(INPUT_SIZES.to_vec());
    pub static ref TYPE_SIG_LIST: HashMap<u64, Vec<TypeSignature>> =
        make_type_sig_list_of_size(INPUT_SIZES.to_vec());
}

fn eval(
    contract_ast: &ContractAST,
    global_context: &mut GlobalContext,
    contract_context: &mut ContractContext,
) {
    global_context
        .execute(|g| eval_all(&contract_ast.expressions, contract_context, g, None))
        .unwrap();
}

/// Run benchmarks for a list of input sizes
///
/// # Arguments
///
/// * `c` - Criterion instance. Automatically passed in by Criterion `bench` function.
/// * `function` - the Clarity cost function that is being benchmarked
/// * `scale` - a scaling parameter used by the Clarity function code generator
/// * `input_sizes` - an optional list of input sizes. a separate benchmark will be run for each size provided. If None, will be benchmarked as constant size.
/// * `maybe_make_store` - an optional closure taking in a reference to OwnedEnvironment. useful if you want to run a benchmark with pre-loaded state.
fn bench_with_input_sizes<'a>(
    c: &mut Criterion,
    function: ClarityCostFunction,
    scale: u16,
    input_sizes: Option<Vec<u64>>,
    maybe_make_store: Option<Box<dyn Fn(&mut OwnedEnvironment) -> ()>>,
) {
    let mut group = c.benchmark_group(function.to_string());

    match input_sizes {
        Some(sizes) => {
            for input_size in sizes.iter() {
                run_bench(
                    &mut group,
                    function,
                    scale,
                    *input_size,
                    &maybe_make_store,
                    eval,
                )
            }
        }
        None => run_bench(&mut group, function, scale, 1, &maybe_make_store, eval),
    }
}

/// Runs a benchmark for a Clarity function
///
/// Make databases:
//      BITCOIND_TEST=1 cargo test --workspace --bin=stacks-node -- --ignored --nocapture neon_integrations::pox
//      cp -r /tmp/stacks-node-tests/integrations-neon/../neon/burnchain/sortition .
//      cp -r /tmp/stacks-node-tests/integrations-neon/../neon/chainstate/vm/clarity .
//      cp /tmp/stacks-node-tests/integrations-neon/../neon/chainstate/vm/index.sqlite* .
//      echo "select * from marf_data" | sqlite3 db-3/clarity/marf.sqlite
//      pick second to last block hash as block id.
/// # Arguments
///
/// * `group` - Criterion benchmark group.
/// * `function` - the Clarity cost function that is being benchmarked
/// * `scale` - a scaling parameter used by the Clarity function code generator
/// * `input_size` - The input size to pass in to the code generator. Pass in 1 if constant.
/// * `maybe_make_store` - an optional closure taking in a reference to OwnedEnvironment. useful if you want to run a benchmark with pre-loaded state.
/// * `code_to_bench` - a function that will run the generated Clarity code
fn run_bench<'a, F>(
    group: &mut BenchmarkGroup<WallTime>,
    function: ClarityCostFunction,
    scale: u16,
    input_size: u64,
    maybe_make_store: &Option<Box<dyn Fn(&mut OwnedEnvironment) -> ()>>,
    code_to_bench: F,
) where
    F: Fn(&ContractAST, &mut GlobalContext, &mut ContractContext),
{
    // Set up MarfedKV
    let miner_tip = StacksBlockHeader::make_index_block_hash(
        &MINER_BLOCK_CONSENSUS_HASH,
        &MINER_BLOCK_HEADER_HASH,
    );
    let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

    // Set up Clarity Backing Store
    // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
    let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
    let new_tip = StacksBlockId::from([5; 32]);
    let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

    // Set up BurnStateDB
    let pox_constants = PoxConstants::new(
        10,
        5,
        3,
        25,
        5,
        u64::MAX,
        u64::MAX,
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
    );
    // If this line panics, see comment above function for how to make a database
    let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
    let sort_tx = sort_db.index_conn();

    // Set up HeaderDB
    let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();

    // Set up data if necessary
    if let Some(ref make_store) = maybe_make_store {
        let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);
        let mut env = OwnedEnvironment::new_free(false, 0, clarity_db, StacksEpochId::Epoch21);
        make_store(&mut env);
    }

    let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

    let mut global_context = GlobalContext::new(
        false,
        0,
        clarity_db,
        LimitedCostTracker::new_free(),
        StacksEpochId::Epoch21,
    );
    global_context.begin();

    let GenOutput {
        setup: pre_contract_opt,
        body: contract,
        input_size: computed_input_size,
    } = gen(function, scale, input_size);

    let contract_identifier =
        QualifiedContractIdentifier::local(&*format!("c{}", computed_input_size)).unwrap();
    let mut contract_context =
        ContractContext::new(contract_identifier.clone(), ClarityVersion::Clarity2);

    let contract_ast = match ast::build_ast(
        &contract_identifier,
        &contract,
        &mut (),
        ClarityVersion::Clarity2,
        StacksEpochId::Epoch21,
    ) {
        Ok(res) => res,
        Err(error) => {
            panic!("Parsing error: {}", error.diagnostic.message);
        }
    };

    match pre_contract_opt {
        Some(pre_contract) => {
            let pre_contract_identifier =
                QualifiedContractIdentifier::local(&*format!("pre{}", computed_input_size))
                    .unwrap();
            let pre_contract_ast = match ast::build_ast(
                &pre_contract_identifier,
                &pre_contract,
                &mut (),
                ClarityVersion::Clarity2,
                StacksEpochId::Epoch21,
            ) {
                Ok(res) => res,
                Err(error) => {
                    panic!("Parsing error: {}", error.diagnostic.message);
                }
            };
            global_context
                .execute(|g| {
                    eval_all(
                        &pre_contract_ast.expressions,
                        &mut contract_context,
                        g,
                        None,
                    )
                })
                .unwrap();
        }
        _ => {}
    }

    group.throughput(Throughput::Bytes(computed_input_size.clone() as u64));
    group.bench_with_input(
        BenchmarkId::from_parameter(computed_input_size),
        &input_size,
        |b, &_| {
            b.iter(|| {
                code_to_bench(&contract_ast, &mut global_context, &mut contract_context);
            })
        },
    );
}

fn dummy_setup_code(
    _ca: &mut ContractAST,
    _lc: &mut TypingContext,
    _tc: &mut TypeChecker,
    _i: u64,
    _c: &mut LimitedCostTracker,
) {
}

fn bench_analysis<F, G>(
    c: &mut Criterion,
    function: ClarityCostFunction,
    scale: u16,
    input_sizes: Vec<u64>,
    setup_code: G,
    code_to_bench: F,
) where
    F: Fn(&mut ContractAST, &mut TypingContext, &mut TypeChecker, u64, &mut LimitedCostTracker),
    G: Fn(&mut ContractAST, &mut TypingContext, &mut TypeChecker, u64, &mut LimitedCostTracker),
{
    let mut group = c.benchmark_group(function.to_string());

    for input_size in input_sizes.iter() {
        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();

        let GenOutput {
            setup: _,
            body: contract,
            input_size: computed_input_size,
        } = gen(function, scale, *input_size);

        let mut contract_ast = match ast::build_ast(
            &contract_identifier,
            &contract,
            &mut (),
            ClarityVersion::Clarity2,
            StacksEpochId::Epoch21,
        ) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };

        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        let mut local_context =
            TypingContext::new(StacksEpochId::latest(), ClarityVersion::Clarity2);
        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut analysis_db = AnalysisDatabase::new(&mut writeable_marf_store);
        let mut type_checker = TypeChecker::_new(
            &mut analysis_db,
            cost_tracker.clone(),
            &QualifiedContractIdentifier::transient(),
            &ClarityVersion::Clarity2,
        );

        group.throughput(Throughput::Bytes(computed_input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(computed_input_size),
            &computed_input_size,
            |b, &_| {
                setup_code(
                    &mut contract_ast,
                    &mut local_context,
                    &mut type_checker,
                    computed_input_size,
                    &mut cost_tracker,
                );
                b.iter(|| {
                    code_to_bench(
                        &mut contract_ast,
                        &mut local_context,
                        &mut type_checker,
                        computed_input_size,
                        &mut cost_tracker,
                    );
                })
            },
        );
    }
}

fn bench_analysis_pass<F>(c: &mut Criterion, function: AnalysisCostFunction, code_to_bench: F) -> ()
where
    F: Fn(&StacksEpochId, &mut ContractAnalysis, &mut AnalysisDatabase) -> CheckResult<()>,
{
    let mut group = c.benchmark_group(function.to_string());

    for input_size in INPUT_SIZES_ANALYSIS_PASS.iter() {
        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();

        let contract = gen_analysis_pass(function, 1, *input_size).body;
        let contract_size = contract.len();

        let mut contract_ast = match ast::build_ast(
            &contract_identifier,
            &contract,
            &mut (),
            ClarityVersion::Clarity2,
            StacksEpochId::Epoch21,
        ) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };
        let cost_tracker = LimitedCostTracker::new_free();
        let mut contract_analysis = ContractAnalysis::new(
            contract_identifier.clone(),
            contract_ast.expressions.clone(),
            cost_tracker,
            StacksEpochId::latest(),
            ClarityVersion::Clarity2,
        );

        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        let mut analysis_db = AnalysisDatabase::new(&mut writeable_marf_store);

        analysis_db.execute::<_, _, ()>(|db| {
            group.throughput(Throughput::Bytes(contract_size as u64));
            group.bench_with_input(
                BenchmarkId::from_parameter(contract_size),
                &contract_size,
                |b, &_| {
                    b.iter(|| {
                        for _ in 0..SCALE {
                            code_to_bench(&StacksEpochId::latest(), &mut contract_analysis, db);
                        }
                    })
                },
            );

            Ok(())
        });
    }
    ()
}

fn bench_analysis_pass_read_only(c: &mut Criterion) {
    bench_analysis_pass(c, AnalysisCostFunction::ReadOnly, ReadOnlyChecker::run_pass)
}

fn bench_analysis_pass_arithmetic_only_checker(c: &mut Criterion) {
    fn wrapper_arithmetic_checker(
        epoch_id: &StacksEpochId,
        contract_analysis: &mut ContractAnalysis,
        _db: &mut AnalysisDatabase,
    ) -> CheckResult<()> {
        ArithmeticOnlyChecker::run(contract_analysis);
        Ok(())
    }
    bench_analysis_pass(
        c,
        AnalysisCostFunction::ArithmeticOnlyChecker,
        wrapper_arithmetic_checker,
    )
}

fn bench_analysis_pass_trait_checker(c: &mut Criterion) {
    let function = AnalysisCostFunction::TraitChecker;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in INPUT_SIZES_ANALYSIS_PASS.iter() {
        // Parse the setup contract
        let GenOutput {
            setup: setup_opt,
            body: mut contract,
            input_size: computed_input_size,
        } = gen_analysis_pass(function, 1, *input_size);

        let setup_contract = setup_opt.unwrap();
        let pre_contract_identifier =
            QualifiedContractIdentifier::local(&*format!("pre{}", computed_input_size)).unwrap();
        let pre_contract_ast = match ast::build_ast(
            &pre_contract_identifier,
            &setup_contract,
            &mut (),
            ClarityVersion::Clarity2,
            StacksEpochId::Epoch21,
        ) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };
        let cost_tracker = LimitedCostTracker::new_free();
        let mut pre_contract_analysis = ContractAnalysis::new(
            pre_contract_identifier.clone(),
            pre_contract_ast.expressions.clone(),
            cost_tracker,
            StacksEpochId::latest(),
            ClarityVersion::Clarity2,
        );

        // add impl-trait statements
        let principal_data = PrincipalData::Standard(pre_contract_identifier.issuer.clone());
        for i in 0..computed_input_size {
            let impl_trait = format!(
                "(impl-trait '{}.{}.dummy-trait-{}) ",
                principal_data, pre_contract_identifier.name, i
            );
            contract.push_str(&impl_trait);
        }
        let contract_size = contract.len();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_ast = match ast::build_ast(
            &contract_identifier,
            &contract,
            &mut (),
            ClarityVersion::Clarity2,
            StacksEpochId::Epoch21,
        ) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };
        let cost_tracker = LimitedCostTracker::new_free();
        let mut contract_analysis = ContractAnalysis::new(
            contract_identifier.clone(),
            contract_ast.expressions.clone(),
            cost_tracker,
            StacksEpochId::latest(),
            ClarityVersion::Clarity2,
        );

        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        let mut analysis_db = AnalysisDatabase::new(&mut writeable_marf_store);

        // add defined traits to pre contract analysis
        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut type_checker = TypeChecker::_new(
            &mut analysis_db,
            cost_tracker.clone(),
            &QualifiedContractIdentifier::transient(),
            &ClarityVersion::Clarity2,
        );
        let mut typing_context =
            TypingContext::new(StacksEpochId::latest(), ClarityVersion::Clarity2);
        for exp in &pre_contract_ast.expressions {
            type_checker._try_type_check_define(exp, &mut typing_context);
        }
        type_checker
            .contract_context
            .into_contract_analysis(&mut pre_contract_analysis);

        // add implemented traits to contract analysis
        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut type_checker = TypeChecker::_new(
            &mut analysis_db,
            cost_tracker.clone(),
            &QualifiedContractIdentifier::transient(),
            &ClarityVersion::Clarity2,
        );
        let mut typing_context =
            TypingContext::new(StacksEpochId::latest(), ClarityVersion::Clarity2);
        for exp in &contract_ast.expressions {
            type_checker._try_type_check_define(exp, &mut typing_context);
        }
        type_checker
            .contract_context
            .into_contract_analysis(&mut contract_analysis);

        analysis_db.execute::<_, _, ()>(|db| {
            db.insert_contract(&pre_contract_identifier, &pre_contract_analysis);

            group.throughput(Throughput::Bytes(contract_size as u64));
            group.bench_with_input(
                BenchmarkId::from_parameter(contract_size),
                &contract_size,
                |b, &_| {
                    b.iter(|| {
                        for _ in 0..SCALE {
                            TraitChecker::run_pass(
                                &StacksEpochId::latest(),
                                &mut contract_analysis,
                                db,
                            );
                        }
                    })
                },
            );

            Ok(())
        });
    }
    ()
}

fn bench_analysis_pass_type_checker(c: &mut Criterion) {
    let function = AnalysisCostFunction::TypeChecker;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in INPUT_SIZES_ANALYSIS_PASS.iter() {
        // Parse the setup contract
        let GenOutput {
            setup: setup_opt,
            body: mut contract,
            input_size: computed_input_size,
        } = gen_analysis_pass(function, 1, *input_size);

        let setup_contract = setup_opt.unwrap();
        let pre_contract_identifier =
            QualifiedContractIdentifier::local(&*format!("pre{}", input_size)).unwrap();
        let pre_contract_ast = match ast::build_ast(
            &pre_contract_identifier,
            &setup_contract,
            &mut (),
            ClarityVersion::Clarity2,
            StacksEpochId::Epoch21,
        ) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };
        let cost_tracker = LimitedCostTracker::new_free();
        let mut pre_contract_analysis = ContractAnalysis::new(
            pre_contract_identifier.clone(),
            pre_contract_ast.expressions.clone(),
            cost_tracker,
            StacksEpochId::latest(),
            ClarityVersion::Clarity2,
        );

        // add use-trait statements
        let principal_data = PrincipalData::Standard(pre_contract_identifier.issuer.clone());
        for i in 0..computed_input_size {
            let impl_trait = format!(
                "(use-trait dummy-trait-{}-alias '{}.{}.dummy-trait-{}) ",
                i, principal_data, pre_contract_identifier.name, i
            );
            contract.push_str(&impl_trait);
        }
        let contract_size = contract.len();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_ast = match ast::build_ast(
            &contract_identifier,
            &contract,
            &mut (),
            ClarityVersion::Clarity2,
            StacksEpochId::Epoch21,
        ) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };
        let cost_tracker = LimitedCostTracker::new_free();
        let mut contract_analysis = ContractAnalysis::new(
            contract_identifier.clone(),
            contract_ast.expressions.clone(),
            cost_tracker,
            StacksEpochId::latest(),
            ClarityVersion::Clarity2,
        );

        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        let mut analysis_db = AnalysisDatabase::new(&mut writeable_marf_store);

        // add defined traits to pre contract analysis
        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut type_checker = TypeChecker::_new(
            &mut analysis_db,
            cost_tracker.clone(),
            &QualifiedContractIdentifier::transient(),
            &ClarityVersion::Clarity2,
        );
        let mut typing_context =
            TypingContext::new(StacksEpochId::latest(), ClarityVersion::Clarity2);
        for exp in &pre_contract_ast.expressions {
            type_checker._try_type_check_define(exp, &mut typing_context);
        }
        type_checker
            .contract_context
            .into_contract_analysis(&mut pre_contract_analysis);

        analysis_db.execute::<_, _, ()>(|db| {
            db.insert_contract(&pre_contract_identifier, &pre_contract_analysis);

            group.throughput(Throughput::Bytes(contract_size as u64));
            group.bench_with_input(
                BenchmarkId::from_parameter(contract_size),
                &contract_size,
                |b, &_| {
                    b.iter(|| {
                        for _ in 0..SCALE {
                            TypeChecker::run_pass(
                                &StacksEpochId::latest(),
                                &mut contract_analysis,
                                db,
                            );
                        }
                    })
                },
            );

            Ok(())
        });
    }
    ()
}

fn helper_deepen_typing_context(
    i: u64,
    input_size: u64,
    context: &TypingContext,
    group: &mut BenchmarkGroup<WallTime>,
) {
    if i != 0 {
        helper_deepen_typing_context(i - 1, input_size, &context.extend().unwrap(), group);
    } else {
        let mut cost_tracker = LimitedCostTracker::new_free();
        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        let mut analysis_db = AnalysisDatabase::new(&mut writeable_marf_store);

        let mut type_checker = TypeChecker::_new(
            &mut analysis_db,
            cost_tracker.clone(),
            &QualifiedContractIdentifier::transient(),
            &ClarityVersion::Clarity2,
        );

        group.throughput(Throughput::Bytes(input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(input_size),
            &input_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        type_checker.bench_analysis_lookup_variable_depth_helper("dummy", &context);
                    }
                })
            },
        );
    }
}

fn bench_analysis_lookup_variable_depth(c: &mut Criterion) {
    let function = ClarityCostFunction::AnalysisLookupVariableDepth;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        let mut local_context =
            TypingContext::new(StacksEpochId::latest(), ClarityVersion::Clarity2);
        helper_deepen_typing_context(*input_size, *input_size, &local_context, &mut group);
    }
}

fn helper_deepen_local_context(
    i: u64,
    input_size: u64,
    context: &LocalContext,
    group: &mut BenchmarkGroup<WallTime>,
) {
    if i != 0 {
        helper_deepen_local_context(i - 1, input_size, &context.extend().unwrap(), group);
    } else {
        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        // Set up BurnStateDB
        let pox_constants = PoxConstants::new(
            10,
            5,
            3,
            25,
            5,
            u64::MAX,
            u64::MAX,
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
        );
        let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
        let sort_tx = sort_db.index_conn();

        // Set up HeaderDB
        let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();

        let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

        let mut global_context = GlobalContext::new(
            false,
            0,
            clarity_db,
            LimitedCostTracker::new_free(),
            StacksEpochId::Epoch21,
        );
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context =
            ContractContext::new(contract_identifier.clone(), ClarityVersion::Clarity2);
        let mut call_stack = CallStack::new();
        let mut environment = Environment::new(
            global_context.borrow_mut(),
            &contract_context,
            &mut call_stack,
            None,
            None,
            None,
        );

        group.throughput(Throughput::Bytes(input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(input_size),
            &input_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        lookup_variable("dummy", &context, &mut environment);
                    }
                })
            },
        );
    }
}

fn bench_lookup_variable_depth(c: &mut Criterion) {
    let function = ClarityCostFunction::LookupVariableDepth;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        let mut local_context = LocalContext::new();
        helper_deepen_local_context(*input_size, *input_size, &local_context, &mut group);
    }
}

// note: could write `bench_run` function, and split out adding nodes to the graph from finding dependencies
// FIXME
// Fails because `computed_input_size` is 0 on each iteration
fn bench_ast_cycle_detection(c: &mut Criterion) {
    let function = ClarityCostFunction::AstCycleDetection;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();

        let GenOutput {
            setup: _,
            body: mut contract,
            input_size: computed_input_size,
        } = gen(function, 1, *input_size);

        let pre_expressions = parser::v2::parse(&contract).unwrap();
        let mut contract_ast = ContractAST::new(contract_identifier.clone(), pre_expressions);
        ExpressionIdentifier::run_pre_expression_pass(&mut contract_ast, ClarityVersion::Clarity2)
            .unwrap();

        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut def_sorter = DefinitionSorter::_new();

        group.throughput(Throughput::Bytes(computed_input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(computed_input_size),
            &computed_input_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        def_sorter.clear_graph();
                        def_sorter.run(
                            &mut contract_ast,
                            &mut cost_tracker,
                            ClarityVersion::Clarity2,
                        );
                    }
                })
            },
        );
    }
}

fn bench_contract_storage(c: &mut Criterion) {
    let function = ClarityCostFunction::ContractStorage;
    let mut group = c.benchmark_group(function.to_string());
    let mut rng = rand::thread_rng();

    for input_size in &INPUT_SIZES {
        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        // Set up BurnStateDB
        let pox_constants = PoxConstants::new(
            10,
            5,
            3,
            25,
            5,
            u64::MAX,
            u64::MAX,
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
        );
        let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
        let sort_tx = sort_db.index_conn();

        // Set up HeaderDB
        let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();

        let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

        let mut global_context = GlobalContext::new(
            false,
            0,
            clarity_db,
            LimitedCostTracker::new_free(),
            StacksEpochId::Epoch21,
        );
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let contract_context =
            ContractContext::new(contract_identifier.clone(), ClarityVersion::Clarity2);

        let GenOutput {
            setup: _,
            body: contract,
            input_size: computed_input_size,
        } = gen(function, 1, *input_size);

        let contract_ast = match ast::build_ast(
            &contract_identifier,
            &contract,
            &mut (),
            ClarityVersion::Clarity2,
            StacksEpochId::Epoch21,
        ) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };

        let mut call_stack = CallStack::new();

        let mut environment = Environment::new(
            global_context.borrow_mut(),
            &contract_context,
            &mut call_stack,
            None,
            None,
            None,
        );

        group.throughput(Throughput::Bytes(computed_input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(computed_input_size),
            &computed_input_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        let contract_identifier =
                            QualifiedContractIdentifier::local(&*format!("c{}", rng.gen::<u32>()))
                                .unwrap();
                        environment.initialize_contract_from_ast(
                            contract_identifier.clone(),
                            ClarityVersion::Clarity2,
                            &contract_ast,
                            &contract,
                        );
                    }
                })
            },
        );
    }
}

fn bench_principal_of(c: &mut Criterion) {
    let function = ClarityCostFunction::PrincipalOf;
    let mut group = c.benchmark_group(function.to_string());

    // Set up MarfedKV
    let miner_tip = StacksBlockHeader::make_index_block_hash(
        &MINER_BLOCK_CONSENSUS_HASH,
        &MINER_BLOCK_HEADER_HASH,
    );
    let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

    // Set up Clarity Backing Store
    // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
    let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
    let new_tip = StacksBlockId::from([5; 32]);
    let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

    // Set up BurnStateDB
    let pox_constants = PoxConstants::new(
        10,
        5,
        3,
        25,
        5,
        u64::MAX,
        u64::MAX,
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
    );
    let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
    let sort_tx = sort_db.index_conn();

    // Set up HeaderDB
    let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();
    let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

    let mut global_context = GlobalContext::new(
        false,
        0,
        clarity_db,
        LimitedCostTracker::new_free(),
        StacksEpochId::Epoch21,
    );
    global_context.begin();

    let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
    let contract_context =
        ContractContext::new(contract_identifier.clone(), ClarityVersion::Clarity2);

    let GenOutput {
        setup: _,
        body: contract,
        input_size: _,
    } = gen(function, SCALE, 1);

    let contract_ast = match ast::build_ast(
        &contract_identifier,
        &contract,
        &mut (),
        ClarityVersion::Clarity2,
        StacksEpochId::Epoch21,
    ) {
        Ok(res) => res,
        Err(error) => {
            panic!("Parsing error: {}", error.diagnostic.message);
        }
    };

    let mut call_stack = CallStack::new();
    let mut environment = Environment::new(
        global_context.borrow_mut(),
        &contract_context,
        &mut call_stack,
        None,
        None,
        None,
    );
    let local_context = LocalContext::new();

    group.throughput(Throughput::Bytes(0));
    group.bench_with_input(BenchmarkId::from_parameter(0), &0, |b, &_| {
        b.iter(|| {
            for expr in &contract_ast.expressions {
                special_principal_of(&[expr.clone()], &mut environment, &local_context);
            }
        })
    });
}

// FIXME
// Test panics while setting `trait_sig`
// Fix panic and re-enable test
fn bench_analysis_use_trait_entry(c: &mut Criterion) {
    let function = ClarityCostFunction::AnalysisUseTraitEntry;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);
        let mut analysis_db = AnalysisDatabase::new(&mut writeable_marf_store);

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();

        let cost_tracker = LimitedCostTracker::new_free();
        let type_checker = TypeChecker::_new(
            &mut analysis_db,
            cost_tracker.clone(),
            &contract_identifier,
            &ClarityVersion::Clarity2,
        );

        let GenOutput {
            setup: _,
            body: contract,
            input_size: _,
        } = gen(function, 1, *input_size);

        let contract_ast = match ast::build_ast(
            &contract_identifier,
            &contract,
            &mut (),
            ClarityVersion::Clarity2,
            StacksEpochId::Epoch21,
        ) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };

        let cost_tracker = LimitedCostTracker::new_free();
        let mut contract_analysis = ContractAnalysis::new(
            contract_identifier.clone(),
            contract_ast.expressions.clone(),
            cost_tracker,
            StacksEpochId::latest(),
            ClarityVersion::Clarity2,
        );

        /*
        let mut typing_context =
            TypingContext::new(StacksEpochId::latest(), ClarityVersion::Clarity2);
        type_checker
            ._type_check_define_trait(&trait_name, &contract_ast.expressions, &mut typing_context)
            .unwrap();
        */

        type_checker
            .contract_context
            .into_contract_analysis(&mut contract_analysis);

        type_checker
            .db
            .execute(|db| {
                db.insert_contract(&contract_identifier, &contract_analysis)
                    .unwrap();
                let trait_name = ClarityName::try_from("dummy-trait".to_string()).unwrap();
                let trait_id = TraitIdentifier {
                    name: trait_name.clone(),
                    contract_identifier: contract_identifier.clone(),
                };

                // get the size of the trait
                let trait_sig = db
                    .get_defined_trait(&contract_identifier, &trait_name, &StacksEpochId::latest())
                    .expect("FATAL: could not load from DB")
                    .expect("FATAL: could not unwrap");
                let type_size = _trait_type_size(&trait_sig).unwrap();

                group.throughput(Throughput::Bytes(type_size));
                group.bench_with_input(
                    BenchmarkId::from_parameter(type_size),
                    &type_size,
                    |b, &_| {
                        b.iter(|| {
                            for _ in 0..SCALE {
                                TypeChecker::bench_analysis_use_trait_entry_in_context(
                                    db, &trait_id,
                                );
                            }
                        })
                    },
                );
                // this snippet is here since the "execute" context needs to determine the return type
                if false {
                    return Err(());
                }

                Ok(())
            })
            .unwrap();
    }
}

fn bench_analysis_get_function_entry(c: &mut Criterion) {
    let function = ClarityCostFunction::AnalysisGetFunctionEntry;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        let mut analysis_db = AnalysisDatabase::new(&mut writeable_marf_store);
        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut type_checker = TypeChecker::_new(
            &mut analysis_db,
            cost_tracker.clone(),
            &QualifiedContractIdentifier::transient(),
            &ClarityVersion::Clarity2,
        );

        let GenOutput {
            setup: _,
            body: contract,
            input_size: _,
        } = gen(function, 1, *input_size);

        let mut contract_identifier =
            QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_ast = match ast::build_ast(
            &contract_identifier,
            &contract,
            &mut (),
            ClarityVersion::Clarity2,
            StacksEpochId::Epoch21,
        ) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };

        let cost_tracker = LimitedCostTracker::new_free();
        let mut contract_analysis = ContractAnalysis::new(
            contract_identifier.clone(),
            contract_ast.expressions.clone(),
            cost_tracker,
            StacksEpochId::latest(),
            ClarityVersion::Clarity2,
        );

        let mut typing_context =
            TypingContext::new(StacksEpochId::latest(), ClarityVersion::Clarity2);
        type_checker._try_type_check_define(&contract_ast.expressions[0], &mut typing_context);
        type_checker
            .contract_context
            .into_contract_analysis(&mut contract_analysis);

        type_checker.db.execute(|db| {
            db.insert_contract(&contract_identifier, &contract_analysis);
            let fn_name = ClarityName::try_from("dummy-fn".to_string()).unwrap();
            let type_size = match db
                .get_read_only_function_type(
                    &contract_identifier,
                    "dummy-fn",
                    &StacksEpochId::latest(),
                )
                .unwrap()
            {
                Some(FunctionType::Fixed(function)) => {
                    let func_sig = FunctionSignature::from(function);
                    func_sig.total_type_size().unwrap()
                }
                _ => panic!("unexpected"),
            };

            group.throughput(Throughput::Bytes(type_size));
            group.bench_with_input(
                BenchmarkId::from_parameter(type_size),
                &type_size,
                |b, &_| {
                    b.iter(|| {
                        for _ in 0..SCALE {
                            bench_analysis_get_function_entry_in_context(
                                db,
                                &contract_identifier,
                                &fn_name,
                            );
                        }
                    })
                },
            );
            // this snippet is here since the "execute" context needs to determine the return type
            if false {
                return Err(());
            }

            Ok(())
        });
    }
}

fn bench_inner_type_check_cost(c: &mut Criterion) {
    let function = ClarityCostFunction::InnerTypeCheckCost;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        // Set up BurnStateDB
        let pox_constants = PoxConstants::new(
            10,
            5,
            3,
            25,
            5,
            u64::MAX,
            u64::MAX,
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
        );
        let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
        let sort_tx = sort_db.index_conn();

        // Set up HeaderDB
        let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();
        let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

        let mut global_context = GlobalContext::new(
            false,
            0,
            clarity_db,
            LimitedCostTracker::new_free(),
            StacksEpochId::Epoch21,
        );
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context =
            ContractContext::new(contract_identifier.clone(), ClarityVersion::Clarity2);

        let GenOutput {
            setup: _,
            body: contract,
            input_size: _,
        } = gen(function, 1, *input_size);

        let mut contract_ast = match ast::build_ast(
            &contract_identifier,
            &contract,
            &mut (),
            ClarityVersion::Clarity2,
            StacksEpochId::Epoch21,
        ) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };

        global_context
            .execute(|g| eval_all(&contract_ast.expressions, &mut contract_context, g, None))
            .unwrap();

        let defined_fn = contract_context.lookup_function("dummy-fn").unwrap();
        let arg_list = [SIZED_VALUES.get(input_size).unwrap().clone()];

        group.throughput(Throughput::Bytes(*input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(input_size),
            &input_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        defined_fn.bench_execute_apply(&arg_list);
                    }
                })
            },
        );
    }
}

fn bench_user_function_application(c: &mut Criterion) {
    let function = ClarityCostFunction::UserFunctionApplication;
    let mut group = c.benchmark_group(function.to_string());
    let mut rng = rand::thread_rng();

    for input_size in &INPUT_SIZES {
        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        // Set up BurnStateDB
        let pox_constants = PoxConstants::new(
            10,
            5,
            3,
            25,
            5,
            u64::MAX,
            u64::MAX,
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
        );
        let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
        let sort_tx = sort_db.index_conn();

        // Set up HeaderDB
        let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();

        let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

        let mut global_context = GlobalContext::new(
            false,
            0,
            clarity_db,
            LimitedCostTracker::new_free(),
            StacksEpochId::Epoch21,
        );
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context =
            ContractContext::new(contract_identifier.clone(), ClarityVersion::Clarity2);

        let GenOutput {
            setup: _,
            body: contract,
            input_size: computed_input_size,
        } = gen(function, 1, *input_size);

        let contract_ast = match ast::build_ast(
            &contract_identifier,
            &contract,
            &mut (),
            ClarityVersion::Clarity2,
            StacksEpochId::Epoch21,
        ) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };

        global_context
            .execute(|g| eval_all(&contract_ast.expressions, &mut contract_context, g, None))
            .unwrap();

        let defined_fn = contract_context.lookup_function("dummy-fn").unwrap();
        let mut arg_list = Vec::new();
        for _ in 0..computed_input_size {
            arg_list.push(Value::UInt(rng.gen()));
        }

        group.throughput(Throughput::Bytes(computed_input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(computed_input_size),
            &computed_input_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        defined_fn.bench_execute_apply(&arg_list).unwrap();
                    }
                })
            },
        );
    }
}

fn bench_analysis_lookup_function_types(c: &mut Criterion) {
    let function = ClarityCostFunction::AnalysisLookupFunctionTypes;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        // Set up BurnStateDB
        let pox_constants = PoxConstants::new(
            10,
            5,
            3,
            25,
            5,
            u64::MAX,
            u64::MAX,
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
        );
        let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
        let sort_tx = sort_db.index_conn();

        // Set up HeaderDB
        let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();
        let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

        let mut global_context = GlobalContext::new(
            false,
            0,
            clarity_db,
            LimitedCostTracker::new_free(),
            StacksEpochId::Epoch21,
        );
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context =
            ContractContext::new(contract_identifier.clone(), ClarityVersion::Clarity2);

        let GenOutput {
            setup: _,
            body: contract,
            input_size: _,
        } = gen(function, 1, *input_size);

        let mut contract_ast = match ast::build_ast(
            &contract_identifier,
            &contract,
            &mut (),
            ClarityVersion::Clarity2,
            StacksEpochId::Epoch21,
        ) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };
        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut null_store = NullBackingStore::new();
        let mut analysis_db = null_store.as_analysis_db();
        let mut type_checker = TypeChecker::_new(
            &mut analysis_db,
            cost_tracker.clone(),
            &QualifiedContractIdentifier::transient(),
            &ClarityVersion::Clarity2,
        );
        global_context
            .execute(|g| eval_all(&contract_ast.expressions, &mut contract_context, g, None))
            .unwrap();

        let trait_obj = contract_context
            .lookup_trait_definition("dummy-trait")
            .unwrap();
        // get size of function signature
        let fn_name = ClarityName::from("dummy-fn");
        let func_signature = trait_obj.get(&*fn_name).unwrap();
        let curr_size = func_signature.total_type_size().unwrap();

        // add trait to the contract context of the type checker
        let trait_clarity_name = ClarityName::from("dummy-trait");
        type_checker
            .contract_context
            .add_defined_trait(trait_clarity_name.clone(), trait_obj);

        // construct trait id
        let trait_id = TraitIdentifier {
            contract_identifier,
            name: trait_clarity_name,
        };

        group.throughput(Throughput::Bytes(curr_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(curr_size),
            &curr_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        bench_check_contract_call(&mut type_checker, &trait_id, &fn_name);
                    }
                })
            },
        );
    }
}

fn bench_lookup_function(c: &mut Criterion) {
    let function = ClarityCostFunction::LookupFunction;
    let mut group = c.benchmark_group(function.to_string());

    // Set up MarfedKV
    let miner_tip = StacksBlockHeader::make_index_block_hash(
        &MINER_BLOCK_CONSENSUS_HASH,
        &MINER_BLOCK_HEADER_HASH,
    );
    let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

    // Set up Clarity Backing Store
    // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
    let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
    let new_tip = StacksBlockId::from([5; 32]);
    let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

    // Set up BurnStateDB
    let pox_constants = PoxConstants::new(
        10,
        5,
        3,
        25,
        5,
        u64::MAX,
        u64::MAX,
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
    );
    let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
    let sort_tx = sort_db.index_conn();

    // Set up HeaderDB
    let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();

    let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

    let mut global_context = GlobalContext::new(
        false,
        0,
        clarity_db,
        LimitedCostTracker::new_free(),
        StacksEpochId::Epoch21,
    );
    global_context.begin();

    let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
    let mut contract_context =
        ContractContext::new(contract_identifier.clone(), ClarityVersion::Clarity2);

    let GenOutput {
        setup: _,
        body: contract,
        input_size: _,
    } = gen(function, SCALE, 1);

    let mut contract_ast = match ast::build_ast(
        &contract_identifier,
        &contract,
        &mut (),
        ClarityVersion::Clarity2,
        StacksEpochId::Epoch21,
    ) {
        Ok(res) => res,
        Err(error) => {
            panic!("Parsing error: {}", error.diagnostic.message);
        }
    };
    global_context
        .execute(|g| eval_all(&contract_ast.expressions, &mut contract_context, g, None))
        .unwrap();

    let mut call_stack = CallStack::new();
    let mut environment = Environment::new(
        global_context.borrow_mut(),
        &contract_context,
        &mut call_stack,
        None,
        None,
        None,
    );

    let mut rng = rand::thread_rng();
    let mut fn_names = Vec::new();
    for i in 0..SCALE {
        match rng.gen_range(0..3) {
            0 => fn_names.push("nonsense".to_string()),
            1 => fn_names.push(format!("fn-{}", i)),
            2 => fn_names.push("no-op".to_string()),
            _ => unimplemented!(),
        }
    }

    group.throughput(Throughput::Bytes(0));
    group.bench_with_input(BenchmarkId::from_parameter(0), &0, |b, &_| {
        b.iter(|| {
            for name in &fn_names {
                lookup_function(name, &mut environment);
            }
        })
    });
}

fn bench_lookup_variable_size(c: &mut Criterion) {
    let function = ClarityCostFunction::LookupVariableSize;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        // Set up BurnStateDB
        let pox_constants = PoxConstants::new(
            10,
            5,
            3,
            25,
            5,
            u64::MAX,
            u64::MAX,
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
        );
        let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
        let sort_tx = sort_db.index_conn();

        // Set up HeaderDB
        let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();

        let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

        let mut global_context = GlobalContext::new(
            false,
            0,
            clarity_db,
            LimitedCostTracker::new_free(),
            StacksEpochId::Epoch21,
        );
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context =
            ContractContext::new(contract_identifier.clone(), ClarityVersion::Clarity2);

        let mut call_stack = CallStack::new();

        let mut environment = Environment::new(
            global_context.borrow_mut(),
            &contract_context,
            &mut call_stack,
            None,
            None,
            None,
        );
        let mut local_context = LocalContext::new();
        let inner_val = SIZED_VALUES.get(input_size).unwrap();
        let val_name = "dummy";
        let clar_val_name = ClarityName::try_from(val_name.to_string()).unwrap();
        local_context
            .variables
            .insert(clar_val_name, inner_val.clone());

        // add more values to the local context
        for _ in 0..1000 {
            let name = ClarityName::try_from(helper_generate_rand_char_string(10)).unwrap();
            local_context.variables.insert(name, inner_val.clone());
        }

        group.throughput(Throughput::Bytes(*input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(input_size),
            &input_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        lookup_variable(val_name, &local_context, &mut environment);
                    }
                })
            },
        );
    }
}

/// ////////////////////////////////////
/// ANALYSIS FUNCTIONS
/// ////////////////////////////////////

fn bench_analysis_option_cons(c: &mut Criterion) {
    fn eval_check_special_some(
        _ast: &mut ContractAST,
        _lc: &mut TypingContext,
        _tc: &mut TypeChecker,
        _i: u64,
        _c: &mut LimitedCostTracker,
    ) {
        for _ in 0..SCALE {
            bench_analysis_option_cons_helper(TypeSignature::BoolType);
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisOptionCons,
        SCALE,
        vec![1],
        dummy_setup_code,
        eval_check_special_some,
    )
}

fn bench_analysis_option_check(c: &mut Criterion) {
    fn eval_check_special_is_response(
        _ast: &mut ContractAST,
        _lc: &mut TypingContext,
        _tc: &mut TypeChecker,
        _i: u64,
        _c: &mut LimitedCostTracker,
    ) {
        for _ in 0..SCALE {
            bench_analysis_option_check_helper(TypeSignature::ResponseType(Box::new((
                TypeSignature::BoolType,
                TypeSignature::BoolType,
            ))));
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisOptionCheck,
        SCALE,
        vec![1],
        dummy_setup_code,
        eval_check_special_is_response,
    )
}

// Cost of the match statement in inner_type_check - doesn't include cost of calls from the match
fn bench_analysis_visit(c: &mut Criterion) {
    fn eval_type_check(
        contract_ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _i: u64,
        _c: &mut LimitedCostTracker,
    ) {
        for exp in &contract_ast.expressions {
            type_checker.bench_analysis_visit_helper(exp, &local_context);
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisVisit,
        SCALE,
        vec![1],
        dummy_setup_code,
        eval_type_check,
    )
}

fn bench_analysis_bind_name(c: &mut Criterion) {
    fn eval_type_check_define<T: CostTracker>(
        _ast: &mut ContractAST,
        _lc: &mut TypingContext,
        type_checker: &mut TypeChecker,
        input_size: u64,
        _c: &mut T,
    ) {
        type_checker.contract_context.clear_variable_types();
        let type_sig = SIZED_TYPE_SIG.get(&input_size).unwrap();
        for _ in 0..SCALE {
            type_checker.bench_analysis_bind_name_helper(type_sig.clone());
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisBindName,
        SCALE,
        INPUT_SIZES.into(),
        dummy_setup_code,
        eval_type_check_define,
    )
}

fn bench_analysis_list_items_check(c: &mut Criterion) {
    fn eval_check_special_list_cons<T: CostTracker>(
        _ast: &mut ContractAST,
        _lc: &mut TypingContext,
        _tc: &mut TypeChecker,
        input_size: u64,
        _c: &mut T,
    ) {
        let type_sig_list = vec![SIZED_TYPE_SIG.get(&input_size).unwrap().clone()];
        for _ in 0..SCALE {
            bench_analysis_list_items_check_helper(&*type_sig_list);
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisListItemsCheck,
        SCALE,
        INPUT_SIZES.into(),
        dummy_setup_code,
        eval_check_special_list_cons,
    )
}

fn bench_analysis_check_tuple_get(c: &mut Criterion) {
    // SIZED_TUPLE_SIG is a lazy static. This setup function makes sur eit is initialized before
    // the benchmarking function is called.
    fn setup_fn<T: CostTracker>(
        _ca: &mut ContractAST,
        _lc: &mut TypingContext,
        _tc: &mut TypeChecker,
        i: u64,
        _c: &mut T,
    ) {
        SIZED_TUPLE_SIG.get(&i);
    }

    fn eval_check_special_get<T: CostTracker>(
        _ca: &mut ContractAST,
        _lc: &mut TypingContext,
        type_checker: &mut TypeChecker,
        i: u64,
        _c: &mut T,
    ) {
        let tuple_type_sig = SIZED_TUPLE_SIG.get(&i).unwrap();
        inner_handle_tuple_get(tuple_type_sig, "id0", type_checker);
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisCheckTupleGet,
        SCALE,
        INPUT_SIZES.into(),
        setup_fn,
        eval_check_special_get,
    )
}

fn bench_analysis_check_tuple_merge(c: &mut Criterion) {
    fn eval_check_special_merge<T: CostTracker>(
        _ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        input_size: u64,
        _c: &mut T,
    ) {
        let sized_tuple_sig =
            TypeSignature::TupleType(SIZED_TUPLE_SIG.get(&input_size).unwrap().clone());
        for _ in 0..SCALE {
            bench_analysis_check_tuple_merge_helper(
                type_checker,
                sized_tuple_sig.clone(),
                sized_tuple_sig.clone(),
                local_context,
            );
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisCheckTupleMerge,
        SCALE,
        INPUT_SIZES.into(),
        dummy_setup_code,
        eval_check_special_merge,
    )
}

fn bench_analysis_check_tuple_cons(c: &mut Criterion) {
    fn eval_check_special_tuple_cons<T: CostTracker>(
        contract_ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _is: u64,
        _c: &mut T,
    ) {
        type_checker.type_map.delete_all();
        for exp in &contract_ast.expressions {
            let exp_list = exp.match_list().unwrap();
            bench_analysis_tuple_cons_helper(type_checker, exp_list, local_context);
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisCheckTupleCons,
        SCALE,
        INPUT_SIZES.into(),
        dummy_setup_code,
        eval_check_special_tuple_cons,
    )
}

fn bench_analysis_tuple_items_check(c: &mut Criterion) {
    fn eval_check_special_tuple_cons<T: CostTracker>(
        _ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        input_size: u64,
        _c: &mut T,
    ) {
        let type_sig = SIZED_TYPE_SIG.get(&input_size).unwrap();
        for _ in 0..SCALE {
            bench_analysis_tuple_items_check_helper(type_checker, type_sig.clone(), local_context);
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisTupleItemsCheck,
        SCALE,
        INPUT_SIZES.into(),
        dummy_setup_code,
        eval_check_special_tuple_cons,
    )
}

fn bench_analysis_check_let(c: &mut Criterion) {
    fn eval_check_special_let<T: CostTracker>(
        _ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        input_size: u64,
        _c: &mut T,
    ) {
        let type_sig_list = TYPE_SIG_LIST.get(&input_size).unwrap();
        for _ in 0..SCALE {
            type_checker.bench_analysis_check_let_helper(type_sig_list.clone(), local_context);
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisCheckLet,
        SCALE,
        INPUT_SIZES.into(),
        dummy_setup_code,
        eval_check_special_let,
    )
}

fn bench_analysis_lookup_function(c: &mut Criterion) {
    let function = ClarityCostFunction::AnalysisLookupFunction;
    let mut group = c.benchmark_group(function.to_string());

    let mut cost_tracker = LimitedCostTracker::new_free();
    let mut null_store = NullBackingStore::new();
    let mut analysis_db = null_store.as_analysis_db();
    let mut type_checker = TypeChecker::_new(
        &mut analysis_db,
        cost_tracker.clone(),
        &QualifiedContractIdentifier::transient(),
        &ClarityVersion::Clarity2,
    );

    let mut rng = rand::thread_rng();
    let mut fn_names = Vec::new();
    for _ in 0..SCALE {
        let fn_name = match rng.gen_range(0..(3 as i8)) {
            0 => {
                // return simple native function
                ["pow", "mod", "xor"].choose(&mut rng).unwrap().clone()
            }
            1 => {
                // return special native function
                ["if", "map", "filter"].choose(&mut rng).unwrap().clone()
            }
            2 => {
                // return non-existant function
                ["efdf", "suddsfb", "apod"]
                    .choose(&mut rng)
                    .unwrap()
                    .clone()
            }
            _ => unreachable!(),
        };
        fn_names.push(fn_name.to_string());
    }

    group.throughput(Throughput::Bytes(0));
    group.bench_with_input(BenchmarkId::from_parameter(0), &0, |b, &_| {
        b.iter(|| {
            for fn_name in &fn_names {
                get_simple_native_or_user_define(fn_name, &mut type_checker);
            }
        })
    });
}

fn bench_analysis_type_annotate(c: &mut Criterion) {
    fn setup_fn(
        contract_ast: &mut ContractAST,
        _lc: &mut TypingContext,
        type_checker: &mut TypeChecker,
        input_size: u64,
        _c: &mut LimitedCostTracker,
    ) {
        let var_type_sig = SIZED_TYPE_SIG.get(&input_size).unwrap();
        for exp in &contract_ast.expressions {
            let var_name = exp.match_atom().unwrap();
            type_checker
                .contract_context
                .add_variable_type(var_name.clone(), var_type_sig.clone());
        }
    }

    fn eval_inner_type_check<T: CostTracker>(
        contract_ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        input_size: u64,
        _c: &mut T,
    ) {
        type_checker.type_map.delete_all();
        let var_type_sig = SIZED_TYPE_SIG.get(&input_size).unwrap();
        for expr in &contract_ast.expressions {
            type_checker.type_map.set_type(expr, var_type_sig.clone());
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisTypeAnnotate,
        SCALE,
        INPUT_SIZES.into(),
        setup_fn,
        eval_inner_type_check,
    )
}

fn bench_analysis_type_check(c: &mut Criterion) {
    fn setup_fn<T: CostTracker>(
        _ca: &mut ContractAST,
        _lc: &mut TypingContext,
        type_checker: &mut TypeChecker,
        i: u64,
        _c: &mut T,
    ) {
        let tuple_type_sig = SIZED_TYPE_SIG.get(&i).unwrap().clone();
        type_checker.set_function_return_tracker(Some(Some(tuple_type_sig.clone())));
    }

    fn eval_track_return_type<T: CostTracker>(
        _ca: &mut ContractAST,
        _lc: &mut TypingContext,
        type_checker: &mut TypeChecker,
        i: u64,
        _c: &mut T,
    ) {
        let tuple_type_sig = SIZED_TYPE_SIG.get(&i).unwrap().clone();
        for _ in 0..SCALE {
            type_checker.track_return_type(tuple_type_sig.clone());
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisTypeCheck,
        100,
        INPUT_SIZES.into(),
        setup_fn,
        eval_track_return_type,
    )
}

fn bench_analysis_iterable_func(c: &mut Criterion) {
    let function = ClarityCostFunction::AnalysisIterableFunc;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        let type_sig_list = vec![
            TypeSignature::SequenceType(SequenceSubtype::BufferType(
                BufferLength::try_from(15u32).unwrap()
            ));
            *input_size as usize
        ];

        let mut local_context =
            TypingContext::new(StacksEpochId::latest(), ClarityVersion::Clarity2);

        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut null_store = NullBackingStore::new();
        let mut analysis_db = null_store.as_analysis_db();
        let mut type_checker = TypeChecker::_new(
            &mut analysis_db,
            cost_tracker.clone(),
            &QualifiedContractIdentifier::transient(),
            &ClarityVersion::Clarity2,
        );

        group.throughput(Throughput::Bytes(*input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(input_size),
            &input_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        bench_analysis_iterable_function_helper(
                            &mut type_checker,
                            &type_sig_list,
                            &mut local_context,
                        );
                    }
                })
            },
        );
    }
}

// this is the cost of storing the contract - measure contract analysis serialization
fn bench_analysis_storage(c: &mut Criterion) {
    let function = ClarityCostFunction::AnalysisStorage;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();

        let GenOutput {
            setup: _,
            body: contract,
            input_size: _,
        } = gen(function, SCALE, *input_size);

        let mut contract_ast = match ast::build_ast(
            &contract_identifier,
            &contract,
            &mut (),
            ClarityVersion::Clarity2,
            StacksEpochId::Epoch21,
        ) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };

        // use warmed up marf
        let mut cost_tracker = LimitedCostTracker::new_free();
        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        let mut analysis_db = AnalysisDatabase::new(&mut writeable_marf_store);
        let mut type_checker = TypeChecker::_new(
            &mut analysis_db,
            cost_tracker.clone(),
            &QualifiedContractIdentifier::transient(),
            &ClarityVersion::Clarity2,
        );

        let mut contract_analyses = Vec::new();
        for exp in &contract_ast.expressions {
            let contract_id = QualifiedContractIdentifier::local("analysis_test").unwrap();
            let exp_list = exp.match_list().unwrap();
            let mut contract_analysis = ContractAnalysis::new(
                contract_id.clone(),
                exp_list.to_vec(),
                cost_tracker.clone(),
                StacksEpochId::latest(),
                ClarityVersion::Clarity2,
            );

            let mut type_checker = TypeChecker::_new(
                &mut analysis_db,
                cost_tracker.clone(),
                &QualifiedContractIdentifier::transient(),
                &ClarityVersion::Clarity2,
            );
            let mut typing_context =
                TypingContext::new(StacksEpochId::latest(), ClarityVersion::Clarity2);
            for exp in exp_list {
                type_checker._try_type_check_define(exp, &mut typing_context);
            }
            type_checker
                .contract_context
                .into_contract_analysis(&mut contract_analysis);

            contract_analyses.push(contract_analysis);
        }

        let mut size: u64 = 0;
        for exp in contract_analyses[0].expressions.iter() {
            depth_traverse(exp, |_x| match size.checked_add(1) {
                Some(new_size) => {
                    size = new_size;
                    Ok(())
                }
                None => Err("overflow"),
            })
            .unwrap();
        }

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &_| {
            b.iter(|| {
                for analysis in &contract_analyses {
                    // serialize the contract
                    let r = &analysis.serialize();
                }
            })
        });
    }
}

fn bench_analysis_type_lookup(c: &mut Criterion) {
    fn setup_fn(
        contract_ast: &mut ContractAST,
        _lc: &mut TypingContext,
        type_checker: &mut TypeChecker,
        input_size: u64,
        _c: &mut LimitedCostTracker,
    ) {
        let token_type = SIZED_TYPE_SIG.get(&input_size).unwrap();
        for exp in &contract_ast.expressions {
            let exp_list = exp.match_list().unwrap();
            let asset_name = exp_list[0].match_atom().unwrap();
            type_checker
                .contract_context
                .add_nft(asset_name.clone(), token_type.clone());
        }
    }

    fn eval_check_special_mint_asset(
        contract_ast: &mut ContractAST,
        _lc: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _i: u64,
        _c: &mut LimitedCostTracker,
    ) {
        for exp in &contract_ast.expressions {
            let exp_list = exp.match_list().unwrap();
            bench_check_special_mint_asset(type_checker, exp_list);
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisTypeLookup,
        SCALE,
        INPUT_SIZES.into(),
        setup_fn,
        eval_check_special_mint_asset,
    )
}

fn bench_analysis_lookup_variable_const(c: &mut Criterion) {
    fn setup_fn(
        contract_ast: &mut ContractAST,
        _lc: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _is: u64,
        _c: &mut LimitedCostTracker,
    ) {
        let mut rng = rand::thread_rng();
        let type_sig_list = [IntType, BoolType, NoType, PrincipalType, UIntType];
        for exp in &contract_ast.expressions {
            let var_name = exp.match_atom().unwrap();
            let var_type_sig = type_sig_list.choose(&mut rng).unwrap();
            type_checker
                .contract_context
                .add_variable_type(var_name.clone(), var_type_sig.clone());
        }
    }

    fn eval_lookup_variable(
        _ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _i: u64,
        _c: &mut LimitedCostTracker,
    ) {
        for i in 0..SCALE {
            let var_name = format!("var-{}", i);
            type_checker._lookup_variable(&var_name, local_context);
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisLookupVariableConst,
        SCALE,
        vec![1],
        setup_fn,
        eval_lookup_variable,
    )
}

/// ////////////////////////////////////
/// AST FUNCTIONS
/// ////////////////////////////////////
fn bench_ast_parse(c: &mut Criterion) {
    // SIZED_CONTRACTS will be generated the first time it is "invoked" in the code since it is
    //  defined in a lazy_static! macro call. The setup_fn uses the object to make sure it is
    //  created before being invoked in the actual benchmark.
    fn setup_fn<T: CostTracker>(
        _ca: &mut ContractAST,
        _lc: &mut TypingContext,
        _tc: &mut TypeChecker,
        input_size: u64,
        _ct: &mut T,
    ) {
        SIZED_CONTRACTS.get(&input_size);
    }

    fn eval_build_ast<T: CostTracker>(
        _ca: &mut ContractAST,
        _lc: &mut TypingContext,
        _tc: &mut TypeChecker,
        input_size: u64,
        cost_tracker: &mut T,
    ) {
        let contract = SIZED_CONTRACTS.get(&input_size).unwrap();
        let contract_id = QualifiedContractIdentifier::transient();
        for _ in 0..SCALE {
            build_ast(
                &contract_id,
                &contract,
                cost_tracker,
                ClarityVersion::Clarity2,
                StacksEpochId::Epoch21,
            );
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AstParse,
        1,
        INPUT_SIZES.into(),
        setup_fn,
        eval_build_ast,
    )
}

fn bench_add(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Add,
        SCALE,
        Some(INPUT_SIZES_ARITHMETIC.into()),
        None,
    )
}

fn bench_sub(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Sub,
        SCALE,
        Some(INPUT_SIZES_ARITHMETIC.into()),
        None,
    )
}

fn bench_mul(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Mul,
        SCALE,
        Some(INPUT_SIZES_ARITHMETIC.into()),
        None,
    )
}

fn bench_div(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Div,
        SCALE,
        Some(INPUT_SIZES_ARITHMETIC.into()),
        None,
    )
}

fn bench_le(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Le,
        SCALE,
        Some(CMP_INPUT_SIZES.to_vec()),
        None,
    )
}

fn bench_leq(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Leq,
        SCALE,
        Some(CMP_INPUT_SIZES.to_vec()),
        None,
    )
}

fn bench_ge(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Ge,
        SCALE,
        Some(CMP_INPUT_SIZES.to_vec()),
        None,
    )
}

fn bench_geq(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Geq,
        SCALE,
        Some(CMP_INPUT_SIZES.to_vec()),
        None,
    )
}

// boolean functions
fn bench_and(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::And,
        SCALE,
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_or(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Or,
        SCALE,
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_bit_and(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::BitwiseAnd,
        SCALE,
        Some(INPUT_SIZES_ARITHMETIC.into()),
        None,
    )
}

fn bench_bit_or(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::BitwiseOr,
        SCALE,
        Some(INPUT_SIZES_ARITHMETIC.into()),
        None,
    )
}

fn bench_bit_xor(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Xor,
        SCALE,
        Some(INPUT_SIZES_ARITHMETIC.into()),
        None,
    )
}

fn bench_bit_not(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::BitwiseNot, SCALE, None, None)
}

fn bench_bit_lshift(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::BitwiseLShift, SCALE, None, None)
}

fn bench_bit_rshift(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::BitwiseRShift, SCALE, None, None)
}

/*
fn bench_xor(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Xor, SCALE, None, None)
}
*/

fn bench_not(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Not, SCALE, None, None)
}

// note: only testing is-eq when the values are bools; could try doing it with ints?
fn bench_eq(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Eq,
        SCALE,
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_mod(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Mod, SCALE, None, None)
}

fn bench_pow(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Pow, SCALE, None, None)
}

fn bench_sqrti(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Sqrti, SCALE, None, None)
}

fn bench_log2(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Log2, SCALE, None, None)
}

fn bench_tuple_get(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::TupleGet,
        SCALE,
        Some(MORE_INPUT_SIZES.into()),
        None,
    )
}

fn bench_tuple_merge(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::TupleMerge,
        SCALE,
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_tuple_cons(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::TupleCons,
        SCALE,
        Some(MORE_INPUT_SIZES.into()),
        None,
    )
}

// hash functions
fn bench_hash160(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Hash160,
        SCALE,
        Some(INPUT_SIZES_DATA.into()),
        None,
    )
}

fn bench_sha256(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Sha256,
        SCALE,
        Some(INPUT_SIZES_DATA.into()),
        None,
    )
}

fn bench_sha512(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Sha512,
        SCALE,
        Some(INPUT_SIZES_DATA.into()),
        None,
    )
}

fn bench_sha512t256(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Sha512t256,
        SCALE,
        Some(INPUT_SIZES_DATA.into()),
        None,
    )
}

fn bench_keccak256(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Keccak256,
        SCALE,
        Some(INPUT_SIZES_DATA.into()),
        None,
    )
}

fn bench_secp256k1recover(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Secp256k1recover, SCALE, None, None)
}

fn bench_secp256k1verify(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Secp256k1verify, SCALE, None, None)
}

fn bench_create_ft_old(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::CreateFt, SCALE.into(), None, None)
}

// note: verify that we want a warmed-up marf for this
fn bench_create_ft(c: &mut Criterion) {
    let function = ClarityCostFunction::CreateFt;
    let mut group = c.benchmark_group(function.to_string());

    // Set up MarfedKV
    let miner_tip = StacksBlockHeader::make_index_block_hash(
        &MINER_BLOCK_CONSENSUS_HASH,
        &MINER_BLOCK_HEADER_HASH,
    );
    let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

    // Set up Clarity Backing Store
    // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
    let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
    let new_tip = StacksBlockId::from([5; 32]);
    let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

    // Set up BurnStateDB
    let pox_constants = PoxConstants::new(
        10,
        5,
        3,
        25,
        5,
        u64::MAX,
        u64::MAX,
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
    );
    let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
    let sort_tx = sort_db.index_conn();

    // Set up HeaderDB
    let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();

    let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

    let mut global_context = GlobalContext::new(
        false,
        0,
        clarity_db,
        LimitedCostTracker::new_free(),
        StacksEpochId::Epoch21,
    );
    global_context.begin();

    let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
    let mut contract_context =
        ContractContext::new(contract_identifier.clone(), ClarityVersion::Clarity2);

    group.throughput(Throughput::Bytes(0));
    group.bench_with_input(BenchmarkId::from_parameter(0), &0, |b, &_| {
        b.iter(|| {
            for _ in 0..SCALE {
                bench_create_ft_in_context(&mut global_context, &mut contract_context);
            }
        })
    });
}

fn bench_mint_ft(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FtMint, SCALE.into(), None, None)
}

fn bench_ft_transfer(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FtTransfer, SCALE.into(), None, None)
}

fn bench_ft_balance(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FtBalance, SCALE.into(), None, None)
}

fn bench_ft_supply(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FtSupply, SCALE.into(), None, None)
}

fn bench_ft_burn(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FtBurn, SCALE.into(), None, None)
}

// note: verify that we want a warmed-up marf for this
fn bench_create_nft(c: &mut Criterion) {
    let function = ClarityCostFunction::CreateNft;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        // Set up BurnStateDB
        let pox_constants = PoxConstants::new(
            10,
            5,
            3,
            25,
            5,
            u64::MAX,
            u64::MAX,
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
        );
        let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
        let sort_tx = sort_db.index_conn();

        // Set up HeaderDB
        let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();

        let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

        let mut global_context = GlobalContext::new(
            false,
            0,
            clarity_db,
            LimitedCostTracker::new_free(),
            StacksEpochId::Epoch21,
        );
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context =
            ContractContext::new(contract_identifier.clone(), ClarityVersion::Clarity2);

        let asset_type = SIZED_TYPE_SIG.get(input_size).unwrap();
        let asset_type_size = asset_type.size();
        group.throughput(Throughput::Bytes(asset_type_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(asset_type_size),
            &asset_type_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        bench_create_nft_in_context(
                            &mut global_context,
                            &mut contract_context,
                            &asset_type,
                        );
                    }
                })
            },
        );
    }
}

fn bench_nft_mint(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::NftMint,
        SCALE.into(),
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_nft_transfer(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::NftTransfer,
        SCALE.into(),
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_nft_owner(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::NftOwner,
        SCALE.into(),
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_nft_burn(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::NftBurn,
        SCALE.into(),
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_is_none(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IsNone, SCALE.into(), None, None)
}

fn bench_is_some(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IsSome, SCALE.into(), None, None)
}

fn bench_is_ok(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IsOkay, SCALE.into(), None, None)
}

fn bench_is_err(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IsErr, SCALE.into(), None, None)
}

fn bench_unwrap(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Unwrap, SCALE.into(), None, None)
}

fn bench_unwrap_ret(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::UnwrapRet, SCALE.into(), None, None)
}

fn bench_unwrap_err(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::UnwrapErr, SCALE.into(), None, None)
}

fn bench_unwrap_err_or_ret(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::UnwrapErrOrRet,
        SCALE.into(),
        None,
        None,
    )
}

// note: verify that we want a warmed-up marf for this
// note: time to clone the type signature for the value in the benching code may be significant
fn bench_create_map(c: &mut Criterion) {
    let function = ClarityCostFunction::CreateMap;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        // Set up BurnStateDB
        let pox_constants = PoxConstants::new(
            10,
            5,
            3,
            25,
            5,
            u64::MAX,
            u64::MAX,
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
        );
        let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
        let sort_tx = sort_db.index_conn();

        // Set up HeaderDB
        let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();

        let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

        let mut global_context = GlobalContext::new(
            false,
            0,
            clarity_db,
            LimitedCostTracker::new_free(),
            StacksEpochId::Epoch21,
        );
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context =
            ContractContext::new(contract_identifier.clone(), ClarityVersion::Clarity2);

        let key_type = TypeSignature::BoolType;
        let value_type = SIZED_TYPE_SIG.get(input_size).unwrap();
        let total_size = (key_type.size() + value_type.size()) as u64;

        group.throughput(Throughput::Bytes(total_size));
        group.bench_with_input(
            BenchmarkId::from_parameter(total_size),
            &total_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        bench_create_map_in_context(
                            &mut global_context,
                            &mut contract_context,
                            key_type.clone(),
                            value_type.clone(),
                        );
                    }
                })
            },
        );
    }
}

// note: verify that we want a warmed-up marf for this
// note: time to clone the type signature for the value may be significant
fn bench_create_var(c: &mut Criterion) {
    let function = ClarityCostFunction::CreateVar;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        // Set up BurnStateDB
        let pox_constants = PoxConstants::new(
            10,
            5,
            3,
            25,
            5,
            u64::MAX,
            u64::MAX,
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
        );
        let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
        let sort_tx = sort_db.index_conn();

        // Set up HeaderDB
        let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();

        let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

        let mut global_context = GlobalContext::new(
            false,
            0,
            clarity_db,
            LimitedCostTracker::new_free(),
            StacksEpochId::Epoch21,
        );
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context =
            ContractContext::new(contract_identifier.clone(), ClarityVersion::Clarity2);

        let value_type = SIZED_TYPE_SIG.get(input_size).unwrap();
        let value_type_size = value_type.size();
        let value = helper_make_value_for_sized_type_sig(*input_size);
        assert!(value_type.admits(&StacksEpochId::latest(), &value).unwrap());
        assert_eq!(value_type.size(), value.size());

        group.throughput(Throughput::Bytes(value_type_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(value_type_size),
            &value_type_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        bench_create_var_in_context(
                            &mut global_context,
                            &mut contract_context,
                            value_type.clone(),
                            value.clone(),
                        );
                    }
                })
            },
        );
    }
}

fn bench_wrapped_data_function(
    mut group: BenchmarkGroup<WallTime>,
    cost_function: ClarityCostFunction,
    input_sizes: Vec<u64>,
    scale: u16,
) {
    for input_size in input_sizes.iter() {
        // Set up MarfedKV
        let miner_tip = StacksBlockHeader::make_index_block_hash(
            &MINER_BLOCK_CONSENSUS_HASH,
            &MINER_BLOCK_HEADER_HASH,
        );
        let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

        // Set up Clarity Backing Store
        // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
        let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
        let new_tip = StacksBlockId::from([5; 32]);
        let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

        // Set up BurnStateDB
        let pox_constants = PoxConstants::new(
            10,
            5,
            3,
            25,
            5,
            u64::MAX,
            u64::MAX,
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
            u32::max_value(),
        );
        let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
        let sort_tx = sort_db.index_conn();

        // Set up HeaderDB
        let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();

        let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

        let mut global_context = GlobalContext::new(
            false,
            0,
            clarity_db,
            LimitedCostTracker::new_free(),
            StacksEpochId::Epoch21,
        );
        global_context.begin();

        let GenOutput {
            setup: pre_contract_opt,
            body: _,
            input_size: _,
        } = gen(cost_function, scale, *input_size);

        let list_len = helper_gen_clarity_list_type(*input_size).1;
        let list_size = 5 + 17 * list_len;

        let contract_identifier =
            QualifiedContractIdentifier::local(&*format!("c{}", list_size)).unwrap();
        let mut contract_context =
            ContractContext::new(contract_identifier.clone(), ClarityVersion::Clarity2);
        let publisher: PrincipalData = contract_context.contract_identifier.issuer.clone().into();

        match pre_contract_opt {
            Some(pre_contract) => {
                let pre_contract_identifier =
                    QualifiedContractIdentifier::local(&*format!("pre{}", list_size)).unwrap();
                let pre_contract_ast = match ast::build_ast(
                    &pre_contract_identifier,
                    &pre_contract,
                    &mut (),
                    ClarityVersion::Clarity2,
                    StacksEpochId::Epoch21,
                ) {
                    Ok(res) => res,
                    Err(error) => {
                        panic!("Parsing error: {}", error.diagnostic.message);
                    }
                };
                global_context
                    .execute(|g| {
                        eval_all(
                            &pre_contract_ast.expressions,
                            &mut contract_context,
                            g,
                            None,
                        )
                    })
                    .unwrap();
            }
            _ => {}
        }

        group.throughput(Throughput::Bytes(list_size.clone() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(list_size),
            &list_size,
            |b, &_| {
                b.iter(|| {
                    global_context
                        .execute(|g| {
                            let mut call_stack = CallStack::new();
                            let mut env = Environment::new(
                                g,
                                &contract_context,
                                &mut call_stack,
                                Some(publisher.clone()),
                                Some(publisher.clone()),
                                None,
                            );
                            let f = lookup_function("execute", &mut env).unwrap();
                            let list = Value::list_from(
                                (0..list_len).map(|i| Value::UInt(i as u128)).collect(),
                            )
                            .unwrap();
                            apply(
                                &f,
                                &[SymbolicExpression::literal_value(list)],
                                &mut env,
                                &LocalContext::new(),
                            )
                        })
                        .unwrap()
                })
            },
        );
    }
}

fn bench_set_var(c: &mut Criterion) {
    let cost_function = ClarityCostFunction::SetVar;
    let mut group = c.benchmark_group(cost_function.to_string());
    group.sample_size(50);
    bench_wrapped_data_function(group, cost_function, INPUT_SIZES_DATA.into(), SCALE)
}

fn bench_fetch_var(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::FetchVar,
        SCALE.into(),
        Some(INPUT_SIZES_DATA.into()),
        None,
    )
}

fn bench_print(c: &mut Criterion) {
    let cost_function = ClarityCostFunction::Print;
    let group = c.benchmark_group(cost_function.to_string());
    bench_wrapped_data_function(group, cost_function, INPUT_SIZES_DATA.into(), SCALE)
}

fn bench_if(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::If, SCALE.into(), None, None)
}

fn bench_asserts(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Asserts, SCALE.into(), None, None)
}

fn bench_ok_cons(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::OkCons, SCALE.into(), None, None)
}

fn bench_err_cons(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::ErrCons, SCALE.into(), None, None)
}

fn bench_some_cons(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::SomeCons, SCALE.into(), None, None)
}

fn bench_concat(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Concat,
        SCALE.into(),
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_as_max_len(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::AsMaxLen, SCALE.into(), None, None)
}

fn bench_begin(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Begin, SCALE.into(), None, None)
}

fn bench_bind_name(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::BindName, SCALE.into(), None, None)
}

fn bench_default_to(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::DefaultTo, SCALE.into(), None, None)
}

fn bench_try(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::TryRet, SCALE.into(), None, None)
}

fn bench_int_cast(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IntCast, SCALE.into(), None, None)
}

fn bench_set_entry(c: &mut Criterion) {
    let cost_function = ClarityCostFunction::SetEntry;
    let group = c.benchmark_group(cost_function.to_string());
    bench_wrapped_data_function(group, cost_function, INPUT_SIZES_DATA.into(), SCALE)
}

fn bench_fetch_entry(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::FetchEntry,
        SCALE.into(),
        Some(INPUT_SIZES_DATA.into()),
        None,
    )
}

fn bench_match(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Match, SCALE.into(), None, None)
}

fn bench_let(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Let,
        SCALE.into(),
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_index_of(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::IndexOf,
        SCALE.into(),
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_element_at(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::ElementAt, SCALE.into(), None, None)
}

fn bench_len(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Len, SCALE.into(), None, None)
}

fn bench_list_cons(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::ListCons,
        SCALE.into(),
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_append(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Append,
        SCALE.into(),
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_filter(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Filter, SCALE.into(), None, None)
}

// note: this takes a lot of time to run; can shorten the list sizes to make it faster
fn bench_map(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Map,
        SCALE.into(),
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_fold(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Fold, SCALE.into(), None, None)
}

fn bench_block_info(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::BlockInfo, SCALE.into(), None, None)
}

fn bench_at_block(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::AtBlock, SCALE.into(), None, None)
}

fn bench_load_contract(c: &mut Criterion) {
    let mut group = c.benchmark_group(ClarityCostFunction::LoadContract.to_string());

    // Set up MarfedKV
    let miner_tip = StacksBlockHeader::make_index_block_hash(
        &MINER_BLOCK_CONSENSUS_HASH,
        &MINER_BLOCK_HEADER_HASH,
    );
    let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

    // Set up Clarity Backing Store
    // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
    let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
    let new_tip = StacksBlockId::from([5; 32]);
    let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

    // Set up BurnStateDB
    let pox_constants = PoxConstants::new(
        10,
        5,
        3,
        25,
        5,
        u64::MAX,
        u64::MAX,
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
    );
    let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
    let sort_tx = sort_db.index_conn();

    // Set up HeaderDB
    let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();

    let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

    let mut owned_env = OwnedEnvironment::new_free(true, 0, clarity_db, StacksEpochId::Epoch21);
    owned_env.begin();

    let mut contract_context = ContractContext::new(
        QualifiedContractIdentifier::transient(),
        ClarityVersion::Clarity2,
    );
    let mut env = owned_env.get_exec_environment(None, None, &mut contract_context);

    for size in INPUT_SIZES.iter() {
        let contract_identifier =
            QualifiedContractIdentifier::local(format!("contract{}", size).as_str()).unwrap();

        let GenOutput {
            setup: _,
            body: contract,
            input_size: _,
        } = gen_read_only_func(*size as u16);

        env.initialize_contract(
            contract_identifier.clone(),
            &contract,
            ASTRules::PrecheckSize,
        )
        .unwrap();

        let contract_size = env
            .global_context
            .database
            .get_contract_size(&contract_identifier)
            .unwrap();

        group.throughput(Throughput::Bytes(contract_size));
        group.bench_with_input(
            BenchmarkId::from_parameter(contract_size),
            &contract_size,
            |b, &_| {
                b.iter(|| {
                    env.load_contract_for_bench(&contract_identifier).unwrap();
                })
            },
        );
    }
}

fn bench_type_parse_step(c: &mut Criterion) {
    fn eval_track_return_type<T: CostTracker>(
        contract_ast: &mut ContractAST,
        _lc: &mut TypingContext,
        _tc: &mut TypeChecker,
        _i: u64,
        cost_tracker: &mut T,
    ) {
        for exp in &contract_ast.expressions {
            TypeSignature::parse_type_repr(StacksEpochId::latest(), exp, cost_tracker);
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::TypeParseStep,
        SCALE,
        vec![1],
        dummy_setup_code,
        eval_track_return_type,
    )
}

fn bench_stx_transfer(c: &mut Criterion) {
    let make_store = |env: &mut OwnedEnvironment| {
        let principal_data = env.eval_raw("tx-sender").unwrap().0;
        if let Value::Principal(pd) = principal_data {
            env.stx_faucet(&pd, 100000000000);
        } else {
            panic!();
        }
    };

    bench_with_input_sizes(
        c,
        ClarityCostFunction::StxTransfer,
        SCALE.into(),
        None,
        Some(Box::new(make_store)),
    )
}

fn bench_stx_get_balance(c: &mut Criterion) {
    let make_store = |env: &mut OwnedEnvironment| {
        let addr = StacksAddress::from_string("S1G2081040G2081040G2081040G208105NK8PE5").unwrap();
        let principal = PrincipalData::from(addr);
        env.stx_faucet(&principal, 1000);
    };

    bench_with_input_sizes(
        c,
        ClarityCostFunction::StxBalance,
        SCALE.into(),
        None,
        Some(Box::new(make_store)),
    )
}

fn bench_poison_microblock(c: &mut Criterion) {
    let mut group = c.benchmark_group(ClarityCostFunction::PoisonMicroblock.to_string());

    // Set up MarfedKV
    let miner_tip = StacksBlockHeader::make_index_block_hash(
        &MINER_BLOCK_CONSENSUS_HASH,
        &MINER_BLOCK_HEADER_HASH,
    );
    let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

    // Set up Clarity Backing Store
    // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
    let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
    let new_tip = StacksBlockId::from([5; 32]);
    let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

    // Set up BurnStateDB
    let pox_constants = PoxConstants::new(
        10,
        5,
        3,
        25,
        5,
        u64::MAX,
        u64::MAX,
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
    );
    let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
    let sort_tx = sort_db.index_conn();

    // Set up HeaderDB
    let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();

    let clarity_db = ClarityDatabase::new(&mut writeable_marf_store, &headers_db, &sort_tx);

    let mut owned_env = OwnedEnvironment::new_free(true, 0, clarity_db, StacksEpochId::Epoch21);
    owned_env.begin();
    // TODO: verify correctness
    let mut contract_context = ContractContext::new(
        QualifiedContractIdentifier::transient(),
        ClarityVersion::Clarity2,
    );
    let mut env = owned_env.get_exec_environment(None, None, &mut contract_context);

    let privk_string = "eb05c83546fdd2c79f10f5ad5434a90dd28f7e3acb7c092157aa1bc3656b012c01";

    let ref h1 = StacksMicroblockHeader {
        version: 18,
        sequence: 8,
        prev_block: BlockHeaderHash::from_hex("06722a7d6537c3dd382a2cf1e56962ed36c26930e3b85faa4489caeb5097f724").unwrap(),
        tx_merkle_root: Sha512Trunc256Sum::from_hex("ce7e657cb5af17c320b41a234efdd6f0d4e45272bfd3087efaf0a12eacb75eae").unwrap(),
        signature: MessageSignature::from_hex("010eae2221e50cac44ef24fc35d691b02158c3697a4200e8573b13b14bf984526947318d6653216f6b73d490f44f2979ffb334e14706e01b350f18c946be1b0e2e").unwrap(),
    };

    let pubkh = h1.check_recover_pubkey().unwrap();

    env.global_context
        .database
        .insert_microblock_pubkey_hash_height(&pubkh, 60)
        .unwrap();

    let sk = StacksPrivateKey::from_hex(privk_string).unwrap();
    let addr = StacksAddress::from_public_keys(
        C32_ADDRESS_VERSION_TESTNET_SINGLESIG,
        &AddressHashMode::SerializeP2PKH,
        1,
        &vec![StacksPublicKey::from_private(&sk)],
    )
    .unwrap();

    env.sender = Some(addr.into());

    group.throughput(Throughput::Bytes(1u64));
    group.bench_with_input(BenchmarkId::from_parameter(1), &1, |b, &_| {
        b.iter(|| {
            for _ in 0..SCALE {
                StacksChainState::handle_poison_microblock(&mut env, h1, h1).unwrap();
            }
        })
    });
}

fn bench_contract_call(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::ContractCall,
        SCALE.into(),
        None,
        None,
    )
}

fn bench_contract_of(c: &mut Criterion) {
    let make_store = |env: &mut OwnedEnvironment| {
        let define_identifier =
            QualifiedContractIdentifier::local("define-trait-contract").unwrap();
        let define_contract = "(define-trait trait-1 ((get-1 (uint) (response uint uint))))";
        env.initialize_contract(
            define_identifier,
            define_contract,
            None,
            ASTRules::PrecheckSize,
        )
        .unwrap();

        let impl_identifier = QualifiedContractIdentifier::local("impl-trait-contract").unwrap();
        let impl_contract = "(impl-trait .define-trait-contract.trait-1)
            (define-public (get-1 (x uint)) (ok u99))";
        env.initialize_contract(impl_identifier, impl_contract, None, ASTRules::PrecheckSize)
            .unwrap();

        let use_identifier = QualifiedContractIdentifier::local("use-trait-contract").unwrap();
        let use_contract = "(use-trait trait-1 .define-trait-contract.trait-1)
            (define-public (bench-contract-of (contract <trait-1>))
                (ok (contract-of contract)))";
        env.initialize_contract(use_identifier, use_contract, None, ASTRules::PrecheckSize)
            .unwrap();
    };

    bench_with_input_sizes(
        c,
        ClarityCostFunction::ContractOf,
        SCALE.into(),
        None,
        Some(Box::new(make_store)),
    )
}

fn bench_as_contract(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::AsContract, SCALE.into(), None, None)
}

//////////////////////////////////
// Clarity 2 Functions
/////////////////////////////////

fn bench_stx_get_account(c: &mut Criterion) {
    let make_store = |env: &mut OwnedEnvironment| {
        let addr = StacksAddress::from_string("S1G2081040G2081040G2081040G208105NK8PE5").unwrap();
        let principal = PrincipalData::from(addr);
        env.stx_faucet(&principal, 1000);
    };

    bench_with_input_sizes(
        c,
        ClarityCostFunction::StxGetAccount,
        SCALE.into(),
        None,
        Some(Box::new(make_store)),
    )
}

fn bench_buff_to_int_le(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::BuffToIntLe,
        SCALE.into(),
        None,
        None,
    )
}

fn bench_buff_to_uint_le(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::BuffToUIntLe,
        SCALE.into(),
        None,
        None,
    )
}

fn bench_buff_to_int_be(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::BuffToIntBe,
        SCALE.into(),
        None,
        None,
    )
}

fn bench_buff_to_uint_be(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::BuffToUIntBe,
        SCALE.into(),
        None,
        None,
    )
}

fn bench_is_standard(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IsStandard, SCALE.into(), None, None)
}

// FIXME:  Panics with `Unchecked(UndefinedFunction("principal-destruct"))``
fn bench_principal_destruct(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::PrincipalDestruct,
        SCALE.into(),
        None,
        None,
    )
}

// FIXME: Panics with `Unchecked(UndefinedFunction("principal-construct"))``
fn bench_principal_construct(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::PrincipalConstruct,
        SCALE.into(),
        None,
        None,
    )
}

// FIXME: Panics with `Unchecked(UndefinedFunction("string-to-int"))``
fn bench_string_to_int(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::StringToInt,
        SCALE.into(),
        None,
        None,
    )
}

// FIXME: Panics with `Unchecked(UndefinedFunction("string-to-uint"))``
fn bench_string_to_uint(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::StringToUInt,
        SCALE.into(),
        None,
        None,
    )
}

fn bench_int_to_ascii(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IntToAscii, SCALE.into(), None, None)
}

fn bench_int_to_utf8(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IntToUtf8, SCALE.into(), None, None)
}

fn bench_slice(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Slice, SCALE.into(), None, None)
}

fn bench_to_consensus_buff(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::ToConsensusBuff,
        SCALE.into(),
        Some(INPUT_SIZES_DATA.into()),
        None,
    )
}

fn bench_from_consensus_buff(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::FromConsensusBuff,
        SCALE.into(),
        Some(INPUT_SIZES.into()),
        None,
    )
}

fn bench_stx_transfer_memo(c: &mut Criterion) {
    let make_store = |env: &mut OwnedEnvironment| {
        let principal_data = env.eval_raw("tx-sender").unwrap().0;
        if let Value::Principal(pd) = principal_data {
            env.stx_faucet(&pd, 100000000000);
        } else {
            panic!();
        }
    };

    bench_with_input_sizes(
        c,
        ClarityCostFunction::StxTransferMemo,
        SCALE.into(),
        None,
        Some(Box::new(make_store)),
    )
}

fn bench_get_burn_block_info(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::GetBurnBlockInfo,
        SCALE.into(),
        None,
        None,
    )
}

fn bench_replace_at(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::ReplaceAt,
        SCALE.into(),
        Some(INPUT_SIZES_DATA.into()),
        None,
    )
}

fn bench_analysis_fetch_contract_entry(c: &mut Criterion) {
    let mut group = c.benchmark_group(ClarityCostFunction::AnalysisFetchContractEntry.to_string());

    // Set up MarfedKV
    let miner_tip = StacksBlockHeader::make_index_block_hash(
        &MINER_BLOCK_CONSENSUS_HASH,
        &MINER_BLOCK_HEADER_HASH,
    );
    let mut marfed_kv = MarfedKV::open(CLARITY_MARF_PATH, Some(&miner_tip), None).unwrap();

    // Set up Clarity Backing Store
    // NOTE: this StacksBlockId comes from the `block_headers` in the chainstate DB (db/index.sqlite)
    let read_tip = StacksBlockId::from_hex(READ_TIP).unwrap();
    let new_tip = StacksBlockId::from([5; 32]);
    let mut writeable_marf_store = marfed_kv.begin(&read_tip, &new_tip);

    // Set up BurnStateDB
    let pox_constants = PoxConstants::new(
        10,
        5,
        3,
        25,
        5,
        u64::MAX,
        u64::MAX,
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
        u32::max_value(),
    );
    let sort_db = SortitionDB::open(SORTITION_MARF_PATH, false, pox_constants).unwrap();
    let sort_tx = sort_db.index_conn();

    // Set up HeaderDB
    let headers_db = StacksChainState::open_db(false, 2147483648, CHAINSTATE_PATH).unwrap();
    let mut analysis_db = AnalysisDatabase::new(&mut writeable_marf_store);
    let mut contract_context = ContractContext::new(
        QualifiedContractIdentifier::transient(),
        ClarityVersion::Clarity2,
    );

    for size in INPUT_SIZES.iter() {
        let contract_identifier =
            QualifiedContractIdentifier::local(format!("contract{}", size).as_str()).unwrap();

        let GenOutput {
            setup: _,
            body: contract,
            input_size: _,
        } = gen_analysis_fetch_contract_entry(*size as u64);

        analysis_db.begin();
        let mut contract_ast = ast::build_ast(
            &contract_identifier,
            &contract,
            &mut (),
            ClarityVersion::Clarity2,
            StacksEpochId::Epoch21,
        )
        .unwrap();
        let mut ct = LimitedCostTracker::new_free();
        let contract_analysis = analysis::run_analysis(
            &contract_identifier,
            &mut contract_ast.expressions,
            &mut analysis_db,
            true,
            ct,
            StacksEpochId::latest(),
            ClarityVersion::Clarity2,
        )
        .unwrap();

        // how many bytes
        let contract_size = contract_analysis.serialize().len();

        group.throughput(Throughput::Bytes(contract_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(contract_size),
            &contract_size,
            |b, &_| {
                b.iter(|| {
                    analysis_db
                        .load_contract(&contract_identifier, &StacksEpochId::latest())
                        .unwrap();
                })
            },
        );
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets =
    bench_add,
    bench_sub,
    bench_mul,
    bench_div,
    bench_le,
    bench_leq,
    bench_ge,
    bench_geq,
    bench_and,
    bench_or,
    // bench_xor,
    bench_not,
    bench_bit_and,
    bench_bit_or,
    bench_bit_xor,
    bench_bit_not,
    bench_bit_lshift,
    bench_bit_rshift,
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
    bench_create_ft,    // g
    bench_mint_ft,      // g
    bench_ft_transfer,  // g
    bench_ft_balance,   // g
    bench_ft_supply,    // g
    bench_ft_burn,      // g
    bench_create_nft,   // g
    bench_nft_mint,     // g
    bench_nft_transfer, // g
    bench_nft_owner,    // g
    bench_nft_burn,     // g
    bench_is_none,
    bench_is_some,
    bench_is_ok,
    bench_is_err,
    bench_unwrap,
    bench_unwrap_ret,
    bench_unwrap_err,
    bench_unwrap_err_or_ret,
    bench_create_map, // g
    bench_create_var, // g
    bench_set_var,    // g
    bench_fetch_var,  // g
    // bench_print,
    bench_if,
    bench_asserts,
    bench_ok_cons,
    bench_some_cons,
    bench_err_cons,
    bench_concat,
    bench_as_max_len,
    bench_begin,
    bench_bind_name,
    bench_default_to,
    bench_try,
    bench_int_cast,
    bench_set_entry,   // g
    bench_fetch_entry, // g
    bench_match,
    bench_let,
    bench_index_of,
    bench_element_at,
    bench_len,
    bench_list_cons,
    bench_append,
    bench_filter,
    bench_fold,
    bench_at_block,
    bench_load_contract,
    bench_map,
    bench_block_info,
    bench_lookup_variable_depth,
    bench_lookup_variable_size,
    bench_lookup_function,
    bench_type_parse_step,
    bench_analysis_option_cons,
    bench_analysis_option_check,
    bench_analysis_visit,
    bench_analysis_bind_name,
    bench_analysis_list_items_check,
    bench_analysis_check_tuple_get,
    bench_analysis_check_tuple_merge,
    bench_analysis_check_tuple_cons,
    bench_analysis_tuple_items_check,
    bench_analysis_check_let,
    bench_analysis_lookup_function,
    bench_analysis_lookup_function_types,
    bench_analysis_type_annotate,
    bench_analysis_iterable_func,
    bench_analysis_storage,
    bench_analysis_type_check,
    bench_analysis_lookup_variable_depth,
    bench_analysis_type_lookup,
    bench_analysis_lookup_variable_const,
    //bench_analysis_use_trait_entry,
    bench_analysis_get_function_entry,
    bench_inner_type_check_cost,
    bench_user_function_application,
    //bench_ast_cycle_detection,
    bench_ast_parse,
    bench_contract_storage,
    bench_principal_of,
    bench_stx_transfer,
    bench_stx_get_balance,
    bench_analysis_pass_read_only,               // g
    bench_analysis_pass_arithmetic_only_checker, // g
    bench_analysis_pass_trait_checker,           // g
    bench_analysis_pass_type_checker,            // g
    bench_poison_microblock,
    bench_contract_call,
    bench_contract_of,
    bench_buff_to_int_le,
    bench_buff_to_uint_le,
    bench_buff_to_int_be,
    bench_buff_to_uint_be,
    bench_is_standard,
    //bench_principal_destruct,
    //bench_principal_construct,
    //bench_string_to_int,
    //bench_string_to_uint,
    bench_int_to_ascii,
    bench_int_to_utf8,
    bench_stx_get_account,
    bench_stx_transfer_memo,
    bench_get_burn_block_info,
    bench_slice,
    bench_to_consensus_buff,
    bench_from_consensus_buff,
    bench_replace_at,
    bench_analysis_fetch_contract_entry,
    bench_as_contract
);

criterion_main!(benches);
