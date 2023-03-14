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

pub type Validator =
    Lazy<Box<dyn Fn(&[FunctionExprArg]) -> Result<(), FunctionValidationError> + Send + Sync>>;

pub type Evaluator =
    Lazy<Box<dyn for<'a> Fn(Vec<JsonPathType<'a>>) -> JsonPathType<'a> + Sync + Send>>;

pub struct Function {
    name: &'static str,
    validator: &'static Validator,
    evaluator: &'static Evaluator,
}

impl Function {
    pub const fn new(
        name: &'static str,
        evaluator: &'static Evaluator,
        validator: &'static Validator,
    ) -> Self {
        Self {
            name,
            evaluator,
            validator,
        }
    }
}

inventory::collect!(Function);

#[derive(Debug)]
pub struct NodesType<'a>(Vec<&'a Value>);

impl<'a> From<Vec<&'a Value>> for NodesType<'a> {
    fn from(value: Vec<&'a Value>) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub enum LogicalType {
    True,
    False,
}

impl From<LogicalType> for bool {
    fn from(value: LogicalType) -> Self {
        match value {
            LogicalType::True => true,
            LogicalType::False => false,
        }
    }
}

#[derive(Debug)]
pub enum ValueType<'a> {
    Value(Value),
    ValueRef(&'a Value),
    Nothing,
}

impl TestFilter for Value {
    fn test_filter<'b>(&self, _current: &'b Value, _root: &'b Value) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => n != &Number::from(0),
            _ => true,
        }
    }
}

#[derive(Debug)]
pub enum JsonPathType<'a> {
    Nodes(NodesType<'a>),
    Value(ValueType<'a>),
    Logical(LogicalType),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionExpr {
    name: String,
    args: Vec<FunctionExprArg>,
}

impl std::fmt::Display for FunctionExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{name}(", name = self.name)?;
        for (i, arg) in self.args.iter().enumerate() {
            write!(
                f,
                "{arg}{comma}",
                comma = if i == self.args.len() - 1 { "" } else { "," }
            )?;
        }
        write!(f, ")")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum FunctionExprArg {
    FilterPath(Query),
    Comparable(Comparable),
}

impl std::fmt::Display for FunctionExprArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionExprArg::FilterPath(query) => write!(f, "{query}"),
            FunctionExprArg::Comparable(comparable) => write!(f, "{comparable}"),
        }
    }
}

impl FunctionExprArg {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Evaluate Function Arg", level = "trace", parent = None, ret))]
    fn evaluate<'a, 'b: 'a>(&'a self, current: &'b Value, root: &'b Value) -> JsonPathType<'a> {
        use FunctionExprArg::*;
        match self {
            FilterPath(q) => JsonPathType::Nodes(q.query(current, root).into()),
            Comparable(c) => c.as_value(current, root),
        }
    }
}

impl FunctionExpr {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Evaluate Function Expr", level = "trace", parent = None, ret))]
    pub fn evaluate<'a, 'b: 'a>(&'a self, current: &'b Value, root: &'b Value) -> JsonPathType<'_> {
        let args: Vec<JsonPathType> = self
            .args
            .iter()
            .map(|a| a.evaluate(current, root))
            .collect();
        for f in inventory::iter::<Function> {
            if f.name == self.name {
                return (f.evaluator)(args);
            }
        }
        // This is unreachable because if the function is not present, or validly declared, then
        // the JSON Path would not be parsed.
        unreachable!()
    }

    pub fn validate(&self) -> Result<(), FunctionValidationError> {
        for f in inventory::iter::<Function> {
            if f.name == self.name {
                return (f.validator)(self.args.as_slice());
            }
        }
        Err(FunctionValidationError::Undefined {
            name: self.name.to_owned(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FunctionValidationError {
    #[error("function name '{name}' is not defined")]
    Undefined { name: String },
}

impl TestFilter for FunctionExpr {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Test Function Expr", level = "trace", parent = None, ret))]
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool {
        match self.evaluate(current, root) {
            JsonPathType::Nodes(nl) => !nl.0.is_empty(),
            JsonPathType::Value(value) => match value {
                ValueType::Value(v) => v.test_filter(current, root),
                ValueType::ValueRef(v) => v.test_filter(current, root),
                ValueType::Nothing => false,
            },
            JsonPathType::Logical(l) => l.into(),
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
