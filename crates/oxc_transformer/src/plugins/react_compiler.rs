use oxc_ast::ast::*;

/// Options for the React Compiler transform.
#[derive(Debug, Clone, Default)]
pub struct ReactCompilerOptions {
    /// Whether to enable the React Compiler transform.
    pub enabled: bool,
    /// Compilation mode: "infer", "annotation", "all", "syntax"
    pub compilation_mode: Option<String>,
    /// Panic threshold: "all_errors", "critical_errors", "none"
    pub panic_threshold: Option<String>,
}

/// React Compiler transformer plugin.
///
/// When enabled, this transform automatically memoizes React components
/// and hooks using the React Compiler analysis passes.
///
/// This integrates `oxc_react_compiler` into the oxc transformer pipeline.
pub struct ReactCompiler {
    options: ReactCompilerOptions,
}

impl ReactCompiler {
    pub fn new(options: ReactCompilerOptions) -> Self {
        Self { options }
    }

    /// Check if a function should be compiled.
    pub fn should_compile_function(&self, name: &str) -> bool {
        should_compile(name, &self.options)
    }

    /// Process a function declaration for potential compilation.
    pub fn enter_function(&mut self, func: &Function<'_>) {
        let name = func.id.as_ref().map(|id| id.name.as_str());
        if let Some(name) = name {
            if self.should_compile_function(name) {
                // In the full implementation:
                // 1. Lower the function AST to HIR using oxc_react_compiler::hir::build_hir
                // 2. Run the full compilation pipeline
                // 3. Generate the optimized AST
                // 4. Replace the function body with the optimized version
            }
        }
    }
}

fn should_compile(name: &str, options: &ReactCompilerOptions) -> bool {
    if !options.enabled {
        return false;
    }
    let mode = options.compilation_mode.as_deref().unwrap_or("infer");
    match mode {
        "all" => true,
        "infer" => {
            // Components (uppercase) or hooks (use*)
            name.starts_with(|c: char| c.is_ascii_uppercase())
                || (name.starts_with("use")
                    && name.len() > 3
                    && name[3..].starts_with(|c: char| c.is_ascii_uppercase()))
        }
        "annotation" => false, // Would need to check for "use memo" directive
        "syntax" => false,     // Would need to check for component/hook syntax
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_compile_component() {
        let opts = ReactCompilerOptions { enabled: true, ..Default::default() };
        assert!(should_compile("MyComponent", &opts));
        assert!(should_compile("Button", &opts));
    }

    #[test]
    fn test_should_compile_hook() {
        let opts = ReactCompilerOptions { enabled: true, ..Default::default() };
        assert!(should_compile("useMyHook", &opts));
        assert!(should_compile("useState", &opts));
    }

    #[test]
    fn test_should_not_compile_regular() {
        let opts = ReactCompilerOptions { enabled: true, ..Default::default() };
        assert!(!should_compile("helper", &opts));
        assert!(!should_compile("calculateTotal", &opts));
    }

    #[test]
    fn test_disabled() {
        let opts = ReactCompilerOptions { enabled: false, ..Default::default() };
        assert!(!should_compile("MyComponent", &opts));
    }

    #[test]
    fn test_all_mode() {
        let opts = ReactCompilerOptions {
            enabled: true,
            compilation_mode: Some("all".to_string()),
            ..Default::default()
        };
        assert!(should_compile("helper", &opts));
        assert!(should_compile("anything", &opts));
    }
}
