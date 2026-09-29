//! Data which is static, but too large to store as `static`s in the binary.
//!
//! Instead, it's generated and heap-allocated at runtime, one copy per [`Lexer`]
//! i.e. one copy per thread.
//!
//! [`Lexer`]: super::Lexer

use crate::pipeline::keywords::Keywords;

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
))]
use crate::pipeline::compress::PairLuts;

/// Static data that's too large to store as `static`s in the binary.
pub(super) struct Tables {
    pub keywords: Keywords,

    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ))]
    pub pair_luts: PairLuts,
}

impl Tables {
    /// Create [`Tables`].
    pub fn new() -> Tables {
        Self {
            keywords: Keywords::new(),
            #[cfg(all(
                target_arch = "x86_64",
                target_feature = "avx2",
                target_feature = "bmi2",
                target_feature = "popcnt"
            ))]
            pair_luts: PairLuts::new(),
        }
    }
}
