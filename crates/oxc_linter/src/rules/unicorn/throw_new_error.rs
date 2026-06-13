use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn throw_new_error_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Require `new` when throwing an error.")
        .with_help("Using `new` ensures the error is correctly initialized.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ThrowNewError;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule makes sure you always use `new` when throwing an error.
    ///
    /// ### Why is this bad?
    ///
    /// In JavaScript, omitting `new` (e.g., `throw Error('message')`) is allowed,
    /// but it does not properly initialize the error object. This can lead to missing
    /// stack traces or incorrect prototype chains. Using `new` makes the intent clear,
    /// ensures consistent behavior, and helps avoid subtle bugs.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// throw Error('🦄');
    /// throw TypeError('unicorn');
    /// throw lib.TypeError('unicorn');
    /// const e = Error('message');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// throw new Error('🦄');
    /// throw new TypeError('unicorn');
    /// throw new lib.TypeError('unicorn');
    /// const e = new Error('message');
    /// ```
    ThrowNewError,
    unicorn,
    style,
    fix,
    version = "0.0.14",
);

impl Rule for ThrowNewError {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        // Skip calls that are the superClass of a Class (extends clause).
        // `new` is not appropriate there, e.g. `class Foo extends mixin() {}`.
        if matches!(ctx.nodes().parent_kind(node.id()), AstKind::Class(_)) {
            return;
        }

        let name = match call_expr.callee.without_parentheses() {
            Expression::Identifier(v) => v.name,
            Expression::StaticMemberExpression(v) => v.property.name,
            _ => return,
        };

        if name.len() >= 5 && name.as_bytes()[0].is_ascii_uppercase() && name.ends_with("Error") {
            ctx.diagnostic_with_fix(throw_new_error_diagnostic(call_expr.span), |fixer| {
                fixer.insert_text_before_range(call_expr.span, "new ")
            });
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "throw new Error()",
        "new Error()",
        "throw new TypeError()",
        "throw new EvalError()",
        "throw new RangeError()",
        "throw new ReferenceError()",
        "throw new SyntaxError()",
        "throw new URIError()",
        "throw new CustomError()",
        "throw new FooBarBazError()",
        "throw new ABCError()",
        "throw getError()",
        "throw CustomError",
        "throw getErrorConstructor()()",
        "throw lib[Error]()",
        r#"throw lib["Error"]()"#,
        "throw lib.getError()",
        "class QueryError extends Data.TaggedError('QueryError') {}",
    ];

    let fail = vec![
        "throw Error()",
        "throw (Error)()",
        "throw lib.Error()",
        "throw lib.mod.Error()",
        "throw lib[mod].Error()",
        "throw (lib.mod).Error()",
        "throw Error('foo')",
        "throw CustomError('foo')",
        "throw FooBarBazError('foo')",
        "throw ABCError('foo')",
        "throw Abc3Error('foo')",
        "throw TypeError()",
        "throw EvalError()",
        "throw RangeError()",
        "throw ReferenceError()",
        "throw SyntaxError()",
        "throw URIError()",
        "throw (( URIError() ))",
        "throw (( URIError ))()",
        "throw getGlobalThis().Error()",
        "throw utils.getGlobalThis().Error()",
        "throw (( getGlobalThis().Error ))()",
        "const error = Error()",
        "throw Object.assign(Error(), {foo})",
        "new Promise((resolve, reject) => {\n        reject(Error('message'));\n    });",
        "function foo() {\n        return [globalThis][0].Error('message');\n    }",
    ];

    let fix = vec![
        ("throw Error()", "throw new Error()"),
        ("throw (( getGlobalThis().Error ))()", "throw new (( getGlobalThis().Error ))()"),
        ("const error = Error()", "const error = new Error()"),
        ("throw Object.assign(Error(), {foo})", "throw Object.assign(new Error(), {foo})"),
        (
            "new Promise((resolve, reject) => {\n        reject(Error('message'));\n    });",
            "new Promise((resolve, reject) => {\n        reject(new Error('message'));\n    });",
        ),
        (
            "function foo() {\n        return [globalThis][0].Error('message');\n    }",
            "function foo() {\n        return new [globalThis][0].Error('message');\n    }",
        ),
    ];

    Tester::new(ThrowNewError::NAME, ThrowNewError::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
