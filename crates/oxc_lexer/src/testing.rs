//! Functions for testing the lexer with each CPU feature set.
//!
//! [`DynCpu::for_each_supported`] gives each CPU feature set which this CPU supports.
//!
//! Only available with the `testing` Cargo feature.

use oxc_simd::DynCpu;

use crate::{LexOptions, Lexer, PAD, Tokens};

/// Same as [`Lexer::lex`], but lexes with CPU feature set `cpu`,
/// instead of the best one this CPU supports.
///
/// Unlike [`Lexer::lex`], `src` does not need padding after it.
/// It is copied into a buffer with [`PAD`] zeroed bytes after it.
///
/// `src` can be a `&str`, or bytes which may not be valid UTF-8
/// (for testing [`LexOptions::validate_utf8`]).
#[inline]
pub fn lex_with_cpu(src: impl AsRef<[u8]>, options: LexOptions, cpu: DynCpu) -> Tokens {
    lex_with_cpu_impl(src.as_ref(), options, cpu)
}

/// Implementation of [`lex_with_cpu`], taking a slice so it's not generic.
fn lex_with_cpu_impl(src: &[u8], options: LexOptions, cpu: DynCpu) -> Tokens {
    let len = src.len();
    let mut buf = Vec::with_capacity(len + PAD);
    buf.extend_from_slice(src);
    buf.resize(len + PAD, 0);

    let mut tokens = Tokens::new();
    Lexer::new_with_cpu(cpu).lex(&buf, len, options, &mut tokens);
    tokens
}
