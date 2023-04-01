//! Function Extensions in JSONPath
//!
//! Function Extensions in JSONPath serve as a way to extend the capability of queries in a way that
//! the standard query syntax can not support. There are various functions included in JSONPath, all
//! of which conform to a specified type system.
//!
//! # The JSONPath Type System
//!
//! The type system used in JSONPath function extensions is comprised of three types: [`NodesType`],
//! [`ValueType`], and [`LogicalType`]. All functions use some combination of these types as their
//! arguments, and use one of these types as their return type.
//!
//! # Registered Functions
//!
//! The IETF JSONPath Specification defines several functions for use in JSONPath query filter
//! expressions, all of which are provided for use in `serde_json_path`, and defined below.
//!
//! ## `length`
//!
//! The `length` function extension provides a way to compute the length of a value and make that
//! available for further processing in the filter expression.
//!
//! ### Parameters
//!
//! | Type | Description |
//! |------|-------------|
//! | [`ValueType`] | string, object, or array, possibly taken from a singular query |
//!
//! ### Result
//!
//! | Type | Description |
//! |------|-------------|
//! | [`ValueType`] | unsigned integer, or nothing |
//!
//! ### Example
//!
//! ```text
//! $[?length(@.authors) >= 5]
//! ```
//!
//! ## `count`
//!
//! The `count` function extension provides a way to obtain the number of nodes in a nodelist and
//! make that available for further processing inthe filter expression.
//!
//! ### Parameters
//!
//! | Type | Description |
//! |------|-------------|
//! | [`NodesType`] | the nodelist whose members are being counted |
//!
//! ### Result
//!
//! | Type | Description |
//! |------|-------------|
//! | [`ValueType`] | an unsigned integer |
//!
//! ### Example
//!
//! ```text
//! $[?count(@.*.author) >= 5]
//! ```
//!
//! ## `match`
//!
//! The `match` function extension provides a way to check whether **the entirety** of a given
//! string matches a given regular expression.
//!
//! ### Parameters
//!
//! | Type | Description |
//! |------|-------------|
//! | [`ValueType`] | a string |
//! | [`ValueType`] | a string representing a valid regular expression |
//!
//! ### Result
//!
//! | Type | Description |
//! |------|-------------|
//! | [`LogicalType`] | true for a match, false otherwise |
//!
//! ### Example
//!
//! ```text
//! $[?match(@.date, "1974-05-..")]
//! ```
//!
//! ## `search`
//!
//! The `search` function extension provides a way to check whether a given string contains a
//! substring that matches a given regular expression.
//!
//! ### Parameters
//!
//! | Type | Description |
//! |------|-------------|
//! | [`ValueType`] | a string |
//! | [`ValueType`] | a string representing a valid regular expression |
//!
//! ### Result
//!
//! | Type | Description |
//! |------|-------------|
//! | [`LogicalType`] | true for a match, false otherwise |
//!
//! ### Example
//!
//! ```text
//! $[?search(@.author, "[BR]ob")]
//! ```
//!
//! ## `value`
//!
//! The `value` function extension provides a way to convert an instance of `NodesType` to a value
//! and make that available for further processing in the filter expression.
//!
//! ### Parameters
//!
//! | Type | Description |
//! |------|-------------|
//! | [`NodesType`] | a nodelist to convert to a value |
//!
//! ### Result
//!
//! | Type | Description |
//! |------|-------------|
//! | [`ValueType`] | if the input nodelist contains a single node, the result is the value of that node, otherwise it is nothing |
//!
//! ### Example
//!
//! ```text
//! $[?value(@..color) == "red"]
//! ```
//!
use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
};

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
    Lazy<Box<dyn for<'a> Fn(VecDeque<JsonPathValue<'a>>) -> JsonPathValue<'a> + Sync + Send>>;

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct Function {
    pub name: &'static str,
    pub result_type: FunctionArgType,
    pub validator: &'static Validator,
    pub evaluator: &'static Evaluator,
}

