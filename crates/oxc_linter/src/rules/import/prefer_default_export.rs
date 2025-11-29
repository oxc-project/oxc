use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    context::LintContext,
    module_record::ExportEntry,
    rule::{DefaultRuleConfig, Rule},
};

fn prefer_default_export_diagnostic(span: Span, target: Target) -> OxcDiagnostic {
    let msg = if target == Target::Single {
        "Prefer default export on a file with single export."
    } else {
        "Prefer default export to be present on every file that has export."
    };
    OxcDiagnostic::warn(msg).with_help("Prefer a default export").with_label(span)
}

#[derive(Debug, Default, PartialEq, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum Target {
    #[default]
    Single,
    Any,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct PreferDefaultExport {
    /// Configuration option to specify the target type for preferring default exports.
    /// - `"single"`: Prefer default export when there is only one export in the module.
    /// - `"any"`: Prefer default export in any module that has exports.
    target: Target,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// In exporting files, this rule checks if there is default export or not.
    ///
    /// ### Why is this bad?
    ///
    /// This rule exists to standardize module exports by preferring default exports
    /// when a module only has one export, enhancing readability, maintainability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for the `{ target: "single" }` option:
    /// ```js
    /// export const foo = 'foo';
    /// ```
    ///
    /// Examples of **correct** code for the `{ target: "single" }` option:
    /// ```js
    /// export const foo = 'foo';
    /// const bar = 'bar';
    /// export default bar;
    /// ```
    ///
    /// Examples of **incorrect** code for the `{ target: "any" }` option:
    /// ```js
    /// export const foo = 'foo';
    /// export const baz = 'baz';
    /// ```
    ///
    /// Examples of **correct** code for the `{ target: "any" }` option:
    /// ```js
    /// export default function bar() {};
    /// ```
    PreferDefaultExport,
    import,
    style,
    config = PreferDefaultExport,
);

impl Rule for PreferDefaultExport {
    fn from_configuration(value: Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<PreferDefaultExport>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        if module_record.export_default.is_some() {
            return;
        }
        let star_export_entries = &module_record.star_export_entries;

        if !star_export_entries.is_empty() {
            return;
        }
        let indirect_entries = &module_record.indirect_export_entries;
        let local_entries = &module_record.local_export_entries;

        if exist_type(indirect_entries) || exist_type(local_entries) {
            return;
        }

        if self.target == Target::Single {
            if indirect_entries.len() + local_entries.len() == 1 {
                for entry in indirect_entries {
                    ctx.diagnostic(prefer_default_export_diagnostic(entry.span, self.target));
                }
                for entry in local_entries {
                    ctx.diagnostic(prefer_default_export_diagnostic(
                        entry.statement_span,
                        self.target,
                    ));
                }
            }
        } else {
            // find the last export statement
            if let Some(last_export_span) = indirect_entries
                .iter()
                .chain(local_entries.iter())
                .max_by_key(|entry| entry.statement_span.start)
            {
                ctx.diagnostic(prefer_default_export_diagnostic(
                    last_export_span.statement_span,
                    self.target,
                ));
            }
        }
    }
}

fn exist_type(export_entries: &[ExportEntry]) -> bool {
    export_entries.iter().any(|entry| entry.is_type)
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("export default a", None),
        ("export const { foo, bar } = item;", None),
        ("export const { foo, bar: baz } = item;", None),
        ("export const { foo: { bar, baz } } = item;", None),
        ("export const [a, b] = item;", None),
        (
            "
                let item;
                export const foo = item;
                export { item };
            ",
            None,
        ),
        (
            "
                let foo;
                export { foo as default }
            ",
            None,
        ),
        ("export const [CounterProvider,, withCounter] = func()", None),
        ("export * from './foo'", None),
        ("export default function bar() {};", None),
        (
            "
                export const foo = 'foo';
                export const bar = 'bar';
            ",
            None,
        ),
        (
            "
                export const foo = 'foo';
                export function bar() {};
            ",
            None,
        ),
        (
            "
                export const foo = 'foo';
                export default bar;
            ",
            None,
        ),
        (
            "
                let foo, bar;
                export { foo, bar }
            ",
            None,
        ),
        (
            "
                export default a;
                export * from './foo'
                export const a = 3;
                export { MemoryValue } from './Memory'
            ",
            None,
        ),
        ("export type foo = string", None),
        ("import * as foo from './foo'", None),
        ("export type UserId = number;", None),
        ("export { a, b } from 'foo.js'", None),
        ("let foo; export { foo as 'default' };", None),
        ("export default function bar() {};", Some(json!([{"target": "any"}]))),
        (
            "
                export const foo = 'foo';
                export const bar = 'bar';
                export default 42;
            ",
            Some(json!([{"target": "any"}])),
        ),
        ("export default a = 2;", Some(json!([{"target": "any"}]))),
        (
            "
                export const a = 2;
                export default function foo() {};
            ",
            Some(json!([{"target": "any"}])),
        ),
        (
            "
                export const a = 5;
                export function bar(){};
                let foo;
                export { foo as default }
            ",
            Some(json!([{"target": "any"}])),
        ),
        ("export * from './foo';", Some(json!([{"target": "any"}]))),
        ("import * as foo from './foo';", Some(json!([{"target": "any"}]))),
        ("const a = 5;", Some(json!([{"target": "any"}]))),
        (
            "export const a = 4; let foo; export { foo as 'default' };",
            Some(json!([{"target": "any"}])),
        ),
        (
            "
                export type foo = string;
                export type bar = number;
            ",
            None,
        ),
        (
            "
                export const a = 2;
                export { a } from './c';
                export type bar = number;
            ",
            Some(json!([{"target": "any"}])),
        ),
    ];

    let fail = vec![
        ("export const a = 3", None),
        ("export { MemoryValue } from './Memory'", None),
        ("export const a = 3", Some(json!([{"target": "any"}]))),
        ("export function bar() {}", None),
        ("export const foo = 'foo';", None),
        ("const foo = 'foo'; export { foo };", None),
        ("export const { foo } = { foo: 'bar' };", None),
        ("export const { foo: { bar } } = { foo: { bar: 'baz' } };", None),
        ("export const [a] = ['foo']", None),
        (
            "
                export const foo = 'foo'
                export const bar = 'bar';
            ",
            Some(json!([{"target": "any"}])),
        ),
        (
            "
                export const foo = 'foo';
                export function bar() {};
            ",
            Some(json!([{"target": "any"}])),
        ),
        (
            "
                let foo, bar;
                export { foo, bar }
            ",
            Some(json!([{"target": "any"}])),
        ),
        (
            "
                let item;
                export const foo = item;
                export { item };
            ",
            Some(json!([{"target": "any"}])),
        ),
        ("export { a, b } from 'foo.js'", Some(json!([{"target": "any"}]))),
        (
            "
                const foo = 'foo';
                export { foo };
            ",
            Some(json!([{"target": "any"}])),
        ),
        ("export const { foo } = { foo: 'bar' };", Some(json!([{"target": "any"}]))),
        (
            "export const { foo: { bar } } = { foo: { bar: 'baz' } };",
            Some(json!([{"target": "any"}])),
        ),
        (
            "
                export const b = 3;
                export const r = 3;
                export { last } from './Memory'
            ",
            Some(json!([{"target": "any"}])),
        ),
    ];

    Tester::new(PreferDefaultExport::NAME, PreferDefaultExport::PLUGIN, pass, fail)
        .test_and_snapshot();
}
