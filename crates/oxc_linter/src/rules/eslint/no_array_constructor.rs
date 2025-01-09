use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_array_constructor_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `new` to create arrays")
        .with_help("Use an array literal instead")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoArrayConstructor;

declare_oxc_lint!(
    /// ### What it does
    /// Disallows creating arrays with the `Array` constructor.
    ///
    /// ### Why is this bad?
    ///
    /// Use of the `Array` constructor to construct a new array is generally
    /// discouraged in favor of array literal notation because of the
    /// single-argument pitfall and because the `Array` global may be redefined.
    /// The exception is when the `Array` constructor is used to intentionally
    /// create sparse arrays of a specified size by giving the constructor a
    /// single numeric argument.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// let arr = new Array();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// let arr = [];
    /// let arr2 = Array.from(iterable);
    /// let arr3 = new Array(9);
    /// ```
    NoArrayConstructor,
    eslint,
    pedantic,
    pending
);

impl Rule for NoArrayConstructor {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (span, callee, arguments, type_parameters, optional) = match node.kind() {
            AstKind::CallExpression(call_expr) => (
                call_expr.span,
                &call_expr.callee,
                &call_expr.arguments,
                &call_expr.type_parameters,
                call_expr.optional,
            ),
            AstKind::NewExpression(new_expr) => (
                new_expr.span,
                &new_expr.callee,
                &new_expr.arguments,
                &new_expr.type_parameters,
                false,
            ),
            _ => {
                return;
            }
        };

        let Expression::Identifier(ident) = &callee else {
            return;
        };

        if ident.is_global_reference_name("Array", ctx.symbols())
            && arguments.len() != 1
            && type_parameters.is_none()
            && !optional
        {
            ctx.diagnostic(no_array_constructor_diagnostic(span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("new Array(x)", None),
        ("Array(x)", None),
        ("new Array(9)", None),
        ("Array(9)", None),
        ("new foo.Array()", None),
        ("foo.Array()", None),
        ("new Array.foo", None),
        ("new Array.foo();", None),
        ("Array.foo", None),
        ("Array.foo()", None),
        ("new Array<Foo>(1, 2, 3);", None),
        ("new Array<Foo>();", None),
        ("Array<Foo>(1, 2, 3);", None),
        ("Array<Foo>();", None),
        ("Array?.(x);", None),
        ("Array?.(9);", None),
        ("foo?.Array();", None),
        ("Array?.foo();", None),
        ("foo.Array?.();", None),
        ("Array.foo?.();", None),
        ("Array?.<Foo>(1, 2, 3);", None),
        ("Array?.<Foo>();", None),
        ("Array?.(0, 1, 2);", None),
        ("Array?.(x, y);", None),
        ("var Array; new Array;", None),
    ];

    let fail = vec![
        ("new Array()", None),
        ("new Array", None),
        ("Array();", None),
        ("new Array(x, y)", None),
        ("new Array(0, 1, 2)", None),
        ("Array(x, y)", None),
        ("Array(0, 1, 2)", None),
    ];

    Tester::new(NoArrayConstructor::NAME, NoArrayConstructor::PLUGIN, pass, fail)
        .test_and_snapshot();
}