impl Function {
    pub const fn new(
        name: &'static str,
        result_type: FunctionArgType,
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

#[cfg(feature = "functions")]
inventory::collect!(Function);

/// JSONPath type epresenting a Nodelist
///
/// This is a thin wrapper around a [`NodeList`], and generally represents the result of a JSONPath
/// query. It may also be produced by a function.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct NodesType<'a>(NodeList<'a>);

impl<'a> NodesType<'a> {
    #[doc(hidden)]
    pub const fn json_path_type() -> JsonPathType {
        JsonPathType::Nodes
    }

    #[doc(hidden)]
    pub const fn function_type() -> FunctionArgType {
        FunctionArgType::Nodelist
    }

    /// Extract all inner nodes as a vector
    ///
    /// Uses the [`NodeList::all`][NodeList::all] method.
    pub fn all(self) -> Vec<&'a Value> {
        self.0.all()
    }
}

impl<'a> IntoIterator for NodesType<'a> {
    type Item = &'a Value;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> Deref for NodesType<'a> {
    type Target = NodeList<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for NodesType<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> From<NodeList<'a>> for NodesType<'a> {
    fn from(value: NodeList<'a>) -> Self {
        Self(value)
    }
}

impl<'a> From<Vec<&'a Value>> for NodesType<'a> {
    fn from(values: Vec<&'a Value>) -> Self {
        Self(values.into())
    }
}

impl<'a> TryFrom<JsonPathValue<'a>> for NodesType<'a> {
    type Error = ConversionError;

    fn try_from(value: JsonPathValue<'a>) -> Result<Self, Self::Error> {
        match value {
            JsonPathValue::Nodes(nl) => Ok(nl.into()),
            JsonPathValue::Value(_) => Err(ConversionError::LiteralToNodes),
            JsonPathValue::Logical(_) => Err(ConversionError::IncompatibleTypes {
                from: JsonPathType::Logical,
                to: JsonPathType::Nodes,
            }),
            JsonPathValue::Node(n) => Ok(Self(vec![n].into())),
            JsonPathValue::Nothing => Ok(Self(vec![].into())),
        }
    }
}

/// JSONPath type representing `LogicalTrue` or `LogicalFalse`
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum LogicalType {
    /// True
    True,
    /// False
    #[default]
    False,
}

impl LogicalType {
    #[doc(hidden)]
    pub const fn json_path_type() -> JsonPathType {
        JsonPathType::Logical
    }

    #[doc(hidden)]
    pub const fn function_type() -> FunctionArgType {
        FunctionArgType::Logical
    }
}

impl<'a> TryFrom<JsonPathValue<'a>> for LogicalType {
    type Error = ConversionError;

