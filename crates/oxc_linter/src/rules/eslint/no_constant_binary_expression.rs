use oxc_ast::{AstKind, ast::*};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator};

use crate::{
    AstNode,
    ast_util::{self, IsConstant},
    context::LintContext,
    rule::Rule,
};

/// `https://eslint.org/docs/latest/rules/no-constant-binary-expression`
/// Original Author: Jordan Eldredge <https://jordaneldredge.com>
#[derive(Debug, Default, Clone)]
pub struct NoConstantBinaryExpression;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow expressions where the operation doesn't affect the value
    ///
    /// ### Why is this bad?
    ///
    /// Comparisons which will always evaluate to true or false and logical expressions (`||`, `&&`, `??`) which either always
    /// short-circuit or never short-circuit are both likely indications of programmer error.
    ///
    /// These errors are especially common in complex expressions where operator precedence is easy to misjudge.
    ///
    /// Additionally, this rule detects comparisons to newly constructed objects/arrays/functions/etc.
    /// In JavaScript, where objects are compared by reference, a newly constructed object can never `===` any other value.
    /// This can be surprising for programmers coming from languages where objects are compared by value.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
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
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const x = a + (b ?? c);
    ///
    /// const isEmpty = x.length === 0;
    /// ```
    NoConstantBinaryExpression,
    eslint,
    correctness
);

fn constant_short_circuit(lhs_name: &str, expr_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Unexpected constant {lhs_name} on the left-hand side of a {expr_name:?} expression"
    ))
    .with_help("This expression always evaluates to the constant on the left-hand side")
    .with_label(span)
}

fn constant_binary_operand(left_or_right: &str, operator: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected constant binary expression")
        .with_help(format!(
            "This compares constantly with the {left_or_right}-hand side of the {operator}"
        ))
        .with_label(span)
}

fn constant_always_new(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected comparison to newly constructed object")
        .with_help("These two values can never be equal")
        .with_label(span)
}

fn constant_both_always_new(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected comparison of two newly constructed objects")
        .with_help("These two values can never be equal")
        .with_label(span)
}

impl Rule for NoConstantBinaryExpression {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::LogicalExpression(expr) => match expr.operator {
                LogicalOperator::Or | LogicalOperator::And if expr.left.is_constant(true, ctx) => {
                    ctx.diagnostic(constant_short_circuit(
                        "truthiness",
                        expr.operator.as_str(),
                        expr.span,
                    ));
                }
                LogicalOperator::Coalesce
                    if Self::has_constant_nullishness(&expr.left, false, ctx) =>
                {
                    ctx.diagnostic(constant_short_circuit(
                        "nullishness",
                        expr.operator.as_str(),
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
                    ctx.diagnostic(constant_binary_operand("left", operator.as_str(), expr.span));
                    return;
                }

                if left_constant_operand.is_some() {
                    ctx.diagnostic(constant_binary_operand("right", operator.as_str(), expr.span));
                    return;
                }

                if matches!(
                    operator,
                    BinaryOperator::StrictEquality | BinaryOperator::StrictInequality
                ) && (Self::is_always_new(left, ctx) || Self::is_always_new(right, ctx))
                {
                    ctx.diagnostic(constant_always_new(expr.span));
                    return;
                }

                if matches!(operator, BinaryOperator::Equality | BinaryOperator::Inequality)
                    && Self::is_always_new(left, ctx)
                    && Self::is_always_new(right, ctx)
                {
                    ctx.diagnostic(constant_both_always_new(expr.span));
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
            expr if expr.is_literal() => true,
            Expression::CallExpression(call_expr) => {
                if let Expression::Identifier(ident) = &call_expr.callee {
                    return ["Boolean", "String", "Number"].contains(&ident.name.as_str())
                        && ctx.scoping().root_unresolved_references().contains_key(&ident.name);
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
                op if op.is_logical() => false,
                _ => true,
            },
            Expression::SequenceExpression(sequence_expr) => sequence_expr
                .expressions
                .iter()
                .last()
                .is_some_and(|last| Self::has_constant_nullishness(last, non_nullish, ctx)),
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
                    || array_expr.elements.iter().filter(|e| e.is_expression()).count() > 1
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
                .is_some_and(|last| Self::has_constant_loose_boolean_comparison(last, ctx)),
            Expression::ParenthesizedExpression(paren_expr) => {
                Self::has_constant_loose_boolean_comparison(&paren_expr.expression, ctx)
            }
            expr if expr.is_literal() => true,
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
            expr if expr.is_literal() => true,
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
                    let unresolved_references = ctx.scoping().root_unresolved_references();
                    if (ident.name == "String" || ident.name == "Number")
                        && unresolved_references.contains_key(&ident.name)
                    {
                        return true;
                    }

                    if ident.name == "Boolean" && unresolved_references.contains_key(&ident.name) {
                        return call_expr
                            .arguments
                            .iter()
                            .next()
                            .is_none_or(|first| first.is_constant(true, ctx));
                    }
                }
                false
            }
            Expression::AssignmentExpression(assign_expr) => match assign_expr.operator {
                AssignmentOperator::Assign => {
                    Self::has_constant_strict_boolean_comparison(&assign_expr.right, ctx)
                }
                op if op.is_logical() => false,
                _ => true,
            },
            Expression::SequenceExpression(sequence_expr) => sequence_expr
                .expressions
                .iter()
                .last()
                .is_some_and(|last| Self::has_constant_strict_boolean_comparison(last, ctx)),
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
                    return ctx.env_contains_var(ident.name.as_str())
                        && ctx.scoping().root_unresolved_references().contains_key(&ident.name);
                }
                false
            }
            Expression::SequenceExpression(sequence_expr) => sequence_expr
                .expressions
                .iter()
                .last()
                .is_some_and(|last| Self::is_always_new(last, ctx)),
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
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // While this _would_ be a constant condition in React, ESLint has a policy of not attributing any specific behavior to JSX.
        "<p /> && foo",
        "<></> && foo",
        "<p /> ?? foo",
        "<></> ?? foo",
        "arbitraryFunction(n) ?? foo",
        "foo.Boolean(n) ?? foo",
        "(x += 1) && foo",
        "`${bar}` && foo",
        "bar && foo",
        "delete bar.baz && foo",
        "true ? foo : bar", // We leave ConditionalExpression for `no-constant-condition`
        "new Foo() == true",
        "foo == true",
        "`${foo}` == true",
        "`${foo}${bar}` == true",
        "`0${foo}` == true",
        "`00000000${foo}` == true",
        "`0${foo}.000` == true",
        "[n] == true",
        "delete bar.baz === true",
        "foo.Boolean(true) && foo",
        // "function Boolean(n) { return n; }; Boolean(x) ?? foo",
        // "function String(n) { return n; }; String(x) ?? foo",
        // "function Number(n) { return n; }; Number(x) ?? foo",
        // "function Boolean(n) { return Math.random(); }; Boolean(x) === 1",
        // "function Boolean(n) { return Math.random(); }; Boolean(1) == true",
        "new Foo() === x",
        "x === new someObj.Promise()",
        "Boolean(foo) === true",
        // "function foo(undefined) { undefined ?? bar;}",
        // "function foo(undefined) { undefined == true;}",
        // "function foo(undefined) { undefined === true;}",
        "[...arr, 1] == true",
        "[,,,] == true",
        // { code: "new Foo() === bar;", globals: { Foo: "writable" } },
        "(foo && true) ?? bar",
        "foo ?? null ?? bar",
        "a ?? (doSomething(), undefined) ?? b",
        "a ?? (something = null) ?? b",
    ];

