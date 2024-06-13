use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_exports_found(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-import(no-unused-modules): No exports found")
        .with_labels([span0.into()])
}

/// <https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/no-unused-modules.md>
#[derive(Debug, Default, Clone)]
pub struct NoUnusedModules {
    missing_exports: bool,
    unused_exports: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports:
    /// * modules without any exports
    /// * individual exports not being statically imported or requireed from other modules in the same project
    /// * dynamic imports are supported if argument is a literal string
    ///
    NoUnusedModules,
    nursery
);

impl Rule for NoUnusedModules {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
            missing_exports: value
                .get("missingExports")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            unused_exports: value
                .get("unusedExports")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        if self.missing_exports && module_record.local_export_entries.is_empty() {
            ctx.diagnostic(no_exports_found(Span::new(0, 0)));
        }
        if self.unused_exports {
            // TODO: implement unused exports
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let missing_exports_options = json!({
      "missingExports": true,
    });

    let pass = vec![
        ("export default function noOptions() {}", None),
        ("export default () => 1", Some(missing_exports_options.clone())),
        ("const a = 1; export { a }", Some(missing_exports_options.clone())),
        ("function a() { return true }; export { a }", Some(missing_exports_options.clone())),
        ("const a = 1; const b = 2; export { a, b }", Some(missing_exports_options.clone())),
        ("const a = 1; export default a", Some(missing_exports_options.clone())),
        ("export class Foo {}", Some(missing_exports_options.clone())),
        ("export const [foobar] = [];", Some(missing_exports_options.clone())),
        ("export const [foobar] = foobarFactory();", Some(missing_exports_options.clone())),
        (
            "export default function NewComponent () {
            return 'I am new component'
          }",
            Some(missing_exports_options.clone()),
        ),
        (
            "export default function NewComponent () {
            return 'I am new component'
          }",
            Some(missing_exports_options.clone()),
        ),
    ];

    let fail = vec![
        ("const a = 1", Some(missing_exports_options.clone())),
        ("/* const a = 1 */", Some(missing_exports_options.clone())),
    ];

    Tester::new(NoUnusedModules::NAME, pass, fail)
        .change_rule_path("missing-exports.js")
        .with_import_plugin(true)
        .test_and_snapshot();

    // TODO: support unused exports
    // let unused_exports_options = json!({
    //   "unusedExports": true,
    //   "src": ["./no-unused-modules/**/*.js"],
    //   "ignoreExports": ["./no-unused-modules/*ignored*.js"],
    // });

    // let pass = vec![
    //     ("export default function noOptions() {}", None),
    //     ("export default () => 1", Some(unused_exports_options)),
    // ];

    // let fail = vec![];

    // Tester::new(NoUnusedModules::NAME, pass, fail)
    //     .change_rule_path("unused-exports.js")
    //     .with_import_plugin(true)
    //     .test_and_snapshot();
}
