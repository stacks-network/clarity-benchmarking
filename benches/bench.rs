use blockstack_lib::chainstate::stacks::index::MarfTrieId;
use blockstack_lib::chainstate::stacks::{StacksBlockHeader, StacksBlockId};
use blockstack_lib::core::{FIRST_BURNCHAIN_CONSENSUS_HASH, FIRST_STACKS_BLOCK_HASH};
use blockstack_lib::vm::clarity::ClarityInstance;
use blockstack_lib::vm::costs::ExecutionCost;
use blockstack_lib::vm::database::{MarfedKV, NULL_BURN_STATE_DB, NULL_HEADER_DB};
use blockstack_lib::vm::types::{PrincipalData, QualifiedContractIdentifier};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

pub fn gen_add(scale: usize, input_size: usize) -> String {
    let mut body = String::new();

    for _ in 0..scale {
        body.push_str("(+ 1 2) ")
    }

    format!("(define-public (test) (begin {}(ok true)))", body)
}

pub fn bench_add(c: &mut Criterion) {
    let marf = MarfedKV::temporary();
    let mut clarity_instance = ClarityInstance::new(marf, ExecutionCost::max_value());

    let p = PrincipalData::from(
        PrincipalData::parse_standard_principal("SM2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKQVX8X0G")
            .unwrap(),
    );

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

    let mut group = c.benchmark_group("add");

    for scale in [50, 100, 150].iter() {
        let contract_identifier =
            QualifiedContractIdentifier::local(&*format!("c{}", scale)).unwrap();
        let contract = gen_add(*scale, 2);

        conn.as_transaction(|tx| {
            let (ct_ast, _ct_analysis) = tx
                .analyze_smart_contract(&contract_identifier, &contract)
                .unwrap();
            tx.initialize_smart_contract(&contract_identifier, &ct_ast, &*contract, |_, _| false);
        });

        group.throughput(Throughput::Bytes(*scale as u64));
        group.bench_with_input(BenchmarkId::from_parameter(scale), scale, |b, &scale| {
            b.iter(|| {
                conn.as_transaction(|tx| {
                    tx.run_contract_call(&p, &contract_identifier, "test", &[], |_, _| false)
                })
                .unwrap()
            })
        });
    }
}

criterion_group!(benches, bench_add);
criterion_main!(benches);
