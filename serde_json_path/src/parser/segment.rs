use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::error::context;
use nom::sequence::terminated;
use nom::{
    branch::alt,
    bytes::complete::take_while1,
    character::complete::{alpha1, digit1, multispace0},
    combinator::{map, recognize},
    multi::{fold_many0, separated_list1},
    sequence::{delimited, pair, preceded},
};
use serde_json_path_core::spec::segment::{QuerySegment, QuerySegmentKind, Segment};
use serde_json_path_core::spec::selector::Selector;

use super::selector::{parse_selector, parse_wildcard_selector};
use super::utils::cut_with;
use super::PResult;

// TODO - I have no idea if this is correct, supposed to be %x80-10FFFF
fn is_non_ascii_unicode(chr: char) -> bool {
    chr >= '\u{0080}'
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_non_ascii_unicode(input: &str) -> PResult<&str> {
    take_while1(is_non_ascii_unicode)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_name_first(input: &str) -> PResult<&str> {
    alt((alpha1, recognize(char('_')), parse_non_ascii_unicode))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_name_char(input: &str) -> PResult<&str> {
    alt((digit1, parse_name_first))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_dot_member_name(input: &str) -> PResult<String> {
    map(
        recognize(pair(
            cut_with(parse_name_first, |_| {
                SegmentError::InvalidFirstNameCharacter
            }),
            fold_many0(parse_name_char, String::new, |mut s, item| {
                s.push_str(item);
                s
            }),
        )),
        |s| s.to_owned(),
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_dot_member_name_shorthand(input: &str) -> PResult<Segment> {
    map(
        preceded(char('.'), context("dot member name", parse_dot_member_name)),
        Segment::DotName,
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_multi_selector(input: &str) -> PResult<Vec<Selector>> {
    separated_list1(
        delimited(multispace0, char(','), multispace0),
        parse_selector,
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_child_long_hand(input: &str) -> PResult<Segment> {
    context(
        "long-hand segment",
        preceded(
            pair(char('['), multispace0),
            terminated(
                map(parse_multi_selector, Segment::LongHand),
                pair(
                    multispace0,
                    cut_with(char(']'), |_| SegmentError::ExpectedClosingBrace),
                ),
            ),
        ),
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_dot_wildcard_shorthand(input: &str) -> PResult<Segment> {
    map(preceded(char('.'), parse_wildcard_selector), |_| {
        Segment::Wildcard
    })(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_child_segment(input: &str) -> PResult<Segment> {
    preceded(
        multispace0,
        alt((
            parse_dot_wildcard_shorthand,
            parse_dot_member_name_shorthand,
            parse_child_long_hand,
        )),
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_descendant_segment(input: &str) -> PResult<Segment> {
    preceded(
        tag(".."),
        alt((
            map(parse_wildcard_selector, |_| Segment::Wildcard),
            parse_child_long_hand,
            map(parse_dot_member_name, Segment::DotName),
        )),
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_segment(input: &str) -> PResult<QuerySegment> {
    alt((
        map(parse_descendant_segment, |inner| QuerySegment {
            kind: QuerySegmentKind::Descendant,
            segment: inner,
        }),
        map(parse_child_segment, |inner| QuerySegment {
            kind: QuerySegmentKind::Child,
            segment: inner,
        }),
    ))(input)
}

#[doc(hidden)]
#[derive(Debug, thiserror::Error)]
pub(crate) enum SegmentError {
    #[error("must start with lowercase alpha or '_'")]
    InvalidFirstNameCharacter,
    #[error("expected closing ']'")]
    ExpectedClosingBrace,
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "trace")]
    use test_log::test;

    use nom::combinator::all_consuming;
    use serde_json_path_core::spec::selector::{index::Index, name::Name, slice::Slice, Selector};

    use super::{
        parse_child_long_hand, parse_child_segment, parse_descendant_segment,
        parse_dot_member_name_shorthand, Segment,
    };

    #[test]
    fn dot_member_names() {
        assert!(matches!(
            parse_dot_member_name_shorthand(".name"),
            Ok(("", Segment::DotName(s))) if s == "name",
        ));
        assert!(matches!(
            parse_dot_member_name_shorthand(".foo_bar"),
            Ok(("", Segment::DotName(s))) if s == "foo_bar",
        ));
        assert!(parse_dot_member_name_shorthand(". space").is_err());
        assert!(all_consuming(parse_dot_member_name_shorthand)(".no-dash").is_err());
        assert!(parse_dot_member_name_shorthand(".1no_num_1st").is_err());
    }

    #[test]
    fn child_long_hand() {
        {
            let (_, sk) = parse_child_long_hand(r#"["name"]"#).unwrap();
            let s = sk.as_long_hand().unwrap();
            assert_eq!(s[0], Selector::Name(Name::from("name")));
        }
        {
            let (_, sk) = parse_child_long_hand(r#"['name']"#).unwrap();
            let s = sk.as_long_hand().unwrap();
            assert_eq!(s[0], Selector::Name(Name::from("name")));
        }
        {
            let (_, sk) = parse_child_long_hand(r#"["name","test"]"#).unwrap();
            let s = sk.as_long_hand().unwrap();
            assert_eq!(s[0], Selector::Name(Name::from("name")));
            assert_eq!(s[1], Selector::Name(Name::from("test")));
        }
        {
            let (_, sk) = parse_child_long_hand(r#"['name',10,0:3]"#).unwrap();
            let s = sk.as_long_hand().unwrap();
            assert_eq!(s[0], Selector::Name(Name::from("name")));
            assert_eq!(s[1], Selector::Index(Index(10)));
            assert_eq!(
                s[2],
                Selector::ArraySlice(Slice::new().with_start(0).with_end(3))
            );
        }
        {
            let (_, sk) = parse_child_long_hand(r#"[::,*]"#).unwrap();
            let s = sk.as_long_hand().unwrap();
            assert_eq!(s[0], Selector::ArraySlice(Slice::new()));
            assert_eq!(s[1], Selector::Wildcard);
        }
        {
            let err = parse_child_long_hand("[010]").unwrap_err();
            match err {
                nom::Err::Error(e) | nom::Err::Failure(e) => println!("{e:#?}"),
                nom::Err::Incomplete(_) => panic!("wrong error kind: {err:?}"),
            }
        }
    }

    #[test]
    fn child_segment() {
        {
            let (_, sk) = parse_child_segment(".name").unwrap();
            assert_eq!(sk.as_dot_name(), Some("name"));
        }
        {
            let (_, sk) = parse_child_segment(".*").unwrap();
            assert!(matches!(sk, Segment::Wildcard));
        }
        {
            let (_, sk) = parse_child_segment("[*]").unwrap();
            let s = sk.as_long_hand().unwrap();
            assert_eq!(s[0], Selector::Wildcard);
        }
    }

    #[test]
    fn descendant_semgent() {
        {
            let (_, sk) = parse_descendant_segment("..['name']").unwrap();
            let s = sk.as_long_hand().unwrap();
            assert_eq!(s[0], Selector::Name(Name::from("name")));
        }
        {
            let (_, sk) = parse_descendant_segment("..name").unwrap();
            assert_eq!(sk.as_dot_name().unwrap(), "name");
        }
        {
            let (_, sk) = parse_descendant_segment("..*").unwrap();
            assert!(matches!(sk, Segment::Wildcard));
        }
    }
}
