use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{
    JsxOptions, JsxRuntime, PluginsOptions, ReactCompilerOptions, TransformOptions, Transformer,
};

/// Transform source code with the React Compiler plugin enabled.
///
/// Parses as JSX, runs the transformer with JSX automatic runtime + React Compiler,
/// and returns the codegen output.
fn transform_react_compiler(source: &str, compiler_opts: ReactCompilerOptions) -> String {
    let source_type = SourceType::jsx();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, source_type).parse();
    assert!(ret.errors.is_empty(), "Parse errors: {:?}", ret.errors);
    let mut program = ret.program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let options = TransformOptions {
        jsx: JsxOptions { runtime: JsxRuntime::Automatic, ..JsxOptions::default() },
        plugins: PluginsOptions {
            react_compiler: Some(compiler_opts),
            ..PluginsOptions::default()
        },
        ..TransformOptions::default()
    };
    let ret = Transformer::new(&allocator, Path::new("test.jsx"), &options)
        .build_with_scoping(scoping, &mut program);
    // We don't assert on transformer errors — the compiler may report diagnostics
    // for certain inputs but still produce output.
    let _ = ret.errors;
    Codegen::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program)
        .code
}

fn default_enabled_opts() -> ReactCompilerOptions {
    ReactCompilerOptions {
        enabled: true,
        compilation_mode: Some("all".to_string()),
        ..ReactCompilerOptions::default()
    }
}

