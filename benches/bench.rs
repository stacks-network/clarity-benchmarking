use std::fs;
use std::num::ParseIntError;

use benchmarking_lib::generators::{
    define_dummy_trait, gen, gen_analysis_pass, gen_read_only_func,
    helper_generate_rand_char_string, helper_make_value_for_sized_type_sig,
    make_sized_contracts_map, make_sized_tuple_sigs_map, make_sized_type_sig_map,
    make_sized_values_map,
};
use benchmarking_lib::headers_db::{SimHeadersDB, TestHeadersDB};
use blockstack_lib::address::AddressHashMode;
use blockstack_lib::chainstate::stacks::db::StacksChainState;
use blockstack_lib::chainstate::stacks::{
    CoinbasePayload, StacksBlock, StacksMicroblock, StacksPrivateKey, StacksPublicKey,
    StacksTransaction, StacksTransactionSigner, TransactionAnchorMode, TransactionAuth,
    TransactionPayload, TransactionVersion, C32_ADDRESS_VERSION_TESTNET_SINGLESIG,
};
use blockstack_lib::clarity_vm::clarity::ClarityInstance;
use blockstack_lib::clarity_vm::database::{marf::MarfedKV, MemoryBackingStore};
use blockstack_lib::core::{
    BLOCK_LIMIT_MAINNET, FIRST_BURNCHAIN_CONSENSUS_HASH, FIRST_STACKS_BLOCK_HASH,
};
use blockstack_lib::types::chainstate::{
    BlockHeaderHash, BurnchainHeaderHash, StacksAddress, StacksBlockHeader, StacksBlockId,
    StacksMicroblockHeader, StacksWorkScore, VRFSeed,
};
use blockstack_lib::types::proof::{ClarityMarfTrieId, TrieHash};
use blockstack_lib::util::hash::{hex_bytes, to_hex, Hash160, MerkleTree, Sha512Trunc256Sum};
use blockstack_lib::util::secp256k1::MessageSignature;
use blockstack_lib::util::vrf::VRFProof;
use blockstack_lib::vm::analysis::arithmetic_checker::ArithmeticOnlyChecker;
use blockstack_lib::vm::analysis::read_only_checker::ReadOnlyChecker;
use blockstack_lib::vm::analysis::trait_checker::TraitChecker;
use blockstack_lib::vm::analysis::type_checker::contexts::TypingContext;
use blockstack_lib::vm::analysis::type_checker::natives::assets::bench_check_special_mint_asset;
use blockstack_lib::vm::analysis::type_checker::natives::options::{
    check_special_is_response, check_special_some,
};
use blockstack_lib::vm::analysis::type_checker::natives::sequences::{
    check_special_map, get_simple_native_or_user_define,
};
use blockstack_lib::vm::analysis::type_checker::natives::{
    bench_analysis_get_function_entry_in_context, bench_check_contract_call, check_special_get,
    check_special_let, check_special_list_cons, check_special_merge, check_special_tuple_cons,
    inner_handle_tuple_get,
};
use blockstack_lib::vm::analysis::type_checker::{trait_type_size, TypeChecker};
use blockstack_lib::vm::analysis::{AnalysisDatabase, AnalysisPass, CheckResult, ContractAnalysis};
use blockstack_lib::vm::ast::definition_sorter::DefinitionSorter;
use blockstack_lib::vm::ast::expression_identifier::ExpressionIdentifier;
use blockstack_lib::vm::ast::{build_ast, parser, ContractAST};
use blockstack_lib::vm::contexts::{ContractContext, GlobalContext, OwnedEnvironment};
use blockstack_lib::vm::contracts::Contract;
use blockstack_lib::vm::costs::cost_functions::{AnalysisCostFunction, ClarityCostFunction};
use blockstack_lib::vm::costs::{CostTracker, ExecutionCost, LimitedCostTracker};
use blockstack_lib::vm::database::clarity_store::NullBackingStore;
use blockstack_lib::vm::database::{
    ClarityDatabase, HeadersDB, NULL_BURN_STATE_DB, NULL_HEADER_DB,
};
use blockstack_lib::vm::functions::crypto::special_principal_of;
use blockstack_lib::vm::representations::depth_traverse;
use blockstack_lib::vm::types::signatures::TypeSignature::{
    BoolType, IntType, NoType, PrincipalType, TupleType, UIntType,
};
use blockstack_lib::vm::types::signatures::{TupleTypeSignature, TypeSignature};
use blockstack_lib::vm::types::{
    FunctionSignature, FunctionType, PrincipalData, QualifiedContractIdentifier,
    StandardPrincipalData, TraitIdentifier,
};
use blockstack_lib::vm::{
    ast, bench_create_ft_in_context, bench_create_map_in_context, bench_create_nft_in_context,
    bench_create_var_in_context, eval_all, lookup_function, lookup_variable, CallStack,
    ClarityName, Environment, LocalContext, SymbolicExpression, Value,
};
use criterion::measurement::WallTime;
use criterion::{
    criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion, Throughput,
};
use lazy_static::lazy_static;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

