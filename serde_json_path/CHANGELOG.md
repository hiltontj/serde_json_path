# Changelog

All noteable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# Unreleased

- **fixed**: edge case where `.` in regexes for `match` and `search` functions was matching `\r\n` properly ([#92])

[#92]: https://github.com/hiltontj/serde_json_path/pull/92

# 0.6.7 (3 March 2024)

- **testing**: support tests for non-determinism in compliance test suite ([#85])
- **fixed**: bug preventing registered functions from being used as arguments to other functions ([#84])

[#85]: https://github.com/hiltontj/serde_json_path/pull/85
[#84]: https://github.com/hiltontj/serde_json_path/pull/84

# 0.6.6 (23 February 2024)

- **docs**: update links to refer to RFC 9535 ([#81])

[#81]: https://github.com/hiltontj/serde_json_path/pull/81

# 0.6.5 (2 February 2024)

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
- **internal**: address new clippy lints in Rust 1.74 ([#70])
- **internal**: code clean-up ([#72])

[#70]: https://github.com/hiltontj/serde_json_path/pull/70
[#72]: https://github.com/hiltontj/serde_json_path/pull/72
[#75]: https://github.com/hiltontj/serde_json_path/pull/75

# 0.6.4 (9 November 2023)

- **added**: `is_empty`, `is_more_than_one`, and `as_more_than_one` methods to `ExactlyOneError` ([#65])
- **fixed**: allow whitespace before dot-name selectors ([#67])
- **fixed**: ensure that the check `== -0` in filters works as expected ([#67]) 

[#65]: https://github.com/hiltontj/serde_json_path/pull/65
[#67]: https://github.com/hiltontj/serde_json_path/pull/67

# 0.6.3 (17 September 2023)

- **documentation**: Add line describing Descendant Operator ([#53])
- **documentation**: Improve example in Filter Selector section of main docs ([#54])
- **documentation**: Improve examples in Slice Slector section of main docs ([#55])
- **documentation**: Other improvements to documentation ([#56])
- **fixed**: Formulate the regex used by the `match` function to correctly handle regular expressions with leading or trailing `|` characters ([#61])

[#53]: https://github.com/hiltontj/serde_json_path/pull/53
[#54]: https://github.com/hiltontj/serde_json_path/pull/54
[#55]: https://github.com/hiltontj/serde_json_path/pull/55
[#56]: https://github.com/hiltontj/serde_json_path/pull/56
[#61]: https://github.com/hiltontj/serde_json_path/pull/61

# 0.6.2 (13 July 2023)

* **fixed**: Fixed an issue in the evaluation of `SingularQuery`s that was producing false positive query results when relative singular queries, e.g., `@.bar`, were being used as comparables in a filter, e.g., `$.foo[?(@.bar == 'baz')]` ([#50])

[#50]: https://github.com/hiltontj/serde_json_path/pull/50

# 0.6.1 (5 July 2023)

* **documentation**: Updated links to JSONPath specification to latest version (base 14) ([#43])
* **fixed**: Support newline characters in query strings where previously they were not being supported ([#44])

[#43]: https://github.com/hiltontj/serde_json_path/pull/43
[#44]: https://github.com/hiltontj/serde_json_path/pull/44

# 0.6.0 (2 April 2023)

## Function Extensions ([#32])

This release introduces the implementation of [Function Extensions][jpspec_func_ext] in `serde_json_path`.

This release ships with support for the standard built-in functions that are part of the base JSONPath specification:

- `length`
- `count`
- `match`
- `search`
- `value`

These can now be used in your JSONPath query filter selectors, and are defined in the crate documentation
in the `functions` module.

In addition, the `#[function]` attribute macro was introduced to enable users of `serde_json_path` to define
their own custom functions for use in their JSONPath queries.

### The `functions` module (**added**)

In addition to the documentation/definitions for built-in functions, the `functions` module includes three new types:

- `ValueType` 
- `NodesType`
- `LogicalType` 

These reflect the type system defined in the JSONPath spec. Each is available through the public API, to be used in custom
function definitions, along with the `#[function]` attribute macro.

### The `#[function]` attribute macro (**added**)

A new attribute macro: `#[function]` was introduced to allow users of `serde_json_path` to define their
own custom functions for use in their JSONPath queries.

Along with the new types introduced by the `functions` module, it can be used like so:

```rust
use serde_json_path::functions::{NodesType, ValueType};

/// A function that takes a node list, and optionally produces the first element as
/// a value, if there are any elements in the list.
#[serde_json_path::function]
fn first(nodes: NodesType) -> ValueType {
    match nodes.first() {
        Some(v) => ValueType::Node(v),
        None => ValueType::Nothing,
    }
}
```

Which will then allow you to use a `first` function in your JSONPath queries:

```
$[? first(@.*) > 5 ]
```

Usage of `first` in you JSONPath queries, like any of the built-in functions, will be validated at parse-time.

The `#[function]` macro is gated behind the `functions` feature, which is enabled by default.

Functions defined using the `#[function]` macro will override any of the built-in functions that are part
of the standard, e.g., `length`, `count`, etc.

### Changed the `Error` type (**breaking**)

The `Error` type was renamed to `ParseError` and was updated to have more concise error messages. It was
refactored internally to better support future improvements to the parser. It is now a struct, vs. an enum,
with a private implementation, and two core APIs:

- `message()`: the parser error message
- `position()`: indicate where the parser error was encountered in the JSONPath query string

This gives far more concise errors than the pre-existing usage of `nom`'s built-in `VerboseError` type.
However, for now, this leads to somewhat of a trade-off, in that errors that are not specially handled
by the parser will present as just `"parser error"` with a position. Over time, the objective is to
isolate cases where specific errors can be propagated up, and give better error messages.

### Repository switched to a workspace

With this release, `serde_json_path` is split into four separate crates:

- `serde_json_path`
- `serde_json_path_macros`
- `serde_json_path_macros_internal`
- `serde_json_path_core`

`serde_json_path` is still the entry point for general consumption. It still contains some of the key
components of the API, e.g., `JsonPath`, `JsonPathExt`, and `Error`, as well as the entire `parser` module.
However, many of the core types used to represent the JSONPath model, as defined in the specification,
were moved into `serde_json_path_core`.

This split was done to accommodate the new `#[function]` attribute macro, which is defined within the
`serde_json_path_macros`/`macros_internal` crates, and discussed below.

[#32]: https://github.com/hiltontj/serde_json_path/pull/32
[jpspec_func_ext]: https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-14.html#name-function-extensions

## Other Changes

- **added:** updated to latest version of CTS to ensure compliance [#33]
- **added:** implement `Eq` for `JsonPath` [#34]
- **breaking:**: Changed the name of `Error` type to `ParseError` [#36]

[#33]: https://github.com/hiltontj/serde_json_path/pull/33
[#34]: https://github.com/hiltontj/serde_json_path/pull/34
[#36]: https://github.com/hiltontj/serde_json_path/pull/36

# 0.5.3 (14 March 2023)

- **fixed:** Fix serialization behaviour of `NodeList` ([#30])

[#30]: https://github.com/hiltontj/serde_json_path/pull/30

# 0.5.2 (13 March 2023)

- **added:** Add `first`, `last`, and `get` methods to `NodeList` type ([#16])
- **changed:** Make `NodeList::at_most_one` and `NodeList::exactly_one` take `&self` instead of `self` ([#16])
- **docs:** Update crate-level docs to better reflect recent changes ([#21])
- **docs:** Corrected a broken link in crate-level docs ([#21])
- **added:** derive `Clone` for `JsonPath` and its descendants ([#24])
- **added:** derive `Default` for `JsonPath` ([#25])
- **added:** implement `Display`  and `Serialize` for `JsonPath` ([#26])

[#16]: https://github.com/hiltontj/serde_json_path/pull/16
[#21]: https://github.com/hiltontj/serde_json_path/pull/21
[#24]: https://github.com/hiltontj/serde_json_path/pull/24
[#25]: https://github.com/hiltontj/serde_json_path/pull/25
[#26]: https://github.com/hiltontj/serde_json_path/pull/26

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
