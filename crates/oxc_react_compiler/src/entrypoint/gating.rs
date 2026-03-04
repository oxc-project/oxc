/// Gating support for the React Compiler.
///
/// Port of `Entrypoint/Gating.ts` from the React Compiler.
///
/// Implements feature gating, where compiled and uncompiled versions of a
/// function are both emitted, and a runtime flag determines which to use.
///
/// Two modes are supported:
///
/// **Non-hoisted (ternary):** For functions not referenced before their declaration.
/// ```js
/// const Foo = gatingFn() ? compiledFn : originalFn;
/// ```
///
/// **Hoisted:** For function declarations referenced before their declaration site.
/// ```js
/// const gating_result = gatingFn();
/// function Foo_optimized() { /* compiled */ }
/// function Foo_unoptimized() { /* original, renamed */ }
/// function Foo(arg0) {
///   if (gating_result) return Foo_optimized(arg0);
///   else return Foo_unoptimized(arg0);
/// }
/// ```
use std::fmt;

use crate::entrypoint::imports::ProgramContext;
use crate::hir::environment::ExternalFunction;

/// The kind of the original function being gated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionKind {
    /// `function Foo(...) { ... }`
    FunctionDeclaration,
    /// `(...) => { ... }` or `(...) => expr`
    ArrowFunction,
    /// `function(...) { ... }` or `function Foo(...) { ... }` (as expression)
    FunctionExpression,
}

/// Context about the parent node, used to determine gating output shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentContext {
    /// The function is the declaration of an `export default` statement.
    ExportDefault,
    /// The function is in a regular (non-export-default) context.
    Other,
}

/// Information about a parameter for the dispatcher function in hoisted mode.
#[derive(Debug, Clone)]
pub struct ParamInfo {
    /// Whether this parameter is a rest element (`...args`).
    pub is_rest: bool,
}

/// Describes the gating output for a single function.
///
/// This struct contains all the information needed to emit the gated output
/// code, whether in hoisted or non-hoisted mode.
#[derive(Debug, Clone)]
pub enum GatingOutput {
    /// Non-hoisted: emit a ternary expression.
    ///
    /// ```js
    /// GATING_FN() ? compiledExpr : originalExpr
    /// ```
    ///
    /// The `wrap` field describes how this ternary is placed in the output:
    /// - `ConstDeclaration { name }`: `const NAME = TERNARY;`
    /// - `ExportDefaultThenConst { name }`: `const NAME = TERNARY;\nexport default NAME;`
    /// - `Inline`: just the ternary expression itself
    Ternary {
        /// The name of the gating function to call (the local import alias).
        gating_fn_name: String,
        /// How to wrap the ternary in the output.
        wrap: TernaryWrap,
    },

    /// Hoisted: emit optimized/unoptimized function declarations and a dispatcher.
    ///
    /// ```js
    /// const GATING_RESULT = GATING_FN();
    /// function NAME_optimized(...) { /* compiled */ }
    /// function NAME_unoptimized(...) { /* original */ }
    /// function NAME(arg0, arg1, ...) {
    ///   if (GATING_RESULT) return NAME_optimized(arg0, arg1, ...);
    ///   else return NAME_unoptimized(arg0, arg1, ...);
    /// }
    /// ```
    Hoisted {
        /// Original function name.
        original_name: String,
        /// Name for the compiled (optimized) version.
        optimized_name: String,
        /// Name for the original (unoptimized) version.
        unoptimized_name: String,
        /// Name of the variable holding the gating call result.
        gating_result_name: String,
        /// The name of the gating function to call (the local import alias).
        gating_fn_name: String,
        /// Parameter information for generating the dispatcher function.
        params: Vec<ParamInfo>,
    },
}

/// How a non-hoisted ternary gating expression is wrapped in output.
#[derive(Debug, Clone)]
pub enum TernaryWrap {
    /// `const NAME = TERNARY;`
    ConstDeclaration { name: String },
    /// `const NAME = TERNARY;\nexport default NAME;`
    ExportDefaultThenConst { name: String },
    /// Just the raw ternary expression (e.g. replacing an arrow in export default).
    Inline,
}

