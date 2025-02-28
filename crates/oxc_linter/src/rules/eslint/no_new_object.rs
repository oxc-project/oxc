use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_new_object_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Calling Object constructors with new is disallowed.")
        .with_help("The object literal notation {} is preferable.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNewObject;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows using `new Object()` to create objects and encourages using
    /// the object literal `{}` instead.
    ///
    /// ### Why is this bad?
    ///
    /// Using `new Object()` is longer and less clear than `{}`. While both approaches
    /// work the same way and have no performance difference, the object literal is more
    /// concise, easier to read, and aligns with standard JavaScript practices.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = new Object()
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = {};
    ///
    /// var Object = function Object() {};
    /// new Object();
    ///
    /// class Object {
    ///     constructor(){}
    /// }
    /// new Object();
    ///
    /// import { Object } from './'
    /// new Object();
    /// ```
    NoNewObject,
    eslint,
    style,
    pending
);

impl Rule for NoNewObject {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        let Expression::Identifier(ident) = new_expr.callee.get_inner_expression() else {
            return;
        };

        // If `Object` refers to a custom identifier defined in the source code then the use of the `new`
        // constructor is allowed.
        if ident.name == "Object" && ctx.semantic().is_reference_to_global_variable(ident) {
            ctx.diagnostic(no_new_object_diagnostic(new_expr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var myObject = {};",
        "var myObject = new CustomObject();",
        "var foo = new foo.Object()",
        "var Object = function Object() {};
		new Object();",
        "var x = something ? MyClass : Object;
		var y = new x();",
        "
		class Object {
			constructor(){}
		}
		new Object();", // { "ecmaVersion": 6 },
        "import { Object } from './'
		new Object();", // { "ecmaVersion": 6, "sourceType": "module" }
    ];

    let fail = vec![
        "var foo = new Object()",
        "new Object();",
        "const a = new Object()", // { "ecmaVersion": 6 }
    ];

    Tester::new(NoNewObject::NAME, NoNewObject::PLUGIN, pass, fail).test_and_snapshot();
}
