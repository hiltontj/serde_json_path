# Changelog

All noteable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# Unreleased

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

