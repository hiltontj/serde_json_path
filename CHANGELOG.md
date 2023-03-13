# Changelog

All noteable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# Unreleased

- **added:** Add `first`, `last`, and `get` methods to `NodeList` type ([#16])
- **changed:** Make `NodeList::at_most_one` and `NodeList::exactly_one` take `&self` instead of `self` ([#16])
- **docs:** Update crate-level docs to better reflect recent changes ([#21])
- **docs:** Corrected a broken link in crate-level docs ([#21])
- **added:** derive `Clone` for `JsonPath` and its descendants ([#24])
- **added:** derive `Default` for `JsonPath` ([#25])

[#16]: https://github.com/hiltontj/serde_json_path/pull/16
[#21]: https://github.com/hiltontj/serde_json_path/pull/21
[#24]: https://github.com/hiltontj/serde_json_path/pull/24
[#25]: https://github.com/hiltontj/serde_json_path/pull/25

# 0.5.1 (11 March 2023)

- **added:** Derive `PartialEq` on `JsonPath` ([#13])
- **added:** Add the `NodeList::at_most_one` method ([#13])
- **added:** Add the `NodeList::exactly_one` method ([#13])
- **deprecated:** Deprecate the `NodeList::one` method in favor of the new `NodeList::at_most_one` method ([#13])

[#13]: https://github.com/hiltontj/serde_json_path/pull/13

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

`JsonPath` implements `serde`'s `Deserialize`, which allows it to be used directly in serialized formats

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

`JsonPath` also implements `FromStr`, for convenience, e.g.,

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