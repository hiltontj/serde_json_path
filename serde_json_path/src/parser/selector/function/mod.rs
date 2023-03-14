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
use crate::NodeList;

use super::filter::{parse_comparable, Comparable, TestFilter};

pub type Validator =
    Lazy<Box<dyn Fn(&[FunctionExprArg]) -> Result<(), FunctionValidationError> + Send + Sync>>;

pub type Evaluator =
    Lazy<Box<dyn for<'a> Fn(Vec<JsonPathType<'a>>) -> JsonPathType<'a> + Sync + Send>>;

pub struct Function {
    name: &'static str,
    result_type: JsonPathTypeKind,
    validator: &'static Validator,
    evaluator: &'static Evaluator,
}

impl Function {
    pub const fn new(
        name: &'static str,
        result_type: JsonPathTypeKind,
        evaluator: &'static Evaluator,
        validator: &'static Validator,
    ) -> Self {
        Self {
            name,
            result_type,
            evaluator,
            validator,
        }
    }
}

inventory::collect!(Function);

#[derive(Debug)]
pub struct NodesType<'a>(NodeList<'a>);

impl<'a> From<NodeList<'a>> for NodesType<'a> {
    fn from(value: NodeList<'a>) -> Self {
        Self(value)
    }
}

impl<'a> TryFrom<JsonPathType<'a>> for NodesType<'a> {
    type Error = ConversionError;

    fn try_from(value: JsonPathType<'a>) -> Result<Self, Self::Error> {
        match value {
            JsonPathType::Nodes(nl) => Ok(nl),
            JsonPathType::Value(_) => Err(ConversionError::new(
                JsonPathTypeKind::Value,
                JsonPathTypeKind::Nodes,
            )),
            JsonPathType::Logical(_) => Err(ConversionError::new(
                JsonPathTypeKind::Logical,
                JsonPathTypeKind::Nodes,
            )),
        }
    }
}

#[derive(Debug)]
pub enum LogicalType {
    True,
    False,
}

impl<'a> TryFrom<JsonPathType<'a>> for LogicalType {
    type Error = ConversionError;

    fn try_from(value: JsonPathType<'a>) -> Result<Self, Self::Error> {
        match value {
            JsonPathType::Nodes(_) => Err(ConversionError::new(
                JsonPathTypeKind::Nodes,
                JsonPathTypeKind::Logical,
            )),
            JsonPathType::Value(_) => Err(ConversionError::new(
                JsonPathTypeKind::Value,
                JsonPathTypeKind::Logical,
            )),
            JsonPathType::Logical(l) => Ok(l),
        }
    }
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

impl<'a> TryFrom<JsonPathType<'a>> for ValueType<'a> {
    type Error = ConversionError;

    fn try_from(value: JsonPathType<'a>) -> Result<Self, Self::Error> {
        match value {
            JsonPathType::Value(v) => Ok(v),
            JsonPathType::Nodes(_) => Err(ConversionError::new(
                JsonPathTypeKind::Nodes,
                JsonPathTypeKind::Value,
            )),
            JsonPathType::Logical(_) => Err(ConversionError::new(
                JsonPathTypeKind::Nodes,
                JsonPathTypeKind::Value,
            )),
        }
    }
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

impl<'a> JsonPathType<'a> {
    pub fn as_kind(&self) -> JsonPathTypeKind {
        match self {
            JsonPathType::Nodes(_) => JsonPathTypeKind::Nodes,
            JsonPathType::Value(_) => JsonPathTypeKind::Value,
            JsonPathType::Logical(_) => JsonPathTypeKind::Logical,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("attempted to convert {from} to {to}")]
pub struct ConversionError {
    from: JsonPathTypeKind,
    to: JsonPathTypeKind,
}

impl ConversionError {
    fn new(from: JsonPathTypeKind, to: JsonPathTypeKind) -> Self {
        Self { from, to }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum JsonPathTypeKind {
    Nodes,
    Value,
    Logical,
}

impl std::fmt::Display for JsonPathTypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonPathTypeKind::Nodes => write!(f, "NodesType"),
            JsonPathTypeKind::Value => write!(f, "ValueType"),
            JsonPathTypeKind::Logical => write!(f, "LogicalType"),
        }
    }
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
            FilterPath(q) => JsonPathType::Nodes(NodesType(q.query(current, root).into())),
            Comparable(c) => c.as_value(current, root),
        }
    }

    fn as_type_kind(&self) -> Result<JsonPathTypeKind, FunctionValidationError> {
        use FunctionExprArg::*;
        match self {
            FilterPath(query) => {
                if query.is_singular() {
                    Ok(JsonPathTypeKind::Value)
                } else {
                    Ok(JsonPathTypeKind::Nodes)
                }
            }
            Comparable(comp) => match comp {
                crate::parser::selector::filter::Comparable::Primitive { kind: _, value: _ } => {
                    Ok(JsonPathTypeKind::Value)
                }
                crate::parser::selector::filter::Comparable::SingularPath(_) => {
                    Ok(JsonPathTypeKind::Value)
                }
                crate::parser::selector::filter::Comparable::FunctionExpr(func) => {
                    for f in inventory::iter::<Function> {
                        if f.name == func.name.as_str() {
                            return Ok(f.result_type);
                        }
                    }
                    Err(FunctionValidationError::Undefined {
                        name: func.name.to_owned(),
                    })
                }
            },
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
