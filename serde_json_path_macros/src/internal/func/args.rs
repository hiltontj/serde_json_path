use syn::{parse::Parse, LitStr};

use crate::common::args::StrArg;

#[derive(Default)]
pub(crate) struct FunctionMacroArgs {
    pub(crate) name: Option<LitStr>,
}

impl Parse for FunctionMacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::name) {
                if args.name.is_some() {
                    return Err(input.error("expected only a single `name` argument"));
                }
                let name = input.parse::<StrArg<kw::name>>()?.value;
                args.name = Some(name);
            } else {
                // TODO - may want to warn here when found a invalid arg - see how
                // tracing::instrument stores warnings and emits them later when generating the
                // expanded token stream.
                let _ = input.parse::<proc_macro2::TokenTree>();
            }
        }
        Ok(args)
    }
}

mod kw {
    syn::custom_keyword!(name);
}