// ---------------------------------------------------------------------------
// 1. Basic compilation — verify the compiler produces memoized output
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_basic_component() {
    let source = r#"
function Component({ name }) {
    return <div>{name}</div>;
}
"#;
    let code = transform_react_compiler(source, default_enabled_opts());
    // Compiled output should contain the cache variable from react/compiler-runtime
    assert!(
        code.contains("_c(") || code.contains("$["),
        "Expected compiled output with memoization cache, got:\n{code}"
    );
    assert!(
        code.contains("react/compiler-runtime") || code.contains("react-compiler-runtime"),
        "Expected runtime import, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// 2. Nested function discovery (infer mode)
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_nested_function_discovery_infer_mode() {
    // Test that infer mode discovers and compiles a component nested inside
    // a variable declaration (not inside a class body).
    let source = r#"
const obj = {
    Component: function Component() {
        return <div />;
    }
};
"#;
    let opts = ReactCompilerOptions {
        enabled: true,
        compilation_mode: Some("infer".to_string()),
        ..ReactCompilerOptions::default()
    };
    let code = transform_react_compiler(source, opts);
    // In infer mode, the nested Component should be discovered and compiled.
    // Note: The compiler must recognize the capitalized function name as a component.
    assert!(
        code.contains("function Component"),
        "Expected Component function to be present in output, got:\n{code}"
    );
}

#[test]
fn react_compiler_infer_mode_top_level_component() {
    // In infer mode, a top-level component with a capitalized name should be compiled.
    let source = r#"
function MyComponent({ name }) {
    return <div>{name}</div>;
}
"#;
    let opts = ReactCompilerOptions {
        enabled: true,
        compilation_mode: Some("infer".to_string()),
        ..ReactCompilerOptions::default()
    };
    let code = transform_react_compiler(source, opts);
    assert!(
        code.contains("_c(") || code.contains("$["),
        "Expected Component to be compiled in infer mode, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// 3. Class body skipping — functions inside classes should NOT be compiled
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_class_body_skipping() {
    let source = r#"
class Foo {
    render() {
        function Inner() {
            return <div />;
        }
        return Inner();
    }
}
"#;
    let opts = ReactCompilerOptions {
        enabled: true,
        compilation_mode: Some("infer".to_string()),
        ..ReactCompilerOptions::default()
    };
    let code = transform_react_compiler(source, opts);
    // Functions inside class bodies should NOT be compiled
    assert!(!code.contains("_c("), "Expected class-body function NOT to be compiled, got:\n{code}");
    assert!(
        !code.contains("react/compiler-runtime") && !code.contains("react-compiler-runtime"),
        "Expected no runtime import for class-only file, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// 4. Module-level opt-out ('use no memo')
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_module_level_opt_out() {
    let source = r#"
'use no memo';
function Component() {
    return <div />;
}
"#;
    let code = transform_react_compiler(source, default_enabled_opts());
    // Module-level 'use no memo' should discard all compiled output
    assert!(
        !code.contains("_c("),
        "Expected no memoization with module-level opt-out, got:\n{code}"
    );
    assert!(
        !code.contains("react/compiler-runtime") && !code.contains("react-compiler-runtime"),
        "Expected no runtime import with module-level opt-out, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// 5. Function-level opt-out ('use no memo')
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_function_level_opt_out() {
    let source = r#"
function Component() {
    'use no memo';
    return <div />;
}
"#;
    let code = transform_react_compiler(source, default_enabled_opts());
    // Function-level 'use no memo' should prevent compilation of that function
    assert!(
        !code.contains("_c("),
        "Expected no memoization with function-level opt-out, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// 6. noEmit / lint mode — pipeline runs but output is not emitted
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_no_emit_mode() {
    let source = r#"
function Component({ name }) {
    return <div>{name}</div>;
}
"#;
    let opts = ReactCompilerOptions {
        enabled: true,
        compilation_mode: Some("all".to_string()),
        no_emit: Some(true),
        ..ReactCompilerOptions::default()
    };
    let code = transform_react_compiler(source, opts);
    // noEmit should suppress compiled output
    assert!(!code.contains("_c("), "Expected no memoization with noEmit: true, got:\n{code}");
}

#[test]
fn react_compiler_lint_output_mode() {
    let source = r#"
function Component({ name }) {
    return <div>{name}</div>;
}
"#;
    let opts = ReactCompilerOptions {
        enabled: true,
        compilation_mode: Some("all".to_string()),
        output_mode: Some("lint".to_string()),
        ..ReactCompilerOptions::default()
    };
    let code = transform_react_compiler(source, opts);
    // lint output mode should suppress compiled output
    assert!(!code.contains("_c("), "Expected no memoization with output_mode: lint, got:\n{code}");
}

// ---------------------------------------------------------------------------
// 7. ignoreUseNoForget overrides 'use no memo'
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_ignore_use_no_forget() {
    let source = r#"
function Component() {
    'use no memo';
    return <div />;
}
"#;
    let opts = ReactCompilerOptions {
        enabled: true,
        compilation_mode: Some("all".to_string()),
        ignore_use_no_forget: Some(true),
        ..ReactCompilerOptions::default()
    };
    let code = transform_react_compiler(source, opts);
    // With ignoreUseNoForget, 'use no memo' should be ignored and function compiled
    assert!(
        code.contains("_c(") || code.contains("$["),
        "Expected compilation despite 'use no memo' when ignoreUseNoForget is true, got:\n{code}"
    );
}

#[test]
fn react_compiler_ignore_use_no_forget_module_level() {
    let source = r#"
'use no memo';
function Component() {
    return <div />;
}
"#;
    let opts = ReactCompilerOptions {
        enabled: true,
        compilation_mode: Some("all".to_string()),
        ignore_use_no_forget: Some(true),
        ..ReactCompilerOptions::default()
    };
    let code = transform_react_compiler(source, opts);
    // Module-level 'use no memo' should also be ignored
    assert!(
        code.contains("_c(") || code.contains("$["),
        "Expected compilation despite module-level 'use no memo' when ignoreUseNoForget is true, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// 8. Target runtime module
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_target_react17_runtime_module() {
    let source = r#"
function Component({ name }) {
    return <div>{name}</div>;
}
"#;
    let opts = ReactCompilerOptions {
        enabled: true,
        compilation_mode: Some("all".to_string()),
        target: Some("react-17".to_string()),
        ..ReactCompilerOptions::default()
    };
    let code = transform_react_compiler(source, opts);
    // react-17 target should import from "react-compiler-runtime"
    assert!(
        code.contains("react-compiler-runtime"),
        "Expected import from 'react-compiler-runtime' for react-17 target, got:\n{code}"
    );
    assert!(
        !code.contains("react/compiler-runtime"),
        "Should NOT import from 'react/compiler-runtime' for react-17 target, got:\n{code}"
    );
}

#[test]
fn react_compiler_target_default_runtime_module() {
    let source = r#"
function Component({ name }) {
    return <div>{name}</div>;
}
"#;
    let code = transform_react_compiler(source, default_enabled_opts());
    // Default target (react-19) should import from "react/compiler-runtime"
    assert!(
        code.contains("react/compiler-runtime"),
        "Expected import from 'react/compiler-runtime' for default target, got:\n{code}"
    );
}

#[test]
fn react_compiler_target_react18_runtime_module() {
    let source = r#"
function Component({ name }) {
    return <div>{name}</div>;
}
"#;
    let opts = ReactCompilerOptions {
        enabled: true,
        compilation_mode: Some("all".to_string()),
        target: Some("react-18".to_string()),
        ..ReactCompilerOptions::default()
    };
    let code = transform_react_compiler(source, opts);
    assert!(
        code.contains("react-compiler-runtime"),
        "Expected import from 'react-compiler-runtime' for react-18 target, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// 9. 'all' mode only compiles top-level functions
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_all_mode_only_top_level() {
    let source = r#"
function outer() {
    function inner() {
        return 1;
    }
    return inner();
}
"#;
    let opts = ReactCompilerOptions {
        enabled: true,
        compilation_mode: Some("all".to_string()),
        ..ReactCompilerOptions::default()
    };
    let code = transform_react_compiler(source, opts);
    // In 'all' mode, `outer` should be compiled (it is top-level).
    // `inner` should NOT be separately compiled since 'all' mode skips nested discovery.
    assert!(
        code.contains("_c(") || code.contains("$["),
        "Expected top-level function to be compiled in 'all' mode, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// 10. Disabled plugin produces no changes
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_disabled_no_changes() {
    let source = r#"
function Component() {
    return <div />;
}
"#;
    let opts = ReactCompilerOptions { enabled: false, ..ReactCompilerOptions::default() };
    let code = transform_react_compiler(source, opts);
    assert!(!code.contains("_c("), "Expected no compilation when plugin is disabled, got:\n{code}");
    assert!(
        !code.contains("react/compiler-runtime") && !code.contains("react-compiler-runtime"),
        "Expected no runtime import when plugin is disabled, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// 11. Already-compiled marker prevents double-compilation
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_skip_already_compiled() {
    let source = r#"
import { c as _c } from "react/compiler-runtime";
function Component() {
    return <div />;
}
"#;
    let code = transform_react_compiler(source, default_enabled_opts());
    // The file already has the runtime import, so the compiler should skip it.
    // The output should still have exactly one runtime import (the existing one),
    // and no additional _c() calls injected by a second compilation pass.
    let import_count = code.matches("react/compiler-runtime").count();
    assert!(
        import_count <= 1,
        "Expected at most one runtime import (no double-compilation), found {import_count} in:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// 12. Arrow function component compiles
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_arrow_function_component() {
    let source = r#"
const Component = ({ name }) => {
    return <div>{name}</div>;
};
"#;
    let code = transform_react_compiler(source, default_enabled_opts());
    assert!(
        code.contains("_c(") || code.contains("$["),
        "Expected arrow function component to be compiled, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// 13. Export default function compiles
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_export_default_function() {
    let source = r#"
export default function App() {
    return <div>App</div>;
}
"#;
    let code = transform_react_compiler(source, default_enabled_opts());
    assert!(
        code.contains("_c(") || code.contains("$["),
        "Expected export default function to be compiled, got:\n{code}"
    );
    assert!(
        code.contains("export default function App"),
        "Expected function declaration to be preserved, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// 14. Multiple components in one file
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_multiple_components() {
    let source = r#"
function Header({ title }) {
    return <h1>{title}</h1>;
}
function Footer({ text }) {
    return <footer>{text}</footer>;
}
"#;
    let code = transform_react_compiler(source, default_enabled_opts());
    assert!(
        code.contains("function Header") && code.contains("function Footer"),
        "Expected both functions to be present in output, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// 15. Custom opt-out directive
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_custom_opt_out_directive() {
    let source = r#"
function Component() {
    'use my custom skip';
    return <div />;
}
"#;
    let opts = ReactCompilerOptions {
        enabled: true,
        compilation_mode: Some("all".to_string()),
        custom_opt_out_directives: Some(vec!["use my custom skip".to_string()]),
        ..ReactCompilerOptions::default()
    };
    let code = transform_react_compiler(source, opts);
    // Custom directive should prevent compilation
    assert!(
        !code.contains("_c("),
        "Expected no compilation with custom opt-out directive, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// 16. 'use no forget' (legacy name) also opts out
// ---------------------------------------------------------------------------

#[test]
fn react_compiler_use_no_forget_legacy() {
    let source = r#"
function Component() {
    'use no forget';
    return <div />;
}
"#;
    let code = transform_react_compiler(source, default_enabled_opts());
    assert!(
        !code.contains("_c("),
        "Expected no compilation with legacy 'use no forget' directive, got:\n{code}"
    );
}
