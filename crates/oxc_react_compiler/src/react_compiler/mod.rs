#[cfg(feature = "debug")]
pub mod debug_print;

/// No-op stand-in for [`debug_print`] when the `debug` feature is off, so the HIR
/// debug-print call sites still compile without pulling in the ~600-line printer.
/// (The runtime `PluginOptions::debug` flag only produces output with the feature on.)
#[cfg(not(feature = "debug"))]
pub mod debug_print {
    use crate::react_compiler_hir::HirFunction;
    use crate::react_compiler_hir::environment::Environment;

    pub fn debug_hir<'h>(_hir: &HirFunction<'h>, _env: &Environment<'h>) -> String {
        String::new()
    }
}

pub mod entrypoint;
