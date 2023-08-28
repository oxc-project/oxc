use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{ast_util::get_node_by_ident, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(no-var-requires): Require statement not part of import statement.")]
#[diagnostic(severity(warning), help("Use ES6 style imports or import instead."))]
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
    /// ```typescript
    /// var foo = require('foo');
    /// const foo = require('foo');
    /// let foo = require('foo');
    /// ```
    NoVarRequires,
    restriction
);

impl Rule for NoVarRequires {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !ctx.source_type().is_typescript() {
            return;
        }
        let AstKind::CallExpression(expr) = node.kind() else { return };

        if expr.is_require_call() && no_local_require_declaration(&expr.callee, ctx) {
            // If the parent is an expression statement => this is a top level require()
            // Or, if the parent is a chain expression (require?.()) and
            // the grandparent is an expression statement => this is a top level require()
            let is_expression_statement = {
                let parent_node = ctx.nodes().parent_node(node.id());
                let grandparent_node = parent_node.and_then(|x| ctx.nodes().parent_node(x.id()));
                matches!(
                    (
                        parent_node.map(oxc_semantic::AstNode::kind),
                        grandparent_node.map(oxc_semantic::AstNode::kind)
                    ),
                    (Some(AstKind::ExpressionStatement(_)), _)
                        | (
                            Some(AstKind::ChainExpression(_)),
                            Some(AstKind::ExpressionStatement(_))
                        )
                )
            };

            // If this is an expression statement, it means the `require()`'s return value is unused.
            // If the return value is unused, this isn't a problem.
            if !is_expression_statement {
                ctx.diagnostic(NoVarRequiresDiagnostic(node.kind().span()));
            }
        }
    }
}

fn no_local_require_declaration(expr: &Expression, ctx: &LintContext) -> bool {
    let Expression::Identifier(ident) = expr else { return true };
    get_node_by_ident(ident, ctx).is_none()
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
        "
            let require = () => 'foo'; 
            {
                let foo = require('foo');
            }
        ",
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
        // Because of TypeScript disallows angle bracket type assertions in .tsx files, comment out this below case all tests parsing as tsx.
        // "const foo = <Foo>require('./foo.json');",
        "const foo: Foo = require('./foo.json').default;",
        r#"
            const configValidator = new Validator(require('./a.json'));
            configValidator.addSchema(require('./a.json'));
        "#,
    ];

    Tester::new_without_config(NoVarRequires::NAME, pass, fail).test_and_snapshot();
}
