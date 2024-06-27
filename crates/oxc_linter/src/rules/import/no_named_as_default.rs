use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::module_record::ImportImportName;

use crate::{context::LintContext, rule::Rule};

fn no_named_as_default_diagnostic(span0: Span, x1: &str, x2: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("eslint-plugin-import(no-named-as-default): Module {x2:?} has named export {x1:?}"))
        .with_help(format!("Using default import as {x1:?} can be confusing. Use another name for default import to avoid confusion."))
        .with_label(span0)
}

/// <https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/no-named-as-default-member.md>
#[derive(Debug, Default, Clone)]
pub struct NoNamedAsDefault;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports use of an exported name as the locally imported name of a default export.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // foo.js
    /// export default 'foo';
    /// export const bar = 'baz';
    /// ```
    /// Valid:
    /// ```javascript
    /// import foo from './foo.js';
    /// ```
    /// Invalid:
    /// ```javascript
    /// // using exported name 'bar' as identifier for default export.
    /// import bar from './foo.js';
    /// ```
    NoNamedAsDefault,
    suspicious
);

impl Rule for NoNamedAsDefault {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        for import_entry in &module_record.import_entries {
            let ImportImportName::Default(import_span) = &import_entry.import_name else {
                continue;
            };

            let specifier = import_entry.module_request.name();
            let Some(remote_module_record_ref) = module_record.loaded_modules.get(specifier) else {
                continue;
            };

            let import_name = import_entry.local_name.name();
            if remote_module_record_ref.exported_bindings.contains_key(import_name) {
                ctx.diagnostic(no_named_as_default_diagnostic(
                    *import_span,
                    import_name,
                    import_entry.module_request.name(),
                ));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import "./malformed.js""#,
        r#"import bar, { foo } from "./bar";"#,
        r#"import bar, { foo } from "./empty-folder";"#,
        // unsupported syntax
        // r#"export default from "./bar";"#,
        r#"import bar, { foo } from "./export-default-string-and-named""#,
    ];

    let fail = vec![
        r#"import foo from "./bar";"#,
        r#"import foo, { foo as bar } from "./bar";"#,
        // unsupported syntax
        // r#"export default, { foo as bar } from "./bar";"#,
        // r#"import foo from "./malformed.js""#,
        r#"import foo from "./export-default-string-and-named""#,
        r#"import foo, { foo as bar } from "./export-default-string-and-named""#,
    ];

    Tester::new(NoNamedAsDefault::NAME, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
