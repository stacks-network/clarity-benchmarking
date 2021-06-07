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
use blockstack_lib::vm::analysis::contract_interface_builder::ContractInterfaceAtomType::principal;
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

/// ////////////////////////////////////
/// FUNGIBLE TOKEN GENERATOR FUNCTIONS
/// ////////////////////////////////////

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
        let amount: u128 = rng.gen();
        let principal_data = helper_create_principal();
        let args = format!("{} u{} {}", token_name, amount, principal_data);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
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

// Returns statement, token_name, the type of the nft, and option for the length of the nft if it is a string
fn helper_define_non_fungible_token_statement(allow_bool_type: bool) -> (String, String, String, Option<u16>) {
    let mut rng = rand::thread_rng();
    let type_no_len = ["int", "uint", "bool"];
    let type_with_len = ["buff", "string-ascii", "string-utf8"];

    let token_name = helper_generate_rand_char_string(rng.gen_range(10..20));
    let (args, nft_type, nft_len) = match rng.gen_range(0..=1) {
        0 => {
            // a type with no length arg
            let max_range = type_no_len.len() - (if allow_bool_type { 0 } else { 1 });
            let index = rng.gen_range(0..max_range);
            let nft_type = type_no_len[index];
            (format!("{} {}", token_name, nft_type), nft_type, None)
        }
        1 => {
            // a type with a length arg
            let index = rng.gen_range(0..type_with_len.len());
            let length = rng.gen_range(2..=50);
            let nft_type = type_with_len[index];
            (format!("{} ({} {})", token_name, nft_type, length), nft_type, Some(length))
        }
        _ => {
            unreachable!("should only be generating numbers in the range 0..=1.")
        }
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

fn helper_gen_nft_value(nft_type: &str, num: u16, nft_len: usize) -> String {
    match nft_type {
        "int" => format!("{}", num),
        "uint" => format!("u{}", num),
        "buff" => {
            let mut buff = "0x".to_string();
            buff.push_str(&helper_generate_rand_hex_string(nft_len));
            buff
        }
        "string-ascii" => {
            let ascii_string = helper_generate_rand_hex_string(nft_len);
            format!(r##""{}""##, ascii_string)
        }
        "string-utf8" => {
            let utf8_string = helper_generate_rand_hex_string(nft_len);
            format!(r##"u"{}""##, utf8_string)
        }
        _ => {
            unreachable!("should only be generating the types int, uint, buff, string-ascii, and string-utf8.")
        }
    }
}

fn gen_nft_mint(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    let (statement, token_name, nft_type, nft_len) = helper_define_non_fungible_token_statement(false);
    body.push_str(&statement);

    let nft_len = nft_len.map_or(0, |len| len) as usize;
    for i in 0..scale {
        let principal_data = helper_create_principal();
        let nft_value = helper_gen_nft_value(&nft_type, i, nft_len);

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
    let nft_value = helper_gen_nft_value(&nft_type, 0, nft_len);
    let invalid_nft_value = helper_gen_nft_value(&nft_type, 0, nft_len);
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
        ClarityCostFunction::Filter => unimplemented!(),
        ClarityCostFunction::Len => unimplemented!(),
        ClarityCostFunction::ElementAt => unimplemented!(),
        ClarityCostFunction::IndexOf => unimplemented!(),
        ClarityCostFunction::Fold => unimplemented!(),
        ClarityCostFunction::ListCons => unimplemented!(),
        ClarityCostFunction::Append => unimplemented!(),
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
        ClarityCostFunction::IsOkay => unimplemented!(),
        ClarityCostFunction::IsNone => unimplemented!(),
        ClarityCostFunction::IsErr => unimplemented!(),
        ClarityCostFunction::IsSome => unimplemented!(),
        // Uncategorized
        ClarityCostFunction::BindName => unimplemented!(),
        ClarityCostFunction::InnerTypeCheckCost => unimplemented!(),
        ClarityCostFunction::UserFunctionApplication => unimplemented!(),
        ClarityCostFunction::Let => unimplemented!(),
        ClarityCostFunction::If => unimplemented!(),
        ClarityCostFunction::Asserts => unimplemented!(),
        ClarityCostFunction::Begin => unimplemented!(),
        ClarityCostFunction::Print => unimplemented!(),
        ClarityCostFunction::TypeParseStep => unimplemented!(),
        ClarityCostFunction::IntCast => unimplemented!(),
        ClarityCostFunction::SomeCons => unimplemented!(),
        ClarityCostFunction::OkCons => unimplemented!(),
        ClarityCostFunction::ErrCons => unimplemented!(),
        ClarityCostFunction::DefaultTo => unimplemented!(),
        ClarityCostFunction::UnwrapRet => unimplemented!(),
        ClarityCostFunction::UnwrapErrOrRet => unimplemented!(),
        ClarityCostFunction::Unwrap => unimplemented!(),
        ClarityCostFunction::UnwrapErr => unimplemented!(),
        ClarityCostFunction::TryRet => unimplemented!(),
        ClarityCostFunction::Concat => unimplemented!(),
        ClarityCostFunction::AsMaxLen => unimplemented!(),
        ClarityCostFunction::ContractCall => unimplemented!(),
        ClarityCostFunction::ContractOf => unimplemented!(),
        ClarityCostFunction::PrincipalOf => unimplemented!(),
        ClarityCostFunction::AtBlock => unimplemented!(),
        ClarityCostFunction::LoadContract => unimplemented!(),
        ClarityCostFunction::CreateMap => unimplemented!(),
        ClarityCostFunction::CreateVar => unimplemented!(),
        ClarityCostFunction::FetchEntry => unimplemented!(),
        ClarityCostFunction::SetEntry => unimplemented!(),
        ClarityCostFunction::FetchVar => unimplemented!(),
        ClarityCostFunction::SetVar => unimplemented!(),
        ClarityCostFunction::ContractStorage => unimplemented!(),
        ClarityCostFunction::Match => unimplemented!(),
    }
}
