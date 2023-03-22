use std::collections::VecDeque;

use once_cell::sync::Lazy;
use serde_json::Value;

use crate::{node::NodeList, spec::query::Queryable};

use super::{
    query::Query,
    selector::filter::{Literal, LogicalOrExpr, SingularQuery, TestFilter},
};

pub type Validator =
    Lazy<Box<dyn Fn(&[FunctionExprArg]) -> Result<(), FunctionValidationError> + Send + Sync>>;

pub type Evaluator =
    Lazy<Box<dyn for<'a> Fn(VecDeque<JsonPathType<'a>>) -> JsonPathType<'a> + Sync + Send>>;

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

impl<'a> NodesType<'a> {
    pub const fn type_kind() -> JsonPathTypeKind {
        JsonPathTypeKind::Nodelist
    }

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
            // TODO - is this correct or even necessary? ValueType does not coalesce into NodesType
            JsonPathType::Node(n) => Ok(Self(vec![n].into())),
            JsonPathType::ValueRef(_) => Err(ConversionError::LiteralToNodes),
            JsonPathType::Nothing => Ok(Self(vec![].into())),
        }
    }
}

impl<'a> AsTypeKind for NodesType<'a> {
    fn as_type_kind(&self) -> JsonPathTypeKind {
        JsonPathTypeKind::Nodelist
    }
}

#[derive(Debug, Default)]
pub enum LogicalType {
    True,
    #[default]
    False,
}

