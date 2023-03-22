use serde_json::{Number, Value};

use crate::spec::{
    functions::{FunctionExpr, JsonPathType},
    query::{Query, QueryKind, Queryable},
    segment::{QuerySegment, Segment},
};

use super::{index::Index, name::Name, Selector};

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

fn check_equal_to(left: &JsonPathType, right: &JsonPathType) -> bool {
    match (left, right) {
        (JsonPathType::Node(v1), JsonPathType::Node(v2)) => v1 == v2,
        (JsonPathType::Node(v1), JsonPathType::Value(v2)) => *v1 == v2,
        (JsonPathType::Node(v1), JsonPathType::ValueRef(v2)) => v1 == v2,
        (JsonPathType::Value(v1), JsonPathType::Node(v2)) => v1 == *v2,
        (JsonPathType::Value(v1), JsonPathType::Value(v2)) => v1 == v2,
        (JsonPathType::Value(v1), JsonPathType::ValueRef(v2)) => v1 == *v2,
        (JsonPathType::ValueRef(v1), JsonPathType::Node(v2)) => v1 == v2,
        (JsonPathType::ValueRef(v1), JsonPathType::Value(v2)) => *v1 == v2,
        (JsonPathType::ValueRef(v1), JsonPathType::ValueRef(v2)) => v1 == v2,
        (JsonPathType::Nothing, JsonPathType::Nothing) => true,
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

fn check_less_than(left: &JsonPathType, right: &JsonPathType) -> bool {
    match (left, right) {
        (JsonPathType::Node(v1), JsonPathType::Node(v2)) => value_less_than(v1, v2),
        (JsonPathType::Node(v1), JsonPathType::Value(v2)) => value_less_than(v1, v2),
        (JsonPathType::Node(v1), JsonPathType::ValueRef(v2)) => value_less_than(v1, v2),
        (JsonPathType::Value(v1), JsonPathType::Node(v2)) => value_less_than(v1, v2),
        (JsonPathType::Value(v1), JsonPathType::Value(v2)) => value_less_than(v1, v2),
        (JsonPathType::Value(v1), JsonPathType::ValueRef(v2)) => value_less_than(v1, v2),
        (JsonPathType::ValueRef(v1), JsonPathType::Node(v2)) => value_less_than(v1, v2),
        (JsonPathType::ValueRef(v1), JsonPathType::Value(v2)) => value_less_than(v1, v2),
        (JsonPathType::ValueRef(v1), JsonPathType::ValueRef(v2)) => value_less_than(v1, v2),
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

fn check_same_type(left: &JsonPathType, right: &JsonPathType) -> bool {
    match (left, right) {
        (JsonPathType::Node(v1), JsonPathType::Node(v2)) => value_same_type(v1, v2),
        (JsonPathType::Node(v1), JsonPathType::Value(v2)) => value_same_type(v1, v2),
        (JsonPathType::Node(v1), JsonPathType::ValueRef(v2)) => value_same_type(v1, v2),
        (JsonPathType::Value(v1), JsonPathType::Node(v2)) => value_same_type(v1, v2),
        (JsonPathType::Value(v1), JsonPathType::Value(v2)) => value_same_type(v1, v2),
        (JsonPathType::Value(v1), JsonPathType::ValueRef(v2)) => value_same_type(v1, v2),
        (JsonPathType::ValueRef(v1), JsonPathType::Node(v2)) => value_same_type(v1, v2),
        (JsonPathType::ValueRef(v1), JsonPathType::Value(v2)) => value_same_type(v1, v2),
        (JsonPathType::ValueRef(v1), JsonPathType::ValueRef(v2)) => value_same_type(v1, v2),
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
            EqualTo => check_equal_to(&left, &right),
            NotEqualTo => !check_equal_to(&left, &right),
            LessThan => check_same_type(&left, &right) && check_less_than(&left, &right),
            GreaterThan => {
                check_same_type(&left, &right)
                    && !check_less_than(&left, &right)
                    && !check_equal_to(&left, &right)
            }
            LessThanEqualTo => {
                check_same_type(&left, &right)
                    && (check_less_than(&left, &right) || check_equal_to(&left, &right))
            }
            GreaterThanEqualTo => check_same_type(&left, &right) && !check_less_than(&left, &right),
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
    Literal(Literal),
    SingularPath(SingularQuery),
    FunctionExpr(FunctionExpr),
}

impl std::fmt::Display for Comparable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Comparable::Literal(lit) => write!(f, "{lit}"),
            Comparable::SingularPath(path) => write!(f, "{path}"),
            Comparable::FunctionExpr(expr) => write!(f, "{expr}"),
        }
    }
}

impl Comparable {
    pub fn as_value<'a, 'b: 'a>(&'a self, current: &'b Value, root: &'b Value) -> JsonPathType<'a> {
        match self {
            Comparable::Literal(lit) => lit.into(),
            Comparable::SingularPath(sp) => match sp.eval_path(current, root) {
                Some(v) => JsonPathType::Node(v),
                None => JsonPathType::Nothing,
            },
            Comparable::FunctionExpr(expr) => expr.evaluate(current, root),
        }
    }

    pub fn as_singular_path(&self) -> Option<&SingularQuery> {
        match self {
            Comparable::SingularPath(sp) => Some(sp),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Number(Number),
    String(String),
    Bool(bool),
    Null,
}

impl<'a> From<&'a Literal> for JsonPathType<'a> {
    fn from(value: &'a Literal) -> Self {
        match value {
            // Cloning here seems cheap, certainly for numbers, but it may not be desireable for
            // strings.
            Literal::Number(n) => JsonPathType::Value(n.to_owned().into()),
            Literal::String(s) => JsonPathType::Value(s.to_owned().into()),
            Literal::Bool(b) => JsonPathType::Value(Value::from(*b)),
            Literal::Null => JsonPathType::Value(Value::Null),
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

#[derive(Debug, PartialEq, Clone)]
pub enum SingularQuerySegment {
    Name(Name),
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
            return Err(NonSingularQueryError);
        }
        match segment.segment {
            Segment::LongHand(mut selectors) => {
                if selectors.len() > 1 {
                    Err(NonSingularQueryError)
                } else if let Some(sel) = selectors.pop() {
                    sel.try_into()
                } else {
                    Err(NonSingularQueryError)
                }
            }
            Segment::DotName(name) => Ok(Self::Name(Name(name))),
            Segment::Wildcard => Err(NonSingularQueryError),
        }
    }
}

impl TryFrom<Selector> for SingularQuerySegment {
    type Error = NonSingularQueryError;

    fn try_from(selector: Selector) -> Result<Self, Self::Error> {
        match selector {
            Selector::Name(n) => Ok(Self::Name(n)),
            Selector::Wildcard => Err(NonSingularQueryError),
            Selector::Index(i) => Ok(Self::Index(i)),
            Selector::ArraySlice(_) => Err(NonSingularQueryError),
            Selector::Filter(_) => Err(NonSingularQueryError),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SingularQuery {
    pub kind: SingularQueryKind,
    pub segments: Vec<SingularQuerySegment>,
}

impl SingularQuery {
    pub fn eval_path<'b>(&self, current: &'b Value, root: &'b Value) -> Option<&'b Value> {
        let mut target = match self.kind {
            SingularQueryKind::Absolute => root,
            SingularQueryKind::Relative => current,
        };
        for segment in &self.segments {
            match segment {
                SingularQuerySegment::Name(name) => {
                    if let Some(v) = target.as_object() {
                        if let Some(t) = v.get(name.as_str()) {
                            target = t;
                        } else {
                            return None;
                        }
                    }
                }
                SingularQuerySegment::Index(index) => {
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
        match self.eval_path(current, root) {
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SingularQueryKind {
    Absolute,
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

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("expected singular query")]
pub struct NonSingularQueryError;
