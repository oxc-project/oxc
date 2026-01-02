use oxc_ast::{
    AstKind,
    ast::{Argument, BindingPattern, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{context::LintContext, module_record::ImportImportName, rule::Rule};

fn prefer_importing_vitest_globals_diagnostic(span: Span, globals: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer importing vitest globals: {globals}"))
        .with_help(
            "Import these globals from 'vitest' explicitly instead of using them as globals.",
        )
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
    fn run_once(&self, ctx: &LintContext<'_>) {
        let import_info = collect_vitest_import_info(ctx);
        let mut unimported_globals: Vec<(&str, Span)> = Vec::new();

        for (name, reference_ids) in ctx.scoping().root_unresolved_references() {
            if VITEST_GLOBALS.contains(name)
                && !import_info.imported_names.contains(*name)
                && let Some(&first_ref_id) = reference_ids.first()
            {
                let reference = ctx.scoping().get_reference(first_ref_id);
                let node = ctx.nodes().get_node(reference.node_id());
                unimported_globals.push((*name, node.span()));
            }
        }

        if unimported_globals.is_empty() {
            return;
        }

        unimported_globals.sort_by_key(|(name, _)| *name);
        // Use the first occurrence in source order for the diagnostic label
        let first_span = unimported_globals
            .iter()
            .min_by_key(|(_, span)| span.start)
            .map_or(Span::empty(0), |(_, span)| *span);
        let globals_list: Vec<&str> = unimported_globals.iter().map(|(name, _)| *name).collect();
        let globals_str = globals_list.join(", ");

        ctx.diagnostic_with_fix(
            prefer_importing_vitest_globals_diagnostic(first_span, &globals_str),
            |fixer| {
                let module_record = ctx.module_record();
                // Filter out type-only imports since they don't create runtime bindings
                let vitest_imports: Vec<_> = module_record
                    .import_entries
                    .iter()
                    .filter(|entry| entry.module_request.name() == "vitest" && !entry.is_type)
                    .collect();

                let vitest_import = vitest_imports
                    .iter()
                    .find(|e| matches!(e.import_name, ImportImportName::Name(_)))
                    .or_else(|| {
                        vitest_imports
                            .iter()
                            .find(|e| matches!(e.import_name, ImportImportName::Default(_)))
                    })
                    .or_else(|| vitest_imports.first())
                    .copied();

                if let Some(import_entry) = vitest_import {
                    match &import_entry.import_name {
                        ImportImportName::NamespaceObject => {
                            let new_import = format!(
                                "import {{ {} }} from 'vitest';\n",
                                globals_list.join(", ")
                            );
                            fixer.insert_text_before_range(import_entry.statement_span, new_import)
                        }
                        ImportImportName::Default(_) => {
                            let source = ctx.source_range(import_entry.statement_span);
                            if let Some(from_pos) = source.find(" from ") {
                                #[expect(clippy::cast_possible_truncation)]
                                let insert_pos =
                                    import_entry.statement_span.start + from_pos as u32;
                                let insert_text = format!(", {{ {} }}", globals_list.join(", "));
                                fixer.insert_text_before_range(Span::empty(insert_pos), insert_text)
                            } else {
                                fixer.noop()
                            }
                        }
                        ImportImportName::Name(_) => {
                            let source = ctx.source_range(import_entry.statement_span);
                            let new_items = globals_list.join(", ");
                            if let Some((replace_span, replace_text)) = compute_brace_insert(
                                source,
                                import_entry.statement_span.start,
                                &new_items,
                            ) {
                                fixer.replace(replace_span, replace_text)
                            } else {
                                fixer.noop()
                            }
                        }
                    }
                } else if let Some(cjs_info) = &import_info.cjs_require {
                    match cjs_info {
                        CommonJSVitestRequire::Destructured { pattern_span } => {
                            let source = ctx.source_range(*pattern_span);
                            let new_items = globals_list.join(", ");
                            if let Some((replace_span, replace_text)) =
                                compute_brace_insert(source, pattern_span.start, &new_items)
                            {
                                fixer.replace(replace_span, replace_text)
                            } else {
                                fixer.noop()
                            }
                        }
                        CommonJSVitestRequire::DefaultOrNamespace { statement_start } => {
                            let new_import = format!(
                                "import {{ {} }} from 'vitest';\n",
                                globals_list.join(", ")
                            );
                            fixer
                                .insert_text_before_range(Span::empty(*statement_start), new_import)
                        }
                    }
                } else {
                    let new_import =
                        format!("import {{ {} }} from 'vitest';\n", globals_list.join(", "));
                    fixer.insert_text_before_range(Span::empty(0), new_import)
                }
            },
        );
    }
}

struct VitestImportInfo {
    imported_names: FxHashSet<String>,
    cjs_require: Option<CommonJSVitestRequire>,
}

enum CommonJSVitestRequire {
    Destructured { pattern_span: Span },
    DefaultOrNamespace { statement_start: u32 },
}

fn collect_vitest_import_info(ctx: &LintContext<'_>) -> VitestImportInfo {
    let mut imported_names = FxHashSet::default();
    let mut cjs_require = None;

    for entry in &ctx.module_record().import_entries {
        if entry.module_request.name() != "vitest" {
            continue;
        }

        // Type-only imports don't create runtime bindings
        if entry.is_type {
            continue;
        }

        if let ImportImportName::Name(_) = &entry.import_name {
            // `import { describe as d }` binds `d`, not `describe`
            imported_names.insert(entry.local_name.name().to_string());
        }
    }

    for node in ctx.nodes() {
        let AstKind::VariableDeclaration(var_decl) = node.kind() else { continue };
        for decl in &var_decl.declarations {
            let Some(Expression::CallExpression(call)) = &decl.init else { continue };
            if !is_require_call(call, "vitest") {
                continue;
            }

            match &decl.id {
                BindingPattern::ObjectPattern(obj_pat) => {
                    // `const { describe: d }` binds `d`, not `describe`
                    for prop in &obj_pat.properties {
                        if let BindingPattern::BindingIdentifier(ident) = &prop.value {
                            imported_names.insert(ident.name.to_string());
                        }
                    }

                    if cjs_require.is_none() {
                        if obj_pat.rest.is_some() {
                            cjs_require = Some(CommonJSVitestRequire::DefaultOrNamespace {
                                statement_start: var_decl.span.start,
                            });
                        } else {
                            cjs_require = Some(CommonJSVitestRequire::Destructured {
                                pattern_span: obj_pat.span,
                            });
                        }
                    }
                }
                BindingPattern::BindingIdentifier(_) => {
                    if cjs_require.is_none() {
                        cjs_require = Some(CommonJSVitestRequire::DefaultOrNamespace {
                            statement_start: var_decl.span.start,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    VitestImportInfo { imported_names, cjs_require }
}

fn is_require_call(call: &oxc_ast::ast::CallExpression<'_>, module_name: &str) -> bool {
    if let Expression::Identifier(ident) = &call.callee
        && ident.name == "require"
        && call.arguments.len() == 1
        && let Argument::StringLiteral(lit) = &call.arguments[0]
        && lit.value == module_name
    {
        return true;
    }
    false
}

/// Computes the span to replace and the replacement text for adding imports to an existing `{ ... }` block.
/// Returns `None` if no closing brace is found.
/// Handles trailing commas to avoid producing `{ foo,, bar }`.
/// Also removes any trailing whitespace before the closing brace.
#[expect(clippy::cast_possible_truncation)]
fn compute_brace_insert(source: &str, span_start: u32, new_items: &str) -> Option<(Span, String)> {
    let close_brace_pos = source.rfind('}')?;
    let before_brace = &source[..close_brace_pos];
    let trimmed = before_brace.trim_end();

    let replace_start = span_start + trimmed.len() as u32;
    let replace_end = span_start + close_brace_pos as u32;
    let needs_comma = !trimmed.ends_with(',');
    let replace_text =
        if needs_comma { format!(", {new_items} }}") } else { format!(" {new_items} }}") };

    Some((Span::new(replace_start, replace_end + 1), replace_text))
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
        "const { describe, describe } = require('vitest'); describe('suite', () => {});",
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
            "import { describe } from 'vitest';
			describe('suite', () => {});",
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
            "import { describe } from 'jest';
			describe('suite', () => {});",
            "import { describe } from 'vitest';
			import { describe } from 'jest';
			describe('suite', () => {});",
            None,
        ),
        (
            "import vitest from 'vitest';
			describe('suite', () => {});",
            "import vitest, { describe } from 'vitest';
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
            r#"import { "default" as vitest } from 'vitest'; describe('suite', () => {});"#,
            r#"import { "default" as vitest, describe } from 'vitest'; describe('suite', () => {});"#,
            None,
        ),
        (
            "const x = require('something', 'else'); describe('suite', () => {});",
            "import { describe } from 'vitest';
			const x = require('something', 'else'); describe('suite', () => {});",
            None,
        ),
        (
            "const x = require('jest'); describe('suite', () => {});",
            "import { describe } from 'vitest';
			const x = require('jest'); describe('suite', () => {});",
            None,
        ),
        (
            "const vitest = require('vitest'); describe('suite', () => {});",
            "import { describe } from 'vitest';
			const vitest = require('vitest'); describe('suite', () => {});",
            None,
        ),
        (
            "const { ...rest } = require('vitest'); describe('suite', () => {});",
            "const { ...rest, describe } = require('vitest'); describe('suite', () => {});",
            None,
        ),
        (
            r#"const { "default": vitest } = require('vitest'); describe('suite', () => {});"#,
            r#"const { "default": vitest, describe } = require('vitest'); describe('suite', () => {});"#,
            None,
        ),
        (
            "const { it } = require('vitest');
			describe('suite', () => {});",
            "const { it, describe } = require('vitest');
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
