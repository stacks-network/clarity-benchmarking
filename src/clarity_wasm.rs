use stackslib::clarity::types::{Address, StacksEpochId};
use stackslib::clarity::vm::analysis::{AnalysisDatabase, ContractAnalysis};
use stackslib::clarity::vm::clarity_wasm::{
    link_host_functions, placeholder_for_type, ClarityWasmContext,
};
use stackslib::clarity::vm::contexts::{ContractContext, GlobalContext, OwnedEnvironment};
use stackslib::clarity::vm::contracts::Contract;
use stackslib::clarity::vm::costs::{CostTracker, ExecutionCost, LimitedCostTracker};
use stackslib::clarity::vm::database::{
    ClarityBackingStore, ClarityDatabase, ClaritySerializable, HeadersDB, MemoryBackingStore,
    NULL_BURN_STATE_DB, NULL_HEADER_DB,
};
use stackslib::clarity::vm::errors::{Error as ClarityError, WasmError};
use stackslib::clarity::vm::types::signatures::TypeSignature::{
    BoolType, IntType, NoType, PrincipalType, TupleType, UIntType,
};
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
use wasmtime::{
    AsContextMut, Caller, Engine, Instance, Linker, Memory, Module, Store, Trap, Val, ValType,
};

use crate::generators::EPOCH_ID;

pub struct WasmVM<'a, 'b> {
    //call_stack: CallStack,
    // context: ClarityWasmContext<'a, 'b>,
    store: Store<ClarityWasmContext<'a, 'b>>,
    linker: Linker<ClarityWasmContext<'a, 'b>>,
    instance: Instance,
}

/// Compile Clarity -> WASM and store it in `contract_context.wasm_module`
pub fn compile(
    global_context: &mut GlobalContext,
    contract_context: &mut ContractContext,
    db: &mut AnalysisDatabase,
    contract: &str,
) -> Result<(), ClarityError> {
    //println!("compiling contract: {contract}");
    let mut compile_result = db
        .execute(|analysis_db| {
            clar2wasm::compile(
                contract,
                &contract_context.contract_identifier,
                LimitedCostTracker::new_free(),
                ClarityVersion::Clarity2,
                EPOCH_ID,
                analysis_db,
            )
        })
        .map_err(|e| ClarityError::Wasm(WasmError::WasmGeneratorError(format!("{e:?}"))))?;

    db.execute(|analysis_db| {
        analysis_db.insert_contract(
            &contract_context.contract_identifier,
            &compile_result.contract_analysis,
        )
    })
    .expect("Failed to insert contract analysis.");

    contract_context.set_wasm_module(compile_result.module.emit_wasm());

    global_context
        .execute(|g| {
            g.database
                .insert_contract_hash(&contract_context.contract_identifier, contract)
        })
        .expect("Failed to insert contract hash.");

    let data_size = contract_context.data_size;
    global_context.database.insert_contract(
        &contract_context.contract_identifier,
        Contract {
            contract_context: contract_context.clone(),
        },
    );
    global_context
        .database
        .set_contract_data_size(&contract_context.contract_identifier, data_size)
        .expect("Failed to set contract data size.");

    global_context.commit().unwrap();

    Ok(())
}

impl<'a, 'b> WasmVM<'a, 'b> {
    /// Must call this with `contract_context.wasm_module` having `Some()` value
    /// Do this by calling `compile()`
    pub fn load_module(
        global_context: &'a mut GlobalContext<'b>,
        contract_context: &'a mut ContractContext,
        call_stack: &'a mut CallStack,
        sponsor: Option<PrincipalData>,
        contract_analysis: Option<&'a ContractAnalysis>,
    ) -> Result<Self, ClarityError> {
        let publisher: PrincipalData = contract_context.contract_identifier.issuer.clone().into();

        let epoch = global_context.epoch_id;
        let init_context = ClarityWasmContext::new_init(
            global_context,
            contract_context,
            call_stack,
            Some(publisher.clone()),
            Some(publisher),
            sponsor.clone(),
            contract_analysis,
        );
        let engine = Engine::default();
        let module = init_context
            .contract_context()
            .with_wasm_module(|wasm_module| {
                Module::from_binary(&engine, wasm_module)
                    .map_err(|e| ClarityError::Wasm(WasmError::UnableToLoadModule(e)))
            })?;
        let mut store = Store::new(&engine, init_context);
        let mut linker = Linker::new(&engine);

        // Link in the host interface functions.
        link_host_functions(&mut linker)?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| ClarityError::Wasm(WasmError::UnableToLoadModule(e)))?;

        let top_level = instance
            .get_func(&mut store, ".top-level")
            .ok_or(ClarityError::Wasm(WasmError::DefinesNotFound))?;

        // Get the return type of the top-level expressions function
        let ty = top_level.ty(&mut store);
        let mut results_iter = ty.results();
        let mut results = vec![];
        while let Some(result_ty) = results_iter.next() {
            results.push(placeholder_for_type(result_ty));
        }

        top_level
            .call(&mut store, &[], results.as_mut_slice())
            .map_err(|e| ClarityError::Wasm(WasmError::Runtime(e)))?;

        Ok(Self {
            store,
            linker,
            instance
        })
    }

    /// Runs the `.top-level` function of a loaded WASM module
    /// Must call `load_module()` to get a `WasmVM` instance
    pub fn run_top_level(&mut self) -> Result<(), ClarityError> {
        let top_level = self.instance
            .get_func(&mut self.store, ".top-level")
            .ok_or(ClarityError::Wasm(WasmError::DefinesNotFound))?;

        // Get the return type of the top-level expressions function
        let ty = top_level.ty(&mut self.store);
        let mut results_iter = ty.results();
        let mut results = vec![];
        while let Some(result_ty) = results_iter.next() {
            results.push(placeholder_for_type(result_ty));
        }

        top_level
            .call(&mut self.store, &[], results.as_mut_slice())
            .map_err(|e| ClarityError::Wasm(WasmError::Runtime(e)))
    }
}

// /// Call a function, don't worry about the return value
// pub fn call_top_level<'a, 'b>(vm: &mut WasmVM<'a, 'b>) -> Result<(), Error> {
// }
