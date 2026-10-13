use oxc_ast::ast::{ExportAllDeclaration, Statement, WithClause};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule};

fn no_useless_re_export_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Redundant re-export already covered by `export *` from the same module.")
        .with_help("Remove this export specifier.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessReExport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows named re-exports that are already covered by an `export *` from the same module.
    ///
    /// Module specifiers and import attributes are compared as written.
    /// Paths are not resolved and the module graph is not inspected.
    /// Named re-exports are ignored when the file has `export *` declarations
    /// for multiple distinct module requests, as those can resolve conflicting names.
    ///
    /// ### Why is this bad?
    ///
    /// `export * from './foo.js'` already re-exports every named export of `./foo.js`,
    /// so an additional `export {foo} from './foo.js'` is redundant.
    /// Removing it keeps the module's API declarations smaller and avoids implying
    /// that the export is special when the wildcard already exposes it.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// export * from './foo.js';
    /// export {foo} from './foo.js';
    /// ```
    ///
    /// ```ts
    /// export * from './foo.js';
    /// export type {Foo} from './foo.js';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// export * from './foo.js';
    /// export {foo as bar} from './foo.js';
    /// export {default} from './foo.js';
    /// ```
    ///
    /// ```ts
    /// // `export type *` does not re-export values.
    /// export type * from './foo.js';
    /// export {foo} from './foo.js';
    /// ```
    ///
    /// ### Differences from `eslint-plugin-unicorn`
    ///
    /// Only top-level `export` declarations are checked.
    /// Declarations inside TypeScript `declare module` blocks are ignored,
    /// since they describe a different module than the file itself.
    NoUselessReExport,
    unicorn,
    style,
    pending,
    version = "next",
    short_description = "Disallow redundant re-exports already covered by `export *`.",
);

fn is_same_module_request(
    ctx: &LintContext,
    (a_source, a_with_clause): (&str, Option<&WithClause>),
    (b_source, b_with_clause): (&str, Option<&WithClause>),
) -> bool {
    if a_source != b_source {
        return false;
    }
    let a = a_with_clause.map(|with| with.with_entries.as_slice()).unwrap_or_default();
    let b = b_with_clause.map(|with| with.with_entries.as_slice()).unwrap_or_default();
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).all(|(x, y)| ctx.source_range(x.span) == ctx.source_range(y.span))
}

impl Rule for NoUselessReExport {
    fn run_once(&self, ctx: &LintContext) {
        let program = ctx.nodes().program();

        let mut wildcard_export: Option<&ExportAllDeclaration> = None;
        let mut has_value_wildcard = false;

        // Wildcards for different module requests may export conflicting names,
        // in which case a named re-export can be meaningful.
        for stmt in &program.body {
            let Statement::ExportAllDeclaration(declaration) = stmt else { continue };
            if declaration.exported.is_some() {
                continue;
            }

            if let Some(first) = wildcard_export {
                if !is_same_module_request(
                    ctx,
                    (first.source.value.as_str(), first.with_clause.as_deref()),
                    (declaration.source.value.as_str(), declaration.with_clause.as_deref()),
                ) {
                    return;
                }
            } else {
                wildcard_export = Some(declaration.as_ref());
            }
            has_value_wildcard |= declaration.export_kind.is_value();
        }
        let Some(wildcard_export) = wildcard_export else { return };

        let wildcard_request =
            (wildcard_export.source.value.as_str(), wildcard_export.with_clause.as_deref());

        for stmt in &program.body {
            let Statement::ExportFromDeclaration(export_from) = stmt else { continue };
            let export_from_request =
                (export_from.source.value.as_str(), export_from.with_clause.as_deref());
            if !is_same_module_request(ctx, export_from_request, wildcard_request) {
                continue;
            }

            for specifier in &export_from.specifiers {
                if specifier.exported.name() == "default" {
                    continue;
                }
                // `foo as foo` is handled by `no-useless-rename`.
                if specifier.local.span() != specifier.exported.span() {
                    continue;
                }
                if !has_value_wildcard
                    && export_from.export_kind.is_value()
                    && specifier.export_kind.is_value()
                {
                    continue;
                }

                ctx.diagnostic(no_useless_re_export_diagnostic(specifier.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "export {foo} from './foo.js';",
        "import {foo} from './foo.js';
            export {foo};",
        "export * as namespace from './foo.js';",
        "import foo from './foo.js';
            export {foo as default};",
        "export * from './foo.js';
            export {foo as bar} from './foo.js';",
        "export * from './foo.js';
            export {foo as foo} from './foo.js';",
        "export * from './foo.js';
            export {default} from './foo.js';",
        "export * from './foo.js';
            export const foo = 1;",
        "export * from './foo.js';
            export * from './foo.js';",
        "import {foo} from './bar.js';
            export * from './foo.js';
            export {foo};",
        "import {foo} from './foo.js';
            export * from './foo.js';
            export {foo as bar};",
        "export * from './foo.js';
            export * from './bar.js';
            export {foo} from './foo.js';",
        "import {foo} from './foo.js';
            export * from './foo.js';
            export * from './bar.js';
            export {foo};",
        "import {foo} from './foo.js';
            export * from './foo.js';
            export {foo};",
        "import {foo as bar} from './foo.js';
            export * from './foo.js';
            export {bar as foo};",
        "export * from './foo.json' with {type: 'json'};
            export {foo} from './foo.json';",
        "export type * from './foo.js';
            export {foo} from './foo.js';",
        "import type {Foo} from './foo.js';
            export * from './foo.js';
            export {type Foo};",
        "export * from './foo.js';
            export * from './bar.js';
            export {bar} from './bar.js';",
        "export * from './foo.js' with {type: 'json', x: '1'};
            export {foo} from './foo.js' with {type: 'json', x: '2'};",
        "export * as ns from './foo.js';
            export {foo} from './foo.js';",
        "export * from './foo.js';
            export {'foo' as 'bar'} from './foo.js';",
        "export * from './foo.js';
            export {'default'} from './foo.js';",
        "declare module 'x' {
                export * from './foo.js';
                export {foo} from './foo.js';
            }",
        "declare module 'x' {
                export * from './foo.js';
            }
            export {foo} from './foo.js';",
    ];

    let fail = vec![
        "export * from './foo.js';
            export {foo} from './foo.js';",
        "export {foo} from './foo.js';
            export * from './foo.js';",
        "export * from './foo.js';
            export type {Foo} from './foo.js';",
        "export type * from './foo.js';
            export type {Foo} from './foo.js';",
        "export * from './foo.js';
            export * from './foo.js';
            export {foo} from './foo.js';",
        "export * from './foo.js';
            export * as ns from './bar.js';
            export {foo} from './foo.js';",
        "export type * from './foo.js';
            export * from './foo.js';
            export {foo} from './foo.js';",
        "export * from './foo.js';
            export type * from './foo.js';
            export {foo} from './foo.js';",
        "export * from './foo.js';
            export {type Foo} from './foo.js';",
        "export * from './foo.js';
            export {foo, bar as baz} from './foo.js';",
        "export * from './foo.js';
            export {'foo'} from './foo.js';",
        "export * from './foo.json' with {type: 'json'};
            export {foo} from './foo.json' with {type: 'json'};",
        "export * from './foo.js';
            export {foo} from './foo.js';
            declare module 'x' {
                export * from './bar.js';
            }",
    ];

    Tester::new(NoUselessReExport::NAME, NoUselessReExport::PLUGIN, pass, fail).test_and_snapshot();
}
