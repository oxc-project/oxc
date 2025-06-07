use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::BinaryOperator;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_instanceof_array_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `Array.isArray()` instead of `instanceof Array`.")
        .with_help("The instanceof Array check doesn't work across realms/contexts, for example, frames/windows in browsers or the vm module in Node.js.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoInstanceofArray;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require `Array.isArray()` instead of `instanceof Array`.
    ///
    /// ### Why is this bad?
    ///
    /// The instanceof Array check doesn't work across realms/contexts, for example, frames/windows in browsers or the vm module in Node.js.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// array instanceof Array;
    /// [1,2,3] instanceof Array;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// Array.isArray(array);
    /// Array.isArray([1,2,3]);
    /// ```
    NoInstanceofArray,
    unicorn,
    pedantic,
    fix
);

impl Rule for NoInstanceofArray {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(expr) = node.kind() else {
            return;
        };
        if expr.operator != BinaryOperator::Instanceof {
            return;
        }

        match &expr.right.without_parentheses() {
            Expression::Identifier(identifier) if identifier.name == "Array" => {
                ctx.diagnostic_with_fix(no_instanceof_array_diagnostic(expr.span), |fixer| {
                    let argument = fixer.source_range(expr.left.span());
                    let mut code = String::with_capacity(15 + argument.len());
                    code.push_str("Array.isArray(");
                    code.push_str(argument);
                    code.push(')');
                    fixer.replace(expr.span, code)
                });
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("Array.isArray(arr)", None),
        ("arr instanceof Object", None),
        ("arr instanceof array", None),
        ("a instanceof 'array'", None),
        ("a instanceof ArrayA", None),
        ("a.x[2] instanceof foo()", None),
        ("Array.isArray([1,2,3]) === true", None),
        ("\"arr instanceof Array\"", None),
    ];

    let fail = vec![
        ("arr instanceof Array", None),
        ("[] instanceof Array", None),
        ("[] instanceof (Array)", None),
        ("[1,2,3] instanceof Array === true", None),
        ("fun.call(1, 2, 3) instanceof Array", None),
        ("obj.arr instanceof Array", None),
        ("foo.bar[2] instanceof Array", None),
        ("(0, array) instanceof Array", None),
        ("function foo(){return [] instanceof Array}", None),
    ];

    let fix = vec![
        ("arr instanceof Array", "Array.isArray(arr)", None),
        ("[] instanceof Array", "Array.isArray([])", None),
        ("[1,2,3] instanceof Array === true", "Array.isArray([1,2,3]) === true", None),
        ("fun.call(1, 2, 3) instanceof Array", "Array.isArray(fun.call(1, 2, 3))", None),
        ("obj.arr instanceof Array", "Array.isArray(obj.arr)", None),
        ("foo.bar[2] instanceof Array", "Array.isArray(foo.bar[2])", None),
        ("(0, array) instanceof Array", "Array.isArray((0, array))", None),
        (
            "function foo(){return [] instanceof Array}",
            "function foo(){return Array.isArray([])}",
            None,
        ),
    ];

    Tester::new(NoInstanceofArray::NAME, NoInstanceofArray::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
