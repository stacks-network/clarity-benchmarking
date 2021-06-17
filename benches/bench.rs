use std::fs;

use benchmarking_lib::generators::gen;
use blockstack_lib::clarity_vm::database::{MemoryBackingStore, marf::MarfedKV};
use blockstack_lib::clarity_vm::clarity::ClarityInstance;
use blockstack_lib::types::proof::ClarityMarfTrieId;
use blockstack_lib::util::hash::Hash160;
use blockstack_lib::vm::ast::ContractAST;
use blockstack_lib::vm::contexts::{GlobalContext, ContractContext};
use blockstack_lib::vm::database::{NULL_BURN_STATE_DB, NULL_HEADER_DB, HeadersDB, ClarityDatabase};
use blockstack_lib::vm::{ast, eval_all, Value};
use blockstack_lib::vm::costs::cost_functions::ClarityCostFunction;
use blockstack_lib::vm::costs::{LimitedCostTracker, ExecutionCost};
use blockstack_lib::vm::types::QualifiedContractIdentifier;
use blockstack_lib::types::chainstate::{BlockHeaderHash, BurnchainHeaderHash, StacksAddress, StacksBlockId, VRFSeed};
use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

const INPUT_SIZES: [u16; 8] = [1, 2, 8, 16, 32, 64, 128, 256];
const MORE_INPUT_SIZES: [u16; 12] = [1, 2, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];
const SCALE: u16 = 100;
const MARF_SCALE: u32 = 100000;

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
        Some(StacksAddress::new(0, Hash160([0u8; 20])))
    }
}


fn as_hash(inp: u32) -> [u8; 32] {
    let mut out = [0; 32];
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

fn eval(contract_ast: &ContractAST, global_context: &mut GlobalContext, contract_context: &mut ContractContext) {
    global_context.execute(|g| eval_all(&contract_ast.expressions, contract_context, g)).unwrap();
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
            let mut marf = setup_chain_state(MARF_SCALE);
            let mut marf_store = marf.begin(
                &StacksBlockId(as_hash(0)),
                &StacksBlockId(as_hash(1)),
            );

            let clarity_db = marf_store.as_clarity_db(&TestHeadersDB, &NULL_BURN_STATE_DB);

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
) where F: Fn(&ContractAST, &mut GlobalContext, &mut ContractContext) {
    let mut global_context = GlobalContext::new(false, clarity_db, LimitedCostTracker::new_free());
    global_context.begin();

    let contract_identifier =
        QualifiedContractIdentifier::local(&*format!("c{}", input_size)).unwrap();
    let mut contract_context = ContractContext::new(contract_identifier.clone());

    let contract = gen(function, scale, input_size);

    let contract_ast = match ast::build_ast(&contract_identifier, &contract, &mut ()) {
        Ok(res) => res,
        Err(error) => {
            panic!("Parsing error: {}", error.diagnostic.message);
        }
    };

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
    bench_with_input_sizes(c, ClarityCostFunction::FtTransfer, SCALE.into(), vec![1], false)
}

fn bench_ft_balance(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FtBalance, SCALE.into(), vec![1], false)
}

fn bench_ft_supply(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FtSupply, SCALE.into(), vec![1], false)
}

fn bench_ft_burn(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FtBurn, SCALE.into(), vec![1], false)
}

fn bench_create_nft(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::CreateNft, SCALE.into(), vec![1], false)
}

fn bench_nft_mint(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::NftMint, SCALE.into(), vec![1], false)
}

fn bench_nft_transfer(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::NftTransfer, SCALE.into(), vec![1], false)
}

fn bench_nft_owner(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::NftOwner, SCALE.into(), vec![1], false)
}

fn bench_nft_burn(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::NftBurn, SCALE.into(), vec![1], false)
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
    bench_with_input_sizes(c, ClarityCostFunction::UnwrapRet, SCALE.into(), vec![1], false)
}

fn bench_unwrap_err(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::UnwrapErr, SCALE.into(), vec![1], false)
}

fn bench_unwrap_err_or_ret(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::UnwrapErrOrRet, SCALE.into(), vec![1], false)
}

fn bench_create_map(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::CreateMap, SCALE.into(), vec![1], false)
}

fn bench_create_var(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::CreateVar, SCALE.into(), vec![1], false)
}

fn bench_set_var(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::SetVar, SCALE.into(), vec![1], false)
}

fn bench_fetch_var(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FetchVar, SCALE.into(), vec![1], false)
}

fn bench_print(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Print, SCALE.into(), vec![1], false)
}

fn bench_if(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::If, SCALE.into(), vec![1], false)
}

fn bench_asserts(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Asserts, SCALE.into(), vec![1], false)
}

fn bench_ok_cons(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::OkCons, SCALE.into(), vec![1], false)
}

fn bench_err_cons(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::ErrCons, SCALE.into(), vec![1], false)
}

fn bench_some_cons(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::SomeCons, SCALE.into(), vec![1], false)
}

fn bench_concat(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Concat, SCALE.into(), vec![1], false)
}

fn bench_as_max_len(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::AsMaxLen, SCALE.into(), vec![1], false)
}

fn bench_begin(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Begin, SCALE.into(), vec![1], false)
}

fn bench_bind_name(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::BindName, SCALE.into(), vec![1], false)
}

fn bench_default_to(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::DefaultTo, SCALE.into(), vec![1], false)
}

fn bench_try(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::TryRet, SCALE.into(), vec![1], false)
}

fn bench_int_cast(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IntCast, SCALE.into(), vec![1], false)
}

fn bench_set_entry(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::SetEntry, SCALE.into(), vec![1], false)
}

fn bench_fetch_entry(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::FetchEntry, SCALE.into(), vec![1], false)
}

fn bench_match(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Match, SCALE.into(), vec![1], false)
}

fn bench_let(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Let, SCALE.into(), vec![1], false)
}

fn bench_index_of(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::IndexOf, SCALE.into(), vec![1], false)
}

fn bench_element_at(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::ElementAt, SCALE.into(), vec![1], false)
}

fn bench_len(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Len, SCALE.into(), vec![1], false)
}

fn bench_list_cons(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::ListCons, SCALE.into(), vec![1], false)
}

fn bench_append(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Append, SCALE.into(), vec![1], false)
}

fn bench_filter(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Filter, SCALE.into(), vec![1], false)
}

fn bench_map(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Map, SCALE.into(), vec![1], false)
}

fn bench_fold(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::Fold, SCALE.into(), vec![1], false)
}

// TODO: figure out more accurate benchmark that doesn't use TestHeadersDB
fn bench_block_info(c: &mut Criterion) {
    bench_with_input_sizes(c, ClarityCostFunction::BlockInfo, SCALE.into(), vec![1], true)
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
    // bench_hash160,
    // bench_sha256,
    // bench_sha512,
    // bench_sha512t256,
    // bench_keccak256,
    // bench_secp256k1recover,
    // bench_secp256k1verify,
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
    // bench_is_none,
    // bench_is_some,
    // bench_is_ok,
    // bench_is_err,
    // bench_unwrap,
    bench_unwrap_ret,
    // bench_unwrap_err,
    // bench_unwrap_err_or_ret,
    // bench_create_map,
    // bench_create_var,
    // bench_set_var,
    // bench_fetch_var,
    // bench_print,
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
    // bench_set_entry,
    // bench_fetch_entry,
    // bench_match,
    // bench_let,
    // bench_index_of,
    // bench_element_at,
    // bench_len,
    // bench_list_cons,
    // bench_append,
    // bench_filter,
    // bench_fold,
    // bench_block_info,
);

criterion_main!(benches);
