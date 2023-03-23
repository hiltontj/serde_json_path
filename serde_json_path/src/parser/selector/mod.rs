use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::map;
use nom::error::context;
use serde_json_path_core::spec::selector::index::Index;
use serde_json_path_core::spec::selector::name::Name;
use serde_json_path_core::spec::selector::Selector;

use self::filter::parse_filter;
use self::slice::parse_array_slice;

use super::primitive::int::parse_int;
use super::primitive::string::parse_string_literal;
use super::PResult;

pub(crate) mod filter;
pub(crate) mod function;
pub(crate) mod slice;

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_wildcard_selector(input: &str) -> PResult<Selector> {
    map(char('*'), |_| Selector::Wildcard)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_name(input: &str) -> PResult<Name> {
    map(parse_string_literal, Name)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_name_selector(input: &str) -> PResult<Selector> {
    map(parse_name, Selector::Name)(input)
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
pub(crate) fn parse_selector(input: &str) -> PResult<Selector> {
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
    use serde_json_path_core::spec::selector::{name::Name, slice::Slice};

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
