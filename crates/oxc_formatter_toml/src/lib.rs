//! TOML formatter built on top of `oxc_formatter_core`, parsing with [oxc-toml](https://docs.rs/oxc-toml).

mod context;
mod format;
mod options;
mod print;

pub use crate::{
    context::TomlFormatContext,
    format::{format, format_to_ir},
    options::{TomlFormatOptions, TrailingCommas},
};
