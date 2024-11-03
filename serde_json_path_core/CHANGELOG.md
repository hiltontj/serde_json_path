# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# Unreleased

# 0.2.1 (3 November 2024)

- **internal**: update `serde_json` to the latest version ([#107])
- **breaking**: ensure integers used as indices are within the [valid range for I-JSON][i-json-range] ([#98])
- **internal**: remove use of `once_cell` and use specific versions for crate dependencies ([#105])

[#98]: https://github.com/hiltontj/serde_json_path/pull/98
[i-json-range]: https://www.rfc-editor.org/rfc/rfc9535.html#section-2.1-4.1
[#105]: https://github.com/hiltontj/serde_json_path/pull/105
[#107]: https://github.com/hiltontj/serde_json_path/pull/107

# 0.1.6 (3 March 2024)

- **testing**: support tests for non-determinism in compliance test suite ([#85])
- **fixed**: bug preventing registered functions from being used as arguments to other functions ([#84])

[#85]: https://github.com/hiltontj/serde_json_path/pull/85
[#84]: https://github.com/hiltontj/serde_json_path/pull/84

# 0.1.5 (23 February 2024)

- **docs**: update links to refer to RFC 9535 ([#81])

[#81]: https://github.com/hiltontj/serde_json_path/pull/81

# 0.1.4 (2 February 2024)

## Added: `NormalizedPath` and `PathElement` types ([#78])

The `NormalizedPath` struct represents the location of a node within a JSON object. Its representation is like so:

```rust
pub struct NormalizedPath<'a>(Vec<PathElement<'a>);

pub enum PathElement<'a> {
    Name(&'a str),
    Index(usize),
}
```

Several methods were included to interact with a `NormalizedPath`, e.g., `first`, `last`, `get`, `iter`, etc., but notably there is a `to_json_pointer` method, which allows direct conversion to a JSON Pointer to be used with the [`serde_json::Value::pointer`][pointer] or [`serde_json::Value::pointer_mut`][pointer-mut] methods.

[pointer]: https://docs.rs/serde_json/latest/serde_json/enum.Value.html#method.pointer
[pointer-mut]: https://docs.rs/serde_json/latest/serde_json/enum.Value.html#method.pointer_mut

The new `PathElement` type also comes equipped with several methods, and both it and `NormalizedPath` have eagerly implemented traits from the standard library / `serde` to help improve interoperability.

## Added: `LocatedNodeList` and `LocatedNode` types ([#78])

The `LocatedNodeList` struct was built to have a similar API surface to the `NodeList` struct, but includes additional methods that give access to the location of each node produced by the original query. For example, it has the `locations` and `nodes` methods to provide dedicated iterators over locations or nodes, respectively, but also provides the `iter` method to iterate over the location/node pairs. Here is an example:

```rust
use serde_json::{json, Value};
use serde_json_path::JsonPath;
let value = json!({"foo": {"bar": 1, "baz": 2}});
let path = JsonPath::parse("$.foo.*")?;
let query = path.query_located(&value);
let nodes: Vec<&Value> = query.nodes().collect();
assert_eq!(nodes, vec![1, 2]);
let locs: Vec<String> = query
    .locations()
    .map(|loc| loc.to_string())
    .collect();
assert_eq!(locs, ["$['foo']['bar']", "$['foo']['baz']"]);
```

The location/node pairs are represented by the `LocatedNode` type.

The `LocatedNodeList` provides one unique bit of functionality over `NodeList`: deduplication of the query results, via the `LocatedNodeList::dedup` and `LocatedNodeList::dedup_in_place` methods.

[#78]: https://github.com/hiltontj/serde_json_path/pull/78

## Other Changes

- **internal**: address new clippy lints in Rust 1.75 ([#75])
- **internal**: address new clippy lints in Rust 1.74 and update some tracing instrumentation ([#70])
- **internal**: code clean-up ([#72])

[#70]: https://github.com/hiltontj/serde_json_path/pull/70
[#72]: https://github.com/hiltontj/serde_json_path/pull/72
[#75]: https://github.com/hiltontj/serde_json_path/pull/75

# 0.1.3 (9 November 2023)

- **added**: `is_empty`, `is_more_than_one`, and `as_more_than_one` methods to `ExactlyOneError` ([#65])
- **fixed**: ensure that the check `== -0` in filters works as expected ([#67]) 

[#65]: https://github.com/hiltontj/serde_json_path/pull/65
[#67]: https://github.com/hiltontj/serde_json_path/pull/67

# 0.1.2 (17 September 2023)

- **documentation**: Improvements to documentation ([#56])

[#56]: https://github.com/hiltontj/serde_json_path/pull/56

# 0.1.1 (13 July 2023)

* **fixed**: Fixed an issue in the evaluation of `SingularQuery`s that was producing false positive query results when relative singular queries, e.g., `@.bar`, were being used as comparables in a filter, e.g., `$.foo[?(@.bar == 'baz')]` ([#50])

[#50]: https://github.com/hiltontj/serde_json_path/pull/50

# 0.1.0 (2 April 2023)

Initial Release

