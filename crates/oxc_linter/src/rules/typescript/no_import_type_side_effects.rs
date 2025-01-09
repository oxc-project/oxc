use oxc_ast::{
    ast::{ImportDeclarationSpecifier, ImportOrExportKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::{ContextHost, LintContext},
    fixer::Fix,
    rule::Rule,
    AstNode,
};

fn no_import_type_side_effects_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("TypeScript will only remove the inline type specifiers which will leave behind a side effect import at runtime.")
        .with_help("Convert this to a top-level type qualifier to properly remove the entire import.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoImportTypeSideEffects;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce the use of top-level `import type` qualifier when an import only
    /// has specifiers with inline type qualifiers.
    ///
    /// ### Why is this bad?
    ///
    /// The `--verbatimModuleSyntax` compiler option causes TypeScript to do
    /// simple and predictable transpilation on import declarations.  Namely, it
    /// completely removes import declarations with a top-level type qualifier,
    /// and it removes any import specifiers with an inline type qualifier.
    ///
    /// The latter behavior does have one potentially surprising effect in that
    /// in certain cases TS can leave behind a "side effect" import at runtime:
    ///
    /// ```ts
    /// import { type A, type B } from 'mod';
    /// ```
    ///
    /// is transpiled to
    ///
    /// ```ts
    /// import {} from 'mod';
    /// // which is the same as
    /// import 'mod';
    /// ```
    ///
    /// For the rare case of needing to import for side effects, this may be
    /// desirable - but for most cases you will not want to leave behind an
    /// unnecessary side effect import.
    ///
    /// ### Example
    /// ```ts
    /// import { type A } from 'mod';
    /// import { type A as AA } from 'mod';
    /// import { type A, type B } from 'mod';
    /// import { type A as AA, type B as BB } from 'mod';
    /// ```
    NoImportTypeSideEffects,
    typescript,
    restriction,
    fix
);

impl Rule for NoImportTypeSideEffects {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ImportDeclaration(import_decl) = node.kind() else {
            return;
        };

        if matches!(import_decl.import_kind, ImportOrExportKind::Type) {
            return;
        }

        let Some(specifiers) = &import_decl.specifiers else {
            return;
        };

        let mut type_specifiers = vec![];

        for specifier in specifiers {
            let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                return;
            };
            if matches!(specifier.import_kind, ImportOrExportKind::Value) {
                return;
            }
            type_specifiers.push(specifier);
        }
        // Can report and fix only if all specifiers are inline `type` qualifier:
        // `import { type A, type B } from 'foo.js'`
        ctx.diagnostic_with_fix(
            no_import_type_side_effects_diagnostic(import_decl.span),
            |_fixer| {
                let raw = ctx.source_range(import_decl.span);
                let mut fixes = vec![];

                // import type A from 'foo.js'
                //        ^^^^ add
                if raw.starts_with("import") {
                    fixes.push(Fix::new(
                        "import type",
                        Span::new(import_decl.span.start, import_decl.span.start + 6),
                    ));
                }

                for specifier in type_specifiers {
                    // import { type    A } from 'foo.js'
                    //          ^^^^^^^^
                    fixes.push(Fix::delete(Span::new(
                        specifier.span.start,
                        specifier.imported.span().start,
                    )));
                }

                fixes
            },
        );
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "import T from 'mod';",
        "import * as T from 'mod';",
        "import { T } from 'mod';",
        "import type { T } from 'mod';",
        "import type { T, U } from 'mod';",
        "import { type T, U } from 'mod';",
        "import { T, type U } from 'mod';",
        "import type T from 'mod';",
        "import type T, { U } from 'mod';",
        "import T, { type U } from 'mod';",
        "import type * as T from 'mod';",
        "import 'mod';",
    ];

    let fail = vec![
        "import { type A } from 'mod';",
        "import { type A as AA } from 'mod';",
        "import { type A, type B } from 'mod';",
        "import { type A as AA, type B as BB } from 'mod';",
    ];

    let fix = vec![
        ("import { type A } from 'mod';", "import type { A } from 'mod';", None),
        ("import { type A as AA } from 'mod';", "import type { A as AA } from 'mod';", None),
        ("import { type A, type B } from 'mod';", "import type { A, B } from 'mod';", None),
        (
            "import { type A as AA, type B as BB } from 'mod';",
            "import type { A as AA, B as BB } from 'mod';",
            None,
        ),
    ];
    Tester::new(NoImportTypeSideEffects::NAME, NoImportTypeSideEffects::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
