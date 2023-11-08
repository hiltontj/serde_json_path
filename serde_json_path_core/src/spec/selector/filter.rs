//! Types representing filter selectors in JSONPath
use serde_json::{Number, Value};

use crate::spec::{
    functions::{FunctionExpr, JsonPathValue, Validated},
    query::{Query, QueryKind, Queryable},
    segment::{QuerySegment, Segment},
};

use super::{index::Index, name::Name, Selector};

mod sealed {
    use serde_json::Value;

    use crate::spec::functions::FunctionExpr;

    use super::{BasicExpr, ComparisonExpr, ExistExpr, LogicalAndExpr, LogicalOrExpr};

    pub trait Sealed {}
    impl Sealed for Value {}
    impl Sealed for LogicalOrExpr {}
    impl Sealed for LogicalAndExpr {}
    impl Sealed for BasicExpr {}
    impl Sealed for ExistExpr {}
    impl Sealed for ComparisonExpr {}
    impl<V> Sealed for FunctionExpr<V> {}
}

/// Trait for testing a filter type
pub trait TestFilter: sealed::Sealed {
    /// Test self using the current and root nodes
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

/// The main filter type for JSONPath
#[derive(Debug, PartialEq, Eq, Clone)]
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
/// This is also `ligical-expression` in the JSONPath specification, but the naming was chosen to
/// make it more clear that it represents the logical OR, and to not have an extra wrapping type.
#[derive(Debug, PartialEq, Eq, Clone)]
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

/// A logical AND expression
#[derive(Debug, PartialEq, Eq, Clone)]
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

/// The basic for m of expression in a filter
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BasicExpr {
    /// An expression wrapped in parenthesis
    Paren(LogicalOrExpr),
    /// A parenthesized expression preceded with a `!`
    NotParen(LogicalOrExpr),
    /// A relationship expression which compares two JSON values
    Relation(ComparisonExpr),
    /// An existence expression
    Exist(ExistExpr),
    /// The inverse of an existence expression, i.e., preceded by `!`
    NotExist(ExistExpr),
    /// A function expression
    FuncExpr(FunctionExpr<Validated>),
    /// The inverse of a function expression, i.e., preceded by `!`
    NotFuncExpr(FunctionExpr<Validated>),
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
    /// Optionally express as a relation expression
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
#[derive(Debug, PartialEq, Eq, Clone)]
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

/// A comparison expression comparing two JSON values
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ComparisonExpr {
    /// The JSON value on the left of the comparison
    pub left: Comparable,
    /// The operator of comparison
    pub op: ComparisonOperator,
    /// The JSON value on the right of the comparison
    pub right: Comparable,
}

fn check_equal_to(left: &JsonPathValue, right: &JsonPathValue) -> bool {
    match (left, right) {
        (JsonPathValue::Node(v1), JsonPathValue::Node(v2)) => value_equal_to(v1, v2),
        (JsonPathValue::Node(v1), JsonPathValue::Value(v2)) => value_equal_to(v1, v2),
        (JsonPathValue::Value(v1), JsonPathValue::Node(v2)) => value_equal_to(v1, v2),
        (JsonPathValue::Value(v1), JsonPathValue::Value(v2)) => value_equal_to(v1, v2),
        (JsonPathValue::Nothing, JsonPathValue::Nothing) => true,
        _ => false,
    }
}

fn value_equal_to(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(l), Value::Number(r)) => number_equal_to(l, r),
        _ => left == right,
    }
}

fn number_equal_to(left: &Number, right: &Number) -> bool {
    if let (Some(l), Some(r)) = (left.as_f64(), right.as_f64()) {
        l == r
    } else if let (Some(l), Some(r)) = (left.as_i64(), right.as_i64()) {
        l == r
    } else if let (Some(l), Some(r)) = (left.as_u64(), right.as_u64()) {
        l == r
    } else {
        false
    }
}

fn value_less_than(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(n1), Value::Number(n2)) => number_less_than(n1, n2),
        (Value::String(s1), Value::String(s2)) => s1 < s2,
        _ => false,
    }
}

