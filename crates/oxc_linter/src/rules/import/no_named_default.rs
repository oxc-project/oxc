use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, module_record::ImportImportName, rule::Rule};

fn no_named_default_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Replace default import with named import.")
        .with_help("Forbid named default exports.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNamedDefault;

declare_oxc_lint!(
    /// ### What it does
    /// Reports use of a default export as a locally named import.
    ///
    /// ### Why is this bad?
    /// Rationale: the syntax exists to import default exports expressively, let's use it.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // message: Using exported name 'bar' as identifier for default export.
    /// import { default as foo } from './foo.js';
    /// import { default as foo, bar } from './foo.js';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import foo from './foo.js';
    /// import foo, { bar } from './foo.js';
    /// ```
    NoNamedDefault,
    import,
    style,
);

impl Rule for NoNamedDefault {
    fn run_once(&self, ctx: &LintContext) {
        ctx.module_record().import_entries.iter().for_each(|entry| {
            let ImportImportName::Name(import_name) = &entry.import_name else {
                return;
            };
            if import_name.name() == "default" && !entry.is_type {
                ctx.diagnostic(no_named_default_diagnostic(import_name.span()));
            }
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import bar from "./bar";"#,
        r#"import bar, { foo } from "./bar";"#,
        r#"import { type default as Foo } from "./bar";"#,
    ];

    let fail = vec![
        r#"import { default as bar } from "./bar";"#,
        r#"import { foo, default as bar } from "./bar";"#,
        r#"import { "default" as bar } from "./bar";"#,
    ];

    Tester::new(NoNamedDefault::NAME, NoNamedDefault::PLUGIN, pass, fail).test_and_snapshot();
}
