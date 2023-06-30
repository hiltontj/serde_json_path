use nom::character::complete::char;
use nom::combinator::{cut, map_res};
use nom::multi::separated_list0;
use nom::sequence::{preceded, terminated};
use nom::{
    branch::alt,
    character::complete::{multispace0, satisfy},
    combinator::map,
    multi::fold_many1,
    sequence::{delimited, pair},
};
use serde_json_path_core::spec::functions::{
    Function, FunctionExpr, FunctionExprArg, FunctionValidationError, Validated,
};

pub(crate) mod registry;

use crate::parser::{parse_query, PResult};

use self::registry::REGISTRY;

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
pub(crate) fn parse_function_expr(input: &str) -> PResult<FunctionExpr<Validated>> {
    cut(map_res(
        pair(
            parse_function_name,
            delimited(
                terminated(char('('), multispace0),
                separated_list0(
                    delimited(multispace0, char(','), multispace0),
                    parse_function_argument,
                ),
                preceded(multispace0, char(')')),
            ),
        ),
        |(name, args)| {
            #[cfg(feature = "functions")]
            for f in inventory::iter::<Function> {
                if f.name == name {
                    (f.validator)(args.as_slice())?;
                    return Ok(FunctionExpr {
                        name,
                        args,
                        return_type: f.result_type,
                        validated: Validated {
                            evaluator: f.evaluator,
                        },
                    });
                }
            }
            if let Some(f) = REGISTRY.get(name.as_str()) {
                (f.validator)(args.as_slice())?;
                return Ok(FunctionExpr {
                    name,
                    args,
                    return_type: f.result_type,
                    validated: Validated {
                        evaluator: f.evaluator,
                    },
                });
            }
            Err(FunctionValidationError::Undefined { name })
        },
    ))(input)
}
