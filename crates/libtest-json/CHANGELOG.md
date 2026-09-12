# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/)
and this project adheres to [Semantic Versioning](https://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

## [0.0.2] - 2025-09-12

- Make schema `snake_case`
- Timing is optional and is time since test start
- Made fields with well known defaults to be optional
- Made `DiscoverCase` order unspecified
- Renamed `DiscoverCase::run` to `selected`
- Rename `Suite` to `Run`
- Rename `RunStatus` to `MessageKind`
- Rename `MessageKind::Fail` to `MessageKind::Error`
- Moved content from `CaseComplete` to `CaseMessage`

## [0.0.1] - 2025-07-18

<!-- next-url -->
[Unreleased]: https://github.com/assert-rs/libtest2/compare/libtest-json-v0.0.2...HEAD
[0.0.2]: https://github.com/assert-rs/libtest2/compare/libtest-json-v0.0.1...libtest-json-v0.0.2
[0.0.1]: https://github.com/assert-rs/libtest2/compare/c96ef27899b410f9f154183989d4ccf60af27da6...libtest-json-v0.0.1
