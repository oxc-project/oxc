pub mod debug_print;
pub mod entrypoint;
pub mod fixture_utils;
pub mod timing;

// Re-export from new crates for backwards compatibility
pub use crate::react_compiler_diagnostics;
pub use crate::react_compiler_hir;
pub use crate::react_compiler_hir as hir;
pub use crate::react_compiler_hir::environment;
pub use crate::react_compiler_lowering::lower;
