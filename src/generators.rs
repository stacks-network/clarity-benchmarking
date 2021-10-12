use blockstack_lib::burnchains::PrivateKey;
use blockstack_lib::util::secp256k1::{Secp256k1PrivateKey, Secp256k1PublicKey};
use blockstack_lib::vm::costs::cost_functions::{AnalysisCostFunction, ClarityCostFunction};
use rand::distributions::Uniform;
use rand::prelude::SliceRandom;
use rand::{Rng, RngCore};

use blockstack_lib::address::AddressHashMode;
use blockstack_lib::chainstate::stacks::{StacksPublicKey, C32_ADDRESS_VERSION_TESTNET_SINGLESIG};
use blockstack_lib::types::chainstate::StacksAddress;
use blockstack_lib::util::hash::to_hex;
use blockstack_lib::vm::analysis::contract_interface_builder::ContractInterfaceAtomType::{
    list, principal,
};
use blockstack_lib::vm::types::signatures::TypeSignature::{
    BoolType, IntType, PrincipalType, TupleType, UIntType,
};
use blockstack_lib::vm::types::{
    ASCIIData, CharType, ListData, OptionalData, SequenceData, TupleData, TupleTypeSignature,
    TypeSignature,
};
use blockstack_lib::vm::{execute, ClarityName, Value};
use lazy_static::lazy_static;
use rand::rngs::ThreadRng;
use secp256k1::Message as LibSecp256k1Message;
use std::cmp::min;
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::fmt::format;

lazy_static! {
    pub static ref TUPLE_NAMES: Vec<String> = create_tuple_names(16);
}

pub struct GenOutput {
    pub setup: Option<String>,
    pub body: String,
    pub input_size: u16,
}

impl GenOutput {
    pub fn new(setup: Option<String>, body: String, input_size: u16) -> Self {
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
fn make_tuple_pair(pairs: u16) -> Value {
    let mut data = Vec::new();
    for i in 0..pairs {
        let name = TUPLE_NAMES[i as usize].clone();
        let val = Value::Bool(true);
        data.push((ClarityName::try_from(name).unwrap(), val));
    }
    let td = TupleData::from_data(data).unwrap();
    Value::Tuple(td)
}

pub fn make_sized_values_map(input_sizes: Vec<u16>) -> HashMap<u16, Value> {
    let mut ret_map = HashMap::new();
    for i in input_sizes {
        let val = match i {
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
        };
        ret_map.insert(i, val);
    }
    ret_map
}

pub fn make_clarity_type_for_sized_value(input_size: u16) -> String {
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
fn make_clarity_statement_for_sized_contract(mult: u16) -> (String, u16) {
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

    (contract.clone(), contract.len() as u16)
}

fn make_sized_contract(input_size: u16) -> (String, u16) {
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

pub fn make_sized_contracts_map(input_sizes: Vec<u16>) -> HashMap<u16, String> {
    let mut ret_map = HashMap::new();
    for i in input_sizes {
        let val = make_sized_contract(i);
        ret_map.insert(val.1, val.0);
    }
    ret_map
}

// make tuple type sigs for AnalysisCheckTupleGet
fn make_tuple_sig(input_size: u16) -> TupleTypeSignature {
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

pub fn make_sized_tuple_sigs_map(input_sizes: Vec<u16>) -> HashMap<u16, TupleTypeSignature> {
    let mut ret_map = HashMap::new();
    for i in input_sizes {
        let val = make_tuple_sig(i);
        ret_map.insert(i, val);
    }
    ret_map
}

fn helper_make_clarity_type_for_sized_type_sig(input_size: u16) -> String {
    match input_size {
        1 => "uint".to_string(),
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
                key_pairs.push_str(&*format!("({} uint) ", name));
            }
            format!("(tuple {})", key_pairs)
        }
    }
}

fn helper_make_clarity_value_for_sized_type_sig(input_size: u16) -> String {
    let mut rng = rand::thread_rng();
    match input_size {
        1 => format!("{}", rng.gen::<u32>()),
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
                    format!("({}{} {})", id_name, i, rng.gen::<u32>())
                })
                .collect::<Vec<String>>()
                .join(" ");

            format!("(tuple {}) ", tuple_vals)
        }
    }
}

