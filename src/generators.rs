use blockstack_lib::vm::costs::cost_functions::ClarityCostFunction;
use rand::{Rng, RngCore};
use rand::distributions::Uniform;
use blockstack_lib::util::secp256k1::{Secp256k1PrivateKey, Secp256k1PublicKey};
use blockstack_lib::burnchains::PrivateKey;

use secp256k1::Message as LibSecp256k1Message;
use blockstack_lib::util::hash::to_hex;
use blockstack_lib::chainstate::stacks::{StacksAddress, C32_ADDRESS_VERSION_TESTNET_SINGLESIG, StacksPublicKey};
use blockstack_lib::address::AddressHashMode;

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

    dbg!(body)
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
                format!("{}", bools[rng.gen_range(0..=1)])
            })
            .collect::<Vec<String>>()
            .join(" ");
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
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

fn gen_create_ft(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..scale {
        let token_name = helper_generate_rand_hex_string(rng.gen_range(10..20));
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

        body.push_str(&*format!("({} {}) ", function_name, args));
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
    let principal = addr.to_account_principal();

    format!("'{}", principal)
}

fn gen_ft_mint(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let token_name = helper_generate_rand_hex_string(rng.gen_range(10..20));
    body.push_str(&*format!("(define-fungible-token {}) ", token_name));

    for _ in 0..scale {
        let amount: u128 = rng.gen();
        let principal = helper_create_principal();
        let args = format!("{} u{} {}", token_name, amount, principal);
        body.push_str(&*format!("({} {}) ", function_name, args));
    }
    body
}

fn helper_create_ft_boilerplate(scale: u16) -> String {
    let mut body = String::new();
    let mut rng = rand::thread_rng();
    let token_name = helper_generate_rand_hex_string(rng.gen_range(10..20));
    body.push_str(&*format!("(define-fungible-token {}) ", token_name));

    let mint_amount = 100 * (scale as u32);
    let principal = helper_create_principal();
    body.push_str(&*format!("(ft-mint? {} u{} {}) ", token_name, mint_amount, principal));
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

// fn gen_tuple_merge(scale: u16, input_size: u16) -> String {
//     let mut body = String::new();

//     format!("()")
// }

pub fn gen(function: ClarityCostFunction, scale: u16, input_size: u16) -> String {
    let body = match function {
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
        ClarityCostFunction::TupleMerge => unimplemented!(),
        ClarityCostFunction::TupleCons => unimplemented!(),
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
        ClarityCostFunction::FtTransfer => unimplemented!(),
        ClarityCostFunction::FtBalance => unimplemented!(),
        ClarityCostFunction::FtSupply => unimplemented!(),
        ClarityCostFunction::FtBurn => unimplemented!(),
        // NFT
        ClarityCostFunction::CreateNft => unimplemented!(),
        ClarityCostFunction::NftMint => unimplemented!(),
        ClarityCostFunction::NftTransfer => unimplemented!(),
        ClarityCostFunction::NftOwner => unimplemented!(),
        ClarityCostFunction::NftBurn => unimplemented!(),
        // Stacks
        ClarityCostFunction::PoisonMicroblock => unimplemented!(),
        ClarityCostFunction::BlockInfo => unimplemented!(),
        ClarityCostFunction::StxBalance => unimplemented!(),
        ClarityCostFunction::StxTransfer => unimplemented!(),
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
        ClarityCostFunction::IsOkay => unimplemented!(),
        ClarityCostFunction::IsNone => unimplemented!(),
        ClarityCostFunction::IsErr => unimplemented!(),
        ClarityCostFunction::IsSome => unimplemented!(),
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
    };

    format!("(define-public (test) (ok {}))", body)
}
