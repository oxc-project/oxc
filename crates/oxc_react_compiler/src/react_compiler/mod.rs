#[cfg(feature = "debug")]
pub mod debug_print;
/// Stub when the `debug` feature is off: the pipeline still calls these in its
/// `if debug_enabled` blocks, so keep the signatures but drop the IR printing.
#[cfg(not(feature = "debug"))]
pub mod debug_print {
    use crate::react_compiler_hir::HirFunction;
    use crate::react_compiler_hir::environment::Environment;
    use crate::react_compiler_hir::print::PrintFormatter;

    pub fn debug_hir(_hir: &HirFunction, _env: &Environment) -> String {
        String::new()
    }

    pub fn format_hir_function_into(_fmt: &mut PrintFormatter, _func: &HirFunction) {}
}
pub mod entrypoint;
pub mod timing;

// Re-export from new crates for backwards compatibility
pub use crate::react_compiler_diagnostics;
pub use crate::react_compiler_hir;
pub use crate::react_compiler_hir as hir;
pub use crate::react_compiler_hir::environment;
pub use crate::react_compiler_lowering::lower;
