use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "typescript-eslint(no-var-requires): Require statement not part of import statement."
)]
#[diagnostic(severity(error))]
struct NoVarRequiresDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoVarRequires;

declare_oxc_lint!(
    /// ### What it does
    /// 
    /// Disallow `require` statements except in import statements
    /// 
    /// ### Why is this bad?
    /// 
    /// In other words, the use of forms such as var foo = require("foo") are banned. Instead use ES6 style imports or import foo = require("foo") imports.
    /// 
    /// ```javascript
    /// var foo = require('foo');
    /// const foo = require('foo');
    /// let foo = require('foo');
    /// ```
    NoVarRequires,
    correctness
);

impl Rule for NoVarRequires {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(expr) = node.kind() && expr.is_require_call() {
            if ctx.scopes().get_bindings(node.scope_id()).contains_key("require") {
                return;
            } 

            if let Some(parent_node) = ctx.nodes().parent_node(node.id()) {
                if let AstKind::Argument(_) = parent_node.kind() {
                    if let Some(parent_node) = ctx.nodes().parent_node(parent_node.id()) {
                        if is_target_node(&parent_node.kind()) {
                            ctx.diagnostic(NoVarRequiresDiagnostic(expr.span));
                        }
                    }
                }

                if is_target_node(&parent_node.kind()) {
                    ctx.diagnostic(NoVarRequiresDiagnostic(expr.span));
                }
            }
        }
    }
}

fn is_target_node(node_kind: &AstKind<'_>) -> bool {
    matches!(node_kind, AstKind::CallExpression(_)
        | AstKind::MemberExpression(_)
        | AstKind::NewExpression(_)
        | AstKind::TSAsExpression(_)
        | AstKind::TSTypeAssertion(_)
        | AstKind::VariableDeclarator(_))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "import foo = require('foo');",
        "require('foo');",
        "require?.('foo');",
        r#"
            import { createRequire } from 'module';
            const require = createRequire('foo');
            const json = require('./some.json');
        "#,
    ];

    let fail = vec![
        "var foo = require('foo');",
        "const foo = require('foo');",
        "let foo = require('foo');",
        "let foo = trick(require('foo'));",
        "var foo = require?.('foo');",
        "const foo = require?.('foo');",
        "let foo = require?.('foo');",
        "let foo = trick(require?.('foo'));",
        "let foo = trick?.(require('foo'));",
        "const foo = require('./foo.json') as Foo;",
        // "const foo = <Foo>require('./foo.json');",
        "const foo: Foo = require('./foo.json').default;",
        r#"
            const configValidator = new Validator(require('./a.json'));
            configValidator.addSchema(require('./a.json'));
        "#
    ];

    Tester::new_without_config(NoVarRequires::NAME, pass, fail).test_and_snapshot();
}