const INPUT_SIZES: [u16; 8] = [1, 2, 8, 16, 32, 64, 128, 256];
const INPUT_SIZES_ANALYSIS_PASS: [u16; 6] = [1, 2, 8, 16, 32, 64];
const INPUT_SIZES_ARITHMETIC: [u16; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
const MORE_INPUT_SIZES: [u16; 12] = [1, 2, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];
const SCALE: u16 = 100;
const MARF_SCALE: u32 = 100000;

pub const NUM_BLOCKS: u32 = 60;

lazy_static! {
    pub static ref SIZED_VALUES: HashMap<u16, Value> = make_sized_values_map(INPUT_SIZES.to_vec());
    pub static ref SIZED_CONTRACTS: HashMap<u16, String> =
        make_sized_contracts_map(INPUT_SIZES.to_vec());
    // The size of the TupleTypeSignature is measured by the length of its type map
    pub static ref SIZED_TUPLE_SIG: HashMap<u16, TupleTypeSignature> =
        make_sized_tuple_sigs_map(INPUT_SIZES.to_vec());
    pub static ref SIZED_TYPE_SIG: HashMap<u16, TypeSignature> =
        make_sized_type_sig_map(INPUT_SIZES.to_vec());
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

fn setup_chain_state<'a>(scaling: u32, headers_db: &'a dyn HeadersDB) -> MarfedKV {
    let pre_initialized_path = format!("/tmp/clarity_bench_{}.marf", scaling);
    let out_path = "/tmp/clarity_bench_last.marf";

    if fs::metadata(&pre_initialized_path).is_err() {
        let marf = MarfedKV::open(&pre_initialized_path, None).unwrap();
        let mut clarity_instance = ClarityInstance::new(false, marf, ExecutionCost::max_value());

        let p1 = QualifiedContractIdentifier::parse("S1G2081040G2081040G2081040G208105NK8PE5.c1")
            .unwrap()
            .issuer;
        let principals: [PrincipalData; 2] =
            [p1.into(), StandardPrincipalData(0, as_hash160(2)).into()];

        let mut conn = clarity_instance.begin_test_genesis_block(
            &StacksBlockId::sentinel(),
            &StacksBlockId(as_hash(0)),
            &*headers_db,
            &NULL_BURN_STATE_DB,
        );

        // mint and scale marf in genesis block
        conn.as_transaction(|tx| {
            // minting
            tx.with_clarity_db(|db| {
                principals.iter().for_each(|p| {
                    let mut stx_account = db.get_stx_balance_snapshot_genesis(&p);
                    stx_account.credit(1_000_000);
                    stx_account.save();
                });
                Ok(())
            })
            .unwrap();

            // scaling
            tx.with_clarity_db(|db| {
                (0..100000).for_each(|j| {
                    db.put(format!("key{}", j).as_str(), &Value::none());
                });
                Ok(())
            })
            .unwrap();
        });

        conn.commit_to_block(&StacksBlockId(as_hash(0)));

        // append more blocks
        let blocks: Vec<_> = (0..=NUM_BLOCKS)
            .into_iter()
            .map(|i| StacksBlockId(as_hash(i)))
            .collect();

        for ix in 1..blocks.len() {
            let parent_block = &blocks[ix - 1];
            let current_block = &blocks[ix];

            let mut conn = clarity_instance.begin_block(
                &parent_block,
                &current_block,
                &*headers_db,
                &NULL_BURN_STATE_DB,
            );

            conn.as_transaction(|tx| {
                if ix == 1 {
                    let define_identifier =
                        QualifiedContractIdentifier::local("define-trait-contract").unwrap();
                    let define_contract =
                        "(define-trait trait-1 ((get-1 (uint) (response uint uint))))";

                    let (define_ast, analysis) = tx
                        .analyze_smart_contract(&define_identifier, define_contract)
                        .unwrap();

                    tx.save_analysis(&define_identifier, &analysis).unwrap();
                    tx.initialize_smart_contract(
                        &define_identifier,
                        &define_ast,
                        &define_contract,
                        |_, _| false,
                    )
                    .unwrap();
                } else if ix == 2 {
                    let impl_identifier =
                        QualifiedContractIdentifier::local("impl-trait-contract").unwrap();
                    let impl_contract = "(impl-trait .define-trait-contract.trait-1)
                        (define-public (get-1 (x uint)) (ok u99))";

                    let (impl_ast, analysis) = tx
                        .analyze_smart_contract(&impl_identifier, impl_contract)
                        .unwrap();

                    tx.save_analysis(&impl_identifier, &analysis).unwrap();
                    tx.initialize_smart_contract(
                        &impl_identifier,
                        &impl_ast,
                        &impl_contract,
                        |_, _| false,
                    )
                    .unwrap();
                } else if ix == 3 {
                    let use_identifier =
                        QualifiedContractIdentifier::local("use-trait-contract").unwrap();
                    let use_contract = "(use-trait trait-1 .define-trait-contract.trait-1)
                        (define-public (bench-contract-of (contract <trait-1>))
                            (ok (contract-of contract)))";

                    let (use_ast, analysis) = tx
                        .analyze_smart_contract(&use_identifier, use_contract)
                        .unwrap();

                    tx.save_analysis(&use_identifier, &analysis).unwrap();
                    tx.initialize_smart_contract(
                        &use_identifier,
                        &use_ast,
                        &use_contract,
                        |_, _| false,
                    )
                    .unwrap();
                }

                tx.with_clarity_db(|db| {
                    (0..100).for_each(|j| {
                        db.put(format!("key{}", j).as_str(), &Value::none());
                    });
                    Ok(())
                })
                .unwrap();
            });

            conn.commit_to_block(current_block);
        }
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

fn eval(
    contract_ast: &ContractAST,
    global_context: &mut GlobalContext,
    contract_context: &mut ContractContext,
) {
    global_context
        .execute(|g| eval_all(&contract_ast.expressions, contract_context, g))
        .unwrap();
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
        if use_marf {
            let headers_db = SimHeadersDB::new();
            let mut marf = setup_chain_state(MARF_SCALE, &headers_db);

            let mut marf_store =
                marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));
            let clarity_db = marf_store.as_clarity_db(&headers_db, &NULL_BURN_STATE_DB);

            run_bench(&mut group, function, scale, *input_size, clarity_db, eval)
        } else {
            let mut memory_backing_store = MemoryBackingStore::new();
            let clarity_db = memory_backing_store.as_clarity_db();

            run_bench(&mut group, function, scale, *input_size, clarity_db, eval)
        }
    }
}

