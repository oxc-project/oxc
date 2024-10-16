use oxc_ast::{
    ast::{match_assignment_target_pattern, Argument, AssignmentTarget, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::LogicalOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_unsafe_optional_chaining_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unsafe usage of optional chaining")
        .with_help("If this short-circuits with 'undefined' the evaluation will throw TypeError")
        .with_label(span)
}

fn no_unsafe_arithmetic_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unsafe arithmetic operation on optional chaining")
        .with_help("This can result in NaN.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeOptionalChaining {
    /// Disallow arithmetic operations on optional chaining expressions (Default false).
    /// If this is true, this rule warns arithmetic operations on optional chaining expressions, which possibly result in NaN.
    disallow_arithmetic_operators: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow use of optional chaining in contexts where the undefined value is not allowed
    ///
    /// ### Why is this bad?
    ///
    /// The optional chaining (?.) expression can short-circuit with a return value of undefined.
    /// Therefore, treating an evaluated optional chaining expression as a function, object, number, etc.,
    /// can cause TypeError or unexpected results. For example:
    ///
    /// ### Example
    /// ```javascript
    /// var obj = undefined;
    /// 1 in obj?.foo;  // TypeError
    /// with (obj?.foo);  // TypeError
    /// for (bar of obj?.foo);  // TypeError
    /// bar instanceof obj?.foo;  // TypeError
    /// const { bar } = obj?.foo;  // TypeError
    /// ```
    NoUnsafeOptionalChaining,
    correctness
);

impl Rule for NoUnsafeOptionalChaining {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
            disallow_arithmetic_operators: value
                .get(0)
                .and_then(|v| v.get("disallowArithmeticOperators"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_default(),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(expr) if !expr.optional => {
                Self::check_unsafe_usage(&expr.callee, ctx);
            }
            AstKind::MemberExpression(expr) if !expr.optional() => {
                Self::check_unsafe_usage(expr.object(), ctx);
            }
            AstKind::TaggedTemplateExpression(expr) => {
                Self::check_unsafe_usage(&expr.tag, ctx);
            }
            AstKind::NewExpression(expr) => {
                Self::check_unsafe_usage(&expr.callee, ctx);
            }
            AstKind::AssignmentExpression(expr) => {
                if matches!(expr.left, match_assignment_target_pattern!(AssignmentTarget)) {
                    Self::check_unsafe_usage(&expr.right, ctx);
                }
                if expr.operator.is_arithmetic() {
                    self.check_unsafe_arithmetic(&expr.right, ctx);
                }
            }
            AstKind::BinaryExpression(expr) => match expr.operator {
                op if op.is_relational() => Self::check_unsafe_usage(&expr.right, ctx),
                op if op.is_arithmetic() => {
                    self.check_unsafe_arithmetic(&expr.left, ctx);
                    self.check_unsafe_arithmetic(&expr.right, ctx);
                }
                _ => {}
            },
            AstKind::UnaryExpression(expr) if expr.operator.is_arithmetic() => {
                self.check_unsafe_arithmetic(&expr.argument, ctx);
            }
            AstKind::ForOfStatement(stmt) => {
                Self::check_unsafe_usage(&stmt.right, ctx);
            }
            AstKind::WithStatement(stmt) => {
                Self::check_unsafe_usage(&stmt.object, ctx);
            }
            AstKind::Class(class) => {
                if let Some(expr) = &class.super_class {
                    Self::check_unsafe_usage(expr, ctx);
                }
            }
            AstKind::AssignmentPattern(pat) if pat.left.kind.is_destructuring_pattern() => {
                Self::check_unsafe_usage(&pat.right, ctx);
            }
            AstKind::Argument(Argument::SpreadElement(elem)) => {
                Self::check_unsafe_usage(&elem.argument, ctx);
            }
            AstKind::VariableDeclarator(decl) if decl.id.kind.is_destructuring_pattern() => {
                if let Some(expr) = &decl.init {
                    Self::check_unsafe_usage(expr, ctx);
                }
            }
            AstKind::AssignmentTargetWithDefault(target) => {
                if matches!(target.binding, match_assignment_target_pattern!(AssignmentTarget)) {
                    Self::check_unsafe_usage(&target.init, ctx);
                }
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy)]
enum ErrorType {
    Usage,
    Arithmetic,
}

impl NoUnsafeOptionalChaining {
    fn check_unsafe_usage<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) {
        Self::check_undefined_short_circuit(expr, ErrorType::Usage, ctx);
    }

    fn check_unsafe_arithmetic<'a>(&self, expr: &Expression<'a>, ctx: &LintContext<'a>) {
        if self.disallow_arithmetic_operators {
            Self::check_undefined_short_circuit(expr, ErrorType::Arithmetic, ctx);
        }
    }

    fn check_undefined_short_circuit<'a>(
        expr: &Expression<'a>,
        error_type: ErrorType,
        ctx: &LintContext<'a>,
    ) {
        match expr.get_inner_expression() {
            Expression::LogicalExpression(expr) => match expr.operator {
                LogicalOperator::Or | LogicalOperator::Coalesce => {
                    Self::check_undefined_short_circuit(&expr.right, error_type, ctx);
                }
                LogicalOperator::And => {
                    Self::check_undefined_short_circuit(&expr.left, error_type, ctx);
                    Self::check_undefined_short_circuit(&expr.right, error_type, ctx);
                }
            },
            Expression::AwaitExpression(expr) => {
                Self::check_undefined_short_circuit(&expr.argument, error_type, ctx);
            }
            Expression::ConditionalExpression(expr) => {
                Self::check_undefined_short_circuit(&expr.consequent, error_type, ctx);
                Self::check_undefined_short_circuit(&expr.alternate, error_type, ctx);
            }
            Expression::SequenceExpression(expr) => {
                if let Some(expr) = expr.expressions.iter().last() {
                    Self::check_undefined_short_circuit(expr, error_type, ctx);
                }
            }
            Expression::ChainExpression(expr) => {
                match error_type {
                    ErrorType::Usage => {
                        ctx.diagnostic(no_unsafe_optional_chaining_diagnostic(expr.span));
                    }
                    ErrorType::Arithmetic => {
                        ctx.diagnostic(no_unsafe_arithmetic_diagnostic(expr.span));
                    }
                };
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo;", None),
        ("class Foo {}", None),
        ("!!obj?.foo", None),
        ("obj?.foo();", None),
        ("obj?.foo?.();", None),
        ("(obj?.foo ?? bar)();", None),
        ("(obj?.foo)?.()", None),
        ("(obj?.foo ?? bar?.baz)?.()", None),
        ("(obj.foo)?.();", None),
        ("obj?.foo.bar;", None),
        ("obj?.foo?.bar;", None),
        ("(obj?.foo)?.bar;", None),
        ("(obj?.foo)?.bar.baz;", None),
        ("(obj?.foo)?.().bar", None),
        ("(obj?.foo ?? bar).baz;", None),
        ("(obj?.foo ?? val)`template`", None),
        ("new (obj?.foo ?? val)()", None),
        ("new bar();", None),
        ("obj?.foo?.()();", None),
        ("const {foo} = obj?.baz || {};", None),
        ("const foo = obj?.bar", None),
        ("foo = obj?.bar", None),
        ("foo.bar = obj?.bar", None),
        ("bar(...obj?.foo ?? []);", None),
        ("var bar = {...foo?.bar};", None),
        ("foo?.bar in {};", None),
        ("foo?.bar < foo?.baz;", None),
        ("foo?.bar <= foo?.baz;", None),
        ("foo?.bar > foo?.baz;", None),
        ("foo?.bar >= foo?.baz;", None),
        ("[foo = obj?.bar] = [];", None),
        ("[foo.bar = obj?.bar] = [];", None),
        ("({foo = obj?.bar} = obj);", None),
        ("({foo: obj.bar = obj?.baz} = obj);", None),
        ("(foo?.bar, bar)();", None),
        ("(foo?.bar ? baz : qux)();", None),
        (
            "\n        async function func() {\n          await obj?.foo();\n          await obj?.foo?.();\n          (await obj?.foo)?.();\n          (await obj?.foo)?.bar;\n          await bar?.baz;\n          await (foo ?? obj?.foo.baz);\n          (await bar?.baz ?? bar).baz;\n          (await bar?.baz ?? await bar).baz;\n          await (foo?.bar ? baz : qux);\n        }\n        ",
            None,
        ),
        ("(obj?.foo ?? bar?.baz ?? qux)();", None),
        ("((obj?.foo ?? bar?.baz) || qux)();", None),
        ("((obj?.foo || bar?.baz) || qux)();", None),
        ("((obj?.foo && bar?.baz) || qux)();", None),
        ("obj?.foo - bar;", None),
        ("obj?.foo + bar;", None),
        ("obj?.foo * bar;", None),
        ("obj?.foo / bar;", None),
        ("obj?.foo % bar;", None),
        ("obj?.foo ** bar;", None),
        ("+obj?.foo;", None),
        ("-obj?.foo;", None),
        ("bar += obj?.foo;", None),
        ("bar -= obj?.foo;", None),
        ("bar %= obj?.foo;", None),
        ("bar **= obj?.foo;", None),
        ("bar *= obj?.boo", None),
        ("bar /= obj?.boo", None),
        (
            "async function func() {\n            await obj?.foo + await obj?.bar;\n            await obj?.foo - await obj?.bar;\n            await obj?.foo * await obj?.bar;\n            +await obj?.foo;\n            -await obj?.foo;\n            bar += await obj?.foo;\n            bar -= await obj?.foo;\n            bar %= await obj?.foo;\n            bar **= await obj?.foo;\n            bar *= await obj?.boo;\n            bar /= await obj?.boo;\n        }\n        ",
            None,
        ),
        ("obj?.foo - bar;", Some(serde_json::json!([{}]))),
        (
            "obj?.foo - bar;",
            Some(serde_json::json!([{
                "disallowArithmeticOperators": false
            }])),
        ),
    ];

    let fail = vec![
        ("(obj?.foo && obj?.baz).bar", None),
        ("with (obj?.foo) {};", None),
        ("async function foo() { with ( await obj?.foo) {}; }", None),
        ("(foo ? obj?.foo : obj?.bar).bar", None),
    ];

    Tester::new(NoUnsafeOptionalChaining::NAME, pass, fail).test_and_snapshot();
}
