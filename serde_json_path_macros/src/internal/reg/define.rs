use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::common::{self, define::Expanded};

use super::args::RegisterMacroArgs;

pub(crate) fn expand(attrs: RegisterMacroArgs, input: ItemFn) -> TokenStream {
    let RegisterMacroArgs { name, target } = attrs;

    let Expanded {
        name_str,
        validator,
        validator_name,
        evaluator,
        evaluator_name,
        result,
        core,
    } = match common::define::expand(input, name) {
        Ok(exp) => exp,
        Err(err) => return err.into(),
    };

    TokenStream::from(quote! {
        #validator
        #evaluator
        static #target: #core::Function = #core::Function::new(
            #name_str,
            #result::function_type(),
            &#evaluator_name,
            &#validator_name,
        );
    })
}
