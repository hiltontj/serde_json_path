//! Types used for representing Function Extensions in JSONPath
use std::collections::VecDeque;

use once_cell::sync::Lazy;
use serde_json::Value;

use crate::{node::NodeList, spec::query::Queryable};

use super::{
    query::Query,
    selector::filter::{Literal, LogicalOrExpr, SingularQuery, TestFilter},
};

#[doc(hidden)]
pub type Validator =
    Lazy<Box<dyn Fn(&[FunctionExprArg]) -> Result<(), FunctionValidationError> + Send + Sync>>;

#[doc(hidden)]
pub type Evaluator =
    Lazy<Box<dyn for<'a> Fn(VecDeque<JsonPathType<'a>>) -> JsonPathType<'a> + Sync + Send>>;

#[doc(hidden)]
#[allow(missing_debug_implementations)]
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

/// JSONPath type epresenting a Nodelist
#[derive(Debug)]
pub struct NodesType<'a>(NodeList<'a>);

impl<'a> NodesType<'a> {
    #[doc(hidden)]
    pub const fn type_kind() -> JsonPathTypeKind {
        JsonPathTypeKind::Nodelist
    }

    /// Extract the inner [`NodeList`]
    pub fn into_inner(self) -> NodeList<'a> {
        self.0
    }
}

impl<'a> From<NodeList<'a>> for NodesType<'a> {
    fn from(value: NodeList<'a>) -> Self {
        Self(value)
    }
}

impl<'a> TryFrom<JsonPathType<'a>> for NodesType<'a> {
    type Error = ConversionError;

    fn try_from(value: JsonPathType<'a>) -> Result<Self, Self::Error> {
        match value {
            JsonPathType::Nodes(nl) => Ok(nl.into()),
            JsonPathType::Value(_) => Err(ConversionError::LiteralToNodes),
            JsonPathType::Logical(_) => Err(ConversionError::IncompatibleTypes {
                from: JsonPathTypeKind::Logical,
                to: JsonPathTypeKind::Nodelist,
            }),
            JsonPathType::Node(n) => Ok(Self(vec![n].into())),
            JsonPathType::Nothing => Ok(Self(vec![].into())),
        }
    }
}

/// JSONPath type representing `LogicalTrue` or `LogicalFalse`
#[derive(Debug, Default)]
pub enum LogicalType {
    /// True
    True,
    /// False
    #[default]
    False,
}

impl LogicalType {
    #[doc(hidden)]
    pub const fn type_kind() -> JsonPathTypeKind {
        JsonPathTypeKind::Logical
    }
}

impl<'a> TryFrom<JsonPathType<'a>> for LogicalType {
    type Error = ConversionError;

