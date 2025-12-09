# 🧩🦀 why-rs: A Causal Inference library for Rust 
![GitHub tag (latest SemVer)](https://img.shields.io/github/v/tag/jwhogg/why-rs?label=version)
[![CI](https://github.com/jwhogg/why-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/jwhogg/why-rs/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/why_rs.svg)](https://crates.io/crates/why_rs)
[![License: MIT OR Apache‑2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](./LICENSE)

### Usage

To use the crate, you can include it in your `cargo.toml`:
```rust
[dependencies]
why_rs = "0.1.0"
```

Or, get the latest version directly from github:

```rust
[dependencies]
my_crate = { git = "https://github.com/jwhogg/why-rs.git" }
```

### Running examples
Currently, there is support for the DAG and FCM (Functional Causal Model) data types.
The following functionality is supported:
- Parsing a .dot file
- Sampling from an FCM
- Causal Discovery with the PC algorithm

Sampling:
```rust
cargo run --example fcm_test
```

PC algorithm:
```rust
cargo run --example pc_test
```
