use nom::{branch::alt, bytes::complete::tag, combinator::value};

use super::PResult;

pub mod int;
pub mod number;
pub mod string;

#[tracing::instrument(level = "trace", parent = None, ret, err)]
pub fn parse_null(input: &str) -> PResult<()> {
    value((), tag("null"))(input)
}

#[tracing::instrument(level = "trace", parent = None, ret, err)]
pub fn parse_bool(input: &str) -> PResult<bool> {
    let parse_true = value(true, tag("true"));
    let parse_false = value(false, tag("false"));
    alt((parse_true, parse_false))(input)
}
