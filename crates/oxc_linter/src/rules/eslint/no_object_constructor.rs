use oxc_ast::ast::Expression;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    AstNode,
};

fn no_object_constructor_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Disallow calls to the `Object` constructor without an argument")
        .with_help("Use object literal notation {} instead")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoObjectConstructor;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow calls to the Object constructor without an argument
    ///
    /// ### Why is this bad?
    ///
    /// Use of the Object constructor to construct a new empty object is generally discouraged in favor of object literal notation because of conciseness and because the Object global may be redefined. The exception is when the Object constructor is used to intentionally wrap a specified value which is passed as an argument.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// Object();
    /// new Object();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// Object("foo");
    /// const obj = { a: 1, b: 2 };
    /// const isObject = value => value === Object(value);
    /// const createObject = Object => new Object();
    /// ```
    NoObjectConstructor,
    pedantic,
    pending
);

impl Rule for NoObjectConstructor {

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (span, callee, arguments, type_parameters) = match node.kind() {
            AstKind::CallExpression(call_expr) => (
                call_expr.span,
                &call_expr.callee,
                &call_expr.arguments,
                &call_expr.type_parameters,
            ),
            AstKind::NewExpression(new_expr) => (
                new_expr.span,
                &new_expr.callee,
                &new_expr.arguments,
                &new_expr.type_parameters,
            ),
            _ => {
                return;
            }
        };

        let Expression::Identifier(ident) = &callee else {
            return;
        };

        if ident.is_global_reference_name("Object", ctx.symbols())
            && arguments.len() == 0
            && type_parameters.is_none()
        {
            ctx.diagnostic(crate::rules::eslint::no_object_constructor::no_object_constructor_diagnostic(span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "new Object(x)",
        "Object(x)",
        "new globalThis.Object",
        "const createObject = Object => new Object()",
        "var Object; new Object;",
        // Disabled because the eslint-test uses languageOptions: { globals: { Object: "off" } }
        /* "new Object()", */
    ];

    let fail = vec![
        "new Object",
        "Object()",
        "const fn = () => Object();",
        "Object() instanceof Object;",
        "const obj = Object?.();",
        "(new Object() instanceof Object);",
    ];

    Tester::new(NoObjectConstructor::NAME, pass, fail).test_and_snapshot();
}
