#![allow(clippy::missing_errors_doc)]

mod ast_impl;
mod body_parser;
mod diagnostics;
mod flags_parser;
mod literal_parser;
mod options;
mod span_factory;
mod surrogate_pair;

mod generated {
    mod derive_clone_in;
    mod derive_content_eq;
    mod derive_content_hash;
}

pub mod ast;
pub use crate::{
    ast_impl::visit, body_parser::PatternParser, flags_parser::FlagsParser, literal_parser::Parser,
    options::ParserOptions,
};
