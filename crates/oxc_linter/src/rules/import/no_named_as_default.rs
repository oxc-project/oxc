use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::module_record::ImportImportName;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-import(no-named-as-default): Module {2:?} has named export {1:?}. Using default import as {1:?} can be confusing")]
#[diagnostic(severity(warning), help("Use another name for default import to avoid confusion"))]
struct NoNamedAsDefaultDiagnostic(#[label] pub Span, String, String);

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
    nursery
);

impl Rule for NoNamedAsDefault {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.semantic().module_record();
        for import_entry in &module_record.import_entries {
            let ImportImportName::Default(import_span) = import_entry.import_name else {
                continue;
            };
            let Some(import_name) = ctx
                .symbols()
                .get_symbol_id_from_span(&import_span)
                .map(|it| ctx.symbols().get_name(it))
            else {
                continue;
            };

            let specifier = import_entry.module_request.name();
            let Some(remote_module_record_ref) = module_record.loaded_modules.get(specifier) else {
                continue;
            };

            if remote_module_record_ref.exported_bindings.contains_key(import_name) {
                ctx.diagnostic(NoNamedAsDefaultDiagnostic(
                    import_span,
                    import_name.to_string(),
                    import_entry.module_request.name().to_string(),
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
        // TODO: parser error
        // r#"export default from "./bar";"#,
        r#"import bar, { foo } from "./export-default-string-and-named""#,
    ];

    let fail = vec![
        r#"import foo from "./bar";"#,
        r#"import foo, { foo as bar } from "./bar";"#,
        // TODO: parser error
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
