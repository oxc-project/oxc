use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

#[allow(dead_code)]
fn no_private_imports_diagnostic(span: Span, source: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Import from `{source}` accesses a private module path."))
        .with_help("Only import from paths that are explicitly exported in the package's `exports` field in package.json.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoPrivateImports;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows importing from private module paths that are not explicitly
    /// exposed in a package's `exports` field.
    ///
    /// ### Why is this bad?
    ///
    /// Packages that define an `exports` field in their `package.json` specify
    /// which paths are public API. Importing from unlisted paths bypasses
    /// this contract and may break when the package is updated.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // If 'lib' has exports: { ".": "./dist/index.js" }
    /// import { internal } from 'lib/dist/internal';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import { exported } from 'lib';
    /// ```
    NoPrivateImports,
    import,
    nursery, // Complex rule that needs package.json exports parsing
    pending
);

impl Rule for NoPrivateImports {
    fn run<'a>(&self, _node: &AstNode<'a>, _ctx: &LintContext<'a>) {
        // TODO: This rule requires:
        // 1. Detecting deep imports into node_modules packages
        // 2. Reading the target package's package.json
        // 3. Checking the `exports` field to see if the import path is allowed
        //
        // This is a placeholder registration for the rule structure.
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["import { foo } from 'lib';", "import { bar } from './local';"];

    let fail = vec![];

    Tester::new(NoPrivateImports::NAME, NoPrivateImports::PLUGIN, pass, fail).test_and_snapshot();
}