pub fn helper_make_value_for_sized_type_sig(input_size: u16) -> Value {
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
fn make_sized_type_sig(input_size: u16) -> TypeSignature {
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

pub fn make_sized_type_sig_map(input_sizes: Vec<u16>) -> HashMap<u16, TypeSignature> {
    let mut ret_map = HashMap::new();
    for i in input_sizes {
        let val = make_sized_type_sig(i);
        ret_map.insert(i, val);
    }
    ret_map
}

pub fn helper_make_sized_clarity_value(input_size: u16) -> String {
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

/// cost_function: Add, Sub, Mul, Div, Sqrti, Log2, Mod
/// input_size: number of arguments
pub fn gen_arithmetic(
    function_name: &'static str,
    scale: u16,
    input_size: u16,
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
/// input_size: double arg function
fn gen_cmp(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let n1: u128 = rng.gen();
        let n2: u128 = rng.gen();
        body.push_str(&*format!("({} u{} u{}) ", function_name, n1, n2));
    }

    GenOutput::new(None, body, 2)
}

/// cost_function: And, Or, Not, Eq
/// input_size: number of arguments
fn gen_logic(function_name: &'static str, scale: u16, input_size: u16) -> GenOutput {
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

/// This function generates a random char string of size n.
pub fn helper_generate_rand_char_string(n: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..n)
        .map(|_| rng.gen_range(b'a'..b'z') as char)
        .collect::<String>()
}

/// This function generates a single value that either has type uint, int, or buff (randomly chosen)
/// This value is set as the argument to a hash function ultimately
///
/// cost_function: Hash160, Sha256, Sha512, Sha512t256, Keccak256
/// input_size: single arg function
fn gen_hash(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let arg = match rng.gen_range(0..=2) {
            0 => {
                // uint
                let x: u128 = rng.gen();
                format!("u{}", x)
            }
            1 => {
                // int
                let x: i128 = rng.gen();
                format!("{}", x)
            }
            2 => {
                // buff
                let mut buff = "0x".to_string();
                buff.push_str(&helper_generate_rand_hex_string(64));
                format!(r##"{}"##, buff)
            }
            _ => {
                unreachable!("should only be generating numbers in the range 0..=2.")
            }
        };

        body.push_str(&*format!("({} {}) ", function_name, arg));
    }

    GenOutput::new(None, body, 1)
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
    let privk = Secp256k1PrivateKey::new().to_hex();

    format!("0x{} ", privk)
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
    println!("{}", body);

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
    println!("{}", body);

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

/// ////////////////////////////////////////
/// NON FUNGIBLE TOKEN GENERATOR FUNCTIONS
/// ////////////////////////////////////////

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

// Returns statement, token_name, the type of the nft, and option for the length of the nft if it is a string
fn helper_define_non_fungible_token_statement(
    allow_bool_type: bool,
) -> (String, String, String, Option<u16>) {
    let mut rng = rand::thread_rng();
    let token_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let (nft_type, nft_len) = helper_gen_clarity_type(allow_bool_type, false, false);
    let args = match nft_len {
        Some(length) => format!("{} ({} {})", token_name, nft_type, length),
        None => format!("{} {}", token_name, nft_type),
    };

    let statement = format!("(define-non-fungible-token {}) ", args);
    (statement, token_name, nft_type.to_string(), nft_len)
}

fn helper_gen_clarity_value(
    value_type: &str,
    num: u16,
    value_len: usize,
    list_type: Option<&str>,
) -> String {
    let mut rng = rand::thread_rng();
    match value_type {
        "int" => format!("{}", num),
        "uint" => format!("u{}", num),
        "buff" => {
            let mut buff = "0x".to_string();
            buff.push_str(&helper_generate_rand_hex_string(value_len));
            buff
        }
        "string-ascii" => {
            let ascii_string = helper_generate_rand_hex_string(value_len);
            format!(r##""{}""##, ascii_string)
        }
        "string-utf8" => {
            let utf8_string = helper_generate_rand_hex_string(value_len);
            format!(r##"u"{}""##, utf8_string)
        }
        "bool" => {
            let rand_bool = rng.gen_bool(0.5);
            format!("{}", rand_bool)
        }
        "list" => {
            let list_type = list_type.unwrap();
            let args = (0..value_len)
                .map(|_| helper_gen_clarity_value(&list_type, num, 0, None))
                .collect::<Vec<String>>()
                .join(" ");
            format!("(list {})", args)
        }
        _ => {
            unreachable!("should only be generating the types int, uint, buff, string-ascii, string-utf8, bool.")
        }
    }
}

fn helper_gen_random_clarity_value(num: u16) -> String {
    let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
    helper_gen_clarity_value(
        &clarity_type,
        num,
        length.map_or(0, |len| len as usize),
        None,
    )
}

/// cost_function: NftMint
/// input_size: size of type signature of asset
/// TODO - take in input size
fn gen_nft_mint(scale: u16, input_size: u16) -> GenOutput {
    let mut body = String::new();
    let (statement, token_name, nft_type, nft_len) =
        helper_define_non_fungible_token_statement(false);

    let nft_len = nft_len.map_or(0, |len| len) as usize;
    for i in 0..scale {
        let principal_data = helper_create_principal();
        let nft_value = helper_gen_clarity_value(&nft_type, i, nft_len, None);

        let statement = format!(
            "(nft-mint? {} {} {}) ",
            token_name, nft_value, principal_data
        );
        body.push_str(&statement);
    }
    println!("{}", body);

    GenOutput::new(Some(statement), body, 1)
}

fn helper_create_nft_fn_boilerplate() -> (String, String, String, String, String) {
    let mut body = String::new();
    let (statement, token_name, nft_type, nft_len) =
        helper_define_non_fungible_token_statement(false);
    body.push_str(&statement);

    let nft_len = nft_len.map_or(0, |len| len) as usize;
    let nft_value = helper_gen_clarity_value(&nft_type, 0, nft_len, None);
    let invalid_nft_value = helper_gen_clarity_value(&nft_type, 0, nft_len, None);
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
        nft_value,
        invalid_nft_value,
    )
}

/// cost_function: NftTransfer
/// input_size: size of type signature of asset
/// TODO - take in input size
fn gen_nft_transfer(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let (mut setup, token_name, mut owner_principal, nft_value, _) =
        helper_create_nft_fn_boilerplate();
    for _ in 0..scale {
        let next_principal = helper_create_principal();
        let args = format!(
            "{} {} {} {}",
            token_name, nft_value, owner_principal, next_principal
        );
        body.push_str(&*format!("({} {}) ", function_name, args));

        owner_principal = next_principal;
    }

    GenOutput::new(Some(setup), body, 1)
}

/// cost_function: NftOwner
/// input_size: size of type signature of asset
/// TODO - take in input size
fn gen_nft_owner(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let (mut setup, token_name, _, nft_value, invalid_nft_value) =
        helper_create_nft_fn_boilerplate();
    for _ in 0..scale {
        let curr_nft_value = match rng.gen_bool(0.5) {
            true => {
                // use valid nft value
                &nft_value
            }
            false => {
                // use invalid nft value
                &invalid_nft_value
            }
        };
        let args = format!("{} {}", token_name, curr_nft_value);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    GenOutput::new(Some(setup), body, 1)
}

/// cost_function: NftBurn
/// input_size: size of type signature of asset
/// TODO - take in input size
fn gen_nft_burn(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let (mut setup, token_name, mut owner_principal, nft_value, _) =
        helper_create_nft_fn_boilerplate();
    for _ in 0..scale {
        let args = format!("{} {} {}", token_name, nft_value, owner_principal);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    GenOutput::new(Some(setup), body, 1)
}

/// ////////////////////////////////////////
/// TUPLE GENERATOR FUNCTIONS
/// ////////////////////////////////////////

fn helper_generate_tuple(input_size: u16) -> String {
    let mut rng = rand::thread_rng();
    let tuple_vals = (0..input_size)
        .map(|i| format!("(id{} {})", i, rng.gen::<u32>()))
        .collect::<Vec<String>>()
        .join(" ");

    format!("(tuple {}) ", tuple_vals)
}

/// cost_function: TupleGet
/// input_size: length of tuple data
fn gen_tuple_get(scale: u16, input_size: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    let tuple = helper_generate_tuple(input_size);

    for _ in 0..scale {
        body.push_str(&*format!(
            "(get id{} test-tuple) ",
            rng.gen_range(0..input_size)
        ));
    }
    println!("{}", tuple);

    GenOutput::new(
        None,
        format!("(let ((test-tuple {})) {})", tuple, body),
        input_size,
    )
}

/// cost_function: TupleMerge
/// input_size: double arg function
/// TODO - perhaps does not need to take in input size here - check graphs
fn gen_tuple_merge(scale: u16, input_size: u16) -> GenOutput {
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
        input_size,
    )
}

/// cost_function: TupleCons
/// input_size: number of bindings in the tuple statement
fn gen_tuple_cons(scale: u16, input_size: u16) -> GenOutput {
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
            let clarity_val = helper_gen_random_clarity_value(num);
            format!("(some {})", clarity_val)
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

fn helper_gen_random_response_value(num: u16, only_ok: bool, only_err: bool) -> String {
    let mut rng = rand::thread_rng();
    let clarity_val = helper_gen_random_clarity_value(num);
    let p = if only_ok {
        0.0
    } else if only_err {
        1.0
    } else {
        0.5
    };
    match rng.gen_bool(p) {
        true => {
            format!("(err {})", clarity_val)
        }
        false => {
            format!("(ok {})", clarity_val)
        }
    }
}

/// cost_function: IsOkay, IsErr
/// input_size: single arg function
fn gen_response(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    for i in 0..scale {
        let args = helper_gen_random_response_value(i, false, false);
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
            helper_gen_random_response_value(i, true, false),
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
                length.map_or(0, |len| len as usize),
                None,
            );
            args = format!("{} {}", args, clarity_val)
        }
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    println!("{}", body);

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
        let mut args = helper_gen_random_response_value(i, false, true);

        if ret_value {
            let clarity_val = helper_gen_random_clarity_value(i);
            args = format!("{} {}", args, clarity_val)
        }
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

fn helper_create_map() -> (
    String,
    String,
    String,
    String,
    Option<u16>,
    String,
    String,
    Option<u16>,
) {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    let map_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let key_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let (key_type, key_type_len) = helper_gen_clarity_type(true, false, false);
    let key_args = match key_type_len {
        Some(length) => format!("{{ {}: ({} {}) }}", key_name, key_type, length),
        None => format!("{{ {}: {} }}", key_name, key_type),
    };

    let value_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let (value_type, value_type_len) = helper_gen_clarity_type(true, false, false);
    let value_args = match value_type_len {
        Some(length) => format!("{{ {}: ({} {}) }}", value_name, value_type, length),
        None => format!("{{ {}: {} }}", value_name, value_type),
    };
    body.push_str(&*format!(
        "(define-map {} {} {}) ",
        map_name, key_args, value_args
    ));
    (
        body,
        map_name,
        key_name,
        key_type,
        key_type_len,
        value_name,
        value_type,
        value_type_len,
    )
}

/// cost_function: CreateMap
/// input_size: sum of key type size and value type size
///     `u64::from(key_type.size()).cost_overflow_add(u64::from(value_type.size()))`
/// TODO - incorporate input size
fn gen_create_map(_function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        let (statement, _, _, _, _, _, _, _) = helper_create_map();
        body.push_str(&statement);
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

// setEntry is the cost for map-delete, map-insert, & map-set
// q: only ever deleting non-existent key; should we change that?
/// cost_function: SetEntry
/// input_size: sum of key type size and value type size
/// TODO - incorporate input size
fn gen_set_entry(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let (
        statement,
        map_name,
        key_name,
        key_type,
        key_type_len,
        value_name,
        value_type,
        value_type_len,
    ) = helper_create_map();
    for i in 0..scale {
        let curr_key = helper_gen_clarity_value(
            &key_type,
            i,
            key_type_len.map_or(0, |len| len as usize),
            None,
        );
        let curr_value = helper_gen_clarity_value(
            &value_type,
            i,
            value_type_len.map_or(0, |len| len as usize),
            None,
        );
        let statement = match rng.gen_range(0..3) {
            0 => {
                format!(
                    "(map-set {} {{ {}: {} }} {{ {}: {} }}) ",
                    map_name, key_name, curr_key, value_name, curr_value
                )
            }
            1 => {
                format!(
                    "(map-insert {} {{ {}: {} }} {{ {}: {} }}) ",
                    map_name, key_name, curr_key, value_name, curr_value
                )
            }
            2 => {
                format!(
                    "(map-delete {} {{ {}: {} }}) ",
                    map_name, key_name, curr_key
                )
            }
            _ => unreachable!("should only gen numbers from 0 to 2 inclusive"),
        };
        body.push_str(&statement);
    }
    println!("{}", body);

    GenOutput::new(Some(statement), body, 1)
}

// TODO: fix input size calculation - reed
/// cost_function: FetchEntry
/// input_size: sum of key type size and value type size
fn gen_fetch_entry(scale: u16) -> GenOutput {
    let mut body = String::new();
    let (
        mut setup,
        map_name,
        key_name,
        key_type,
        key_type_len,
        value_name,
        value_type,
        value_type_len,
    ) = helper_create_map();

    // insert a value into map
    let curr_key = helper_gen_clarity_value(
        &key_type,
        23,
        key_type_len.map_or(0, |len| len as usize),
        None,
    );
    let curr_value = helper_gen_clarity_value(
        &value_type,
        89,
        value_type_len.map_or(0, |len| len as usize),
        None,
    );

    setup.push_str(&format!(
        "(map-insert {} {{ {}: {} }} {{ {}: {} }}) ",
        map_name, key_name, curr_key, value_name, curr_value
    ));
    for i in 0..scale {
        let curr_key_value = if i % 2 == 0 {
            helper_gen_clarity_value(
                &key_type,
                i,
                key_type_len.map_or(0, |len| len as usize),
                None,
            )
        } else {
            curr_key.clone()
        };

        let statement = format!(
            "(map-get? {} {{ {}: {} }}) ",
            map_name, key_name, curr_key_value
        );
        body.push_str(&statement);
    }
    println!("{}", body);

    GenOutput::new(
        Some(setup),
        body,
        key_type_len.unwrap() + value_type_len.unwrap(),
    )
}

/// cost_function: CreateVar
/// input_size: value type size
///     `value_type.size()`
/// TODO - incorporate size
fn gen_create_var(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for i in 0..scale {
        let var_name = helper_generate_rand_char_string(rng.gen_range(10..20));
        let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
        let clarity_value =
            helper_gen_clarity_value(&clarity_type, i, length.map_or(0, |len| len as usize), None);
        let args = match length {
            Some(l) => format!("{} ({} {}) {}", var_name, clarity_type, l, clarity_value),
            None => format!("{} {} {}", var_name, clarity_type, clarity_value),
        };
        body.push_str(&*format!("(define-data-var {}) ", args));
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
}


/// cost_function: FetchVar, SetVar
/// input_size: value type size
///     `data_types.value_type.size()`
/// TODO - incorporate this
fn gen_var_set_get(function_name: &'static str, scale: u16, set: bool) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    let var_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
    let clarity_value = helper_gen_clarity_value(
        &clarity_type,
        rng.gen_range(10..200),
        length.map_or(0, |len| len as usize),
        None,
    );
    let args = match length {
        Some(l) => format!("{} ({} {}) {}", var_name, clarity_type, l, clarity_value),
        None => format!("{} {} {}", var_name, clarity_type, clarity_value),
    };
    let setup = format!("({} {}) ", "define-data-var", args);
    for i in 0..scale {
        let args = if set {
            let new_val = helper_gen_clarity_value(
                &clarity_type,
                i,
                length.map_or(0, |len| len as usize),
                None,
            );
            format!("{} {}", var_name, new_val)
        } else {
            format!("{}", var_name)
        };
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    println!("{}", body);

    GenOutput::new(Some(setup), body, 1)
}

/// cost_function:
/// input_size:
/// print: size of given Value for print
/// SomeCons/OkCons/ErrCons: single arg function
/// begin: multi arg function
/// TODO - fix above
fn gen_single_clar_value(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    for i in 0..scale {
        let args = helper_gen_random_clarity_value(i);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

/// cost_function: If
/// input_size: 0
fn gen_if(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for i in 0..scale {
        let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
        let if_case_value =
            helper_gen_clarity_value(&clarity_type, i, length.map_or(0, |len| len as usize), None);
        let else_case_value =
            helper_gen_clarity_value(&clarity_type, i, length.map_or(0, |len| len as usize), None);
        let curr_bool = rng.gen_bool(0.5);

        body.push_str(&*format!(
            "({} {} {} {}) ",
            function_name, curr_bool, if_case_value, else_case_value
        ));
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: Asserts
/// input_size: 0
fn gen_asserts(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    for i in 0..scale {
        let clarity_val = helper_gen_random_clarity_value(i);
        body.push_str(&*format!("({} true {}) ", function_name, clarity_val));
    }
    println!("{}", body);

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
                    )
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
                    )
                })
                .collect()
        }
    }
}

/// cost_function: Concat
/// input_size: sum of Value size of input sequences
///     `u64::from(wrapped_seq.size()).cost_overflow_add(u64::from(other_wrapped_seq.size())`
/// TODO - fix
fn gen_concat(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        let (list_type, _) = helper_gen_clarity_type(true, false, true);
        let operands = helper_generate_sequences(&list_type, 2);
        body.push_str(&*format!(
            "({} {} {}) ",
            function_name, operands[0], operands[1]
        ));
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
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
        body.push_str(&*format!("({} {} {}) ", function_name, operand[0], len));
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

/// cost_function: BindName
/// input_size: 0
fn gen_define_constant(function_name: &'static str, scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for i in 0..scale {
        let name = helper_generate_rand_char_string(rng.gen_range(10..50));
        let value = helper_gen_random_clarity_value(i);
        body.push_str(&*format!("({} {} {}) ", function_name, name, value));
    }
    println!("{}", body);

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
            helper_gen_clarity_value(&clarity_type, i, length.map_or(0, |len| len as usize), None);
        let opt_string = match rng.gen_bool(0.5) {
            true => "none".to_string(),
            false => {
                let inner_val = helper_gen_clarity_value(
                    &clarity_type,
                    i,
                    length.map_or(0, |len| len as usize),
                    None,
                );
                format!("(some {})", inner_val)
            }
        };
        body.push_str(&*format!(
            "({} {} {}) ",
            function_name, default_val, opt_string
        ));
    }
    println!("{}", body);

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
    println!("{}", body);

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
                let match_val = helper_gen_random_response_value(i, false, false);
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
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

/// cost_function: Let
/// input_size: number of bindings in the let statement
/// TODO - factor in input size
fn gen_let(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for i in 0..scale {
        let var_name = helper_generate_rand_char_string(rng.gen_range(10..20));
        let var_value = helper_gen_random_clarity_value(i);
        let statement = format!("(let (({} {})) (no-op)) ", var_name, var_value);
        body.push_str(&statement);
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

fn helper_generate_random_sequence() -> (String, usize, String) {
    let mut rng = rand::thread_rng();
    let value_len = rng.gen_range(2..50) * 2;
    match rng.gen_bool(0.75) {
        true => {
            // non-list case
            let (clarity_type, _) = helper_gen_clarity_type(true, true, false);
            let value =
                helper_gen_clarity_value(&clarity_type, rng.gen_range(2..50), value_len, None);
            (value, value_len, clarity_type)
        }
        false => {
            // list case
            let (list_type, _) = helper_gen_clarity_type(true, false, true);
            let value =
                helper_gen_clarity_value("list", rng.gen_range(2..50), value_len, Some(&list_type));
            (value, value_len, list_type)
        }
    }
}

/// cost_function: IndexOf
/// input_size: double arg function
fn gen_index_of(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let (seq, _, seq_inner_type) = helper_generate_random_sequence();
        let item_len = if seq_inner_type == "buff" { 2 } else { 1 };
        let item_val =
            helper_gen_clarity_value(&seq_inner_type, rng.gen_range(2..50), item_len, None);
        let statement = format!("(index-of {} {}) ", seq, item_val);
        body.push_str(&statement);
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
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
    println!("{}", body);

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
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

// q: not sure if we are testing worst case here; not allowing list of buffs, for example
/// cost_function: Append
/// input_size: max of value type sig size (which is to be appended) and size of the entry type of the list
///     `u64::from(cmp::max(entry_type.size(), element_type.size()))`
/// TODO: take in input size
fn gen_append(scale: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let (list_type, _) = helper_gen_clarity_type(true, false, true);
        let list_val = helper_gen_clarity_value(
            "list",
            rng.gen_range(2..50),
            rng.gen_range(2..50) * 2,
            Some(&list_type),
        );
        let new_item_val = helper_gen_clarity_value(&list_type, rng.gen_range(2..50), 1, None);
        let statement = format!("(append {} {}) ", list_val, new_item_val);
        body.push_str(&statement);
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

/// cost_function: ListCons
/// input_size: sum of Value sizes of args to be added
///     ```for a in args.iter() {
///         arg_size = arg_size.cost_overflow_add(a.size().into())?;
///     }```
/// TODO - make sure input_size used appropriately
fn gen_list_cons(scale: u16, input_size: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let (list_type, _) = helper_gen_clarity_type(true, false, true);
        let list_len = input_size;
        let item_val = "true";
        let mut args = String::new();
        for _ in 0..list_len {
            args.push_str(&*format!("{} ", item_val));
        }
        let statement = format!("(list {}) ", args);
        body.push_str(&statement);
    }
    println!("{}", body);

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
        let statement = format!("(filter no-op {}) ", list_val);
        body.push_str(&statement);
    }
    println!("{}", body);

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
        let statement = format!("(fold no-op {} true) ", list_val);
        body.push_str(&statement);
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

/// cost_function: Map
/// input_size: number of arguments
fn gen_map(scale: u16, input_size: u16) -> GenOutput {
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
            lists.push_str(&list_val);
            lists.push_str(" ");
        }

        let statement = format!("(map no-op {}) ", lists);
        body.push_str(&statement);
    }
    println!("{}", body);

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

    // must use block 5 here b/c it has a hardcoded id_bhh
    // TODO: consider hardcoding more id_bhhs and making this random
    for _ in 0..scale {
        body.push_str(format!("(get-block-info? {} u5) ", props.choose(&mut rng).unwrap()).as_str())
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: AtBlock
/// input_size: 0
fn gen_at_block(scale: u16) -> GenOutput {
    let mut body = String::new();

    for _ in 0..scale {
        body.push_str("(at-block 0x0000000000000000000000000000000000000000000000000000000000000000 (no-op)) ");
    }

    GenOutput::new(None, body, 1)
}

// helper function used in bench.rs
pub fn gen_read_only_func(scale: u16) -> GenOutput {
    let mut body = String::new();
    let arith_string = gen_arithmetic("+", scale, 2).body;
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

/// cost_function: AnalysisOptionCons
/// input_size: 0
fn gen_analysis_option_cons(scale: u16) -> GenOutput {
    let mut body = String::new();
    for i in 0..scale {
        let args = helper_gen_random_clarity_value(i);
        body.push_str(&*format!("{} ", args));
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

/// cost_function: AnalysisOptionCheck
/// input_size: 0
fn gen_analysis_option_check(scale: u16) -> GenOutput {
    let mut body = String::new();
    for i in 0..scale {
        let args = helper_gen_random_response_value(i, false, false);
        body.push_str(&*format!("{} ", args));
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

/// cost_function: AnalysisBindName
/// input_size: type size (could be value, constant, function, total map size, etc.)
///     `v_type.type_size()`
fn gen_analysis_bind_name(scale: u16, input_size: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let var_name = "dummy-name";
        let clar_type = helper_make_clarity_type_for_sized_type_sig(input_size);
        let clar_val = helper_make_clarity_value_for_sized_type_sig(input_size);

        match rng.gen_range(0..3) {
            0 => {
                let args = format!("{} {} {}", var_name, clar_type, clar_val);
                body.push_str(&*format!("(define-data-var {}) ", args));
            }
            1 => {
                let args = format!("{} {}", var_name, clar_val);
                body.push_str(&*format!("(define-constant {}) ", args));
            }
            2 => {
                let args = format!("{} {}", var_name, clar_type);
                body.push_str(&*format!("(define-non-fungible-token {}) ", args));
            }
            _ => unreachable!("Numbers out of range should not be generated."),
        };
    }
    println!("{}", body);

    GenOutput::new(None, body, input_size)
}

/// cost_function: AnalysisListItemsCheck
/// input_size: type signature size of item
///     `type_arg.type_size()`
fn gen_analysis_list_items_check(scale: u16, input_size: u16) -> GenOutput {
    let mut body = String::new();
    for i in 0..scale {
        let (base_type, _) = helper_gen_clarity_type(true, false, true);
        body.push_str("(");
        for _ in 0..input_size {
            let base_val = helper_gen_clarity_value(&base_type, i, 0, None);
            body.push_str(&*format!("{} ", base_val));
        }
        body.push_str(") ");
    }
    println!("{}", body);

    GenOutput::new(None, body, input_size)
}

/// cost_function: AnalysisCheckTupleGet
/// input_size: length of tuple
///     `tuple_type_sig.len()`
fn gen_analysis_tuple_get(scale: u16, input_size: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        body.push_str(&helper_generate_tuple(input_size));
    }
    println!("{}", body);

    GenOutput::new(None, body, input_size)
}

/// cost_function: AnalysisCheckTupleMerge
/// input_size: length of second tuple
///     `update.len()`
fn gen_analysis_tuple_merge(scale: u16, input_size: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        body.push_str("(");
        body.push_str(&helper_generate_tuple(input_size));
        body.push_str(&helper_generate_tuple(input_size));
        body.push_str(") ");
    }
    println!("{}", body);

    GenOutput::new(None, body, input_size)
}

/// cost_function: AnalysisCheckTupleCons
/// input_size: number of arguments provided
///     `args.len()`
fn gen_analysis_tuple_cons(scale: u16, input_size: u16) -> GenOutput {
    let mut body = String::new();
    for i in 0..scale {
        body.push_str("(");
        for _ in 0..input_size {
            let var_val = helper_gen_random_clarity_value(i);
            let var_name = helper_generate_rand_char_string(10);
            body.push_str(&*format!("({} {}) ", var_name, var_val));
        }
        body.push_str(") ");
    }
    println!("{}", body);

    GenOutput::new(None, body, input_size)
}

/// cost_function: AnalysisTupleItemsCheck
/// input_size: type signature size of value
///     `var_type.type_size()`
fn gen_analysis_tuple_items_check(scale: u16, input_size: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        let field_name = helper_generate_rand_char_string(10);
        let sized_val = helper_make_sized_clarity_value(input_size);
        body.push_str(&*format!("({} {}) ", field_name, sized_val));
    }
    println!("{}", body);

    GenOutput::new(None, body, input_size)
}

/// cost_function: AnalysisCheckLet
/// input_size: number of arguments total (the binding list counts as an arg)
///     `args.len()`
fn gen_analysis_check_let(scale: u16, input_size: u16) -> GenOutput {
    let mut body = String::new();
    for i in 0..(scale) {
        let no_ops = (0..input_size).map(|_x| "(no-op) ").collect::<String>();
        let var_val = helper_gen_random_clarity_value(i);
        let var_name = helper_generate_rand_char_string(10);
        body.push_str(&*format!("((({} {})) {}) ", var_name, var_val, no_ops));
    }
    println!("{}", body);

    GenOutput::new(None, body, input_size)
}

// note: includes AnalysisLookupFunction cost
/// cost_function: AnalysisIterableFunc
/// input_size: 0 in most cases, `args.len()` in `check_special_map`
/// TODO - check this is benched correctly
fn gen_analysis_iterable_func(scale: u16, input_size: u16) -> GenOutput {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let mut lists = String::new();
        for _ in 0..input_size {
            let list_val = helper_gen_clarity_value("list", rng.gen_range(2..50), 3, Some("int"));
            lists.push_str(&list_val);
            lists.push_str(" ");
        }

        let statement = format!("(no-op {}) ", lists);
        body.push_str(&statement);
    }
    println!("{}", body);

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
fn gen_analysis_storage(scale: u16, input_size: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        let mut defines = String::new();
        for j in 0..input_size {
            let (base_type, _) = helper_gen_clarity_type(true, false, true);
            let base_val = helper_gen_clarity_value(&base_type, j, 0, None);
            let constant_name = helper_generate_rand_char_string(10);
            defines.push_str(&*format!(
                "(define-constant {} {}) ",
                constant_name, base_val
            ));
        }
        let statement = format!("({}) ", defines);
        body.push_str(&statement);
    }
    println!("{}", body);

    GenOutput::new(None, body, input_size)
}

/// cost_function: AstCycleDetection, LookupFunction
/// input_size: number of edges in AST / 0
///     `self.graph.edges_count()`
fn gen_ast_cycle_detection(input_size: u16) -> GenOutput {
    let mut body = String::new();
    body.push_str(&*format!("(define-read-only (fn-0) (no-op)) "));
    for i in 1..(input_size + 1) {
        body.push_str(&*format!("(define-read-only (fn-{}) (fn-{})) ", i, i - 1));
    }
    println!("{}", body);

    GenOutput::new(None, body, input_size)
}

/// cost_function: AstParse, AnalysisTypeCheck
/// input_size: `source_code.len()` / `return_type.type_size()`
fn gen_empty() -> GenOutput {
    GenOutput::new(None, "".to_string(), 1)
}

/// cost_function: ContractStorage
/// input_size: length of contract string
///     `contract_string.len()`
fn gen_contract_storage(input_size: u16) -> GenOutput {
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
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

/// cost_function: PrincipalOf
/// input_size: 0
fn gen_principal_of(scale: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        body.push_str(&helper_create_principal_in_hex());
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

/// cost_function: AnalysisTypeLookup
/// input_size: type signature size of value being looked up
///     `expected_asset_type.type_size()`
fn gen_analysis_type_lookup(scale: u16, input_size: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        let asset_name = helper_generate_rand_char_string(10);
        let owner = helper_create_principal();
        let tuple = helper_make_clarity_value_for_sized_type_sig(input_size);
        body.push_str(&*format!("({} {} {}) ", asset_name, tuple, owner));
    }
    println!("{}", body);

    GenOutput::new(None, body, input_size)
}

/// cost_function: AnalysisTypeAnnotate, AnalysisLookupVariableConst
/// input_size: type signature size of SymbolicExpression / 0
///     `type_sig.type_size()` / 0
/// TODO - make sure first cost function is using input size
fn gen_analysis_lookup_variable_const(scale: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        body.push_str(&helper_generate_rand_char_string(10));
        body.push_str(" ");
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

/// cost_function: AnalysisVisit, AnalysisLookupFunction
/// input_size: 0
fn gen_no_op_with_scale_repetitions(scale: u16) -> GenOutput {
    let mut body = String::new();
    for _ in 0..scale {
        body.push_str("(no-op) ")
    }
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

/// cost_function: AnalysisLookupFunctionTypes, AnalysisUseTraitEntry
/// input_size: type signature size of function / sum of type size of function sigs in a trait
///     `func_signature.total_type_size()` / `trait_type_size(&trait_sig)`
fn gen_analysis_lookup_function_types(input_size: u16) -> GenOutput {
    let args = (0..input_size).map(|_x| "uint ").collect::<String>();
    let dummy_fn = format!("(dummy-fn ({}) (response uint uint))", args);
    let body = format!("(define-trait dummy-trait ({})) ", dummy_fn);
    println!("{}", body);

    GenOutput::new(None, body, 1)
}

/// cost_function: AnalysisGetFunctionEntry, UserFunctionApplication
/// input_size: type size of function signature / number of arguments
///    `func_signature.total_type_size()` / `self.arguments.len()`
fn gen_analysis_get_function_entry(input_size: u16) -> GenOutput {
    let mut body = String::new();
    let args = (0..input_size)
        .map(|i| format!(" (f{} uint) ", i))
        .collect::<String>();
    body.push_str(&*format!("(define-read-only (dummy-fn {}) (no-op)) ", args));

    println!("{}", body);

    GenOutput::new(None, body, 1)
}

/// cost_function: InnerTypeCheckCost
/// input_size: type signature size of argument
///     `arg_type.size()`
fn gen_inner_type_check_cost(input_size: u16) -> GenOutput {
    let mut body = String::new();
    let clar_type = make_clarity_type_for_sized_value(input_size);
    body.push_str(&*format!(
        "(define-read-only (dummy-fn (f0 {})) (no-op)) ",
        clar_type
    ));

    println!("{}", body);

    GenOutput::new(None, body, 1)
}

/// cost_function: StxTransfer
/// input_size: 0
pub fn gen_stx_transfer(scale: u16) -> GenOutput {
    let mut body = String::new();

    for _ in 0..scale {
        body.push_str("(stx-transfer? u1 tx-sender 'S0G0000000000000000000000000000015XM0F7) ");
    }

    GenOutput::new(None, body, 1)
}

/// cost_function: StxBalance
/// input_size: 0
pub fn gen_stx_get_balance(scale: u16) -> GenOutput {
    let mut body = String::new();

    for _ in 0..scale {
        body.push_str("(stx-get-balance 'S1G2081040G2081040G2081040G208105NK8PE5) ");
    }

    GenOutput::new(None, body, 1)
}
////////////////////// ANALYSIS PASS COSTS /////////////////////////

pub fn gen_analysis_pass_read_only(input_size: u16) -> GenOutput {
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
    println!("{}", body);

    GenOutput::new(None, body, input_size)
}

pub fn gen_analysis_pass_arithmetic_only(input_size: u16) -> GenOutput {
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
    println!("{}", body);

    GenOutput::new(None, body, input_size)
}

pub fn define_dummy_trait(i: u16, clarity_type: &str) -> String {
    let dummy_fn = format!("(dummy-fn-{} ({}) (response uint uint))", i, clarity_type);
    format!("(define-trait dummy-trait-{} ({})) ", i, dummy_fn)
}

pub fn gen_analysis_pass_trait_checker(input_size: u16) -> GenOutput {
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

pub fn gen_analysis_pass_type_checker(input_size: u16) -> GenOutput {
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
    println!("{}", body);

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

/// Returns tuple of optional setup clarity code, and "main" clarity code
/// The `reviewed` comment above each cost function should list the GitHub usernames of those
///    who have verified that the benchmark for that cost function seems accurate (given the code
///    in `benches.rs`, the code in `generators.rs`, and the benchmark data.
pub fn gen(function: ClarityCostFunction, scale: u16, input_size: u16) -> GenOutput {
    match function {
        /// Arithmetic ///////////////////////
        /// reviewed:
        ClarityCostFunction::Add => gen_arithmetic("+", scale, input_size),
        ClarityCostFunction::Sub => gen_arithmetic("-", scale, input_size),
        ClarityCostFunction::Mul => gen_arithmetic("*", scale, input_size),
        ClarityCostFunction::Div => gen_arithmetic("/", scale, input_size),
        ClarityCostFunction::Sqrti => gen_arithmetic("sqrti", scale, 1),
        ClarityCostFunction::Log2 => gen_arithmetic("log2", scale, 1),
        ClarityCostFunction::Mod => gen_arithmetic("mod", scale, input_size),

        /// reviewed:
        ClarityCostFunction::Pow => gen_pow(scale),

        /// Logic /////////////////////////////
        /// reviewed:
        ClarityCostFunction::Le => gen_cmp("<", scale),
        ClarityCostFunction::Leq => gen_cmp("<=", scale),
        ClarityCostFunction::Ge => gen_cmp(">", scale),
        ClarityCostFunction::Geq => gen_cmp(">=", scale),


        /// Boolean ///////////////////////////
        /// reviewed:
        ClarityCostFunction::And => gen_logic("and", scale, input_size),
        ClarityCostFunction::Or => gen_logic("or", scale, input_size),
        ClarityCostFunction::Not => gen_logic("not", scale, input_size),
        ClarityCostFunction::Eq => gen_logic("is-eq", scale, input_size),
        /// reviewed:
        ClarityCostFunction::Xor => gen_xor("xor", scale),


        /// Tuples ////////////////////////////
        /// reviewed:
        ClarityCostFunction::TupleGet => gen_tuple_get(scale, input_size),

        /// reviewed:
        ClarityCostFunction::TupleMerge => gen_tuple_merge(scale, input_size),

        /// reviewed:
        ClarityCostFunction::TupleCons => gen_tuple_cons(scale, input_size),


        /// Analysis //////////////////
        /// reviewed:
        ClarityCostFunction::AnalysisTypeAnnotate => gen_analysis_lookup_variable_const(scale),

        /// reviewed:
        ClarityCostFunction::AnalysisTypeCheck => gen_empty(),

        /// reviewed:
        ClarityCostFunction::AnalysisTypeLookup => gen_analysis_type_lookup(scale, input_size),

        /// reviewed:
        ClarityCostFunction::AnalysisVisit => gen_no_op_with_scale_repetitions(scale),

        /// reviewed:
        ClarityCostFunction::AnalysisIterableFunc => gen_analysis_iterable_func(scale, input_size),

        /// reviewed:
        ClarityCostFunction::AnalysisOptionCons => gen_analysis_option_cons(scale),

        /// reviewed:
        ClarityCostFunction::AnalysisOptionCheck => gen_analysis_option_check(scale),

        /// reviewed:
        ClarityCostFunction::AnalysisBindName => gen_analysis_bind_name(scale, input_size),

        /// reviewed:
        ClarityCostFunction::AnalysisListItemsCheck => {
            gen_analysis_list_items_check(scale, input_size)
        }

        /// reviewed:
        ClarityCostFunction::AnalysisCheckTupleGet => gen_analysis_tuple_get(scale, input_size),

        /// reviewed:
        ClarityCostFunction::AnalysisCheckTupleMerge => gen_analysis_tuple_merge(scale, input_size),

        /// reviewed:
        ClarityCostFunction::AnalysisCheckTupleCons => gen_analysis_tuple_cons(scale, input_size),

        /// reviewed:
        ClarityCostFunction::AnalysisTupleItemsCheck => {
            gen_analysis_tuple_items_check(scale, input_size)
        }

        /// reviewed:
        ClarityCostFunction::AnalysisCheckLet => gen_analysis_check_let(scale, input_size),

        /// reviewed:
        ClarityCostFunction::AnalysisLookupFunction => gen_no_op_with_scale_repetitions(scale),

        /// reviewed:
        ClarityCostFunction::AnalysisLookupFunctionTypes => {
            gen_analysis_lookup_function_types(input_size)
        }

        /// reviewed:
        ClarityCostFunction::AnalysisLookupVariableConst => {
            gen_analysis_lookup_variable_const(scale)
        }

        /// reviewed:
        ClarityCostFunction::AnalysisLookupVariableDepth => unimplemented!(), // no gen function needed

        /// reviewed:
        ClarityCostFunction::AnalysisStorage => gen_analysis_storage(scale, input_size),

        /// reviewed:
        ClarityCostFunction::AnalysisUseTraitEntry => {
            gen_analysis_lookup_function_types(input_size)
        }

        /// reviewed:
        ClarityCostFunction::AnalysisGetFunctionEntry => {
            gen_analysis_get_function_entry(input_size)
        }

        /// reviewed:
        ClarityCostFunction::AnalysisFetchContractEntry => unimplemented!(), // not used anywhere


        /// Ast ////////////////////////////////
        /// reviewed:
        ClarityCostFunction::AstParse => gen_empty(),

        /// reviewed:
        ClarityCostFunction::AstCycleDetection => gen_ast_cycle_detection(input_size),

        /// reviewed:
        ClarityCostFunction::ContractStorage => gen_contract_storage(input_size),


        /// Lookup ////////////////////////////////
        /// reviewed:
        ClarityCostFunction::LookupVariableDepth => unimplemented!(), // no gen function needed

        /// reviewed:
        ClarityCostFunction::LookupVariableSize => unimplemented!(),  // no gen function needed

        /// reviewed:
        ClarityCostFunction::LookupFunction => gen_ast_cycle_detection(input_size),


        /// List ////////////////////////////////
        /// reviewed:
        ClarityCostFunction::Map => gen_map(scale, input_size), // includes LookupFunction cost

        /// reviewed:
        ClarityCostFunction::Filter => gen_filter(scale),       // includes LookupFunction cost

        /// reviewed:
        ClarityCostFunction::Fold => gen_fold(scale),           // includes LookupFunction cost

        /// reviewed:
        ClarityCostFunction::Len => gen_len(scale),

        /// reviewed:
        ClarityCostFunction::ElementAt => gen_element_at(scale),

        /// reviewed:
        ClarityCostFunction::IndexOf => gen_index_of(scale),

        /// reviewed:
        ClarityCostFunction::ListCons => gen_list_cons(scale, input_size),

        /// reviewed:
        ClarityCostFunction::Append => gen_append(scale),


        /// Hash ////////////////////////////////
        /// reviewed:
        ClarityCostFunction::Hash160 => gen_hash("hash160", scale),

        /// reviewed:
        ClarityCostFunction::Sha256 => gen_hash("sha256", scale),

        /// reviewed:
        ClarityCostFunction::Sha512 => gen_hash("sha512", scale),

        /// reviewed:
        ClarityCostFunction::Sha512t256 => gen_hash("sha512/256", scale),

        /// reviewed:
        ClarityCostFunction::Keccak256 => gen_hash("keccak256", scale),

        /// reviewed:
        ClarityCostFunction::Secp256k1recover => gen_secp256k1("secp256k1-recover?", scale, false),

        /// reviewed:
        ClarityCostFunction::Secp256k1verify => gen_secp256k1("secp256k1-verify", scale, true),

        /// FT ////////////////////////////////
        /// reviewed:
        ClarityCostFunction::CreateFt => gen_create_ft("define-fungible-token", scale),

        /// reviewed:
        ClarityCostFunction::FtMint => gen_ft_mint("ft-mint?", scale),

        /// reviewed:
        ClarityCostFunction::FtTransfer => gen_ft_transfer("ft-transfer?", scale),

        /// reviewed:
        ClarityCostFunction::FtBalance => gen_ft_balance("ft-get-balance", scale),

        /// reviewed:
        ClarityCostFunction::FtSupply => gen_ft_supply("ft-get-supply", scale),

        /// reviewed:
        ClarityCostFunction::FtBurn => gen_ft_burn("ft-burn?", scale),


        /// NFT ////////////////////////////////
        /// reviewed:
        ClarityCostFunction::CreateNft => unimplemented!(),

        /// reviewed:
        ClarityCostFunction::NftMint => gen_nft_mint(scale, input_size),

        /// reviewed:
        ClarityCostFunction::NftTransfer => gen_nft_transfer("nft-transfer?", scale),

        /// reviewed:
        ClarityCostFunction::NftOwner => gen_nft_owner("nft-get-owner?", scale),

        /// reviewed:
        ClarityCostFunction::NftBurn => gen_nft_burn("nft-burn?", scale),

        /// Stacks ////////////////////////////////
        /// reviewed:
        ClarityCostFunction::PoisonMicroblock => unimplemented!(), // don't need a gen for this

        /// reviewed:
        ClarityCostFunction::BlockInfo => gen_get_block_info(scale),

        /// reviewed:
        ClarityCostFunction::StxBalance => gen_stx_get_balance(scale),

        /// reviewed:
        ClarityCostFunction::StxTransfer => gen_stx_transfer(scale),


        /// Option & result checks ////////////////////////////////
        /// reviewed:
        ClarityCostFunction::IsSome => gen_optional("is-some", scale),

        /// reviewed:
        ClarityCostFunction::IsNone => gen_optional("is-none", scale),

        /// reviewed:
        ClarityCostFunction::IsOkay => gen_response("is-ok", scale),

        /// reviewed:
        ClarityCostFunction::IsErr => gen_response("is-err", scale),

        /// reviewed:
        ClarityCostFunction::DefaultTo => gen_default_to("default-to", scale),


        /// Unwrap functions ////////////////////////////////
        /// reviewed:
        ClarityCostFunction::Unwrap => gen_unwrap("unwrap-panic", scale, false),

        /// reviewed:
        ClarityCostFunction::UnwrapRet => gen_unwrap("unwrap!", scale, true),

        /// reviewed:
        ClarityCostFunction::UnwrapErr => gen_unwrap_err("unwrap-err-panic", scale, false),

        /// reviewed:
        ClarityCostFunction::UnwrapErrOrRet => gen_unwrap_err("unwrap-err!", scale, true),

        /// reviewed:
        ClarityCostFunction::TryRet => gen_unwrap("try!", scale, false),


        /// Map ////////////////////////////////
        /// reviewed:
        ClarityCostFunction::CreateMap => gen_create_map("define-map", scale),

        /// reviewed:
        ClarityCostFunction::FetchEntry => gen_fetch_entry(scale), // map-get?

        /// reviewed:
        ClarityCostFunction::SetEntry => gen_set_entry(scale),     // map-set


        /// Var ////////////////////////////////
        /// reviewed:
        ClarityCostFunction::CreateVar => gen_create_var(scale),

        /// reviewed:
        ClarityCostFunction::FetchVar => gen_var_set_get("var-get", scale, false),

        /// reviewed:
        ClarityCostFunction::SetVar => gen_var_set_get("var-set", scale, true),

        /// reviewed:
        ClarityCostFunction::BindName => gen_define_constant("define-constant-bench", scale), // used for define var and define function


        /// Functions with single clarity value input ////////////////////////////////
        /// reviewed:
        ClarityCostFunction::Print => gen_single_clar_value("print", scale),

        /// reviewed:
        ClarityCostFunction::SomeCons => gen_single_clar_value("some", scale),

        /// reviewed:
        ClarityCostFunction::OkCons => gen_single_clar_value("ok", scale),

        /// reviewed:
        ClarityCostFunction::ErrCons => gen_single_clar_value("err", scale),

        /// reviewed:
        ClarityCostFunction::Begin => gen_single_clar_value("begin", scale),


        /// Type Checking ////////////////////////////////
        /// reviewed:
        ClarityCostFunction::InnerTypeCheckCost => gen_inner_type_check_cost(input_size),

        /// reviewed:
        ClarityCostFunction::TypeParseStep => gen_type_parse_step(scale), // called by `parse_type_repr` in `signatures.rs` (takes in symbolic expression)


        /// Uncategorized ////////////////////////////////
        /// reviewed:
        ClarityCostFunction::If => gen_if("if", scale),

        /// reviewed:
        ClarityCostFunction::Asserts => gen_asserts("asserts!", scale),

        /// reviewed:
        ClarityCostFunction::Concat => gen_concat("concat", scale),

        /// reviewed:
        ClarityCostFunction::IntCast => gen_int_cast(scale),

        /// reviewed:
        ClarityCostFunction::Let => gen_let(scale),

        /// reviewed:
        ClarityCostFunction::Match => gen_match(scale),

        /// reviewed:
        ClarityCostFunction::AsMaxLen => gen_as_max_len("as-max-len?", scale),

        /// reviewed:
        ClarityCostFunction::UserFunctionApplication => gen_analysis_get_function_entry(input_size),

        /// reviewed:
        ClarityCostFunction::ContractCall => gen_contract_call(scale),

        /// reviewed:
        ClarityCostFunction::ContractOf => gen_contract_of(scale),

        /// reviewed:
        ClarityCostFunction::PrincipalOf => gen_principal_of(scale),

        /// reviewed:
        ClarityCostFunction::AtBlock => gen_at_block(scale),

        /// reviewed:
        ClarityCostFunction::LoadContract => unimplemented!(), // called at start of execute_contract
    }
}


/// Returns tuple of optional setup clarity code, and "main" clarity code
pub fn gen_analysis_pass(
    function: AnalysisCostFunction,
    _scale: u16,
    input_size: u16,
) -> GenOutput {
    match function {
        /// reviewed:
        AnalysisCostFunction::ReadOnly => gen_analysis_pass_read_only(input_size),

        /// reviewed:
        AnalysisCostFunction::TypeChecker => gen_analysis_pass_type_checker(input_size),

        /// reviewed:
        AnalysisCostFunction::TraitChecker => gen_analysis_pass_trait_checker(input_size),

        /// reviewed:
        AnalysisCostFunction::ArithmeticOnlyChecker => {
            gen_analysis_pass_arithmetic_only(input_size)
        }
    }
}