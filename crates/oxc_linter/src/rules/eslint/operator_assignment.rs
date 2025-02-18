use oxc_ast::{
    ast::{
        AssignmentExpression, AssignmentOperator, AssignmentTarget, BinaryOperator, Expression,
        MemberExpression, SimpleAssignmentTarget,
    },
    match_simple_assignment_target, AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule, utils::is_same_member_expression, AstNode};

fn operator_assignment_diagnostic(mode: Mode, span: Span, operator: &str) -> OxcDiagnostic {
    let msg = if Mode::Never == mode {
        format!("Unexpected operator assignment ({operator}) shorthand.")
    } else {
        format!("Assignment (=) can be replaced with operator assignment ({operator}).")
    };
    OxcDiagnostic::warn(msg).with_label(span)
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
enum Mode {
    #[default]
    Always,
    Never,
}

impl Mode {
    pub fn from(raw: &str) -> Self {
        if raw == "never" {
            Self::Never
        } else {
            Self::Always
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct OperatorAssignment {
    mode: Mode,
}

declare_oxc_lint!(
    /// ### What it does
    /// This rule requires or disallows assignment operator shorthand where possible.
    ///
    /// ### Why is this bad?
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
    OperatorAssignment,
    eslint,
    style,
    pending
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
            verfy(assign_expr, self.mode, ctx);
        }
    }
}

fn verfy(expr: &AssignmentExpression, mode: Mode, ctx: &LintContext) {
    if expr.operator != AssignmentOperator::Assign {
        return;
    }
    let left = &expr.left;

    if let Expression::BinaryExpression(binary_expr) = &expr.right.without_parentheses() {
        let binary_operator = binary_expr.operator;
        let is_commutative_operator = is_commutative_operator_with_shorthand(binary_operator);
        let is_non_commutative_operator =
            is_non_commutative_operator_with_shorthand(binary_operator);
        // for always
        if is_commutative_operator || is_non_commutative_operator {
            let replace_operator = format!("{}=", binary_operator.as_str());
            if check_is_same_reference(left, &binary_expr.left, ctx) {
                ctx.diagnostic(operator_assignment_diagnostic(mode, expr.span, &replace_operator));
            } else if check_is_same_reference(left, &binary_expr.right, ctx)
                && is_commutative_operator
            {
                // todo fix
                ctx.diagnostic(operator_assignment_diagnostic(mode, expr.span, &replace_operator));
            }
        }
    }
}

fn prohibit(expr: &AssignmentExpression, mode: Mode, ctx: &LintContext) {
    if expr.operator != AssignmentOperator::Assign
        && !check_is_logical_assign_operator(expr.operator)
    {
        if can_be_fixed(&expr.left) {
            // todo fix
            ctx.diagnostic(operator_assignment_diagnostic(mode, expr.span, expr.operator.as_str()));
        } else {
            ctx.diagnostic(operator_assignment_diagnostic(mode, expr.span, expr.operator.as_str()));
        }
    }
}

fn can_be_fixed(target: &AssignmentTarget) -> bool {
    match target {
        match_simple_assignment_target!(AssignmentTarget) => {
            let simple_assignment_target = target.to_simple_assignment_target();

            if matches!(
                simple_assignment_target,
                SimpleAssignmentTarget::AssignmentTargetIdentifier(_)
            ) {
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
                    matches!(
                        static_expr.object,
                        Expression::Identifier(_) | Expression::ThisExpression(_)
                    )
                }
                MemberExpression::PrivateFieldExpression(_) => false,
            }
        }
        _ => false,
    }
}

fn check_is_same_reference(left: &AssignmentTarget, right: &Expression, ctx: &LintContext) -> bool {
    match left {
        match_simple_assignment_target!(AssignmentTarget) => {
            let simple_assignment_target = left.to_simple_assignment_target();
            if let SimpleAssignmentTarget::AssignmentTargetIdentifier(id1) =
                simple_assignment_target
            {
                return matches!(right, Expression::Identifier(id2) if id2.name == id1.name);
            }

            if let Some(left_member_expr) = simple_assignment_target.as_member_expression() {
                if let Some(right_member_expr) = right.without_parentheses().get_member_expr() {
                    // x.y vs x['y']
                    if (matches!(left_member_expr, MemberExpression::ComputedMemberExpression(_))
                        && !matches!(
                            right_member_expr,
                            MemberExpression::ComputedMemberExpression(_)
                        ))
                        || (!matches!(
                            left_member_expr,
                            MemberExpression::ComputedMemberExpression(_)
                        ) && matches!(
                            right_member_expr,
                            MemberExpression::ComputedMemberExpression(_)
                        ))
                    {
                        return false;
                    }
                    return is_same_member_expression(left_member_expr, right_member_expr, ctx);
                }
            }
            false
        }
        _ => false,
    }
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

fn check_is_logical_assign_operator(operator: AssignmentOperator) -> bool {
    matches!(
        operator,
        AssignmentOperator::LogicalAnd
            | AssignmentOperator::LogicalOr
            | AssignmentOperator::LogicalNullish
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
    ];

    // let fix = vec![
    //     ("x = x + y", "x += y", None),
    //     ("x = x - y", "x -= y", None),
    //     ("x = x * y", "x *= y", None),
    //     ("x = x / y", "x /= y", None),
    //     ("x = x % y", "x %= y", None),
    //     ("x = x << y", "x <<= y", None),
    //     ("x = x >> y", "x >>= y", None),
    //     ("x = x >>> y", "x >>>= y", None),
    //     ("x = x & y", "x &= y", None),
    //     ("x = x ^ y", "x ^= y", None),
    //     ("x = x | y", "x |= y", None),
    //     ("x[0] = x[0] - y", "x[0] -= y", None),
    //     ("x = x + y", "x += y", Some(serde_json::json!(["always"]))),
    //     ("x = (x + y)", "x += y", Some(serde_json::json!(["always"]))),
    //     ("x = x + (y)", "x += (y)", Some(serde_json::json!(["always"]))),
    //     ("x += (y)", "x = x + (y)", Some(serde_json::json!(["never"]))),
    //     ("x += y", "x = x + y", Some(serde_json::json!(["never"]))),
    //     ("foo.bar = foo.bar + baz", "foo.bar += baz", None),
    //     ("foo.bar += baz", "foo.bar = foo.bar + baz", Some(serde_json::json!(["never"]))),
    //     ("this.foo = this.foo + bar", "this.foo += bar", None),
    //     ("this.foo += bar", "this.foo = this.foo + bar", Some(serde_json::json!(["never"]))),
    //     ("foo[5] = foo[5] / baz", "foo[5] /= baz", None),
    //     ("this[5] = this[5] / foo", "this[5] /= foo", None),
    //     (
    //         "/*1*/x/*2*/./*3*/y/*4*/= x.y +/*5*/z/*6*/./*7*/w/*8*/;",
    //         "/*1*/x/*2*/./*3*/y/*4*/+=/*5*/z/*6*/./*7*/w/*8*/;",
    //         Some(serde_json::json!(["always"])),
    //     ),
    //     (
    //         "x // 1
    // 		 . // 2
    // 		 y // 3
    // 		 = x.y + //4
    // 		 z //5
    // 		 . //6
    // 		 w;",
    //         "x // 1
    // 		 . // 2
    // 		 y // 3
    // 		 += //4
    // 		 z //5
    // 		 . //6
    // 		 w;",
    //         Some(serde_json::json!(["always"])),
    //     ),
    //     ("/*1*/x +=/*2*/y/*3*/;", "/*1*/x = x +/*2*/y/*3*/;", Some(serde_json::json!(["never"]))),
    //     (
    //         "x +=//1
    // 		 y",
    //         "x = x +//1
    // 		 y",
    //         Some(serde_json::json!(["never"])),
    //     ),
    //     ("(/*1*/x += y)", "(/*1*/x = x + y)", Some(serde_json::json!(["never"]))),
    //     (
    //         "(foo.bar) ^= ((((((((((((((((baz))))))))))))))))",
    //         "(foo.bar) = (foo.bar) ^ ((((((((((((((((baz))))))))))))))))",
    //         Some(serde_json::json!(["never"])),
    //     ),
    //     ("foo = foo ** bar", "foo **= bar", None),
    //     ("foo **= bar", "foo = foo ** bar", Some(serde_json::json!(["never"]))),
    //     ("foo *= bar + 1", "foo = foo * (bar + 1)", Some(serde_json::json!(["never"]))),
    //     ("foo -= bar - baz", "foo = foo - (bar - baz)", Some(serde_json::json!(["never"]))),
    //     ("foo += bar + baz", "foo = foo + (bar + baz)", Some(serde_json::json!(["never"]))),
    //     ("foo += bar = 1", "foo = foo + (bar = 1)", Some(serde_json::json!(["never"]))),
    //     ("foo *= (bar + 1)", "foo = foo * (bar + 1)", Some(serde_json::json!(["never"]))),
    //     ("foo+=-bar", "foo= foo+-bar", Some(serde_json::json!(["never"]))),
    //     ("foo/=bar", "foo= foo/bar", Some(serde_json::json!(["never"]))),
    //     ("foo/=/**/bar", "foo= foo/ /**/bar", Some(serde_json::json!(["never"]))),
    //     (
    //         "foo/=//
    // 		bar",
    //         "foo= foo/ //
    // 		bar",
    //         Some(serde_json::json!(["never"])),
    //     ),
    //     ("foo/=/^bar$/", "foo= foo/ /^bar$/", Some(serde_json::json!(["never"]))),
    //     ("foo+=+bar", "foo= foo+ +bar", Some(serde_json::json!(["never"]))),
    //     ("foo+= +bar", "foo= foo+ +bar", Some(serde_json::json!(["never"]))),
    //     ("foo+=/**/+bar", "foo= foo+/**/+bar", Some(serde_json::json!(["never"]))),
    //     ("foo+=+bar===baz", "foo= foo+(+bar===baz)", Some(serde_json::json!(["never"]))),
    // ];
    // Tester::new(OperatorAssignment::NAME, OperatorAssignment::PLUGIN, pass, fail)
    //     .expect_fix(fix)
    //     .test_and_snapshot();
    Tester::new(OperatorAssignment::NAME, OperatorAssignment::PLUGIN, pass, fail)
        .test_and_snapshot();
}