impl LogicalType {
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
            // TODO - is this correct, given that only NodesType coalesces to Logical...
            JsonPathType::Node(_) => Ok(Self::True),
            JsonPathType::ValueRef(_) => Err(ConversionError::IncompatibleTypes {
                from: JsonPathTypeKind::ValueRef,
                to: JsonPathTypeKind::Logical,
            }),
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

impl AsTypeKind for LogicalType {
    fn as_type_kind(&self) -> JsonPathTypeKind {
        JsonPathTypeKind::Logical
    }
}

#[derive(Debug)]
pub enum ValueType<'a> {
    Value(Value),
    ValueRef(&'a Value),
    Node(&'a Value),
    Nothing,
}

impl<'a> ValueType<'a> {
    pub const fn type_kind() -> JsonPathTypeKind {
        JsonPathTypeKind::Value
    }

    pub fn as_value(&self) -> Option<&Value> {
        match self {
            ValueType::Value(v) => Some(v),
            ValueType::ValueRef(v) => Some(v),
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
            JsonPathType::ValueRef(vr) => Ok(Self::ValueRef(vr)),
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

// impl<'a> From<Option<&'a Value>> for ValueType<'a> {
//     fn from(value: Option<&'a Value>) -> Self {
//         match value {
//             Some(v) => Self::ValueRef(v),
//             None => Self::Nothing,
//         }
//     }
// }

impl<'a, T> From<T> for ValueType<'a>
where
    T: Into<Value>,
{
    fn from(value: T) -> Self {
        Self::Value(value.into())
    }
}

impl<'a> AsTypeKind for ValueType<'a> {
    fn as_type_kind(&self) -> JsonPathTypeKind {
        match self {
            ValueType::Value(_) => JsonPathTypeKind::Value,
            ValueType::ValueRef(_) => JsonPathTypeKind::ValueRef,
            ValueType::Node(_) => JsonPathTypeKind::Node,
            ValueType::Nothing => JsonPathTypeKind::Nothing,
        }
    }
}

#[derive(Debug)]
pub enum JsonPathType<'a> {
    Nodes(NodeList<'a>),
    Logical(LogicalType),
    Node(&'a Value),
    Value(Value),
    ValueRef(&'a Value),
    Nothing,
}

impl<'a> JsonPathType<'a> {
    pub fn as_kind(&self) -> JsonPathTypeKind {
        match self {
            JsonPathType::Nodes(_) => JsonPathTypeKind::Nodelist,
            JsonPathType::Value(_) => JsonPathTypeKind::Value,
            JsonPathType::Logical(_) => JsonPathTypeKind::Logical,
            JsonPathType::Node(_) => JsonPathTypeKind::Node,
            JsonPathType::ValueRef(_) => JsonPathTypeKind::ValueRef,
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
            ValueType::ValueRef(vr) => Self::ValueRef(vr),
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

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("attempted to convert {from} to {to}")]
    IncompatibleTypes {
        from: JsonPathTypeKind,
        to: JsonPathTypeKind,
    },
    #[error("cannot use a literal value in place of NodesType")]
    LiteralToNodes,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JsonPathTypeKind {
    Nodelist,
    Node,
    Value,
    ValueRef,
    Logical,
    Nothing,
}

impl JsonPathTypeKind {
    pub fn converts_to(&self, other: Self) -> bool {
        matches!(
            (self, other),
            (JsonPathTypeKind::Nodelist, JsonPathTypeKind::Nodelist)
                | (JsonPathTypeKind::Nodelist, JsonPathTypeKind::Logical)
                // | (JsonPathTypeKind::Node, JsonPathTypeKind::Nodelist)
                | (JsonPathTypeKind::Node, JsonPathTypeKind::Node)
                | (JsonPathTypeKind::Node, JsonPathTypeKind::Value)
                | (JsonPathTypeKind::Node, JsonPathTypeKind::ValueRef)
                | (JsonPathTypeKind::Value, JsonPathTypeKind::Node)
                | (JsonPathTypeKind::Value, JsonPathTypeKind::Value)
                | (JsonPathTypeKind::Value, JsonPathTypeKind::ValueRef)
                | (JsonPathTypeKind::ValueRef, JsonPathTypeKind::Node)
                | (JsonPathTypeKind::ValueRef, JsonPathTypeKind::Value)
                | (JsonPathTypeKind::ValueRef, JsonPathTypeKind::ValueRef)
                | (JsonPathTypeKind::Logical, JsonPathTypeKind::Logical)
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
            JsonPathTypeKind::ValueRef => write!(f, "ValueType"),
            JsonPathTypeKind::Nothing => write!(f, "ValueType"),
        }
    }
}

pub trait AsTypeKind {
    fn as_type_kind(&self) -> JsonPathTypeKind;
}

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
        use FunctionExprArg::*;
        match self {
            Literal(lit) => lit.into(),
            SingularQuery(q) => match q.eval_path(current, root) {
                Some(n) => JsonPathType::Node(n),
                None => JsonPathType::Nothing,
            },
            FilterQuery(q) => JsonPathType::Nodes(q.query(current, root).into()),
            LogicalExpr(l) => match l.test_filter(current, root) {
                true => JsonPathType::Logical(LogicalType::True),
                false => JsonPathType::Logical(LogicalType::False),
            },
            FunctionExpr(f) => f.evaluate(current, root),
        }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(name = "Function Arg As Type Kind", level = "trace", parent = None, ret))]
    pub fn as_type_kind(&self) -> Result<JsonPathTypeKind, FunctionValidationError> {
        use FunctionExprArg::*;
        match self {
            // TODO - is this a case that the type distinction is no longer needed??
            Literal(_) => Ok(JsonPathTypeKind::Value),
            SingularQuery(_) => Ok(JsonPathTypeKind::Node),
            FilterQuery(query) => {
                if query.is_singular() {
                    Ok(JsonPathTypeKind::Node)
                } else {
                    Ok(JsonPathTypeKind::Nodelist)
                }
            },
            LogicalExpr(_) => Ok(JsonPathTypeKind::Logical),
            FunctionExpr(func) => {
                for f in inventory::iter::<Function> {
                    if f.name == func.name.as_str() {
                        return Ok(f.result_type);
                    }
                }
                Err(FunctionValidationError::Undefined {
                    name: func.name.to_owned(),
                })
            }
            // Comparable(comp) => match comp {
            //     super::selector::filter::Comparable::Literal(_) => Ok(JsonPathTypeKind::ValueRef),
            //     super::selector::filter::Comparable::SingularPath(_) => Ok(JsonPathTypeKind::Value),
            //     super::selector::filter::Comparable::FunctionExpr(func) => {
            //     }
            // },
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum FunctionValidationError {
    #[error("function name '{name}' is not defined")]
    Undefined { name: String },
    #[error("expected {expected} args, but received {received}")]
    NumberOfArgsMismatch { expected: usize, received: usize },
    #[error("in argument position {position}, expected a type that converts to {expected}, received {received}")]
    MismatchTypeKind {
        expected: JsonPathTypeKind,
        received: JsonPathTypeKind,
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
            JsonPathType::ValueRef(vr) => vr.test_filter(current, root),
            JsonPathType::Nothing => false,
        }
    }
}
