//! Internal macro implementation for the `serde_json_path` crate

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

mod common;
mod func;
mod reg;

/// Define a function for use in JSONPath queries
///
/// Refer to the documentation on the re-export in the `serde_json_path_macros` crate.
#[proc_macro_attribute]
pub fn function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as func::FunctionMacroArgs);
    let item_fn = parse_macro_input!(item as ItemFn);

    func::expand(args, item_fn)
}

/// Register a function for use in JSONPath queries
///
/// Refer to the documentation on the re-export in the `serde_json_path_macros` crate.
#[proc_macro_attribute]
pub fn register(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as reg::RegisterMacroArgs);
    let item_fn = parse_macro_input!(item as ItemFn);

    reg::expand(args, item_fn)
}
