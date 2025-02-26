use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn no_new_object_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Calling Object constructors with new is disallowed.")
        .with_help("The object literal notation {} is preferable.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNewObject;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows calling the Object constructor with new
    ///
    /// ### Why is this bad?
    ///
    /// The Object constructor is used to create new generic objects in JavaScript, such as:
    ///
    /// ```javascript
    /// var myObject = new Object();
    /// ```
    ///
    /// However, this is no different from using the more concise object literal syntax:
    ///
    /// ```javascript
    /// var myObject = {};
    /// ```
    ///
    /// For this reason, many prefer to always use the object literal syntax and never use the
    /// Object constructor.
    ///
    /// While there are no performance differences between the two approaches, the byte savings and
    /// conciseness of the object literal form is what has made it the de facto way of creating new
    /// objects.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var foo = new Object()
    /// ```
    ///
    /// ```javascript
    /// new Object()
    /// ```
    ///
    /// ```javascript
    /// const a = new Object()
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// var myObject = {};
    /// ```
    ///
    /// ```javascript
    /// var Object = function Object() {};
    /// new Object();
    /// ```
    ///
    /// ```javascript
    /// class Object {
    ///     constructor(){}
    /// }
    /// new Object();
    /// ```
    ///
    /// ```javascript
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

        let Expression::Identifier(ident) = &new_expr.callee else {
            return;
        };

        // If `Object` refers to a custom identifier defined in the source code then the use of the `new`
        // constructor is allowed.
        let is_custom_object_id: bool = !ident.is_global_reference_name("Object", ctx.symbols());

        if is_custom_object_id {
            return;
        } else if ident.name != "Object" {
            return;
        }

        ctx.diagnostic(no_new_object_diagnostic(new_expr.span));
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
			            constructor(){
			
			            }
			        }
			        new Object();
			        ", // { "ecmaVersion": 6 },
        "
			        import { Object } from './'
			        new Object();
			        ", // { "ecmaVersion": 6, "sourceType": "module" }
    ];

    let fail = vec![
        "var foo = new Object()",
        "new Object();",
        "const a = new Object()", // { "ecmaVersion": 6 }
    ];

    Tester::new(NoNewObject::NAME, NoNewObject::PLUGIN, pass, fail).test_and_snapshot();
}
