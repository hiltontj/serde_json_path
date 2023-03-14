use proc_macro2::{Ident, TokenStream};
use syn::{punctuated::Punctuated, token::Comma, FnArg, ReturnType, Signature, Type};

pub struct FunctionDefinition {
    name: Ident,
    result: TokenStream,
    inputs: Punctuated<FnArg, Comma>,
}

pub fn validate_signature(input: Signature) -> Result<FunctionDefinition, ValidationError> {
    let Signature {
        ident: name,
        inputs,
        output,
        // TODO - do we reject for any of the below? or just ignore...
        constness: _,
        asyncness: _,
        unsafety: _,
        abi: _,
        fn_token: _,
        generics: _,
        paren_token: _,
        variadic: _,
    } = input;

    let result = match output {
        ReturnType::Default => return Err(ValidationError::MissingOutput),
        ReturnType::Type(_, t) => {
            if let Type::Verbatim(ts) = *t {
                ts
            } else {
                return Err(ValidationError::VerbatimReturnTypeOnly);
            }
        }
    };

    for i in &inputs {
        match i {
            FnArg::Receiver(_) => return Err(ValidationError::ReceiverArgsNotAccepted),
            FnArg::Typed(_) => todo!(),
        }
    }

    Ok(FunctionDefinition {
        name,
        result,
        inputs,
    })
}

#[derive(Debug)]
pub enum ValidationError {
    MissingOutput,
    ReceiverArgsNotAccepted,
    VerbatimReturnTypeOnly,
}
