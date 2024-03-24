use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-import(no-default-export): Prefer named exports")]
#[diagnostic(severity(warning))]
struct NoDefaultExportDiagnostic(#[label] Span);

#[derive(Debug, Default, Clone)]
pub struct NoDefaultExport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbid a module to have a default exports. This help your editor to provide better auto imports.
    ///
    /// ### Examples
    ///
    /// ```javascript
    /// // bad1.js
    ///
    /// // There is a default export.
    /// export const foo = 'foo';
    /// const bar = 'bar';
    /// export default 'bar';
    /// ```
    ///
    /// ```javascript
    /// // bad2.js
    ///
    /// // There is a default export.
    /// const foo = 'foo';
    /// export { foo as default }
    /// ```
    ///
    NoDefaultExport,
    nursery
);

impl Rule for NoDefaultExport {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.semantic().module_record();
        write_diagnostic_optional(ctx, module_record.export_default);
        module_record.export_default_duplicated.iter().for_each(|it| write_diagnostic(ctx, *it));
        write_diagnostic_optional(ctx, module_record.exported_bindings.get("default").copied());
    }
}

fn write_diagnostic(ctx: &LintContext<'_>, span: Span) {
    ctx.diagnostic(NoDefaultExportDiagnostic(span));
}
fn write_diagnostic_optional(ctx: &LintContext<'_>, span_option: Option<Span>) {
    if let Some(span) = span_option {
        write_diagnostic(ctx, span);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "export const foo = 'foo'; export const bar = 'bar';",
        "export const foo = 'foo'; export function bar() {};",
        "export const foo = 'foo';",
        "const foo = 'foo'; export { foo };",
        "let foo, bar; export { foo, bar }",
        "export const { foo, bar } = item;",
        "export const { foo, bar: baz } = item;",
        "export const { foo: { bar, baz } } = item;",
        "let item; export const foo = item; export { item };",
        "export * from './foo';",
        "export const { foo } = { foo: 'bar' };",
        "export const { foo: { bar } } = { foo: { bar: 'baz' } };",
        "export { a, b } from 'foo.js'",
        "import * as foo from './foo';",
        "import foo from './foo';",
        "import {default as foo} from './foo';",
        "export type UserId = number;",
    ];
    let fail = vec![
        "export default function bar() {};",
        "export const foo = 'foo';\nexport default bar;",
        "export default class Bar {};",
        "export default function() {};",
        "export default class {};",
        "let foo; export { foo as default }",
        // "export default from \"foo.js\"",
    ];

    Tester::new(NoDefaultExport::NAME, pass, fail)
        .with_import_plugin(true)
        .change_rule_path("index.ts")
        .test_and_snapshot();
}