/// Build the gating output description for a function.
///
/// Port of `insertGatedFunctionDeclaration` from `Entrypoint/Gating.ts` lines 125-176.
///
/// This does not directly emit code; it returns a `GatingOutput` that describes
/// how the gated output should be structured. The caller (e.g. the transformer
/// plugin) uses this to emit the final code.
///
/// # Arguments
///
/// * `original_name` - The name of the original function (if it has one).
/// * `function_kind` - Whether the function is a declaration, arrow, or expression.
/// * `parent_context` - Whether the function is inside `export default`.
/// * `gating` - The external function config for the gating import.
/// * `referenced_before_declaration` - Whether the function is referenced before its
///   declaration site (requires hoisted mode).
/// * `params` - Parameter information for the function (needed for hoisted mode).
/// * `program_context` - The program context for managing imports and unique names.
pub fn build_gating_output(
    original_name: Option<&str>,
    function_kind: FunctionKind,
    parent_context: ParentContext,
    gating: &ExternalFunction,
    referenced_before_declaration: bool,
    params: &[ParamInfo],
    program_context: &mut ProgramContext,
) -> GatingOutput {
    let gating_fn_name = program_context.add_import_specifier(gating);

    if referenced_before_declaration && function_kind == FunctionKind::FunctionDeclaration {
        // Hoisted mode: function declarations referenced before their declaration site.
        //
        // Port of `insertAdditionalFunctionDeclaration` from Gating.ts lines 36-124.
        let name = original_name.unwrap_or("anonymous");
        let gating_result_name = program_context.new_uid(&format!("{gating_fn_name}_result"));
        let unoptimized_name = program_context.new_uid(&format!("{name}_unoptimized"));
        let optimized_name = program_context.new_uid(&format!("{name}_optimized"));

        GatingOutput::Hoisted {
            original_name: name.to_string(),
            optimized_name,
            unoptimized_name,
            gating_result_name,
            gating_fn_name,
            params: params.to_vec(),
        }
    } else {
        // Non-hoisted mode: ternary expression.
        //
        // Port of `insertGatedFunctionDeclaration` else-branch from Gating.ts lines 140-175.
        let wrap = if parent_context != ParentContext::ExportDefault
            && function_kind == FunctionKind::FunctionDeclaration
            && original_name.is_some()
        {
            // `const NAME = GATING_FN() ? compiled : original;`
            TernaryWrap::ConstDeclaration { name: original_name.unwrap_or("").to_string() }
        } else if parent_context == ParentContext::ExportDefault
            && function_kind != FunctionKind::ArrowFunction
            && original_name.is_some()
        {
            // `const NAME = GATING_FN() ? compiled : original;\nexport default NAME;`
            TernaryWrap::ExportDefaultThenConst { name: original_name.unwrap_or("").to_string() }
        } else {
            // Inline replacement (arrow functions, anonymous export default, etc.)
            TernaryWrap::Inline
        };

        GatingOutput::Ternary { gating_fn_name, wrap }
    }
}

impl GatingOutput {
    /// Format the hoisted dispatcher function body.
    ///
    /// Generates:
    /// ```js
    /// function NAME(arg0, arg1, ...argN) {
    ///   if (GATING_RESULT) return NAME_optimized(arg0, arg1, ...argN);
    ///   else return NAME_unoptimized(arg0, arg1, ...argN);
    /// }
    /// ```
    fn format_dispatcher(
        original_name: &str,
        optimized_name: &str,
        unoptimized_name: &str,
        gating_result_name: &str,
        params: &[ParamInfo],
    ) -> String {
        let param_list: Vec<String> = params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if p.is_rest {
                    format!("...arg{i}")
                } else {
                    format!("arg{i}")
                }
            })
            .collect();
        let arg_list: Vec<String> = params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if p.is_rest {
                    format!("...arg{i}")
                } else {
                    format!("arg{i}")
                }
            })
            .collect();
        let params_str = param_list.join(", ");
        let args_str = arg_list.join(", ");

        format!(
            "function {original_name}({params_str}) {{\n\
             \x20 if ({gating_result_name}) return {optimized_name}({args_str});\n\
             \x20 else return {unoptimized_name}({args_str});\n\
             }}"
        )
    }

    /// Format the gating variable declaration for hoisted mode.
    ///
    /// Generates: `const RESULT_NAME = GATING_FN();`
    fn format_gating_const(gating_result_name: &str, gating_fn_name: &str) -> String {
        format!("const {gating_result_name} = {gating_fn_name}();")
    }
}

