use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};
use oxc_syntax::module_record::ImportImportName;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-import(default): No default export found in imported module {0:?}")]
#[diagnostic(severity(warning))]
struct DefaultDiagnostic(Atom, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct Default;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// ### Example
    ///
    /// ```javascript
    /// ```
    Default,
    correctness
);

impl Rule for Default {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.semantic().module_record();
        for import_entry in &module_record.import_entries {
            let ImportImportName::Default(_) = import_entry.import_name else { continue };

            let specifier = import_entry.module_request.name();
            let Some(remote_module_record_ref) = module_record.loaded_modules.get(specifier) else {
                continue;
            };

            if remote_module_record_ref.export_default.is_none() {
                ctx.diagnostic(DefaultDiagnostic(
                    specifier.clone(),
                    import_entry.module_request.span(),
                ))
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["import './malformed.js'"];

    let fail = vec!["import baz from './named-exports';"];

    Tester::new_without_config(Default::NAME, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
