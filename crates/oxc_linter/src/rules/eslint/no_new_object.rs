use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
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
    ///  This rule disallows calling the Object constructor with new
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
    /// For this reason, many prefer to always use the object literal syntax and never use the Object constructor.
    ///
    /// While there are no performance differences between the two approaches, the byte savings and conciseness of the object literal form is what has made it the de facto way of creating new objects.
    ///
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoNewObject,
    eslint,
    style,
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for NoNewObject {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
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
