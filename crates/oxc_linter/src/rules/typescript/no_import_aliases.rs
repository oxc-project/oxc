use oxc_ast::{AstKind, ast::ImportOrExportKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_import_aliases_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected import alias")
        .with_help("Import aliases are not erasable syntax and are incompatible with TypeScript's --erasableSyntaxOnly flag. Consider using regular import syntax instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoImportAliases;

// Ported from <https://github.com/JoshuaKGoldberg/eslint-plugin-erasable-syntax-only/blob/main/src/rules/import-aliases.ts/>
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow TypeScript import aliases
    ///
    /// ### Why is this bad?
    ///
    /// TypeScript 5.8 introduces the `--erasableSyntaxOnly` flag. When this flag is enabled,
    /// TypeScript will only allow you to use constructs that can be erased from a file, and
    /// will issue an error if it encounters any constructs that cannot be erased.
    ///
    /// Import aliases are not erasable syntax because they generate runtime code and cannot be
    /// completely removed during compilation. This makes them incompatible with the
    /// `--erasableSyntaxOnly` flag.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// import values = require("values");
    /// import utils = MyUtils;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// // Use regular import syntax instead
    /// import values from "values";
    /// import * as values from "values";
    ///
    /// // Type-only imports are allowed
    /// import type utils = MyUtils;
    /// ```
    NoImportAliases,
    typescript,
    restriction,
    pending
);

impl Rule for NoImportAliases {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::TSImportEqualsDeclaration(import_decl) = node.kind() {
            if import_decl.import_kind == ImportOrExportKind::Type {
                // Type-only imports are allowed
                return;
            }

            ctx.diagnostic(no_import_aliases_diagnostic(import_decl.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Regular import syntax
        r#"import values from "values";"#,
        // Namespace import
        r#"import * as values from "values";"#,
        // Type-only import alias (should be allowed)
        r"import type values = Values;",
        // Named import
        r#"import { values } from "values";"#,
        // Default and named imports
        r#"import values, { other } from "values";"#,
    ];

    let fail = vec![
        // Import alias with require
        r#"import values = require("values");"#,
        // Import alias with identifier
        r"import values = Values;",
        // Import alias with module reference
        r"import utils = MyNamespace.Utils;",
        // Exported import alias
        r#"export import values = require("values");"#,
        // Import alias in module
        r#"module MyModule { import values = require("values"); }"#,
    ];

    Tester::new(NoImportAliases::NAME, NoImportAliases::PLUGIN, pass, fail).test_and_snapshot();
}