fn check_less_than(left: &JsonPathValue, right: &JsonPathValue) -> bool {
    match (left, right) {
        (JsonPathValue::Node(v1), JsonPathValue::Node(v2)) => value_less_than(v1, v2),
        (JsonPathValue::Node(v1), JsonPathValue::Value(v2)) => value_less_than(v1, v2),
        (JsonPathValue::Value(v1), JsonPathValue::Node(v2)) => value_less_than(v1, v2),
        (JsonPathValue::Value(v1), JsonPathValue::Value(v2)) => value_less_than(v1, v2),
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

fn check_same_type(left: &JsonPathValue, right: &JsonPathValue) -> bool {
    match (left, right) {
        (JsonPathValue::Node(v1), JsonPathValue::Node(v2)) => value_same_type(v1, v2),
        (JsonPathValue::Node(v1), JsonPathValue::Value(v2)) => value_same_type(v1, v2),
        (JsonPathValue::Value(v1), JsonPathValue::Node(v2)) => value_same_type(v1, v2),
        (JsonPathValue::Value(v1), JsonPathValue::Value(v2)) => value_same_type(v1, v2),
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
        let left = self.left.as_value(current, root);
        let right = self.right.as_value(current, root);
        match self.op {
            ComparisonOperator::EqualTo => check_equal_to(&left, &right),
            ComparisonOperator::NotEqualTo => !check_equal_to(&left, &right),
            ComparisonOperator::LessThan => {
                check_same_type(&left, &right) && check_less_than(&left, &right)
            }
            ComparisonOperator::GreaterThan => {
                check_same_type(&left, &right)
                    && !check_less_than(&left, &right)
                    && !check_equal_to(&left, &right)
            }
            ComparisonOperator::LessThanEqualTo => {
                check_same_type(&left, &right)
                    && (check_less_than(&left, &right) || check_equal_to(&left, &right))
            }
            ComparisonOperator::GreaterThanEqualTo => {
                check_same_type(&left, &right) && !check_less_than(&left, &right)
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

/// The comparison operator
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ComparisonOperator {
    /// `==`
    EqualTo,
    /// `!=`
    NotEqualTo,
    /// `<`
    LessThan,
    /// `>`
    GreaterThan,
    /// `<=`
    LessThanEqualTo,
    /// `>=`
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

/// A type that is comparable
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Comparable {
    /// A literal JSON value, excluding objects and arrays
    Literal(Literal),
    /// A singular query
    ///
    /// This will only produce a single node, i.e., JSON value, or nothing
    SingularQuery(SingularQuery),
    /// A function expression that can only produce a `ValueType`
    FunctionExpr(FunctionExpr<Validated>),
}

impl std::fmt::Display for Comparable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Comparable::Literal(lit) => write!(f, "{lit}"),
            Comparable::SingularQuery(path) => write!(f, "{path}"),
            Comparable::FunctionExpr(expr) => write!(f, "{expr}"),
        }
    }
}

impl Comparable {
    #[doc(hidden)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Comparable::as_value", level = "trace", parent = None, ret))]
    pub fn as_value<'a, 'b: 'a>(
        &'a self,
        current: &'b Value,
        root: &'b Value,
    ) -> JsonPathValue<'a> {
        match self {
            Comparable::Literal(lit) => lit.into(),
            Comparable::SingularQuery(sp) => match sp.eval_query(current, root) {
                Some(v) => JsonPathValue::Node(v),
                None => JsonPathValue::Nothing,
            },
            Comparable::FunctionExpr(expr) => expr.evaluate(current, root),
        }
    }

    #[doc(hidden)]
    pub fn as_singular_path(&self) -> Option<&SingularQuery> {
        match self {
            Comparable::SingularQuery(sp) => Some(sp),
            _ => None,
        }
    }
}

/// A literal JSON value that can be represented in a JSONPath query
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literal {
    /// A valid JSON number
    Number(Number),
    /// A string
    String(String),
    /// `true` or `false`
    Bool(bool),
    /// `null`
    Null,
}

impl<'a> From<&'a Literal> for JsonPathValue<'a> {
    fn from(value: &'a Literal) -> Self {
        match value {
            // Cloning here seems cheap, certainly for numbers, but it may not be desireable for
            // strings.
            Literal::Number(n) => JsonPathValue::Value(n.to_owned().into()),
            Literal::String(s) => JsonPathValue::Value(s.to_owned().into()),
            Literal::Bool(b) => JsonPathValue::Value(Value::from(*b)),
            Literal::Null => JsonPathValue::Value(Value::Null),
        }
    }
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Number(n) => write!(f, "{n}"),
            Literal::String(s) => write!(f, "'{s}'"),
            Literal::Bool(b) => write!(f, "{b}"),
            Literal::Null => write!(f, "null"),
        }
    }
}

/// A segment in a singular query
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SingularQuerySegment {
    /// A single name segment
    Name(Name),
    /// A single index segment
    Index(Index),
}

