use oxc_ast::{
    AstKind,
    ast::{
        AssignmentExpression, AssignmentTarget, BinaryOperator, Expression, MemberExpression,
        SimpleAssignmentTarget, UnaryOperator, UpdateOperator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::precedence::GetPrecedence;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_same_member_expression};

fn operator_assignment_diagnostic(mode: Mode, span: Span, operator: &str) -> OxcDiagnostic {
    let msg = if Mode::Never == mode {
        format!("Unexpected operator assignment ({operator}) shorthand.")
    } else {
        format!("Assignment (=) can be replaced with operator assignment ({operator}).")
    };
    OxcDiagnostic::warn(msg).with_label(span)
}

#[derive(Debug, Default, PartialEq, Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum Mode {
    /// Requires assignment operator shorthand where possible.
    #[default]
    Always,
    /// Disallows assignment operator shorthand.
    Never,
}

impl Mode {
    pub fn from(raw: &str) -> Self {
        if raw == "never" { Self::Never } else { Self::Always }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct OperatorAssignment {
    mode: Mode,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule requires or disallows assignment operator shorthand where possible.
    /// It encourages the use of shorthand assignment operators like `+=`, `-=`, `*=`, `/=`, etc.
    /// to make the code more concise and readable.
    ///
    /// ### Why is this bad?
    ///
    /// JavaScript provides shorthand operators that combine variable assignment and simple
    /// mathematical operations. Failing to use these shorthand operators can lead to unnecessarily
    /// verbose code and can be seen as a missed opportunity for clarity and simplicity.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `always` option:
    /// ```js
    /// x = x + y;
    /// x = y * x;
    /// x[0] = x[0] / y;
    /// x.y = x.y << z;
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `always` option:
    /// ```js
    /// x = y;
    /// x += y;
    /// x = y * z;
    /// x = (x * y) * z;
    /// x[0] /= y;
    /// x[foo()] = x[foo()] % 2;
    /// x = y + x; // `+` is not always commutative (e.g. x = "abc")
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `never` option:
    /// ```js
    /// x *= y;
    /// x ^= (y + z) / foo();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `never` option:
    /// ```js
    /// x = x + y;
    /// x.y = x.y / a.b;
    /// ```
    OperatorAssignment,
    eslint,
    style,
    fix_dangerous,
    config = Mode,
);

impl Rule for OperatorAssignment {
    fn from_configuration(value: Value) -> Self {
        Self { mode: value.get(0).and_then(Value::as_str).map(Mode::from).unwrap_or_default() }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assign_expr) = node.kind() else {
            return;
        };
        if self.mode == Mode::Never {
            prohibit(assign_expr, self.mode, ctx);
        } else {
            verify(assign_expr, self.mode, ctx);
        }
    }
}

fn verify(expr: &AssignmentExpression, mode: Mode, ctx: &LintContext) {
    if !expr.operator.is_assign() {
        return;
    }
    let left = &expr.left;
    let Expression::BinaryExpression(binary_expr) = &expr.right.without_parentheses() else {
        return;
    };
    let binary_operator = binary_expr.operator;
    let is_commutative_operator = is_commutative_operator_with_shorthand(binary_operator);
    let is_non_commutative_operator = is_non_commutative_operator_with_shorthand(binary_operator);
    if is_commutative_operator || is_non_commutative_operator {
        let replace_operator = format!("{}=", binary_operator.as_str());
        if check_is_same_reference(left, &binary_expr.left, ctx) {
            ctx.diagnostic_with_fix(
                operator_assignment_diagnostic(mode, expr.span, &replace_operator),
                |fixer| {
                    if !can_be_fixed(left) {
                        return fixer.noop();
                    }
                    let operator_span = get_operator_span(
                        Span::new(left.span().end, binary_expr.left.span().start),
                        "=",
                        ctx,
                    );
                    let binary_operator_span = get_operator_span(
                        Span::new(binary_expr.left.span().end, binary_expr.right.span().start),
                        binary_operator.as_str(),
                        ctx,
                    );
                    if ctx.has_comments_between(Span::new(
                        operator_span.end,
                        binary_operator_span.start,
                    )) {
                        return fixer.noop();
                    }
                    // e.g. x = x + y => x += y
                    // binary_operator = "+" and replace_operator = "+="
                    // left_text = "x " right_text = " y"
                    let left_text = Span::new(expr.span.start, operator_span.start)
                        .source_text(ctx.source_text());
                    let right_text = Span::new(binary_operator_span.end, binary_expr.span.end)
                        .source_text(ctx.source_text());
                    fixer.replace(expr.span, format!("{left_text}{replace_operator}{right_text}"))
                },
            );
        } else if check_is_same_reference(left, &binary_expr.right, ctx) && is_commutative_operator
        {
            ctx.diagnostic(operator_assignment_diagnostic(mode, expr.span, &replace_operator));
        }
    }
}

fn prohibit(expr: &AssignmentExpression, mode: Mode, ctx: &LintContext) {
    if !expr.operator.is_assign() && !expr.operator.is_logical() {
        ctx.diagnostic_with_dangerous_fix(
operator_assignment_diagnostic(mode, expr.span, expr.operator.as_str()),
        |fixer| {
                if !can_be_fixed(&expr.left) {
                    return fixer.noop();
                }
                let right_expr = &expr.right;

                let operator_span = get_operator_span(
                    Span::new(expr.left.span().end, right_expr.span().start),
                    expr.operator.as_str(),
                    ctx
                );
                if ctx.has_comments_between(Span::new(expr.span.start, operator_span.start)) {
                    return fixer.noop();
                }
                let Some(new_operator) = expr.operator.to_binary_operator() else {
                    return fixer.noop()
                };
                let left_text = Span::new(expr.span.start, operator_span.start).source_text(ctx.source_text());
                let right_text = {
                    let right_expr_text = right_expr.span().source_text(ctx.source_text());
                    let former = Span::new(operator_span.end, right_expr.span().start).source_text(ctx.source_text());
                    match right_expr {
                        // For some special cases, we need to wrap the expression in a pair of ()
                        // e.g. "x += y + 1" => "x = x + (y + 1)"
                        // "x += () => {}" => "x = x + (() => {})"
                        // "x += y = 1" => "x = x + (y = 1)"
                        // "x += yield foo()" => "x = x + (yield foo())"
                        // "x += y || 3" => "x = x + (y || 3)"
                        Expression::BinaryExpression(binary_expr) if binary_expr.operator.precedence() <= new_operator.precedence() => {
                            format!("{former}({right_expr_text})")
                        }
                        Expression::AssignmentExpression(_)
                        | Expression::ArrowFunctionExpression(_)
                        | Expression::YieldExpression(_)
                        | Expression::LogicalExpression(_) => {
                            format!("{former}({right_expr_text})")
                        }
                        // For the rest
                        _ => {
                            let temp_right_text = Span::new(operator_span.end, right_expr.span().end).source_text(ctx.source_text());
                            let no_gap = right_expr.span().start == operator_span.end;
                            // we match the binary operator to determine whether right_text_prefix needs to be preceded by a space
                            let need_fill_space = match new_operator {
                                BinaryOperator::Division => {
                                    if let Some(first_comment) = ctx.comments().iter().find(|comment| {
                                        Span::new(operator_span.end, right_expr.span().end).contains_inclusive(comment.span)
                                    }) {
                                        // e.g. x /=/** comments */ y
                                        first_comment.span.start == operator_span.end
                                    } else {
                                        // e.g. x /=/^abc/
                                        matches!(right_expr, Expression::RegExpLiteral(regex_literal) if regex_literal.span.start == operator_span.end)
                                    }
                                }
                                // x+=+y => x=x+ +y;
                                BinaryOperator::Addition if no_gap => {
                                    matches!(right_expr, Expression::UnaryExpression(unary_expr) if unary_expr.operator == UnaryOperator::UnaryPlus)
                                        || matches!(right_expr, Expression::UpdateExpression(update_expr) if update_expr.operator == UpdateOperator::Increment)
                                }
                                // x-=-y => x= x- -y
                                BinaryOperator::Subtraction if no_gap => {
                                    matches!(right_expr, Expression::UnaryExpression(unary_expr) if unary_expr.operator == UnaryOperator::UnaryNegation)
                                        || matches!(right_expr, Expression::UpdateExpression(update_expr) if update_expr.operator == UpdateOperator::Decrement)
                                }
                                _ => false,
                            };
                            let right_text_prefix = if need_fill_space { " " } else { "" };
                            format!("{right_text_prefix}{temp_right_text}")
                        }
                    }
                };
                fixer.replace(expr.span, format!("{left_text}= {left_text}{}{right_text}", new_operator.as_str()))
            }
        );
    }
}

fn can_be_fixed(target: &AssignmentTarget) -> bool {
    let Some(simple_assignment_target) = target.as_simple_assignment_target() else { return false };

    if matches!(simple_assignment_target, SimpleAssignmentTarget::AssignmentTargetIdentifier(_)) {
        return true;
    }
    let Some(expr) = simple_assignment_target.as_member_expression() else {
        return false;
    };
    match expr {
        MemberExpression::ComputedMemberExpression(computed_expr) => {
            matches!(
                computed_expr.object,
                Expression::Identifier(_) | Expression::ThisExpression(_)
            ) && computed_expr.expression.is_literal()
        }
        MemberExpression::StaticMemberExpression(static_expr) => {
            matches!(static_expr.object, Expression::Identifier(_) | Expression::ThisExpression(_))
        }
        MemberExpression::PrivateFieldExpression(_) => false,
    }
}

#[expect(clippy::cast_possible_truncation)]
fn get_operator_span(init_span: Span, operator: &str, ctx: &LintContext) -> Span {
    let offset = init_span.source_text(ctx.source_text()).find(operator).unwrap_or(0) as u32;
    let start = init_span.start + offset;
    Span::new(start, start + operator.len() as u32)
}

fn check_is_same_reference(left: &AssignmentTarget, right: &Expression, ctx: &LintContext) -> bool {
    let Some(simple_assignment_target) = left.as_simple_assignment_target() else { return false };
    if let SimpleAssignmentTarget::AssignmentTargetIdentifier(id1) = simple_assignment_target {
        return matches!(right, Expression::Identifier(id2) if id2.name == id1.name);
    }

    let Some(left_member_expr) = simple_assignment_target.as_member_expression() else {
        return false;
    };
    let Some(right_member_expr) = right.without_parentheses().get_member_expr() else {
        return false;
    };
    // x.y vs x['y']
    if (matches!(left_member_expr, MemberExpression::ComputedMemberExpression(_))
        && !matches!(right_member_expr, MemberExpression::ComputedMemberExpression(_)))
        || (!matches!(left_member_expr, MemberExpression::ComputedMemberExpression(_))
            && matches!(right_member_expr, MemberExpression::ComputedMemberExpression(_)))
    {
        return false;
    }
    is_same_member_expression(left_member_expr, right_member_expr, ctx)
}

fn is_commutative_operator_with_shorthand(operator: BinaryOperator) -> bool {
    matches!(
        operator,
        BinaryOperator::Multiplication
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::BitwiseOR
    )
}

fn is_non_commutative_operator_with_shorthand(operator: BinaryOperator) -> bool {
    matches!(
        operator,
        BinaryOperator::Addition
            | BinaryOperator::Subtraction
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill
            | BinaryOperator::Exponential
    )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("x = y", None),
        ("x = y + x", None),
        ("x += x + y", None),
        ("x = (x + y) - z", None),
        ("x -= y", None),
        ("x = y - x", None),
        ("x *= x", None),
        ("x = y * z", None),
        ("x = (x * y) * z", None),
        ("x = y / x", None),
        ("x /= y", None),
        ("x %= y", None),
        ("x <<= y", None),
        ("x >>= x >> y", None),
        ("x >>>= y", None),
        ("x &= y", None),
        ("x **= y", None),
        ("x ^= y ^ z", None),
        ("x |= x | y", None),
        ("x = x && y", None),
        ("x = x || y", None),
        ("x = x < y", None),
        ("x = x > y", None),
        ("x = x <= y", None),
        ("x = x >= y", None),
        ("x = x instanceof y", None),
        ("x = x in y", None),
        ("x = x == y", None),
        ("x = x != y", None),
        ("x = x === y", None),
        ("x = x !== y", None),
        ("x[y] = x['y'] + z", None),
        ("x.y = x['y'] / z", None),
        ("x.y = z + x.y", None),
        ("x[fn()] = x[fn()] + y", None),
        ("x += x + y", Some(serde_json::json!(["always"]))),
        ("x = x + y", Some(serde_json::json!(["never"]))),
        ("x = x ** y", Some(serde_json::json!(["never"]))),
        ("x = y ** x", None),
        ("x = x * y + z", None),
        ("this.x = this.y + z", Some(serde_json::json!(["always"]))),
        ("this.x = foo.x + y", Some(serde_json::json!(["always"]))),
        ("this.x = foo.this.x + y", Some(serde_json::json!(["always"]))),
        ("const foo = 0; class C { foo = foo + 1; }", None),
        ("x = x && y", Some(serde_json::json!(["always"]))),
        ("x = x || y", Some(serde_json::json!(["always"]))),
        ("x = x ?? y", Some(serde_json::json!(["always"]))),
        ("x &&= y", Some(serde_json::json!(["never"]))),
        ("x ||= y", Some(serde_json::json!(["never"]))),
        ("x ??= y", Some(serde_json::json!(["never"]))),
        ("x = () => {};", Some(serde_json::json!(["never"]))),
    ];

