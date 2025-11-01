# desert-rust Agent Guide

## Build/Lint/Test Commands

- **Build all**: `cargo build`
- **Test all**: `cargo test`
- **Test single**: `cargo test --package <crate> --lib -- <test_name>` (e.g., `cargo test --package desert_core --lib -- roundtrip_i32`)
- **Lint**: `cargo clippy --no-deps --all-targets -- -Dwarnings`
- **Format check**: `cargo fmt --all -- --check`
- **Format fix**: `cargo fmt --all`
- **Dependency check**: `cargo deny check`

## Architecture & Structure

This is a Rust workspace for **desert-rust**, a binary serialization library with schema evolution support.

**Key crates:**
- `desert`: Main library exposing public API
- `desert_core`: Core serialization/deserialization logic, error types, context management
- `desert_macro`: Procedural macros for `#[derive(BinaryCodec)]`
- `desert_benchmarks`: Performance benchmarks
- `github`: CI/GitHub Actions utilities

**Features:**
- Binary serialization with schema evolution (field additions/removals)
- String deduplication mode
- Reference tracking for cyclic data structures
- Property-based testing with proptest

**No external databases or services.**

## Code Style Guidelines

**Naming:**
- Functions/variables: `snake_case`
- Types/structs/enums: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

**Imports:**
```rust
// Group: std, external crates, local crates
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::error::Result;
```

**Error handling:**
- Use `Result<T>` with `?` operator
- Define custom `Error` enum in `error.rs`
- Panic only for programmer errors, not runtime failures

**Types:**
- Use `#[derive(Debug, Clone, Eq, PartialEq, Hash)]` for data structs
- Implement `BinarySerializer`/`BinaryDeserializer` for serialization
- Use `lazy_static!` for static metadata/constants

**Testing:**
- Use `test_r` framework: `test_r::enable!()`
- Property-based tests with `proptest!`
- Roundtrip tests for serialization correctness
- Integration tests in `tests/` directory
