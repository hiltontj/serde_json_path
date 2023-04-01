use proc_macro2::Ident;
use syn::{parse::Parse, LitStr};

use crate::common::args::{IdentArg, StrArg};

pub(crate) struct RegisterMacroArgs {
    pub(crate) name: Option<LitStr>,
    pub(crate) target: Ident,
}

#[derive(Default)]
struct RegisterMacroArgsBuilder {
    name: Option<LitStr>,
    target: Option<Ident>,
}

impl RegisterMacroArgsBuilder {
    fn build(self, input: &syn::parse::ParseStream) -> syn::Result<RegisterMacroArgs> {
        if let Some(target) = self.target {
            Ok(RegisterMacroArgs {
                name: self.name,
                target,
            })
        } else {
            Err(input.error("missing `target` argument"))
        }
    }
}

impl Parse for RegisterMacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut builder = RegisterMacroArgsBuilder::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::name) {
                if builder.name.is_some() {
                    return Err(input.error("expected only a single `name` argument"));
                }
                let name = input.parse::<StrArg<kw::name>>()?.value;
                builder.name = Some(name);
            } else if lookahead.peek(kw::target) {
                if builder.target.is_some() {
                    return Err(input.error("expected only a single `target` argument"));
                }
                let target = input.parse::<IdentArg<kw::target>>()?.value;
                builder.target = Some(target);
            } else {
                let _ = input.parse::<proc_macro2::TokenTree>();
            }
        }
        builder.build(&input)
    }
}

mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(target);
}
