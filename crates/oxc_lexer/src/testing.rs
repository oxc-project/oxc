//! Functions for testing the lexer with each CPU feature set.
//!
//! [`DynCpu::for_each_supported`] gives each CPU feature set which this CPU supports.
//!
//! Only available with the `testing` Cargo feature.

use oxc_simd::DynCpu;

use crate::{LexOptions, Lexer, Tokens};

/// Same as Lexer::lex, but lexes with CPU feature set cpu,
/// instead of the best one this CPU supports.
///
/// # Panics
/// Panics if `src` does not extend at least [`PAD`] zeroed bytes past `len`.
///
/// [`PAD`]: crate::PAD
pub fn lex_with_cpu(src: &[u8], len: u32, options: LexOptions, cpu: DynCpu) -> Tokens {
    let mut tokens = Tokens::new();
    Lexer::new_with_cpu(cpu).lex(src, len as usize, options, &mut tokens);
    tokens
}
