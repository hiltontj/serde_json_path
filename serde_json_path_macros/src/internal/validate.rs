use proc_macro2::{Ident, TokenStream};
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, Error, FnArg, PatType, Path, Result,
    ReturnType, Signature, Type, TypePath,
};

pub struct FunctionDefinition {
    name: Ident,
    result: TokenStream,
    inputs: Punctuated<FnArg, Comma>,
}

fn extract_type_path(ty: &Type) -> Option<&Path> {
    match ty {
        Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
        _ => None,
    }
}

fn is_valid_json_path_type(p: &Path) -> Result<()> {
    // May need to support full type path to ensure that
    // correct type is being used.
    let ident = p
        .segments
        .last()
        .ok_or_else(|| Error::new(p.span(), "expected a type identifier"))?;
    todo!()
}

pub fn validate_signature(input: Signature) -> Result<FunctionDefinition> {
    let Signature {
        ident: ref name,
        ref inputs,
        ref output,
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
        ReturnType::Default => {
            return Err(Error::new(
                input.span(),
                "function signature expected to have return type",
            ))
        }
        ReturnType::Type(_, ty) => {
            if let Some(path) = extract_type_path(ty.as_ref()) {
                todo!()
            } else {
                return Err(Error::new(
                    ty.span(),
                    "return type can only be one of the serde_json_path types: NodesType, \
                        ValueType, or LogicalType",
                ));
            }
        }
    };

    // for i in &inputs {
    //     match i {
    //         FnArg::Receiver(_) => return Err(Error::new(
    //             inputs.span(),
    //             "receiver arguments like self, &self, or &mut self are not supported"
    //         )),
    //         FnArg::Typed(PatType{ attrs: _, pat: _, colon_token: _, ty }) => todo!(),
    //     }
    // }

    // Ok(FunctionDefinition {
    //     name,
    //     result,
    //     inputs,
    // })
    todo!()
}

#[derive(Debug)]
pub enum ValidationError {
    MissingOutput,
    ReceiverArgsNotAccepted,
    VerbatimReturnTypeOnly,
}
