#![allow(clippy::missing_errors_doc)]

pub mod ast;
mod body_parser;
mod flag_parser;
mod literal_parser;
mod options;
mod span;

pub use crate::body_parser::PatternParser;
pub use crate::flag_parser::FlagsParser;
pub use crate::literal_parser::Parser;
pub use crate::options::ParserOptions;
