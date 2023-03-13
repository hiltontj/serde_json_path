// use once_cell::sync::Lazy;
// use regex::Regex;
// use serde_json::Value;

// use crate::parser::selector::function::Function;

// use super::{Evaluator, JsonPathType};

// fn value_length<'b>(v: &Value) -> JsonPathType<'b> {
//     match v {
//         Value::String(s) => JsonPathType::Value(s.len().into()),
//         Value::Array(a) => JsonPathType::Value(a.len().into()),
//         Value::Object(o) => JsonPathType::Value(o.len().into()),
//         _ => JsonPathType::Nothing,
//     }
// }

// static LENGTH: Evaluator = Lazy::new(|| {
//     Box::new(|v| {
//         if let Some(arg) = v.first() {
//             match arg {
//                 JsonPathType::Nodelist(nl) => {
//                     if nl.len() == 1 {
//                         nl.first().map(|v| value_length(v)).unwrap_or_default()
//                     } else {
//                         JsonPathType::Nothing
//                     }
//                 }
//                 JsonPathType::Node(n) => n.map(value_length).unwrap_or_default(),
//                 JsonPathType::Value(v) => value_length(v),
//                 JsonPathType::ValueRef(v) => value_length(v),
//                 JsonPathType::Nothing => JsonPathType::Nothing,
//             }
//         } else {
//             JsonPathType::Nothing
//         }
//     })
// });

// inventory::submit! {
//     Function::new(
//         "length",
//         &LENGTH,
//     )
// }

// static COUNT: Evaluator = Lazy::new(|| {
//     Box::new(|v| {
//         if let Some(arg) = v.first() {
//             match arg {
//                 JsonPathType::Nodelist(nl) => JsonPathType::Value(nl.len().into()),
//                 _ => JsonPathType::Value(0.into()),
//             }
//         } else {
//             JsonPathType::Value(0.into())
//         }
//     })
// });

// inventory::submit! {
//     Function::new(
//         "count",
//         &COUNT,
//     )
// }

// static MATCH: Evaluator = Lazy::new(|| {
//     Box::new(|v| {
//         if let (Some(JsonPathType::Nodelist(nl)), Some(JsonPathType::ValueRef(Value::String(rgx_str)))) =
//             (v.get(0), v.get(1))
//         {
//             if let Some(Value::String(test_str)) = nl.first() {
//                 Regex::new(format!("^{rgx_str}$").as_str())
//                     .map(|rgx| rgx.is_match(test_str))
//                     .map(|b| JsonPathType::Value(b.into()))
//                     .unwrap_or_default()
//             } else {
//                 JsonPathType::Nothing
//             }
//         } else {
//             JsonPathType::Nothing
//         }
//     })
// });

// inventory::submit! {
//     Function::new(
//         "match",
//         &MATCH,
//     )
// }

// static SEARCH: Evaluator = Lazy::new(|| {
//     Box::new(|v| {
//         if let (Some(JsonPathType::Nodelist(nl)), Some(JsonPathType::ValueRef(Value::String(rgx_str)))) =
//             (v.get(0), v.get(1))
//         {
//             if let Some(Value::String(test_str)) = nl.first() {
//                 Regex::new(rgx_str)
//                     .map(|rgx| rgx.is_match(test_str))
//                     .map(|b| JsonPathType::Value(b.into()))
//                     .unwrap_or_default()
//             } else {
//                 JsonPathType::Nothing
//             }
//         } else {
//             JsonPathType::Nothing
//         }
//     })
// });

// inventory::submit! {
//     Function::new(
//         "search",
//         &SEARCH,
//     )
// }
