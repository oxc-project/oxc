use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_relative_parent_imports_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Relative imports from parent directories are not allowed")
        .with_help(
            "Move the file to the same directory, use dependency injection, or convert to a package.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRelativeParentImports;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids importing modules from parent directories using relative paths.
    ///
    /// ### Why is this bad?
    ///
    /// This restriction enforces tree-like folder structures instead of complex
    /// graph-like structures, making large codebases easier to maintain.
    /// Dependencies flow in one direction (parent to child), which clarifies
    /// module relationships.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import foo from '../bar';
    /// import foo from '../../utils/helper';
    /// const baz = require('../config');
    /// export { qux } from '../shared';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import foo from 'lodash';
    /// import a from './lib/a';
    /// import b from './b';
    /// ```
    NoRelativeParentImports,
    import,
    restriction,
);

impl Rule for NoRelativeParentImports {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            // ESM import declarations
            AstKind::ImportDeclaration(import_decl)
                if is_parent_import(import_decl.source.value.as_str()) =>
            {
                ctx.diagnostic(no_relative_parent_imports_diagnostic(import_decl.source.span));
            }
            // ESM export { } from '...'
            AstKind::ExportNamedDeclaration(export_decl) => {
                if let Some(source) = &export_decl.source
                    && is_parent_import(source.value.as_str())
                {
                    ctx.diagnostic(no_relative_parent_imports_diagnostic(source.span));
                }
            }
            // ESM export * from '...'
            AstKind::ExportAllDeclaration(export_decl)
                if is_parent_import(export_decl.source.value.as_str()) =>
            {
                ctx.diagnostic(no_relative_parent_imports_diagnostic(export_decl.source.span));
            }
            // Dynamic import expressions: import('../foo')
            AstKind::ImportExpression(import_expr) => {
                if let Expression::StringLiteral(str_literal) = &import_expr.source
                    && is_parent_import(str_literal.value.as_str())
                {
                    ctx.diagnostic(no_relative_parent_imports_diagnostic(str_literal.span));
                }
            }
            // CommonJS require() calls
            AstKind::CallExpression(call_expr) => {
                if let Expression::Identifier(ident) = &call_expr.callee
                    && ident.name == "require"
                    && call_expr.arguments.len() == 1
                    && let Argument::StringLiteral(str_literal) = &call_expr.arguments[0]
                    && is_parent_import(str_literal.value.as_str())
                {
                    ctx.diagnostic(no_relative_parent_imports_diagnostic(str_literal.span));
                }
            }
            _ => {}
        }
    }
}

/// Check if the import path is a relative parent import.
/// Matches paths like `../foo`, `..`, `./../foo`, etc.
fn is_parent_import(path: &str) -> bool {
    let mut normalized = path;
    while let Some(rest) = normalized.strip_prefix("./") {
        normalized = rest;
    }
    normalized == ".." || normalized.starts_with("../")
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // ESLint test cases
        r#"import foo from "./internal.js""#,
        r#"import foo from "./app/index.js""#,
        r#"import foo from "package""#,
        r#"require("./internal.js")"#,
        r#"require("./app/index.js")"#,
        r#"require("package")"#,
        r#"import("./internal.js")"#,
        r#"import("./app/index.js")"#,
        r#"import(".")"#,
        r#"import("path")"#,
        r#"import("package")"#,
        r#"import("@scope/package")"#,
        // Additional: exports (not tested by ESLint)
        r#"export { foo } from "./sibling""#,
        r#"export * from "./another""#,
    ];

    let fail = vec![
        // ESLint test cases
        r#"import foo from "../plugin.js""#,
        r#"import foo from "./../plugin.js""#,
        r#"import foo from "../../api/service""#,
        r#"require("../plugin.js")"#,
        r#"import("../plugin.js")"#,
        r#"import("../../api/service")"#,
        // Additional:
        r#"import foo from "./..""#,
        r#"require("./..")"#,
        r#"import("./..")"#,
        r#"export { foo } from "../parent""#,
        r#"export * from "../parent""#,
        r#"export { foo } from "./..""#,
        r#"export * from "./..""#,
    ];

    Tester::new(NoRelativeParentImports::NAME, NoRelativeParentImports::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
