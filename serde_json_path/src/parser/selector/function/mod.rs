use nom::character::complete::char;
use nom::combinator::{cut, map_res};
use nom::multi::separated_list0;
use nom::sequence::{preceded, terminated};
use nom::{
    branch::alt,
    character::complete::{satisfy, space0},
    combinator::map,
    multi::fold_many1,
    sequence::{delimited, pair},
};
use serde_json_path_core::spec::functions::{FunctionExpr, FunctionExprArg};

#[cfg(feature = "registry")]
pub(crate) mod registry;

use crate::parser::{parse_query, PResult};

use super::filter::{parse_literal, parse_logical_or_expr, parse_singular_path};

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
        map(parse_literal, FunctionExprArg::Literal),
        map(parse_singular_path, FunctionExprArg::SingularQuery),
        map(parse_query, FunctionExprArg::FilterQuery),
        map(parse_function_expr, FunctionExprArg::FunctionExpr),
        map(parse_logical_or_expr, FunctionExprArg::LogicalExpr),
    ))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_function_expr(input: &str) -> PResult<FunctionExpr> {
    cut(map_res(
        pair(
            parse_function_name,
            delimited(
                terminated(char('('), space0),
                separated_list0(
                    delimited(space0, char(','), space0),
                    parse_function_argument,
                ),
                preceded(space0, char(')')),
            ),
        ),
        |(name, args)| FunctionExpr::validate(name, args),
    ))(input)
}
