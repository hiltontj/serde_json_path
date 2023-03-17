use nom::character::complete::{char, space0};
use nom::combinator::map;
use nom::multi::{many0, separated_list1};
use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};
use nom::{branch::alt, bytes::complete::tag, combinator::value};
use serde_json::Value;
use serde_json_path_core::spec::selector::filter::{
    BasicExpr, Comparable, ComparablePrimitiveKind, ComparisonExpr, ComparisonOperator, ExistExpr,
    Filter, LogicalAndExpr, LogicalOrExpr, SingularPath, SingularPathKind, SingularPathSegment,
};

use super::function::parse_function_expr;
use super::{parse_index, parse_name, Index, Name};
use crate::parser::primitive::number::parse_number;
use crate::parser::primitive::string::parse_string_literal;
use crate::parser::primitive::{parse_bool, parse_null};
use crate::parser::segment::parse_dot_member_name;
use crate::parser::{parse_path, PResult};

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub fn parse_filter(input: &str) -> PResult<Filter> {
    map(
        preceded(pair(char('?'), space0), parse_logical_or_expr),
        Filter,
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_logical_and(input: &str) -> PResult<LogicalAndExpr> {
    map(
        separated_list1(tuple((space0, tag("&&"), space0)), parse_basic_expr),
        LogicalAndExpr,
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_logical_or_expr(input: &str) -> PResult<LogicalOrExpr> {
    map(
        separated_list1(tuple((space0, tag("||"), space0)), parse_logical_and),
        LogicalOrExpr,
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_exist_expr_inner(input: &str) -> PResult<ExistExpr> {
    map(parse_path, ExistExpr)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_exist_expr(input: &str) -> PResult<BasicExpr> {
    map(parse_exist_expr_inner, BasicExpr::Exist)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_not_exist_expr(input: &str) -> PResult<BasicExpr> {
    map(
        preceded(pair(char('!'), space0), parse_exist_expr_inner),
        BasicExpr::NotExist,
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_func_expr(input: &str) -> PResult<BasicExpr> {
    map(parse_function_expr, BasicExpr::FuncExpr)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_not_func_expr(input: &str) -> PResult<BasicExpr> {
    map(
        preceded(pair(char('!'), space0), parse_function_expr),
        BasicExpr::NotFuncExpr,
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_paren_expr_inner(input: &str) -> PResult<LogicalOrExpr> {
    delimited(
        pair(char('('), space0),
        parse_logical_or_expr,
        pair(space0, char(')')),
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_paren_expr(input: &str) -> PResult<BasicExpr> {
    map(parse_paren_expr_inner, BasicExpr::Paren)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_not_parent_expr(input: &str) -> PResult<BasicExpr> {
    map(
        preceded(pair(char('!'), space0), parse_paren_expr_inner),
        BasicExpr::NotParen,
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_basic_expr(input: &str) -> PResult<BasicExpr> {
    alt((
        parse_not_parent_expr,
        parse_paren_expr,
        map(parse_comp_expr, BasicExpr::Relation),
        parse_func_expr,
        parse_not_func_expr,
        parse_not_exist_expr,
        parse_exist_expr,
    ))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
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

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
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

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_null_comparable(input: &str) -> PResult<Comparable> {
    map(parse_null, |_| Comparable::Primitive {
        kind: ComparablePrimitiveKind::Null,
        value: Value::Null,
    })(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_bool_comparable(input: &str) -> PResult<Comparable> {
    map(parse_bool, |b| Comparable::Primitive {
        kind: ComparablePrimitiveKind::Bool,
        value: Value::Bool(b),
    })(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_number_comparable(input: &str) -> PResult<Comparable> {
    map(parse_number, |n| Comparable::Primitive {
        kind: ComparablePrimitiveKind::Number,
        value: Value::Number(n),
    })(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_string_comparable(input: &str) -> PResult<Comparable> {
    map(parse_string_literal, |s| Comparable::Primitive {
        kind: ComparablePrimitiveKind::String,
        value: Value::String(s),
    })(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_singular_path_index_segment(input: &str) -> PResult<Index> {
    delimited(char('['), parse_index, char(']'))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_singular_path_name_segment(input: &str) -> PResult<Name> {
    alt((
        delimited(char('['), parse_name, char(']')),
        map(preceded(char('.'), parse_dot_member_name), Name),
    ))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
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

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
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

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_singular_path_comparable(input: &str) -> PResult<Comparable> {
    map(parse_singular_path, Comparable::SingularPath)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_function_expr_comparable(input: &str) -> PResult<Comparable> {
    map(parse_function_expr, Comparable::FunctionExpr)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub fn parse_comparable(input: &str) -> PResult<Comparable> {
    alt((
        parse_null_comparable,
        parse_bool_comparable,
        parse_number_comparable,
        parse_string_comparable,
        parse_singular_path_comparable,
        parse_function_expr_comparable,
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
