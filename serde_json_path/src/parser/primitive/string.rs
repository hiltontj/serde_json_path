use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{anychar, char};
use nom::character::streaming::one_of;
use nom::combinator::{cut, map, recognize, verify};
use nom::error::context;
use nom::sequence::{pair, separated_pair, tuple};
use nom::{
    branch::alt,
    bytes::complete::take_while_m_n,
    combinator::{map_opt, map_res, value},
    multi::fold_many0,
    sequence::{delimited, preceded},
};

use crate::parser::utils::cut_with;
use crate::parser::PResult;

#[derive(Debug, Copy, Clone)]
enum Quotes {
    Single,
    Double,
}

fn is_digit(chr: &char) -> bool {
    chr.is_ascii_digit()
}

fn is_hex_digit(chr: char) -> bool {
    is_digit(&chr) || ('A'..='F').contains(&chr) || ('a'..='f').contains(&chr)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_digit(input: &str) -> PResult<char> {
    verify(anychar, is_digit)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None))]
fn parse_n_hex_digits(n: usize) -> impl Fn(&str) -> PResult<&str> {
    move |input: &str| take_while_m_n(n, n, is_hex_digit)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_non_surrogate(input: &str) -> PResult<char> {
    let non_d_base = alt((parse_digit, one_of("ABCEFabcdef")));
    let non_d_based = pair(non_d_base, parse_n_hex_digits(3));
    let zero_to_7 = verify(anychar, |c: &char| ('0'..='7').contains(c));
    let d_based = tuple((one_of("Dd"), zero_to_7, parse_n_hex_digits(2)));
    let parse_u32 = map_res(alt((recognize(non_d_based), recognize(d_based))), |hex| {
        u32::from_str_radix(hex, 16)
    });
    context("non surrogate", map_opt(parse_u32, char::from_u32))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_low_surrogate(input: &str) -> PResult<u16> {
    context(
        "low surrogate",
        map_res(
            recognize(tuple((
                one_of("Dd"),
                one_of("CDEFcdef"),
                parse_n_hex_digits(2),
            ))),
            |hex| u16::from_str_radix(hex, 16),
        ),
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_high_surrogate(input: &str) -> PResult<u16> {
    context(
        "high surrogate",
        map_res(
            recognize(tuple((char('D'), one_of("89AB"), parse_n_hex_digits(2)))),
            |hex| u16::from_str_radix(hex, 16),
        ),
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_surrogate(input: &str) -> PResult<String> {
    context(
        "surrogate pair",
        map_res(
            separated_pair(parse_high_surrogate, tag("\\u"), parse_low_surrogate),
            |(h, l)| String::from_utf16(&[h, l]),
        ),
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_hex_char(input: &str) -> PResult<String> {
    alt((map(parse_non_surrogate, String::from), parse_surrogate))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_unicode_sequence(input: &str) -> PResult<String> {
    context("unicode sequence", preceded(char('u'), parse_hex_char))(input)
}

fn parse_escaped_quote(quoted_with: Quotes) -> impl Fn(&str) -> PResult<char> {
    move |input: &str| match quoted_with {
        Quotes::Single => value('\u{0027}', char('\''))(input),
        Quotes::Double => value('\u{0022}', char('"'))(input),
    }
}

fn parse_escaped_char(quoted_with: Quotes) -> impl Fn(&str) -> PResult<String> {
    move |input: &str| {
        context(
            "escaped character",
            preceded(
                char('\\'),
                alt((
                    map(
                        alt((
                            value('\u{0008}', char('b')),
                            value('\u{0009}', char('t')),
                            value('\u{000A}', char('n')),
                            value('\u{000C}', char('f')),
                            value('\u{000D}', char('r')),
                            value('\u{002F}', char('/')),
                            value('\u{005C}', char('\\')),
                            parse_escaped_quote(quoted_with),
                        )),
                        String::from,
                    ),
                    parse_unicode_sequence,
                )),
            ),
        )(input)
    }
}

fn is_valid_unescaped_char(chr: char, quoted_with: Quotes) -> bool {
    let invalid_quote_char = match quoted_with {
        Quotes::Single => '\'',
        Quotes::Double => '"',
    };
    if chr == invalid_quote_char {
        return false;
    }
    match chr {
        '\u{20}'..='\u{5B}' // Omit control characters
        | '\u{5D}'..='\u{10FFFF}' => true, // Omit \
        _ => false,
    }
}

fn parse_unescaped(quoted_with: Quotes) -> impl Fn(&str) -> PResult<&str> {
    move |input: &str| {
        context(
            "unescaped character",
            verify(
                take_while(|chr| is_valid_unescaped_char(chr, quoted_with)),
                |s: &str| !s.is_empty(),
            ),
        )(input)
    }
}

fn parse_fragment(quoted_with: Quotes) -> impl Fn(&str) -> PResult<String> {
    move |input: &str| {
        alt((
            map(parse_unescaped(quoted_with), String::from),
            parse_escaped_char(quoted_with),
        ))(input)
    }
}

fn parse_internal(quoted_with: Quotes) -> impl Fn(&str) -> PResult<String> {
    move |input: &str| {
        fold_many0(
            parse_fragment(quoted_with),
            String::new,
            |mut string, fragment| {
                string.push_str(fragment.as_str());
                string
            },
        )(input)
    }
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_single_quoted(input: &str) -> PResult<String> {
    context(
        "single quoted",
        delimited(
            char('\''),
            parse_internal(Quotes::Single),
            cut_with(char('\''), |_| StringError::ExpectedEndQuote),
        ),
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_double_quoted(input: &str) -> PResult<String> {
    context(
        "double quoted",
        delimited(char('"'), parse_internal(Quotes::Double), cut(char('"'))),
    )(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_string_literal(input: &str) -> PResult<String> {
    context(
        "string literal",
        alt((parse_single_quoted, parse_double_quoted)),
    )(input)
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum StringError {
    #[error("expected an ending quote")]
    ExpectedEndQuote,
}

#[cfg(test)]
mod tests {
    use crate::parser::primitive::string::{parse_escaped_char, Quotes};

    use super::parse_string_literal;

    #[test]
    fn valid_double_quoted_selectors() {
        assert_eq!(
            parse_string_literal("\"test\""),
            Ok(("", String::from("test")))
        );
        assert_eq!(
            parse_string_literal("\"test\\n\""),
            Ok(("", String::from("test\n")))
        );
        assert_eq!(
            parse_string_literal("\"test\\ntest\""),
            Ok(("", String::from("test\ntest")))
        );
        assert_eq!(
            parse_string_literal("\"test\\\"\""),
            Ok(("", String::from("test\"")))
        );
        assert_eq!(
            parse_string_literal("\"tes't\""),
            Ok(("", String::from("tes't")))
        );
    }

    #[test]
    fn valid_single_quoted_selectors() {
        assert_eq!(
            parse_string_literal("'test'"),
            Ok(("", String::from("test")))
        );
        assert_eq!(
            parse_string_literal(r#"'te"st'"#),
            Ok(("", String::from("te\"st")))
        );
        assert_eq!(
            parse_string_literal(r"'te\'st'"),
            Ok(("", String::from("te'st")))
        );
    }

    #[test]
    fn invalid_unicode() {
        {
            for c in '\u{00}'..'\u{20}' {
                let input = format!("{c}");
                let result = parse_escaped_char(Quotes::Double)(&input);
                assert!(result.is_err());
            }
        }
    }
}