    fn try_from(value: JsonPathValue<'a>) -> Result<Self, Self::Error> {
        match value {
            JsonPathValue::Nodes(nl) => {
                if nl.is_empty() {
                    Ok(Self::False)
                } else {
                    Ok(Self::True)
                }
            }
            JsonPathValue::Value(_) => Err(ConversionError::IncompatibleTypes {
                from: JsonPathType::Value,
                to: JsonPathType::Logical,
            }),
            JsonPathValue::Logical(l) => Ok(l),
            JsonPathValue::Node(_) => Ok(Self::True),
            JsonPathValue::Nothing => Ok(Self::False),
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
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum ValueType<'a> {
    /// This may come from a literal value declared in a JSONPath query, or be produced by a
    /// function.
    Value(Value),
    /// This would be a reference to a location in the JSON object being queried, i.e., the result
    /// of a singular query, or produced by a function.
    Node(&'a Value),
    /// This would be the result of a singular query that does not result in any nodes, or be
    /// produced by a function.
    #[default]
    Nothing,
}

impl<'a> ValueType<'a> {
    #[doc(hidden)]
    pub const fn json_path_type() -> JsonPathType {
        JsonPathType::Value
    }

    #[doc(hidden)]
    pub const fn function_type() -> FunctionArgType {
        FunctionArgType::Value
    }

    /// Convert to a reference of a [`serde_json::Value`] if possible
    pub fn as_value(&self) -> Option<&Value> {
        match self {
            ValueType::Value(v) => Some(v),
            ValueType::Node(v) => Some(v),
            ValueType::Nothing => None,
        }
    }

    /// Check if this `ValueType` is nothing
    pub fn is_nothing(&self) -> bool {
        matches!(self, ValueType::Nothing)
    }
}

impl<'a> TryFrom<JsonPathValue<'a>> for ValueType<'a> {
    type Error = ConversionError;

    fn try_from(value: JsonPathValue<'a>) -> Result<Self, Self::Error> {
        match value {
            JsonPathValue::Value(v) => Ok(Self::Value(v)),
            JsonPathValue::Node(n) => Ok(Self::Node(n)),
            JsonPathValue::Nothing => Ok(Self::Nothing),
            JsonPathValue::Nodes(_) => Err(ConversionError::IncompatibleTypes {
                from: JsonPathType::Nodes,
                to: JsonPathType::Value,
            }),
            JsonPathValue::Logical(_) => Err(ConversionError::IncompatibleTypes {
                from: JsonPathType::Nodes,
                to: JsonPathType::Value,
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
pub enum JsonPathValue<'a> {
    Nodes(NodeList<'a>),
    Logical(LogicalType),
    Node(&'a Value),
    Value(Value),
    Nothing,
}

impl<'a> From<NodesType<'a>> for JsonPathValue<'a> {
    fn from(value: NodesType<'a>) -> Self {
        Self::Nodes(value.0)
    }
}

impl<'a> From<ValueType<'a>> for JsonPathValue<'a> {
    fn from(value: ValueType<'a>) -> Self {
        match value {
            ValueType::Value(v) => Self::Value(v),
            ValueType::Node(n) => Self::Node(n),
            ValueType::Nothing => Self::Nothing,
        }
    }
}

impl<'a> From<LogicalType> for JsonPathValue<'a> {
    fn from(value: LogicalType) -> Self {
        Self::Logical(value)
    }
}

#[doc(hidden)]
/// Error used to convey JSONPath queries that are not well-typed
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    /// Cannot convert `from` into `to`
    #[error("attempted to convert {from} to {to}")]
    IncompatibleTypes {
        /// The type being converted from
        from: JsonPathType,
        /// The type being converted to
        to: JsonPathType,
    },
    /// Literal values can not be considered nodes
    #[error("cannot convert a literal value to NodesType")]
    LiteralToNodes,
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JsonPathType {
    Nodes,
    Value,
    Logical,
}

impl std::fmt::Display for JsonPathType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonPathType::Nodes => write!(f, "NodesType"),
            JsonPathType::Logical => write!(f, "LogicalType"),
            JsonPathType::Value => write!(f, "ValueType"),
        }
    }
}

#[doc(hidden)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionExpr<V> {
    pub name: String,
    pub args: Vec<FunctionExprArg>,
    pub return_type: FunctionArgType,
    pub validated: V,
}

#[doc(hidden)]
#[derive(Debug)]
pub struct NotValidated;

#[doc(hidden)]
#[derive(Clone)]
pub struct Validated {
    pub evaluator: &'static Evaluator,
}

impl PartialEq for Validated {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for Validated {}

impl std::fmt::Debug for Validated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Validated")
            .field("evaluator", &"static function")
            .finish()
    }
}

impl FunctionExpr<Validated> {
    #[cfg_attr(
        feature = "trace",
        tracing::instrument(name = "Evaluate Function Expr", level = "trace", parent = None, ret)
    )]
    pub fn evaluate<'a, 'b: 'a>(
        &'a self,
        current: &'b Value,
        root: &'b Value,
    ) -> JsonPathValue<'_> {
        let args: VecDeque<JsonPathValue> = self
            .args
            .iter()
            .map(|a| a.evaluate(current, root))
            .collect();
        (self.validated.evaluator)(args)
    }
}

impl FunctionExpr<NotValidated> {
    pub fn validate(
        name: String,
        args: Vec<FunctionExprArg>,
    ) -> Result<FunctionExpr<Validated>, FunctionValidationError> {
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
        Err(FunctionValidationError::Undefined { name })
    }
}