impl fmt::Display for GatingOutput {
    /// Format the structural part of the gating output (not the function bodies).
    ///
    /// For `Ternary` mode, this produces the gating call expression pattern that
    /// the caller wraps around compiled/original function strings.
    ///
    /// For `Hoisted` mode, this produces the full structural skeleton:
    /// - The `const RESULT = gatingFn();` declaration
    /// - The dispatcher function
    ///
    /// The actual compiled and original function bodies must be inserted by the caller.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GatingOutput::Hoisted {
                original_name,
                optimized_name,
                unoptimized_name,
                gating_result_name,
                gating_fn_name,
                params,
            } => {
                // Emit the gating const declaration
                writeln!(
                    f,
                    "{}",
                    GatingOutput::format_gating_const(gating_result_name, gating_fn_name)
                )?;
                // Placeholder comment for where compiled + original functions go
                writeln!(f, "// [compiled function: {optimized_name}]")?;
                writeln!(f, "// [original function: {unoptimized_name}]")?;
                // Emit the dispatcher function
                write!(
                    f,
                    "{}",
                    GatingOutput::format_dispatcher(
                        original_name,
                        optimized_name,
                        unoptimized_name,
                        gating_result_name,
                        params,
                    )
                )
            }
            GatingOutput::Ternary { gating_fn_name, wrap } => match wrap {
                TernaryWrap::ConstDeclaration { name } => {
                    write!(
                        f,
                        "const {name} = {gating_fn_name}()\n  ? [compiled]\n  : [original];"
                    )
                }
                TernaryWrap::ExportDefaultThenConst { name } => {
                    writeln!(
                        f,
                        "const {name} = {gating_fn_name}()\n  ? [compiled]\n  : [original];"
                    )?;
                    write!(f, "export default {name};")
                }
                TernaryWrap::Inline => {
                    write!(f, "{gating_fn_name}()\n  ? [compiled]\n  : [original]")
                }
            },
        }
    }
}

/// Format a complete ternary gating expression with actual function code.
///
/// This is the main entry point for emitting non-hoisted gating output.
/// It takes the gating function name, compiled function code, and original
/// function code and produces the complete ternary expression.
///
/// # Arguments
///
/// * `gating_fn_name` - Local name of the imported gating function.
/// * `compiled_code` - The compiled function expression as a code string.
/// * `original_code` - The original function expression as a code string.
pub fn format_ternary_expression(
    gating_fn_name: &str,
    compiled_code: &str,
    original_code: &str,
) -> String {
    format!("{gating_fn_name}()\n  ? {compiled_code}\n  : {original_code}")
}

