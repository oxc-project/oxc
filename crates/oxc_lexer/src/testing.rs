//! Functions for testing the lexer with each CPU feature set.
//!
//! [`DynCpu::for_each_supported`] gives each CPU feature set which this CPU supports.
//!
//! Only available with the `testing` Cargo feature.

use oxc_simd::DynCpu;

use crate::{Arena, LexOptions, LexResult, SCRATCH, lex_utf8};

/// Same as [`lex_utf8`], but lexes with CPU feature set `cpu`,
/// instead of the best one this CPU supports.
///
/// # Panics
/// Panics if `src` does not extend at least [`PAD`] zeroed bytes past `len`.
///
/// [`PAD`]: crate::PAD
pub fn lex_utf8_with_cpu(
    src: &[u8],
    len: u32,
    options: LexOptions,
    cpu: DynCpu,
) -> (LexResult, Arena) {
    let _switch = SwitchCpu::new(cpu);
    lex_utf8(src, len, options)
}

/// Guard which switches the thread's [`Lexer`] to a different CPU feature set,
/// and switches it back when dropped, even if lexing panics.
///
/// [`Lexer`]: crate::Lexer
struct SwitchCpu {
    /// The CPU feature set to switch back to.
    previous: DynCpu,
}

impl SwitchCpu {
    /// Switch the thread's [`Lexer`] to CPU feature set `cpu`.
    ///
    /// [`Lexer`]: crate::Lexer
    fn new(cpu: DynCpu) -> Self {
        let previous = SCRATCH.with(|cell| cell.borrow_mut().replace_cpu(cpu));
        Self { previous }
    }
}

impl Drop for SwitchCpu {
    /// Switch the thread's [`Lexer`] back to the CPU feature set it had before.
    ///
    /// [`Lexer`]: crate::Lexer
    fn drop(&mut self) {
        SCRATCH.with(|cell| cell.borrow_mut().replace_cpu(self.previous));
    }
}