    fn try_from(value: JsonPathType<'a>) -> Result<Self, Self::Error> {
        match value {
            JsonPathType::Nodes(nl) => {
                if nl.is_empty() {
                    Ok(Self::False)
                } else {
                    Ok(Self::True)
                }
            }
            JsonPathType::Value(_) => Err(ConversionError::IncompatibleTypes {
                from: JsonPathTypeKind::Value,
                to: JsonPathTypeKind::Logical,
            }),
            JsonPathType::Logical(l) => Ok(l),
            JsonPathType::Node(_) => Ok(Self::True),
            JsonPathType::Nothing => Ok(Self::False),
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

impl From<bool> for LogicalType {
    fn from(value: bool) -> Self {
        match value {
            true => Self::True,
            false => Self::False,
        }
    }
}

/// JSONPath type representing a JSON value or Nothing
#[derive(Debug)]
pub enum ValueType<'a> {
    /// A literal value
    Value(Value),
    /// A reference to a node in a JSON object, generally resulting from a singular query
    Node(&'a Value),
    /// Nothing, generally the result of a singular query that did not produce a node
    Nothing,
}

impl<'a> ValueType<'a> {
    #[doc(hidden)]
    pub const fn type_kind() -> JsonPathTypeKind {
        JsonPathTypeKind::Value
    }

    /// Convert to a reference of a [`serde_json::Value`] if possible
    pub fn as_value(&self) -> Option<&Value> {
        match self {
            ValueType::Value(v) => Some(v),
            ValueType::Node(v) => Some(v),
            ValueType::Nothing => None,
        }
    }
}

impl<'a> TryFrom<JsonPathType<'a>> for ValueType<'a> {
    type Error = ConversionError;

    fn try_from(value: JsonPathType<'a>) -> Result<Self, Self::Error> {
        match value {
            JsonPathType::Value(v) => Ok(Self::Value(v)),
            JsonPathType::Node(n) => Ok(Self::Node(n)),
            JsonPathType::Nothing => Ok(Self::Nothing),
            JsonPathType::Nodes(_) => Err(ConversionError::IncompatibleTypes {
                from: JsonPathTypeKind::Nodelist,
                to: JsonPathTypeKind::Value,
            }),
            JsonPathType::Logical(_) => Err(ConversionError::IncompatibleTypes {
                from: JsonPathTypeKind::Nodelist,
                to: JsonPathTypeKind::Value,
            }),
        }
    }
}

impl<'a, T> From<T> for ValueType<'a>
where
    T: Into<Value>,
{
    fn from(value: T) -> Self {
        Self::Value(value.into())
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum JsonPathType<'a> {
    Nodes(NodeList<'a>),
    Logical(LogicalType),
    Node(&'a Value),
    Value(Value),
    Nothing,
}

impl<'a> JsonPathType<'a> {
    pub fn as_kind(&self) -> JsonPathTypeKind {
        match self {
            JsonPathType::Nodes(_) => JsonPathTypeKind::Nodelist,
            JsonPathType::Value(_) => JsonPathTypeKind::Value,
            JsonPathType::Logical(_) => JsonPathTypeKind::Logical,
            JsonPathType::Node(_) => JsonPathTypeKind::Node,
            JsonPathType::Nothing => JsonPathTypeKind::Nothing,
        }
    }
}

impl<'a> From<NodesType<'a>> for JsonPathType<'a> {
    fn from(value: NodesType<'a>) -> Self {
        Self::Nodes(value.0)
    }
}

impl<'a> From<ValueType<'a>> for JsonPathType<'a> {
    fn from(value: ValueType<'a>) -> Self {
        match value {
            ValueType::Value(v) => Self::Value(v),
            ValueType::Node(n) => Self::Node(n),
            ValueType::Nothing => Self::Nothing,
        }
    }
}

impl<'a> From<LogicalType> for JsonPathType<'a> {
    fn from(value: LogicalType) -> Self {
        Self::Logical(value)
    }
}

/// Error used to convey JSONPath queries that are not well-typed
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    /// Cannot convert `from` into `to`
    #[error("attempted to convert {from} to {to}")]
    IncompatibleTypes {
        /// The type being converted from
        from: JsonPathTypeKind,
        /// The type being converted to
        to: JsonPathTypeKind,
    },
    /// Literal values can not be considered nodes
    #[error("cannot use a literal value in place of NodesType")]
    LiteralToNodes,
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JsonPathTypeKind {
    Nodelist,
    Node,
    Value,
    Logical,
    Nothing,
}

impl JsonPathTypeKind {
    pub fn converts_to(&self, other: Self) -> bool {
        matches!(
            (self, other),
            (
                JsonPathTypeKind::Nodelist,
                JsonPathTypeKind::Nodelist | JsonPathTypeKind::Logical
            ) | (
                JsonPathTypeKind::Node,
                JsonPathTypeKind::Nodelist | JsonPathTypeKind::Node | JsonPathTypeKind::Value
            ) | (
                JsonPathTypeKind::Value,
                JsonPathTypeKind::Node | JsonPathTypeKind::Value
            ) | (JsonPathTypeKind::Logical, JsonPathTypeKind::Logical)
        )
    }
}

impl std::fmt::Display for JsonPathTypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonPathTypeKind::Nodelist => write!(f, "NodesType"),
            JsonPathTypeKind::Logical => write!(f, "LogicalType"),
            JsonPathTypeKind::Node => write!(f, "ValueType"),
            JsonPathTypeKind::Value => write!(f, "ValueType"),
            JsonPathTypeKind::Nothing => write!(f, "ValueType"),
        }
    }
}

#[doc(hidden)]
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionExpr {
    pub name: String,
    pub args: Vec<FunctionExprArg>,
}

