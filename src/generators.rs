#[allow(unused_variables)]
#[allow(unused_imports)]
use std::cmp;
use stackslib::burnchains::PrivateKey;
use stackslib::util::secp256k1::{Secp256k1PrivateKey, Secp256k1PublicKey};
use stackslib::clarity::vm::ast::build_ast_pre;
use stackslib::clarity::vm::ast::definition_sorter::DefinitionSorter;
use stackslib::clarity::vm::costs::LimitedCostTracker;
use stackslib::clarity::vm::costs::cost_functions::{AnalysisCostFunction, ClarityCostFunction};
use stackslib::clarity::vm::database::ClaritySerializable;
use rand::distributions::Uniform;
use rand::prelude::SliceRandom;
use rand::{Rng, RngCore};

use stackslib::address::AddressHashMode;
use stackslib::chainstate::stacks::{StacksPublicKey, C32_ADDRESS_VERSION_TESTNET_SINGLESIG};
use stackslib::types::chainstate::StacksAddress;
use stackslib::util::hash::to_hex;
use stackslib::clarity::vm::analysis::contract_interface_builder::ContractInterfaceAtomType::{
};
use stackslib::clarity::vm::types::signatures::TypeSignature::{
    BoolType, IntType, PrincipalType, TupleType, UIntType,
};
use stackslib::clarity::vm::types::{ASCIIData, CharType, OptionalData, QualifiedContractIdentifier, SequenceData, TupleData, TupleTypeSignature, TypeSignature};
use stackslib::clarity::vm::{ClarityName, Value};
use lazy_static::lazy_static;
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use stackslib::clarity::vm::ClarityVersion;
use stackslib::clarity::vm::execute;
use stackslib::clarity::vm::types::StacksAddressExtensions;
use stackslib::clarity::codec::StacksMessageCodec;

lazy_static! {
    pub static ref TUPLE_NAMES: Vec<String> = create_tuple_names(16);
}

// This index_block_hash corresponds to block height 7 in the chainstate DB
// pub const READ_TIP: &str = "df3d88b6d70cecc1d94442c4dc23ccc5a4466101454002d26620f972f0b30299";
pub const READ_TIP: &str = "24d3f81a0bad21b113af437dfc0872824d39cd6ad46d0a79fc80db3bcedbd687";

fn string_to_value(s: String) -> Value {
    execute(s.as_str()).unwrap().unwrap()
}

fn size_of_value(s: String) -> u64 {
    let v = string_to_value(s);
    v.serialize().len() as u64 / 2
}

fn serialized_size(s: String) -> u64 {
    let v = string_to_value(s);
    v.serialized_size() as u64
}

#[derive(Debug)]
pub struct GenOutput {
    pub setup: Option<String>,
    pub body: String,
    pub input_size: u64,
}

impl GenOutput {
    pub fn new(setup: Option<String>, body: String, input_size: u64) -> Self {
        GenOutput {
            setup,
            body,
            input_size,
        }
    }
}

fn create_tuple_names(len: u16) -> Vec<String> {
    let mut names = Vec::new();
    for _ in 0..len {
        names.push(helper_generate_rand_char_string(5));
    }
    names
}

// make values for analysis functions
fn make_tuple_pair(pairs: u64) -> Value {
    let mut data = Vec::new();
    for i in 0..pairs {
        let name = TUPLE_NAMES[i as usize].clone();
        let val = Value::Bool(true);
        data.push((ClarityName::try_from(name).unwrap(), val));
    }
    let td = TupleData::from_data(data).unwrap();
    Value::Tuple(td)
}

pub fn make_sized_value(input_size: u64) -> Value{
    match input_size {
        1 => Value::Bool(true),
        2 => Value::some(Value::Bool(true)).unwrap(),
        8 => Value::Sequence(SequenceData::String(CharType::ASCII(ASCIIData {
            data: vec![5, 9, 10, 6],
        }))),
        n => {
            // assuming n is a multiple of 16
            let mult = n / 16;
            make_tuple_pair(mult)
        }
    }
}

pub fn make_sized_values_map(input_sizes: Vec<u64>) -> HashMap<u64, Value> {
    let mut ret_map = HashMap::new();
    for i in input_sizes {
        let val = make_sized_value(i);
        ret_map.insert(i, val);
    }
    ret_map
}

pub fn make_type_sig_list_of_size(input_sizes: Vec<u64>) -> HashMap<u64, Vec<TypeSignature>> {
    let mut ret_map = HashMap::new();
    for i in input_sizes {
        let val = vec![TypeSignature::BoolType; i as usize];
        ret_map.insert(i, val);
    }
    ret_map
}

pub fn make_clarity_type_for_sized_value(input_size: u64) -> String {
    match input_size {
        1 => "bool".to_string(),
        2 => "(optional bool)".to_string(),
        8 => "(string-ascii 4)".to_string(),
        n => {
            // assuming n is a multiple of 16
            let mult = n / 16;
            // (tuple (key-name-0 key-type-0) (key-name-1 key-type-1) ...)
            let mut key_pairs = String::new();
            for i in 0..mult {
                let name = TUPLE_NAMES[i as usize].clone();
                key_pairs.push_str(&*format!("({} bool) ", name));
            }
            format!("(tuple {})", key_pairs)
        }
    }
}

// make contract for ast parse
fn make_clarity_statement_for_sized_contract(mult: u64) -> (String, u64) {
    let mut rng = rand::thread_rng();
    let contract = (0..mult)
        .map(|_x| {
            format!(
                "(list u{} u{}) ",
                rng.gen_range(10..99),
                rng.gen_range(100..999)
            )
        })
        .collect::<String>();

    (contract.clone(), contract.len() as u64)
}

fn make_sized_contract(input_size: u64) -> (String, u64) {
    match input_size {
        1 => ("1".to_string(), 1),
        2 => ("u8".to_string(), 2),
        8 => ("(no-op) ".to_string(), 8),
        n => {
            // assuming n is a multiple of 16
            let mult = n / 16;
            let contract = make_clarity_statement_for_sized_contract(mult);
            (contract.0, contract.1)
        }
    }
}

pub fn make_sized_contracts_map(input_sizes: Vec<u64>) -> HashMap<u64, String> {
    let mut ret_map = HashMap::new();
    for i in input_sizes {
        let val = make_sized_contract(i);
        ret_map.insert(val.1, val.0);
    }
    ret_map
}

// make tuple type sigs for AnalysisCheckTupleGet
fn make_tuple_sig(input_size: u64) -> TupleTypeSignature {
    let mut rng = rand::thread_rng();
    let type_list = [IntType, UIntType, BoolType, PrincipalType];
    let mut type_map = Vec::new();
    for i in 0..input_size {
        let name = ClarityName::try_from(format!("id{}", i)).unwrap();
        let type_sig = type_list.choose(&mut rng).unwrap().clone();
        type_map.push((name, type_sig));
    }
    TupleTypeSignature::try_from(type_map).unwrap()
}

pub fn make_sized_tuple_sigs_map(input_sizes: Vec<u64>) -> HashMap<u64, TupleTypeSignature> {
    let mut ret_map = HashMap::new();
    for i in input_sizes {
        let val = make_tuple_sig(i);
        ret_map.insert(i, val);
    }
    ret_map
}

fn helper_make_clarity_type_for_sized_type_sig(input_size: u64) -> String {
    match input_size {
        1 => "bool".to_string(),
        2 => "(optional bool)".to_string(),
        n => {
            let mult = n / 8;
            // (tuple (key-name-0 key-type-0) (key-name-1 key-type-1) ...)
            let mut key_pairs = String::new();
            for i in 0..mult {
                // the id name is constructed like this to ensure key names all have equal length
                let id_name = if i < 10 {
                    "id--"
                } else if i < 100 {
                    "id-"
                } else {
                    "id"
                };
                let name = format!("{}{}", id_name, i);
                key_pairs.push_str(&*format!("({} bool) ", name));
            }
            format!("(tuple {})", key_pairs)
        }
    }
}

fn helper_make_clarity_value_for_sized_type_sig(input_size: u64) -> String {
    let mut rng = rand::thread_rng();
    match input_size {
        1 => format!("{}", rng.gen::<bool>()),
        2 => format!("(some {})", rng.gen_bool(0.5)),
        n => {
            let mult = n / 8;
            // assume n is a multiple of 8
            let tuple_vals = (0..mult)
                .map(|i| {
                    // the id name is constructed like this to ensure key names all have equal length
                    let id_name = if i < 10 {
                        "id--"
                    } else if i < 100 {
                        "id-"
                    } else {
                        "id"
                    };
                    format!("({}{} {})", id_name, i, rng.gen::<bool>())
                })
                .collect::<Vec<String>>()
                .join(" ");

            format!("(tuple {}) ", tuple_vals)
        }
    }
}

pub fn helper_make_value_for_sized_type_sig(input_size: u64) -> Value {
    let mut rng = rand::thread_rng();
    match input_size {
        1 => Value::Bool(rng.gen()),
        2 => Value::Optional(OptionalData {
            data: Some(Box::new(Value::Bool(rng.gen_bool(0.5)))),
        }),
        n => {
            let mult = n / 8;
            // assume n is a multiple of 8
            let mut type_map = BTreeMap::new();
            let mut data_map = BTreeMap::new();
            let value_type_sig = TypeSignature::BoolType;
            for i in 0..mult {
                let id_name = if i < 10 {
                    format!("id--{}", i)
                } else {
                    format!("id-{}", i)
                };
                let clarity_name = ClarityName::try_from(id_name).unwrap();
                let value = Value::Bool(rng.gen());
                type_map.insert(clarity_name.clone(), value_type_sig.clone());
                data_map.insert(clarity_name, value);
            }
            let tuple_data = TupleData {
                type_signature: TupleTypeSignature::try_from(type_map).unwrap(),
                data_map,
            };

            Value::Tuple(tuple_data)
        }
    }
}

// make sized type sigs for AnalysisTypeCheck
fn make_sized_type_sig(input_size: u64) -> TypeSignature {
    let mut rng = rand::thread_rng();
    match input_size {
        1 => TypeSignature::BoolType,
        2 => TypeSignature::OptionalType(Box::new(TypeSignature::BoolType)),
        n => {
            // assume n is a multiple of 8
            let type_list = [TypeSignature::BoolType];
            let mut type_map = Vec::new();
            let mult = n / 8;
            for i in 0..mult {
                // the id name is constructed like this to ensure key names all have equal length
                let id_name = if i < 10 {
                    "id--"
                } else if i < 100 {
                    "id-"
                } else {
                    "id"
                };
                let name = ClarityName::try_from(format!("{}{}", id_name, i)).unwrap();
                let type_sig = type_list.choose(&mut rng).unwrap().clone();
                type_map.push((name, type_sig));
            }
            TupleType(TupleTypeSignature::try_from(type_map).unwrap())
        }
    }
}

pub fn make_sized_type_sig_map(input_sizes: Vec<u64>) -> HashMap<u64, TypeSignature> {
    let mut ret_map = HashMap::new();
    for i in input_sizes {
        let val = make_sized_type_sig(i);
        ret_map.insert(i, val);
    }
    ret_map
}

pub fn helper_make_sized_clarity_value(input_size: u64) -> String {
    let mut rng = rand::thread_rng();

    match input_size {
        1 => "true".to_string(),
        2 => "(some true) ".to_string(),
        n => {
            // assuming n is a multiple of 8
            let mut val = String::new();
            let mult = n / 8;
            for _ in 0..mult {
                let name = helper_generate_rand_char_string(5);
                val.push_str(&*format!("({} {}) ", name, rng.gen::<u16>()));
            }
            format!("(tuple {}) ", val).to_string()
        }
    }
}

