#[cfg(feature = "regex")]
use std::{cell::RefCell, rc::Rc};
use std::{collections::HashMap, sync::LazyLock};

use serde_json::Value;
use serde_json_path_core::spec::functions::{Function, LogicalType, NodesType, ValueType};

/// The main registry of functions for use in JSONPath queries
///
/// These come directly from the JSONPath specification, which includes a registry of standardized
/// functions.
///
/// # Note
///
/// There is a function in `serde_json_path_core/src/spec/functions.rs` that gives
/// the return type for each function registered here. When adding new functions to
/// the register, i.e., when new functions are standardized, the function there needs
/// to be updated too.
pub(crate) static REGISTRY: LazyLock<HashMap<&'static str, &'static Function>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert("length", &LENGTH_FUNC);
        m.insert("count", &COUNT_FUNC);
        #[cfg(feature = "regex")]
        {
            m.insert("match", &MATCH_FUNC);
            m.insert("search", &SEARCH_FUNC);
        }
        m.insert("value", &VALUE_FUNC);
        m
    });

fn value_length(value: &Value) -> Option<usize> {
    match value {
        Value::String(s) => Some(s.chars().count()),
        Value::Array(a) => Some(a.len()),
        Value::Object(o) => Some(o.len()),
        _ => None,
    }
}

#[serde_json_path_macros::register(target = LENGTH_FUNC)]
fn length(value: ValueType<'_>) -> ValueType<'_> {
    match value {
        ValueType::Value(v) => value_length(&v),
        ValueType::Node(v) => value_length(v),
        ValueType::Nothing => None,
    }
    .map_or(ValueType::Nothing, |l| ValueType::Value(l.into()))
}

#[serde_json_path_macros::register(target = COUNT_FUNC)]
fn count(nodes: NodesType<'_>) -> ValueType<'_> {
    nodes.len().into()
}

#[cfg(feature = "regex")]
#[serde_json_path_macros::register(name = "match", target = MATCH_FUNC)]
fn match_func(value: ValueType<'_>, rgx: ValueType<'_>) -> LogicalType {
    match (value.as_value(), rgx.as_value()) {
        (Some(Value::String(s)), Some(Value::String(r))) => cached_regex(&format!("(?R)^({r})$"))
            .map(|r| r.is_match(s))
            .map(Into::into)
            .unwrap_or_default(),
        _ => LogicalType::False,
    }
}

#[cfg(feature = "regex")]
#[serde_json_path_macros::register(target = SEARCH_FUNC)]
fn search(value: ValueType<'_>, rgx: ValueType<'_>) -> LogicalType {
    match (value.as_value(), rgx.as_value()) {
        (Some(Value::String(s)), Some(Value::String(r))) => cached_regex(&format!("(?R)({r})"))
            .map(|r| r.is_match(s))
            .map(Into::into)
            .unwrap_or_default(),
        _ => LogicalType::False,
    }
}

/// How many compiled patterns each thread keeps for `match()` and `search()`.
#[cfg(feature = "regex")]
const REGEX_CACHE_SIZE: usize = 8;

#[cfg(feature = "regex")]
thread_local! {
    // match() and search() run once per node, usually with the same few patterns,
    // so each thread keeps the patterns it compiled last.
    static REGEX_CACHE: RefCell<Vec<(String, Rc<regex::Regex>)>> =
        const { RefCell::new(Vec::new()) };
}

/// Returns the compiled `pattern`, reusing it if it is among the last `REGEX_CACHE_SIZE` compiled
/// on this thread.
#[cfg(feature = "regex")]
fn cached_regex(pattern: &str) -> Result<Rc<regex::Regex>, regex::Error> {
    REGEX_CACHE
        .try_with(|cache| {
            let mut cache = cache.borrow_mut();
            if let Some((_, regex)) = cache.iter().find(|(p, _)| p == pattern) {
                return Ok(Rc::clone(regex));
            }
            let regex = Rc::new(regex::Regex::new(pattern)?);
            if cache.len() == REGEX_CACHE_SIZE {
                cache.remove(0);
            }
            cache.push((pattern.to_owned(), Rc::clone(&regex)));
            Ok(regex)
        })
        // The cache is gone while the thread's locals are being destroyed.
        .unwrap_or_else(|_| regex::Regex::new(pattern).map(Rc::new))
}

#[serde_json_path_macros::register(target = VALUE_FUNC)]
fn value(nodes: NodesType<'_>) -> ValueType<'_> {
    if nodes.len() > 1 {
        ValueType::Nothing
    } else {
        match nodes.first() {
            Some(v) => ValueType::Node(v),
            None => ValueType::Nothing,
        }
    }
}