impl std::fmt::Display for SingularQuerySegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SingularQuerySegment::Name(name) => write!(f, "{name}"),
            SingularQuerySegment::Index(index) => write!(f, "{index}"),
        }
    }
}

impl TryFrom<QuerySegment> for SingularQuerySegment {
    type Error = NonSingularQueryError;

    fn try_from(segment: QuerySegment) -> Result<Self, Self::Error> {
        if segment.is_descendent() {
            return Err(NonSingularQueryError::Descendant);
        }
        match segment.segment {
            Segment::LongHand(mut selectors) => {
                if selectors.len() > 1 {
                    Err(NonSingularQueryError::TooManySelectors)
                } else if let Some(sel) = selectors.pop() {
                    sel.try_into()
                } else {
                    Err(NonSingularQueryError::NoSelectors)
                }
            }
            Segment::DotName(name) => Ok(Self::Name(Name(name))),
            Segment::Wildcard => Err(NonSingularQueryError::Wildcard),
        }
    }
}

impl TryFrom<Selector> for SingularQuerySegment {
    type Error = NonSingularQueryError;

    fn try_from(selector: Selector) -> Result<Self, Self::Error> {
        match selector {
            Selector::Name(n) => Ok(Self::Name(n)),
            Selector::Wildcard => Err(NonSingularQueryError::Wildcard),
            Selector::Index(i) => Ok(Self::Index(i)),
            Selector::ArraySlice(_) => Err(NonSingularQueryError::Slice),
            Selector::Filter(_) => Err(NonSingularQueryError::Filter),
        }
    }
}

/// Represents a singular query in JSONPath
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SingularQuery {
    /// The kind of singular query, relative or absolute
    pub kind: SingularQueryKind,
    /// The segments making up the query
    pub segments: Vec<SingularQuerySegment>,
}

impl SingularQuery {
    /// Evaluate the singular query
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SingularQuery::eval_query", level = "trace", parent = None, ret))]
    pub fn eval_query<'b>(&self, current: &'b Value, root: &'b Value) -> Option<&'b Value> {
        let mut target = match self.kind {
            SingularQueryKind::Absolute => root,
            SingularQueryKind::Relative => current,
        };
        for segment in &self.segments {
            match segment {
                SingularQuerySegment::Name(name) => {
                    if let Some(t) = target.as_object().and_then(|o| o.get(name.as_str())) {
                        target = t;
                    } else {
                        return None;
                    }
                }
                SingularQuerySegment::Index(index) => {
                    if let Some(t) = target
                        .as_array()
                        .and_then(|l| usize::try_from(index.0).ok().and_then(|i| l.get(i)))
                    {
                        target = t;
                    } else {
                        return None;
                    }
                }
            }
        }
        Some(target)
    }
}

impl TryFrom<Query> for SingularQuery {
    type Error = NonSingularQueryError;

    fn try_from(query: Query) -> Result<Self, Self::Error> {
        let kind = SingularQueryKind::from(query.kind);
        let segments = query
            .segments
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<Vec<SingularQuerySegment>, Self::Error>>()?;
        Ok(Self { kind, segments })
    }
}

impl Queryable for SingularQuery {
    fn query<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
        match self.eval_query(current, root) {
            Some(v) => vec![v],
            None => vec![],
        }
    }
}

impl std::fmt::Display for SingularQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            SingularQueryKind::Absolute => write!(f, "$")?,
            SingularQueryKind::Relative => write!(f, "@")?,
        }
        for s in &self.segments {
            write!(f, "[{s}]")?;
        }
        Ok(())
    }
}

/// The kind of singular query
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SingularQueryKind {
    /// Referencing the root node, i.e., `$`
    Absolute,
    /// Referencing the current node, i.e., `@`
    Relative,
}

impl From<QueryKind> for SingularQueryKind {
    fn from(qk: QueryKind) -> Self {
        match qk {
            QueryKind::Root => Self::Absolute,
            QueryKind::Current => Self::Relative,
        }
    }
}

/// Error when parsing a singular query
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum NonSingularQueryError {
    /// Descendant segment
    #[error("descendant segments are not singular")]
    Descendant,
    /// Long hand segment with too many internal selectors
    #[error("long hand segment contained more than one selector")]
    TooManySelectors,
    /// Long hand segment with no selectors
    #[error("long hand segment contained no selectors")]
    NoSelectors,
    /// A wildcard segment
    #[error("wildcard segments are not singular")]
    Wildcard,
    /// A slice segment
    #[error("slice segments are not singular")]
    Slice,
    /// A filter segment
    #[error("filter segments are not singular")]
    Filter,
}
