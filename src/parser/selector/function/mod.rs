use nom::{branch::alt, combinator::map, sequence::{pair, delimited}, character::complete::{space0, satisfy}, multi::{many0, fold_many1}};
use nom::character::complete::char;
use once_cell::sync::Lazy;
use serde_json::Value;

pub mod registry;

use crate::parser::{Query, PResult, parse_path, Queryable};

use super::filter::{Comparable, TestFilter, parse_comparable};

pub type Evaluator = Lazy<Box<dyn for<'a> Fn(Vec<FuncType<'a, 'a>>) -> FuncType<'a, 'a> + Sync + Send>>;

pub struct Function {
    name: &'static str,
    evaluator: &'static Evaluator,
}

impl Function {
    pub const fn new(name: &'static str, evaluator: &'static Evaluator) -> Self {
        Self { name, evaluator }
    }
}

inventory::collect!(Function);

pub enum FuncType<'a, 'b> {
    Nodelist(Vec<&'b Value>),
    Node(Option<&'a Value>),
    Value(Value),
    Nothing,
}

#[derive(Debug, PartialEq)]
pub struct FunctionExpr {
    name: String,
    args: Vec<FunctionExprArg>,
}

#[derive(Debug, PartialEq)]
pub enum FunctionExprArg {
    FilterPath(Query),
    Comparable(Comparable),
}

impl FunctionExprArg {
    fn evaluate<'a, 'b: 'a>(&'a self, current: &'b Value, root: &'b Value) -> FuncType<'a, 'b> {
        use FunctionExprArg::*;
        match self {
            FilterPath(q) => FuncType::Nodelist(q.query(current, root)),
            Comparable(c) => FuncType::Node(c.as_value(current, root)),
        }
    }
}

impl FunctionExpr {
    fn evaluate<'a, 'b: 'a>(&'a self, current: &'b Value, root: &'b Value) -> FuncType<'_, '_> {
        let args: Vec<FuncType> = self.args.iter().map(|a| a.evaluate(current, root)).collect();
        for f in inventory::iter::<Function> {
            if f.name == self.name {
                return (f.evaluator)(args);
            }
        }
        return FuncType::Nothing;
    }
}

impl TestFilter for FunctionExpr {
    fn test_filter<'b>(&self, current: &'b Value, root: &'b Value) -> bool {
        match self.evaluate(current, root) {
            FuncType::Nodelist(nl) => !nl.is_empty(),
            FuncType::Node(n) => n.is_some(),
            FuncType::Value(v) => match v {
                Value::Null => false,
                Value::Bool(b) => b,
                // TODO - probably wrong:
                _ => true,
            },
            FuncType::Nothing => false,
        }
    }
}

fn parse_function_name_first(input: &str) -> PResult<char> {
    satisfy(|c| ('a'..='z').contains(&c))(input)
}

fn parse_function_name_char(input: &str) -> PResult<char> {
    alt((
        parse_function_name_first,
        char('_'),
        satisfy(|c| ('0'..='9').contains(&c)),
    ))(input)
}

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

fn parse_function_argument(input: &str) -> PResult<FunctionExprArg> {
    alt((
        map(parse_comparable, FunctionExprArg::Comparable),
        map(parse_path, FunctionExprArg::FilterPath),
    ))(input)
}

pub fn parse_function_expr(input: &str) -> PResult<FunctionExpr> {
    map(
        pair(
            parse_function_name,
            delimited(
                pair(char('('), space0),
                many0(parse_function_argument),
                pair(space0, char(')')),
            )
        ),
        |(name, args)| FunctionExpr { name, args }
    )(input)
}

// #[test]
// mod tests {
//     #[test]
//     fn length_fn() {
        
//     }
// }