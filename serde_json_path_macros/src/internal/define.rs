use proc_macro::TokenStream;
use syn::{ItemFn, Signature};

use crate::validate::validate_signature;

pub fn expand(input: ItemFn) -> TokenStream {
    let ItemFn {
        attrs: _,
        vis: _,
        sig,
        block,
    } = input;

    let func_def = validate_signature(sig).expect("valid function signature");

    todo!();
}
