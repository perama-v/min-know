# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Nil

## [0.2.0] - 2022-01-16

### Added

- Implement database: 4 byte signature support (as `SignaturesSpec`).

### Changed

- Replaced ssz crate with ssz_rs to have natural constants rather than Typenums.
- Moved manifest related code to manifest module for separation of serialization.

### Fix

- Fixed incorrect check of raw nametag filename matching a chapter.
- *Chapter.records struct members now use SSZ List for spec conformity.

## [0.1.0] - 2022-01-09

### Added

- Make library generic over different databases.
- Implement database: address appearance index.
- Implement database: nametags.

[unreleased]: https://github.com/perama-v/min-know/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/perama-v/min-know/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/perama-v/min-know/releases/tag/v0.1.0