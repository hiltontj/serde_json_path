use serde_json::{Number, Value};

use crate::spec::{
    functions::{FunctionExpr, JsonPathType, ValueType},
    query::{Query, Queryable},
};

use super::{index::Index, name::Name};

pub trait TestFilter {
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool;
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

#[derive(Debug, PartialEq, Clone)]
pub struct Filter(pub LogicalOrExpr);

impl std::fmt::Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{expr}", expr = self.0)
    }
}

impl Queryable for Filter {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Query Filter", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
        if let Some(list) = current.as_array() {
            list.iter()
                .filter(|v| self.0.test_filter(v, root))
                .collect()
        } else if let Some(obj) = current.as_object() {
            obj.iter()
                .map(|(_, v)| v)
                .filter(|v| self.0.test_filter(v, root))
                .collect()
        } else {
            vec![]
        }
    }
}

/// The top level boolean expression type
///
/// This is also `boolean-expression` in the JSONPath specification, but the naming
/// was chosen to make it more clear that it represents the logical OR.
#[derive(Debug, PartialEq, Clone)]
pub struct LogicalOrExpr(pub Vec<LogicalAndExpr>);

impl std::fmt::Display for LogicalOrExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, expr) in self.0.iter().enumerate() {
            write!(
                f,
                "{expr}{logic}",
                logic = if i == self.0.len() - 1 { "" } else { " || " }
            )?;
        }
        Ok(())
    }
}

impl TestFilter for LogicalOrExpr {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Test Logical Or Expr", level = "trace", parent = None, ret))]
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool {
        self.0.iter().any(|expr| expr.test_filter(current, root))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LogicalAndExpr(pub Vec<BasicExpr>);

impl std::fmt::Display for LogicalAndExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, expr) in self.0.iter().enumerate() {
            write!(
                f,
                "{expr}{logic}",
                logic = if i == self.0.len() - 1 { "" } else { " && " }
            )?;
        }
        Ok(())
    }
}

impl TestFilter for LogicalAndExpr {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Test Logical And Expr", level = "trace", parent = None, ret))]
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool {
        self.0.iter().all(|expr| expr.test_filter(current, root))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BasicExpr {
    Paren(LogicalOrExpr),
    NotParen(LogicalOrExpr),
    Relation(ComparisonExpr),
    Exist(ExistExpr),
    NotExist(ExistExpr),
    FuncExpr(FunctionExpr),
    NotFuncExpr(FunctionExpr),
}

impl std::fmt::Display for BasicExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BasicExpr::Paren(expr) => write!(f, "({expr})"),
            BasicExpr::NotParen(expr) => write!(f, "!({expr})"),
            BasicExpr::Relation(rel) => write!(f, "{rel}"),
            BasicExpr::Exist(exist) => write!(f, "{exist}"),
            BasicExpr::NotExist(exist) => write!(f, "!{exist}"),
            BasicExpr::FuncExpr(expr) => write!(f, "{expr}"),
            BasicExpr::NotFuncExpr(expr) => write!(f, "{expr}"),
        }
    }
}

impl BasicExpr {
    pub fn as_relation(&self) -> Option<&ComparisonExpr> {
        match self {
            BasicExpr::Relation(cx) => Some(cx),
            _ => None,
        }
    }
}

impl TestFilter for BasicExpr {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Test Basic Expr", level = "trace", parent = None, ret))]
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool {
        match self {
            BasicExpr::Paren(expr) => expr.test_filter(current, root),
            BasicExpr::NotParen(expr) => !expr.test_filter(current, root),
            BasicExpr::Relation(expr) => expr.test_filter(current, root),
            BasicExpr::Exist(expr) => expr.test_filter(current, root),
            BasicExpr::NotExist(expr) => !expr.test_filter(current, root),
            BasicExpr::FuncExpr(expr) => expr.test_filter(current, root),
            BasicExpr::NotFuncExpr(expr) => !expr.test_filter(current, root),
        }
    }
}

/// Existence expression
///
/// ### Implementation Note
///
/// This does not support the function expression notation outlined in the JSONPath spec.
#[derive(Debug, PartialEq, Clone)]
pub struct ExistExpr(pub Query);

impl std::fmt::Display for ExistExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{query}", query = self.0)
    }
}

