use blockstack_lib::chainstate::stacks::index::MarfTrieId;
use blockstack_lib::chainstate::stacks::{StacksBlockHeader, StacksBlockId};
use blockstack_lib::core::{FIRST_BURNCHAIN_CONSENSUS_HASH, FIRST_STACKS_BLOCK_HASH};
use blockstack_lib::vm::clarity::ClarityInstance;
use blockstack_lib::vm::costs::ExecutionCost;
use blockstack_lib::vm::database::{MarfedKV, NULL_BURN_STATE_DB, NULL_HEADER_DB};
use blockstack_lib::vm::types::{PrincipalData, QualifiedContractIdentifier};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use blockstack_lib::vm::costs::cost_functions::ClarityCostFunction;

const INPUT_SIZES: [usize; 8] = [1, 2, 8, 16, 32, 64, 128, 256];
const SCALE: usize = 1000;

// generate arithmetic function call
fn gen_arithmetic(function_name: &'static str, scale: usize, input_size:usize) -> String {
    let mut body = String::new();

    for _ in 0..scale {
        let args = (0..input_size).map(|x| x.to_string()).collect::<Vec<String>>().join(" ");
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    body
}

// generate clarity code for benchmarking
fn gen(function: ClarityCostFunction, scale: usize, input_size: usize) -> String {
    let mut body = String::new();

    match function {
        ClarityCostFunction::Add => {
            body = gen_arithmetic("+", scale, input_size);
        },
        ClarityCostFunction::Sub => {
            body = gen_arithmetic("-", scale, input_size);
        },
        ClarityCostFunction::Mul => {
            body = gen_arithmetic("*", scale, input_size);
        },
        ClarityCostFunction::Div => {
            body = gen_arithmetic("/", scale, input_size);
        },
        _ => {}
    }

    format!("(define-public (test) (begin {}(ok true)))", body)
}

fn bench(c: &mut Criterion, function: ClarityCostFunction, scale: usize, input_sizes: Vec<usize>) {
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
        group.bench_with_input(BenchmarkId::from_parameter(input_size), input_size, |b, &_| {
            b.iter(|| {
                conn.as_transaction(|tx| {
                    tx.run_contract_call(&p, &contract_identifier, "test", &[], |_, _| false)
                })
                    .unwrap()
            })
        });
    }
}

fn bench_add(c: &mut Criterion) {
    bench(c, ClarityCostFunction::Add, SCALE.into(), INPUT_SIZES.into())
}

criterion_group!(benches, bench_add);
criterion_main!(benches);
