#![allow(clippy::missing_errors_doc)]

mod ast_impl;
mod diagnostics;
mod options;
mod parser;
mod surrogate_pair;

mod generated {
    mod derive_clone_in;
    mod derive_content_eq;
    mod derive_content_hash;
}

pub mod ast;
pub use crate::{ast_impl::visit, options::ParserOptions, parser::Parser};