/// Format the complete hoisted gating output with actual function code.
///
/// Produces the full hoisted pattern:
/// ```js
/// const RESULT = gatingFn();
/// function Name_optimized(...) { /* compiled body */ }
/// function Name_unoptimized(...) { /* original body */ }
/// function Name(arg0, ...) {
///   if (RESULT) return Name_optimized(arg0, ...);
///   else return Name_unoptimized(arg0, ...);
/// }
/// ```
///
/// # Arguments
///
/// * `gating_fn_name` - Local name of the imported gating function.
/// * `gating_result_name` - Name for the const holding the gating call result.
/// * `original_name` - The original function name.
/// * `optimized_name` - Name for the compiled version.
/// * `unoptimized_name` - Name for the original version.
/// * `compiled_fn_code` - The full compiled function declaration as a code string.
/// * `original_fn_code` - The full original function declaration as a code string
///   (with its name already renamed to `unoptimized_name`).
/// * `params` - Parameter information for the dispatcher.
pub fn format_hoisted_gating(
    gating_fn_name: &str,
    gating_result_name: &str,
    original_name: &str,
    optimized_name: &str,
    unoptimized_name: &str,
    compiled_fn_code: &str,
    original_fn_code: &str,
    params: &[ParamInfo],
) -> String {
    let gating_const = GatingOutput::format_gating_const(gating_result_name, gating_fn_name);
    let dispatcher = GatingOutput::format_dispatcher(
        original_name,
        optimized_name,
        unoptimized_name,
        gating_result_name,
        params,
    );

    format!("{gating_const}\n{compiled_fn_code}\n{original_fn_code}\n{dispatcher}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_expression() {
        let result = format_ternary_expression(
            "isForgetEnabled_Fixtures",
            "function Foo() { /* compiled */ }",
            "function Foo() { /* original */ }",
        );
        assert!(result.contains("isForgetEnabled_Fixtures()"));
        assert!(result.contains("? function Foo() { /* compiled */ }"));
        assert!(result.contains(": function Foo() { /* original */ }"));
    }

    #[test]
    fn test_hoisted_gating() {
        let params = vec![ParamInfo { is_rest: false }];
        let result = format_hoisted_gating(
            "isForgetEnabled_Fixtures",
            "isForgetEnabled_Fixtures_result",
            "Foo",
            "Foo_optimized",
            "Foo_unoptimized",
            "function Foo_optimized(t0) { /* compiled */ }",
            "function Foo_unoptimized({ prop1 }) { /* original */ }",
            &params,
        );
        assert!(result.contains("const isForgetEnabled_Fixtures_result = isForgetEnabled_Fixtures();"));
        assert!(result.contains("function Foo_optimized(t0) { /* compiled */ }"));
        assert!(result.contains("function Foo_unoptimized({ prop1 }) { /* original */ }"));
        assert!(result.contains("function Foo(arg0)"));
        assert!(result.contains("if (isForgetEnabled_Fixtures_result) return Foo_optimized(arg0);"));
        assert!(result.contains("else return Foo_unoptimized(arg0);"));
    }

    #[test]
    fn test_hoisted_gating_with_rest_param() {
        let params = vec![ParamInfo { is_rest: false }, ParamInfo { is_rest: true }];
        let result = format_hoisted_gating(
            "gatingFn",
            "gatingFn_result",
            "Bar",
            "Bar_optimized",
            "Bar_unoptimized",
            "function Bar_optimized(a, ...b) {}",
            "function Bar_unoptimized(a, ...b) {}",
            &params,
        );
        assert!(result.contains("function Bar(arg0, ...arg1)"));
        assert!(result.contains("return Bar_optimized(arg0, ...arg1)"));
        assert!(result.contains("return Bar_unoptimized(arg0, ...arg1)"));
    }

    #[test]
    fn test_build_gating_output_non_hoisted_const_decl() {
        let gating = ExternalFunction {
            source: "ReactForgetFeatureFlag".to_string(),
            import_specifier_name: "isForgetEnabled_Fixtures".to_string(),
        };
        let mut ctx = ProgramContext::new();
        let output = build_gating_output(
            Some("Foo"),
            FunctionKind::FunctionDeclaration,
            ParentContext::Other,
            &gating,
            false,
            &[],
            &mut ctx,
        );
        match output {
            GatingOutput::Ternary { ref wrap, .. } => {
                assert!(matches!(wrap, TernaryWrap::ConstDeclaration { name } if name == "Foo"));
            }
            _ => panic!("Expected Ternary output"),
        }
    }

    #[test]
    fn test_build_gating_output_hoisted() {
        let gating = ExternalFunction {
            source: "ReactForgetFeatureFlag".to_string(),
            import_specifier_name: "isForgetEnabled_Fixtures".to_string(),
        };
        let mut ctx = ProgramContext::new();
        let params = vec![ParamInfo { is_rest: false }];
        let output = build_gating_output(
            Some("Foo"),
            FunctionKind::FunctionDeclaration,
            ParentContext::Other,
            &gating,
            true,
            &params,
            &mut ctx,
        );
        match output {
            GatingOutput::Hoisted {
                original_name,
                optimized_name,
                unoptimized_name,
                ..
            } => {
                assert_eq!(original_name, "Foo");
                assert_eq!(optimized_name, "Foo_optimized");
                assert_eq!(unoptimized_name, "Foo_unoptimized");
            }
            _ => panic!("Expected Hoisted output"),
        }
        // Verify the import was added
        assert!(ctx.imports.contains_key("ReactForgetFeatureFlag"));
    }

    #[test]
    fn test_build_gating_output_export_default() {
        let gating = ExternalFunction {
            source: "ReactForgetFeatureFlag".to_string(),
            import_specifier_name: "isForgetEnabled_Fixtures".to_string(),
        };
        let mut ctx = ProgramContext::new();
        let output = build_gating_output(
            Some("Foo"),
            FunctionKind::FunctionDeclaration,
            ParentContext::ExportDefault,
            &gating,
            false,
            &[],
            &mut ctx,
        );
        match output {
            GatingOutput::Ternary { ref wrap, .. } => {
                assert!(
                    matches!(wrap, TernaryWrap::ExportDefaultThenConst { name } if name == "Foo")
                );
            }
            _ => panic!("Expected Ternary output"),
        }
    }

    #[test]
    fn test_build_gating_output_arrow_inline() {
        let gating = ExternalFunction {
            source: "ReactForgetFeatureFlag".to_string(),
            import_specifier_name: "isForgetEnabled_Fixtures".to_string(),
        };
        let mut ctx = ProgramContext::new();
        let output = build_gating_output(
            None,
            FunctionKind::ArrowFunction,
            ParentContext::ExportDefault,
            &gating,
            false,
            &[],
            &mut ctx,
        );
        match output {
            GatingOutput::Ternary { ref wrap, .. } => {
                assert!(matches!(wrap, TernaryWrap::Inline));
            }
            _ => panic!("Expected Ternary output"),
        }
    }

    #[test]
    fn test_new_uid_avoids_collisions() {
        let mut ctx = ProgramContext::new();
        ctx.add_reference("foo");
        let uid = ctx.new_uid("foo");
        assert_eq!(uid, "foo_0");
        // Second call should get foo_1 since foo_0 is now taken
        let uid2 = ctx.new_uid("foo");
        assert_eq!(uid2, "foo_1");
    }

    #[test]
    fn test_add_import_specifier_dedup() {
        let mut ctx = ProgramContext::new();
        let ext_fn = ExternalFunction {
            source: "my-module".to_string(),
            import_specifier_name: "myFn".to_string(),
        };
        let name1 = ctx.add_import_specifier(&ext_fn);
        let name2 = ctx.add_import_specifier(&ext_fn);
        assert_eq!(name1, name2);
        // Should only have one import specifier
        assert_eq!(ctx.imports.get("my-module").map(|v| v.len()), Some(1));
    }
}
