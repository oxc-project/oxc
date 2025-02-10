use oxc_ast::{
    ast::{match_member_expression, CallExpression, Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{context::LintContext, rule::Rule, AstNode};

fn prefer_type_error_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Prefer throwing a `TypeError` over a generic `Error` after a type checking if-statement",
    )
    .with_help("Change to `throw new TypeError(...)`")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferTypeError;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce throwing a `TypeError` instead of a generic `Error` after a type checking if-statement.
    ///
    /// ### Why is this bad?
    ///
    /// Throwing a `TypeError` instead of a generic `Error` after a type checking if-statement is more specific and helps to catch bugs.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (Array.isArray(foo)) {
    ///     throw new Error('Expected foo to be an array');
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (Array.isArray(foo)) {
    ///     throw new TypeError('Expected foo to be an array');
    /// }
    /// ```
    PreferTypeError,
    unicorn,
    pedantic,
    fix
);

impl Rule for PreferTypeError {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ThrowStatement(throw_stmt) = node.kind() else {
            return;
        };

        let Expression::NewExpression(new_expr) = &throw_stmt.argument.without_parentheses() else {
            return;
        };

        if !new_expr.callee.is_specific_id("Error") {
            return;
        }

        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        let AstKind::BlockStatement(block_stmt) = parent.kind() else {
            return;
        };

        if block_stmt.body.len() != 1 {
            return;
        }

        let Some(parent) = ctx.nodes().parent_node(parent.id()) else {
            return;
        };

        let AstKind::IfStatement(if_stmt) = parent.kind() else {
            return;
        };

        if is_type_checking_expr(&if_stmt.test) {
            ctx.diagnostic_with_fix(
                prefer_type_error_diagnostic(new_expr.callee.span()),
                |fixer| fixer.replace(new_expr.callee.span(), "TypeError"),
            );
        }
    }
}

fn is_type_checking_expr(expr: &Expression) -> bool {
    match expr {
        match_member_expression!(Expression) => {
            is_type_checking_member_expr(expr.to_member_expression())
        }
        Expression::CallExpression(call_expr) => is_typechecking_call_expr(call_expr),
        Expression::UnaryExpression(unary_expr) => {
            if unary_expr.operator == UnaryOperator::Typeof {
                return true;
            }

            if unary_expr.operator == UnaryOperator::LogicalNot {
                return is_type_checking_expr(&unary_expr.argument);
            }
            false
        }
        Expression::BinaryExpression(bin_expr) => {
            if bin_expr.operator == BinaryOperator::Instanceof {
                return true;
            }
            is_type_checking_expr(&bin_expr.left) || is_type_checking_expr(&bin_expr.right)
        }
        Expression::LogicalExpression(logical_expr) => {
            is_type_checking_expr(&logical_expr.left) && is_type_checking_expr(&logical_expr.right)
        }
        _ => false,
    }
}

fn is_typechecking_call_expr(call_expr: &CallExpression) -> bool {
    if call_expr.arguments.len() == 0 {
        return false;
    }

    match &call_expr.callee {
        Expression::Identifier(ident) => {
            TYPE_CHECKING_GLOBAL_IDENTIFIERS.contains(ident.name.as_str())
        }
        callee @ match_member_expression!(Expression) => {
            if let Some(ident) = callee.to_member_expression().static_property_name() {
                return TYPE_CHECKING_IDENTIFIERS.contains(ident);
            }
            false
        }
        _ => false,
    }
}

fn is_type_checking_member_expr(member_expr: &MemberExpression) -> bool {
    if let Some(ident) = member_expr.static_property_name() {
        return TYPE_CHECKING_IDENTIFIERS.contains(ident);
    }

    false
}

const TYPE_CHECKING_IDENTIFIERS: phf::Set<&'static str> = phf::phf_set!(
    "isArray",
    "isArrayBuffer",
    "isArrayLike",
    "isArrayLikeObject",
    "isBigInt",
    "isBoolean",
    "isBuffer",
    "isDate",
    "isElement",
    "isError",
    "isFinite",
    "isFunction",
    "isInteger",
    "isLength",
    "isMap",
    "isNaN",
    "isNative",
    "isNil",
    "isNull",
    "isNumber",
    "isObject",
    "isObjectLike",
    "isPlainObject",
    "isPrototypeOf",
    "isRegExp",
    "isSafeInteger",
    "isSet",
    "isString",
    "isSymbol",
    "isTypedArray",
    "isUndefined",
    "isView",
    "isWeakMap",
    "isWeakSet",
    "isWindow",
    "isXMLDoc",
);

