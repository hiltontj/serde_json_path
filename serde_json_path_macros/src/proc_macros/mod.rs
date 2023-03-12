use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr};

#[proc_macro_attribute]
pub fn json_path(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let block = func.block;
    let sig = func.sig;
    let args = sig.inputs;
    let ret = sig.output;
    let fname = sig.ident;
    let static_fname = Ident::new(fname.to_string().to_uppercase().as_str(), Span::call_site());
    let str_fname = LitStr::new(fname.to_string().as_str(), Span::call_site());
    let inventory = quote! {
        ::serde_json_path_macros::inventory
    };
    let function_struct = quote! {
        serde_json_path::Function
    };
    let evaluator = quote! {
        serde_json_path::Evaluator
    };
    let lazy = quote! {
        ::serde_json_path_macros::once_cell::sync::Lazy
    };
    TokenStream::from(quote! {
        fn #fname(#args) #ret #block
        static #static_fname: #evaluator = #lazy::new(|| {
            Box::new(#fname)
        });
        #inventory::submit! {
            #function_struct::new(
                #str_fname,
                &#static_fname,
            )
        }
    })
}
