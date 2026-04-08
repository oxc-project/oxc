use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

#[allow(dead_code)]
fn no_unresolved_imports_diagnostic(span: Span, source: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unable to resolve import `{source}`."))
        .with_help("Ensure the module exists, the path is correct, and the package is installed.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnresolvedImports;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects imports that cannot be resolved.
    ///
    /// ### Why is this bad?
    ///
    /// Importing a module that doesn't exist or can't be resolved will cause
    /// a runtime error. This rule catches these issues at lint time.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import foo from './nonexistent';
    /// import bar from 'not-installed-package';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import foo from './existing-module';
    /// import bar from 'installed-package';
    /// ```
    NoUnresolvedImports,
    import,
    nursery, // Complex rule that needs module resolution
    pending
);

impl Rule for NoUnresolvedImports {
    fn run<'a>(&self, _node: &AstNode<'a>, _ctx: &LintContext<'a>) {
        // TODO: This rule requires module resolution capabilities.
        // The full implementation would:
        // 1. Get the import source path
        // 2. Resolve it relative to the current file
        // 3. Check if the resolved path exists
        // 4. For node_modules, check if the package is installed
        //
        // This is a placeholder registration. Module resolution is complex
        // and depends on tsconfig paths, package.json exports, etc.
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["import { foo } from './utils';", "const x = 1;"];

    let fail = vec![];

    Tester::new(NoUnresolvedImports::NAME, NoUnresolvedImports::PLUGIN, pass, fail)
        .test_and_snapshot();
}
