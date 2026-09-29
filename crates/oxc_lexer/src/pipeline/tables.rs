//! Data which is static, but too large to store as `static`s in the binary.
//!
//! Instead, it's generated at runtime on first use, heap-allocated, and stored in a `static` `OnceLock`.
//! So there's one copy per process, shared by all threads.

use std::sync::OnceLock;

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
))]
use crate::pipeline::compress::PairLuts;

/// Single copy of [`Tables`], shared across all threads.
///
/// `Box<Tables>` not `Tables`, to avoid uninitialized `Tables` being stored
/// in the binary's data section, which would bloat binary by ~22 KB.
static TABLES: OnceLock<Box<Tables>> = OnceLock::new();

/// Static data that's too large to store as `static`s in the binary.
pub(super) struct Tables {
    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ))]
    pub pair_luts: PairLuts,
}

impl Tables {
    /// Get reference to [`Tables`].
    ///
    /// `Tables` is created on the first call, and shared by all threads after that.
    pub fn get() -> &'static Tables {
        TABLES.get_or_init(|| Box::new(Tables::new()))
    }

    /// Create [`Tables`].
    fn new() -> Tables {
        Self {
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
