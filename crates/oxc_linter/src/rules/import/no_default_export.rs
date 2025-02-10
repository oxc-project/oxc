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
    /// Forbids a module from having default exports. This helps your editor
    /// provide better auto-import functionality, as named exports offer more
    /// explicit and predictable imports compared to default exports.
    ///
    /// ### Why is this bad?
    ///
    /// Default exports can lead to confusion, as the name of the imported value
    /// can vary based on how it's imported. This can make refactoring and
    /// auto-imports less reliable.
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
    NoDefaultExport,
    import,
    restriction
);

impl Rule for NoDefaultExport {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        if let Some(span) = module_record.export_default {
            ctx.diagnostic(no_default_export_diagnostic(span));
        }
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

    Tester::new(NoDefaultExport::NAME, NoDefaultExport::PLUGIN, pass, fail)
        .with_import_plugin(true)
        .change_rule_path("index.ts")
        .test_and_snapshot();
}
