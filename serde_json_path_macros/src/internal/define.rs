use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, LitStr};

use crate::{
    args::FunctionMacroArgs,
    extract::{extract_components, Components, FnArgument},
};

pub(crate) fn expand(attrs: FunctionMacroArgs, input: ItemFn) -> TokenStream {
    let ItemFn {
        attrs: _,
        vis: _,
        sig,
        block,
    } = input;

    let Components {
        name,
        validator_name,
        evaluator_name,
        result,
        args,
        ret,
        inputs,
    } = match extract_components(sig) {
        Ok(fd) => fd,
        Err(e) => return e.into_compile_error().into(),
    };
    let args_len = args.len();
    let inventory = quote! {
        ::serde_json_path_macros::inventory
    };
    let lazy = quote! {
        ::serde_json_path_macros::once_cell::sync::Lazy
    };
    let core = quote! {
        ::serde_json_path_macros::serde_json_path_core::spec::functions
    };
    let res = quote! {
        std::result::Result
    };
    let arg_checks = args.iter().enumerate().map(|(idx, arg)| {
        let FnArgument { ident: _, ty } = arg;
        quote! {
            match a[#idx].as_type_kind() {
                #res::Ok(tk) => {
                    if !tk.converts_to(#ty::json_path_type()) {
                        return #res::Err(#core::FunctionValidationError::MismatchTypeKind {
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

    let arg_declarations = args.iter().map(|arg| {
        let FnArgument { ident, ty } = arg;
        // validation should ensure unwrap is okay here:
        quote! {
            let #ident = #ty::try_from(v.pop_front().unwrap()).unwrap();
        }
    });
    let arg_names = args.iter().map(|arg| {
        let FnArgument { ident, ty: _ } = arg;
        ident
    });

    let evaluator = quote! {
        fn #name(#inputs) #ret #block
        static #evaluator_name: #core::Evaluator = #lazy::new(|| {
            std::boxed::Box::new(|mut v: std::collections::VecDeque<#core::JsonPathValue>| {
                #(#arg_declarations)*
                return #name(#(#arg_names,)*).into()
            })
        });
    };

    // TODO - may just put the str in the components directly, if the ident is not used for anything
    //            else
    let name_str = attrs
        .name
        .unwrap_or_else(|| LitStr::new(name.to_string().as_str(), name.span()));

    TokenStream::from(quote! {
        #validator
        #evaluator
        #inventory::submit! {
            #core::Function::new(
                #name_str,
                #result::function_type(),
                &#evaluator_name,
                &#validator_name,
            )
        }
    })
}
