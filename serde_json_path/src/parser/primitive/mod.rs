use nom::{branch::alt, bytes::complete::tag, combinator::value};

use super::PResult;

pub(crate) mod int;
pub(crate) mod number;
pub(crate) mod string;

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_null(input: &str) -> PResult<()> {
    value((), tag("null"))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_bool(input: &str) -> PResult<bool> {
    let parse_true = value(true, tag("true"));
    let parse_false = value(false, tag("false"));
    alt((parse_true, parse_false))(input)
}
