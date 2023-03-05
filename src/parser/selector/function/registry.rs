use once_cell::sync::Lazy;
use serde_json::Value;

use crate::parser::selector::function::Function;

use super::{Evaluator, FuncType};

static LENGTH: Evaluator = Lazy::new(|| {
    Box::new(|v| {
        println!("length: {v:?}");
        if let Some(arg) = v.first() {
            match arg {
                FuncType::Nodelist(nl) => {
                    if let Some(v) = nl.first() {
                        match v {
                            Value::String(s) => FuncType::Value(s.len().into()),
                            Value::Array(a) => FuncType::Value(a.len().into()),
                            Value::Object(o) => FuncType::Value(o.len().into()),
                            _ => FuncType::Nothing,
                        }
                    } else {
                        FuncType::Nothing
                    }
                }
                FuncType::Node(val) => {
                    if let Some(v) = val {
                        match v {
                            Value::String(s) => FuncType::Value(s.len().into()),
                            Value::Array(a) => FuncType::Value(a.len().into()),
                            Value::Object(o) => FuncType::Value(o.len().into()),
                            _ => FuncType::Node(None),
                        }
                    } else {
                        FuncType::Nothing
                    }
                }
                FuncType::Value(val) => match val {
                    Value::String(s) => FuncType::Value(s.len().into()),
                    Value::Array(a) => FuncType::Value(a.len().into()),
                    Value::Object(o) => FuncType::Value(o.len().into()),
                    _ => FuncType::Nothing,
                },
                FuncType::ValueRef(val) => match val {
                    Value::String(s) => FuncType::Value(s.len().into()),
                    Value::Array(a) => FuncType::Value(a.len().into()),
                    Value::Object(o) => FuncType::Value(o.len().into()),
                    _ => FuncType::Nothing,
                },
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
        println!("count: {v:?}");
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
