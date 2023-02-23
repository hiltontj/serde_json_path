use nom::{
    sequence::{pair, delimited}, 
    bytes::complete::tag, 
    combinator::{map, value}, 
    multi::many0,
    character::complete::{char, space0}, 
    branch::alt
};

use crate::parser::{Query, PResult, parse_path};

use super::filter::{Comparable, parse_comparable, TestFilter};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FunctionName {
    Length,
    Count,
    Search,
    Match,
}

#[derive(Debug, PartialEq)]
pub enum FunctionArg {
    FilterPath(Query),
    Comparable(Comparable),
}

#[derive(Debug, PartialEq)]
pub struct FunctionExpr {
    name: FunctionName,
    args: Vec<FunctionArg>,
}

impl FunctionExpr {
    pub fn validate_args(&self) -> bool {
        use FunctionName::*;
        match self.name {
            Length => todo!(),
            Count => todo!(),
            Search => todo!(),
            Match => todo!(),
        }
    }

    pub fn is_comparable(&self) -> bool {
        use FunctionName::*;
        match self.name {
            Length => todo!(),
            Count => todo!(),
            Search => todo!(),
            Match => todo!(),
        }
    }

    pub fn is_filter_path(&self) -> bool {
        use FunctionName::*;
        match self.name {
            Length => todo!(),
            Count => todo!(),
            Search => todo!(),
            Match => todo!(),
        }
    }
}

impl TestFilter for FunctionExpr {
    fn test_filter<'b>(&self, current: &'b serde_json::Value, root: &'b serde_json::Value) -> bool {
        todo!()
    }
}

fn parse_function_name(input: &str) -> PResult<FunctionName> {
    use FunctionName::*;
    alt((
        value(Length, tag("length")),
        value(Count, tag("count")),
        value(Match, tag("match")),
        value(Search, tag("search")),
    ))(input)
}

fn parse_function_argument(input: &str) -> PResult<FunctionArg> {
    alt((
        map(parse_comparable, FunctionArg::Comparable),
        map(parse_path, FunctionArg::FilterPath),
    ))(input)
}

pub fn parse_function_expr(input: &str) -> PResult<FunctionExpr> {
    map(
        pair(
            parse_function_name,
            delimited(
                pair(char('('), space0),
                many0(parse_function_argument),
                pair(space0, char(')')),
            )
        ),
        |(name, args)| FunctionExpr { name, args }
    )(input)
}