impl TestFilter for ExistExpr {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Test Exists Expr", level = "trace", parent = None, ret))]
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool {
        !self.0.query(current, root).is_empty()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ComparisonExpr {
    pub left: Comparable,
    pub op: ComparisonOperator,
    pub right: Comparable,
}

fn func_type_equal_to(left: &JsonPathType, right: &JsonPathType) -> bool {
    match (left, right) {
        (JsonPathType::Value(l), JsonPathType::Value(r)) => match (l, r) {
            (ValueType::Value(v1), ValueType::Value(v2)) => v1 == v2,
            (ValueType::Value(v1), ValueType::ValueRef(v2)) => v1 == *v2,
            (ValueType::ValueRef(v1), ValueType::Value(v2)) => *v1 == v2,
            (ValueType::ValueRef(v1), ValueType::ValueRef(v2)) => v1 == v2,
            (ValueType::Nothing, ValueType::Nothing) => true,
            _ => false,
        },
        _ => false,
    }
}

fn value_less_than(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(n1), Value::Number(n2)) => number_less_than(n1, n2),
        (Value::String(s1), Value::String(s2)) => s1 < s2,
        _ => false,
    }
}

fn func_type_less_than(left: &JsonPathType, right: &JsonPathType) -> bool {
    match (left, right) {
        (JsonPathType::Value(l), JsonPathType::Value(r)) => match (l, r) {
            (ValueType::Value(v1), ValueType::Value(v2)) => value_less_than(v1, v2),
            (ValueType::Value(v1), ValueType::ValueRef(v2)) => value_less_than(v1, v2),
            (ValueType::ValueRef(v1), ValueType::Value(v2)) => value_less_than(v1, v2),
            (ValueType::ValueRef(v1), ValueType::ValueRef(v2)) => value_less_than(v1, v2),
            _ => false,
        },
        _ => false,
    }
}

fn value_same_type(left: &Value, right: &Value) -> bool {
    matches!((left, right), (Value::Null, Value::Null))
        | matches!((left, right), (Value::Bool(_), Value::Bool(_)))
        | matches!((left, right), (Value::Number(_), Value::Number(_)))
        | matches!((left, right), (Value::String(_), Value::String(_)))
        | matches!((left, right), (Value::Array(_), Value::Array(_)))
        | matches!((left, right), (Value::Object(_), Value::Object(_)))
}

fn func_type_same_type(left: &JsonPathType, right: &JsonPathType) -> bool {
    match (left, right) {
        (JsonPathType::Value(l), JsonPathType::Value(r)) => match (l, r) {
            (ValueType::Value(v1), ValueType::Value(v2)) => value_same_type(v1, v2),
            (ValueType::Value(v1), ValueType::ValueRef(v2)) => value_same_type(v1, v2),
            (ValueType::ValueRef(v1), ValueType::Value(v2)) => value_same_type(v1, v2),
            (ValueType::ValueRef(v1), ValueType::ValueRef(v2)) => value_same_type(v1, v2),
            _ => false,
        },
        _ => false,
    }
}

impl std::fmt::Display for ComparisonExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{left}{op}{right}",
            left = self.left,
            op = self.op,
            right = self.right
        )
    }
}

impl TestFilter for ComparisonExpr {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Test Comparison Expr", level = "trace", parent = None, ret))]
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool {
        use ComparisonOperator::*;
        let left = self.left.as_value(current, root);
        let right = self.right.as_value(current, root);
        match self.op {
            EqualTo => func_type_equal_to(&left, &right),
            NotEqualTo => !func_type_equal_to(&left, &right),
            LessThan => func_type_same_type(&left, &right) && func_type_less_than(&left, &right),
            GreaterThan => {
                func_type_same_type(&left, &right)
                    && !func_type_less_than(&left, &right)
                    && !func_type_equal_to(&left, &right)
            }
            LessThanEqualTo => {
                func_type_same_type(&left, &right)
                    && (func_type_less_than(&left, &right) || func_type_equal_to(&left, &right))
            }
            GreaterThanEqualTo => {
                func_type_same_type(&left, &right) && !func_type_less_than(&left, &right)
            }
        }
    }
}

