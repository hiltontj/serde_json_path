use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::map;
use nom::error::context;
use serde_json::Value;

use self::filter::{parse_filter, Filter};
use self::slice::parse_array_slice;
use self::slice::Slice;

use super::primitive::int::parse_int;
use super::primitive::string::parse_string_literal;
use super::{PResult, Queryable};

pub mod filter;
pub mod function;
pub mod slice;

#[derive(Debug, PartialEq, Clone)]
pub enum Selector {
    Name(Name),
    Wildcard,
    Index(Index),
    ArraySlice(Slice),
    Filter(Filter),
}

impl std::fmt::Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Selector::Name(name) => write!(f, "{name}"),
            Selector::Wildcard => write!(f, "*"),
            Selector::Index(index) => write!(f, "{index}"),
            Selector::ArraySlice(slice) => write!(f, "{slice}"),
            Selector::Filter(filter) => write!(f, "?{filter}"),
        }
    }
}

impl Queryable for Selector {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Query Selector", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
        let mut query = Vec::new();
        match self {
            Selector::Name(name) => query.append(&mut name.query(current, root)),
            Selector::Wildcard => {
                if let Some(list) = current.as_array() {
                    for v in list {
                        query.push(v);
                    }
                } else if let Some(obj) = current.as_object() {
                    for (_, v) in obj {
                        query.push(v);
                    }
                }
            }
            Selector::Index(index) => query.append(&mut index.query(current, root)),
            Selector::ArraySlice(slice) => query.append(&mut slice.query(current, root)),
            Selector::Filter(filter) => query.append(&mut filter.query(current, root)),
        }
        query
    }
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub fn parse_wildcard_selector(input: &str) -> PResult<Selector> {
    map(char('*'), |_| Selector::Wildcard)(input)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Name(pub String);

impl Name {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{name}'", name = self.0)
    }
}

impl Queryable for Name {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Query Name", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, _root: &'b Value) -> Vec<&'b Value> {
        if let Some(obj) = current.as_object() {
            obj.get(&self.0).into_iter().collect()
        } else {
            vec![]
        }
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub fn parse_name(input: &str) -> PResult<Name> {
    map(parse_string_literal, Name)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_name_selector(input: &str) -> PResult<Selector> {
    map(parse_name, Selector::Name)(input)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Index(pub isize);

impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{index}", index = self.0)
    }
}

impl Queryable for Index {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Query Index", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, _root: &'b Value) -> Vec<&'b Value> {
        if let Some(list) = current.as_array() {
            if self.0 < 0 {
                self.0
                    .checked_abs()
                    .and_then(|i| usize::try_from(i).ok())
                    .and_then(|i| list.len().checked_sub(i))
                    .and_then(|i| list.get(i))
                    .into_iter()
                    .collect()
            } else {
                usize::try_from(self.0)
                    .ok()
                    .and_then(|i| list.get(i))
                    .into_iter()
                    .collect()
            }
        } else {
            vec![]
        }
    }
}

impl From<isize> for Index {
    fn from(i: isize) -> Self {
        Self(i)
    }
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_index(input: &str) -> PResult<Index> {
    map(parse_int, Index)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_index_selector(input: &str) -> PResult<Selector> {
    map(parse_index, Selector::Index)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_array_slice_selector(input: &str) -> PResult<Selector> {
    map(parse_array_slice, Selector::ArraySlice)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_filter_selector(input: &str) -> PResult<Selector> {
    map(parse_filter, Selector::Filter)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub fn parse_selector(input: &str) -> PResult<Selector> {
    context(
        "selector",
        alt((
            parse_wildcard_selector,
            parse_name_selector,
            parse_array_slice_selector,
            parse_index_selector,
            parse_filter_selector,
        )),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::selector::{slice::Slice, Name};

    use super::{parse_selector, parse_wildcard_selector, Index, Selector};

    #[test]
    fn wildcard() {
        assert!(matches!(
            parse_wildcard_selector("*"),
            Ok(("", Selector::Wildcard))
        ));
    }

    #[test]
    fn all_selectors() {
        {
            let (_, s) = parse_selector("0").unwrap();
            assert_eq!(s, Selector::Index(Index(0)));
        }
        {
            let (_, s) = parse_selector("10").unwrap();
            assert_eq!(s, Selector::Index(Index(10)));
        }
        {
            let (_, s) = parse_selector("'name'").unwrap();
            assert_eq!(s, Selector::Name(Name(String::from("name"))));
        }
        {
            let (_, s) = parse_selector("\"name\"").unwrap();
            assert_eq!(s, Selector::Name(Name(String::from("name"))));
        }
        {
            let (_, s) = parse_selector("0:3").unwrap();
            assert_eq!(
                s,
                Selector::ArraySlice(Slice::new().with_start(0).with_end(3))
            );
        }
    }
}
