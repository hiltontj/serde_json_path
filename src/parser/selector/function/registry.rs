use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

use crate::parser::selector::function::Function;

use super::{Evaluator, FuncType};

fn value_length<'b>(v: &Value) -> FuncType<'b> {
    match v {
        Value::String(s) => FuncType::Value(s.len().into()),
        Value::Array(a) => FuncType::Value(a.len().into()),
        Value::Object(o) => FuncType::Value(o.len().into()),
        _ => FuncType::Nothing,
    }
}

static LENGTH: Evaluator = Lazy::new(|| {
    Box::new(|v| {
        if let Some(arg) = v.first() {
            match arg {
                FuncType::Nodelist(nl) => {
                    if nl.len() == 1 {
                        nl.first().map(|v| value_length(v)).unwrap_or_default()
                    } else {
                        FuncType::Nothing
                    }
                }
                FuncType::Node(n) => n.map(value_length).unwrap_or_default(),
                FuncType::Value(v) => value_length(v),
                FuncType::ValueRef(v) => value_length(v),
                FuncType::Nothing => FuncType::Nothing,
            }
        } else {
            FuncType::Nothing
        }
    })
});

inventory::submit! {
    Function::new(
        "length",
        &LENGTH,
    )
}

static COUNT: Evaluator = Lazy::new(|| {
    Box::new(|v| {
        if let Some(arg) = v.first() {
            match arg {
                FuncType::Nodelist(nl) => FuncType::Value(nl.len().into()),
                _ => FuncType::Value(0.into()),
            }
        } else {
            FuncType::Value(0.into())
        }
    })
});

inventory::submit! {
    Function::new(
        "count",
        &COUNT,
    )
}

static MATCH: Evaluator = Lazy::new(|| {
    Box::new(|v| {
        if let (Some(FuncType::Nodelist(nl)), Some(FuncType::ValueRef(Value::String(rgx_str)))) =
            (v.get(0), v.get(1))
        {
            if let Some(Value::String(test_str)) = nl.first() {
                Regex::new(format!("^{rgx_str}$").as_str())
                    .map(|rgx| rgx.is_match(test_str))
                    .map(|b| FuncType::Value(b.into()))
                    .unwrap_or_default()
            } else {
                FuncType::Nothing
            }
        } else {
            FuncType::Nothing
        }
    })
});

inventory::submit! {
    Function::new(
        "match",
        &MATCH,
    )
}

static SEARCH: Evaluator = Lazy::new(|| {
    Box::new(|v| {
        if let (Some(FuncType::Nodelist(nl)), Some(FuncType::ValueRef(Value::String(rgx_str)))) =
            (v.get(0), v.get(1))
        {
            if let Some(Value::String(test_str)) = nl.first() {
                Regex::new(rgx_str)
                    .map(|rgx| rgx.is_match(test_str))
                    .map(|b| FuncType::Value(b.into()))
                    .unwrap_or_default()
            } else {
                FuncType::Nothing
            }
        } else {
            FuncType::Nothing
        }
    })
});

inventory::submit! {
    Function::new(
        "search",
        &SEARCH,
    )
}
