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
use once_cell::sync::Lazy;
use serde_json::{Number, Value};

#[cfg(feature = "registry")]
pub mod registry;

use crate::parser::{parse_path, PResult, Query, Queryable};

use super::filter::{parse_comparable, Comparable, TestFilter};

pub type Evaluator = Lazy<Box<dyn for<'a> Fn(Vec<FuncType<'a>>) -> FuncType<'a> + Sync + Send>>;

pub struct Function {
    name: &'static str,
    evaluator: &'static Evaluator,
}

impl Function {
    pub const fn new(name: &'static str, evaluator: &'static Evaluator) -> Self {
        Self { name, evaluator }
    }
}

inventory::collect!(Function);

#[derive(Debug, Default)]
pub enum FuncType<'a> {
    Nodelist(Vec<&'a Value>),
    Node(Option<&'a Value>),
    Value(Value),
    ValueRef(&'a Value),
    #[default]
    Nothing,
}

#[derive(Debug, PartialEq)]
pub struct FunctionExpr {
    name: String,
    args: Vec<FunctionExprArg>,
}

#[derive(Debug, PartialEq)]
pub enum FunctionExprArg {
    FilterPath(Query),
    Comparable(Comparable),
}

impl FunctionExprArg {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Evaluate Function Arg", level = "trace", parent = None, ret))]
    fn evaluate<'a, 'b: 'a>(&'a self, current: &'b Value, root: &'b Value) -> FuncType<'a> {
        use FunctionExprArg::*;
        match self {
            FilterPath(q) => FuncType::Nodelist(q.query(current, root)),
            Comparable(c) => c.as_value(current, root),
        }
    }
}

impl FunctionExpr {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Evaluate Function Expr", level = "trace", parent = None, ret))]
    pub fn evaluate<'a, 'b: 'a>(&'a self, current: &'b Value, root: &'b Value) -> FuncType<'_> {
        let args: Vec<FuncType> = self
            .args
            .iter()
            .map(|a| a.evaluate(current, root))
            .collect();
        for f in inventory::iter::<Function> {
            if f.name == self.name {
                return (f.evaluator)(args);
            }
        }
        FuncType::Nothing
    }
}

impl TestFilter for FunctionExpr {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Test Function Expr", level = "trace", parent = None, ret))]
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool {
        match self.evaluate(current, root) {
            FuncType::Nodelist(nl) => !nl.is_empty(),
            FuncType::Node(n) => n.is_some(),
            FuncType::Value(v) => match v {
                Value::Null => false,
                Value::Bool(b) => b,
                Value::Number(n) => n != Number::from(0),
                _ => true,
            },
            FuncType::Nothing => false,
            FuncType::ValueRef(v) => match v {
                Value::Null => false,
                Value::Bool(b) => *b,
                Value::Number(n) => n != &Number::from(0),
                _ => true,
            },
        }
    }
}

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
        map(parse_path, FunctionExprArg::FilterPath),
        map(parse_comparable, FunctionExprArg::Comparable),
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
