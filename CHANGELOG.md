# Changelog

All noteable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# Unreleased

# 0.5.0 (10 March 2023)

## The `JsonPath` type

- **added:** Add the `JsonPath` type ([#10])

`JsonPath` is a new struct that contains a parsed and valid JSON Path query, and can be re-used to query `serde_json::Value`s.

```rust
let value = json!({"foo": [1, 2, 3]});
let path = JsonPath::parse("$.foo.*")?;
let nodes = path.query(&value).all();
assert_eq!(nodes, vec![1, 2, 3]);
```

`JsonPath` comes with some trait implementations, namely:

- `serde`'s `Deserialize`, which allows it to be used directly in serialized formats
```rust
#[derive(Deserialize)]
struct Config {
    pub path: JsonPath,
}
let config_json = json!({ "path": "$.foo.*" });
let config = from_value::<Config>(config_json).expect("deserializes");
let value = json!({"foo": [1, 2, 3]});
let nodes = config.path.query(&value).all();
assert_eq!(nodes, vec![1, 2, 3]);
```
- `FromStr`, for convenience, e.g.,
```rust
let path = "$.foo.*".parse::<JsonPath>()?;
```

## Other changes

- **breaking:** Alter the `JsonPathExt::json_path` API to accept a `&JsonPath` and to be infallible ([#10])

```rust
let value = json!({"foo": [1, 2, 3]});
let path = JsonPath::parse("$.foo.*")?;   // <- note, this is fallible
let nodes = value.json_path(&path).all(); // <- while this is not
assert_eq!(nodes, vec![1, 2, 3]);
```

[#10]: https://github.com/hiltontj/serde_json_path/pull/10

# Previous Versions

Previous versions are not documented here.