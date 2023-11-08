use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit0, digit1, one_of},
    combinator::{map_res, opt, recognize},
    sequence::{preceded, tuple},
};
use serde_json::Number;

use crate::parser::PResult;

use super::int::parse_int_string;

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_fractional(input: &str) -> PResult<&str> {
    preceded(char('.'), digit1)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_exponent(input: &str) -> PResult<&str> {
    recognize(tuple((one_of("eE"), opt(one_of("-+")), digit0)))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_number_string(input: &str) -> PResult<&str> {
    recognize(tuple((
        alt((parse_int_string, tag("-0"))),
        opt(parse_fractional),
        opt(parse_exponent),
    )))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_number(input: &str) -> PResult<Number> {
    map_res(parse_number_string, Number::from_str)(input)
}

#[cfg(test)]
mod tests {
    use serde_json::Number;

    use super::parse_number;

    #[test]
    fn test_numbers() {
        assert_eq!(parse_number("123"), Ok(("", Number::from(123))));
        assert_eq!(parse_number("-1"), Ok(("", Number::from(-1))));
        assert_eq!(
            parse_number("1e10"),
            Ok(("", Number::from_f64(1e10).unwrap()))
        );
        assert_eq!(
            parse_number("1.0001"),
            Ok(("", Number::from_f64(1.0001).unwrap()))
        );
        assert_eq!(
            parse_number("-0"),
            Ok(("", Number::from_f64(-0.0).unwrap()))
        );
    }
}
