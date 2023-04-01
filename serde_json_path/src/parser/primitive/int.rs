use nom::character::complete::char;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    character::complete::digit0,
    combinator::{map_res, opt, recognize},
    sequence::tuple,
};

use crate::parser::PResult;

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_zero(input: &str) -> PResult<&str> {
    tag("0")(input)
}

fn is_non_zero_digit(chr: char) -> bool {
    ('1'..='9').contains(&chr)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_non_zero_first_digit(input: &str) -> PResult<&str> {
    take_while_m_n(1, 1, is_non_zero_digit)(input)
}

/// Parse a non-zero integer as `i64`
///
/// This does not allow leading `0`'s, e.g., `0123`
#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_non_zero_int(input: &str) -> PResult<&str> {
    recognize(tuple((opt(char('-')), parse_non_zero_first_digit, digit0)))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_int_string(input: &str) -> PResult<&str> {
    alt((parse_zero, parse_non_zero_int))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_int(input: &str) -> PResult<isize> {
    map_res(parse_int_string, |i_str| i_str.parse::<isize>())(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::primitive::int::parse_int;

    #[test]
    fn parse_integers() {
        assert_eq!(parse_int("0"), Ok(("", 0)));
        assert_eq!(parse_int("10"), Ok(("", 10)));
        assert_eq!(parse_int("-10"), Ok(("", -10)));
        // TODO - I don't know if this following test demonstrates the actual behaviour we want
        // i.e., do we want this to fail instead? or do we rely on higher level sequenced parsers
        // to fail, e.g., if a delimiter was expected after.
        assert_eq!(parse_int("010"), Ok(("10", 0)));
    }
}
