use oxc_ast::{
    AstKind,
    ast::{Expression, IdentifierReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn global_require_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected require().")
        .with_label(span)
        .with_help("Move require() to top-level module scope")
}

#[derive(Debug, Default, Clone)]
pub struct GlobalRequire;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require `require()` calls to be placed at top-level module scope
    ///
    /// ### Why is this bad?
    ///
    /// In Node.js, module dependencies are included using the `require()` function, such as:
    /// ```js
    /// var fs = require("fs");
    /// ```
    ///
    /// While `require()` may be called anywhere in code, some style guides prescribe that it should be called only in the top level of a module to make it easier to identify dependencies.
    /// For instance, it's arguably harder to identify dependencies when they are deeply nested inside of functions and other statements:
    /// ```js
    /// function foo() {
    ///    if (condition) {
    ///        var fs = require("fs");
    ///    }
    ///}
    /// ```
    ///
    /// Since `require()` does a synchronous load, it can cause performance problems when used in other locations.
    /// Further, ES6 modules mandate that import and export statements can only occur in the top level of the module's body.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // calling require() inside of a function is not allowed
    /// function readFile(filename, callback) {
    ///     var fs = require('fs');
    ///     fs.readFile(filename, callback)
    /// }
    ///
    /// // conditional requires like this are also not allowed
    /// if (DEBUG) { require('debug'); }
    ///
    /// // a require() in a switch statement is also flagged
    /// switch(x) { case '1': require('1'); break; }
    ///
    /// // you may not require() inside an arrow function body
    /// var getModule = (name) => require(name);
    ///
    /// // you may not require() inside of a function body as well
    /// function getModule(name) { return require(name); }
    ///
    /// // you may not require() inside of a try/catch block
    /// try {
    ///     require(unsafeModule);
    /// } catch(e) {
    ///     console.log(e);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // all these variations of require() are ok
    /// require('x');
    /// var y = require('y');
    /// var z;
    /// z = require('z').initialize();
    ///
    /// // requiring a module and using it in a function is ok
    /// var fs = require('fs');
    /// function readFile(filename, callback) {
    ///     fs.readFile(filename, callback)
    /// }
    ///
    /// // you can use a ternary to determine which module to require
    /// var logger = DEBUG ? require('dev-logger') : require('logger');
    ///
    /// // if you want you can require() at the end of your module
    /// function doSomethingA() {}
    /// function doSomethingB() {}
    /// var x = require("x"),
    ///     z = require("z");
    /// ```
    GlobalRequire,
    node,
    style,
);

impl Rule for GlobalRequire {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Expression::Identifier(ident) = &call_expr.callee else {
            return;
        };

        if ident.name != "require" {
            return;
        }

        if is_shadowed(ident, ctx) {
            return;
        }

        let is_good_require =
            ctx.nodes().ancestors(node.id()).all(|ancestor| is_acceptable_parent(&ancestor.kind()));

        if !is_good_require {
            ctx.diagnostic(global_require_diagnostic(call_expr.span));
        }
    }
}

fn is_acceptable_parent(kind: &AstKind) -> bool {
    matches!(
        kind,
        AstKind::AssignmentExpression(_)
            | AstKind::VariableDeclarator(_)
            | AstKind::StaticMemberExpression(_)
            | AstKind::ComputedMemberExpression(_)
            | AstKind::PrivateFieldExpression(_)
            | AstKind::ExpressionStatement(_)
            | AstKind::CallExpression(_)
            | AstKind::ConditionalExpression(_)
            | AstKind::Program(_)
            | AstKind::VariableDeclaration(_)
    )
}

fn is_shadowed(ident: &IdentifierReference, ctx: &LintContext) -> bool {
    let reference = ctx.scoping().get_reference(ident.reference_id());
    reference.symbol_id().is_some()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var x = require('y');",
        "if (x) { x.require('y'); }",
        "var x;
			x = require('y');",
        "var x = 1, y = require('y');",
        "var x = require('y'), y = require('y'), z = require('z');",
        "var x = require('y').foo;",
        "require('y').foo();",
        "require('y');",
        "function x(){}


			x();


			if (x > y) {
				doSomething()

			}

			var x = require('y').foo;",
        "var logger = require(DEBUG ? 'dev-logger' : 'logger');",
        "var logger = DEBUG ? require('dev-logger') : require('logger');",
        "function localScopedRequire(require) { require('y'); }",
        "var someFunc = require('./someFunc'); someFunc(function(require) { return('bananas'); });",
        "function outer() { function require() {} require('y'); }",
        "function foo(require) { function bar() { require('y'); } }",
    ];

    let fail = vec![
        "if (process.env.NODE_ENV === 'DEVELOPMENT') {
				require('debug');
			}",
        "var x; if (y) { x = require('debug'); }",
        "var x; if (y) { x = require('debug').baz; }",
        "function x() { require('y') }",
        "try { require('x'); } catch (e) { console.log(e); }",
        "var getModule = x => require(x);",
        "var x = (x => require(x))('weird')",
        "switch(x) { case '1': require('1'); break; }",
        "var obj = { get x() { return require('y'); } };",
        "class Foo { static { require('y'); } }",
    ];

    Tester::new(GlobalRequire::NAME, GlobalRequire::PLUGIN, pass, fail).test_and_snapshot();
}
