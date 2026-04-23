use itertools::Itertools;
use oxc_ast::{
    AstKind,
    ast::{BindingPattern, Expression, ImportOrExportKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use rustc_hash::FxHashSet;

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    module_record::ImportImportName,
    rule::Rule,
};

fn prefer_importing_vitest_globals_diagnostic(
    spans: &[Span],
    globals_founds: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use vitest global functions")
        .with_help(format!(
            "Import global functions `{globals_founds}` from `vitest` package instead of using globals."
        ))
        .with_labels(spans.iter().map(|span| span.label("Add this global vitest import")))
}

// Vitest provides a `jest` global for migration compatibility

/// Vitest globals that should be imported explicitly.
const VITEST_GLOBALS: [CompactStr; 17] = [
    CompactStr::new_const("afterAll"),
    CompactStr::new_const("afterEach"),
    CompactStr::new_const("beforeAll"),
    CompactStr::new_const("beforeEach"),
    CompactStr::new_const("bench"),
    CompactStr::new_const("describe"),
    CompactStr::new_const("expect"),
    CompactStr::new_const("expectTypeOf"),
    CompactStr::new_const("fdescribe"),
    CompactStr::new_const("fit"),
    CompactStr::new_const("it"),
    CompactStr::new_const("pending"),
    CompactStr::new_const("test"),
    CompactStr::new_const("vi"),
    CompactStr::new_const("xdescribe"),
    CompactStr::new_const("xit"),
    CompactStr::new_const("xtest"),
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
    fix,
    version = "1.59.0",
);

impl Rule for PreferImportingVitestGlobals {
    fn run_once(&self, ctx: &LintContext) {
        let scoping = ctx.scoping();
        let root_scope = scoping.root_scope_id();

        let mut missing_globals: FxHashSet<&str> = FxHashSet::default();
        let mut globals_spans: Vec<Span> = vec![];

        for (name, references_ids) in scoping.root_unresolved_references() {
            if !VITEST_GLOBALS.contains(&CompactStr::new(name.as_str())) {
                continue;
            }

            missing_globals.insert(name.as_str());

            for &reference_id in references_ids {
                let reference = scoping.get_reference(reference_id);
                let AstKind::CallExpression(call_expr) =
                    ctx.nodes().parent_node(reference.node_id()).kind()
                else {
                    continue;
                };

                globals_spans.push(call_expr.callee.span());
            }
        }

        for name in VITEST_GLOBALS {
            let Some(symbol_id) = scoping.get_binding(root_scope, name.as_str().into()) else {
                continue;
            };

            if !scoping.symbol_flags(symbol_id).is_import() {
                continue;
            }

            let AstKind::ImportDeclaration(import_decl) =
                ctx.nodes().parent_kind(scoping.symbol_declaration(symbol_id))
            else {
                continue;
            };

            if import_decl.source.value == "vitest"
                && import_decl.import_kind == ImportOrExportKind::Value
            {
                continue;
            }

            for reference in scoping.get_resolved_references(symbol_id) {
                let AstKind::CallExpression(call_expr) =
                    ctx.nodes().parent_node(reference.node_id()).kind()
                else {
                    continue;
                };

                if let Expression::Identifier(identifier) = &call_expr.callee
                    && identifier.name.as_str() == name.as_str()
                {
                    globals_spans.push(call_expr.span);
                    missing_globals.insert(identifier.name.as_str());
                }
            }
        }

        if missing_globals.is_empty() {
            return;
        }

        let globals_imports = missing_globals.iter().join(", ");

        ctx.diagnostic_with_fix(
            prefer_importing_vitest_globals_diagnostic(globals_spans.as_ref(), &globals_imports),
            |fixer| Self::build_fix(ctx, &globals_imports, fixer),
        );
    }
}

impl PreferImportingVitestGlobals {
    fn build_fix<'a>(
        ctx: &LintContext<'a>,
        globals_imports: &str,
        fixer: RuleFixer<'_, 'a>,
    ) -> RuleFix {
        let module_record = ctx.module_record();

        let vitest_esm_import = module_record
            .import_entries
            .iter()
            .find(|e| e.module_request.name() == "vitest" && !e.is_type);

        // 1. Existing `import { ... } from 'vitest'` — append to named specifiers.
        if let Some(entry) = vitest_esm_import
            && vitest_esm_import
                .is_some_and(|import| matches!(import.import_name, ImportImportName::Name(_)))
        {
            let source = ctx.source_range(entry.statement_span);
            if let Some(close_brace_pos) = source.rfind('}') {
                let before_brace = &source[..close_brace_pos];
                let trimmed = before_brace.trim_end();
                let needs_comma = !trimmed.ends_with(',');

                #[expect(clippy::cast_possible_truncation)]
                let replace_start = entry.statement_span.start + trimmed.len() as u32;
                #[expect(clippy::cast_possible_truncation)]
                let replace_end = entry.statement_span.start + close_brace_pos as u32;

                let text = if needs_comma {
                    format!(", {globals_imports} }}")
                } else {
                    format!(" {globals_imports} }}")
                };

                return fixer.replace(Span::new(replace_start, replace_end + 1), text);
            }
        }

        // 2. Existing `import defaultName from 'vitest'` — convert to
        //    `import defaultName, { ... } from 'vitest'`.
        if let Some(entry) = vitest_esm_import
            && vitest_esm_import.is_some_and(|import| import.import_name.is_default())
        {
            return fixer.replace(
                Span::new(entry.local_name.span.end, entry.local_name.span.end),
                format!(", {{ {globals_imports} }}"),
            );
        }

        // 3. Existing `const { ... } = require('vitest')` — append to destructuring.
        if let Some(fix) = Self::try_fix_cjs_require(ctx, globals_imports, &fixer) {
            return fix;
        }

        // 4. Fallback: add a new `import { ... } from 'vitest'` at the top.
        fixer.insert_text_before_range(
            Span::empty(0),
            format!("import {{ {globals_imports} }} from 'vitest';\n"),
        )
    }

    /// Try to append missing names to an existing `const { ... } = require('vitest')`.
    fn try_fix_cjs_require<'a>(
        ctx: &LintContext<'a>,
        globals_imports: &str,
        fixer: &RuleFixer<'_, 'a>,
    ) -> Option<RuleFix> {
        let require_refs = ctx.scoping().root_unresolved_references().get("require")?;

        for &ref_id in require_refs {
            let reference = ctx.scoping().get_reference(ref_id);
            let parent = ctx.nodes().parent_node(reference.node_id());
            let AstKind::CallExpression(call) = parent.kind() else { continue };

            let is_vitest_require = call.arguments.len() == 1
                && call.arguments.first().is_some_and(|arg| {
                    arg.as_expression().is_some_and(|expr| {
                        matches!(expr, Expression::StringLiteral(lit) if lit.value == "vitest")
                    })
                });

            if !is_vitest_require {
                continue;
            }

            let Some(AstKind::VariableDeclarator(vitest_require_declarator)) = ctx
                .nodes()
                .ancestors(parent.id())
                .find(|ancestor| matches!(ancestor.kind(), AstKind::VariableDeclarator(_)))
                .map(AstNode::kind)
            else {
                return None;
            };

            let BindingPattern::ObjectPattern(pattern) = &vitest_require_declarator.id else {
                continue;
            };

            let properties_joined = pattern
                .properties
                .iter()
                .map(|property| ctx.source_range(property.span))
                .join(", ");

            let rest_variable =
                pattern.rest.as_ref().map_or("", |rest| ctx.source_range(rest.span));

            let comma_before_globals = if properties_joined.is_empty() { "" } else { ", " };
            let comma_after_globals = if rest_variable.is_empty() { "" } else { ", " };

            let text = format!(
                "{{ {properties_joined}{comma_before_globals}{globals_imports}{comma_after_globals}{rest_variable} }}"
            );

            return Some(fixer.replace(vitest_require_declarator.id.span(), text));
        }

        None
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "vitest.describe('suite', () => {});",
        "import { describe } from 'vitest'; describe('suite', () => {});",
        "import { describe, it } from 'vitest'; describe('suite', () => {});",
        "import { describe, desccribe } from 'vitest'; describe('suite', () => {});",
        "const { describe } = require('vitest'); describe('suite', () => {});",
        "const { describe, it } = require('vitest'); describe('suite', () => {});",
        "const { describe, desccribe } = require('vitest'); describe('suite', () => {});",
        "import { describe, expect, it } from 'vitest'; describe('suite', () => { it('test', () => { let test = 5; expect(test).toBe(5); }); });",
        "import { describe, expect, it } from 'vitest'; describe('suite', () => { it('test', () => { const test = () => true; expect(test()).toBe(true); }); });",
        "import { describe, expect, it } from 'vitest'; describe('suite', () => { it('test', () => { function fn(test) { return test; } expect(fn(5)).toBe(5); }); });",
    ];

    let fail = vec![
        "describe('suite', () => {});",
        "import { it } from 'vitest';
            describe('suite', () => {});",
        "import { describe } from 'jest';
            describe('suite', () => {});",
        "import vitest from 'vitest';
            describe('suite', () => {});",
        "import * as abc from 'vitest';
            describe('suite', () => {});",
        r#"import { "default" as vitest } from 'vitest'; describe('suite', () => {});"#,
        "const x = require('something', 'else'); describe('suite', () => {});",
        "const x = require('jest'); describe('suite', () => {});",
        "const vitest = require('vitest'); describe('suite', () => {});",
        "const { ...rest } = require('vitest'); describe('suite', () => {});",
        r#"const { "default": vitest } = require('vitest'); describe('suite', () => {});"#,
        "const { it } = require('vitest');
            describe('suite', () => {});",
    ];

    let fix = vec![
        (
            "describe('suite', () => {});",
            "import { describe } from 'vitest';\ndescribe('suite', () => {});",
        ),
        (
            "import { it } from 'vitest';
            describe('suite', () => {});",
            "import { it, describe } from 'vitest';
            describe('suite', () => {});",
        ),
        (
            "import { describe } from 'jest';
            describe('suite', () => {});",
            "import { describe } from 'vitest';\nimport { describe } from 'jest';
            describe('suite', () => {});",
        ),
        (
            "import vitest from 'vitest';
            describe('suite', () => {});",
            "import vitest, { describe } from 'vitest';
            describe('suite', () => {});",
        ),
        (
            "import * as abc from 'vitest';
            describe('suite', () => {});",
            "import { describe } from 'vitest';\nimport * as abc from 'vitest';
            describe('suite', () => {});",
        ),
        (
            r#"import { "default" as vitest } from 'vitest'; describe('suite', () => {});"#,
            r#"import { "default" as vitest, describe } from 'vitest'; describe('suite', () => {});"#,
        ),
        (
            "const x = require('something', 'else'); describe('suite', () => {});",
            "import { describe } from 'vitest';\nconst x = require('something', 'else'); describe('suite', () => {});",
        ),
        (
            "const x = require('jest'); describe('suite', () => {});",
            "import { describe } from 'vitest';\nconst x = require('jest'); describe('suite', () => {});",
        ),
        (
            "const vitest = require('vitest'); describe('suite', () => {});",
            "import { describe } from 'vitest';\nconst vitest = require('vitest'); describe('suite', () => {});",
        ),
        // Oxc emit an error if ...rest it's declared as first value instead of second.
        (
            "const { ...rest } = require('vitest');
            describe('suite', () => {});",
            "const { describe, ...rest } = require('vitest');
            describe('suite', () => {});",
        ),
        (
            r#"const { "default": vitest } = require('vitest'); describe('suite', () => {});"#,
            r#"const { "default": vitest, describe } = require('vitest'); describe('suite', () => {});"#,
        ),
        (
            "const { it } = require('vitest');
            describe('suite', () => {});",
            "const { it, describe } = require('vitest');
            describe('suite', () => {});",
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
