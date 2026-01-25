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
            AstKind::ImportDeclaration(import_decl) => {
                if is_parent_import(import_decl.source.value.as_str()) {
                    ctx.diagnostic(no_relative_parent_imports_diagnostic(import_decl.source.span));
                }
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
            AstKind::ExportAllDeclaration(export_decl) => {
                if is_parent_import(export_decl.source.value.as_str()) {
                    ctx.diagnostic(no_relative_parent_imports_diagnostic(export_decl.source.span));
                }
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
                let Expression::Identifier(ident) = &call_expr.callee else {
                    return;
                };

                if ident.name != "require" {
                    return;
                }

                if call_expr.arguments.len() != 1 {
                    return;
                }

                let Argument::StringLiteral(str_literal) = &call_expr.arguments[0] else {
                    return;
                };

                if is_parent_import(str_literal.value.as_str()) {
                    ctx.diagnostic(no_relative_parent_imports_diagnostic(str_literal.span));
                }
            }
            _ => {}
        }
    }
}

/// Check if the import path is a relative parent import (starts with `../` or is `..`)
fn is_parent_import(path: &str) -> bool {
    path == ".." || path.starts_with("../")
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Package imports
        r"import foo from 'lodash'",
        r"import foo from '@scope/package'",
        r"import foo from 'foo/bar'",
        // Sibling imports
        r"import foo from './foo'",
        r"import foo from './foo/bar'",
        // Current directory
        r"import foo from '.'",
        r"import foo from './'",
        // Require - packages
        r"var foo = require('lodash')",
        r"var foo = require('@scope/package')",
        // Require - siblings
        r"var foo = require('./foo')",
        r"var foo = require('./foo/bar')",
        // Export - siblings
        r"export { foo } from './foo'",
        r"export * from './bar'",
        // Absolute paths (handled by no-absolute-path rule, not this one)
        r"import foo from '/absolute/path'",
        // Dynamic imports - siblings OK
        r"import('./foo')",
        r"import('./sub/bar')",
    ];

    let fail = vec![
        // Basic parent imports
        r"import foo from '../foo'",
        r"import foo from '../../foo'",
        r"import foo from '../../../foo'",
        // Parent with subdirectory
        r"import foo from '../foo/bar'",
        r"import foo from '../../foo/bar/baz'",
        // Parent index
        r"import foo from '..'",
        r"import foo from '../'",
        // Require parent
        r"var foo = require('../foo')",
        r"var foo = require('../../foo')",
        r"var foo = require('../foo/bar')",
        // Export from parent
        r"export { foo } from '../foo'",
        r"export { foo } from '../../bar'",
        r"export * from '../baz'",
        r"export * from '../../qux'",
        // Dynamic imports from parent
        r"import('../foo')",
        r"import('../../bar')",
    ];

    Tester::new(NoRelativeParentImports::NAME, NoRelativeParentImports::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
