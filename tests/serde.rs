use serde::Deserialize;
use serde_json::{from_value, json};
use serde_json_path::JsonPath;

#[derive(Deserialize)]
struct Config {
    pub path: JsonPath,
}

#[test]
fn can_deserialize_json_path() {
    let config_json = json!({ "path": "$.foo.*" });
    let config = from_value::<Config>(config_json).expect("deserializes");
    let value = json!({"foo": [1, 2, 3]});
    let nodes = config.path.query(&value).all();
    assert_eq!(nodes, vec![1, 2, 3]);
}
