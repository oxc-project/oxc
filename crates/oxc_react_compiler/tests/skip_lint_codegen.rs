//! `PluginOptions::skip_lint_codegen` trades codegen-stage diagnostics for speed:
//! it skips the AST rebuild in lint mode, which also drops the bailout/invariant
//! diagnostics codegen would emit. Callers reporting bailouts must leave it off.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_react_compiler::{CompilationMode, PluginOptions, lint};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

/// A component whose `for..in` reassigns the loop variable into a context variable —
/// a shape codegen can't emit, so codegen records a `Todo: Support non-trivial
/// for..in inits` bailout. The bailout comes only from codegen.
const FOR_IN_BAILOUT: &str = r"
function Component(props) {
  const data = useHook();
  const items = [];
  for (let key in props.data) {
    key = key ?? null;
    items.push(<div key={key} onClick={() => data.set(key)}>{key}</div>);
  }
  return <div>{items}</div>;
}
";

fn lint_diagnostics(source: &str, skip_lint_codegen: bool) -> Vec<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::tsx().with_module(true)).parse();
    assert!(parsed.diagnostics.is_empty(), "unexpected parse errors");
    let program = parsed.program;
    let semantic = SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;
    let options = PluginOptions {
        compilation_mode: CompilationMode::All,
        skip_lint_codegen,
        ..Default::default()
    };
    lint(&program, &semantic, &allocator, options)
        .diagnostics
        .as_slice()
        .iter()
        .map(|d| format!("{d:?}"))
        .collect()
}

#[test]
fn skip_lint_codegen_drops_codegen_bailouts_but_running_codegen_keeps_them() {
    // Skipped (the `reportAllBailouts: false` default): the codegen bailout is gone.
    let skipped = lint_diagnostics(FOR_IN_BAILOUT, true);
    assert!(
        skipped.iter().all(|d| !d.contains("for..in")),
        "skip_lint_codegen should drop the codegen for..in bailout, got: {skipped:?}"
    );

    // Codegen runs (the `reportAllBailouts: true` path): the bailout is reported.
    let ran = lint_diagnostics(FOR_IN_BAILOUT, false);
    assert!(
        ran.iter().any(|d| d.contains("for..in")),
        "running codegen should keep the for..in bailout, got: {ran:?}"
    );
}