    let fail = vec![
        ("x = x + y", None),
        ("x = x - y", None),
        ("x = x * y", None),
        ("x = y * x", None),
        ("x = (y * z) * x", None),
        ("x = x / y", None),
        ("x = x % y", None),
        ("x = x << y", None),
        ("x = x >> y", None),
        ("x = x >>> y", None),
        ("x = x & y", None),
        ("x = x ^ y", None),
        ("x = x | y", None),
        ("x[0] = x[0] - y", None),
        ("x.y[z['a']][0].b = x.y[z['a']][0].b * 2", None),
        ("x = x + y", Some(serde_json::json!(["always"]))),
        ("x = (x + y)", Some(serde_json::json!(["always"]))),
        ("x = x + (y)", Some(serde_json::json!(["always"]))),
        ("x += (y)", Some(serde_json::json!(["never"]))),
        ("x += y", Some(serde_json::json!(["never"]))),
        ("foo.bar = foo.bar + baz", None),
        ("foo.bar += baz", Some(serde_json::json!(["never"]))),
        ("this.foo = this.foo + bar", None),
        ("this.foo += bar", Some(serde_json::json!(["never"]))),
        ("foo.bar.baz = foo.bar.baz + qux", None),
        ("foo.bar.baz += qux", Some(serde_json::json!(["never"]))),
        ("this.foo.bar = this.foo.bar + baz", None),
        ("this.foo.bar += baz", Some(serde_json::json!(["never"]))),
        ("foo[bar] = foo[bar] + baz", None),
        ("this[foo] = this[foo] + bar", None),
        ("foo[bar] >>>= baz", Some(serde_json::json!(["never"]))),
        ("this[foo] >>>= bar", Some(serde_json::json!(["never"]))),
        ("foo[5] = foo[5] / baz", None),
        ("this[5] = this[5] / foo", None),
        (
            "/*1*/x/*2*/./*3*/y/*4*/= x.y +/*5*/z/*6*/./*7*/w/*8*/;",
            Some(serde_json::json!(["always"])),
        ),
        (
            "x // 1
			 . // 2
			 y // 3
			 = x.y + //4
			 z //5
			 . //6
			 w;",
            Some(serde_json::json!(["always"])),
        ),
        ("x = /*1*/ x + y", Some(serde_json::json!(["always"]))),
        (
            "x = //1
			 x + y",
            Some(serde_json::json!(["always"])),
        ),
        ("x.y = x/*1*/.y + z", Some(serde_json::json!(["always"]))),
        (
            "x.y = x. //1
			 y + z",
            Some(serde_json::json!(["always"])),
        ),
        ("x = x /*1*/ + y", Some(serde_json::json!(["always"]))),
        (
            "x = x //1
			 + y",
            Some(serde_json::json!(["always"])),
        ),
        ("/*1*/x +=/*2*/y/*3*/;", Some(serde_json::json!(["never"]))),
        (
            "x +=//1
			 y",
            Some(serde_json::json!(["never"])),
        ),
        ("(/*1*/x += y)", Some(serde_json::json!(["never"]))),
        ("x/*1*/+=  y", Some(serde_json::json!(["never"]))),
        (
            "x //1
			 +=  y",
            Some(serde_json::json!(["never"])),
        ),
        ("(/*1*/x) +=  y", Some(serde_json::json!(["never"]))),
        ("x/*1*/.y +=  z", Some(serde_json::json!(["never"]))),
        (
            "x.//1
			 y +=  z",
            Some(serde_json::json!(["never"])),
        ),
        ("(foo.bar) ^= ((((((((((((((((baz))))))))))))))))", Some(serde_json::json!(["never"]))),
        ("foo = foo ** bar", None),
        ("foo **= bar", Some(serde_json::json!(["never"]))),
        ("foo *= bar + 1", Some(serde_json::json!(["never"]))),
        ("foo -= bar - baz", Some(serde_json::json!(["never"]))),
        ("foo += bar + baz", Some(serde_json::json!(["never"]))),
        ("foo += bar = 1", Some(serde_json::json!(["never"]))),
        ("foo *= (bar + 1)", Some(serde_json::json!(["never"]))),
        ("foo+=-bar", Some(serde_json::json!(["never"]))),
        ("foo/=bar", Some(serde_json::json!(["never"]))),
        ("foo/=/**/bar", Some(serde_json::json!(["never"]))),
        (
            "foo/=//
			bar",
            Some(serde_json::json!(["never"])),
        ),
        ("foo/=/^bar$/", Some(serde_json::json!(["never"]))),
        ("foo+=+bar", Some(serde_json::json!(["never"]))),
        ("foo+= +bar", Some(serde_json::json!(["never"]))),
        ("foo+=/**/+bar", Some(serde_json::json!(["never"]))),
        ("foo+=+bar===baz", Some(serde_json::json!(["never"]))),
        ("(obj?.a).b = (obj?.a).b + y", None),
        ("obj.a = obj?.a + b", None),
        ("x += + (() => {})", Some(serde_json::json!(["never"]))),
        ("x += + /** fooo */ (() => {})", Some(serde_json::json!(["never"]))),
    ];

