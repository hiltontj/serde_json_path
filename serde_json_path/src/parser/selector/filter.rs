use nom::character::complete::{char, multispace0};
use nom::combinator::{map, map_res};
use nom::multi::separated_list1;
use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};
use nom::{branch::alt, bytes::complete::tag, combinator::value};
use serde_json_path_core::spec::functions::{
    FunctionArgType, FunctionExpr, FunctionValidationError, Validated,
};
use serde_json_path_core::spec::selector::filter::{
    BasicExpr, Comparable, ComparisonExpr, ComparisonOperator, ExistExpr, Filter, Literal,
    LogicalAndExpr, LogicalOrExpr, SingularQuery,
};

use super::function::parse_function_expr;
use crate::parser::primitive::number::parse_number;
use crate::parser::primitive::string::parse_string_literal;
use crate::parser::primitive::{parse_bool, parse_null};
use crate::parser::utils::uncut;
use crate::parser::{parse_query, PResult};

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_filter(input: &str) -> PResult<Filter> {
    map(
        preceded(pair(char('?'), multispace0), parse_logical_or_expr),
        Filter,
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_logical_and(input: &str) -> PResult<LogicalAndExpr> {
    map(
        separated_list1(
            tuple((multispace0, tag("&&"), multispace0)),
            parse_basic_expr,
        ),
        LogicalAndExpr,
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_logical_or_expr(input: &str) -> PResult<LogicalOrExpr> {
    map(
        separated_list1(
            tuple((multispace0, tag("||"), multispace0)),
            parse_logical_and,
        ),
        LogicalOrExpr,
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_exist_expr_inner(input: &str) -> PResult<ExistExpr> {
    map(parse_query, ExistExpr)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_exist_expr(input: &str) -> PResult<BasicExpr> {
    map(parse_exist_expr_inner, BasicExpr::Exist)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_not_exist_expr(input: &str) -> PResult<BasicExpr> {
    map(
        preceded(pair(char('!'), multispace0), parse_exist_expr_inner),
        BasicExpr::NotExist,
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_func_expr_inner(input: &str) -> PResult<FunctionExpr<Validated>> {
    map_res(parse_function_expr, |fe| match fe.return_type {
        FunctionArgType::Logical | FunctionArgType::Nodelist => Ok(fe),
        _ => Err(FunctionValidationError::IncorrectFunctionReturnType),
    })(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_func_expr(input: &str) -> PResult<BasicExpr> {
    map(parse_func_expr_inner, BasicExpr::FuncExpr)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_not_func_expr(input: &str) -> PResult<BasicExpr> {
    map(
        preceded(pair(char('!'), multispace0), parse_func_expr_inner),
        BasicExpr::NotFuncExpr,
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_paren_expr_inner(input: &str) -> PResult<LogicalOrExpr> {
    delimited(
        pair(char('('), multispace0),
        parse_logical_or_expr,
        pair(multispace0, char(')')),
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_paren_expr(input: &str) -> PResult<BasicExpr> {
    map(parse_paren_expr_inner, BasicExpr::Paren)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_not_parent_expr(input: &str) -> PResult<BasicExpr> {
    map(
        preceded(pair(char('!'), multispace0), parse_paren_expr_inner),
        BasicExpr::NotParen,
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_basic_expr(input: &str) -> PResult<BasicExpr> {
    alt((
        parse_not_parent_expr,
        parse_paren_expr,
        map(parse_comp_expr, BasicExpr::Relation),
        parse_not_exist_expr,
        parse_exist_expr,
        parse_not_func_expr,
        parse_func_expr,
    ))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_comp_expr(input: &str) -> PResult<ComparisonExpr> {
    map(
        separated_pair(
            parse_comparable,
            multispace0,
            separated_pair(parse_comparison_operator, multispace0, parse_comparable),
        ),
        |(left, (op, right))| ComparisonExpr { left, op, right },
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_comparison_operator(input: &str) -> PResult<ComparisonOperator> {
    alt((
        value(ComparisonOperator::EqualTo, tag("==")),
        value(ComparisonOperator::NotEqualTo, tag("!=")),
        value(ComparisonOperator::LessThanEqualTo, tag("<=")),
        value(ComparisonOperator::GreaterThanEqualTo, tag(">=")),
        value(ComparisonOperator::LessThan, char('<')),
        value(ComparisonOperator::GreaterThan, char('>')),
    ))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_literal(input: &str) -> PResult<Literal> {
    alt((
        map(parse_string_literal, Literal::String),
        map(parse_number, Literal::Number),
        map(parse_bool, Literal::Bool),
        value(Literal::Null, parse_null),
    ))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_literal_comparable(input: &str) -> PResult<Comparable> {
    map(parse_literal, Comparable::Literal)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_singular_path(input: &str) -> PResult<SingularQuery> {
    map_res(parse_query, |q| q.try_into())(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_singular_path_comparable(input: &str) -> PResult<Comparable> {
    map(parse_singular_path, Comparable::SingularQuery)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_function_expr_comparable(input: &str) -> PResult<Comparable> {
    map_res(parse_function_expr, |fe| {
        match fe.return_type {
            FunctionArgType::Value => Ok(fe),
            _ => Err(FunctionValidationError::IncorrectFunctionReturnType),
        }
        .map(Comparable::FunctionExpr)
    })(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_comparable(input: &str) -> PResult<Comparable> {
    uncut(alt((
        parse_literal_comparable,
        parse_singular_path_comparable,
        parse_function_expr_comparable,
    )))(input)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "trace")]
    use test_log::test;

    use serde_json::Number;
    use serde_json_path_core::spec::selector::filter::{Comparable, Literal, SingularQuerySegment};

    use crate::parser::selector::{
        filter::{parse_literal, ComparisonOperator},
        Index, Name,
    };

    use super::{parse_basic_expr, parse_comp_expr, parse_comparable};

    #[test]
    fn literals() {
        {
            let (_, lit) = parse_literal("null").unwrap();
            assert!(matches!(lit, Literal::Null));
        }
        {
            let (_, lit) = parse_literal("true").unwrap();
            assert!(matches!(lit, Literal::Bool(true)));
        }
        {
            let (_, lit) = parse_literal("false").unwrap();
            assert!(matches!(lit, Literal::Bool(false)));
        }
        {
            let (_, lit) = parse_literal("\"test\"").unwrap();
            assert!(matches!(lit, Literal::String(s) if s == "test"));
        }
        {
            let (_, lit) = parse_literal("'test'").unwrap();
            assert!(matches!(lit, Literal::String(s) if s == "test"));
        }
        {
            let (_, lit) = parse_literal("123").unwrap();
            assert!(matches!(lit, Literal::Number(n) if n == Number::from(123)));
        }
    }

    #[test]
    fn comp_expr() {
        // TODO - test more
        let (_, cxp) = parse_comp_expr("true != false").unwrap();
        assert!(matches!(cxp.left, Comparable::Literal(Literal::Bool(true))));
        assert!(matches!(cxp.op, ComparisonOperator::NotEqualTo));
        assert!(matches!(
            cxp.right,
            Comparable::Literal(Literal::Bool(false))
        ));
    }

    #[test]
    fn basic_expr() {
        let (_, bxp) = parse_basic_expr("true == true").unwrap();
        let cx = bxp.as_relation().unwrap();
        assert!(matches!(cx.left, Comparable::Literal(Literal::Bool(true))));
        assert!(matches!(cx.right, Comparable::Literal(Literal::Bool(true))));
        assert!(matches!(cx.op, ComparisonOperator::EqualTo));
    }

    #[test]
    fn singular_path_comparables() {
        {
            let (_, cmp) = parse_comparable("@.name").unwrap();
            let sp = &cmp.as_singular_path().unwrap().segments;
            assert!(matches!(&sp[0], SingularQuerySegment::Name(Name(s)) if s == "name"));
        }
        {
            let (_, cmp) = parse_comparable("$.data[0].id").unwrap();
            let sp = &cmp.as_singular_path().unwrap().segments;
            assert!(matches!(&sp[0], SingularQuerySegment::Name(Name(s)) if s == "data"));
            assert!(matches!(&sp[1], SingularQuerySegment::Index(Index(i)) if i == &0));
            assert!(matches!(&sp[2], SingularQuerySegment::Name(Name(s)) if s == "id"));
        }
    }
}
