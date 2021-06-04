use blockstack_lib::vm::costs::cost_functions::ClarityCostFunction;
use rand::Rng;

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
        body.push_str(&*format!("({} {}) ", function_name, args));
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

fn gen_logic(function_name: &'static str, scale: u16) -> String {
    let mut body = String::new();
    for _ in 0..scale {
        body.push_str(&*format!("({} true false) ", function_name));
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
        ClarityCostFunction::And => gen_logic("and", scale),
        ClarityCostFunction::Or => gen_logic("or", scale),
        ClarityCostFunction::Xor => gen_logic("xor", scale),
        ClarityCostFunction::Not => gen_logic("not", scale),
        ClarityCostFunction::Eq => gen_logic("eq", scale),
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
        ClarityCostFunction::Hash160 => unimplemented!(),
        ClarityCostFunction::Sha256 => unimplemented!(),
        ClarityCostFunction::Sha512 => unimplemented!(),
        ClarityCostFunction::Sha512t256 => unimplemented!(),
        ClarityCostFunction::Keccak256 => unimplemented!(),
        ClarityCostFunction::Secp256k1recover => unimplemented!(),
        ClarityCostFunction::Secp256k1verify => unimplemented!(),
        // FT
        ClarityCostFunction::CreateFt => unimplemented!(),
        ClarityCostFunction::FtMint => unimplemented!(),
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
    }
}