/// cost_function: Add, Sub, Mul, Div, Sqrti, Log2, Mod, Xor, bit-and, bit-or
/// input_size: number of arguments
pub fn gen_arithmetic(
    function_name: &'static str,
    scale: u16,
    input_size: u64,
) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    let s = match function_name {
        "/" => 1,
        _ => 0,
    };

    for _ in 0..scale {
        let args = (0..input_size)
            .map(|i| rng.gen_range(s..i16::MAX).to_string())
            .collect::<Vec<String>>()
            .join(" ");
        body.push_str(&format!("({} {}) ", function_name, args));
    }

    GenOutput::new(None, body, input_size)
}

/// cost_function: Pow
/// input_size: double arg function
fn gen_pow(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let n1: u16 = rng.gen();
        let n2: u8 = rng.gen_range(0..8);
        body.push_str(&*format!("(pow u{} u{}) ", n1, n2));
    }

    GenOutput::new(None, body, 2)
}

/// cost_function: Le, Leq, Ge, Geq
/// input_size: number of machine words (word size is 128 bits)
fn gen_cmp(function_name: &'static str, scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        if input_size < 16 {
            let n1: u128 = rng.gen();
            let n2: u128 = rng.gen();
            let code = format!("({} u{} u{}) ", function_name, n1, n2);
            body.push_str(&*code);
        }
        else {
            loop {
                let (val_1, type_1) = helper_generate_random_sequence_fixed_len(input_size);
                if val_1.find("list").is_some() {
                    continue;
                }
                let val_2 = helper_generate_random_sequence_fixed_len_fixed_type(input_size, &type_1);
                let code = format!("({} {} {}) ", function_name, val_1, val_2);
                body.push_str(&*code);
                break;
            }
        }
    }

    GenOutput::new(None, body, input_size)
}

/// cost_function: And, Or, Not, Eq
/// input_size: number of arguments
/// input_size eq: sum of serialized_size of arguments. booleans are size 1, so input_size
/// is fine here.
fn gen_logic(function_name: &'static str, scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let args = (0..input_size)
            .map(|_| match function_name {
                "and" => format!("true"),
                "or" => format!("false"),
                _ => format!("{}", rng.gen_bool(0.5)),
            })
            .collect::<Vec<String>>()
            .join(" ");

        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    GenOutput::new(None, body, input_size)
}

/// cost_function: Xor
/// input_size: double arg function
fn gen_xor(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let args = match rng.gen_range(0..=1) {
            0 => {
                // uint
                let x: u128 = rng.gen();
                let y: u128 = rng.gen();
                format!("u{} u{}", x, y)
            }
            1 => {
                // int
                let x: i128 = rng.gen();
                let y: i128 = rng.gen();
                format!("{} {}", x, y)
            }
            _ => {
                unreachable!("should only be generating numbers in the range 0..=1.")
            }
        };
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    GenOutput::new(None, body, 2)
}

/// cost_function: bit-shift-left
/// input_size: double arg function
fn gen_lshift(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let args = match rng.gen_range(0..=1) {
            0 => {
                // uint
                let x: u128 = rng.gen();
                let y: u128 = rng.gen();
                format!("u{} u{}", x, y)
            }
            1 => {
                // int
                let x: i128 = rng.gen();
                let y: u128 = rng.gen();
                format!("{} u{}", x, y)
            }
            _ => {
                unreachable!("should only be generating numbers in the range 0..=1.")
            }
        };
        body.push_str(&*format!("(bit-shift-left {}) ", args));
    }

    GenOutput::new(None, body, 2)
}

/// cost_function: bit-shift-right
/// input_size: double arg function
fn gen_rshift(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let args = match rng.gen_range(0..=1) {
            0 => {
                // uint
                let x: u128 = rng.gen();
                let y: u128 = rng.gen();
                format!("u{} u{}", x, y)
            }
            1 => {
                // int
                let x: i128 = rng.gen();
                let y: u128 = rng.gen();
                format!("{} u{}", x, y)
            }
            _ => {
                unreachable!("should only be generating numbers in the range 0..=1.")
            }
        };
        body.push_str(&*format!("(bit-shift-right {}) ", args));
    }

    GenOutput::new(None, body, 2)
}

/// This function generates a random hex string of size n.
fn helper_generate_rand_hex_string(n: usize) -> String {
    let hex_chars = [
        "a", "b", "c", "d", "e", "f", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
    ];

    let hex_range = Uniform::new_inclusive(0, 15);
    rand::thread_rng()
        .sample_iter(&hex_range)
        .take(n)
        .map(|x| hex_chars[x])
        .collect::<String>()
}

/// This function generates a random numeric string of size n.
pub fn helper_generate_rand_numeric_string(n: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..n)
        .map(|_| rng.gen_range(b'0'..b'9') as char)
        .collect::<String>()
}

/// This function generates a random char string of size n.
pub fn helper_generate_rand_char_string(n: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..n)
        .map(|_| rng.gen_range(b'a'..b'z') as char)
        .collect::<String>()
}

/// This function generates a hash function (scaled) with an argument that either has type uint, int, or buff (randomly chosen)
///
/// cost_function: Hash160, Sha256, Sha512, Sha512t256, Keccak256
/// input_size: single arg function
fn gen_hash(function_name: &'static str, scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    let arg = match input_size {
        // size of (u)ints
        17 => {
            match rng.gen_range(0..=2) {
                0 => {
                    // uint
                    let x: u128 = rng.gen();
                    format!("u{}", x)
                },
                1 => {
                    // int
                    let x: i128 = rng.gen();
                    format!("{}", x)
                },
                2 => {
                    let buff = helper_gen_clarity_value("buff", 0, 128, None);
                    format!(r##"{}"##, buff.0)
                },
                _ => {
                    unreachable!("should only be generating numbers in the range 0..=2.")
                }
            }
        },
        _ => {
            let buff = helper_gen_clarity_value("buff", 0, input_size, None);
            format!(r##"{}"##, buff.0)
        }
    };

    for _ in 0..scale {
        body.push_str(&*format!("({} {}) ", function_name, arg));
    }

    GenOutput::new(None, body, serialized_size(arg))
}


// The bool verify is used to distinguish which cost function is being tested.
/// cost_function: Secp256k1recover, Secp256k1verify
/// input_size: 0
fn gen_secp256k1(
    function_name: &'static str,
    scale: u16,
    verify: bool,
) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let mut msg = [0u8; 32];
        rng.fill_bytes(&mut msg);

        let privk = Secp256k1PrivateKey::new();
        let sig = privk.sign(&msg).unwrap();
        let secp256k1_sig = sig.to_secp256k1_recoverable().unwrap();
        let (rec_id, sig_bytes) = secp256k1_sig.serialize_compact();
        let rec_id_byte = rec_id.to_i32() as u8;
        let mut sig_bytes_vec = sig_bytes.to_vec();
        sig_bytes_vec.push(rec_id_byte);

        let args = if verify {
            let pubk = Secp256k1PublicKey::from_private(&privk);
            format!(
                "0x{} 0x{} 0x{}",
                to_hex(&msg),
                to_hex(&sig_bytes_vec),
                pubk.to_hex()
            )
        } else {
            format!("0x{} 0x{}", to_hex(&msg), to_hex(&sig_bytes_vec))
        };

        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    GenOutput::new(None, body, 1)
}

/// ////////////////////////////////////
/// FUNGIBLE TOKEN GENERATOR FUNCTIONS
/// ////////////////////////////////////

fn helper_define_fungible_token_statement() -> (String, String) {
    let mut rng = rand::thread_rng();
    let token_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let args = match rng.gen_range(0..=1) {
        0 => {
            // no supply arg
            format!("{}", token_name)
        }
        1 => {
            // provide supply arg
            let supply: u128 = rng.gen();
            format!("{} u{}", token_name, supply)
        }
        _ => {
            unreachable!("should only be generating numbers in the range 0..=1.")
        }
    };
    let statement = format!("(define-fungible-token {}) ", args);
    (statement, token_name)
}

/// cost_function: CreateFt
/// input_size: 0
fn gen_create_ft(_function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();

    for _ in 0..scale {
        let (statement, _) = helper_define_fungible_token_statement();
        body.push_str(&statement);
    }

    GenOutput::new(None, body, 1)
}

fn helper_create_principal_in_hex() -> String {
    let privk = Secp256k1PrivateKey::new();
    let pubk = Secp256k1PublicKey::from_private(&privk).to_hex();

    format!("0x{} ", pubk)
}

/// Creates a random principal to use in a clarity contract. The output includes the prefixing tick mark.
fn helper_create_principal() -> String {
    let privk = Secp256k1PrivateKey::new();
    let addr = StacksAddress::from_public_keys(
        C32_ADDRESS_VERSION_TESTNET_SINGLESIG,
        &AddressHashMode::SerializeP2PKH,
        1,
        &vec![StacksPublicKey::from_private(&privk)],
    )
    .unwrap();
    let principal_data = addr.to_account_principal();
    format!("'{}", principal_data)
}

