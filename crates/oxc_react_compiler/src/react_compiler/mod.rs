#[cfg(feature = "debug")]
pub mod debug_print;

/// No-op stand-in for [`debug_print`] when the `debug` feature is off, so the HIR
/// debug-print call sites still compile without pulling in the ~600-line printer.
/// (The runtime `PluginOptions::debug` flag only produces output with the feature on.)
#[cfg(not(feature = "debug"))]
pub mod debug_print {
    use crate::react_compiler_hir::HirFunction;
    use crate::react_compiler_hir::environment::Environment;
    use crate::react_compiler_hir::print::PrintFormatter;

    pub fn debug_hir<'h>(_hir: &HirFunction<'h>, _env: &Environment<'h>) -> String {
        String::new()
    }

    pub fn format_hir_function_into<'h>(
        _reactive_fmt: &mut PrintFormatter<'_, 'h>,
        _func: &HirFunction<'h>,
    ) {
    }
}

pub mod entrypoint;

// Re-export from new crates for backwards compatibility
pub use crate::react_compiler_diagnostics;
pub use crate::react_compiler_hir;
pub use crate::react_compiler_hir as hir;
pub use crate::react_compiler_hir::environment;
pub use crate::react_compiler_lowering::lower;
