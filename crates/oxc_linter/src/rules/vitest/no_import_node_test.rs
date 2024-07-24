use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_import_node_test(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("disallow importing `node:test`")
        .with_help("Import from `vitest` instead of `node:test`")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoImportNodeTest;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns when `node:test` is imported (usually accidentally). With `--fix`, it will replace the import with `vitest`.
    ///
    /// ### Examples
    ///
    /// ```javascript
    /// // invalid
    /// import { test } from 'node:test'
    /// import { expect } from 'vitest'
    ///
    /// test('foo', () => {
    ///   expect(1).toBe(1)
    /// })
    /// ```
    ///
    /// ```javascript
    /// // valid
    /// import { test, expect } from 'vitest'
    ///
    /// test('foo', () => {
    ///   expect(1).toBe(1)
    /// })
    /// ```
    NoImportNodeTest,
    style,
);

impl Rule for NoImportNodeTest {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ImportDeclaration(import_decl) = node.kind() {
            if import_decl.source.value.eq("node:test") {
                let span = import_decl.source.span;

                ctx.diagnostic_with_fix(no_import_node_test(span), |fixer| {
                    fixer.replace(span, "\"vitest\"")
                });
            }
        };
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![(r#"import { test } from "vitest""#, None)];

    let fail = vec![
        (r#"import { test } from "node:test""#, None),
        ("import * as foo from 'node:test'", None),
    ];

    let fix = vec![
        (r#"import { test } from "node:test""#, r#"import { test } from "vitest""#, None),
        (r#"import * as foo from "node:test""#, r#"import * as foo from "vitest""#, None),
    ];

    Tester::new(NoImportNodeTest::NAME, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
