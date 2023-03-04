use once_cell::sync::Lazy;

use crate::parser::selector::function::Function;

use super::{Evaluator, FuncType};

static LENGTH: Evaluator = Lazy::new(|| Box::new(
    |v| {
        if let Some(arg) = v.first() {
            match arg {
                FuncType::Nodelist(_) => FuncType::Node(None),
                FuncType::Node(val) => if let Some(v)  = val{
                    match v {
                        serde_json::Value::String(s) => FuncType::Value(s.len().into()),
                        serde_json::Value::Array(a) => FuncType::Value(a.len().into()),
                        serde_json::Value::Object(o) => FuncType::Value(o.len().into()),
                        _ => FuncType::Node(None),
                    }
                } else {
                    FuncType::Node(None)
                },
                FuncType::Value(_) => FuncType::Nothing,
                FuncType::Nothing => FuncType::Nothing,
            }
        } else {
            FuncType::Nothing
        }
    }
));

inventory::submit! {
    Function::new(
        "length",
        &LENGTH,
    )
}
