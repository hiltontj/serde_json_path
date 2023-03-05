use nom::character::complete::{char, space0};
use nom::combinator::map;
use nom::multi::{many0, separated_list1};
use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};
use nom::{branch::alt, bytes::complete::tag, combinator::value};
use serde_json::{Number, Value};

use crate::parser::primitive::number::parse_number;
use crate::parser::primitive::string::parse_string_literal;
use crate::parser::primitive::{parse_bool, parse_null};
use crate::parser::segment::parse_dot_member_name;
use crate::parser::{parse_path, PResult, Query, Queryable};

use super::function::{parse_function_expr, FunctionExpr};
use super::{parse_index, parse_name, Index, Name};

pub trait TestFilter {
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool;
}

#[derive(Debug, PartialEq)]
pub struct Filter(LogicalOrExpr);

impl Queryable for Filter {
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

pub fn parse_filter(input: &str) -> PResult<Filter> {
    map(
        preceded(pair(char('?'), space0), parse_logical_or_expr),
        Filter,
    )(input)
}

/// The top level boolean expression type
///
/// This is also `boolean-expression` in the JSONPath specification, but the naming
/// was chosen to make it more clear that it represents the logical OR.
#[derive(Debug, PartialEq)]
struct LogicalOrExpr(Vec<LogicalAndExpr>);

impl TestFilter for LogicalOrExpr {
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool {
        self.0.iter().any(|expr| expr.test_filter(current, root))
    }
}

#[derive(Debug, PartialEq)]
struct LogicalAndExpr(Vec<BasicExpr>);

impl TestFilter for LogicalAndExpr {
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool {
        self.0.iter().all(|expr| expr.test_filter(current, root))
    }
}

fn parse_logical_and(input: &str) -> PResult<LogicalAndExpr> {
    map(
        separated_list1(tuple((space0, tag("&&"), space0)), parse_basic_expr),
        LogicalAndExpr,
    )(input)
}

fn parse_logical_or_expr(input: &str) -> PResult<LogicalOrExpr> {
    map(
        separated_list1(tuple((space0, tag("||"), space0)), parse_logical_and),
        LogicalOrExpr,
    )(input)
}

#[derive(Debug, PartialEq)]
enum BasicExpr {
    Paren(LogicalOrExpr),
    NotParen(LogicalOrExpr),
    Relation(ComparisonExpr),
    Exist(ExistExpr),
    NotExist(ExistExpr),
    FuncExpr(FunctionExpr),
    NotFuncExpr(FunctionExpr),
}

#[cfg(test)]
impl BasicExpr {
    pub fn as_relation(&self) -> Option<&ComparisonExpr> {
        match self {
            BasicExpr::Relation(cx) => Some(cx),
            _ => None,
        }
    }
}

impl TestFilter for BasicExpr {
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
#[derive(Debug, PartialEq)]
struct ExistExpr(Query);

impl TestFilter for ExistExpr {
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool {
        !self.0.query(current, root).is_empty()
    }
}

fn parse_exist_expr_inner(input: &str) -> PResult<ExistExpr> {
    map(parse_path, ExistExpr)(input)
}

fn parse_exist_expr(input: &str) -> PResult<BasicExpr> {
    map(parse_exist_expr_inner, BasicExpr::Exist)(input)
}

fn parse_not_exist_expr(input: &str) -> PResult<BasicExpr> {
    map(
        preceded(pair(char('!'), space0), parse_exist_expr_inner),
        BasicExpr::NotExist,
    )(input)
}

fn parse_func_expr(input: &str) -> PResult<BasicExpr> {
    map(parse_function_expr, BasicExpr::FuncExpr)(input)
}

fn parse_not_func_expr(input: &str) -> PResult<BasicExpr> {
    map(
        preceded(pair(char('!'), space0), parse_function_expr),
        BasicExpr::NotFuncExpr,
    )(input)
}

fn parse_paren_expr_inner(input: &str) -> PResult<LogicalOrExpr> {
    delimited(
        pair(char('('), space0),
        parse_logical_or_expr,
        pair(space0, char(')')),
    )(input)
}

fn parse_paren_expr(input: &str) -> PResult<BasicExpr> {
    map(parse_paren_expr_inner, BasicExpr::Paren)(input)
}

fn parse_not_parent_expr(input: &str) -> PResult<BasicExpr> {
    map(
        preceded(pair(char('!'), space0), parse_paren_expr_inner),
        BasicExpr::NotParen,
    )(input)
}

fn parse_basic_expr(input: &str) -> PResult<BasicExpr> {
    alt((
        parse_not_parent_expr,
        parse_paren_expr,
        map(parse_comp_expr, BasicExpr::Relation),
        parse_not_exist_expr,
        parse_exist_expr,
        parse_func_expr,
        parse_not_func_expr,
    ))(input)
}

#[derive(Debug, PartialEq)]
struct ComparisonExpr {
    pub left: Comparable,
    pub op: ComparisonOperator,
    pub right: Comparable,
}

impl TestFilter for ComparisonExpr {
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool {
        use ComparisonOperator::*;
        let left = self.left.as_value(current, root);
        let right = self.right.as_value(current, root);
        match (left, right) {
            (None, None) => true,
            (Some(l), Some(r)) => match self.op {
                EqualTo => l == r,
                NotEqualTo => l != r,
                LessThan => match (l, r) {
                    (Value::Number(n1), Value::Number(n2)) => number_less_than(n1, n2),
                    (Value::String(s1), Value::String(s2)) => s1 < s2,
                    _ => false,
                },
                GreaterThan => match (l, r) {
                    (Value::Number(n1), Value::Number(n2)) => !number_less_than(n1, n2) && n1 != n2,
                    (Value::String(s1), Value::String(s2)) => s1 > s2,
                    _ => false,
                },
                LessThanEqualTo => match (l, r) {
                    (Value::Number(n1), Value::Number(n2)) => number_less_than(n1, n2) || n1 == n2,
                    (Value::String(s1), Value::String(s2)) => s1 <= s2,
                    (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
                    (Value::Null, Value::Null) => true,
                    _ => false,
                },
                GreaterThanEqualTo => match (l, r) {
                    (Value::Number(n1), Value::Number(n2)) => !number_less_than(n1, n2),
                    (Value::String(s1), Value::String(s2)) => s1 >= s2,
                    (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
                    (Value::Null, Value::Null) => true,
                    _ => false,
                },
            },
            (None, Some(_)) | (Some(_), None) => matches!(self.op, NotEqualTo),
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

fn parse_comp_expr(input: &str) -> PResult<ComparisonExpr> {
    map(
        separated_pair(
            parse_comparable,
            space0,
            separated_pair(parse_comparison_operator, space0, parse_comparable),
        ),
        |(left, (op, right))| ComparisonExpr { left, op, right },
    )(input)
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum ComparisonOperator {
    EqualTo,
    NotEqualTo,
    LessThan,
    GreaterThan,
    LessThanEqualTo,
    GreaterThanEqualTo,
}

fn parse_comparison_operator(input: &str) -> PResult<ComparisonOperator> {
    use ComparisonOperator::*;
    alt((
        value(EqualTo, tag("==")),
        value(NotEqualTo, tag("!=")),
        value(LessThanEqualTo, tag("<=")),
        value(GreaterThanEqualTo, tag(">=")),
        value(LessThan, char('<')),
        value(GreaterThan, char('>')),
    ))(input)
}

#[derive(Debug, PartialEq)]
pub enum Comparable {
    Primitive {
        kind: ComparablePrimitiveKind,
        value: Value,
    },
    SingularPath(SingularPath),
    // FunctionExpr, // TODO - function expressions are hard
}

#[derive(Debug, PartialEq)]
pub enum ComparablePrimitiveKind {
    Number,
    String,
    Bool,
    Null,
}

impl Comparable {
    pub fn as_value<'a, 'b: 'a>(&'a self, current: &'b Value, root: &'b Value) -> Option<&Value> {
        use Comparable::*;
        match self {
            Primitive { kind: _, value } => Some(value),
            SingularPath(sp) => sp.eval_path(current, root),
        }
    }
}

#[cfg(test)]
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

fn parse_null_comparable(input: &str) -> PResult<Comparable> {
    map(parse_null, |_| Comparable::Primitive {
        kind: ComparablePrimitiveKind::Null,
        value: Value::Null,
    })(input)
}

fn parse_bool_comparable(input: &str) -> PResult<Comparable> {
    map(parse_bool, |b| Comparable::Primitive {
        kind: ComparablePrimitiveKind::Bool,
        value: Value::Bool(b),
    })(input)
}

fn parse_number_comparable(input: &str) -> PResult<Comparable> {
    map(parse_number, |n| Comparable::Primitive {
        kind: ComparablePrimitiveKind::Number,
        value: Value::Number(n),
    })(input)
}

fn parse_string_comparable(input: &str) -> PResult<Comparable> {
    map(parse_string_literal, |s| Comparable::Primitive {
        kind: ComparablePrimitiveKind::String,
        value: Value::String(s),
    })(input)
}

#[derive(Debug, PartialEq)]
pub enum SingularPathSegment {
    Name(Name),
    Index(Index),
}

fn parse_singular_path_index_segment(input: &str) -> PResult<Index> {
    delimited(char('['), parse_index, char(']'))(input)
}

fn parse_singular_path_name_segment(input: &str) -> PResult<Name> {
    alt((
        delimited(char('['), parse_name, char(']')),
        map(preceded(char('.'), parse_dot_member_name), Name),
    ))(input)
}

fn parse_singular_path_segments(input: &str) -> PResult<Vec<SingularPathSegment>> {
    many0(preceded(
        space0,
        alt((
            map(parse_singular_path_name_segment, SingularPathSegment::Name),
            map(
                parse_singular_path_index_segment,
                SingularPathSegment::Index,
            ),
        )),
    ))(input)
}

#[derive(Debug, PartialEq)]
pub struct SingularPath {
    kind: SingularPathKind,
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

#[derive(Debug, PartialEq)]
enum SingularPathKind {
    Absolute,
    Relative,
}

fn parse_singular_path(input: &str) -> PResult<SingularPath> {
    alt((
        map(
            preceded(char('$'), parse_singular_path_segments),
            |segments| SingularPath {
                kind: SingularPathKind::Absolute,
                segments,
            },
        ),
        map(
            preceded(char('@'), parse_singular_path_segments),
            |segments| SingularPath {
                kind: SingularPathKind::Relative,
                segments,
            },
        ),
    ))(input)
}

fn parse_singular_path_comparable(input: &str) -> PResult<Comparable> {
    map(parse_singular_path, Comparable::SingularPath)(input)
}

pub fn parse_comparable(input: &str) -> PResult<Comparable> {
    alt((
        parse_null_comparable,
        parse_bool_comparable,
        parse_number_comparable,
        parse_string_comparable,
        parse_singular_path_comparable,
    ))(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::selector::{
        filter::{ComparisonOperator, SingularPathSegment},
        Index, Name,
    };

    use super::{parse_basic_expr, parse_comp_expr, parse_comparable};

    #[test]
    fn primitive_comparables() {
        {
            let (_, cmp) = parse_comparable("null").unwrap();
            dbg!(&cmp);
            assert!(cmp.is_null());
        }
        {
            let (_, cmp) = parse_comparable("true").unwrap();
            assert!(cmp.as_bool().unwrap());
        }
        {
            let (_, cmp) = parse_comparable("false").unwrap();
            assert!(!cmp.as_bool().unwrap());
        }
        {
            let (_, cmp) = parse_comparable("\"test\"").unwrap();
            assert_eq!(cmp.as_str().unwrap(), "test");
        }
        {
            let (_, cmp) = parse_comparable("'test'").unwrap();
            assert_eq!(cmp.as_str().unwrap(), "test");
        }
        {
            let (_, cmp) = parse_comparable("123").unwrap();
            assert_eq!(cmp.as_i64().unwrap(), 123);
        }
    }

    #[test]
    fn comp_expr() {
        // TODO - test more
        let (_, cxp) = parse_comp_expr("true != false").unwrap();
        assert!(cxp.left.as_bool().unwrap());
        assert!(matches!(cxp.op, ComparisonOperator::NotEqualTo));
        assert!(!cxp.right.as_bool().unwrap());
    }

    #[test]
    fn basic_expr() {
        let (_, bxp) = parse_basic_expr("true == true").unwrap();
        let cx = bxp.as_relation().unwrap();
        assert!(cx.left.as_bool().unwrap());
        assert!(cx.right.as_bool().unwrap());
        assert!(matches!(cx.op, ComparisonOperator::EqualTo));
    }

    #[test]
    fn singular_path_comparables() {
        {
            let (_, cmp) = parse_comparable("@.name").unwrap();
            let sp = &cmp.as_singular_path().unwrap().segments;
            assert!(matches!(&sp[0], SingularPathSegment::Name(Name(s)) if s == "name"));
        }
        {
            let (_, cmp) = parse_comparable("$.data[0].id").unwrap();
            let sp = &cmp.as_singular_path().unwrap().segments;
            assert!(matches!(&sp[0], SingularPathSegment::Name(Name(s)) if s == "data"));
            assert!(matches!(&sp[1], SingularPathSegment::Index(Index(i)) if i == &0));
            assert!(matches!(&sp[2], SingularPathSegment::Name(Name(s)) if s == "id"));
        }
    }
}
