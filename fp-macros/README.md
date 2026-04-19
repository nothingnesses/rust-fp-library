# fp-macros

[![crates.io](https://img.shields.io/crates/v/fp-macros.svg)](https://crates.io/crates/fp-macros)
[![docs.rs](https://docs.rs/fp-macros/badge.svg)](https://docs.rs/fp-macros)
[![GitHub License](https://img.shields.io/github/license/nothingnesses/rust-fp-library?color=blue)](https://github.com/nothingnesses/rust-fp-library/blob/main/LICENSE)

Procedural macros for the [`fp-library`](https://github.com/nothingnesses/rust-fp-library) crate.

This crate provides a suite of macros designed to facilitate working with Higher-Kinded Types (HKT) in Rust. It automates the generation of `Kind` traits, simplifies their implementation for specific `Brand` types, and provides a convenient syntax for type application.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fp-macros = "0.8"
```

> **Note:** If you are using [`fp-library`](https://crates.io/crates/fp-library), these macros are already re-exported at the crate root. You only need to add this dependency if you are using the macros independently.

## License

This project is licensed under the [Blue Oak Model License 1.0.0](../LICENSE).
