// `oxc_lexer` does not support big endian at present
#![cfg(target_endian = "little")]

pub mod error;
pub mod options;
mod pipeline;
mod token;
mod tokens;

#[cfg(feature = "oxc_diagnostics")]
pub mod diagnostics;
#[cfg(feature = "testing")]
pub mod testing;

pub use error::{DiagCode, DiagSeverity, Diagnostic};
pub use options::LexOptions;
pub use pipeline::Lexer;
pub use token::{KW_KIND_BASE, SPAN_SENTINELS, TRIVIA_MAX, TRIVIA_MIN, TokenKind, token_flags};
pub use tokens::Tokens;

/// Number of bytes of zeros that must follow source text in `src` param passed to [`Lexer::lex`].
pub const PAD: usize = 64;
