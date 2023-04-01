use std::collections::VecDeque;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, Error, FnArg, Generics, Pat, PatType,
    Path, Result, ReturnType, Signature, Type,
};

pub struct Components {
    pub name: Ident,
    pub generics: Generics,
    pub validator_name: Ident,
    pub evaluator_name: Ident,
    pub result: TokenStream,
    pub ret: ReturnType,
    pub inputs: Punctuated<FnArg, Comma>,
    pub args: VecDeque<FnArgument>,
}

pub struct FnArgument {
    pub ident: Ident,
    pub ty: TokenStream,
}

fn extract_pat_ident(pat: &Pat) -> Option<Ident> {
    if let Pat::Ident(ref pat_ident) = pat {
        Some(pat_ident.ident.to_owned())
    } else {
        None
    }
}

fn extract_type_path(ty: &Type) -> Option<&Path> {
    match ty {
        Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
        Type::Reference(ref typeref) => match *typeref.elem {
            Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
            _ => None,
        },
        _ => None,
    }
}

fn extract_json_path_type(p: &Path) -> Result<TokenStream> {
    // TODO - support full type path to ensure that correct type is being used?
    //      - i.e., instead of just looking at last path segment
    let p_seg = p
        .segments
        .last()
        .ok_or_else(|| Error::new(p.span(), "expected a type identifier"))?;
    let ts = match p_seg.ident.to_string().as_str() {
        "NodesType" => quote! {
            ::serde_json_path_macros::serde_json_path_core::spec::functions::NodesType
        },
        "ValueType" => quote! {
            ::serde_json_path_macros::serde_json_path_core::spec::functions::ValueType
        },
        "LogicalType" => quote! {
            ::serde_json_path_macros::serde_json_path_core::spec::functions::LogicalType
        },
        other => {
            return Err(Error::new(
                p_seg.ident.span(),
                format!(
                    "expected 'NodesType', 'ValueType', or 'LogicalType', got '{}'",
                    other,
                ),
            ))
        }
    };
    Ok(ts)
}

pub fn extract_components(input: Signature) -> Result<Components> {
    let name = input.ident.clone();
    let generics = input.generics.clone();
    let inputs = input.inputs.clone();
    let ret = input.output.clone();

    let result = match &ret {
        ReturnType::Default => {
            return Err(Error::new(
                input.span(),
                "function signature expected to have return type",
            ))
        }
        ReturnType::Type(_, ty) => {
            if let Some(path) = extract_type_path(ty.as_ref()) {
                extract_json_path_type(path)?
            } else {
                return Err(Error::new(
                    ty.span(),
                    "return type can only be one of the serde_json_path types: NodesType, \
                        ValueType, or LogicalType",
                ));
            }
        }
    };

    let args: Result<VecDeque<FnArgument>> = inputs
        .iter()
        .map(|i| match i {
            FnArg::Receiver(_) => Err(Error::new(
                inputs.span(),
                "receiver arguments like self, &self, or &mut self are not supported",
            )),
            FnArg::Typed(PatType {
                attrs: _,
                pat,
                colon_token: _,
                ty,
            }) => {
                let ident = if let Some(id) = extract_pat_ident(pat) {
                    id
                } else {
                    return Err(Error::new(
                        pat.span(),
                        "expected identifier in function argument",
                    ));
                };
                let ty = if let Some(path) = extract_type_path(ty) {
                    extract_json_path_type(path)?
                } else {
                    return Err(Error::new(
                        ty.span(),
                        "argument type can only be one of the serde_json_path types: NodesType, \
                                ValueType, or LogicalType",
                    ));
                };
                Ok(FnArgument { ident, ty })
            }
        })
        .collect();

    let args = args?;
    let validator_name = Ident::new(
        format!("{name}_validator").to_uppercase().as_str(),
        name.span(),
    );
    let evaluator_name = Ident::new(
        format!("{name}_evaluator").to_uppercase().as_str(),
        name.span(),
    );

    Ok(Components {
        name,
        generics,
        result,
        inputs,
        ret,
        args,
        validator_name,
        evaluator_name,
    })
}
