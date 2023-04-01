use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{ItemFn, LitStr};

use crate::common::extract::FnArgument;

use super::extract::{extract_components, Components};

pub(crate) struct Expanded {
    pub(crate) name_str: LitStr,
    pub(crate) validator: TokenStream,
    pub(crate) validator_name: Ident,
    pub(crate) evaluator: TokenStream,
    pub(crate) evaluator_name: Ident,
    pub(crate) result: TokenStream,
    pub(crate) core: TokenStream,
}

/// Expand the macro input to produce the common elements used in the `#[function]` and
/// `#[register]` macros.
pub(crate) fn expand(input: ItemFn, name_str: Option<LitStr>) -> Result<Expanded, TokenStream> {
    let ItemFn {
        attrs: _,
        vis: _,
        sig,
        block,
    } = input;

    let Components {
        name,
        generics,
        validator_name,
        evaluator_name,
        result,
        args,
        ret,
        inputs,
    } = match extract_components(sig) {
        Ok(fd) => fd,
        Err(e) => return Err(e.into_compile_error()),
    };
    // Stringified name of the function:
    let name_str = name_str.unwrap_or_else(|| LitStr::new(name.to_string().as_str(), name.span()));
    // The number of arguments the function accepts:
    let args_len = args.len();
    // Generate token streams for some needed types:
    let lazy = quote! {
        ::serde_json_path_macros::once_cell::sync::Lazy
    };
    let core = quote! {
        ::serde_json_path_macros::serde_json_path_core::spec::functions
    };
    let res = quote! {
        std::result::Result
    };
    // Generate code for checking each individual argument in a query at parse time:
    let arg_checks = args.iter().enumerate().map(|(idx, arg)| {
        let FnArgument { ident: _, ty } = arg;
        quote! {
            match a[#idx].as_type_kind() {
                #res::Ok(tk) => {
                    if !tk.converts_to(#ty::json_path_type()) {
                        return #res::Err(#core::FunctionValidationError::MismatchTypeKind {
                            name: String::from(#name_str),
                            expected: #ty::json_path_type(),
                            received: tk,
                            position: #idx,
                        });
                    }
                },
                #res::Err(err) => return #res::Err(err)
            }
        }
    });
    // Generate the validator function used at parse time to validate a function declaration:
    let validator = quote! {
        static #validator_name: #core::Validator = #lazy::new(|| {
            std::boxed::Box::new(|a: &[#core::FunctionExprArg]| {
                if a.len() != #args_len {
                    return #res::Err(#core::FunctionValidationError::NumberOfArgsMismatch {
                        expected: a.len(),
                        received: #args_len,
                    });
                }
                #(#arg_checks)*
                Ok(())
            })
        });
    };
    // Generate the code to declare each individual argument for evaluation, at query time:
    let arg_declarations = args.iter().map(|arg| {
        let FnArgument { ident, ty } = arg;
        // validation should ensure unwrap is okay here:
        quote! {
            let #ident = #ty::try_from(v.pop_front().unwrap()).unwrap();
        }
    });
    // Produce the argument name identifiers:
    let arg_names = args.iter().map(|arg| {
        let FnArgument { ident, ty: _ } = arg;
        ident
    });
    // Generate the evaluator function used to evaluate a function at query time:
    let evaluator = quote! {
        fn #name #generics (#inputs) #ret #block
        static #evaluator_name: #core::Evaluator = #lazy::new(|| {
            std::boxed::Box::new(|mut v: std::collections::VecDeque<#core::JsonPathValue>| {
                #(#arg_declarations)*
                return #name(#(#arg_names,)*).into()
            })
        });
    };

    Ok(Expanded {
        name_str,
        validator,
        validator_name,
        evaluator,
        evaluator_name,
        result,
        core,
    })
}
