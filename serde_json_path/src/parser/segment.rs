use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::cut;
use nom::error::context;
use nom::sequence::terminated;
use nom::{
    branch::alt,
    bytes::complete::take_while1,
    character::complete::{alpha1, digit1, space0},
    combinator::{map, recognize},
    multi::{fold_many0, separated_list1},
    sequence::{delimited, pair, preceded},
};
use serde_json::Value;

use super::selector::{parse_selector, parse_wildcard_selector, Selector};
use super::{PResult, Queryable};

#[derive(Debug, PartialEq, Clone)]
pub struct PathSegment {
    pub kind: PathSegmentKind,
    pub segment: Segment,
}

impl PathSegment {
    pub fn is_child(&self) -> bool {
        matches!(self.kind, PathSegmentKind::Child)
    }

    pub fn is_descendent(&self) -> bool {
        !self.is_child()
    }
}

impl std::fmt::Display for PathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if matches!(self.kind, PathSegmentKind::Descendant) {
            write!(f, "..")?;
        }
        write!(f, "{segment}", segment = self.segment)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PathSegmentKind {
    Child,
    Descendant,
}

impl Queryable for PathSegment {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Query Path Segment", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
        let mut query = self.segment.query(current, root);
        if matches!(self.kind, PathSegmentKind::Descendant) {
            query.append(&mut descend(self, current, root));
        }
        query
    }
}

fn descend<'b>(segment: &PathSegment, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
    let mut query = Vec::new();
    if let Some(list) = current.as_array() {
        for v in list {
            query.append(&mut segment.query(v, root));
        }
    } else if let Some(obj) = current.as_object() {
        for (_, v) in obj {
            query.append(&mut segment.query(v, root));
        }
    }
    query
}

#[derive(Debug, PartialEq, Clone)]
pub enum Segment {
    LongHand(Vec<Selector>),
    DotName(String),
    Wildcard,
}

impl Segment {
    pub fn is_singular(&self) -> bool {
        match self {
            Segment::LongHand(selectors) => {
                if selectors.len() > 1 {
                    return false;
                }
                if let Some(s) = selectors.first() {
                    return s.is_singular();
                } else {
                    // if the selector list is empty, this shouldn't be a valid
                    // JSONPath, but at least, it would be selecting nothing, and
                    // that could be considered singular, i.e., None.
                    return true;
                }
            }
            Segment::DotName(_) => true,
            Segment::Wildcard => false,
        }
    }

    #[cfg(test)]
    pub fn as_long_hand(&self) -> Option<&[Selector]> {
        match self {
            Segment::LongHand(v) => Some(v.as_slice()),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn as_dot_name(&self) -> Option<&str> {
        match self {
            Segment::DotName(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl std::fmt::Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::LongHand(selectors) => {
                write!(f, "[")?;
                for (i, s) in selectors.iter().enumerate() {
                    write!(
                        f,
                        "{s}{comma}",
                        comma = if i == selectors.len() - 1 { "" } else { "," }
                    )?;
                }
                write!(f, "]")?;
            }
            Segment::DotName(name) => write!(f, ".{name}")?,
            Segment::Wildcard => write!(f, ".*")?,
        }
        Ok(())
    }
}

impl Queryable for Segment {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Query Segment", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
        let mut query = Vec::new();
        match self {
            Segment::LongHand(selectors) => {
                for selector in selectors {
                    query.append(&mut selector.query(current, root));
                }
            }
            Segment::DotName(key) => {
                if let Some(obj) = current.as_object() {
                    if let Some(v) = obj.get(key) {
                        query.push(v);
                    }
                }
            }
            Segment::Wildcard => {
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
        }
        query
    }
}

// TODO - I have no idea if this is correct, supposed to be %x80-10FFFF
fn is_non_ascii_unicode(chr: char) -> bool {
    chr >= '\u{0080}'
}

fn parse_non_ascii_unicode(input: &str) -> PResult<&str> {
    take_while1(is_non_ascii_unicode)(input)
}

fn parse_name_first(input: &str) -> PResult<&str> {
    alt((alpha1, recognize(char('_')), parse_non_ascii_unicode))(input)
}

fn parse_name_char(input: &str) -> PResult<&str> {
    alt((digit1, parse_name_first))(input)
}

pub fn parse_dot_member_name(input: &str) -> PResult<String> {
    map(
        recognize(pair(
            parse_name_first,
            fold_many0(parse_name_char, String::new, |mut s, item| {
                s.push_str(item);
                s
            }),
        )),
        |s| s.to_string(),
    )(input)
}

fn parse_dot_member_name_shorthand(input: &str) -> PResult<Segment> {
    map(preceded(char('.'), parse_dot_member_name), Segment::DotName)(input)
}

fn parse_multi_selector(input: &str) -> PResult<Vec<Selector>> {
    separated_list1(delimited(space0, char(','), space0), parse_selector)(input)
}

fn parse_child_long_hand(input: &str) -> PResult<Segment> {
    context(
        "child long-hand segment",
        preceded(
            pair(char('['), space0),
            cut(terminated(
                map(parse_multi_selector, Segment::LongHand),
                pair(space0, char(']')),
            )),
        ),
    )(input)
}

fn parse_dot_wildcard_shorthand(input: &str) -> PResult<Segment> {
    map(preceded(char('.'), parse_wildcard_selector), |_| {
        Segment::Wildcard
    })(input)
}

fn parse_child_segment(input: &str) -> PResult<Segment> {
    alt((
        parse_dot_wildcard_shorthand,
        parse_dot_member_name_shorthand,
        parse_child_long_hand,
    ))(input)
}

fn parse_descendant_segment(input: &str) -> PResult<Segment> {
    preceded(
        tag(".."),
        alt((
            map(parse_wildcard_selector, |_| Segment::Wildcard),
            map(parse_dot_member_name, Segment::DotName),
            parse_child_segment,
        )),
    )(input)
}

pub fn parse_segment(input: &str) -> PResult<PathSegment> {
    alt((
        map(parse_descendant_segment, |inner| PathSegment {
            kind: PathSegmentKind::Descendant,
            segment: inner,
        }),
        map(parse_child_segment, |inner| PathSegment {
            kind: PathSegmentKind::Child,
            segment: inner,
        }),
    ))(input)
}

#[cfg(test)]
mod tests {
    use nom::{combinator::all_consuming, error::convert_error};

    use crate::parser::selector::{slice::Slice, Index, Name, Selector};

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
            let i = "[010]";
            let err = parse_child_long_hand("[010]").unwrap_err();
            match err {
                nom::Err::Error(e) | nom::Err::Failure(e) => println!("{}", convert_error(i, e)),
                _ => panic!("wrong error kind: {err:?}"),
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
            let (_, sk) = parse_descendant_segment("...name").unwrap();
            assert_eq!(sk.as_dot_name().unwrap(), "name");
        }
        {
            let (_, sk) = parse_descendant_segment("..*").unwrap();
            assert!(matches!(sk, Segment::Wildcard));
        }
        {
            let (_, sk) = parse_descendant_segment("...*").unwrap();
            assert!(matches!(sk, Segment::Wildcard));
        }
    }
}