impl<V> std::fmt::Display for FunctionExpr<V> {
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FunctionExprArg {
    Literal(Literal),
    SingularQuery(SingularQuery),
    FilterQuery(Query),
    LogicalExpr(LogicalOrExpr),
    FunctionExpr(FunctionExpr<Validated>),
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
    #[cfg_attr(
        feature = "trace",
        tracing::instrument(name = "Evaluate Function Arg", level = "trace", parent = None, ret)
    )]
    fn evaluate<'a, 'b: 'a>(&'a self, current: &'b Value, root: &'b Value) -> JsonPathValue<'a> {
        match self {
            FunctionExprArg::Literal(lit) => lit.into(),
            FunctionExprArg::SingularQuery(q) => match q.eval_query(current, root) {
                Some(n) => JsonPathValue::Node(n),
                None => JsonPathValue::Nothing,
            },
            FunctionExprArg::FilterQuery(q) => JsonPathValue::Nodes(q.query(current, root).into()),
            FunctionExprArg::LogicalExpr(l) => match l.test_filter(current, root) {
                true => JsonPathValue::Logical(LogicalType::True),
                false => JsonPathValue::Logical(LogicalType::False),
            },
            FunctionExprArg::FunctionExpr(f) => f.evaluate(current, root),
        }
    }

    #[cfg_attr(
        feature = "trace",
        tracing::instrument(name = "Function Arg As Type Kind", level = "trace", parent = None, ret)
    )]
    pub fn as_type_kind(&self) -> Result<FunctionArgType, FunctionValidationError> {
        match self {
            FunctionExprArg::Literal(_) => Ok(FunctionArgType::Literal),
            FunctionExprArg::SingularQuery(_) => Ok(FunctionArgType::SingularQuery),
            FunctionExprArg::FilterQuery(query) => {
                if query.is_singular() {
                    Ok(FunctionArgType::SingularQuery)
                } else {
                    Ok(FunctionArgType::Nodelist)
                }
            }
            FunctionExprArg::LogicalExpr(_) => Ok(FunctionArgType::Logical),
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

/// Function argument types
///
/// This is used to describe the type of a function argument to determine if it will be valid as a
/// parameter to the function it is being passed to.
///
/// The reason for having this type in addition to [`JsonPathType`] is that we need to have an
/// intermediate representation of arguments that are singular queries. This is because singular
/// queries can be used as an argument to both [`ValueType`] and [`NodesType`] parameters.
/// Therefore, we require a `Node` variant here to indicate that an argument may be converted into
/// either type of parameter.
#[doc(hidden)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FunctionArgType {
    /// Denotes a literal owned JSON value
    Literal,
    /// Denotes a borrowed JSON value from a singular query
    SingularQuery,
    /// Denotes a literal or borrowed JSON value, used to represent functions that return [`ValueType`]
    Value,
    /// Denotes a node list, either from a filter query argument, or a function that returns [`NodesType`]
    Nodelist,
    /// Denotes a logical, either from a logical expression, or from a function that returns [`LogicalType`]
    Logical,
}

impl std::fmt::Display for FunctionArgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionArgType::Literal => write!(f, "literal"),
            FunctionArgType::SingularQuery => write!(f, "singular query"),
            FunctionArgType::Value => write!(f, "value type"),
            FunctionArgType::Nodelist => write!(f, "nodes type"),
            FunctionArgType::Logical => write!(f, "logical type"),
        }
    }
}

impl FunctionArgType {
    pub fn converts_to(&self, json_path_type: JsonPathType) -> bool {
        matches!(
            (self, json_path_type),
            (
                FunctionArgType::Literal | FunctionArgType::Value,
                JsonPathType::Value
            ) | (
                FunctionArgType::SingularQuery,
                JsonPathType::Value | JsonPathType::Nodes | JsonPathType::Logical
            ) | (
                FunctionArgType::Nodelist,
                JsonPathType::Nodes | JsonPathType::Logical
            ) | (FunctionArgType::Logical, JsonPathType::Logical),
        )
    }
}

#[doc(hidden)]
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
    #[error("in function {name}, in argument position {position}, expected a type that converts to {expected}, received {received}")]
    MismatchTypeKind {
        /// Function name
        name: String,
        /// Expected type
        expected: JsonPathType,
        /// Received type
        received: FunctionArgType,
        /// Argument position
        position: usize,
    },
    #[error("function with incorrect return type used")]
    IncorrectFunctionReturnType,
}

impl TestFilter for FunctionExpr<Validated> {
    #[cfg_attr(
        feature = "trace",
        tracing::instrument(name = "Test Function Expr", level = "trace", parent = None, ret)
    )]
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool {
        match self.evaluate(current, root) {
            JsonPathValue::Nodes(nl) => !nl.is_empty(),
            JsonPathValue::Value(v) => v.test_filter(current, root),
            JsonPathValue::Logical(l) => l.into(),
            JsonPathValue::Node(n) => n.test_filter(current, root),
            JsonPathValue::Nothing => false,
        }
    }
}
