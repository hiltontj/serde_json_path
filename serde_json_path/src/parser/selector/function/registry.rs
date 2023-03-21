use regex::Regex;
use serde_json::Value;
use serde_json_path_core::spec::functions::{LogicalType, NodesType, ValueType};

fn value_length(value: &Value) -> Option<usize> {
    match value {
        Value::String(s) => Some(s.len()),
        Value::Array(a) => Some(a.len()),
        Value::Object(o) => Some(o.len()),
        _ => None,
    }
}

#[serde_json_path_macros::function]
fn length(value: ValueType) -> ValueType {
    match value {
        ValueType::Value(v) => value_length(&v),
        ValueType::ValueRef(v) | ValueType::Node(v) => value_length(v),
        ValueType::Nothing => None,
    }
    .map_or(ValueType::Nothing, |l| ValueType::Value(l.into()))
}

#[serde_json_path_macros::function]
fn count(nodes: NodesType) -> ValueType {
    nodes.into_inner().len().into()
}

#[serde_json_path_macros::function]
fn matcho(value: ValueType, rgx: ValueType) -> LogicalType {
    match (value.as_value(), rgx.as_value()) {
        (Some(Value::String(s)), Some(Value::String(r))) => Regex::new(format!("^{r}$").as_str())
            .map(|r| r.is_match(s))
            .map(Into::into)
            .unwrap_or_default(),
        _ => LogicalType::False,
    }
}
