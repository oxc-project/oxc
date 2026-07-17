#![doc = include_str!("../README.md")]
#![allow(rustdoc::bare_urls, rustdoc::broken_intra_doc_links, rustdoc::invalid_rust_codeblocks)]

pub mod ast;
mod lexer;

mod error;
mod limit;
mod parser_ast;

pub use crate::error::Error;
pub use crate::lexer::Lexer;
pub use crate::lexer::Token;
pub use crate::lexer::TokenKind;
pub use crate::limit::LimitTracker;
pub use crate::parser_ast::Parser;
pub use oxc_allocator::Allocator;
