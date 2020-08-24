[![embed-wasm-build on crates.io](https://img.shields.io/crates/v/embed-wasm-build)](https://crates.io/crates/embed-wasm-build) [![Documentation (latest release)](https://docs.rs/embed-wasm-build/badge.svg)](https://docs.rs/embed-wasm-build/) [![Documentation (master)](https://img.shields.io/badge/docs-master-brightgreen)](https://inanna-malick.github.io/embed-wasm/embed-wasm-build/)[![License](https://img.shields.io/badge/license-MIT-green.svg)](../LICENSE)

# embed-wasm-build

Current version: 0.1.0-alpha

This crate provides compile-time utilities for packaging 'cargo-web' build output
(rust compiled as wasm and associated html/css/etc files) inside native binaries
and is meant to be invoked from custom build.rs scripts

Designed for use with the [`embed-wasm` crate](https://crates.io/crates/embed-wasm).
See [embed-wasm-example](https://github.com/inanna-malick/embed-wasm-example) for a full example.

## License

MIT

<!--
README.md is generated from README.tpl by cargo readme. To regenerate:
cargo install cargo-readme
cargo readme > README.md
-->
