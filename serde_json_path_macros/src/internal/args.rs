use syn::{
    parse::{Parse, ParseStream},
    LitStr, Token,
};

#[derive(Debug, Default)]
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

struct StrArg<T> {
    value: LitStr,
    _p: std::marker::PhantomData<T>,
}

impl<T: Parse> Parse for StrArg<T> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _ = input.parse::<T>()?;
        let _ = input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self {
            value,
            _p: std::marker::PhantomData,
        })
    }
}

mod kw {
    syn::custom_keyword!(name);
}