fn run_bench<F>(
    group: &mut BenchmarkGroup<WallTime>,
    function: ClarityCostFunction,
    scale: u16,
    input_size: u16,
    clarity_db: ClarityDatabase,
    code_to_bench: F,
) where
    F: Fn(&ContractAST, &mut GlobalContext, &mut ContractContext),
{
    let mut global_context = GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
    global_context.begin();

    let contract_identifier =
        QualifiedContractIdentifier::local(&*format!("c{}", input_size)).unwrap();
    let mut contract_context = ContractContext::new(contract_identifier.clone());

    let (pre_contract_opt, contract) = gen(function, scale, input_size);

    let contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
        Ok(res) => res,
        Err(error) => {
            panic!("Parsing error: {}", error.diagnostic.message);
        }
    };

    match pre_contract_opt {
        Some(pre_contract) => {
            let pre_contract_identifier =
                QualifiedContractIdentifier::local(&*format!("pre{}", input_size)).unwrap();
            let pre_contract_ast =
                match ast::build_ast(&pre_contract_identifier, &pre_contract, &mut ()) {
                    Ok(res) => res,
                    Err(error) => {
                        panic!("Parsing error: {}", error.diagnostic.message);
                    }
                };
            global_context
                .execute(|g| eval_all(&pre_contract_ast.expressions, &mut contract_context, g))
                .unwrap();
        }
        _ => {}
    }

    group.throughput(Throughput::Bytes(input_size.clone() as u64));
    group.bench_with_input(
        BenchmarkId::from_parameter(input_size),
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
    _i: u16,
    _c: &mut LimitedCostTracker,
) {
}

fn bench_analysis<F, G>(
    c: &mut Criterion,
    function: ClarityCostFunction,
    scale: u16,
    input_sizes: Vec<u16>,
    setup_code: G,
    code_to_bench: F,
) where
    F: Fn(&mut ContractAST, &mut TypingContext, &mut TypeChecker, u16, &mut LimitedCostTracker),
    G: Fn(&mut ContractAST, &mut TypingContext, &mut TypeChecker, u16, &mut LimitedCostTracker),
{
    let mut group = c.benchmark_group(function.to_string());

    for input_size in input_sizes.iter() {
        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();

        let (_, contract) = gen(function, scale, *input_size);

        let mut contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };

        // use warmed up marf
        let headers_db = SimHeadersDB::new();
        let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
        let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));

        let mut local_context = TypingContext::new();
        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut analysis_db = marf_store.as_analysis_db();
        let mut type_checker = TypeChecker::new(&mut analysis_db, cost_tracker.clone());

        group.throughput(Throughput::Bytes(*input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(input_size),
            &input_size,
            |b, &_| {
                setup_code(
                    &mut contract_ast,
                    &mut local_context,
                    &mut type_checker,
                    *input_size,
                    &mut cost_tracker,
                );
                b.iter(|| {
                    code_to_bench(
                        &mut contract_ast,
                        &mut local_context,
                        &mut type_checker,
                        *input_size,
                        &mut cost_tracker,
                    );
                })
            },
        );
    }
}

