use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_import_node_test(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not import from `node:test`")
        .with_help("Import from `vitest` instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoImportNodeTest;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns when `node:test` is imported (usually accidentally).
    /// With `--fix`, it will replace the import with `vitest`.
    ///
    /// ### Why is this bad?
    ///
    /// Using `node:test` instead of `vitest` can lead to inconsistent test results
    /// and missing features. `vitest` should be used for all testing to ensure
    /// compatibility and access to its full functionality.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import { test } from 'node:test'
    /// import { expect } from 'vitest'
    ///
    /// test('foo', () => {
    ///   expect(1).toBe(1)
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import { test, expect } from 'vitest'
    ///
    /// test('foo', () => {
    ///   expect(1).toBe(1)
    /// })
    /// ```
    NoImportNodeTest,
    vitest,
    style,
    fix
);

impl Rule for NoImportNodeTest {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();

        if let Some(node_test_module) = module_record.requested_modules.get("node:test") {
            if let Some(requested_module) = node_test_module.first() {
                ctx.diagnostic_with_fix(no_import_node_test(requested_module.span), |fixer| {
                    fixer.replace(requested_module.span, "\"vitest\"")
                });
            }
        }
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

    Tester::new(NoImportNodeTest::NAME, NoImportNodeTest::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