    let fail = vec![
        // Error messages
        "[] && greeting",
        "[] || greeting",
        "[] ?? greeting",
        "[] == true",
        "true == []",
        "[] != true",
        "[] === true",
        "[] !== true",
        // Motivating examples from the original proposal https://github.com/eslint/eslint/issues/13752
        "!foo == null",
        "!foo ?? bar",
        "(a + b) / 2 ?? bar",
        "String(foo.bar) ?? baz",
        "'hello' + name ?? ''",
        "[foo?.bar ?? ''] ?? []",
        // Logical expression with constant truthiness
        "true && hello",
        "true || hello",
        "true && foo",
        "'' && foo",
        "100 && foo",
        "+100 && foo",
        "-100 && foo",
        "~100 && foo",
        "/[a-z]/ && foo",
        "Boolean([]) && foo",
        "Boolean() && foo",
        "Boolean([], n) && foo",
        "({}) && foo",
        "[] && foo",
        "(() => {}) && foo",
        "(function() {}) && foo",
        "(class {}) && foo",
        "(class { valueOf() { return x; } }) && foo",
        "(class { [x]() { return x; } }) && foo",
        "new Foo() && foo",
        // (boxed values are always truthy)
        "new Boolean(unknown) && foo",
        "(bar = false) && foo",
        "(bar.baz = false) && foo",
        "(bar[0] = false) && foo",
        "`hello ${hello}` && foo",
        "void bar && foo",
        "!true && foo",
        "typeof bar && foo",
        "(bar, baz, true) && foo",
        "undefined && foo",
        // Logical expression with constant nullishness
        "({}) ?? foo",
        "([]) ?? foo",
        "(() => {}) ?? foo",
        "(function() {}) ?? foo",
        "(class {}) ?? foo",
        "new Foo() ?? foo",
        "1 ?? foo",
        "/[a-z]/ ?? foo",
        "`${''}` ?? foo",
        "(a = true) ?? foo",
        "(a += 1) ?? foo",
        "(a -= 1) ?? foo",
        "(a *= 1) ?? foo",
        "(a /= 1) ?? foo",
        "(a %= 1) ?? foo",
        "(a <<= 1) ?? foo",
        "(a >>= 1) ?? foo",
        "(a >>>= 1) ?? foo",
        "(a |= 1) ?? foo",
        "(a ^= 1) ?? foo",
        "(a &= 1) ?? foo",
        "undefined ?? foo",
        "!bar ?? foo",
        "void bar ?? foo",
        "typeof bar ?? foo",
        "+bar ?? foo",
        "-bar ?? foo",
        "~bar ?? foo",
        "++bar ?? foo",
        "bar++ ?? foo",
        "--bar ?? foo",
        "bar-- ?? foo",
        "(x == y) ?? foo",
        "(x + y) ?? foo",
        "(x / y) ?? foo",
        "(x instanceof String) ?? foo",
        "(x in y) ?? foo",
        "Boolean(x) ?? foo",
        "String(x) ?? foo",
        "Number(x) ?? foo",
        // Binary expression with comparison to null
        "({}) != null",
        "({}) == null",
        "null == ({})",
        "({}) == undefined",
        "undefined == ({})",
        // Binary expression with loose comparison to boolean
        "({}) != true",
        "({}) == true",
        "([]) == true",
        "([a, b]) == true",
        "(() => {}) == true",
        "(function() {}) == true",
        "void foo == true",
        "typeof foo == true",
        "![] == true",
        "true == class {}",
        "true == 1",
        "undefined == true",
        "true == undefined",
        "`hello` == true",
        "/[a-z]/ == true",
        "({}) == Boolean({})",
        "({}) == Boolean()",
        "({}) == Boolean(() => {}, foo)",
        // Binary expression with strict comparison to boolean
        "({}) !== true",
        "({}) == !({})",
        "({}) === true",
        "([]) === true",
        "(function() {}) === true",
        "(() => {}) === true",
        "!{} === true",
        "typeof n === true",
        "void n === true",
        "+n === true",
        "-n === true",
        "~n === true",
        "true === true",
        "1 === true",
        "'hello' === true",
        "/[a-z]/ === true",
        "undefined === true",
        "(a = {}) === true",
        "(a += 1) === true",
        "(a -= 1) === true",
        "(a *= 1) === true",
        "(a %= 1) === true",
        "(a ** b) === true",
        "(a << b) === true",
        "(a >> b) === true",
        "(a >>> b) === true",
        "--a === true",
        "a-- === true",
        "++a === true",
        "a++ === true",
        "(a + b) === true",
        "(a - b) === true",
        "(a * b) === true",
        "(a / b) === true",
        "(a % b) === true",
        "(a | b) === true",
        "(a ^ b) === true",
        "(a & b) === true",
        "Boolean(0) === Boolean(1)",
        "true === String(x)",
        "true === Number(x)",
        "Boolean(0) == !({})",
        // Binary expression with strict comparison to null
        "({}) !== null",
        "({}) === null",
        "([]) === null",
        "(() => {}) === null",
        "(function() {}) === null",
        "(class {}) === null",
        "new Foo() === null",
        "`` === null",
        "1 === null",
        "'hello' === null",
        "/[a-z]/ === null",
        "true === null",
        "null === null",
        "a++ === null",
        "++a === null",
        "--a === null",
        "a-- === null",
        "!a === null",
        "typeof a === null",
        "delete a === null",
        "void a === null",
        "undefined === null",
        "(x = {}) === null",
        "(x += y) === null",
        "(x -= y) === null",
        "(a, b, {}) === null",
        // Binary expression with strict comparison to undefined
        "({}) !== undefined",
        "({}) === undefined",
        "([]) === undefined",
        "(() => {}) === undefined",
        "(function() {}) === undefined",
        "(class {}) === undefined",
        "new Foo() === undefined",
        "`` === undefined",
        "1 === undefined",
        "'hello' === undefined",
        "/[a-z]/ === undefined",
        "true === undefined",
        "null === undefined",
        "a++ === undefined",
        "++a === undefined",
        "--a === undefined",
        "a-- === undefined",
        "!a === undefined",
        "typeof a === undefined",
        "delete a === undefined",
        "void a === undefined",
        "undefined === undefined",
        "(x = {}) === undefined",
        "(x += y) === undefined",
        "(x -= y) === undefined",
        "(a, b, {}) === undefined",
        /*
         * If both sides are newly constructed objects, we can tell they will
         * never be equal, even with == equality.
         */
        "[a] == [a]",
        "[a] != [a]",
        "({}) == []",
        // Comparing to always new objects
        "x === {}",
        "x !== {}",
        "x === []",
        "x === (() => {})",
        "x === (function() {})",
        "x === (class {})",
        "x === new Boolean()",
        "x === new Promise()",
        "x === new WeakSet()",
        "x === (foo, {})",
        "x === (y = {})",
        "x === (y ? {} : [])",
        "x === /[a-z]/",
        // It's not obvious what this does, but it compares the old value of `x` to the new object.
        "x === (x = {})",
        "window.abc && false && anything",
        "window.abc || true || anything",
        "window.abc ?? 'non-nullish' ?? anything",
    ];

    Tester::new(NoConstantBinaryExpression::NAME, NoConstantBinaryExpression::PLUGIN, pass, fail)
        .test_and_snapshot();
}
