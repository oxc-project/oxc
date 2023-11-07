use lazy_static::lazy_static;
use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};

use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::Regex;

use crate::{
    ast_util::{outermost_paren, outermost_paren_parent},
    context::LintContext,
    rule::Rule,
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(throw-new-error): Require `new` when throwing an error.")]
#[diagnostic(severity(warning), help("While it's possible to create a new error without using the `new` keyword, it's better to be explicit."))]
struct ThrowNewErrorDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct ThrowNewError;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require `new` when throwing an error.`
    ///
    /// ### Why is this bad?
    ///
    /// While it's possible to create a new error without using the `new` keyword, it's better to be explicit.
    ///
    /// ### Example
    /// ```javascript
    /// // Fail
    /// throw Error('🦄');
    /// throw TypeError('unicorn');
    /// throw lib.TypeError('unicorn');
    ///
    /// // Pass
    /// throw new Error('🦄');
    /// throw new TypeError('unicorn');
    /// throw new lib.TypeError('unicorn');
    ///
    /// ```
    ThrowNewError,
    style
);

impl Rule for ThrowNewError {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        let Some(outermost_paren_node) = outermost_paren_parent(node, ctx) else { return };

        let AstKind::ThrowStatement(_) = outermost_paren(outermost_paren_node, ctx).kind() else {
            return;
        };

        match &call_expr.callee.without_parenthesized() {
            Expression::Identifier(v) => {
                if !CUSTOM_ERROR_REGEX_PATTERN.is_match(&v.name) {
                    return;
                }
            }
            Expression::MemberExpression(v) => {
                if v.is_computed() {
                    return;
                }
                if let Some(v) = v.static_property_name() {
                    if !CUSTOM_ERROR_REGEX_PATTERN.is_match(v) {
                        return;
                    }
                }
            }
            _ => return,
        }

        ctx.diagnostic(ThrowNewErrorDiagnostic(call_expr.span));
    }
}

lazy_static! {
    static ref CUSTOM_ERROR_REGEX_PATTERN: Regex =
        Regex::new(r"^(?:[A-Z][\da-z]*)*Error$").unwrap();
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("throw new Error()", None),
        ("new Error()", None),
        ("throw new TypeError()", None),
        ("throw new EvalError()", None),
        ("throw new RangeError()", None),
        ("throw new ReferenceError()", None),
        ("throw new SyntaxError()", None),
        ("throw new URIError()", None),
        ("throw new CustomError()", None),
        ("throw new FooBarBazError()", None),
        ("throw new ABCError()", None),
        ("throw getError()", None),
        ("throw CustomError", None),
        ("throw getErrorConstructor()()", None),
        ("throw lib[Error]()", None),
        ("throw lib[\"Error\"]()", None),
        ("throw lib.getError()", None),
    ];

    let fail = vec![
        ("throw Error()", None),
        ("throw (Error)()", None),
        ("throw lib.Error()", None),
        ("throw lib.mod.Error()", None),
        ("throw lib[mod].Error()", None),
        ("throw (lib.mod).Error()", None),
        ("throw Error('foo')", None),
        ("throw CustomError('foo')", None),
        ("throw FooBarBazError('foo')", None),
        ("throw ABCError('foo')", None),
        ("throw Abc3Error('foo')", None),
        ("throw TypeError()", None),
        ("throw EvalError()", None),
        ("throw RangeError()", None),
        ("throw ReferenceError()", None),
        ("throw SyntaxError()", None),
        ("throw URIError()", None),
        ("throw (( URIError() ))", None),
        ("throw (( URIError ))()", None),
        ("throw getGlobalThis().Error()", None),
        ("throw utils.getGlobalThis().Error()", None),
        ("throw (( getGlobalThis().Error ))()", None),
    ];

    Tester::new(ThrowNewError::NAME, pass, fail).test_and_snapshot();
}