const TYPE_CHECKING_GLOBAL_IDENTIFIERS: phf::Set<&'static str> =
    phf::phf_set!("isFinite", "isNaN",);

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"
            if (MrFuManchu.name !== 'Fu Manchu' || MrFuManchu.isMale === false) {
                throw new Error('How cant Fu Manchu be Fu Manchu?');
            }
        ",
        r"
            if (Array.isArray(foo) || ArrayBuffer.isView(foo)) {
                throw new TypeError();
            }
        ",
        r"
            if (wrapper.g.ary.isArray(foo) || wrapper.f.g.ary.isView(foo)) {
                throw new TypeError();
            }
        ",
        r"
            if (wrapper.g.ary(foo) || wrapper.f.g.ary.isPiew(foo)) {
                throw new Error();
            }
        ",
        r"
            if (Array.isArray()) {
                throw new Error('Woohoo - isArray is broken!');
            }
        ",
        r"
            if (Array.isArray(foo) || ArrayBuffer.isView(foo)) {
                throw new CustomError();
            }
        ",
        r"
            if (Array.isArray(foo)) {
                throw new Error.foo();
            }
        ",
        r"
            if (Array.isArray(foo)) {
                throw new Error.foo;
            }
        ",
        r"
            if (Array.isArray(foo)) {
                throw new foo.Error;
            }
        ",
        r"
            if (Array.isArray(foo)) {
                throw new foo.Error('My name is Foo Manchu');
            }
        ",
        r"
            if (Array.isArray(foo) || ArrayBuffer.isView(foo)) {
                throw Error('This is for FooBar', foo);
            }
        ",
        r"
            if (Array.isArray(foo) || ArrayBuffer.isView(foo)) {
                new Error('This is for FooBar', foo);
            }
        ",
        r"
            function test(foo) {
                if (Array.isArray(foo) || ArrayBuffer.isView(foo)) {
                    return new Error('This is for FooBar', foo);
                }
                return foo;
            }
        ",
        r"
            if (Array.isArray(foo) || ArrayBuffer.isView(foo)) {
                lastError = new Error('This is for FooBar', foo);
            }
        ",
        r"
            if (!isFinite(foo)) {
                throw new TypeError();
            }
        ",
        r"
            if (isNaN(foo)) {
                throw new TypeError();
            }
        ",
        r"
            if (isArray(foo)) {
                throw new Error();
            }
        ",
        r"
            if (foo instanceof boo) {
                throw new TypeError();
            }
        ",
        r"
            if (typeof boo === 'Boo') {
                throw new TypeError();
            }
        ",
        r"
            if (typeof boo === 'Boo') {
                some.thing.else.happens.before();
                throw new Error();
            }
        ",
        r"
            if (Number.isNaN(foo)) {
                throw new TypeError();
            }
        ",
        r"
            if (Number.isFinite(foo) && Number.isSafeInteger(foo) && Number.isInteger(foo)) {
                throw new TypeError();
            }
        ",
        r"
            if (Array.isArray(foo) || (Blob.isBlob(foo) || Blip.isBlip(foo))) {
                throw new TypeError();
            }
        ",
        r"
            if (typeof foo === 'object' || (Object.isFrozen(foo) || 'String' === typeof foo)) {
                throw new TypeError();
            }
        ",
        r"
            if (isNaN) {
                throw new Error();
            }
        ",
        r"
            if (isObjectLike) {
                throw new Error();
            }
        ",
        r"
            if (isNaN.foo()) {
                throw new Error();
            }
        ",
        r"
            if (typeof foo !== 'object' || foo.bar() === false) {
                throw new TypeError('Expected Foo being bar!');
            }
        ",
        r"
            if (foo instanceof Foo) {
                throw new TypeError('Expected Foo being bar!');
            }
        ",
        r"
            if (!foo instanceof Foo) {
                throw new TypeError('Expected Foo being bar!');
            }
        ",
        r"
            if (foo instanceof Foo === false) {
                throw new TypeError('Expected Foo being bar!');
            }
        ",
        r"throw new Error('💣')",
        r"
            if (!Number.isNaN(foo) && foo === 10) {
                throw new Error('foo is not 10!');
            }
        ",
        r"
            function foo(foo) {
                if (!Number.isNaN(foo) && foo === 10) {
                    timesFooWas10 += 1;
                    if (calculateAnswerToLife() !== 42) {
                        openIssue('Your program is buggy!');
                    } else {
                        return printAwesomeAnswer(42);
                    }
                    throw new Error('foo is 10');
                }
            }
        ",
        r"
            function foo(foo) {
                if (!Number.isNaN(foo)) {
                    timesFooWas10 += 1;
                    if (calculateAnswerToLife({with: foo}) !== 42) {
                        openIssue('Your program is buggy!');
                    } else {
                        return printAwesomeAnswer(42);
                    }
                    throw new Error('foo is 10');
                }
            }
        ",
        r"
            if (!x.isFudge()) {
                throw new Error('x is no fudge!');
            }
        ",
        r"
            if (!_.isFudge(x)) {
                throw new Error('x is no fudge!');
            }
        ",
        r"
            switch (something) {
                case 1:
                    break;
                default:
                    throw new Error('Unknown');
            }
        ",
    ];

    let fail = vec![
        r"
            if (!isFinite(foo)) {
                throw new Error();
            }
        ",
        r"
            if (isNaN(foo) === false) {
                throw new Error();
            }
        ",
        r"
            if (Array.isArray(foo)) {
                throw new Error('foo is an Array');
            }
        ",
        r"
            if (foo instanceof bar) {
                throw new Error(foobar);
            }
        ",
        r"
            if (_.isElement(foo)) {
                throw new Error();
            }
        ",
        r"
            if (_.isElement(foo)) {
                throw new Error;
            }
        ",
        r"
            if (wrapper._.isElement(foo)) {
                throw new Error;
            }
        ",
        // disabled for now - not sure how do differentiate between `throw new Error` and `throw new Error()`
        // r#"                if (wrapper._.isElement(foo)) {
        //             throw new TypeError;
        //         }
        //         "#,
        r#"
            if (typeof foo == 'Foo' || 'Foo' === typeof foo) {
                throw new Error();
            }
        r#"
            if (Number.isFinite(foo) && Number.isSafeInteger(foo) && Number.isInteger(foo)) {
                throw new Error();
            }
        "#,
        r"
            if (wrapper.n.isFinite(foo) && wrapper.n.isSafeInteger(foo) && wrapper.n.isInteger(foo)) {
                throw new Error();
            }
        ",
        r"
            if (wrapper.f.g.n.isFinite(foo) && wrapper.g.n.isSafeInteger(foo) && wrapper.n.isInteger(foo)) {
                throw new Error();
            }
        ",
        r"
            if (_.isElement(foo)) {
                throw (new Error());
            }
        ",
    ];

    let fix = vec![
        (
            r"if (!isFinite(foo)) { throw new Error(); }",
            r"if (!isFinite(foo)) { throw new TypeError(); }",
        ),
        (
            r"if (isNaN(foo) === false) { throw new Error(); }",
            r"if (isNaN(foo) === false) { throw new TypeError(); }",
        ),
        (
            r"if (Array.isArray(foo)) { throw new Error('foo is an Array'); }",
            r"if (Array.isArray(foo)) { throw new TypeError('foo is an Array'); }",
        ),
        (
            r"if (foo instanceof bar) { throw new Error(foobar); }",
            r"if (foo instanceof bar) { throw new TypeError(foobar); }",
        ),
        (
            r"if (_.isElement(foo)) { throw new Error(); }",
            r"if (_.isElement(foo)) { throw new TypeError(); }",
        ),
        (
            r"if (_.isElement(foo)) { throw new Error; }",
            r"if (_.isElement(foo)) { throw new TypeError; }",
        ),
        (
            r"if (wrapper._.isElement(foo)) { throw new Error; }",
            r"if (wrapper._.isElement(foo)) { throw new TypeError; }",
        ),
        (
            r"if (typeof foo == 'Foo' || 'Foo' === typeof foo) { throw new Error(); }",
            r"if (typeof foo == 'Foo' || 'Foo' === typeof foo) { throw new TypeError(); }",
        ),
        (
            r"if (Number.isFinite(foo) && Number.isSafeInteger(foo) && Number.isInteger(foo)) { throw new Error(); }",
            r"if (Number.isFinite(foo) && Number.isSafeInteger(foo) && Number.isInteger(foo)) { throw new TypeError(); }",
        ),
        (
            r"if (wrapper.n.isFinite(foo) && wrapper.n.isSafeInteger(foo) && wrapper.n.isInteger(foo)) { throw new Error(); }",
            r"if (wrapper.n.isFinite(foo) && wrapper.n.isSafeInteger(foo) && wrapper.n.isInteger(foo)) { throw new TypeError(); }",
        ),
        (
            r"if (wrapper.f.g.n.isFinite(foo) && wrapper.g.n.isSafeInteger(foo) && wrapper.n.isInteger(foo)) { throw new Error(); }",
            r"if (wrapper.f.g.n.isFinite(foo) && wrapper.g.n.isSafeInteger(foo) && wrapper.n.isInteger(foo)) { throw new TypeError(); }",
        ),
        (
            r"if (_.isElement(foo)) { throw (new Error()); }",
            r"if (_.isElement(foo)) { throw (new TypeError()); }",
        ),
    ];

    Tester::new(PreferTypeError::NAME, PreferTypeError::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