    let fix = vec![
        ("x = x + y", "x += y", None),
        ("x = x - y", "x -= y", None),
        ("x = x * y", "x *= y", None),
        ("x = x / y", "x /= y", None),
        ("x = x % y", "x %= y", None),
        ("x = x << y", "x <<= y", None),
        ("x = x >> y", "x >>= y", None),
        ("x = x >>> y", "x >>>= y", None),
        ("x = x & y", "x &= y", None),
        ("x = x ^ y", "x ^= y", None),
        ("x = x | y", "x |= y", None),
        ("x[0] = x[0] - y", "x[0] -= y", None),
        ("x = x + y", "x += y", Some(serde_json::json!(["always"]))),
        ("x = (x + y)", "x += y", Some(serde_json::json!(["always"]))),
        ("x = x + (y)", "x += (y)", Some(serde_json::json!(["always"]))),
        ("x += (y)", "x = x + (y)", Some(serde_json::json!(["never"]))),
        ("x += y", "x = x + y", Some(serde_json::json!(["never"]))),
        ("foo.bar = foo.bar + baz", "foo.bar += baz", None),
        ("foo.bar += baz", "foo.bar = foo.bar + baz", Some(serde_json::json!(["never"]))),
        ("this.foo = this.foo + bar", "this.foo += bar", None),
        ("this.foo += bar", "this.foo = this.foo + bar", Some(serde_json::json!(["never"]))),
        ("foo[5] = foo[5] / baz", "foo[5] /= baz", None),
        ("this[5] = this[5] / foo", "this[5] /= foo", None),
        (
            "/*1*/x/*2*/./*3*/y/*4*/= x.y +/*5*/z/*6*/./*7*/w/*8*/;",
            "/*1*/x/*2*/./*3*/y/*4*/+=/*5*/z/*6*/./*7*/w/*8*/;",
            Some(serde_json::json!(["always"])),
        ),
        (
            "x // 1
    		 . // 2
    		 y // 3
    		 = x.y + //4
    		 z //5
    		 . //6
    		 w;",
            "x // 1
    		 . // 2
    		 y // 3
    		 += //4
    		 z //5
    		 . //6
    		 w;",
            Some(serde_json::json!(["always"])),
        ),
        ("/*1*/x +=/*2*/y/*3*/;", "/*1*/x = x +/*2*/y/*3*/;", Some(serde_json::json!(["never"]))),
        (
            "x +=//1
    		 y",
            "x = x +//1
    		 y",
            Some(serde_json::json!(["never"])),
        ),
        ("(/*1*/x += y)", "(/*1*/x = x + y)", Some(serde_json::json!(["never"]))),
        (
            "(foo.bar) ^= ((((((((((((((((baz))))))))))))))))",
            "(foo.bar) = (foo.bar) ^ ((((((((((((((((baz))))))))))))))))",
            Some(serde_json::json!(["never"])),
        ),
        ("foo = foo ** bar", "foo **= bar", None),
        ("foo **= bar", "foo = foo ** bar", Some(serde_json::json!(["never"]))),
        ("foo *= bar + 1", "foo = foo * (bar + 1)", Some(serde_json::json!(["never"]))),
        ("foo -= bar - baz", "foo = foo - (bar - baz)", Some(serde_json::json!(["never"]))),
        ("foo += bar + baz", "foo = foo + (bar + baz)", Some(serde_json::json!(["never"]))),
        ("foo += bar = 1", "foo = foo + (bar = 1)", Some(serde_json::json!(["never"]))),
        ("foo *= (bar + 1)", "foo = foo * (bar + 1)", Some(serde_json::json!(["never"]))),
        ("foo+=-bar", "foo= foo+-bar", Some(serde_json::json!(["never"]))),
        ("foo/=bar", "foo= foo/bar", Some(serde_json::json!(["never"]))),
        ("foo/=/**/bar", "foo= foo/ /**/bar", Some(serde_json::json!(["never"]))),
        (
            "foo/=//
    		bar",
            "foo= foo/ //
    		bar",
            Some(serde_json::json!(["never"])),
        ),
        ("foo/=/^bar$/", "foo= foo/ /^bar$/", Some(serde_json::json!(["never"]))),
        ("foo+=+bar", "foo= foo+ +bar", Some(serde_json::json!(["never"]))),
        ("foo+= +bar", "foo= foo+ +bar", Some(serde_json::json!(["never"]))),
        ("foo+=/**/+bar", "foo= foo+/**/+bar", Some(serde_json::json!(["never"]))),
        ("foo+=+bar===baz", "foo= foo+(+bar===baz)", Some(serde_json::json!(["never"]))),
        ("x += () => {}", "x = x + (() => {})", Some(serde_json::json!(["never"]))),
        ("x += + (() => {})", "x = x + + (() => {})", Some(serde_json::json!(["never"]))),
        (
            "x += + /** fooo */ (() => {})",
            "x = x + + /** fooo */ (() => {})",
            Some(serde_json::json!(["never"])),
        ),
    ];
    Tester::new(OperatorAssignment::NAME, OperatorAssignment::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
