#![allow(clippy::missing_errors_doc)]

pub mod ast;
mod body_parser;
mod display;
mod flag_parser;
mod literal_parser;
mod options;
mod span;
mod surrogate_pair;

mod generated {
    mod derive_clone_in;
    mod derive_content_eq;
    mod derive_content_hash;
}

pub use crate::{
    body_parser::PatternParser, flag_parser::FlagsParser, literal_parser::Parser,
    options::ParserOptions,
};
