use blockstack_lib::vm::costs::cost_functions::ClarityCostFunction;
use rand::{Rng, RngCore};
use rand::distributions::Uniform;
use blockstack_lib::util::secp256k1::{Secp256k1PrivateKey, Secp256k1PublicKey};
use blockstack_lib::burnchains::PrivateKey;

use secp256k1::Message as LibSecp256k1Message;
use blockstack_lib::util::hash::to_hex;
use blockstack_lib::chainstate::stacks::{C32_ADDRESS_VERSION_TESTNET_SINGLESIG, StacksPublicKey};
use blockstack_lib::address::AddressHashMode;
use blockstack_lib::types::chainstate::StacksAddress;
use rand::distributions::uniform::{UniformChar, UniformSampler};
use rand::rngs::ThreadRng;
use blockstack_lib::vm::analysis::contract_interface_builder::ContractInterfaceAtomType::{principal, list};
use std::cmp::min;

// generate arithmetic function call
pub fn gen_arithmetic(function_name: &'static str, scale: u16, input_size: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let args = (0..input_size)
            .map(|_| {
                let max = i128::MAX / (i128::from(input_size) + 1);
                format!("{}", rng.gen_range(1..max).to_string())
            })
            .collect::<Vec<String>>()
            .join(" ");
        body.push_str(&format!("({} {}) ", function_name, args));
    }

    body
}

fn gen_pow(scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let n1: u16 = rng.gen();
        let n2: u8 = rng.gen_range(0..8);
        body.push_str(&*format!("(pow u{} u{}) ", n1, n2));
    }

    body
}

fn gen_cmp(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let n1: u128 = rng.gen();
        let n2: u128 = rng.gen();
        body.push_str(&*format!("({} u{} u{}) ", function_name, n1, n2));
    }

    body
}

