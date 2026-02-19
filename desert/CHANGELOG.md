# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.7](https://github.com/vigoo/desert-rust/compare/desert_rust-v0.1.6...desert_rust-v0.1.7) - 2026-02-19

### Other

- Warning fix
- Removed the stable-intrinsics dep
- Merge pull request #27 from vigoo/no-stable-intrinsics

## [0.1.6](https://github.com/vigoo/desert-rust/compare/desert_rust-v0.1.5...desert_rust-v0.1.6) - 2025-11-16

### Other

- Extend the transparent variant feature to work on unit cases
- Clippy fixes

## [0.1.5](https://github.com/vigoo/desert-rust/compare/desert_rust-v0.1.4...desert_rust-v0.1.5) - 2025-11-16

### Other

- Ability to write custom serializers for transparent variant cases
- More optimizations
- Serializer opts
- Further deser opt
- Better benchmark, v0 deser optimizations

## [0.1.4](https://github.com/vigoo/desert-rust/compare/desert_rust-v0.1.3...desert_rust-v0.1.4) - 2025-11-11

### Fixed

- Fixes fixed length array codec

### Other

- clippy fixes
- clippy fixes

## [0.1.3](https://github.com/vigoo/desert-rust/compare/desert_rust-v0.1.2...desert_rust-v0.1.3) - 2025-11-11

### Added

- Support options and support all character values

## [0.1.2](https://github.com/vigoo/desert-rust/compare/desert_rust-v0.1.1...desert_rust-v0.1.2) - 2025-11-06

### Fixed

- support raw identifiers in type names

## [0.1.1](https://github.com/vigoo/desert-rust/compare/desert_rust-v0.1.0...desert_rust-v0.1.1) - 2025-11-05

### Other

- MacAddress
- Url and BitVec support
- Support generics
- IpAddr and serde-json support
- Support transparent record-style structs
- nonempty-collections feature
- Moved other properties to desert() attribute
- Introduced desert(transparent)
- Moved tests to desert
- Range
- Bound
- Downgrade bitvec
- Clippy
- VecDeque
- nonzero numeric types
- usize codec
- Reexported lazy_static
- Some more benchmarks
- Updates
- Generated code optimization
- Macro fix
- Ambiguity fix
