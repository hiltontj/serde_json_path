use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

use crate::args::FunctionMacroArgs;

mod args;
mod define;
mod extract;

/// Register a function for use in JSONPath queries
#[proc_macro_attribute]
pub fn function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as FunctionMacroArgs);
    let func = parse_macro_input!(item as ItemFn);

    define::expand(args, func)
}