fn number_less_than(n1: &Number, n2: &Number) -> bool {
    if let (Some(a), Some(b)) = (n1.as_f64(), n2.as_f64()) {
        a < b
    } else if let (Some(a), Some(b)) = (n1.as_i64(), n2.as_i64()) {
        a < b
    } else if let (Some(a), Some(b)) = (n1.as_u64(), n2.as_u64()) {
        a < b
    } else {
        false
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ComparisonOperator {
    EqualTo,
    NotEqualTo,
    LessThan,
    GreaterThan,
    LessThanEqualTo,
    GreaterThanEqualTo,
}

impl std::fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonOperator::EqualTo => write!(f, "=="),
            ComparisonOperator::NotEqualTo => write!(f, "!="),
            ComparisonOperator::LessThan => write!(f, "<"),
            ComparisonOperator::GreaterThan => write!(f, ">"),
            ComparisonOperator::LessThanEqualTo => write!(f, "<="),
            ComparisonOperator::GreaterThanEqualTo => write!(f, ">="),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Comparable {
    Primitive {
        kind: ComparablePrimitiveKind,
        value: Value,
    },
    SingularPath(SingularPath),
    FunctionExpr(FunctionExpr),
}

impl std::fmt::Display for Comparable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Comparable::Primitive { kind: _, value } => write!(f, "{value}"),
            Comparable::SingularPath(path) => write!(f, "{path}"),
            Comparable::FunctionExpr(expr) => write!(f, "{expr}"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ComparablePrimitiveKind {
    Number,
    String,
    Bool,
    Null,
}

impl Comparable {
    pub fn as_value<'a, 'b: 'a>(&'a self, current: &'b Value, root: &'b Value) -> JsonPathType<'a> {
        use Comparable::*;
        match self {
            Primitive { kind: _, value } => JsonPathType::Value(ValueType::ValueRef(value)),
            SingularPath(sp) => match sp.eval_path(current, root) {
                Some(v) => JsonPathType::Value(ValueType::ValueRef(v)),
                None => JsonPathType::Value(ValueType::Nothing),
            },
            FunctionExpr(expr) => expr.evaluate(current, root),
        }
    }
}

impl Comparable {
    pub fn is_null(&self) -> bool {
        match self {
            Comparable::Primitive { kind, value }
                if matches!(kind, ComparablePrimitiveKind::Null) =>
            {
                value.is_null()
            }
            _ => false,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Comparable::Primitive { kind, value }
                if matches!(kind, ComparablePrimitiveKind::Bool) =>
            {
                value.as_bool()
            }
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Comparable::Primitive { kind, value }
                if matches!(kind, ComparablePrimitiveKind::String) =>
            {
                value.as_str()
            }
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Comparable::Primitive { kind, value }
                if matches!(kind, ComparablePrimitiveKind::Number) =>
            {
                value.as_i64()
            }
            _ => None,
        }
    }

    pub fn as_singular_path(&self) -> Option<&SingularPath> {
        match self {
            Comparable::SingularPath(sp) => Some(sp),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SingularPathSegment {
    Name(Name),
    Index(Index),
}

impl std::fmt::Display for SingularPathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SingularPathSegment::Name(name) => write!(f, "{name}"),
            SingularPathSegment::Index(index) => write!(f, "{index}"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SingularPath {
    pub kind: SingularPathKind,
    pub segments: Vec<SingularPathSegment>,
}

impl SingularPath {
    pub fn eval_path<'b>(&self, current: &'b Value, root: &'b Value) -> Option<&'b Value> {
        let mut target = match self.kind {
            SingularPathKind::Absolute => root,
            SingularPathKind::Relative => current,
        };
        for segment in &self.segments {
            match segment {
                SingularPathSegment::Name(name) => {
                    if let Some(v) = target.as_object() {
                        if let Some(t) = v.get(name.as_str()) {
                            target = t;
                        } else {
                            return None;
                        }
                    }
                }
                SingularPathSegment::Index(index) => {
                    if let Some(l) = target.as_array() {
                        if let Some(t) = usize::try_from(index.0).ok().and_then(|i| l.get(i)) {
                            target = t;
                        } else {
                            return None;
                        }
                    }
                }
            }
        }
        Some(target)
    }
}

impl std::fmt::Display for SingularPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            SingularPathKind::Absolute => write!(f, "$")?,
            SingularPathKind::Relative => write!(f, "@")?,
        }
        for s in &self.segments {
            write!(f, "[{s}]")?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SingularPathKind {
    Absolute,
    Relative,
}