fn bench_analysis_pass<F>(c: &mut Criterion, function: AnalysisCostFunction, code_to_bench: F) -> ()
where
    F: Fn(&mut ContractAnalysis, &mut AnalysisDatabase) -> CheckResult<()>,
{
    let mut group = c.benchmark_group(function.to_string());

    for input_size in INPUT_SIZES_ANALYSIS_PASS.iter() {
        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();

        let (_, contract) = gen_analysis_pass(function, 1, *input_size);
        let contract_size = contract.len();

        let mut contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
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
        );

        // use warmed up marf
        let headers_db = SimHeadersDB::new();
        let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
        let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));

        let mut analysis_db = marf_store.as_analysis_db();

        analysis_db.execute::<_, _, ()>(|db| {
            group.throughput(Throughput::Bytes(contract_size as u64));
            group.bench_with_input(
                BenchmarkId::from_parameter(contract_size),
                &contract_size,
                |b, &_| {
                    b.iter(|| {
                        for _ in 0..SCALE {
                            code_to_bench(&mut contract_analysis, db);
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
        let (setup_opt, mut contract) = gen_analysis_pass(function, 1, *input_size);
        let setup_contract = setup_opt.unwrap();
        let pre_contract_identifier =
            QualifiedContractIdentifier::local(&*format!("pre{}", input_size)).unwrap();
        let pre_contract_ast =
            match ast::build_ast(&pre_contract_identifier, &setup_contract, &mut ()) {
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
        );

        // add impl-trait statements
        let principal_data = PrincipalData::Standard(pre_contract_identifier.issuer.clone());
        for i in 0..*input_size {
            let impl_trait = format!(
                "(impl-trait '{}.{}.dummy-trait-{}) ",
                principal_data, pre_contract_identifier.name, i
            );
            contract.push_str(&impl_trait);
        }
        let contract_size = contract.len();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
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
        );

        // use warmed up marf
        let headers_db = SimHeadersDB::new();
        let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
        let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));
        let mut analysis_db = marf_store.as_analysis_db();

        // add defined traits to pre contract analysis
        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut type_checker = TypeChecker::new(&mut analysis_db, cost_tracker.clone());
        let mut typing_context = TypingContext::new();
        for exp in &pre_contract_ast.expressions {
            type_checker.try_type_check_define(exp, &mut typing_context);
        }
        type_checker
            .contract_context
            .into_contract_analysis(&mut pre_contract_analysis);

        // add implemented traits to contract analysis
        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut type_checker = TypeChecker::new(&mut analysis_db, cost_tracker.clone());
        let mut typing_context = TypingContext::new();
        for exp in &contract_ast.expressions {
            type_checker.try_type_check_define(exp, &mut typing_context);
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
                            TraitChecker::run_pass(&mut contract_analysis, db);
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
        let (setup_opt, mut contract) = gen_analysis_pass(function, 1, *input_size);
        let setup_contract = setup_opt.unwrap();
        let pre_contract_identifier =
            QualifiedContractIdentifier::local(&*format!("pre{}", input_size)).unwrap();
        let pre_contract_ast =
            match ast::build_ast(&pre_contract_identifier, &setup_contract, &mut ()) {
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
        );

        // add use-trait statements
        let principal_data = PrincipalData::Standard(pre_contract_identifier.issuer.clone());
        for i in 0..*input_size {
            let impl_trait = format!(
                "(use-trait dummy-trait-{}-alias '{}.{}.dummy-trait-{}) ",
                i, principal_data, pre_contract_identifier.name, i
            );
            contract.push_str(&impl_trait);
        }
        let contract_size = contract.len();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
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
        );

        // use warmed up marf
        let headers_db = SimHeadersDB::new();
        let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
        let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));
        let mut analysis_db = marf_store.as_analysis_db();

        // add defined traits to pre contract analysis
        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut type_checker = TypeChecker::new(&mut analysis_db, cost_tracker.clone());
        let mut typing_context = TypingContext::new();
        for exp in &pre_contract_ast.expressions {
            type_checker.try_type_check_define(exp, &mut typing_context);
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
                            TypeChecker::run_pass(&mut contract_analysis, db);
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
    i: u16,
    input_size: u16,
    context: &TypingContext,
    group: &mut BenchmarkGroup<WallTime>,
) {
    if i != 0 {
        helper_deepen_typing_context(i - 1, input_size, &context.extend().unwrap(), group);
    } else {
        let mut cost_tracker = LimitedCostTracker::new_free();
        let headers_db = SimHeadersDB::new();
        let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
        let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));
        let mut analysis_db = marf_store.as_analysis_db();
        let mut type_checker = TypeChecker::new(&mut analysis_db, cost_tracker.clone());

        group.throughput(Throughput::Bytes(input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(input_size),
            &input_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        type_checker.lookup_variable("dummy", &context);
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
        let mut local_context = TypingContext::new();
        helper_deepen_typing_context(*input_size, *input_size, &local_context, &mut group);
    }
}

fn helper_deepen_local_context(
    i: u16,
    input_size: u16,
    context: &LocalContext,
    group: &mut BenchmarkGroup<WallTime>,
) {
    if i != 0 {
        helper_deepen_local_context(i - 1, input_size, &context.extend().unwrap(), group);
    } else {
        let mut memory_backing_store = MemoryBackingStore::new();
        let clarity_db = memory_backing_store.as_clarity_db();
        let mut global_context =
            GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context = ContractContext::new(contract_identifier.clone());
        let mut call_stack = CallStack::new();
        let mut environment = Environment::new(
            global_context.borrow_mut(),
            &contract_context,
            &mut call_stack,
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
fn bench_ast_cycle_detection(c: &mut Criterion) {
    let function = ClarityCostFunction::AstCycleDetection;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();

        let (_, contract) = gen(function, 1, *input_size);

        let pre_expressions = parser::parse(&contract).unwrap();
        let mut contract_ast = ContractAST::new(contract_identifier.clone(), pre_expressions);
        ExpressionIdentifier::run_pre_expression_pass(&mut contract_ast).unwrap();

        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut def_sorter = DefinitionSorter::new();

        group.throughput(Throughput::Bytes(*input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(input_size),
            &input_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        def_sorter.clear_graph();
                        def_sorter.run(&mut contract_ast, &mut cost_tracker);
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
        // want a warmed-up marf
        let headers_db = SimHeadersDB::new();

        let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
        let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));

        let clarity_db = marf_store.as_clarity_db(&headers_db, &NULL_BURN_STATE_DB);

        let mut global_context =
            GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context = ContractContext::new(contract_identifier.clone());

        let (_, contract) = gen(function, 1, *input_size);
        let mut contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
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
        );

        group.throughput(Throughput::Bytes(*input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(input_size),
            &input_size,
            |b, &_| {
                b.iter(|| {
                    for _ in 0..SCALE {
                        let contract_identifier =
                            QualifiedContractIdentifier::local(&*format!("c{}", rng.gen::<u32>()))
                                .unwrap();
                        environment.initialize_contract_from_ast(
                            contract_identifier.clone(),
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

    let mut memory_backing_store = MemoryBackingStore::new();
    let clarity_db = memory_backing_store.as_clarity_db();
    let mut global_context = GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
    global_context.begin();

    let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
    let mut contract_context = ContractContext::new(contract_identifier.clone());
    let (_, contract) = gen(function, SCALE, 0);
    let mut contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
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
    );
    let mut local_context = LocalContext::new();

    group.throughput(Throughput::Bytes(0));
    group.bench_with_input(BenchmarkId::from_parameter(0), &0, |b, &_| {
        b.iter(|| {
            for expr in &contract_ast.expressions {
                special_principal_of(&[expr.clone()], &mut environment, &local_context);
            }
        })
    });
}

fn bench_analysis_use_trait_entry(c: &mut Criterion) {
    let function = ClarityCostFunction::AnalysisUseTraitEntry;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        let headers_db = SimHeadersDB::new();

        let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
        let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));

        let mut analysis_db = marf_store.as_analysis_db();
        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut type_checker = TypeChecker::new(&mut analysis_db, cost_tracker.clone());

        let (_, contract) = gen(function, 1, *input_size);
        let mut contract_identifier =
            QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
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
        );

        let mut typing_context = TypingContext::new();
        type_checker.try_type_check_define(&contract_ast.expressions[0], &mut typing_context);
        type_checker
            .contract_context
            .into_contract_analysis(&mut contract_analysis);

        type_checker.db.execute(|db| {
            db.insert_contract(&contract_identifier, &contract_analysis);
            let trait_name = ClarityName::try_from("dummy-trait".to_string()).unwrap();
            let trait_id = TraitIdentifier {
                name: trait_name.clone(),
                contract_identifier: contract_identifier.clone(),
            };

            // get the size of the trait
            let trait_sig = db
                .get_defined_trait(&contract_identifier, &trait_name)
                .unwrap()
                .unwrap();
            let type_size = trait_type_size(&trait_sig).unwrap();

            group.throughput(Throughput::Bytes(type_size));
            group.bench_with_input(
                BenchmarkId::from_parameter(type_size),
                &type_size,
                |b, &_| {
                    b.iter(|| {
                        for _ in 0..SCALE {
                            TypeChecker::bench_analysis_use_trait_entry_in_context(db, &trait_id);
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

fn bench_analysis_get_function_entry(c: &mut Criterion) {
    let function = ClarityCostFunction::AnalysisGetFunctionEntry;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        let headers_db = SimHeadersDB::new();

        let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
        let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));

        let mut analysis_db = marf_store.as_analysis_db();
        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut type_checker = TypeChecker::new(&mut analysis_db, cost_tracker.clone());

        let (_, contract) = gen(function, 1, *input_size);
        let mut contract_identifier =
            QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
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
        );

        let mut typing_context = TypingContext::new();
        type_checker.try_type_check_define(&contract_ast.expressions[0], &mut typing_context);
        type_checker
            .contract_context
            .into_contract_analysis(&mut contract_analysis);

        type_checker.db.execute(|db| {
            db.insert_contract(&contract_identifier, &contract_analysis);
            let fn_name = ClarityName::try_from("dummy-fn".to_string()).unwrap();
            let type_size = match db
                .get_read_only_function_type(&contract_identifier, "dummy-fn")
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
        let mut memory_backing_store = MemoryBackingStore::new();
        let clarity_db = memory_backing_store.as_clarity_db();
        let mut global_context =
            GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context = ContractContext::new(contract_identifier.clone());
        let (_, contract) = gen(function, 1, *input_size);
        let mut contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };

        global_context
            .execute(|g| eval_all(&contract_ast.expressions, &mut contract_context, g))
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
        let mut memory_backing_store = MemoryBackingStore::new();
        let clarity_db = memory_backing_store.as_clarity_db();
        let mut global_context =
            GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context = ContractContext::new(contract_identifier.clone());
        let (_, contract) = gen(function, 1, *input_size);
        let mut contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };

        global_context
            .execute(|g| eval_all(&contract_ast.expressions, &mut contract_context, g))
            .unwrap();

        let defined_fn = contract_context.lookup_function("dummy-fn").unwrap();
        let mut arg_list = Vec::new();
        for _ in 0..*input_size {
            arg_list.push(Value::UInt(rng.gen()));
        }

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

// q: should I add more traits to the contract context?
fn bench_analysis_lookup_function_types(c: &mut Criterion) {
    let function = ClarityCostFunction::AnalysisLookupFunctionTypes;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        let mut memory_backing_store = MemoryBackingStore::new();
        let clarity_db = memory_backing_store.as_clarity_db();
        let mut global_context =
            GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context = ContractContext::new(contract_identifier.clone());

        let (_, contract) = gen(function, 1, *input_size);
        let mut contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };
        let mut cost_tracker = LimitedCostTracker::new_free();
        let mut null_store = NullBackingStore::new();
        let mut analysis_db = null_store.as_analysis_db();
        let mut type_checker = TypeChecker::new(&mut analysis_db, cost_tracker.clone());
        global_context
            .execute(|g| eval_all(&contract_ast.expressions, &mut contract_context, g))
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
            .add_trait(trait_clarity_name.clone(), trait_obj);

        // construct trait id
        let trait_id = TraitIdentifier {
            contract_identifier: contract_identifier,
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

    // use warmed up marf
    let headers_db = SimHeadersDB::new();
    let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
    let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));
    let clarity_db = marf_store.as_clarity_db(&headers_db, &NULL_BURN_STATE_DB);

    let mut global_context = GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
    global_context.begin();

    let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
    let mut contract_context = ContractContext::new(contract_identifier.clone());
    let (_, contract) = gen(function, SCALE, SCALE * 2);
    let mut contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
        Ok(res) => res,
        Err(error) => {
            panic!("Parsing error: {}", error.diagnostic.message);
        }
    };
    global_context
        .execute(|g| eval_all(&contract_ast.expressions, &mut contract_context, g))
        .unwrap();

    let mut call_stack = CallStack::new();
    let mut environment = Environment::new(
        global_context.borrow_mut(),
        &contract_context,
        &mut call_stack,
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
        let mut memory_backing_store = MemoryBackingStore::new();
        let clarity_db = memory_backing_store.as_clarity_db();
        let mut global_context =
            GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context = ContractContext::new(contract_identifier.clone());

        let mut call_stack = CallStack::new();

        let mut environment = Environment::new(
            global_context.borrow_mut(),
            &contract_context,
            &mut call_stack,
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
        contract_ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _i: u16,
        _c: &mut LimitedCostTracker,
    ) {
        type_checker.type_map.delete_all();
        for exp in &contract_ast.expressions {
            let arg = [exp.clone()];
            check_special_some(type_checker.borrow_mut(), &arg, &local_context);
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
        contract_ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _i: u16,
        _c: &mut LimitedCostTracker,
    ) {
        type_checker.type_map.delete_all();
        for exp in &contract_ast.expressions {
            let arg = [exp.clone()];
            check_special_is_response(type_checker.borrow_mut(), &arg, &local_context);
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

// q: need review of benching function
fn bench_analysis_visit(c: &mut Criterion) {
    fn eval_type_check(
        contract_ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _i: u16,
        _c: &mut LimitedCostTracker,
    ) {
        for exp in &contract_ast.expressions {
            type_checker.bench_type_check(exp, &local_context);
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

// includes cost of AnalysisVisit in inner type check
fn bench_analysis_bind_name(c: &mut Criterion) {
    fn eval_type_check_define<T: CostTracker>(
        contract_ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _i: u16,
        _c: &mut T,
    ) {
        type_checker.type_map.delete_all();
        for exp in &contract_ast.expressions {
            type_checker.bench_try_type_check_define(&exp, local_context);
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
        contract_ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _is: u16,
        _c: &mut T,
    ) {
        type_checker.type_map.delete_all();
        for exp in &contract_ast.expressions {
            let exp_list = exp.match_list().unwrap();
            check_special_list_cons(type_checker, exp_list, local_context);
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
        i: u16,
        _c: &mut T,
    ) {
        SIZED_TUPLE_SIG.get(&i);
    }

    fn eval_check_special_get<T: CostTracker>(
        _ca: &mut ContractAST,
        _lc: &mut TypingContext,
        type_checker: &mut TypeChecker,
        i: u16,
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
        contract_ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _i: u16,
        _c: &mut T,
    ) {
        type_checker.type_map.delete_all();
        for exp in &contract_ast.expressions {
            let exp_list = exp.match_list().unwrap();
            check_special_merge(type_checker, exp_list, local_context);
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
        _is: u16,
        _c: &mut T,
    ) {
        type_checker.type_map.delete_all();
        for exp in &contract_ast.expressions {
            let exp_list = exp.match_list().unwrap();
            check_special_tuple_cons(type_checker, exp_list, local_context);
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

// includes cost of AnalysisCheckTupleCons
fn bench_analysis_tuple_items_check(c: &mut Criterion) {
    fn eval_check_special_tuple_cons<T: CostTracker>(
        contract_ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _is: u16,
        _c: &mut T,
    ) {
        type_checker.type_map.delete_all();
        for exp in &contract_ast.expressions {
            let arg = [exp.clone()];
            check_special_tuple_cons(type_checker, &arg, local_context);
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
        contract_ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _i: u16,
        _c: &mut T,
    ) {
        type_checker.type_map.delete_all();
        for exp in &contract_ast.expressions {
            let exp_list = exp.match_list().unwrap();
            check_special_let(type_checker, exp_list, local_context);
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
    let mut type_checker = TypeChecker::new(&mut analysis_db, cost_tracker.clone());

    let mut rng = rand::thread_rng();
    let mut fn_names = Vec::new();
    for _ in 0..SCALE {
        let fn_name = match rng.gen_range(0..3) {
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
        input_size: u16,
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
        _is: u16,
        _c: &mut T,
    ) {
        type_checker.type_map.delete_all();
        for expr in &contract_ast.expressions {
            type_checker.inner_type_check(expr, local_context);
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
        i: u16,
        _c: &mut T,
    ) {
        let tuple_type_sig = SIZED_TYPE_SIG.get(&i).unwrap().clone();
        type_checker.function_return_tracker = Some(Some(tuple_type_sig.clone()));
    }

    fn eval_track_return_type<T: CostTracker>(
        _ca: &mut ContractAST,
        _lc: &mut TypingContext,
        type_checker: &mut TypeChecker,
        i: u16,
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
    fn eval_check_special_map<T: CostTracker>(
        contract_ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _i: u16,
        _c: &mut T,
    ) {
        type_checker.type_map.delete_all();
        for exp in &contract_ast.expressions {
            let exp_list = exp.match_list().unwrap();
            check_special_map(type_checker, exp_list, local_context);
        }
    }

    bench_analysis(
        c,
        ClarityCostFunction::AnalysisIterableFunc,
        SCALE,
        INPUT_SIZES.into(),
        dummy_setup_code,
        eval_check_special_map,
    )
}

fn bench_analysis_storage(c: &mut Criterion) {
    let function = ClarityCostFunction::AnalysisStorage;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();

        let (_, contract) = gen(function, 1, *input_size);

        let mut contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
            Ok(res) => res,
            Err(error) => {
                panic!("Parsing error: {}", error.diagnostic.message);
            }
        };

        // use warmed up marf
        let mut cost_tracker = LimitedCostTracker::new_free();
        let headers_db = SimHeadersDB::new();
        let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
        let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));
        let mut analysis_db = marf_store.as_analysis_db();
        let mut type_checker = TypeChecker::new(&mut analysis_db, cost_tracker.clone());

        let mut contract_analyses = Vec::new();
        for exp in &contract_ast.expressions {
            let contract_id = QualifiedContractIdentifier::local("analysis_test").unwrap();
            let cost_tracker = LimitedCostTracker::new_free();
            let exp_list = exp.match_list().unwrap();
            let mut contract_analysis =
                ContractAnalysis::new(contract_id.clone(), exp_list.to_vec(), cost_tracker);
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
                type_checker.type_map.delete_all();
                type_checker.contract_context.clear_variable_types();
                for analysis in &mut contract_analyses {
                    type_checker.run(analysis);
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
        input_size: u16,
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
        _i: u16,
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
        _is: u16,
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
        contract_ast: &mut ContractAST,
        local_context: &mut TypingContext,
        type_checker: &mut TypeChecker,
        _i: u16,
        _c: &mut LimitedCostTracker,
    ) {
        for exp in &contract_ast.expressions {
            let var_name = exp.match_atom().unwrap().to_string();
            type_checker.lookup_variable(&var_name, local_context);
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
    fn setup_fn<T: CostTracker>(
        _ca: &mut ContractAST,
        _lc: &mut TypingContext,
        _tc: &mut TypeChecker,
        input_size: u16,
        _ct: &mut T,
    ) {
        SIZED_CONTRACTS.get(&input_size);
    }

    fn eval_build_ast<T: CostTracker>(
        _ca: &mut ContractAST,
        _lc: &mut TypingContext,
        _tc: &mut TypeChecker,
        input_size: u16,
        cost_tracker: &mut T,
    ) {
        let contract = SIZED_CONTRACTS.get(&input_size).unwrap();
        let contract_id = QualifiedContractIdentifier::transient();
        for _ in 0..SCALE {
            build_ast(&contract_id, &contract, cost_tracker);
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
        INPUT_SIZES_ARITHMETIC.into(),
        false,
    )
}

fn bench_sub(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Sub,
        SCALE,
        INPUT_SIZES_ARITHMETIC.into(),
        false,
    )
}

fn bench_mul(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Mul,
        SCALE,
        INPUT_SIZES_ARITHMETIC.into(),
        false,
    )
}

fn bench_div(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Div,
        SCALE,
        INPUT_SIZES_ARITHMETIC.into(),
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
    bench_with_input_sizes(
        c,
        ClarityCostFunction::And,
        SCALE,
        INPUT_SIZES.into(),
        false,
    )
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
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Secp256k1recover,
        SCALE,
        vec![1],
        false,
    )
}

fn bench_secp256k1verify(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Secp256k1verify,
        SCALE,
        vec![1],
        false,
    )
}

fn bench_create_ft_old(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::CreateFt,
        SCALE.into(),
        vec![1],
        false,
    )
}

// note: verify that we want a warmed-up marf for this
fn bench_create_ft(c: &mut Criterion) {
    let function = ClarityCostFunction::CreateFt;
    let mut group = c.benchmark_group(function.to_string());

    // want a warmed-up marf
    let headers_db = SimHeadersDB::new();

    let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
    let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));

    let clarity_db = marf_store.as_clarity_db(&headers_db, &NULL_BURN_STATE_DB);

    let mut global_context = GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
    global_context.begin();

    let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
    let mut contract_context = ContractContext::new(contract_identifier.clone());

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
    bench_with_input_sizes(c, ClarityCostFunction::FtMint, SCALE.into(), vec![1], false)
}

fn bench_ft_transfer(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::FtTransfer,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_ft_balance(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::FtBalance,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_ft_supply(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::FtSupply,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_ft_burn(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FtBurn, SCALE.into(), vec![1], false)
}

// note: verify that we want a warmed-up marf for this
fn bench_create_nft(c: &mut Criterion) {
    let function = ClarityCostFunction::CreateNft;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        // want a warmed-up marf
        let headers_db = SimHeadersDB::new();

        let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
        let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));

        let clarity_db = marf_store.as_clarity_db(&headers_db, &NULL_BURN_STATE_DB);

        let mut global_context =
            GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context = ContractContext::new(contract_identifier.clone());

        let asset_type = SIZED_TYPE_SIG.get(input_size).unwrap();
        group.throughput(Throughput::Bytes(*input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(input_size),
            &input_size,
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
        vec![1],
        false,
    )
}

fn bench_nft_transfer(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::NftTransfer,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_nft_owner(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::NftOwner,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_nft_burn(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::NftBurn,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_is_none(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IsNone, SCALE.into(), vec![1], false)
}

fn bench_is_some(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IsSome, SCALE.into(), vec![1], false)
}

fn bench_is_ok(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IsOkay, SCALE.into(), vec![1], false)
}

fn bench_is_err(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IsErr, SCALE.into(), vec![1], false)
}

fn bench_unwrap(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Unwrap, SCALE.into(), vec![1], false)
}

fn bench_unwrap_ret(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::UnwrapRet,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_unwrap_err(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::UnwrapErr,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_unwrap_err_or_ret(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::UnwrapErrOrRet,
        SCALE.into(),
        vec![1],
        false,
    )
}

// note: verify that we want a warmed-up marf for this
// note: time to clone the type signature for the value may be significant
fn bench_create_map(c: &mut Criterion) {
    let function = ClarityCostFunction::CreateMap;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        // want a warmed-up marf
        let headers_db = SimHeadersDB::new();

        let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
        let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));

        let clarity_db = marf_store.as_clarity_db(&headers_db, &NULL_BURN_STATE_DB);

        let mut global_context =
            GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context = ContractContext::new(contract_identifier.clone());

        let key_type = TypeSignature::UIntType;
        let value_type = SIZED_TYPE_SIG.get(input_size).unwrap();
        let total_size = (key_type.type_size().unwrap() + value_type.type_size().unwrap()) as u64;
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

fn bench_create_var_old(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::CreateVar,
        SCALE.into(),
        vec![1],
        false,
    )
}

// note: verify that we want a warmed-up marf for this
// note: time to clone the type signature for the value may be significant
fn bench_create_var(c: &mut Criterion) {
    let function = ClarityCostFunction::CreateVar;
    let mut group = c.benchmark_group(function.to_string());

    for input_size in &INPUT_SIZES {
        // want a warmed-up marf
        let headers_db = SimHeadersDB::new();

        let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
        let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));

        let clarity_db = marf_store.as_clarity_db(&headers_db, &NULL_BURN_STATE_DB);

        let mut global_context =
            GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
        global_context.begin();

        let contract_identifier = QualifiedContractIdentifier::local(&*format!("c{}", 0)).unwrap();
        let mut contract_context = ContractContext::new(contract_identifier.clone());

        let value_type = SIZED_TYPE_SIG.get(input_size).unwrap();
        let value = helper_make_value_for_sized_type_sig(*input_size);
        group.throughput(Throughput::Bytes(*input_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(input_size),
            &input_size,
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

fn bench_set_var(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::SetVar, SCALE.into(), vec![1], false)
}

fn bench_fetch_var(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::FetchVar,
        SCALE.into(),
        INPUT_SIZES.into(),
        false,
    )
}

fn bench_print(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Print, SCALE.into(), vec![1], false)
}

fn bench_if(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::If, SCALE.into(), vec![1], false)
}

fn bench_asserts(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Asserts,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_ok_cons(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::OkCons, SCALE.into(), vec![1], false)
}

fn bench_err_cons(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::ErrCons,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_some_cons(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::SomeCons,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_concat(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Concat, SCALE.into(), vec![1], false)
}

fn bench_as_max_len(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::AsMaxLen,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_begin(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Begin, SCALE.into(), vec![1], false)
}

fn bench_bind_name(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::BindName,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_default_to(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::DefaultTo,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_try(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::TryRet, SCALE.into(), vec![1], false)
}

fn bench_int_cast(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::IntCast,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_set_entry(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::SetEntry,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_fetch_entry(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::FetchEntry,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_match(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Match, SCALE.into(), vec![1], false)
}

fn bench_let(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Let, SCALE.into(), vec![1], false)
}

fn bench_index_of(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::IndexOf,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_element_at(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::ElementAt,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_len(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Len, SCALE.into(), vec![1], false)
}

fn bench_list_cons(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::ListCons,
        SCALE.into(),
        INPUT_SIZES.into(),
        false,
    )
}

fn bench_append(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Append, SCALE.into(), vec![1], false)
}

fn bench_filter(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Filter, SCALE.into(), vec![1], false)
}

// note: this takes a lot of time to run; can shorten the list sizes to make it faster
fn bench_map(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::Map,
        SCALE.into(),
        INPUT_SIZES.into(),
        false,
    )
}

fn bench_fold(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Fold, SCALE.into(), vec![1], false)
}

fn bench_block_info(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::BlockInfo,
        SCALE.into(),
        vec![1],
        true,
    )
}

fn bench_at_block(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::AtBlock, SCALE.into(), vec![1], true)
}

fn bench_load_contract(c: &mut Criterion) {
    let mut group = c.benchmark_group(ClarityCostFunction::LoadContract.to_string());

    let headers_db = SimHeadersDB::new();

    let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
    let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));

    let clarity_db = marf_store.as_clarity_db(&headers_db, &NULL_BURN_STATE_DB);

    let mut owned_env = OwnedEnvironment::new_free(true, clarity_db);
    owned_env.begin();

    let mut env = owned_env.get_exec_environment(None);

    for size in INPUT_SIZES.iter() {
        let contract_identifier =
            QualifiedContractIdentifier::local(format!("contract{}", size).as_str()).unwrap();
        let (_, contract) = gen_read_only_func(*size);

        env.initialize_contract(contract_identifier.clone(), &contract)
            .unwrap();

        group.throughput(Throughput::Bytes(contract.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(contract.len()),
            &contract.len(),
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
        _i: u16,
        cost_tracker: &mut T,
    ) {
        for exp in &contract_ast.expressions {
            TypeSignature::parse_type_repr(exp, cost_tracker);
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
    bench_with_input_sizes(
        c,
        ClarityCostFunction::StxTransfer,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_stx_get_balance(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::StxBalance,
        SCALE.into(),
        vec![1],
        false,
    )
}

fn bench_poison_microblock(c: &mut Criterion) {
    let mut group = c.benchmark_group(ClarityCostFunction::PoisonMicroblock.to_string());

    let headers_db = SimHeadersDB::new();
    let mut marf = setup_chain_state(MARF_SCALE, &headers_db);
    let mut marf_store = marf.begin(&StacksBlockId(as_hash(60)), &StacksBlockId(as_hash(61)));
    let clarity_db = marf_store.as_clarity_db(&headers_db, &NULL_BURN_STATE_DB);

    let mut owned_env = OwnedEnvironment::new_free(true, clarity_db);
    owned_env.begin();
    let mut env = owned_env.get_exec_environment(None);

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
                env.handle_poison_microblock(h1, h1).unwrap();
            }
        })
    });
}

fn bench_contract_call(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::ContractCall,
        SCALE.into(),
        vec![1],
        true,
    )
}

fn bench_contract_of(c: &mut Criterion) {
    bench_with_input_sizes(
        c,
        ClarityCostFunction::ContractOf,
        SCALE.into(),
        vec![1],
        true,
    )
}

criterion_group!(
    benches,
    // bench_add,
    // bench_sub,
    // bench_mul,
    // bench_div,
    // bench_le,
    // bench_leq,
    // bench_ge,
    // bench_geq,
    // bench_and,
    // bench_or,
    // bench_xor,
    // bench_not,
    // bench_eq,
    // bench_mod,
    // bench_pow,
    // bench_sqrti,
    // bench_log2,
    // bench_tuple_get,
    // bench_tuple_merge,
    // bench_tuple_cons,
    bench_hash160,
    // bench_sha256,
    // bench_sha512,
    // bench_sha512t256,
    // bench_keccak256,
    // bench_secp256k1recover,
    // bench_secp256k1verify,
    // bench_create_ft, // g
    // bench_mint_ft, // g
    // bench_ft_transfer, // g
    // bench_ft_balance, // g
    // bench_ft_supply, // g
    // bench_ft_burn, // g
    // bench_create_nft, // g
    // bench_nft_mint, // g
    // bench_nft_transfer, // g
    // bench_nft_owner, // g
    // bench_nft_burn, // g
    // bench_is_none,
    // bench_is_some,
    // bench_is_ok,
    // bench_is_err,
    // bench_unwrap,
    // bench_unwrap_ret,
    // bench_unwrap_err,
    // bench_unwrap_err_or_ret,
    // bench_create_map, // g
    // bench_create_var, // g
    // bench_set_var, // g
    bench_fetch_var, // g
                     bench_print,
                     // bench_if,
                     // bench_asserts,
                     // bench_ok_cons,
                     // bench_some_cons,
                     // bench_err_cons,
                     // bench_concat,
                     // bench_as_max_len,
                     // bench_begin,
                     // bench_bind_name,
                     // bench_default_to,
                     // bench_try,
                     // bench_int_cast,
                     bench_set_entry, // g
                     bench_fetch_entry, // g
                     // bench_match,
                     // bench_let,
                      bench_index_of,
                     // bench_element_at,
                     // bench_len,
                     // bench_list_cons,
                     // bench_append,
                     // bench_filter,
                     // bench_fold,
                     // bench_at_block,
                     // bench_load_contract,
                     // bench_map,
                     // bench_block_info,
                     // bench_lookup_variable_depth,
                     // bench_lookup_variable_size,
                     // bench_lookup_function,
                     // bench_type_parse_step,
                     // bench_analysis_option_cons,
                     // bench_analysis_option_check,
                     // bench_analysis_visit,
                     // bench_analysis_bind_name,
                     // bench_analysis_list_items_check,
                     // bench_analysis_check_tuple_get,
                     // bench_analysis_check_tuple_merge,
                     // bench_analysis_check_tuple_cons,
                     // bench_analysis_tuple_items_check,
                     // bench_analysis_check_let,
                     // bench_analysis_lookup_function,
                     // bench_analysis_lookup_function_types,
                     // bench_analysis_type_annotate,
                     // bench_analysis_iterable_func,
                     // bench_analysis_storage,
                     // bench_analysis_type_check,
                     // bench_analysis_lookup_variable_depth,
                     // bench_analysis_type_lookup,
                     // bench_analysis_lookup_variable_const,
                     // bench_analysis_use_trait_entry,
                     // bench_analysis_get_function_entry,
                     // bench_inner_type_check_cost,
                     // bench_user_function_application,
                     // bench_ast_cycle_detection,
                     // bench_ast_parse,
                     // bench_contract_storage,
                     // bench_principal_of,
                     // bench_stx_transfer,
                     // bench_stx_get_balance,
                     // bench_analysis_pass_read_only, // g
                     // bench_analysis_pass_arithmetic_only_checker, // g
                     // bench_analysis_pass_trait_checker, // g
                     // bench_analysis_pass_type_checker, // g
                     // bench_poison_microblock,
                     // bench_contract_call,
                     // bench_contract_of,
);

criterion_main!(benches);
