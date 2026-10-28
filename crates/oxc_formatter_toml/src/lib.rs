//! TOML formatter wrapping [oxc-toml](https://docs.rs/oxc-toml), a formatter-only fork of Taplo.

mod format;
mod options;

pub use crate::{
    format::{format, format_to_ir},
    options::{TomlFormatOptions, TrailingCommas},
};