/// cost_function: FtMint
/// input_size: 0
fn gen_ft_mint(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let (statement, token_name) = helper_define_fungible_token_statement();

    for _ in 0..scale {
        let amount: u128 = rng.gen_range(1..1000);
        let principal_data = helper_create_principal();
        let args = format!("{} u{} {}", token_name, amount, principal_data);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    GenOutput::new(Some(statement), body, 1)
}

fn helper_create_ft_boilerplate(mint_amount: u16) -> (String, String, String) {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let token_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    body.push_str(&*format!("(define-fungible-token {}) ", token_name));

    let principal_data = helper_create_principal();
    body.push_str(&*format!(
        "(ft-mint? {} u{} {}) ",
        token_name, mint_amount, principal_data
    ));
    (token_name, principal_data, body)
}

/// cost_function: FtTransfer
/// input_size: 0
fn gen_ft_transfer(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let max_transfer = 100;
    let (token_name, sender_principal, template) =
        helper_create_ft_boilerplate(scale * max_transfer);

    let recipient_principal = helper_create_principal();
    for _ in 0..scale {
        let transfer_amount = rng.gen_range(1..=max_transfer);
        let args = format!(
            "{} u{} {} {}",
            token_name, transfer_amount, sender_principal, recipient_principal
        );
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    GenOutput::new(Some(template), body, 1)
}

/// cost_function: FtBalance
/// input_size: 0
fn gen_ft_balance(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let (token_name, principal_data, template) = helper_create_ft_boilerplate(100);
    let args = format!("{} {}", token_name, principal_data);
    for _ in 0..scale {
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    GenOutput::new(Some(template), body, 1)
}

/// cost_function: FtSupply
/// input_size: 0
fn gen_ft_supply(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let (token_name, _, template) = helper_create_ft_boilerplate(100);
    let args = format!("{}", token_name);
    for _ in 0..scale {
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    GenOutput::new(Some(template), body, 1)
}

/// cost_function: FtBurn
/// input_size: 0
fn gen_ft_burn(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let max_burn = 100;
    let (token_name, principal_data, template) = helper_create_ft_boilerplate(scale * max_burn);
    for _ in 0..scale {
        let burn_amount = rng.gen_range(1..=max_burn);
        let args = format!("{} u{} {}", token_name, burn_amount, principal_data);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    GenOutput::new(Some(template), body, 1)
}

fn helper_gen_clarity_list_with_len(len: u64) -> String {
    let mut rng = rand::thread_rng();
    let mut values = "".to_string();
    for _ in 0..len {
        let num: u128 = rng.gen();
        values.push_str(format!("u{} ", num).as_str());
    }

    format!("(list {})", values)
}

// size of argument is in bytes
fn helper_gen_clarity_list_size(approx_size: u64) -> String {
    let uint_size = 17;
    let list_bytes = 5;
    let len: u64 =  ((approx_size - list_bytes) / uint_size).max(1);

    helper_gen_clarity_list_with_len(len)
}

// generate list type of approximate size
pub fn helper_gen_clarity_list_type(approx_size: u64) -> (String, u64) {
    let uint_size = 17;
    let list_bytes = 5;
    let len: u64 =  (approx_size - list_bytes) / uint_size;
    (format!("(list {} uint)", len), len)
}

fn helper_gen_clarity_type(
    allow_bool_type: bool,
    only_sequence_types: bool,
    only_non_seqence_types: bool,
) -> (String, Option<u16>) {
    let mut rng = rand::thread_rng();
    let type_no_len = ["int", "uint", "bool"];
    let type_with_len = ["buff", "string-ascii", "string-utf8"];

    let p = if only_sequence_types {
        0.0
    } else if only_non_seqence_types {
        1.0
    } else {
        0.5
    };
    let (nft_type, nft_len) = match rng.gen_bool(p) {
        true => {
            // a type with no length arg
            let max_range = type_no_len.len() - (if allow_bool_type { 0 } else { 1 });
            let index = rng.gen_range(0..max_range);
            let nft_type = type_no_len[index];
            (nft_type, None)
        }
        false => {
            // a type with a length arg
            let index = rng.gen_range(0..type_with_len.len());
            let mut length = rng.gen_range(2..=50);
            length = if length % 2 == 0 { length } else { length + 1 };
            let nft_type = type_with_len[index];
            (nft_type, Some(length))
        }
    };
    (nft_type.to_string(), nft_len)
}

/// ////////////////////////////////////////
/// NON FUNGIBLE TOKEN GENERATOR FUNCTIONS
/// ////////////////////////////////////////

// Returns statement (that creates nft in clarity) and token_name
fn helper_define_non_fungible_token_statement(
    input_size: u64,
) -> (String, String) {
    let mut rng = rand::thread_rng();
    let token_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let nft_type = helper_make_clarity_type_for_sized_type_sig(input_size);

    let statement = format!("(define-non-fungible-token {} {}) ", token_name, nft_type);
    (statement, token_name)
}

fn helper_gen_clarity_value(
    value_type: &str,
    num: u16,
    value_len: u64,
    list_type: Option<&str>,
) -> (String, u64) {
    let mut rng = rand::thread_rng();
    match value_type {
        "int" => (format!("{}", num), 17),
        "uint" => (format!("u{}", num), 17),
        "buff" => {
            let mut buff = "0x".to_string();
            buff.push_str(&helper_generate_rand_hex_string(value_len as usize));
            if value_len == 1 {
                // buffers of 1 byte have 2 chars
                buff.push_str(&helper_generate_rand_hex_string(value_len as usize));
            }
            (buff.clone(), size_of_value(buff))
        }
        "string-ascii" => {
            let ascii_string = helper_generate_rand_hex_string(value_len as usize);
            let val = format!(r##""{}""##, ascii_string);
            (val.clone(), size_of_value(val))
        }
        "string-utf8" => {
            let utf8_string = helper_generate_rand_hex_string(value_len as usize);
            let val = format!(r##"u"{}""##, utf8_string);
            (val.clone(), size_of_value(val))
        }
        "bool" => {
            let rand_bool = rng.gen_bool(0.5);
            let val = format!("{}", rand_bool);
            (val.clone(), size_of_value(val))
        }
        "list" => {
            let list_type = list_type.unwrap();
            let args = (0..value_len)
                .map(|_| helper_gen_clarity_value(&list_type, num, 0, None).0)
                .collect::<Vec<String>>()
                .join(" ");

            let val = format!("(list {})", args);
            (val.clone(), size_of_value(val))
        }
        _ => {
            unreachable!("should only be generating the types int, uint, buff, string-ascii, string-utf8, bool.")
        }
    }
}

fn helper_gen_random_clarity_value() -> (String, u64) {
    let mut rng = rand::thread_rng();
    let num: u16 = rng.gen();
    let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
    helper_gen_clarity_value(
        &clarity_type,
        num,
        length.map_or(0, |l| l.into()),
        None,
    )
}

/// cost_function: NftMint
/// input_size: size of type signature of asset
///     `expected_asset_type.size()`
fn gen_nft_mint(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    let (statement, token_name) =
        helper_define_non_fungible_token_statement(input_size);

    let nft_type = make_sized_type_sig(input_size);
    let nft_value_size = nft_type.size();
    assert!(nft_value_size <= u16::MAX as u32);

    for i in 0..scale {
        let principal_data = helper_create_principal();
        let nft_value = helper_make_value_for_sized_type_sig(input_size);
        assert_eq!(nft_value_size, nft_value.size());

        let statement = format!(
            "(nft-mint? {} {} {}) ",
            token_name, nft_value, principal_data
        );
        body.push_str(&statement);
    }
    



    GenOutput::new(Some(statement), body, nft_value_size as u64)
}

fn helper_create_nft_fn_boilerplate(input_size: u64) -> (String, String, String, String, u64) {
    let mut body = String::new();
    let (statement, token_name) =
        helper_define_non_fungible_token_statement(input_size);
    body.push_str(&statement);

    let nft_type = make_sized_type_sig(input_size);
    let nft_type_size = nft_type.size();
    assert!(nft_type_size <= u16::MAX as u32);

    let nft_value = helper_make_value_for_sized_type_sig(input_size);
    assert_eq!(nft_type_size, nft_value.size());
    let mut owner_principal = helper_create_principal();
    let mint_statement = format!(
        "(nft-mint? {} {} {}) ",
        token_name, nft_value, owner_principal
    );
    body.push_str(&mint_statement);
    (
        body,
        token_name,
        owner_principal,
        nft_value.to_string(),
        nft_type_size as u64
    )
}

/// cost_function: NftTransfer
/// input_size: size of type signature of asset
///     `expected_asset_type.size()`
fn gen_nft_transfer(function_name: &'static str, scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    let (mut setup, token_name, mut owner_principal, nft_value, nft_type_size) =
        helper_create_nft_fn_boilerplate(input_size);
    for _ in 0..scale {
        let next_principal = helper_create_principal();
        let args = format!(
            "{} {} {} {}",
            token_name, nft_value, owner_principal, next_principal
        );
        body.push_str(&*format!("({} {}) ", function_name, args));

        owner_principal = next_principal;
    }

    
    GenOutput::new(Some(setup), body, nft_type_size)
}

/// cost_function: NftOwner
/// input_size: size of type signature of asset
///     `expected_asset_type.size()`
fn gen_nft_owner(function_name: &'static str, scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let (mut setup, token_name, _, nft_value, nft_type_size) =
        helper_create_nft_fn_boilerplate(input_size);
    let invalid_nft_value = helper_make_value_for_sized_type_sig(input_size);
    assert!(invalid_nft_value.size() <= u16::MAX as u32);
    assert_eq!(nft_type_size, invalid_nft_value.size() as u64);
    let invalid_nft_as_str = invalid_nft_value.to_string();
    for _ in 0..scale {
        let curr_nft_value = match rng.gen_bool(0.5) {
            true => {
                // use valid nft value
                &nft_value
            }
            false => {
                // use invalid nft value
                &invalid_nft_as_str
            }
        };
        let args = format!("{} {}", token_name, curr_nft_value);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    
    GenOutput::new(Some(setup), body, nft_type_size)
}

/// cost_function: NftBurn
/// input_size: size of type signature of asset
///     `expected_asset_type.size()`
fn gen_nft_burn(function_name: &'static str, scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    let (mut setup, token_name, mut owner_principal, nft_value, nft_type_size) =
        helper_create_nft_fn_boilerplate(input_size);
    for _ in 0..scale {
        let args = format!("{} {} {}", token_name, nft_value, owner_principal);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    
    GenOutput::new(Some(setup), body, nft_type_size)
}

/// ////////////////////////////////////////
/// TUPLE GENERATOR FUNCTIONS
/// ////////////////////////////////////////

fn helper_generate_tuple(input_size: u64) -> String {
    let mut rng = rand::thread_rng();
    let tuple_vals = (0..input_size)
        .map(|i| format!("(id{} {})", i, rng.gen::<u32>()))
        .collect::<Vec<String>>()
        .join(" ");

    format!("(tuple {}) ", tuple_vals)
}

/// cost_function: TupleGet
/// input_size: length of tuple data == number of items
fn gen_tuple_get(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    let tuple = helper_generate_tuple(input_size);

    for _ in 0..scale {
        body.push_str(&*format!(
            "(get id{} test-tuple) ",
            rng.gen_range(0..input_size)
        ));
    }

    GenOutput::new(
        None,
        format!("(let ((test-tuple {})) {})", tuple, body),
        input_size,
    )
}

/// cost_function: TupleMerge
/// input_size: sum of serialized size of args
fn gen_tuple_merge(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();

    let tuple_a_vals = (0..input_size)
        .map(|i| format!("(a{} 1337)", i))
        .collect::<Vec<String>>()
        .join(" ");

    let tuple_b_vals = (0..input_size)
        .map(|i| format!("(b{} 1337)", i))
        .collect::<Vec<String>>()
        .join(" ");

    let tuple_a = format!("(tuple {})", tuple_a_vals);
    let tuple_b = format!("(tuple {})", tuple_b_vals);

    for _ in 0..scale {
        body.push_str(&*format!("(merge tuple-a tuple-b) "));
    }

    GenOutput::new(
        None,
        format!(
            "(let ((tuple-a {}) (tuple-b {})) {})",
            tuple_a, tuple_b, body
        ),
        serialized_size(tuple_a) + serialized_size(tuple_b),
    )
}

/// cost_function: TupleCons
/// input_size: number of bindings in the tuple statement
fn gen_tuple_cons(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();

    let tuple_vals = (0..input_size)
        .map(|i| format!("(id{} 1337)", i))
        .collect::<Vec<String>>()
        .join(" ");

    let tuple = format!("(tuple {}) ", tuple_vals);

    for _ in 0..scale {
        body.push_str(&tuple);
    }

    GenOutput::new(None, body, input_size)
}

/// ////////////////////////////////////////
/// OPTIONAL/ RESPONSE GENERATOR FUNCTIONS
/// ////////////////////////////////////////

fn helper_gen_random_optional_value(num: u16, only_some: bool) -> String {
    let mut rng = rand::thread_rng();
    let p = if only_some { 0.0 } else { 0.5 };
    match rng.gen_bool(p) {
        true => "none".to_string(),
        false => {
            let clarity_val = helper_gen_random_clarity_value();
            format!("(some {})", clarity_val.0)
        }
    }
}

/// cost_function: IsSome, IsNone
/// input_size: single arg function
fn gen_optional(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    for i in 0..scale {
        let args = helper_gen_random_optional_value(i, false);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    GenOutput::new(None, body, 1)
}

fn helper_gen_random_response_value(only_ok: bool, only_err: bool) -> String {
    let mut rng = rand::thread_rng();
    let clarity_val = helper_gen_random_clarity_value();
    let p = if only_ok {
        0.0
    } else if only_err {
        1.0
    } else {
        0.5
    };
    match rng.gen_bool(p) {
        true => {
            format!("(err {})", clarity_val.0)
        }
        false => {
            format!("(ok {})", clarity_val.0)
        }
    }
}

/// cost_function: IsOkay, IsErr
/// input_size: single arg function
fn gen_response(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    for i in 0..scale {
        let args = helper_gen_random_response_value(false, false);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: Unwrap, UnwrapRet, TryRet
/// input_size:
///    if ret_value == true: double arg function
///    else: single arg function
fn gen_unwrap(
    function_name: &'static str,
    scale: u16,
    ret_value: bool,
) -> GenOutput {
    let mut rng = rand::thread_rng();
    let mut body = String::new();
    for i in 0..scale {
        let mut args = [
            helper_gen_random_response_value(true, false),
            helper_gen_random_optional_value(i, true),
        ]
        .choose(&mut rng)
        .unwrap()
        .clone();

        if ret_value {
            let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
            let clarity_val = helper_gen_clarity_value(
                &clarity_type,
                i,
                length.map_or(0, |len| len as u64),
                None,
            );
            args = format!("{} {}", args, clarity_val.0)
        }
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    

    GenOutput::new(None, body, 1)
}

/// cost_function: UnwrapErr, UnwrapErrOrRet
/// input_size:
///    if ret_value == true: double arg function
///    else: single arg function
fn gen_unwrap_err(
    function_name: &'static str,
    scale: u16,
    ret_value: bool,
) -> GenOutput {
    let mut body = String::new();
    for i in 0..scale {
        let mut args = helper_gen_random_response_value(false, true);

        if ret_value {
            let clarity_val = helper_gen_random_clarity_value();
            args = format!("{} {}", args, clarity_val.0)
        }
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    

    GenOutput::new(None, body, 1)
}

#[derive(Debug)]
struct DefineMap {
    body: String,
    map_name: String,
    key_name: String,
    key_type: (String, Option<u16>),
    value_name: String,
    value_type: (String, u64),
}

// generate a define map statement
// size = approximate size in bytes of key + value
fn helper_create_map(size: u64) -> DefineMap {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    // random map name
    let map_name = helper_generate_rand_char_string(rng.gen_range(10..20));

    // create key name + type
    let key_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let (key_type, key_type_len) = helper_gen_clarity_type(false, false, false);
    let key_type_formatted = match key_type_len {
        Some(length) => format!("{{ {}: ({} {}) }}", key_name, key_type, length),
        None => format!("{{ {}: {} }}", key_name, key_type),
    };

    // create value name + type
    let value_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let value_type = helper_gen_clarity_list_type(size);
    let value_type_formatted = format!("{{ {}: {} }}", value_name, value_type.0);

    body.push_str(&*format!(
        "(define-map {} {} {}) ",
        map_name, key_type_formatted, value_type_formatted
    ));

    DefineMap {
        body,
        map_name,
        key_name,
        key_type: (key_type, key_type_len),
        value_name,
        value_type: (value_type.0, value_type.1),
    }
}

// setEntry is the cost for map-delete, map-insert, & map-set
// q: only ever deleting non-existent key; should we change that?
/// cost_function: SetEntry
/// input_size: sum of key type size and value type size
fn gen_set_entry(scale: u16, input_size: u64) -> GenOutput {
    let body = String::new();

    let DefineMap {
        body: mut setup,
        map_name,
        key_name,
        key_type,
        value_name,
        value_type,
    } = helper_create_map(input_size);

    let output = format!(" (define-private (execute (input-value {})) (begin ", value_type.0);
    setup.push_str(&output);

    let curr_key = helper_gen_clarity_value(
        &key_type.0,
        89,
        key_type.1.map_or(0, |len| len as u64),
        None,
    );
    let curr_value = helper_gen_clarity_value(
        "list",
        0,
        value_type.1,
        Some("uint"),
    );

    for i in 0..scale {
        let statement = match i % 3 {
            0 => {
                format!(
                    "(map-insert {} {{ {}: {} }} {{ {}: {} }}) ",
                    map_name, key_name, curr_key.0, value_name, curr_value.0
                )
            }
            1 => {
                format!(
                    "(map-set {} {{ {}: {} }} {{ {}: {} }}) ",
                    map_name, key_name, curr_key.0, value_name, curr_value.0
                )
            }
            2 => {
                format!(
                    "(map-delete {} {{ {}: {} }}) ",
                    map_name, key_name, curr_key.0
                )
            }
            _ => unreachable!("should only gen numbers from 0 to 2 inclusive"),
        };
        setup.push_str(&statement);
    }

    setup.push_str("))");

    GenOutput::new(Some(setup), body, curr_key.1 + curr_value.1)
}

/// cost_function: FetchEntry
/// input_size: sum of key type size and value type size
fn gen_fetch_entry(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();

    // define an arbitrary map
    let DefineMap {
        body: mut setup,
        map_name,
        key_name,
        key_type,
        value_name,
        value_type,
    } = helper_create_map(input_size);

    // construct a properly typed key for the map
    let curr_key = helper_gen_clarity_value(
        &key_type.0,
        23,
        key_type.1.map_or(0, |len| len as u64),
        None,
    );

    // construct a properly typed value for the map
    let curr_value = helper_gen_clarity_value(
        "list",
        89,
        value_type.1,
        Some("uint"),
    );

    // insert the key value pair into the map
    setup.push_str(&format!(
        "(map-insert {} {{ {}: {} }} {{ {}: {} }}) ",
        map_name, key_name, curr_key.0, value_name, curr_value.0
    ));

    // construct map-get statements
    for i in 0..scale {
        let curr_key_value = if i % 2 == 0 {
            helper_gen_clarity_value(
                &key_type.0,
                i,
                key_type.1.map_or(0, |len| len as u64),
                None,
            )
        } else {
            curr_key.clone()
        };

        let statement = format!(
            "(map-get? {} {{ {}: {} }}) ",
            map_name, key_name, curr_key_value.0
        );
        body.push_str(&statement);
    }

    GenOutput::new(
        Some(setup),
        body,
        curr_key.1 + curr_value.1,
    )
}

/// helper that wrapes a Clarity code string in a private function called execute.
/// expects code_to_wrap to reference `input-value` variable.
/// invokes code_to_wrap `scale` times.
fn helper_gen_execute_fn(scale: u16, code_to_wrap: String, clarity_type: String) -> String {
    let mut output = format!("(define-private (execute (input-value {})) (begin ", clarity_type);

    for _ in 0..scale {
        output.push_str(&code_to_wrap);
    }

    output.push_str("))");

    output
}


/// cost_function: SetVar
/// input_size: dynamic size of data being persisted
/// generates setup code for var-set benchmarking, with the
/// function calls inside a function body. this allows the
/// benchmarking function generate the the input value as
/// code instead of parsing a large Clarity string (takes too long)
fn gen_var_set(scale: u16, input_size: u64) -> GenOutput {
    let body = String::new();
    let mut rng = rand::thread_rng();

    let var_name = helper_generate_rand_char_string(rng.gen_range(10..20));

    let (clarity_type, length) = helper_gen_clarity_list_type(input_size);

    let clarity_value = helper_gen_clarity_value(
        "list",
        0,
        length,
        Some("uint"),
    );

    let mut setup = format!("(define-data-var {} {} {}) ", var_name, clarity_type, clarity_value.0);

    let var_set = format!("(var-set {} input-value) ", var_name);
    setup.push_str(&helper_gen_execute_fn(scale, var_set, clarity_type));

    GenOutput::new(Some(setup), body, clarity_value.1)
}

/// cost_function: SetVar
/// input_size: dynamic size of data being persisted
fn gen_var_get(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    let var_name = helper_generate_rand_char_string(rng.gen_range(10..20));

    let (clarity_type, length) = helper_gen_clarity_list_type(input_size);

    let clarity_value = helper_gen_clarity_value(
        "list",
        0,
        length,
        Some("uint"),
    );

    let setup = format!("(define-data-var {} {} {})", var_name, clarity_type, clarity_value.0);

    for _ in 0..scale {
        let args = format!("{}", var_name);
        body.push_str(&*format!("(var-get {}) ", args));
    }

    GenOutput::new(Some(setup), body, clarity_value.1)
}

/// cost_function: Print
/// input_size: dynamic size of data being printed
///    `TypeSignature::type_of(self).size()`
fn gen_print(scale: u16, input_size: u64) -> GenOutput {
    let body = String::new();
    let mut setup = String::new();

    let (clarity_type, length) = helper_gen_clarity_list_type(input_size);

    let clarity_value = helper_gen_clarity_value(
        "list",
        0,
        length,
        Some("uint"),
    );
    let size = string_to_value(clarity_value.0).size();

    let print = format!("(print input-value) ");
    setup.push_str(&helper_gen_execute_fn(scale, print, clarity_type));

    dbg!(&setup);

    GenOutput::new(Some(setup), body, size as u64)
}

/// cost_function:
/// input_size:
/// print: size of given Value for print
/// SomeCons/OkCons/ErrCons/ToConsensusBuff: single arg function
/// begin: multi arg function
fn gen_single_clar_value(function_name: &'static str, scale: u16, input_size: Option<u64>) -> GenOutput {
    let mut body = String::new();

    let l = helper_gen_clarity_list_size(input_size.unwrap_or(20));
    let l_size = serialized_size(l.clone());

    for _ in 0..scale {
        let arg = match input_size {
            Some(_) => l.clone(),
            None => helper_gen_random_clarity_value().0,
        };
        body.push_str(&*format!("({} {}) ", function_name, arg));
    }

    GenOutput::new(None, body, l_size)
}

/// cost_function: If
/// input_size: 0
fn gen_if(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for i in 0..scale {
        let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
        let if_case_value =
            helper_gen_clarity_value(&clarity_type, i, length.map_or(0, |len| len as u64), None);
        let else_case_value =
            helper_gen_clarity_value(&clarity_type, i, length.map_or(0, |len| len as u64), None);
        let curr_bool = rng.gen_bool(0.5);

        body.push_str(&*format!(
            "({} {} {} {}) ",
            function_name, curr_bool, if_case_value.0, else_case_value.0
        ));
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: Asserts
/// input_size: 0
fn gen_asserts(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    for i in 0..scale {
        let clarity_val = helper_gen_random_clarity_value();
        body.push_str(&*format!("({} true {}) ", function_name, clarity_val.0));
    }
    

    GenOutput::new(None, body, 1)
}

fn helper_generate_sequences(list_type: &str, output: u16) -> Vec<String> {
    let mut rng = rand::thread_rng();
    match rng.gen_bool(0.75) {
        true => {
            // non-list case
            let (clarity_type, _) = helper_gen_clarity_type(true, true, false);
            (0..output)
                .map(|_| {
                    helper_gen_clarity_value(
                        &clarity_type,
                        rng.gen_range(2..50),
                        rng.gen_range(2..50) * 2,
                        None,
                    ).0
                })
                .collect()
        }
        false => {
            // list case
            (0..output)
                .map(|_| {
                    helper_gen_clarity_value(
                        "list",
                        rng.gen_range(2..50),
                        rng.gen_range(2..50) * 2,
                        Some(list_type),
                    ).0
                })
                .collect()
        }
    }
}

/// cost_function: Concat
/// input_size: sum of Value size of input sequences
///     len() of Value::Sequence
fn gen_concat(function_name: &'static str, scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();

    for _ in 0..scale {
        let first_val = helper_gen_clarity_list_with_len(input_size);
        let second_val = helper_gen_clarity_list_with_len(input_size);
        body.push_str(&*format!(
            "({} (list {}) (list {})) ",
            function_name, first_val, second_val
        ));
    }
    let val = string_to_value(helper_gen_clarity_list_with_len(input_size));
    let total_len = if let Value::Sequence(data) = val {
        data.len()*2
    } else {
        panic!("value should be a sequence.")
    };

    GenOutput::new(None, body, total_len as u64)
}

/// cost_function: AsMaxLen
/// input_size: 0
fn gen_as_max_len(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let (list_type, _) = helper_gen_clarity_type(true, false, true);
        let operand = helper_generate_sequences(&list_type, 1);
        let len = helper_gen_clarity_value("uint", rng.gen_range(2..50), 0, None);
        body.push_str(&*format!("({} {} {}) ", function_name, operand[0], len.0));
    }
    

    GenOutput::new(None, body, 1)
}

/// cost_function: BindName
/// input_size: 0
fn gen_define_constant(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for i in 0..scale {
        let name = helper_generate_rand_char_string(rng.gen_range(10..50));
        let value = helper_gen_random_clarity_value();
        body.push_str(&*format!("({} {} {}) ", function_name, name, value.0));
    }
    

    GenOutput::new(None, body, 1)
}

/// cost_function: DefaultTo
/// input_size: double arg function
fn gen_default_to(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for i in 0..scale {
        let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
        let default_val =
            helper_gen_clarity_value(&clarity_type, i, length.map_or(0, |len| len as u64), None);
        let opt_string = match rng.gen_bool(0.5) {
            true => "none".to_string(),
            false => {
                let inner_val = helper_gen_clarity_value(
                    &clarity_type,
                    i,
                    length.map_or(0, |len| len as u64),
                    None,
                );
                format!("(some {})", inner_val.0)
            }
        };
        body.push_str(&*format!(
            "({} {} {}) ",
            function_name, default_val.0, opt_string
        ));
    }
    

    GenOutput::new(None, body, 1)
}

/// cost_function: IntCast
/// input_size: single arg function
fn gen_int_cast(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let statement = match rng.gen_bool(0.5) {
            true => {
                // to-uint
                let curr_int = format!("{}", rng.gen_range(0..100));
                format!("(to-uint {}) ", curr_int)
            }
            false => {
                // to-int
                let curr_uint = format!("u{}", rng.gen_range(0..100));
                format!("(to-int {}) ", curr_uint)
            }
        };
        body.push_str(&statement);
    }
    

    GenOutput::new(None, body, 1)
}

/// cost_function: Match
/// input_size: 0
fn gen_match(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for i in 0..scale {
        let first_branch_name = helper_generate_rand_char_string(rng.gen_range(10..20));

        let statement = match rng.gen_bool(0.5) {
            true => {
                let match_val = helper_gen_random_response_value(false, false);
                let second_branch_name = helper_generate_rand_char_string(rng.gen_range(10..20));
                format!(
                    "(match {} {} (no-op) {} (no-op)) ",
                    match_val, first_branch_name, second_branch_name
                )
            }
            false => {
                let match_val = helper_gen_random_optional_value(i, false);
                format!(
                    "(match {} {} (no-op) (no-op)) ",
                    match_val, first_branch_name
                )
            }
        };
        body.push_str(&statement);
    }
    

    GenOutput::new(None, body, 1)
}

/// cost_function: Let
/// input_size: number of bindings in the let statement
///     `bindings.len()`
fn gen_let(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for i in 0..scale {
        let mut bindings = String::new();
        for _ in 0..input_size {
            let var_name = helper_generate_rand_char_string(rng.gen_range(10..20));
            let var_value = helper_gen_random_clarity_value();
            bindings.push_str(&*format!("({} {}) ", var_name, var_value.0));
        }
        let statement = format!("(let ({}) (no-op)) ", bindings);
        body.push_str(&statement);
    }
    

    GenOutput::new(None, body, input_size)
}

fn helper_generate_random_sequence() -> (String, usize, String) {
    let mut rng = rand::thread_rng();
    let value_len = rng.gen_range(2..50) * 2;
    let (value_str, type_str) = helper_generate_random_sequence_fixed_len(value_len);
    (value_str, value_len as usize, type_str)
}

fn helper_generate_random_sequence_fixed_len(value_len: u64) -> (String, String) {
    let mut rng = rand::thread_rng();
    match rng.gen_bool(0.75) {
        true => {
            // non-list case
            let (clarity_type, _) = helper_gen_clarity_type(true, true, false);
            let value =
                helper_gen_clarity_value(&clarity_type, rng.gen_range(2..50), value_len, None);
            (value.0, clarity_type)
        }
        false => {
            // list case
            let (list_type, _) = helper_gen_clarity_type(true, false, true);
            let value =
                helper_gen_clarity_value("list", rng.gen_range(2..50), value_len, Some(&list_type));
            (value.0, list_type)
        }
    }
}

fn helper_generate_random_sequence_fixed_len_fixed_type(value_len: u64, type_str: &str) -> String {
    if type_str != "list" { 
        // non-list case
        let (clarity_value, _) = helper_gen_clarity_value(type_str, 1, value_len, None);
        clarity_value
    }
    else {
        // list case
        let (clarity_value, _) = helper_gen_clarity_value(type_str, 1, value_len, Some(type_str));
        clarity_value
    }
}

/// cost_function: IndexOf
/// input_size: the sum of the len of the serialized versions of the args
fn gen_index_of(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    let seq = helper_gen_clarity_value("list", 17, input_size, Some("uint"));
    let item_val = helper_gen_clarity_value("uint", rng.gen_range(2..50), 0, None);

    for _ in 0..scale {
        let statement = format!("(index-of {} {}) ", seq.0, item_val.0);
        body.push_str(&statement);
    }

    let size = string_to_value(seq.0).serialized_size() + string_to_value(item_val.0).serialized_size();

    GenOutput::new(None, body, size as u64)
}

/// cost_function: ElementAt
/// input_size: double arg function
fn gen_element_at(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let (seq, seq_len, _) = helper_generate_random_sequence();
        let index_to_query = rng.gen_range(0..seq_len * 2);
        let statement = format!("(element-at {} u{}) ", seq, index_to_query);
        body.push_str(&statement);
    }
    

    GenOutput::new(None, body, 1)
}

/// cost_function: Len
/// input_size: single arg function
fn gen_len(scale: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        let (seq, _, _) = helper_generate_random_sequence();
        let statement = format!("(len {}) ", seq);
        body.push_str(&statement);
    }
    

    GenOutput::new(None, body, 1)
}

/// cost_function: Append
/// input_size: max of value size (which is to be appended) and size of the type of the list
///     `u64::from(cmp::max(entry_type.size(), element_type.size()))`
fn gen_append(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    let value_size = make_sized_type_sig(input_size).size();
    assert!(value_size < u16::MAX as u32);
    for _ in 0..scale {
        let first_val = helper_make_clarity_value_for_sized_type_sig(input_size);
        let second_val = helper_make_clarity_value_for_sized_type_sig(input_size);

        let statement = format!("(append (list {}) {}) ", first_val, second_val);
        body.push_str(&statement);
    }
    

    GenOutput::new(None, body, value_size as u64)
}

/// cost_function: ListCons
/// input_size: sum of Value sizes of args to be added
///     ```for a in args.iter() {
///         arg_size = arg_size.cost_overflow_add(a.size().into())?;
///     }```
fn gen_list_cons(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        let item_val = "true";
        let mut args = String::new();
        for _ in 0..input_size {
            args.push_str(&*format!("{} ", item_val));
        }
        let statement = format!("(list {}) ", args);
        body.push_str(&statement);
    }
    

    GenOutput::new(None, body, input_size)
}


/// cost_function: Filter
/// input_size: 0
fn gen_filter(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let (list_type, _) = helper_gen_clarity_type(true, false, true);
        let list_val = helper_gen_clarity_value(
            "list",
            rng.gen_range(2..50),
            rng.gen_range(1..5) * 2,
            Some(&list_type),
        );
        let statement = format!("(filter no-op {}) ", list_val.0);
        body.push_str(&statement);
    }
    

    GenOutput::new(None, body, 1)
}

// fixed type of B to be bool
/// cost_function: Fold
/// input_size: 0
fn gen_fold(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let (list_type, _) = helper_gen_clarity_type(true, false, true);
        let list_val = helper_gen_clarity_value(
            "list",
            rng.gen_range(2..50),
            rng.gen_range(1..5) * 2,
            Some(&list_type),
        );
        let statement = format!("(fold no-op {} true) ", list_val.0);
        body.push_str(&statement);
    }
    

    GenOutput::new(None, body, 1)
}

/// cost_function: Map
/// input_size: number of arguments
fn gen_map(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let mut lists = String::new();
        for _ in 0..input_size {
            let (list_type, _) = helper_gen_clarity_type(true, false, true);
            let list_val = helper_gen_clarity_value(
                "list",
                rng.gen_range(2..50),
                rng.gen_range(2..50) * 2,
                Some(&list_type),
            );
            lists.push_str(&list_val.0);
            lists.push_str(" ");
        }

        let statement = format!("(map no-op {}) ", lists);
        body.push_str(&statement);
    }


    GenOutput::new(None, body, input_size)
}

/// cost_function: BlockInfo
/// input_size: 0
fn gen_get_block_info(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    let props = [
        "time",
        "header-hash",
        "burnchain-header-hash",
        "id-header-hash",
        "miner-address",
        "vrf-seed",
    ];

    for i in 0..scale {
        let height = i % 2 + 5;
        body.push_str(format!("(get-block-info? {} u{}) ", props.choose(&mut rng).unwrap(), height).as_str())
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: AtBlock
/// input_size: 0
/// NOTE: Need to provide a index_block_hash from the chainstate DB (index.sqlite)
fn gen_at_block(scale: u16) -> GenOutput {
    let mut body = String::new();

    for _ in 0..scale {
        body.push_str(&*format!("(at-block 0x{} (no-op)) ", READ_TIP));
    }

    GenOutput::new(None, body, 1)
}

// helper function used in bench.rs
pub fn gen_read_only_func(size: u16) -> GenOutput {
    let mut body = String::new();
    let arith_string = gen_arithmetic("+", size, 2).body;
    body.push_str(arith_string.as_str());

    GenOutput::new(
        None,
        format!(
            "(define-read-only (benchmark-load-contract) (begin {}))",
            body
        ),
        1,
    )
}

/// cost_function: AnalysisBindName
/// input_size: type size (could be value, constant, function, total map size, etc.)
///     `v_type.type_size()`
fn gen_type_sig_size(input_size: u64) -> GenOutput {
    let type_sig_map = make_sized_type_sig_map(vec![input_size]);
    let type_sig_size = type_sig_map.get(&input_size).unwrap().type_size().unwrap();
    assert!(type_sig_size < u16::MAX as u32);

    GenOutput::new(None, "".to_string(), type_sig_size as u64)
}

/// cost_function: AnalysisListItemsCheck
/// input_size: type signature size of item
///     `type_arg.type_size()`
fn gen_analysis_list_items_check(_scale: u16, input_size: u64) -> GenOutput {
    let type_size = make_sized_type_sig(input_size).type_size().unwrap();

    GenOutput::new(None, String::new(), type_size as u64)
}

/// cost_function: AnalysisCheckTupleGet
/// input_size: length of tuple
///     `tuple_type_sig.len()`
fn gen_analysis_tuple_get(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        body.push_str(&helper_generate_tuple(input_size));
    }
    

    GenOutput::new(None, body, input_size)
}

fn gen_tuple_size(input_size: u64) -> GenOutput {
    let tuple_map = make_sized_tuple_sigs_map(vec![input_size]);
    let tuple_sig_size = tuple_map.get(&input_size).unwrap().len();
    assert!(tuple_sig_size < u16::MAX as u64);

    GenOutput::new(None, "".to_string(), tuple_sig_size as u64)
}

/// cost_function: AnalysisCheckTupleCons
/// input_size: number of arguments provided
///     `args.len()`
fn gen_analysis_tuple_cons(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    for i in 0..scale {
        body.push_str("(");
        for _ in 0..input_size {
            let var_val = helper_gen_random_clarity_value();
            let var_name = helper_generate_rand_char_string(10);
            body.push_str(&*format!("({} {}) ", var_name, var_val.0));
        }
        body.push_str(") ");
    }
    

    GenOutput::new(None, body, input_size)
}

/// cost_function: AnalysisCheckLet
/// input_size: number of arguments total (the binding list counts as an arg)
///     `args.len()`
fn gen_analysis_check_let(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    for i in 0..(scale) {
        let no_ops = (0..input_size).map(|_x| "(no-op) ").collect::<String>();
        let var_val = helper_gen_random_clarity_value();
        let var_name = helper_generate_rand_char_string(10);
        body.push_str(&*format!("((({} {})) {}) ", var_name, var_val.0, no_ops));
    }
    

    GenOutput::new(None, body, input_size)
}

/// cost_function: AnalysisStorage
/// input_size: size of AST
/// ```for exp in contract_analysis.expressions.iter() {
///        depth_traverse(exp, |_x| match size.cost_overflow_add(1) {
///            Ok(new_size) => {
///                size = new_size;
///                Ok(())
///            }
///            Err(e) => Err(e),
///        })?;
///    }```
fn gen_analysis_storage(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        let mut defines = String::new();
        for j in 0..input_size {
            let (base_type, _) = helper_gen_clarity_type(true, false, true);
            let base_val = helper_gen_clarity_value(&base_type, j as u16, 0, None);
            let constant_name = helper_generate_rand_char_string(10);
            defines.push_str(&*format!(
                "(define-constant {} {}) ",
                constant_name, base_val.0
            ));
        }
        let statement = format!("({}) ", defines);
        body.push_str(&statement);
    }
    

    GenOutput::new(None, body, input_size)
}

/// cost_function: AstCycleDetection, LookupFunction
/// input_size: number of edges in AST / 0
///     `self.graph.edges_count()`
fn gen_ast_cycle_detection(input_size: u64) -> GenOutput {
    let mut body = String::new();
    body.push_str(&*format!("(define-read-only (fn-0) (no-op)) "));
    for i in 1..(input_size + 1) {
        body.push_str(&*format!("(define-read-only (fn-{}) (fn-{})) ", i, i - 1));
    }
    

    let mut cost_tracker = LimitedCostTracker::new_free();

    let mut ast = build_ast_pre(
        &QualifiedContractIdentifier::transient(),
        &body,
        &mut cost_tracker,
        ClarityVersion::latest(),
    ).unwrap();

    let mut definition_sorter = DefinitionSorter::new();
    definition_sorter.run(&mut ast, &mut cost_tracker, ClarityVersion::Clarity2).unwrap();

    let edges = definition_sorter.graph.edges_count().unwrap();

    GenOutput::new(None, body, edges as u64)
}

/// cost_function: AstParse, AnalysisTypeCheck
/// input_size: `source_code.len()` / `return_type.type_size()`
fn gen_empty(input_size: u64) -> GenOutput {
    GenOutput::new(None, "".to_string(), input_size)
}

/// cost_function: ContractStorage
/// input_size: length of contract string
///     `contract_string.len()`
fn gen_contract_storage(input_size: u64) -> GenOutput {
    let contract = make_sized_contract(input_size);
    GenOutput::new(None, contract.0, contract.1)
}

/// cost_function: TypeParseStep
/// input_size: 0
fn gen_type_parse_step(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let type_list = ["bool ", "int ", "uint ", "principal ", "RANDOM "];
    for _ in 0..scale {
        let curr_type = type_list.choose(&mut rng).unwrap();
        body.push_str(curr_type);
    }
    

    GenOutput::new(None, body, 1)
}

/// cost_function: PrincipalOf
/// input_size: 0
fn gen_principal_of(scale: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        body.push_str(&helper_create_principal_in_hex());
    }
    

    GenOutput::new(None, body, 1)
}

/// cost_function: AnalysisTypeLookup
/// input_size: type signature size of value being looked up
///     `expected_asset_type.type_size()`
fn gen_analysis_type_lookup(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        let asset_name = helper_generate_rand_char_string(10);
        let owner = helper_create_principal();
        let tuple = helper_make_clarity_value_for_sized_type_sig(input_size);
        body.push_str(&*format!("({} {} {}) ", asset_name, tuple, owner));
    }
    

    GenOutput::new(None, body, input_size)
}

/// cost_function: AnalysisTypeAnnotate, AnalysisLookupVariableConst
/// input_size: type signature size of SymbolicExpression / 0
///     `type_sig.type_size()` / 0
fn gen_analysis_type_annotate(scale: u16, input_size: u64) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        body.push_str(&helper_generate_rand_char_string(10));
        body.push_str(" ");
    }
    
    let type_sig_map = make_sized_type_sig_map(vec![input_size]);
    let type_sig_size = type_sig_map.get(&input_size).unwrap().type_size().unwrap();
    assert!(type_sig_size < u16::MAX as u32);

    GenOutput::new(None, body, type_sig_size as u64)
}


/// cost_function: AnalysisLookupVariableConst
/// input_size: 0
fn gen_analysis_lookup_variable_const(scale: u16) -> GenOutput {
    let mut body = String::new();
    for i in 0..scale {
        body.push_str(&*format!("var-{}", i));
        body.push_str(" ");
    }
    

    GenOutput::new(None, body, 1)
}


/// cost_function: AnalysisVisit
/// input_size: 0
fn gen_no_op_with_scale_repetitions(scale: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        body.push_str("(no-op) ")
    }
    

    GenOutput::new(None, body, 1)
}

/// cost_function: AnalysisLookupFunctionTypes, AnalysisUseTraitEntry
/// input_size: type signature size of function / sum of type size of function sigs in a trait
///     `func_signature.total_type_size()` / `trait_type_size(&trait_sig)`
fn gen_analysis_lookup_function_types(input_size: u64) -> GenOutput {
    let args = (0..input_size).map(|_x| "uint ").collect::<String>();
    let dummy_fn = format!("(dummy-fn ({}) (response uint uint))", args);
    let body = format!("(define-trait dummy-trait ({})) ", dummy_fn);
    

    // The input size is calculated in `bench.rs`
    GenOutput::new(None, body, 1)
}

/// cost_function: AnalysisGetFunctionEntry, UserFunctionApplication
/// input_size: type size of function signature / number of arguments
///    `func_signature.total_type_size()` / `self.arguments.len()`
fn gen_analysis_get_function_entry(input_size: u64) -> GenOutput {
    let mut body = String::new();

    let args = (0..input_size)
        .map(|i| format!(" (f{} uint) ", i))
        .collect::<String>();
    body.push_str(&*format!("(define-read-only (dummy-fn {}) (no-op)) ", args));

    GenOutput::new(None, body, input_size)
}

/// cost_function: AnalysisFetchContractEntry
/// input_size: size of serialized analysis data, in bytes
pub fn gen_analysis_fetch_contract_entry(input_size: u64) -> GenOutput {
    let dummy_fns = (0..input_size)
        .map(|i| format!("(define-public (dummy-fn-{} (a uint) (b uint)) (ok true))", i))
        .collect();

    GenOutput::new(None, dummy_fns, input_size)
}

/// cost_function: InnerTypeCheckCost
/// input_size: type signature size of argument
///     `arg_type.size()`
fn gen_inner_type_check_cost(input_size: u64) -> GenOutput {
    let mut body = String::new();
    let clar_type = make_clarity_type_for_sized_value(input_size);
    body.push_str(&*format!(
        "(define-read-only (dummy-fn (f0 {})) (no-op)) ",
        clar_type
    ));

    

    GenOutput::new(None, body, 1)
}

/// cost_function: StxTransfer
/// input_size: 0
pub fn gen_stx_transfer(scale: u16) -> GenOutput {
    let mut body = String::new();

    for _ in 0..scale {
        body.push_str("(stx-transfer? u1 tx-sender 'S0G0000000000000000000000000000015XM0F7)");
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: StxBalance
/// input_size: 0
pub fn gen_stx_get_balance(scale: u16) -> GenOutput {
    let mut body = String::new();

    for _ in 0..scale {
        body.push_str("(stx-get-balance 'S1G2081040G2081040G2081040G208105NK8PE5)");
    }

    GenOutput::new(None, body, 1)
}
////////////////////// ANALYSIS PASS COSTS /////////////////////////

pub fn gen_analysis_pass_read_only(input_size: u64) -> GenOutput {
    let mut body = String::new();
    for i in 0..input_size {
        let fn_body = if i == 0 {
            "(let ((a 2) (b (+ 5 6 7))) (+ a b))".to_string()
        } else {
            format!("(let ((a (dummy-fn-{})) (b (+ 5 6 7))) (+ a b))", i - 1)
        };
        let fn_def = format!("(define-read-only (dummy-fn-{}) (begin {})) ", i, fn_body);
        body.push_str(&fn_def);
    }
    

    GenOutput::new(None, body, input_size)
}

pub fn gen_analysis_pass_arithmetic_only(input_size: u64) -> GenOutput {
    let mut body = String::new();
    for i in 0..input_size {
        let fn_body = if i == 0 {
            "(let ((a 2) (b (+ 5 6 7))) (+ a b))".to_string()
        } else {
            format!("(let ((a (dummy-fn-{})) (b (+ 5 6 7))) (+ a b))", i - 1)
        };
        let fn_def = format!(
            "(define-read-only (dummy-fn-{}) (begin (no-op none) {})) ",
            i, fn_body
        );
        body.push_str(&fn_def);
    }
    

    GenOutput::new(None, body, input_size)
}

pub fn define_dummy_trait(i: u64, clarity_type: &str) -> String {
    let dummy_fn = format!("(dummy-fn-{} ({}) (response uint uint))", i, clarity_type);
    format!("(define-trait dummy-trait-{} ({})) ", i, dummy_fn)
}

pub fn gen_analysis_pass_trait_checker(input_size: u64) -> GenOutput {
    let mut setup_body = String::new();
    let mut body = String::new();
    for i in 0..input_size {
        let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
        let final_clarity_type = match length {
            Some(l) => format!("({} {})", clarity_type, l),
            None => clarity_type,
        };

        let curr_trait = define_dummy_trait(i, &final_clarity_type);
        setup_body.push_str(&curr_trait);

        let impl_fn = format!(
            "(define-public (dummy-fn-{} (arg1 {})) (ok u0)) ",
            i, final_clarity_type
        );
        body.push_str(&impl_fn);
    }

    GenOutput::new(Some(setup_body), body, input_size)
}

pub fn gen_analysis_pass_type_checker(input_size: u64) -> GenOutput {
    let mut setup_body = String::new();
    let mut body = String::new();
    for i in 0..input_size {
        let curr_trait = define_dummy_trait(i, "uint");
        setup_body.push_str(&curr_trait);

        let inner_let = "(let ((c 7)) (- c 0))";
        let fn_body = format!("(let ((a {}) (b (+ 5 6 7))) (+ a b))", inner_let);
        let fn_def = format!("(define-read-only (dummy-fn-{}) (begin {})) ", i, fn_body);
        body.push_str(&fn_def);
    }
    

    GenOutput::new(Some(setup_body), body, input_size)
}

// contract-call-bench? does everything contract-call? does, except load and execute the contract code
/// cost_function: ContractCall
/// input_size: 0
fn gen_contract_call(scale: u16) -> GenOutput {
    let mut body = String::new();

    for _ in 0..scale {
        body.push_str(
            "(contract-call-bench? 'SP000000000000000000002Q6VF78.cost-voting get-counter) ",
        );
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: ContractOf
/// input_size: 0
fn gen_contract_of(scale: u16) -> GenOutput {
    let mut body = String::new();

    for _ in 0..scale {
        body.push_str(
            "(contract-call? .use-trait-contract bench-contract-of .impl-trait-contract) ",
        );
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: AsContract
/// input_size: 0
fn gen_as_contract(scale: u16) -> GenOutput {
    let mut body = String::new();

    for _ in 0..scale {
        body.push_str(
            "(as-contract true)"
        );
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: BuffToNumber
/// input_size: 0
fn gen_buff_to_numeric_type(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        let buff = helper_gen_clarity_value("buff", 0, 32, None);
        body.push_str(&*format!("({} {}) ", function_name, buff.0));
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: IsStandard
/// input_size: 0
fn gen_is_standard(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        let principal = helper_create_principal();
        body.push_str(&*format!("({} {}) ", function_name, principal));
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: PrincipalDestruct
/// input_size: 0
fn gen_principal_destruct(function_name: &'static str, scale: u16) -> GenOutput {
    let mut rng = rand::thread_rng();
    let mut body = String::new();
    for _ in 0..scale {
        let principal = match rng.gen_bool(0.5) {
            true => {
                helper_create_principal()
            }
            false => {
                format!("{}.{}", helper_create_principal(), helper_generate_rand_char_string(8))
            }
        };
        body.push_str(&*format!("({} {}) ", function_name, principal));
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: PrincipalConstruct
/// input_size: 0
fn gen_principal_construct(function_name: &'static str, scale: u16) -> GenOutput {
    let mut rng = rand::thread_rng();
    let mut body = String::new();
    for _ in 0..scale {
        let version_byte = helper_gen_clarity_value("buff", 0, 2, None);
        let pub_key_hash = helper_gen_clarity_value("buff", 0, 40, None);
        let args = match rng.gen_bool(0.5) {
            true => {
                format!("{} {}", version_byte.0, pub_key_hash.0)
            }
            false => {
                format!("{} {} \"{}\"", version_byte.0, pub_key_hash.0, helper_generate_rand_char_string(8))
            }
        };
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: StringToInt / StringToUInt
/// input_size: 0
fn gen_string_to_number(function_name: &'static str, scale: u16) -> GenOutput {
    let mut rng = rand::thread_rng();
    let mut body = String::new();
    for _ in 0..scale {
        let rand_str = match rng.gen_bool(0.5) {
            true => {
                helper_generate_rand_numeric_string(8)
            }
            false => {
                helper_generate_rand_char_string(8)
            }
        };
        let formatted_str = match rng.gen_bool(0.5) {
            true => {
                format!("\"{}\"", rand_str)
            }
            false => {
                format!("u\"{}\"", rand_str)
            }
        };
        body.push_str(&*format!("({} {}) ", function_name, formatted_str));
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: IntToAscii / IntToUtf8
/// input_size: 0
fn gen_number_to_string(function_name: &'static str, scale: u16) -> GenOutput {
    let mut rng = rand::thread_rng();
    let mut body = String::new();
    for _ in 0..scale {
        let num = rng.gen_range(0..10000);
        let formatted_num = match rng.gen_bool(0.5) {
            true => {
                format!("{}", num)
            }
            false => {
                format!("u{}", num)
            }
        };
        body.push_str(&*format!("({} {}) ", function_name, formatted_num));
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: StxTransferMemo
/// input_size: 0
pub fn gen_stx_transfer_memo(function_name: &'static str, scale: u16) -> GenOutput {
    let mut rng = rand::thread_rng();
    let mut body = String::new();

    for _ in 0..scale {
        let len = rng.gen_range(1..15)*2;
        let memo = helper_gen_clarity_value("buff", 0, len, None);
        body.push_str(&*format!("({} u1 tx-sender 'S0G0000000000000000000000000000015XM0F7 {}) ", function_name, memo.0));
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: Slice
/// input_size: 0
pub fn gen_slice(function_name: &'static str, scale: u16) -> GenOutput {
    let mut rng = rand::thread_rng();
    let mut body = String::new();

    for _ in 0..scale {
        let (seq, seq_len, _) = helper_generate_random_sequence();
        let (left, right) = match rng.gen_bool(0.8) {
            true => {
                // valid range
                let left_pos = rng.gen_range(0..seq_len-2);
                let right_pos = rng.gen_range(left_pos..seq_len);
                (left_pos, right_pos)
            }
            false => {
                match rng.gen_range(0..=2) {
                    0 => {
                        // right < left
                        let right_pos = rng.gen_range(0..seq_len-3);
                        let left_pos = rng.gen_range(right_pos+1..seq_len);
                        (left_pos, right_pos)
                    }
                    1 => {
                        // right > len
                        let left_pos = rng.gen_range(0..seq_len-2);
                        let right_pos = seq_len + 3;
                        (left_pos, right_pos)
                    }
                    2 => {
                        // left > len
                        let left_pos = seq_len + 3;
                        let right_pos = rng.gen_range(0..seq_len-2);
                        (left_pos, right_pos)
                    }
                    _ => {
                        unreachable!("should only be generating numbers in the range 0..=2.")
                    }
                }
            }
        };

        body.push_str(&*format!("({} {} u{} u{}) ", function_name, seq, left, right));
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: ReplaceAt
/// input_size: the size of the item being inserted
pub fn gen_replace_at(function_name: &'static str, scale: u16) -> GenOutput {
    let mut rng = rand::thread_rng();
    let mut body = String::new();
    let mut sz = 0;

    for _ in 0..scale {
        let (seq, seq_len, seq_type) = helper_generate_random_sequence();
        let (replace_val, replace_val_sz) = helper_gen_clarity_value(&seq_type, rng.gen_range(2..50), 1, None);
        let index = if rng.gen_bool(0.8) {
            // index is in range
            rng.gen_range(0..seq_len)
        }
        else {
            // index is not in range 
            rng.gen_range(seq_len..(seq_len * 2))
        };

        let stmt = format!("(replace-at? {} u{} {})", &seq, index, &replace_val);
        body.push_str(&stmt);

        sz += replace_val_sz;
    }
    GenOutput::new(None, body, sz)
}


/// cost_function: FromConsensusBuff
/// input_size: number of bytes in the input buffer
pub fn gen_from_consensus_buff(function_name: &'static str, scale: u16, input_size: u64) -> GenOutput {
    let mut rng = rand::thread_rng();
    let mut body = String::new();

    let clar_value = make_sized_value(input_size).serialize_to_vec();
    let len = clar_value.len();

    for _ in 0..scale {
        // let clar_value = helper_make_value_for_sized_type_sig(input_size).serialize_to_vec();
        let clar_value = make_sized_value(input_size).serialize_to_vec();
        let clar_type = make_clarity_type_for_sized_value(input_size);
        let clar_buff_serialized = match Value::buff_from(clar_value) {
            Ok(x) => x,
            Err(_) => panic!()
        };
        body.push_str(&*format!("({} {} {}) ", function_name, clar_type, clar_buff_serialized));
    }
    println!("{}", body);

    GenOutput::new(None, body, len as u64)
}

/// cost_function: StxGetAccount
/// input_size: 0
pub fn gen_stx_get_account(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();

    for _ in 0..scale {
        body.push_str(&*format!("({} 'S1G2081040G2081040G2081040G208105NK8PE5) ", function_name));
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: GetBurnBlockInfo
/// input_size: 0
fn gen_get_burn_block_info(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    let props = [
        "header-hash",
        "pox-addrs"
    ];
    for i in 0..scale {
        let height = i % 2 + 5;
        body.push_str(
            &*format!("({} {} u{}) ",
                      function_name,
                      props.choose(&mut rng).unwrap(), height)
        )
    }

    GenOutput::new(None, body, 1)
}

/// Returns tuple of optional setup clarity code, and "main" clarity code
/// The `reviewed` comment above each cost function should list the GitHub usernames of those
///    who have verified that the benchmark for that cost function seems accurate (given the code
///    in `benches.rs`, the code in `generators.rs`, and the benchmark data.
pub fn gen(function: ClarityCostFunction, scale: u16, input_size: u64) -> GenOutput {
    match function {
        // Arithmetic ///////////////////////
        
        ClarityCostFunction::Add => gen_arithmetic("+", scale, input_size),
        ClarityCostFunction::Sub => gen_arithmetic("-", scale, input_size),
        ClarityCostFunction::Mul => gen_arithmetic("*", scale, input_size),
        ClarityCostFunction::Div => gen_arithmetic("/", scale, input_size),
        ClarityCostFunction::Sqrti => gen_arithmetic("sqrti", scale, 1),
        ClarityCostFunction::Log2 => gen_arithmetic("log2", scale, 1),
        ClarityCostFunction::Mod => gen_arithmetic("mod", scale, 2),

        
        ClarityCostFunction::Pow => gen_pow(scale),

        // Logic /////////////////////////////
        
        ClarityCostFunction::Le => gen_cmp("<", scale, input_size),
        ClarityCostFunction::Leq => gen_cmp("<=", scale, input_size),
        ClarityCostFunction::Ge => gen_cmp(">", scale, input_size),
        ClarityCostFunction::Geq => gen_cmp(">=", scale, input_size),


        // Boolean ///////////////////////////
        
        ClarityCostFunction::And => gen_logic("and", scale, input_size),
        ClarityCostFunction::Or => gen_logic("or", scale, input_size),
        ClarityCostFunction::Not => gen_logic("not", scale, input_size),
        ClarityCostFunction::Eq => gen_logic("is-eq", scale, input_size),
        
        ClarityCostFunction::Xor => gen_arithmetic("bit-xor", scale, input_size),


        // Tuples ////////////////////////////
        
        ClarityCostFunction::TupleGet => gen_tuple_get(scale, input_size),

        
        ClarityCostFunction::TupleMerge => gen_tuple_merge(scale, input_size),

        
        ClarityCostFunction::TupleCons => gen_tuple_cons(scale, input_size),


        // Analysis //////////////////////////
        
        ClarityCostFunction::AnalysisTypeAnnotate => gen_analysis_type_annotate(scale, input_size),

        
        ClarityCostFunction::AnalysisTypeCheck => gen_type_sig_size(input_size),

        
        ClarityCostFunction::AnalysisTypeLookup => gen_analysis_type_lookup(scale, input_size),

        
        ClarityCostFunction::AnalysisVisit => gen_no_op_with_scale_repetitions(scale),

        
        // input_size: 0 in most cases, `args.len()` in `check_special_map`
        ClarityCostFunction::AnalysisIterableFunc => unimplemented!(),

        
        // input_size: 0
        ClarityCostFunction::AnalysisOptionCons => gen_empty(input_size),

        
        // input_size: 0
        ClarityCostFunction::AnalysisOptionCheck => gen_empty(input_size),

        
        // TODO: super slow, get second review
        // input_size: type signature size of item
        ClarityCostFunction::AnalysisBindName => gen_type_sig_size(input_size),

        
        // input_size: type signature size of item
        ClarityCostFunction::AnalysisListItemsCheck => gen_type_sig_size(input_size),

        
        ClarityCostFunction::AnalysisCheckTupleGet => gen_analysis_tuple_get(scale, input_size),

        
        // input_size: length of second tuple
        ClarityCostFunction::AnalysisCheckTupleMerge => gen_tuple_size(input_size),

        
        ClarityCostFunction::AnalysisCheckTupleCons => gen_analysis_tuple_cons(scale, input_size),

        
        // input_size: type signature size of value, `var_type.type_size()`
        ClarityCostFunction::AnalysisTupleItemsCheck => gen_type_sig_size(input_size),

        
        // TODO: size is args.len() not binding_list.len()
        ClarityCostFunction::AnalysisCheckLet => gen_analysis_check_let(scale, input_size),

        
        // input_size: 0
        ClarityCostFunction::AnalysisLookupFunction => unimplemented!(),

        
        ClarityCostFunction::AnalysisLookupFunctionTypes => {
            gen_analysis_lookup_function_types(input_size)
        }

        
        ClarityCostFunction::AnalysisLookupVariableConst => {
            gen_analysis_lookup_variable_const(scale)
        }

        
        ClarityCostFunction::AnalysisLookupVariableDepth => unimplemented!(), // no gen function needed

        
        ClarityCostFunction::AnalysisStorage => gen_analysis_storage(scale, input_size),

        
        ClarityCostFunction::AnalysisUseTraitEntry => {
            gen_analysis_lookup_function_types(input_size)
        }

        
        ClarityCostFunction::AnalysisGetFunctionEntry => {
            gen_analysis_get_function_entry(input_size)
        }

        
        ClarityCostFunction::AnalysisFetchContractEntry => {
            gen_analysis_fetch_contract_entry(input_size)
        }


        // Ast ////////////////////////////////
        
        ClarityCostFunction::AstParse => gen_empty(input_size),

        
        ClarityCostFunction::AstCycleDetection => gen_ast_cycle_detection(input_size),

        
        ClarityCostFunction::ContractStorage => gen_contract_storage(input_size),


        // Lookup ////////////////////////////////
        
        ClarityCostFunction::LookupVariableDepth => unimplemented!(), // no gen function needed

        
        ClarityCostFunction::LookupVariableSize => unimplemented!(),  // no gen function needed

        
        ClarityCostFunction::LookupFunction => gen_ast_cycle_detection(input_size),


        // List ////////////////////////////////
        
        ClarityCostFunction::Map => gen_map(scale, input_size), // includes LookupFunction cost

        
        ClarityCostFunction::Filter => gen_filter(scale),       // includes LookupFunction cost

        
        ClarityCostFunction::Fold => gen_fold(scale),           // includes LookupFunction cost

        
        ClarityCostFunction::Len => gen_len(scale),

        ClarityCostFunction::ElementAt => gen_element_at(scale),

        
        ClarityCostFunction::IndexOf => gen_index_of(scale, input_size),

        
        ClarityCostFunction::ListCons => gen_list_cons(scale, input_size),

        
        ClarityCostFunction::Append => gen_append(scale, input_size),


        // Hash ////////////////////////////////
        
        ClarityCostFunction::Hash160 => gen_hash("hash160", scale, input_size),

        
        ClarityCostFunction::Sha256 => gen_hash("sha256", scale, input_size),

        
        ClarityCostFunction::Sha512 => gen_hash("sha512", scale, input_size),

        
        ClarityCostFunction::Sha512t256 => gen_hash("sha512/256", scale, input_size),

        
        ClarityCostFunction::Keccak256 => gen_hash("keccak256", scale, input_size),

        
        ClarityCostFunction::Secp256k1recover => gen_secp256k1("secp256k1-recover?", scale, false),

        
        ClarityCostFunction::Secp256k1verify => gen_secp256k1("secp256k1-verify", scale, true),

        // FT ////////////////////////////////
        
        ClarityCostFunction::CreateFt => gen_create_ft("define-fungible-token", scale),

        
        ClarityCostFunction::FtMint => gen_ft_mint("ft-mint?", scale),

        
        ClarityCostFunction::FtTransfer => gen_ft_transfer("ft-transfer?", scale),

        
        ClarityCostFunction::FtBalance => gen_ft_balance("ft-get-balance", scale),

        
        ClarityCostFunction::FtSupply => gen_ft_supply("ft-get-supply", scale),

        
        ClarityCostFunction::FtBurn => gen_ft_burn("ft-burn?", scale),


        // NFT ////////////////////////////////
        
        // cost_function: CreateNft
        // input_size: size of asset type
        //     `asset_type.size()`
        ClarityCostFunction::CreateNft => unimplemented!(),

        
        ClarityCostFunction::NftMint => gen_nft_mint(scale, input_size),

        
        ClarityCostFunction::NftTransfer => gen_nft_transfer("nft-transfer?", scale, input_size),

        
        ClarityCostFunction::NftOwner => gen_nft_owner("nft-get-owner?", scale, input_size),

        
        ClarityCostFunction::NftBurn => gen_nft_burn("nft-burn?", scale, input_size),

        // Stacks ////////////////////////////////
        
        ClarityCostFunction::PoisonMicroblock => unimplemented!(), // don't need a gen for this

        
        ClarityCostFunction::BlockInfo => gen_get_block_info(scale),

        
        ClarityCostFunction::StxBalance => gen_stx_get_balance(scale),

        
        ClarityCostFunction::StxTransfer => gen_stx_transfer(scale),


        // Option & result checks ////////////////////////////////
        
        ClarityCostFunction::IsSome => gen_optional("is-some", scale),

        
        ClarityCostFunction::IsNone => gen_optional("is-none", scale),

        
        ClarityCostFunction::IsOkay => gen_response("is-ok", scale),

        
        ClarityCostFunction::IsErr => gen_response("is-err", scale),

        
        ClarityCostFunction::DefaultTo => gen_default_to("default-to", scale),


        // Unwrap functions ////////////////////////////////
        
        ClarityCostFunction::Unwrap => gen_unwrap("unwrap-panic", scale, false),

        
        ClarityCostFunction::UnwrapRet => gen_unwrap("unwrap!", scale, true),

        
        ClarityCostFunction::UnwrapErr => gen_unwrap_err("unwrap-err-panic", scale, false),

        
        ClarityCostFunction::UnwrapErrOrRet => gen_unwrap_err("unwrap-err!", scale, true),

        
        ClarityCostFunction::TryRet => gen_unwrap("try!", scale, false),


        // Map ////////////////////////////////
        
        // cost_function: CreateMap
        // input_size: sum of key type size and value type size
        //     `u64::from(key_type.size()).cost_overflow_add(u64::from(value_type.size()))`
        ClarityCostFunction::CreateMap => unimplemented!(),

        
        ClarityCostFunction::FetchEntry => gen_fetch_entry(scale, input_size), // map-get?

        
        ClarityCostFunction::SetEntry => gen_set_entry(scale, input_size),     // map-set


        // Var ////////////////////////////////
        
        // cost_function: CreateVar
        // input_size: value type size
        //     `value_type.size()`
        ClarityCostFunction::CreateVar => unimplemented!(),

        
        ClarityCostFunction::FetchVar => gen_var_get(scale, input_size),

        
        ClarityCostFunction::SetVar => gen_var_set(scale, input_size),

        
        ClarityCostFunction::BindName => gen_define_constant("define-constant-bench", scale), // used for define var and define function


        // Functions with single clarity value input ////////////////////////////////
        
        ClarityCostFunction::Print => gen_print(scale, input_size),

        
        ClarityCostFunction::SomeCons => gen_single_clar_value("some", scale, None),

        
        ClarityCostFunction::OkCons => gen_single_clar_value("ok", scale, None),

        
        ClarityCostFunction::ErrCons => gen_single_clar_value("err", scale, None),

        
        ClarityCostFunction::Begin => gen_single_clar_value("begin", scale, None),


        // Type Checking ////////////////////////////////
        
        ClarityCostFunction::InnerTypeCheckCost => gen_inner_type_check_cost(input_size),

        
        ClarityCostFunction::TypeParseStep => gen_type_parse_step(scale), // called by `parse_type_repr` in `signatures.rs` (takes in symbolic expression)


        // Uncategorized ////////////////////////////////
        
        ClarityCostFunction::If => gen_if("if", scale),

        
        ClarityCostFunction::Asserts => gen_asserts("asserts!", scale),

        
        ClarityCostFunction::Concat => gen_concat("concat", scale, input_size),

        
        ClarityCostFunction::IntCast => gen_int_cast(scale),

        
        ClarityCostFunction::Let => gen_let(scale, input_size),

        
        ClarityCostFunction::Match => gen_match(scale),

        
        ClarityCostFunction::AsMaxLen => gen_as_max_len("as-max-len?", scale),

        
        ClarityCostFunction::UserFunctionApplication => gen_analysis_get_function_entry(input_size),

        
        ClarityCostFunction::ContractCall => gen_contract_call(scale),

        
        ClarityCostFunction::ContractOf => gen_contract_of(scale),

        
        ClarityCostFunction::PrincipalOf => gen_principal_of(scale),

        
        ClarityCostFunction::AtBlock => gen_at_block(scale),

        
        ClarityCostFunction::LoadContract => unimplemented!(), // called at start of execute_contract
        ClarityCostFunction::Unimplemented => unimplemented!(),

        // Clarity 2 functions
        ClarityCostFunction::BuffToIntLe => gen_buff_to_numeric_type("buff-to-int-le", scale),
        ClarityCostFunction::BuffToUIntLe => gen_buff_to_numeric_type("buff-to-uint-le", scale),
        ClarityCostFunction::BuffToIntBe => gen_buff_to_numeric_type("buff-to-int-be", scale),
        ClarityCostFunction::BuffToUIntBe => gen_buff_to_numeric_type("buff-to-uint-be", scale),
        ClarityCostFunction::IsStandard => gen_is_standard("is-standard", scale),
        ClarityCostFunction::PrincipalDestruct => gen_principal_destruct("principal-destruct", scale),
        ClarityCostFunction::PrincipalConstruct => gen_principal_construct("principal-construct", scale),
        ClarityCostFunction::StringToInt => gen_string_to_number("string-to-int", scale),
        ClarityCostFunction::StringToUInt => gen_string_to_number("string-to-uint", scale),
        ClarityCostFunction::IntToAscii => gen_number_to_string("int-to-ascii", scale),
        ClarityCostFunction::IntToUtf8 => gen_number_to_string("int-to-utf8", scale),
        ClarityCostFunction::GetBurnBlockInfo => gen_get_burn_block_info("get-burn-block-info?", scale),
        ClarityCostFunction::StxGetAccount => gen_stx_get_account("stx-account", scale),
        ClarityCostFunction::Slice => gen_slice("slice?", scale),
        ClarityCostFunction::ToConsensusBuff => gen_single_clar_value("to-consensus-buff?", scale, Some(input_size)),
        ClarityCostFunction::FromConsensusBuff => gen_from_consensus_buff("from-consensus-buff?", scale, input_size),
        ClarityCostFunction::StxTransferMemo => gen_stx_transfer_memo("stx-transfer-memo?", scale),
        ClarityCostFunction::ReplaceAt => gen_replace_at("replace-at?", scale),
        ClarityCostFunction::AsContract => gen_as_contract(scale),

        // Clarity 2 bitwise functions
        ClarityCostFunction::BitwiseAnd => gen_arithmetic("bit-and", scale, input_size),
        ClarityCostFunction::BitwiseOr => gen_arithmetic("bit-or", scale, input_size),
        ClarityCostFunction::BitwiseNot => gen_arithmetic("bit-not", scale, 1),
        ClarityCostFunction::BitwiseLShift => gen_lshift(scale),
        ClarityCostFunction::BitwiseRShift => gen_rshift(scale),
    }
}


/// Returns tuple of optional setup clarity code, and "main" clarity code
pub fn gen_analysis_pass(
    function: AnalysisCostFunction,
    _scale: u16,
    input_size: u64,
) -> GenOutput {
    match function {
        
        AnalysisCostFunction::ReadOnly => gen_analysis_pass_read_only(input_size),

        
        AnalysisCostFunction::TypeChecker => gen_analysis_pass_type_checker(input_size),

        
        AnalysisCostFunction::TraitChecker => gen_analysis_pass_trait_checker(input_size),

        
        AnalysisCostFunction::ArithmeticOnlyChecker => {
            gen_analysis_pass_arithmetic_only(input_size)
        }
    }
}
