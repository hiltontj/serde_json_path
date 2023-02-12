use nom::branch::alt;
use nom::character::complete::char;
use nom::{combinator::map, IResult};
use serde_json::Value;

use self::filter::{parse_filter, Filter};
use self::slice::parse_array_slice;
use self::slice::Slice;

use super::primitive::int::parse_int;
use super::primitive::string::parse_string_literal;
use super::QueryValue;

pub mod filter;
pub mod slice;

#[derive(Debug, PartialEq)]
pub enum Selector {
    Name(Name),
    Wildcard,
    Index(Index),
    ArraySlice(Slice),
    Filter(Filter),
}

impl Selector {
    pub fn as_name(&self) -> Option<&str> {
        match self {
            Selector::Name(n) => Some(n.0.as_str()),
            _ => None,
        }
    }

    pub fn is_name(&self) -> bool {
        self.as_name().is_some()
    }

    pub fn is_wildcard(&self) -> bool {
        matches!(self, Selector::Wildcard)
    }

    pub fn as_index(&self) -> Option<isize> {
        match self {
            Selector::Index(i) => Some(i.0),
            _ => None,
        }
    }

    pub fn is_index(&self) -> bool {
        self.as_index().is_some()
    }

    pub fn as_array_slice(&self) -> Option<&Slice> {
        match self {
            Selector::ArraySlice(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_array_slice(&self) -> bool {
        self.as_array_slice().is_some()
    }

    pub fn as_filter(&self) -> Option<&Filter> {
        match self {
            Selector::Filter(f) => Some(f),
            _ => None,
        }
    }
}

impl QueryValue for Selector {
    fn query_value<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
        let mut query = Vec::new();
        match self {
            Selector::Name(name) => query.append(&mut name.query_value(current, root)),
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
            Selector::Index(index) => query.append(&mut index.query_value(current, root)),
            Selector::ArraySlice(slice) => query.append(&mut slice.query_value(current, root)),
            Selector::Filter(filter) => query.append(&mut filter.query_value(current, root)),
        }
        query
    }
}

pub fn parse_wildcard_selector(input: &str) -> IResult<&str, Selector> {
    map(char('*'), |_| Selector::Wildcard)(input)
}

#[derive(Debug, PartialEq)]
pub struct Name(String);

impl Name {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl QueryValue for Name {
    fn query_value<'b>(&self, current: &'b Value, _root: &'b Value) -> Vec<&'b Value> {
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

pub fn parse_name(input: &str) -> IResult<&str, Name> {
    map(parse_string_literal, Name)(input)
}

fn parse_name_selector(input: &str) -> IResult<&str, Selector> {
    map(parse_name, Selector::Name)(input)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Index(pub(crate) isize);

impl Index {
    pub fn as_isize(&self) -> isize {
        self.0
    }
}

impl QueryValue for Index {
    fn query_value<'b>(&self, current: &'b Value, _root: &'b Value) -> Vec<&'b Value> {
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

fn parse_index(input: &str) -> IResult<&str, Index> {
    map(parse_int, Index)(input)
}

fn parse_index_selector(input: &str) -> IResult<&str, Selector> {
    map(parse_index, Selector::Index)(input)
}

fn parse_array_slice_selector(input: &str) -> IResult<&str, Selector> {
    map(parse_array_slice, Selector::ArraySlice)(input)
}

fn parse_filter_selector(input: &str) -> IResult<&str, Selector> {
    map(parse_filter, Selector::Filter)(input)
}

pub fn parse_selector(input: &str) -> IResult<&str, Selector> {
    alt((
        parse_wildcard_selector,
        parse_name_selector,
        parse_array_slice_selector,
        parse_index_selector,
        parse_filter_selector,
    ))(input)
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
