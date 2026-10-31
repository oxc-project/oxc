use oxc_ast::{AstKind, ast::ImportDeclarationSpecifier};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    fixer::Fix,
    rule::Rule,
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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// import { type A } from 'mod';
    /// import { type A as AA } from 'mod';
    /// import { type A, type B } from 'mod';
    /// import { type A as AA, type B as BB } from 'mod';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// import type { A } from 'mod';
    /// import type { A as AA } from 'mod';
    /// import type { A, B } from 'mod';
    /// import type { A as AA, B as BB } from 'mod';
    /// ```
    NoImportTypeSideEffects,
    typescript,
    restriction,
    fix,
    version = "0.5.0",
    short_description = "Enforce the use of top-level `import type` qualifier when an import only has specifiers with inline type qualifiers.",
);

impl Rule for NoImportTypeSideEffects {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ImportDeclaration(import_decl) = node.kind() else {
            return;
        };

        if import_decl.import_kind.is_type() {
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
            if specifier.import_kind.is_value() {
                return;
            }
            type_specifiers.push(specifier);
        }
        // Can report and fix only if all specifiers are inline `type` qualifier:
        // `import { type A, type B } from 'foo.js'`
        ctx.diagnostic_with_fix(
            no_import_type_side_effects_diagnostic(import_decl.span),
            |fixer| {
                let mut fix = fixer.new_fix_with_capacity(type_specifiers.len() + 2);
                let raw = ctx.source_range(import_decl.span);

                // import type A from 'foo.js'
                //        ^^^^ add
                if raw.starts_with("import") {
                    fix.push(Fix::new("import type", Span::sized(import_decl.span.start, 6)));
                }

                for specifier in type_specifiers {
                    // import { type    A } from 'foo.js'
                    //          ^^^^^^^^
                    fix.push(Fix::delete(Span::new(
                        specifier.span.start,
                        specifier.imported.span().start,
                    )));
                }

                // Cover the entire declaration so another fix cannot merge value specifiers
                // into an import that is becoming type-only in the same pass.
                fix.push(Fix::new("", Span::empty(import_decl.span.end)));

                fix.with_message("Convert to top-level type import")
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
        ("import { type A } from 'mod';", "import type { A } from 'mod';"),
        ("import { type A as AA } from 'mod';", "import type { A as AA } from 'mod';"),
        ("import { type A, type B } from 'mod';", "import type { A, B } from 'mod';"),
        (
            "import { type A as AA, type B as BB } from 'mod';",
            "import type { A as AA, B as BB } from 'mod';",
        ),
    ];

    Tester::new(NoImportTypeSideEffects::NAME, NoImportTypeSideEffects::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_no_duplicates_fix_conflict() {
    use serde_json::json;

    use crate::tester::Tester;

    let config = json!({
        "plugins": ["typescript", "import"],
        "rules": {
            "import/no-duplicates": ["error", { "prefer-inline": true }],
        },
    });

    let source = r#"import { type Foo } from "./x";
import { baz } from "./x";
export type T = Foo;
export const v = baz;"#;
    let first_pass = r#"import type { Foo } from "./x";
import { baz } from "./x";
export type T = Foo;
export const v = baz;"#;
    let fixed = r#"import { type Foo,baz } from "./x";
export type T = Foo;
export const v = baz;"#;

    let reversed = r#"import { baz } from "./x";
import { type Foo } from "./x";
export type T = Foo;
export const v = baz;"#;
    let reversed_fixed = r#"import { baz,type Foo } from "./x";
export type T = Foo;
export const v = baz;"#;

    let pass =
        vec![(fixed, None, Some(config.clone())), (reversed_fixed, None, Some(config.clone()))];
    let fix = vec![
        // Defer merging until the next pass, when the top-level type qualifier is visible.
        (source, first_pass, None, Some(config.clone())),
        (first_pass, fixed, None, Some(config.clone())),
        // Merging first must also prevent the other rule from converting the value import.
        (reversed, reversed_fixed, None, Some(config)),
    ];

    Tester::new(NoImportTypeSideEffects::NAME, NoImportTypeSideEffects::PLUGIN, pass, vec![])
        .with_import_plugin(true)
        .expect_fix(fix)
        .test();
}
