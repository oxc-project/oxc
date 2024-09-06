use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_default_export_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer named exports").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDefaultExport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbid a module to have a default exports. This help your editor to provide better auto imports.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// export default 'bar';
    ///
    /// const foo = 'foo';
    /// export { foo as default }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// export const foo = 'foo';
    /// export const bar = 'bar';
    /// ```
    ///
    NoDefaultExport,
    restriction
);

impl Rule for NoDefaultExport {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        write_diagnostic_optional(ctx, module_record.export_default);
        module_record.export_default_duplicated.iter().for_each(|it| write_diagnostic(ctx, *it));
        write_diagnostic_optional(ctx, module_record.exported_bindings.get("default").copied());
    }
}

fn write_diagnostic(ctx: &LintContext<'_>, span: Span) {
    ctx.diagnostic(no_default_export_diagnostic(span));
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
