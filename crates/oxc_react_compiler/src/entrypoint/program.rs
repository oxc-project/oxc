
use crate::{
    compiler_error::{CompilerError, ErrorSeverity},
    entrypoint::options::{
        CompilationMode, PanicThreshold,
        OPT_IN_DIRECTIVES, OPT_OUT_DIRECTIVES,
    },
    hir::ReactFunctionType,
    utils::{component_declaration::is_component_name, hook_declaration::is_hook_name},
};

/// Result of compiling a program.
#[derive(Debug)]
pub struct ProgramCompilationResult {
    /// Number of functions that were successfully compiled.
    pub compiled_count: u32,
    /// Number of functions that were skipped.
    pub skipped_count: u32,
    /// Number of functions that errored.
    pub error_count: u32,
}

/// Determine if a function should be compiled based on the compilation mode.
pub fn should_compile_function(
    name: Option<&str>,
    directives: &[String],
    mode: CompilationMode,
) -> Option<ReactFunctionType> {
    // Check for opt-out directives
    for directive in directives {
        if OPT_OUT_DIRECTIVES.contains(&directive.as_str()) {
            return None;
        }
    }

    // Check for opt-in directives
    let has_opt_in = directives
        .iter()
        .any(|d| OPT_IN_DIRECTIVES.contains(&d.as_str()));

    match mode {
        CompilationMode::Annotation => {
            // Only compile if explicitly annotated
            if has_opt_in { Some(ReactFunctionType::Other) } else { None }
        }
        CompilationMode::All => {
            // Compile everything
            Some(infer_function_type(name))
        }
        CompilationMode::Infer => {
            // Compile if annotated, or if it looks like a component/hook
            if has_opt_in {
                return Some(infer_function_type(name));
            }
            match name {
                Some(n) if is_component_name(n) => Some(ReactFunctionType::Component),
                Some(n) if is_hook_name(n) => Some(ReactFunctionType::Hook),
                _ => None,
            }
        }
        CompilationMode::Syntax => {
            // Only compile component/hook syntax declarations
            // In the Babel version, this checks for Flow component/hook syntax
            // In Rust, we only have standard JS function syntax
            if has_opt_in {
                return Some(infer_function_type(name));
            }
            None
        }
    }
}

/// Infer the function type from its name.
fn infer_function_type(name: Option<&str>) -> ReactFunctionType {
    match name {
        Some(n) if is_component_name(n) => ReactFunctionType::Component,
        Some(n) if is_hook_name(n) => ReactFunctionType::Hook,
        _ => ReactFunctionType::Other,
    }
}

/// Check if a function has a directive that enables memoization.
pub fn find_directive_enabling_memoization(
    directives: &[String],
) -> Option<String> {
    directives
        .iter()
        .find(|d| OPT_IN_DIRECTIVES.contains(&d.as_str()))
        .cloned()
}

/// Check if a function has a directive that disables memoization.
pub fn find_directive_disabling_memoization(
    directives: &[String],
    custom_opt_out: Option<&[String]>,
) -> Option<String> {
    // Check custom opt-out directives first
    if let Some(custom) = custom_opt_out {
        if let Some(found) = directives.iter().find(|d| custom.contains(d)) {
            return Some(found.clone());
        }
    }
    // Then check standard opt-out
    directives
        .iter()
        .find(|d| OPT_OUT_DIRECTIVES.contains(&d.as_str()))
        .cloned()
}

/// Determine how to handle a compilation error based on the panic threshold.
pub fn handle_compilation_error(
    error: &CompilerError,
    threshold: PanicThreshold,
) -> ErrorAction {
    match threshold {
        PanicThreshold::AllErrors => ErrorAction::Panic,
        PanicThreshold::CriticalErrors => {
            if error.has_errors() {
                // Check if any errors are critical (invariants, config)
                let has_critical = error.details.iter().any(|detail| {
                    let severity = detail.severity();
                    severity == ErrorSeverity::Error
                });
                if has_critical {
                    ErrorAction::Panic
                } else {
                    ErrorAction::Skip
                }
            } else {
                ErrorAction::Skip
            }
        }
        PanicThreshold::None => ErrorAction::Skip,
    }
}

/// Action to take when encountering a compilation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorAction {
    /// Panic (throw/propagate the error).
    Panic,
    /// Skip this function and continue.
    Skip,
}
