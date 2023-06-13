use oxc_ast::ast::Expression::CallExpression;
use oxc_ast::ast::ModuleDeclaration;
use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-self-import): module importing itself is not allowed")]
#[diagnostic(severity(warning))]
struct NoSelfImportDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoSelfImport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbid a module from importing itself. This can sometimes happen during refactoring.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // foo.js
    /// import foo from './foo.js'
    /// const foo = require('./foo')
    /// ```
    NoSelfImport,
    correctness
);

impl Rule for NoSelfImport {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if ctx.source_type().is_module() {
            let AstKind::ModuleDeclaration(module_decl) = node.get().kind() else { return; };

            if let ModuleDeclaration::ImportDeclaration(_import_decl) = module_decl {
              // TODO: Compare import_decl.source
            }
        } else {
            let AstKind::VariableDeclaration(declaration) = node.get().kind() else { return; };
            let Some(declarator) = declaration.declarations.last() else { return; };

            if let Some(init) = declarator.init.as_ref() {
                if let CallExpression(expr) = init {
                    if expr.callee.is_require() {
                        ctx.diagnostic(NoSelfImportDiagnostic(expr.span));
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("import bar from './bar';", None), // foo.js
        ("const bar = require('./bar');", None), // foo.js
    ];

    let fail = vec![
        ("import foo from './foo';", None), // foo.js
        ("const foo = require('./foo');", None), // foo.js
        ("import index from '.';", None), // index.js
        ("const index = require('.');", None), // index.js
    ];

    Tester::new(NoSelfImport::NAME, pass, fail).test_and_snapshot();
}
