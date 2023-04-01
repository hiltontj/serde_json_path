use proc_macro2::Ident;
use syn::{
    parse::{Parse, ParseStream},
    LitStr, Token,
};

pub(crate) struct StrArg<T> {
    pub(crate) value: LitStr,
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

pub(crate) struct IdentArg<T> {
    pub(crate) value: Ident,
    _p: std::marker::PhantomData<T>,
}

impl<T: Parse> Parse for IdentArg<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<T>()?;
        let _ = input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self {
            value,
            _p: std::marker::PhantomData,
        })
    }
}
