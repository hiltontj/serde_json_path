use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

mod define;
mod extract;

#[proc_macro_attribute]
pub fn function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let expanded = define::expand(func);
    TokenStream::from(expanded)
}
