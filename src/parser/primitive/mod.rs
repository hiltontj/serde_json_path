use nom::{branch::alt, bytes::complete::tag, combinator::value, IResult};

pub mod int;
pub mod number;
pub mod string;

pub fn parse_null(input: &str) -> IResult<&str, ()> {
    value((), tag("null"))(input)
}

pub fn parse_bool(input: &str) -> IResult<&str, bool> {
    alt((value(true, tag("true")), value(false, tag("false"))))(input)
}
