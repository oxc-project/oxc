use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    module_record::{ExportExportName, ExportImportName, ImportImportName, ModuleRecord},
    rule::Rule,
};

fn no_named_as_default_diagnostic(
    span: Span,
    module_name: &str,
    export_name: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Module {export_name:?} has named export {module_name:?}"))
        .with_help(format!("Using default import as {module_name:?} can be confusing. Use another name for default import to avoid confusion."))
        .with_label(span)
}

// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/no-named-as-default.md>
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
            let Some(remote_module_record) = module_record.get_loaded_module(specifier) else {
                continue;
            };

            let import_name = import_entry.local_name.name();
            if !remote_module_record.exported_bindings.contains_key(import_name) {
                continue;
            }

            if default_and_named_are_same_reexport(&remote_module_record, import_name) {
                continue;
            }

            ctx.diagnostic(no_named_as_default_diagnostic(
                *import_span,
                import_name,
                import_entry.module_request.name(),
            ));
        }
    }
}

/// Check if the remote module re-exports both the default and the given named export
/// from the same source module with the same local identifier.
///
/// This is a special case where using the named export's name for the default import is allowed,
/// because they refer to the same value.
///
/// See <https://github.com/import-js/eslint-plugin-import/pull/3032>
/// and <https://github.com/oxc-project/oxc/issues/19099>
fn default_and_named_are_same_reexport(remote_module_record: &ModuleRecord, name: &str) -> bool {
    // Find the default re-export entry.
    // Only re-exports like `export { foo as default }` are found here.
    let Some(default_entry) = remote_module_record.indirect_export_entries.iter().find(
        |entry| matches!(&entry.export_name, ExportExportName::Name(n) if n.name() == "default"),
    ) else {
        return false;
    };

    // Find the named re-export entry
    let Some(named_entry) = remote_module_record
        .indirect_export_entries
        .iter()
        .find(|entry| matches!(&entry.export_name, ExportExportName::Name(n) if n.name() == name))
    else {
        return false;
    };

    // Both must have a module_request (i.e., be re-exports)
    let (Some(default_module), Some(named_module)) =
        (&default_entry.module_request, &named_entry.module_request)
    else {
        return false;
    };

    // Both must re-export from the same module
    if default_module.name() != named_module.name() {
        return false;
    }

    // Both must import the same identifier from that module.
    let (ExportImportName::Name(default_import), ExportImportName::Name(named_import)) =
        (&default_entry.import_name, &named_entry.import_name)
    else {
        return false;
    };

    // For direct re-exports (`export { foo as default } from './source'`), the import_name
    // directly reflects the source binding name and can be compared as-is.
    //
    // For import-then-export (`import foo from './source'; export { foo as default }`),
    // the parser sets import_name to the local name ("foo") rather than "default".
    // In that case the import_name is unreliable, so we resolve the actual source name
    // by checking the remote module's import entries.
    let default_source_name = remote_module_record
        .import_entries
        .iter()
        .find(|entry| {
            entry.import_name.is_default()
                && entry.module_request.name() == default_module.name()
                && entry.local_name.name() == default_import.name()
        })
        .map_or(default_import.name(), |_| "default");

    default_source_name == named_import.name()
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
        // When both default and named exports are re-exported from the same source,
        // using the named export's name for the default import is not confusing.
        r#"import userEvent from "./re-export-default-and-named""#,
        // Also allowed.
        r#"import { userEvent } from "./re-export-default-and-named""#,
        // Import-then-export of the same named binding as both default and named.
        // Both refer to the same source binding, so this is allowed.
        r#"import userEvent from "./re-export-default-and-named-import-then-export""#,
    ];

    let fail = vec![
        r#"import foo from "./bar";"#,
        r#"import foo, { foo as bar } from "./bar";"#,
        // unsupported syntax
        // r#"export default, { foo as bar } from "./bar";"#,
        // r#"import foo from "./malformed.js""#,
        r#"import foo from "./export-default-string-and-named""#,
        r#"import foo, { foo as bar } from "./export-default-string-and-named""#,
        // When default and named exports are re-exported from different sources,
        // it should still report.
        r#"import userEvent from "./re-export-default-and-named-misleading""#,
        // When default and named exports are re-exported through local aliases
        // that map to different remote symbols, it should still report.
        r#"import userEvent from "./re-export-default-and-named-alias-misleading""#,
        // When default and named exports are re-exported from the same source
        // but refer to different bindings, it should still report.
        r#"import userEvent from "./re-export-default-and-named-different-binding""#,
    ];

    Tester::new(NoNamedAsDefault::NAME, NoNamedAsDefault::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