fn gen_logic(function_name: &'static str, scale: u16, input_size: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let bools = vec!["true", "false"];

    for _ in 0..scale {
        let args = (0..input_size)
            .map(|_| {
                match function_name {
                    "and" => format!("true"),
                    "or"  => format!("false"),
                    _     => format!("{}", bools[rng.gen_range(0..=1)]),
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        body.push_str(&*format!("({} {}) ", function_name, args));
    }

    /* match function_name {
        "and" => {
            for _ in 0..scale {
                let args = (0..input_size)
                    .map(|_| format!("true"))
                    .collect::<Vec<String>>()
                    .join(" ");
                body.push_str(&*format!("({} {}) ", function_name, args));
            }
        },
        "or" => {
            for _ in 0..scale {
                let args = (0..input_size)
                    .map(|_| format!("false"))
                    .collect::<Vec<String>>()
                    .join(" ");
                body.push_str(&*format!("({} {}) ", function_name, args));
            }
        },
        _ => {
            for _ in 0..scale {
                let args = (0..input_size)
                    .map(|_| {
                        format!("{}", bools[rng.gen_range(0..=1)])
                    })
                    .collect::<Vec<String>>()
                    .join(" ");
                body.push_str(&*format!("({} {}) ", function_name, args));
            }
        },
    } */

    body
}

fn gen_xor(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let args = match rng.gen_range(0..=1) {
            0 => {
                // uint
                let x: u128 = rng.gen();
                let y: u128 = rng.gen();
                format!("{} {}", x, y)
            }
            1 => {
                // uint
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
    body
}


fn helper_generate_rand_hex_string(n: usize) -> String {
    let hex_chars = ["a", "b", "c", "d", "e", "f", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

    let hex_range = Uniform::new_inclusive(0, 15);
    rand::thread_rng()
        .sample_iter(&hex_range)
        .take(n)
        .map(|x| hex_chars[x])
        .collect::<String>()
}

fn helper_generate_rand_char_string(n: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..n).map(|_| rng.gen_range(b'a'..b'z') as char).collect::<String>()
}

/// This function generates a single value that either has type uint, int, or buff (randomly chosen)
/// This value is set as the argument to a hash function ultimately
fn gen_hash(function_name: &'static str, scale: u16) -> String {
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
    body
}

fn gen_secp256k1(function_name: &'static str, scale: u16, verify: bool) -> String {
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
            format!("0x{} 0x{} 0x{}", to_hex(&msg), to_hex(&sig_bytes_vec), pubk.to_hex())
        } else {
            format!("0x{} 0x{}", to_hex(&msg), to_hex(&sig_bytes_vec))
        };

        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    body
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

/// todo: remove function name input for the generator functions that map to a single clarity fn?
fn gen_create_ft(_function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();

    for _ in 0..scale {
        let (statement, _) = helper_define_fungible_token_statement();
        body.push_str(&statement);
    }
    body
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

fn gen_ft_mint(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let (statement, token_name) = helper_define_fungible_token_statement();
    body.push_str(&statement);

    for _ in 0..scale {
        let amount: u128 = rng.gen_range(1..1000);
        let principal_data = helper_create_principal();
        let args = format!("{} u{} {}", token_name, amount, principal_data);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    println!("{}", body);
    body
}

fn helper_create_ft_boilerplate(mint_amount: u16) -> (String, String, String) {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let token_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    body.push_str(&*format!("(define-fungible-token {}) ", token_name));

    let principal_data = helper_create_principal();
    body.push_str(&*format!("(ft-mint? {} u{} {}) ", token_name, mint_amount, principal_data));
    (token_name, principal_data, body)
}

fn gen_ft_transfer(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let max_transfer = 100;
    let (token_name, sender_principal, template) = helper_create_ft_boilerplate(scale*max_transfer);
    body.push_str(&template);

    let recipient_principal = helper_create_principal();
    for _ in 0..scale {
        let transfer_amount = rng.gen_range(1..=max_transfer);
        let args = format!("{} u{} {} {}", token_name, transfer_amount, sender_principal, recipient_principal);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    println!("{}", body);
    body
}

fn gen_ft_balance(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    let (token_name, principal_data, template) = helper_create_ft_boilerplate(100);
    body.push_str(&template);
    let args = format!("{} {}", token_name, principal_data);
    for _ in 0..scale {
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    body
}

fn gen_ft_supply(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    let (token_name, _, template) = helper_create_ft_boilerplate(100);
    body.push_str(&template);
    let args = format!("{}", token_name);
    for _ in 0..scale {
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    body
}

fn gen_ft_burn(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let max_burn = 100;
    let (token_name, principal_data, template) = helper_create_ft_boilerplate(scale*max_burn);
    body.push_str(&template);
    for _ in 0..scale {
        let burn_amount = rng.gen_range(1..=max_burn);
        let args = format!("{} u{} {}", token_name, burn_amount, principal_data);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    body
}

/// ////////////////////////////////////////
/// NON FUNGIBLE TOKEN GENERATOR FUNCTIONS
/// ////////////////////////////////////////

fn helper_gen_clarity_type(allow_bool_type: bool, only_sequence_types: bool, only_non_seqence_types: bool) -> (String, Option<u16>) {
    let mut rng = rand::thread_rng();
    let type_no_len = ["int", "uint", "bool"];
    let type_with_len = ["buff", "string-ascii", "string-utf8"];

    let p = if only_sequence_types {0.0} else if only_non_seqence_types {1.0} else {0.5};
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
            length = if length % 2 == 0 {length} else { length + 1 };
            let nft_type = type_with_len[index];
            (nft_type, Some(length))
        }
    };
    (nft_type.to_string(), nft_len)
}

// Returns statement, token_name, the type of the nft, and option for the length of the nft if it is a string
fn helper_define_non_fungible_token_statement(allow_bool_type: bool) -> (String, String, String, Option<u16>) {
    let mut rng = rand::thread_rng();
    let token_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let (nft_type, nft_len) = helper_gen_clarity_type(allow_bool_type, false, false);
    let args = match nft_len {
        Some(length) => format!("{} ({} {})", token_name, nft_type, length),
        None => format!("{} {}", token_name, nft_type)
    };

    let statement = format!("(define-non-fungible-token {}) ", args);
    (statement, token_name, nft_type.to_string(), nft_len)
}

fn gen_create_nft(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    for _ in 0..scale {
        let (statement, _, _, _) = helper_define_non_fungible_token_statement(true);
        body.push_str(&statement);
    }
    body
}

fn helper_gen_clarity_value(value_type: &str, num: u16, value_len: usize, list_type: Option<&str>) -> String {
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
            // let (list_type, _) = helper_gen_clarity_type(true, false, true);
            let list_type = list_type.unwrap();
            let args = (0..value_len).map(|_| helper_gen_clarity_value(&list_type, num, 0, None)).collect::<Vec<String>>().join(" ");
            format!("(list {})", args)
        }
        _ => {
            unreachable!("should only be generating the types int, uint, buff, string-ascii, string-utf8, bool.")
        }
    }
}

fn helper_gen_random_clarity_value(num: u16) -> String {
    let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
    helper_gen_clarity_value(&clarity_type, num, length.map_or(0, |len| len as usize), None)
}

fn gen_nft_mint(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    let (statement, token_name, nft_type, nft_len) = helper_define_non_fungible_token_statement(false);
    body.push_str(&statement);

    let nft_len = nft_len.map_or(0, |len| len) as usize;
    for i in 0..scale {
        let principal_data = helper_create_principal();
        let nft_value = helper_gen_clarity_value(&nft_type, i, nft_len, None);

        let args = format!("{} {} {}", token_name, nft_value, principal_data);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    println!("{}", body);
    body
}

fn helper_create_nft_fn_boilerplate() -> (String, String, String, String, String) {
    let mut body = String::new();
    let (statement, token_name, nft_type, nft_len) = helper_define_non_fungible_token_statement(false);
    body.push_str(&statement);

    let nft_len = nft_len.map_or(0, |len| len) as usize;
    let nft_value = helper_gen_clarity_value(&nft_type, 0, nft_len, None);
    let invalid_nft_value = helper_gen_clarity_value(&nft_type, 0, nft_len, None);
    let mut owner_principal = helper_create_principal();
    let mint_statement = format!("(nft-mint? {} {} {}) ", token_name, nft_value, owner_principal);
    body.push_str(&mint_statement);
    (body, token_name, owner_principal, nft_value, invalid_nft_value)
}

fn gen_nft_transfer(function_name: &'static str, scale: u16) -> String {
    let (mut body, token_name, mut owner_principal, nft_value, _) = helper_create_nft_fn_boilerplate();
    for _ in 0..scale {
        let next_principal = helper_create_principal();
        let args = format!("{} {} {} {}", token_name, nft_value, owner_principal, next_principal);
        body.push_str(&*format!("({} {}) ", function_name, args));

        owner_principal = next_principal;
    }
    body
}

fn gen_nft_owner(function_name: &'static str, scale: u16) -> String {
    let mut rng = rand::thread_rng();
    let (mut body, token_name, _, nft_value, invalid_nft_value) = helper_create_nft_fn_boilerplate();
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
    body
}

fn gen_nft_burn(function_name: &'static str, scale: u16) -> String {
    let (mut body, token_name, mut owner_principal, nft_value, _) = helper_create_nft_fn_boilerplate();
    for _ in 0..scale {
        let args = format!("{} {} {}", token_name, nft_value, owner_principal);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    body
}

/// ////////////////////////////////////////
/// TUPLE GENERATOR FUNCTIONS
/// ////////////////////////////////////////

fn gen_tuple_get(scale: u16, input_size: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    let tuple_vals = (0..input_size)
        .map(|i| format!("(id{} 1337)", i))
        .collect::<Vec<String>>()
        .join(" ");

    let tuple = format!("(tuple {})", tuple_vals);

    for _ in 0..scale {
        body.push_str(&*format!(
            "(get id{} test-tuple) ",
            rng.gen_range(0..input_size)
        ));
    }

    format!("(let ((test-tuple {})) {})", tuple, body)
}

fn gen_tuple_merge(scale: u16, input_size: u16) -> String {
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
        body.push_str(&*format!("(merge tuple-a tuple-b)"));
    }

    format!("(let ((tuple-a {}) (tuple-b {})) {})", tuple_a, tuple_b, body)

}

fn gen_tuple_cons(scale: u16, input_size: u16) -> String {
    let mut body = String::new();

    let tuple_vals = (0..input_size)
        .map(|i| format!("(id{} 1337)", i))
        .collect::<Vec<String>>()
        .join(" ");

    let tuple = format!("(tuple {})", tuple_vals);

    for _ in 0..scale {
        body.push_str(&tuple);
    }

    body
}

/// ////////////////////////////////////////
/// OPTIONAL/ RESPONSE GENERATOR FUNCTIONS
/// ////////////////////////////////////////

fn helper_gen_random_optional_value(num: u16, only_some: bool) -> String {
    let mut rng = rand::thread_rng();
    let p = if only_some {0.0} else {0.5};
    match rng.gen_bool(p) {
        true => {
            "none".to_string()
        }
        false => {
            let clarity_val = helper_gen_random_clarity_value(num);
            format!("(some {})", clarity_val)
        }
    }
}

fn gen_optional(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    for i in 0..scale {
        let args = helper_gen_random_optional_value(i, false);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    body
}

fn helper_gen_random_response_value(num: u16, only_ok: bool, only_err: bool) -> String {
    let mut rng = rand::thread_rng();
    let clarity_val = helper_gen_random_clarity_value(num);
    let p = if only_ok {0.0} else if only_err {1.0} else {0.5};
    match rng.gen_bool(p) {
        true => {
            format!("(err {})", clarity_val)
        }
        false => {

            format!("(ok {})", clarity_val)
        }
    }
}

fn gen_response(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    for i in 0..scale {
        let args = helper_gen_random_response_value(i, false, false);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    body
}

fn gen_unwrap(function_name: &'static str, scale: u16, ret_value: bool) -> String {
    let mut rng = rand::thread_rng();
    let mut body = String::new();
    for i in 0..scale {
        let mut args = match rng.gen_bool(0.5) {
            true => helper_gen_random_response_value(i, true, false),
            false => helper_gen_random_optional_value(i, true)
        };
        if ret_value {
            let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
            let clarity_val = helper_gen_clarity_value(&clarity_type, i, length.map_or(0, |len| len as usize), None);
            args = format!("{} {}", args, clarity_val)
        }
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    println!("{}", body);
    body
}

fn gen_unwrap_err(function_name: &'static str, scale: u16, ret_value: bool) -> String {
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
    body
}

fn helper_create_map() -> (String, String, String, String, Option<u16>, String, String, Option<u16>) {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    let map_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let key_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let (key_type, key_type_len) = helper_gen_clarity_type(true, false, false);
    let key_args = match key_type_len {
        Some(length) => format!("{{ {}: ({} {}) }}", key_name, key_type, length),
        None => format!("{{ {}: {} }}", key_name, key_type)
    };

    let value_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let (value_type, value_type_len) = helper_gen_clarity_type(true, false, false);
    let value_args = match value_type_len {
        Some(length) => format!("{{ {}: ({} {}) }}", value_name, value_type, length),
        None => format!("{{ {}: {} }}", value_name, value_type)
    };
    body.push_str(&*format!("(define-map {} {} {}) ", map_name, key_args, value_args));
    (body, map_name, key_name, key_type, key_type_len, value_name, value_type, value_type_len)
}

fn gen_create_map(_function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    for _ in 0..scale {
        let (statement, _, _, _, _, _, _, _) = helper_create_map();
        body.push_str(&statement);
    }
    println!("{}", body);
    body
}

// setEntry is the cost for map-delete, map-insert, & map-set
fn gen_set_entry(scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let (statement, map_name, key_name, key_type, key_type_len, value_name, value_type, value_type_len) = helper_create_map();
    body.push_str(&statement);
    for i in 0..scale {
        let curr_key = helper_gen_clarity_value(&key_type, i, key_type_len.map_or(0, |len| len as usize), None);
        let curr_value = helper_gen_clarity_value(&value_type, i, value_type_len.map_or(0, |len| len as usize), None);
        let statement = match rng.gen_range(0..3) {
            0 => {
                format!("(map-set {} {{ {}: {} }} {{ {}: {} }}) ", map_name, key_name, curr_key, value_name, curr_value)
            }
            1 => {
                format!("(map-insert {} {{ {}: {} }} {{ {}: {} }}) ", map_name, key_name, curr_key, value_name, curr_value)
            }
            2 => {
                format!("(map-delete {} {{ {}: {} }}) ", map_name, key_name, curr_key)
            }
            _ => unreachable!("should only gen numbers from 0 to 2 inclusive")
        };
        body.push_str(&statement);
    }
    println!("{}", body);
    body
}

// todo: might not be worst case because keys are already in the map
fn gen_fetch_entry(scale: u16) -> String {
    let mut body = String::new();
    let (statement, map_name, key_name, key_type, key_type_len, value_name, value_type, value_type_len) = helper_create_map();
    body.push_str(&statement);
    for i in 0..scale {
        let new_key = helper_gen_clarity_value(&key_type, i, key_type_len.map_or(0, |len| len as usize), None);
        let statement = format!("(map-delete {} {{ {}: {} }}) ", map_name, key_name, new_key);
        body.push_str(&statement);
    }
    println!("{}", body);
    body
}

fn gen_create_var(function_name: &'static str, scale: u16) -> String  {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for i in 0..scale {
        let var_name = helper_generate_rand_char_string(rng.gen_range(10..20));
        let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
        let clarity_value = helper_gen_clarity_value(&clarity_type, i, length.map_or(0, |len| len as usize), None);
        let args = match length {
            Some(l) => format!("{} ({} {}) {}", var_name, clarity_type, l, clarity_value),
            None => format!("{} {} {}", var_name, clarity_type, clarity_value),
        };
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    println!("{}", body);
    body
}

fn gen_var_set_get(function_name:  &'static str, scale: u16, set: bool) -> String  {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    let var_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
    let clarity_value = helper_gen_clarity_value(&clarity_type, rng.gen_range(10..200), length.map_or(0, |len| len as usize), None);
    let args = match length {
        Some(l) => format!("{} ({} {}) {}", var_name, clarity_type, l, clarity_value),
        None => format!("{} {} {}", var_name, clarity_type, clarity_value),
    };
    body.push_str(&*format!("({} {}) ", "define-data-var", args));
    for i in 0..scale {
        let args = if set {
            let new_val = helper_gen_clarity_value(&clarity_type, i, length.map_or(0, |len| len as usize), None);
            format!("{} {}", var_name, new_val)
        } else {
            format!("{}", var_name)
        };
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    println!("{}", body);
    body
}

fn gen_single_clar_value(function_name:  &'static str, scale: u16) -> String {
    let mut body = String::new();
    for i in 0..scale {
        let args = helper_gen_random_clarity_value(i);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    println!("{}", body);
    body
}

fn gen_if(function_name:  &'static str, scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for i in 0..scale {
        let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
        let if_case_value = helper_gen_clarity_value(&clarity_type, i, length.map_or(0, |len| len as usize), None);
        let else_case_value = helper_gen_clarity_value(&clarity_type, i, length.map_or(0, |len| len as usize), None);
        let curr_bool = rng.gen_bool(0.5);

        body.push_str(&*format!("({} {} {} {}) ", function_name, curr_bool, if_case_value, else_case_value));
    }
    body
}

fn gen_asserts(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    for i in 0..scale {
        let clarity_val = helper_gen_random_clarity_value(i);
        body.push_str(&*format!("({} true {}) ", function_name, clarity_val));
    }
    println!("{}", body);
    body
}

fn helper_generate_sequences(list_type: &str, output: u16) -> Vec<String> {
    let mut rng = rand::thread_rng();
    match rng.gen_bool(0.75) {
        true => {
            // non-list case
            let (clarity_type, _) = helper_gen_clarity_type(true, true, false);
            (0..output).map(|_| helper_gen_clarity_value(&clarity_type, rng.gen_range(2..50), rng.gen_range(2..50)*2, None)).collect()
        }
        false => {
            // list case
            (0..output).map(|_| helper_gen_clarity_value("list", rng.gen_range(2..50), rng.gen_range(2..50)*2, Some(list_type))).collect()
        }
    }
}

fn gen_concat(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    for _ in 0..scale {
        let (list_type, _) = helper_gen_clarity_type(true, false, true);
        let operands = helper_generate_sequences(&list_type, 2);
        body.push_str(&*format!("({} {} {}) ", function_name, operands[0], operands[1]));
    }
    println!("{}", body);
    body
}

fn gen_as_max_len(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let (list_type, _) = helper_gen_clarity_type(true, false, true);
        let operand = helper_generate_sequences(&list_type, 1);
        let len = helper_gen_clarity_value("uint", rng.gen_range(2..50), 0, None);
        body.push_str(&*format!("({} {} {}) ", function_name, operand[0], len));
    }
    println!("{}", body);
    body
}

// todo: This is to bench BindName - this cost is also used in define function, so should take worst case of both
fn gen_define_constant(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for i in 0..scale {
        let name = helper_generate_rand_char_string(rng.gen_range(10..50));
        let value = helper_gen_random_clarity_value(i);
        body.push_str(&*format!("({} {} {}) ", function_name, name, value));
    }
    println!("{}", body);
    body
}

fn gen_default_to(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for i in 0..scale {
        let (clarity_type, length) = helper_gen_clarity_type(true, false, false);
        let default_val = helper_gen_clarity_value(&clarity_type, i, length.map_or(0, |len| len as usize), None);
        let opt_string = match rng.gen_bool(0.5) {
            true => "none".to_string(),
            false => {
                let inner_val = helper_gen_clarity_value(&clarity_type, i, length.map_or(0, |len| len as usize), None);
                format!("(some {})", inner_val)
            }
        };
        body.push_str(&*format!("({} {} {}) ", function_name, default_val, opt_string));
    }
    println!("{}", body);
    body
}

fn gen_int_cast(scale: u16) -> String {
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
    body
}

// todo - bench (print 0) and subtract from cost
fn gen_match(scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for i in 0..scale {
        let dummy_statement = "(print 0) ";
        let first_branch_name = helper_generate_rand_char_string(rng.gen_range(10..20));

        let statement = match rng.gen_bool(0.5) {
            true => {
                let match_val = helper_gen_random_response_value(i, false, false);
                let second_branch_name = helper_generate_rand_char_string(rng.gen_range(10..20));
                format!("(match {} {} {} {} {}) ", match_val, first_branch_name, dummy_statement, second_branch_name, dummy_statement)
            }
            false => {
                let match_val = helper_gen_random_optional_value(i, false);
                format!("(match {} {} {} {}) ", match_val, first_branch_name, dummy_statement, dummy_statement)
            }
        };
        body.push_str(&statement);
    }
    println!("{}", body);
    body
}

// todo - bench (print x) and subtract from cost
fn gen_let(scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for i in 0..scale {
        let var_name = helper_generate_rand_char_string(rng.gen_range(10..20));
        let var_value = helper_gen_random_clarity_value(i);
        let statement = format!("(let (({} {})) (print {})) ", var_name, var_value, var_name);
        body.push_str(&statement);
    }
    println!("{}", body);
    body
}

fn helper_generate_random_sequence() -> (String, usize, String) {
    let mut rng = rand::thread_rng();
    let value_len = rng.gen_range(2..50)*2;
    match rng.gen_bool(0.75) {
        true => {
            // non-list case
            let (clarity_type, _) = helper_gen_clarity_type(true, true, false);
            let value = helper_gen_clarity_value(&clarity_type, rng.gen_range(2..50), value_len, None);
            (value, value_len, clarity_type)
        }
        false => {
            // list case
            let (list_type, _) = helper_gen_clarity_type(true, false, true);
            let value = helper_gen_clarity_value("list", rng.gen_range(2..50), value_len, Some(&list_type));
            (value, value_len, list_type)
        }
    }
}

fn gen_index_of(scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let (seq, _, seq_inner_type) = helper_generate_random_sequence();
        let item_len = if seq_inner_type == "buff" { 2 } else { 1 };
        let item_val = helper_gen_clarity_value(&seq_inner_type, rng.gen_range(2..50), item_len, None);
        let statement = format!("(index-of {} {}) ", seq, item_val);
        body.push_str(&statement);
    }
    println!("{}", body);
    body
}

fn gen_element_at(scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let (seq, seq_len, _) = helper_generate_random_sequence();
        let index_to_query = rng.gen_range(0..seq_len*2);
        let statement = format!("(element-at {} u{}) ", seq, index_to_query);
        body.push_str(&statement);
    }
    println!("{}", body);
    body
}

fn gen_len(scale: u16) -> String {
    let mut body = String::new();
    for _ in 0..scale {
        let (seq, _, _) = helper_generate_random_sequence();
        let statement = format!("(len {}) ", seq);
        body.push_str(&statement);
    }
    println!("{}", body);
    body
}

// q: not sure if we are testing worst case here; not allowing list of buffs, for example
fn gen_append(scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let (list_type, _) = helper_gen_clarity_type(true, false, true);
        let list_val = helper_gen_clarity_value("list", rng.gen_range(2..50), rng.gen_range(2..50) * 2, Some(&list_type));
        let new_item_val = helper_gen_clarity_value(&list_type, rng.gen_range(2..50), 1, None);
        let statement = format!("(append {} {}) ", list_val, new_item_val);
        body.push_str(&statement);

    }
    println!("{}", body);
    body
}

fn gen_list_cons(scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..scale {
        let (list_type, _) = helper_gen_clarity_type(true, false, true);
        let list_len = rng.gen_range(1..10);
        // let mut args = "(".to_string();
        let mut args = String::new();
        for _ in 0..list_len {
            let new_item_val = helper_gen_clarity_value(&list_type, rng.gen_range(2..50), 1, None);
            args.push_str(&*format!("{} ", new_item_val));
        }
        // args.push_str(")");
        let statement = format!("(list {}) ", args);
        body.push_str(&statement);

    }
    println!("{}", body);
    body
}

// todo - benchmark cost of (begin true) and subtract
fn gen_filter(scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    body.push_str("(define-read-only (dummy_int (input int)) (begin true)) ");
    body.push_str("(define-read-only (dummy_uint (input uint)) (begin true)) ");
    body.push_str("(define-read-only (dummy_bool (input bool)) (begin true)) ");
    for _ in 0..scale {
        let (list_type, _) = helper_gen_clarity_type(true, false, true);
        let list_val = helper_gen_clarity_value("list", rng.gen_range(2..50), rng.gen_range(2..50) * 2, Some(&list_type));
        let statement = format!("(filter dummy_{} {}) ", list_type, list_val);
        body.push_str(&statement);
    }
    println!("{}", body);
    body
}

// fixed type of B to be bool
// todo - benchmark cost of (begin true) and subtract
fn gen_fold(scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    body.push_str("(define-read-only (dummy_int (input int) (input_two bool)) (begin true)) ");
    body.push_str("(define-read-only (dummy_uint (input uint) (input_two bool)) (begin true)) ");
    body.push_str("(define-read-only (dummy_bool (input bool) (input_two bool)) (begin true)) ");
    for _ in 0..scale {
        let (list_type, _) = helper_gen_clarity_type(true, false, true);
        let list_val = helper_gen_clarity_value("list", rng.gen_range(2..50), rng.gen_range(2..50) * 2, Some(&list_type));
        let statement = format!("(fold dummy_{} {} true) ", list_type, list_val);
        body.push_str(&statement);
    }
    println!("{}", body);
    body
}

pub fn gen(function: ClarityCostFunction, scale: u16, input_size: u16) -> String {
    match function {
        // arithmetic
        ClarityCostFunction::Add => gen_arithmetic("+", scale, input_size),
        ClarityCostFunction::Sub =>  gen_arithmetic("-", scale, input_size),
        ClarityCostFunction::Mul => gen_arithmetic("*", scale, input_size),
        ClarityCostFunction::Div => gen_arithmetic("/", scale, input_size),
        ClarityCostFunction::Sqrti => gen_arithmetic("sqrti", scale, 1),
        ClarityCostFunction::Log2 => gen_arithmetic("log2", scale, 1),
        ClarityCostFunction::Mod => gen_arithmetic("mod", scale, input_size),
        ClarityCostFunction::Pow => gen_pow(scale),
        // logic
        ClarityCostFunction::Le => gen_cmp("<", scale),
        ClarityCostFunction::Leq => gen_cmp("<=", scale),
        ClarityCostFunction::Ge => gen_cmp(">", scale),
        ClarityCostFunction::Geq => gen_cmp(">=", scale),
        // boolean
        ClarityCostFunction::And => gen_logic("and", scale, input_size),
        ClarityCostFunction::Or => gen_logic("or", scale, input_size),
        ClarityCostFunction::Xor => gen_xor("xor", scale),
        ClarityCostFunction::Not => gen_logic("not", scale, input_size),
        ClarityCostFunction::Eq => gen_logic("is-eq", scale, input_size),
        // tuples
        ClarityCostFunction::TupleGet => gen_tuple_get(scale, input_size),
        ClarityCostFunction::TupleMerge => gen_tuple_merge(scale, input_size),
        ClarityCostFunction::TupleCons => gen_tuple_cons(scale, input_size),
        // Analysis
        ClarityCostFunction::AnalysisTypeAnnotate => unimplemented!(),
        ClarityCostFunction::AnalysisTypeCheck => unimplemented!(),
        ClarityCostFunction::AnalysisTypeLookup => unimplemented!(),
        ClarityCostFunction::AnalysisVisit => unimplemented!(),
        ClarityCostFunction::AnalysisIterableFunc => unimplemented!(),
        ClarityCostFunction::AnalysisOptionCons => unimplemented!(),
        ClarityCostFunction::AnalysisOptionCheck => unimplemented!(),
        ClarityCostFunction::AnalysisBindName => unimplemented!(),
        ClarityCostFunction::AnalysisListItemsCheck => unimplemented!(),
        ClarityCostFunction::AnalysisCheckTupleGet => unimplemented!(),
        ClarityCostFunction::AnalysisCheckTupleMerge => unimplemented!(),
        ClarityCostFunction::AnalysisCheckTupleCons => unimplemented!(),
        ClarityCostFunction::AnalysisTupleItemsCheck => unimplemented!(),
        ClarityCostFunction::AnalysisCheckLet => unimplemented!(),
        ClarityCostFunction::AnalysisLookupFunction => unimplemented!(),
        ClarityCostFunction::AnalysisLookupFunctionTypes => unimplemented!(),
        ClarityCostFunction::AnalysisLookupVariableConst => unimplemented!(),
        ClarityCostFunction::AnalysisLookupVariableDepth => unimplemented!(),
        ClarityCostFunction::AnalysisStorage => unimplemented!(),
        ClarityCostFunction::AnalysisUseTraitEntry => unimplemented!(),
        ClarityCostFunction::AnalysisGetFunctionEntry => unimplemented!(),
        ClarityCostFunction::AnalysisFetchContractEntry => unimplemented!(),
        // Ast
        ClarityCostFunction::AstParse => unimplemented!(),
        ClarityCostFunction::AstCycleDetection => unimplemented!(),
        // Lookup
        ClarityCostFunction::LookupVariableDepth => unimplemented!(),
        ClarityCostFunction::LookupVariableSize => unimplemented!(),
        ClarityCostFunction::LookupFunction => unimplemented!(),
        // List
        ClarityCostFunction::Map => unimplemented!(),
        ClarityCostFunction::Filter => gen_filter(scale),
        ClarityCostFunction::Fold => gen_fold(scale),
        ClarityCostFunction::Len => gen_len(scale),
        ClarityCostFunction::ElementAt => gen_element_at(scale),
        ClarityCostFunction::IndexOf => gen_index_of(scale),
        ClarityCostFunction::ListCons => gen_list_cons(scale),
        ClarityCostFunction::Append => gen_append(scale),
        // Hash
        ClarityCostFunction::Hash160 => gen_hash("hash160", scale),
        ClarityCostFunction::Sha256 => gen_hash("sha256", scale),
        ClarityCostFunction::Sha512 => gen_hash("sha512", scale),
        ClarityCostFunction::Sha512t256 => gen_hash("sha512/256", scale),
        ClarityCostFunction::Keccak256 => gen_hash("keccak256", scale),
        ClarityCostFunction::Secp256k1recover => gen_secp256k1("secp256k1-recover?", scale, false),
        ClarityCostFunction::Secp256k1verify => gen_secp256k1("secp256k1-verify", scale, true),
        // FT
        ClarityCostFunction::CreateFt => gen_create_ft("define-fungible-token", scale),
        ClarityCostFunction::FtMint => gen_ft_mint("ft-mint?", scale),
        ClarityCostFunction::FtTransfer => gen_ft_transfer("ft-transfer?", scale),
        ClarityCostFunction::FtBalance => gen_ft_balance("ft-get-balance", scale),
        ClarityCostFunction::FtSupply => gen_ft_supply("ft-get-supply", scale),
        ClarityCostFunction::FtBurn => gen_ft_burn("ft-burn?", scale),
        // NFT
        ClarityCostFunction::CreateNft => gen_create_nft("define-non-fungible-token", scale),
        ClarityCostFunction::NftMint => gen_nft_mint("nft-mint?", scale),
        ClarityCostFunction::NftTransfer =>  gen_nft_transfer("nft-transfer?", scale),
        ClarityCostFunction::NftOwner =>  gen_nft_owner("nft-get-owner?", scale),
        ClarityCostFunction::NftBurn =>  gen_nft_burn("nft-burn?", scale),
        // Stacks
        ClarityCostFunction::PoisonMicroblock => unimplemented!(),
        ClarityCostFunction::BlockInfo => unimplemented!(),
        ClarityCostFunction::StxBalance => unimplemented!(),
        ClarityCostFunction::StxTransfer => unimplemented!(),
        // Option & result checks
        ClarityCostFunction::IsSome => gen_optional("is-some", scale),
        ClarityCostFunction::IsNone => gen_optional("is-none", scale),
        ClarityCostFunction::IsOkay => gen_response("is-ok", scale),
        ClarityCostFunction::IsErr => gen_response("is-err", scale),
        ClarityCostFunction::DefaultTo => gen_default_to("default-to", scale),
        // Unwrap functions
        ClarityCostFunction::Unwrap => gen_unwrap("unwrap-panic", scale, false),
        ClarityCostFunction::UnwrapRet => gen_unwrap("unwrap!", scale, true),
        ClarityCostFunction::UnwrapErr => gen_unwrap_err("unwrap-err-panic", scale, false),
        ClarityCostFunction::UnwrapErrOrRet => gen_unwrap_err("unwrap-err!", scale, true),
        ClarityCostFunction::TryRet => gen_unwrap("try!", scale, false),
        // Map
        ClarityCostFunction::CreateMap => gen_create_map("define-map", scale),
        ClarityCostFunction::FetchEntry => gen_fetch_entry(scale),  // map-get?
        ClarityCostFunction::SetEntry => gen_set_entry(scale),  // map-set
        // Var
        ClarityCostFunction::CreateVar => gen_create_var("define-data-var", scale),
        ClarityCostFunction::FetchVar => gen_var_set_get("var-get", scale, false),
        ClarityCostFunction::SetVar => gen_var_set_get("var-set", scale, true),
        ClarityCostFunction::BindName => gen_define_constant("define-constant", scale), // used for define var and define function
        // Functions with single clarity value input
        ClarityCostFunction::Print => gen_single_clar_value("print", scale),
        ClarityCostFunction::SomeCons => gen_single_clar_value("some", scale),
        ClarityCostFunction::OkCons => gen_single_clar_value("ok", scale),
        ClarityCostFunction::ErrCons => gen_single_clar_value("err", scale),
        ClarityCostFunction::Begin => gen_single_clar_value("begin", scale),
        // If
        ClarityCostFunction::If => gen_if("if", scale),
        // Asserts
        ClarityCostFunction::Asserts => gen_asserts("asserts!", scale),
        // Concat
        ClarityCostFunction::Concat => gen_concat("concat", scale),
        // Sequence
        ClarityCostFunction::AsMaxLen => gen_as_max_len("as-max-len?", scale),
        // Int
        ClarityCostFunction::IntCast => gen_int_cast(scale),
        // Let
        ClarityCostFunction::Let => gen_let(scale),
        // Match
        ClarityCostFunction::Match => gen_match(scale),
        // Uncategorized
        ClarityCostFunction::InnerTypeCheckCost => unimplemented!(),
        ClarityCostFunction::UserFunctionApplication => unimplemented!(),
        ClarityCostFunction::TypeParseStep => unimplemented!(), // called by `parse_type_repr` in `signatures.rs` (takes in symbolic expression)
        ClarityCostFunction::ContractCall => unimplemented!(),
        ClarityCostFunction::ContractOf => unimplemented!(),
        ClarityCostFunction::PrincipalOf => unimplemented!(),
        ClarityCostFunction::AtBlock => unimplemented!(),
        ClarityCostFunction::LoadContract => unimplemented!(), // called at start of execute_contract
        ClarityCostFunction::ContractStorage => unimplemented!(), // start of `initialize_contract_from_ast`
    }
}
