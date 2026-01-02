use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, module_record::ImportImportName, rule::Rule};

fn prefer_importing_vitest_globals_diagnostic(span: Span, global: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer importing vitest global `{global}`"))
        .with_help(format!("Import `{global}` from 'vitest' explicitly."))
        .with_label(span)
}

/// Vitest globals that should be imported explicitly.
const VITEST_GLOBALS: phf::Set<&'static str> = phf::phf_set![
    "afterAll",
    "afterEach",
    "beforeAll",
    "beforeEach",
    "bench",
    "describe",
    "expect",
    "expectTypeOf",
    "fdescribe",
    "fit",
    "it",
    // Vitest provides a `jest` global for migration compatibility
    "jest",
    "pending",
    "test",
    "vi",
    "xdescribe",
    "xit",
    "xtest",
];

#[derive(Debug, Default, Clone)]
pub struct PreferImportingVitestGlobals;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces explicit imports from 'vitest' instead of using vitest globals.
    ///
    /// ### Why is this bad?
    ///
    /// Using vitest globals without importing them relies on implicit global configuration
    /// (`globals: true` in vitest config). Explicit imports make dependencies clear,
    /// improve IDE support, and ensure compatibility across different setups.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// describe('suite', () => {
    ///   it('test', () => {
    ///     expect(true).toBe(true)
    ///   })
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import { describe, it, expect } from 'vitest'
    ///
    /// describe('suite', () => {
    ///   it('test', () => {
    ///     expect(true).toBe(true)
    ///   })
    /// })
    /// ```
    PreferImportingVitestGlobals,
    vitest,
    style,
    fix
);

impl Rule for PreferImportingVitestGlobals {
    fn run(&self, node: &AstNode, ctx: &LintContext) {
        let AstKind::CallExpression(call) = node.kind() else { return };
        let Expression::Identifier(ident) = &call.callee else { return };

        if !VITEST_GLOBALS.contains(ident.name.as_str()) {
            return;
        }

        let reference = ctx.scoping().get_reference(ident.reference_id());
        if reference.symbol_id().is_some() {
            return;
        }

        ctx.diagnostic_with_fix(
            prefer_importing_vitest_globals_diagnostic(ident.span, ident.name.as_str()),
            |fixer| {
                let module_record = ctx.module_record();

                let vitest_import = module_record.import_entries.iter().find(|e| {
                    e.module_request.name() == "vitest"
                        && !e.is_type
                        && matches!(e.import_name, ImportImportName::Name(_))
                });

                if let Some(entry) = vitest_import {
                    let source = ctx.source_range(entry.statement_span);
                    if let Some(close_brace_pos) = source.rfind('}') {
                        let before_brace = &source[..close_brace_pos];
                        let trimmed = before_brace.trim_end();
                        let needs_comma = !trimmed.ends_with(',');

                        #[expect(clippy::cast_possible_truncation)]
                        let replace_start = entry.statement_span.start + trimmed.len() as u32;
                        #[expect(clippy::cast_possible_truncation)]
                        let replace_end = entry.statement_span.start + close_brace_pos as u32;

                        let new_import = ident.name.as_str();
                        let text = if needs_comma {
                            format!(", {new_import} }}")
                        } else {
                            format!(" {new_import} }}")
                        };

                        return fixer.replace(Span::new(replace_start, replace_end + 1), text);
                    }
                }

                let is_cjs = module_record.import_entries.is_empty();

                if is_cjs {
                    fixer.insert_text_before_range(
                        Span::empty(0),
                        format!("const {{ {} }} = require('vitest');\n", ident.name),
                    )
                } else {
                    fixer.insert_text_before_range(
                        Span::empty(0),
                        format!("import {{ {} }} from 'vitest';\n", ident.name),
                    )
                }
            },
        );
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "vitest.describe('suite', () => {});",
        "import { describe } from 'vitest';
describe('suite', () => {});",
        "import { describe, it } from 'vitest';
describe('suite', () => {});",
        "import { it as i, describe as d } from 'vitest';
d('suite', () => { i('test', () => {}); });",
        "const { describe } = require('vitest');
describe('suite', () => {});",
        "const { describe, it } = require('vitest');
describe('suite', () => {});",
        "const { describe: d, it: i } = require('vitest');
d('suite', () => { i('test', () => {}); });",
        "import { describe, expect, it } from 'vitest';
describe('suite', () => {
    it('test', () => {
        let test = 5;
        expect(test).toBe(5);
    });
});",
        "import { describe, expect, it } from 'vitest';
describe('suite', () => {
    it('test', () => {
        const test = () => true;
        expect(test()).toBe(true);
    });
});",
        "import { describe, expect, it } from 'vitest';
describe('suite', () => {
    it('test', () => {
        function fn(test) { return test; }
        expect(fn(5)).toBe(5);
    });
});",
    ];

    let fail = vec![
        "describe('suite', () => {});",
        "import { it } from 'vitest';
describe('suite', () => {});",
        "import vitest from 'vitest';
describe('suite', () => {});",
        "import * as abc from 'vitest';
describe('suite', () => {});",
        "import type { describe } from 'vitest';
describe('suite', () => {});",
        "const vitest = require('vitest');
describe('suite', () => {});",
        "const { it } = require('vitest');
describe('suite', () => {});",
    ];

    let fix = vec![
        (
            "describe('suite', () => {});",
            "const { describe } = require('vitest');\ndescribe('suite', () => {});",
            None,
        ),
        (
            "import { it } from 'vitest';
describe('suite', () => {});",
            "import { it, describe } from 'vitest';
describe('suite', () => {});",
            None,
        ),
        (
            "import vitest from 'vitest';
describe('suite', () => {});",
            "import { describe } from 'vitest';
import vitest from 'vitest';
describe('suite', () => {});",
            None,
        ),
        (
            "import * as abc from 'vitest';
describe('suite', () => {});",
            "import { describe } from 'vitest';
import * as abc from 'vitest';
describe('suite', () => {});",
            None,
        ),
        (
            "import type { describe } from 'vitest';
describe('suite', () => {});",
            "import { describe } from 'vitest';
import type { describe } from 'vitest';
describe('suite', () => {});",
            None,
        ),
        (
            "const vitest = require('vitest');
describe('suite', () => {});",
            "const { describe } = require('vitest');
const vitest = require('vitest');
describe('suite', () => {});",
            None,
        ),
        (
            "const { it } = require('vitest');
describe('suite', () => {});",
            "const { describe } = require('vitest');
const { it } = require('vitest');
describe('suite', () => {});",
            None,
        ),
    ];

    Tester::new(
        PreferImportingVitestGlobals::NAME,
        PreferImportingVitestGlobals::PLUGIN,
        pass,
        fail,
    )
    .with_vitest_plugin(true)
    .expect_fix(fix)
    .test_and_snapshot();
}
