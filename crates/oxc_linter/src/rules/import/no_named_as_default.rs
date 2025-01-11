use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, module_record::ImportImportName, rule::Rule};

fn no_named_as_default_diagnostic(
    span: Span,
    module_name: &str,
    export_name: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Module {export_name:?} has named export {module_name:?}"))
        .with_help(format!("Using default import as {module_name:?} can be confusing. Use another name for default import to avoid confusion."))
        .with_label(span)
}

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/no-named-as-default-member.md>
#[derive(Debug, Default, Clone)]
pub struct NoNamedAsDefault;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports use of an exported name as the locally imported name of a default export.
    /// This happens when an imported default export is assigned a name that conflicts
    /// with a named export from the same module.
    ///
    /// ### Why is this bad?
    ///
    /// Using a named export's identifier for a default export can cause confusion
    /// and errors in understanding which value is being imported. It also reduces
    /// code clarity, making it harder for other developers to understand the intended
    /// imports.
    ///
    ///
    /// ### Examples
    ///
    /// Given
    /// ```javascript
    /// // foo.js
    /// export default 'foo';
    /// export const bar = 'baz';
    /// ```
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// // Invalid: using exported name 'bar' as the identifier for default export.
    /// import bar from './foo.js';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// // Valid: correctly importing default export with a non-conflicting name.
    /// import foo from './foo.js';
    /// ```
    NoNamedAsDefault,
    import,
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
            let remote_module_record = module_record.loaded_modules.read().unwrap();
            let Some(remote_module_record) = remote_module_record.get(specifier) else {
                continue;
            };

            let import_name = import_entry.local_name.name();
            if remote_module_record.exported_bindings.contains_key(import_name) {
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

    Tester::new(NoNamedAsDefault::NAME, NoNamedAsDefault::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