impl FunctionExpr {
    #[cfg_attr(
        feature = "trace",
        tracing::instrument(name = "Evaluate Function Expr", level = "trace", parent = None, ret)
    )]
    pub fn evaluate<'a, 'b: 'a>(&'a self, current: &'b Value, root: &'b Value) -> JsonPathType<'_> {
        let args: VecDeque<JsonPathType> = self
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

    pub fn validate(self) -> Result<Self, FunctionValidationError> {
        for f in inventory::iter::<Function> {
            if f.name == self.name {
                match (f.validator)(self.args.as_slice()) {
                    Ok(_) => return Ok(self),
                    Err(e) => return Err(e),
                }
            }
        }
        Err(FunctionValidationError::Undefined { name: self.name })
    }
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

#[doc(hidden)]
#[derive(Debug, PartialEq, Clone)]
pub enum FunctionExprArg {
    Literal(Literal),
    SingularQuery(SingularQuery),
    FilterQuery(Query),
    LogicalExpr(LogicalOrExpr),
    FunctionExpr(FunctionExpr),
}

impl std::fmt::Display for FunctionExprArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionExprArg::Literal(lit) => write!(f, "{lit}"),
            FunctionExprArg::FilterQuery(query) => write!(f, "{query}"),
            FunctionExprArg::SingularQuery(sq) => write!(f, "{sq}"),
            FunctionExprArg::LogicalExpr(log) => write!(f, "{log}"),
            FunctionExprArg::FunctionExpr(func) => write!(f, "{func}"),
        }
    }
}

impl FunctionExprArg {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Evaluate Function Arg", level = "trace", parent = None, ret))]
    fn evaluate<'a, 'b: 'a>(&'a self, current: &'b Value, root: &'b Value) -> JsonPathType<'a> {
        match self {
            FunctionExprArg::Literal(lit) => lit.into(),
            FunctionExprArg::SingularQuery(q) => match q.eval_query(current, root) {
                Some(n) => JsonPathType::Node(n),
                None => JsonPathType::Nothing,
            },
            FunctionExprArg::FilterQuery(q) => JsonPathType::Nodes(q.query(current, root).into()),
            FunctionExprArg::LogicalExpr(l) => match l.test_filter(current, root) {
                true => JsonPathType::Logical(LogicalType::True),
                false => JsonPathType::Logical(LogicalType::False),
            },
            FunctionExprArg::FunctionExpr(f) => f.evaluate(current, root),
        }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(name = "Function Arg As Type Kind", level = "trace", parent = None, ret))]
    pub fn as_type_kind(&self) -> Result<JsonPathTypeKind, FunctionValidationError> {
        match self {
            FunctionExprArg::Literal(_) => Ok(JsonPathTypeKind::Value),
            FunctionExprArg::SingularQuery(_) => Ok(JsonPathTypeKind::Node),
            FunctionExprArg::FilterQuery(query) => {
                if query.is_singular() {
                    Ok(JsonPathTypeKind::Node)
                } else {
                    Ok(JsonPathTypeKind::Nodelist)
                }
            }
            FunctionExprArg::LogicalExpr(_) => Ok(JsonPathTypeKind::Logical),
            FunctionExprArg::FunctionExpr(func) => {
                for f in inventory::iter::<Function> {
                    if f.name == func.name.as_str() {
                        return Ok(f.result_type);
                    }
                }
                Err(FunctionValidationError::Undefined {
                    name: func.name.to_owned(),
                })
            }
        }
    }
}

/// An error occurred while validating a function
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum FunctionValidationError {
    /// Function not defined in inventory
    #[error("function name '{name}' is not defined")]
    Undefined {
        /// The name of the function
        name: String,
    },
    /// Mismatch in number of function arguments
    #[error("expected {expected} args, but received {received}")]
    NumberOfArgsMismatch {
        /// Expected number of arguments
        expected: usize,
        /// Received number of arguments
        received: usize,
    },
    /// The type of received argument does not match the function definition
    #[error("in argument position {position}, expected a type that converts to {expected}, received {received}")]
    MismatchTypeKind {
        /// Expected type
        expected: JsonPathTypeKind,
        /// Received type
        received: JsonPathTypeKind,
        /// Argument position
        position: usize,
    },
}

impl TestFilter for FunctionExpr {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Test Function Expr", level = "trace", parent = None, ret))]
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool {
        match self.evaluate(current, root) {
            JsonPathType::Nodes(nl) => !nl.is_empty(),
            JsonPathType::Value(v) => v.test_filter(current, root),
            JsonPathType::Logical(l) => l.into(),
            JsonPathType::Node(n) => n.test_filter(current, root),
            JsonPathType::Nothing => false,
        }
    }
}
