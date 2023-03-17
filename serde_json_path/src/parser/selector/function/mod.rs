use nom::character::complete::char;
use nom::multi::separated_list0;
use nom::sequence::tuple;
use nom::{
    branch::alt,
    character::complete::{satisfy, space0},
    combinator::map,
    multi::fold_many1,
    sequence::{delimited, pair},
};
use serde_json_path_core::spec::functions::{FunctionExpr, FunctionExprArg};

#[cfg(feature = "registry")]
pub mod registry;

use crate::parser::{parse_path, PResult};

use super::filter::parse_comparable;

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_function_name_first(input: &str) -> PResult<char> {
    satisfy(|c| c.is_ascii_lowercase())(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_function_name_char(input: &str) -> PResult<char> {
    alt((
        parse_function_name_first,
        char('_'),
        satisfy(|c| c.is_ascii_digit()),
    ))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_function_name(input: &str) -> PResult<String> {
    map(
        pair(
            parse_function_name_first,
            fold_many1(
                parse_function_name_char,
                String::new,
                |mut string, fragment| {
                    string.push(fragment);
                    string
                },
            ),
        ),
        |(first, rest)| format!("{first}{rest}"),
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_function_argument(input: &str) -> PResult<FunctionExprArg> {
    alt((
        map(parse_comparable, FunctionExprArg::Comparable),
        map(parse_path, FunctionExprArg::FilterPath),
    ))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub fn parse_function_expr(input: &str) -> PResult<FunctionExpr> {
    map(
        pair(
            parse_function_name,
            delimited(
                pair(char('('), space0),
                separated_list0(tuple((space0, char(','), space0)), parse_function_argument),
                pair(space0, char(')')),
            ),
        ),
        |(name, args)| FunctionExpr { name, args },
    )(input)
}
