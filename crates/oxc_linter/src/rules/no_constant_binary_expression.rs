#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    ast_util::{self, IsConstant},
    context::LintContext,
    globals::BUILTINS,
    rule::Rule,
    AstNode,
};

/// `https://eslint.org/docs/latest/rules/no-constant-binary-expression`
/// Original Author: Jordan Eldredge <https://jordaneldredge.com>
#[derive(Debug, Default, Clone)]
pub struct NoConstantBinaryExpression;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow expressions where the operation doesn't affect the value
    ///
    /// ### Why is this bad?
    /// Comparisons which will always evaluate to true or false and logical expressions (||, &&, ??) which either always
    /// short-circuit or never short-circuit are both likely indications of programmer error.
    ///
    /// These errors are especially common in complex expressions where operator precedence is easy to misjudge.
    ///
    /// Additionally, this rule detects comparisons to newly constructed objects/arrays/functions/etc.
    /// In JavaScript, where objects are compared by reference, a newly constructed object can never === any other value.
    /// This can be surprising for programmers coming from languages where objects are compared by value.
    ///
    /// ### Example
    /// ```javascript
    /// // One might think this would evaluate as `a + (b ?? c)`:
    /// const x = a + b ?? c;
    ///
    /// // But it actually evaluates as `(a + b) ?? c`. Since `a + b` can never be null,
    /// // the `?? c` has no effect.
    ///
    /// // Programmers coming from a language where objects are compared by value might expect this to work:
    /// const isEmpty = x === [];
    ///
    /// // However, this will always result in `isEmpty` being `false`.
    /// ```
    NoConstantBinaryExpression,
    correctness
);

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint(no-constant-binary-expression): Disallow expressions where the operation doesn't affect the value"
)]
#[diagnostic()]
struct NoConstantBinaryExpressionDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint(no-constant-binary-expression): Unexpected constant {0:?} on the left-hand side of a `{1:?}` expression"
)]
#[diagnostic(severity(warning))]
struct ConstantShortCircuit(
    &'static str, // property
    String,       // operator
    #[label("This expression always evaluates to the constant on the left-hand side")] Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-constant-binary-expression): Unexpected constant binary expression")]
#[diagnostic(severity(warning))]
struct ConstantBinaryOperand(
    &'static str, // otherSide
    String,       // operator
    #[label("This compares constantly with the {0}-hand side of the `{1}`")] Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-constant-binary-expression): Unexpected comparison to newly constructed object")]
#[diagnostic(severity(warning))]
struct ConstantAlwaysNew(#[label("These two values can never be equal")] Span);

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint(no-constant-binary-expression): Unexpected comparison of two newly constructed objects"
)]
#[diagnostic(severity(warning))]
struct ConstantBothAlwaysNew(#[label("These two values can never be equal")] Span);

impl Rule for NoConstantBinaryExpression {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.get().kind() {
            AstKind::LogicalExpression(expr) => match expr.operator {
                LogicalOperator::Or | LogicalOperator::And if expr.left.is_constant(true, ctx) => {
                    ctx.diagnostic(ConstantShortCircuit(
                        "truthiness",
                        expr.operator.to_string(),
                        expr.span,
                    ));
                }
                LogicalOperator::Coalesce
                    if Self::has_constant_nullishness(&expr.left, false, ctx) =>
                {
                    ctx.diagnostic(ConstantShortCircuit(
                        "nullishness",
                        expr.operator.to_string(),
                        expr.span,
                    ));
                }
                _ => {}
            },
            AstKind::BinaryExpression(expr) => {
                let left = &expr.left;
                let right = &expr.right;
                let operator = expr.operator;

                let right_constant_operand =
                    Self::find_binary_expression_constant_operand(left, right, operator, ctx);

                let left_constant_operand =
                    Self::find_binary_expression_constant_operand(right, left, operator, ctx);

                if right_constant_operand.is_some() {
                    ctx.diagnostic(ConstantBinaryOperand("left", operator.to_string(), expr.span));
                    return;
                }

                if left_constant_operand.is_some() {
                    ctx.diagnostic(ConstantBinaryOperand("right", operator.to_string(), expr.span));
                    return;
                }

                if matches!(
                    operator,
                    BinaryOperator::StrictEquality | BinaryOperator::StrictInequality
                ) && (Self::is_always_new(left, ctx) || Self::is_always_new(right, ctx))
                {
                    ctx.diagnostic(ConstantAlwaysNew(expr.span));
                    return;
                }

                if matches!(operator, BinaryOperator::Equality | BinaryOperator::Inequality)
                    && Self::is_always_new(left, ctx)
                    && Self::is_always_new(right, ctx)
                {
                    ctx.diagnostic(ConstantBothAlwaysNew(expr.span));
                }
            }
            _ => {}
        }
    }
}

impl NoConstantBinaryExpression {
    ///  Test if an AST node has a statically knowable constant nullishness. Meaning,
    /// it will always resolve to a constant value of either: `null`, `undefined`
    /// or not `null` _or_ `undefined`. An expression that can vary between those
    /// three states at runtime would return `false`.
    fn has_constant_nullishness<'a>(
        expr: &Expression<'a>,
        non_nullish: bool,
        ctx: &LintContext<'a>,
    ) -> bool {
        if non_nullish && (expr.is_null() || expr.evaluate_to_undefined()) {
            return false;
        }
        match expr.get_inner_expression() {
            Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ClassExpression(_)
            | Expression::NewExpression(_)
            | Expression::TemplateLiteral(_)
            | Expression::UpdateExpression(_)
            | Expression::BinaryExpression(_)
            | Expression::UnaryExpression(_) => true,
            expr if expr.is_literal_expression() => true,
            Expression::CallExpression(call_expr) => {
                if let Expression::Identifier(ident) = &call_expr.callee {
                    return ["Boolean", "String", "Number"].contains(&ident.name.as_str())
                        && ctx.is_reference_to_global_variable(ident);
                }
                false
            }
            Expression::LogicalExpression(logical_expr)
                if logical_expr.operator == LogicalOperator::Coalesce =>
            {
                Self::has_constant_nullishness(&logical_expr.right, true, ctx)
            }
            Expression::AssignmentExpression(assign_expr) => match assign_expr.operator {
                AssignmentOperator::Assign => {
                    Self::has_constant_nullishness(&assign_expr.right, non_nullish, ctx)
                }
                op if op.is_logical_operator() => false,
                _ => true,
            },
            Expression::SequenceExpression(sequence_expr) => sequence_expr
                .expressions
                .iter()
                .last()
                .map_or(false, |last| Self::has_constant_nullishness(last, non_nullish, ctx)),
            Expression::Identifier(_) => expr.evaluate_to_undefined(),
            _ => false,
        }
    }

    /// Checks if one operand will cause the result to be constant.
    fn find_binary_expression_constant_operand<'a>(
        a: &'a Expression<'a>,
        b: &'a Expression<'a>,
        operator: BinaryOperator,
        ctx: &LintContext<'a>,
    ) -> Option<&'a Expression<'a>> {
        match operator {
            BinaryOperator::Equality | BinaryOperator::Inequality => {
                if (a.is_null_or_undefined() && Self::has_constant_nullishness(b, false, ctx))
                    || (ast_util::is_static_boolean(a, ctx)
                        && Self::has_constant_loose_boolean_comparison(b, ctx))
                {
                    return Some(b);
                }
            }
            BinaryOperator::StrictEquality | BinaryOperator::StrictInequality => {
                if (a.is_null_or_undefined() && Self::has_constant_nullishness(b, false, ctx))
                    || (ast_util::is_static_boolean(a, ctx)
                        && Self::has_constant_strict_boolean_comparison(b, ctx))
                {
                    return Some(b);
                }
            }
            _ => {}
        }
        None
    }

    /// Test if an AST node will always give the same result when compared to a
    /// boolean value. Note that comparison to boolean values is different than
    /// truthiness.
    /// `https://262.ecma-international.org/5.1/#sec-11.9.3`
    fn has_constant_loose_boolean_comparison<'a>(
        expr: &Expression<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        match expr {
            Expression::ObjectExpression(_)
            | Expression::ClassExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_) => true,
            Expression::ArrayExpression(array_expr) => {
                array_expr.elements.is_empty()
                    || array_expr
                        .elements
                        .iter()
                        .filter(|e| matches!(e, Some(Argument::Expression(_))))
                        .count()
                        > 1
            }
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::Void | UnaryOperator::Typeof => true,
                UnaryOperator::LogicalNot => unary_expr.argument.is_constant(true, ctx),
                _ => false,
            },
            Expression::CallExpression(call_expr) => call_expr.is_constant(true, ctx),
            Expression::TemplateLiteral(lit) => lit.expressions.is_empty(),
            Expression::AssignmentExpression(assignment_expr) => {
                assignment_expr.operator == AssignmentOperator::Assign
                    && Self::has_constant_loose_boolean_comparison(&assignment_expr.right, ctx)
            }
            Expression::SequenceExpression(sequence_expr) => sequence_expr
                .expressions
                .iter()
                .last()
                .map_or(false, |last| Self::has_constant_loose_boolean_comparison(last, ctx)),
            Expression::ParenthesizedExpression(paren_expr) => {
                Self::has_constant_loose_boolean_comparison(&paren_expr.expression, ctx)
            }
            expr if expr.is_literal_expression() => true,
            expr if expr.evaluate_to_undefined() => true,
            _ => false,
        }
    }

    /// Test if an AST node will always give the same result when _strictly_ compared
    /// to a boolean value. This can happen if the expression can never be boolean, or
    /// if it is always the same boolean value.
    fn has_constant_strict_boolean_comparison<'a>(
        expr: &Expression<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        match expr {
            Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::NewExpression(_)
            | Expression::TemplateLiteral(_)
            | Expression::UpdateExpression(_) => true,
            expr if expr.is_literal_expression() => true,
            Expression::BinaryExpression(binary_expr) => {
                binary_expr.operator.is_numeric_or_string_binary_operator()
            }
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::Delete => false,
                UnaryOperator::LogicalNot => unary_expr.argument.is_constant(true, ctx),
                _ => true,
            },
            Expression::CallExpression(call_expr) => {
                if let Expression::Identifier(ident) = &call_expr.callee {
                    if ident.name == "String"
                        || ident.name == "Number" && ctx.is_reference_to_global_variable(ident)
                    {
                        return true;
                    }

                    if ident.name == "Boolean" && ctx.is_reference_to_global_variable(ident) {
                        return call_expr
                            .arguments
                            .iter()
                            .next()
                            .map_or(true, |first| first.is_constant(true, ctx));
                    }
                }
                false
            }
            Expression::AssignmentExpression(assign_expr) => match assign_expr.operator {
                AssignmentOperator::Assign => {
                    Self::has_constant_strict_boolean_comparison(&assign_expr.right, ctx)
                }
                op if op.is_logical_operator() => false,
                _ => true,
            },
            Expression::SequenceExpression(sequence_expr) => sequence_expr
                .expressions
                .iter()
                .last()
                .map_or(false, |last| Self::has_constant_strict_boolean_comparison(last, ctx)),
            Expression::ParenthesizedExpression(paren_expr) => {
                Self::has_constant_strict_boolean_comparison(&paren_expr.expression, ctx)
            }
            Expression::Identifier(_) => expr.evaluate_to_undefined(),
            _ => false,
        }
    }

    /// Test if an AST node will always result in a newly constructed object
    fn is_always_new<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
        match expr {
            Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ClassExpression(_)
            | Expression::RegExpLiteral(_) => true,
            Expression::NewExpression(call_expr) => {
                if let Expression::Identifier(ident) = &call_expr.callee {
                    return BUILTINS.contains_key(ident.name.as_str())
                        && ctx.is_reference_to_global_variable(ident);
                }
                false
            }
            Expression::SequenceExpression(sequence_expr) => sequence_expr
                .expressions
                .iter()
                .last()
                .map_or(false, |last| Self::is_always_new(last, ctx)),
            Expression::AssignmentExpression(assignment_expr)
                if assignment_expr.operator == AssignmentOperator::Assign =>
            {
                Self::is_always_new(&assignment_expr.right, ctx)
            }
            Expression::ConditionalExpression(cond_expr) => {
                Self::is_always_new(&cond_expr.consequent, ctx)
                    && Self::is_always_new(&cond_expr.alternate, ctx)
            }
            Expression::ParenthesizedExpression(paren_expr) => {
                Self::is_always_new(&paren_expr.expression, ctx)
            }
            _ => false,
        }
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // While this _would_ be a constant condition in React, ESLint has a policy of not attributing any specific behavior to JSX.
        ("<p /> && foo", None),
        ("<></> && foo", None),
        ("<p /> ?? foo", None),
        ("<></> ?? foo", None),
        ("arbitraryFunction(n) ?? foo", None),
        ("foo.Boolean(n) ?? foo", None),
        ("(x += 1) && foo", None),
        ("`${bar}` && foo", None),
        ("bar && foo", None),
        ("delete bar.baz && foo", None),
        ("true ? foo : bar", None), // We leave ConditionalExpression for `no-constant-condition`
        ("new Foo() == true", None),
        ("foo == true", None),
        ("`${foo}` == true", None),
        ("`${foo}${bar}` == true", None),
        ("`0${foo}` == true", None),
        ("`00000000${foo}` == true", None),
        ("`0${foo}.000` == true", None),
        ("[n] == true", None),
        ("delete bar.baz === true", None),
        ("foo.Boolean(true) && foo", None),
        // ("function Boolean(n) { return n; }; Boolean(x) ?? foo", None),
        // ("function String(n) { return n; }; String(x) ?? foo", None),
        // ("function Number(n) { return n; }; Number(x) ?? foo", None),
        // ("function Boolean(n) { return Math.random(); }; Boolean(x) === 1", None),
        // ("function Boolean(n) { return Math.random(); }; Boolean(1) == true", None),
        ("new Foo() === x", None),
        ("x === new someObj.Promise()", None),
        ("Boolean(foo) === true", None),
        // ("function foo(undefined) { undefined ?? bar;}", None),
        // ("function foo(undefined) { undefined == true;}", None),
        // ("function foo(undefined) { undefined === true;}", None),
        ("[...arr, 1] == true", None),
        ("[,,,] == true", None),
        // { code: "new Foo() === bar;", globals: { Foo: "writable" } },
        ("(foo && true) ?? bar", None),
        ("foo ?? null ?? bar", None),
        ("a ?? (doSomething(), undefined) ?? b", None),
        ("a ?? (something = null) ?? b", None),
    ];

    let fail = vec![
        // Error messages
        ("[] && greeting", None),
        ("[] || greeting", None),
        ("[] ?? greeting", None),
        ("[] == true", None),
        ("true == []", None),
        ("[] != true", None),
        ("[] === true", None),
        ("[] !== true", None),
        // Motivating examples from the original proposal https://github.com/eslint/eslint/issues/13752
        ("!foo == null", None),
        ("!foo ?? bar", None),
        ("(a + b) / 2 ?? bar", None),
        ("String(foo.bar) ?? baz", None),
        ("'hello' + name ?? ''", None),
        ("[foo?.bar ?? ''] ?? []", None),
        // Logical expression with constant truthiness
        ("true && hello", None),
        ("true || hello", None),
        ("true && foo", None),
        ("'' && foo", None),
        ("100 && foo", None),
        ("+100 && foo", None),
        ("-100 && foo", None),
        ("~100 && foo", None),
        ("/[a-z]/ && foo", None),
        ("Boolean([]) && foo", None),
        ("Boolean() && foo", None),
        ("Boolean([], n) && foo", None),
        ("({}) && foo", None),
        ("[] && foo", None),
        ("(() => {}) && foo", None),
        ("(function() {}) && foo", None),
        ("(class {}) && foo", None),
        ("(class { valueOf() { return x; } }) && foo", None),
        ("(class { [x]() { return x; } }) && foo", None),
        ("new Foo() && foo", None),
        // (boxed values are always truthy)
        ("new Boolean(unknown) && foo", None),
        ("(bar = false) && foo", None),
        ("(bar.baz = false) && foo", None),
        ("(bar[0] = false) && foo", None),
        ("`hello ${hello}` && foo", None),
        ("void bar && foo", None),
        ("!true && foo", None),
        ("typeof bar && foo", None),
        ("(bar, baz, true) && foo", None),
        ("undefined && foo", None),
        // Logical expression with constant nullishness
        ("({}) ?? foo", None),
        ("([]) ?? foo", None),
        ("(() => {}) ?? foo", None),
        ("(function() {}) ?? foo", None),
        ("(class {}) ?? foo", None),
        ("new Foo() ?? foo", None),
        ("1 ?? foo", None),
        ("/[a-z]/ ?? foo", None),
        ("`${''}` ?? foo", None),
        ("(a = true) ?? foo", None),
        ("(a += 1) ?? foo", None),
        ("(a -= 1) ?? foo", None),
        ("(a *= 1) ?? foo", None),
        ("(a /= 1) ?? foo", None),
        ("(a %= 1) ?? foo", None),
        ("(a <<= 1) ?? foo", None),
        ("(a >>= 1) ?? foo", None),
        ("(a >>>= 1) ?? foo", None),
        ("(a |= 1) ?? foo", None),
        ("(a ^= 1) ?? foo", None),
        ("(a &= 1) ?? foo", None),
        ("undefined ?? foo", None),
        ("!bar ?? foo", None),
        ("void bar ?? foo", None),
        ("typeof bar ?? foo", None),
        ("+bar ?? foo", None),
        ("-bar ?? foo", None),
        ("~bar ?? foo", None),
        ("++bar ?? foo", None),
        ("bar++ ?? foo", None),
        ("--bar ?? foo", None),
        ("bar-- ?? foo", None),
        ("(x == y) ?? foo", None),
        ("(x + y) ?? foo", None),
        ("(x / y) ?? foo", None),
        ("(x instanceof String) ?? foo", None),
        ("(x in y) ?? foo", None),
        ("Boolean(x) ?? foo", None),
        ("String(x) ?? foo", None),
        ("Number(x) ?? foo", None),
        // Binary expression with comparison to null
        ("({}) != null", None),
        ("({}) == null", None),
        ("null == ({})", None),
        ("({}) == undefined", None),
        ("undefined == ({})", None),
        // Binary expression with loose comparison to boolean
        ("({}) != true", None),
        ("({}) == true", None),
        ("([]) == true", None),
        ("([a, b]) == true", None),
        ("(() => {}) == true", None),
        ("(function() {}) == true", None),
        ("void foo == true", None),
        ("typeof foo == true", None),
        ("![] == true", None),
        ("true == class {}", None),
        ("true == 1", None),
        ("undefined == true", None),
        ("true == undefined", None),
        ("`hello` == true", None),
        ("/[a-z]/ == true", None),
        ("({}) == Boolean({})", None),
        ("({}) == Boolean()", None),
        ("({}) == Boolean(() => {}, foo)", None),
        // Binary expression with strict comparison to boolean
        ("({}) !== true", None),
        ("({}) == !({})", None),
        ("({}) === true", None),
        ("([]) === true", None),
        ("(function() {}) === true", None),
        ("(() => {}) === true", None),
        ("!{} === true", None),
        ("typeof n === true", None),
        ("void n === true", None),
        ("+n === true", None),
        ("-n === true", None),
        ("~n === true", None),
        ("true === true", None),
        ("1 === true", None),
        ("'hello' === true", None),
        ("/[a-z]/ === true", None),
        ("undefined === true", None),
        ("(a = {}) === true", None),
        ("(a += 1) === true", None),
        ("(a -= 1) === true", None),
        ("(a *= 1) === true", None),
        ("(a %= 1) === true", None),
        ("(a ** b) === true", None),
        ("(a << b) === true", None),
        ("(a >> b) === true", None),
        ("(a >>> b) === true", None),
        ("--a === true", None),
        ("a-- === true", None),
        ("++a === true", None),
        ("a++ === true", None),
        ("(a + b) === true", None),
        ("(a - b) === true", None),
        ("(a * b) === true", None),
        ("(a / b) === true", None),
        ("(a % b) === true", None),
        ("(a | b) === true", None),
        ("(a ^ b) === true", None),
        ("(a & b) === true", None),
        ("Boolean(0) === Boolean(1)", None),
        ("true === String(x)", None),
        ("true === Number(x)", None),
        ("Boolean(0) == !({})", None),
        // Binary expression with strict comparison to null
        ("({}) !== null", None),
        ("({}) === null", None),
        ("([]) === null", None),
        ("(() => {}) === null", None),
        ("(function() {}) === null", None),
        ("(class {}) === null", None),
        ("new Foo() === null", None),
        ("`` === null", None),
        ("1 === null", None),
        ("'hello' === null", None),
        ("/[a-z]/ === null", None),
        ("true === null", None),
        ("null === null", None),
        ("a++ === null", None),
        ("++a === null", None),
        ("--a === null", None),
        ("a-- === null", None),
        ("!a === null", None),
        ("typeof a === null", None),
        ("delete a === null", None),
        ("void a === null", None),
        ("undefined === null", None),
        ("(x = {}) === null", None),
        ("(x += y) === null", None),
        ("(x -= y) === null", None),
        ("(a, b, {}) === null", None),
        // Binary expression with strict comparison to undefined
        ("({}) !== undefined", None),
        ("({}) === undefined", None),
        ("([]) === undefined", None),
        ("(() => {}) === undefined", None),
        ("(function() {}) === undefined", None),
        ("(class {}) === undefined", None),
        ("new Foo() === undefined", None),
        ("`` === undefined", None),
        ("1 === undefined", None),
        ("'hello' === undefined", None),
        ("/[a-z]/ === undefined", None),
        ("true === undefined", None),
        ("null === undefined", None),
        ("a++ === undefined", None),
        ("++a === undefined", None),
        ("--a === undefined", None),
        ("a-- === undefined", None),
        ("!a === undefined", None),
        ("typeof a === undefined", None),
        ("delete a === undefined", None),
        ("void a === undefined", None),
        ("undefined === undefined", None),
        ("(x = {}) === undefined", None),
        ("(x += y) === undefined", None),
        ("(x -= y) === undefined", None),
        ("(a, b, {}) === undefined", None),
        /*
         * If both sides are newly constructed objects, we can tell they will
         * never be equal, even with == equality.
         */
        ("[a] == [a]", None),
        ("[a] != [a]", None),
        ("({}) == []", None),
        // Comparing to always new objects
        ("x === {}", None),
        ("x !== {}", None),
        ("x === []", None),
        ("x === (() => {})", None),
        ("x === (function() {})", None),
        ("x === (class {})", None),
        ("x === new Boolean()", None),
        ("x === new Promise()", None),
        ("x === new WeakSet()", None),
        ("x === (foo, {})", None),
        ("x === (y = {})", None),
        ("x === (y ? {} : [])", None),
        ("x === /[a-z]/", None),
        // It's not obvious what this does, but it compares the old value of `x` to the new object.
        ("x === (x = {})", None),
        ("window.abc && false && anything", None),
        ("window.abc || true || anything", None),
        ("window.abc ?? 'non-nullish' ?? anything", None),
    ];

    Tester::new(NoConstantBinaryExpression::NAME, pass, fail).test_and_snapshot();
}
