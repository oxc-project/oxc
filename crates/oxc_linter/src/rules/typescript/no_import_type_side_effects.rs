use oxc_ast::{
    ast::{ImportDeclarationSpecifier, ImportOrExportKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_import_type_side_effects_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("typescript-eslint(no-import-type-side-effects): TypeScript will only remove the inline type specifiers which will leave behind a side effect import at runtime.")
        .with_help("Convert this to a top-level type qualifier to properly remove the entire import.")
        .with_labels([span0.into()])
}

#[derive(Debug, Default, Clone)]
pub struct NoImportTypeSideEffects;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce the use of top-level import type qualifier when an import only has specifiers with inline type qualifiers.
    ///
    /// ### Why is this bad?
    ///
    /// The `--verbatimModuleSyntax` compiler option causes TypeScript to do simple and predictable transpilation on import declarations.
    /// Namely, it completely removes import declarations with a top-level type qualifier, and it removes any import specifiers with an inline type qualifier.
    ///
    /// The latter behavior does have one potentially surprising effect in that in certain cases TS can leave behind a "side effect" import at runtime:

    /// ```javascript
    /// import { type A, type B } from 'mod';
    /// ```

    /// is transpiled to
    ///
    /// ```javascript
    /// import {} from 'mod';
    /// which is the same as
    /// import 'mod';
    /// ```

    /// For the rare case of needing to import for side effects, this may be desirable - but for most cases you will not want to leave behind an unnecessary side effect import.
    ///
    /// ### Example
    /// ```javascript
    /// import { type A } from 'mod';
    /// import { type A as AA } from 'mod';
    /// import { type A, type B } from 'mod';
    /// import { type A as AA, type B as BB } from 'mod';
    /// ```
    NoImportTypeSideEffects,
    restriction,
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
            |fixer| {
                let mut delete_ranges = vec![];

                for specifier in type_specifiers {
                    // import { type    A } from 'foo.js'
                    //          ^^^^^^^^
                    delete_ranges
                        .push(Span::new(specifier.span.start, specifier.imported.span().start));
                }

                let mut output = String::new();
                let mut last_pos = import_decl.span.start;
                for range in delete_ranges {
                    // import      { type A } from 'foo.js'
                    // ^^^^^^^^^^^^^^^
                    // |             |
                    // [last_pos      range.start)
                    output.push_str(ctx.source_range(Span::new(last_pos, range.start)));
                    // import      { type A } from 'foo.js'
                    //                    ^
                    //                    |
                    //                    last_pos
                    last_pos = range.end;
                }

                // import      { type A } from 'foo.js'
                //                    ^^^^^^^^^^^^^^^^^^
                //                    ^                ^
                //                    |                |
                //                    [last_pos        import_decl_span.end)
                output.push_str(ctx.source_range(Span::new(last_pos, import_decl.span.end)));

                if let Some(output) = output.strip_prefix("import ") {
                    let output = format!("import type {output}");
                    fixer.replace(import_decl.span, output)
                } else {
                    // Do not do anything, this should never happen
                    fixer.replace(import_decl.span, ctx.source_range(import_decl.span))
                }
            },
        );
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
    Tester::new(NoImportTypeSideEffects::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
