use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::common::{self, define::Expanded};

use super::args::FunctionMacroArgs;

pub(crate) fn expand(attrs: FunctionMacroArgs, input: ItemFn) -> TokenStream {
    let Expanded {
        name_str,
        validator,
        validator_name,
        evaluator,
        evaluator_name,
        result,
        core,
    } = match common::define::expand(input, attrs.name) {
        Ok(exp) => exp,
        Err(err) => return err.into(),
    };

    let inventory = quote! {
        ::serde_json_path_macros::inventory
    };

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
