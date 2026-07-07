//! Regression: in lint mode with `skip_lint_codegen = false` (the `reportAllBailouts`
//! path) codegen runs, and `enable_memoization()` is true for lint mode — so a
//! memoizing component reaches the `const $ = _c(N)` emit. The memo cache import
//! name must therefore be reserved for lint mode too, not just client mode, or
//! codegen panics on the unset name.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_react_compiler::{CompilationMode, PluginOptions, lint};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

/// Reads props and returns JSX — memoizes (caches the element) without bailing, so
/// codegen reaches the memo-cache declaration.
const MEMOIZING: &str = r"
function Component(props) {
  return <div>{props.a}{props.b}</div>;
}
";

#[test]
fn lint_mode_running_codegen_memoizes_without_panicking() {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, MEMOIZING, SourceType::tsx().with_module(true)).parse();
    assert!(parsed.diagnostics.is_empty(), "unexpected parse errors");
    let program = parsed.program;
    let semantic = SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;
    let options = PluginOptions {
        compilation_mode: CompilationMode::All,
        // `reportAllBailouts` path: codegen runs in lint mode and memoizes.
        skip_lint_codegen: false,
        ..Default::default()
    };
    // Must not panic: the memo cache name is reserved for every non-SSR mode.
    let _ = lint(&program, &semantic, &allocator, options);
}
