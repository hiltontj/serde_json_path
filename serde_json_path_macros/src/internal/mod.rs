use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

mod common;
mod func;
mod reg;

#[proc_macro_attribute]
pub fn function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as func::FunctionMacroArgs);
    let item_fn = parse_macro_input!(item as ItemFn);

    func::expand(args, item_fn)
}

#[proc_macro_attribute]
pub fn register(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as reg::RegisterMacroArgs);
    let item_fn = parse_macro_input!(item as ItemFn);

    reg::expand(args, item_fn)
}
