use oxc_ast::{
    AstKind,
    ast::{AssignmentTarget, match_assignment_target_pattern},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;
use oxc_syntax::operator::LogicalOperator;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

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

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoUnsafeOptionalChaining {
    /// Disallow arithmetic operations on optional chaining expressions.
    /// If this is true, this rule warns arithmetic operations on optional chaining expressions, which possibly result in NaN.
    disallow_arithmetic_operators: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow use of optional chaining in contexts where the `undefined` value is not allowed.
    ///
    /// ### Why is this bad?
    ///
    /// The optional chaining (`?.`) expression can short-circuit with a return value of `undefined`.
    /// Therefore, treating an evaluated optional chaining expression as a function, object, number, etc.,
    /// can cause TypeError or unexpected results. For example:
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var obj = undefined;
    /// 1 in obj?.foo;  // TypeError
    /// with (obj?.foo);  // TypeError
    /// for (bar of obj?.foo);  // TypeError
    /// bar instanceof obj?.foo;  // TypeError
    /// const { bar } = obj?.foo;  // TypeError
    /// ```
    NoUnsafeOptionalChaining,
    eslint,
    correctness,
    config = NoUnsafeOptionalChaining,
    version = "0.0.5",
);

impl Rule for NoUnsafeOptionalChaining {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ChainExpression(chain) = node.kind() else {
            return;
        };

        match self.unsafe_context(node.id(), ctx) {
            Some(ErrorType::Usage) => {
                ctx.diagnostic(no_unsafe_optional_chaining_diagnostic(chain.span));
            }
            Some(ErrorType::Arithmetic) => {
                ctx.diagnostic(no_unsafe_arithmetic_diagnostic(chain.span));
            }
            None => {}
        }
    }
}

#[derive(Clone, Copy)]
enum ErrorType {
    Usage,
    Arithmetic,
}

impl NoUnsafeOptionalChaining {
    fn unsafe_context(&self, chain_id: NodeId, ctx: &LintContext<'_>) -> Option<ErrorType> {
        let mut current_id = chain_id;

        for parent in ctx.nodes().ancestors(chain_id) {
            let parent_id = parent.id();
            match parent.kind() {
                AstKind::ParenthesizedExpression(expr)
                    if expr.expression.node_id() == current_id =>
                {
                    current_id = parent_id;
                }
                AstKind::TSAsExpression(expr) if expr.expression.node_id() == current_id => {
                    current_id = parent_id;
                }
                AstKind::TSSatisfiesExpression(expr) if expr.expression.node_id() == current_id => {
                    current_id = parent_id;
                }
                AstKind::TSTypeAssertion(expr) if expr.expression.node_id() == current_id => {
                    current_id = parent_id;
                }
                AstKind::TSNonNullExpression(expr) if expr.expression.node_id() == current_id => {
                    current_id = parent_id;
                }
                AstKind::TSInstantiationExpression(expr)
                    if expr.expression.node_id() == current_id =>
                {
                    current_id = parent_id;
                }
                AstKind::AwaitExpression(expr) if expr.argument.node_id() == current_id => {
                    current_id = parent_id;
                }
                AstKind::LogicalExpression(expr) => {
                    let propagates_short_circuit = match expr.operator {
                        LogicalOperator::And => {
                            expr.left.node_id() == current_id || expr.right.node_id() == current_id
                        }
                        LogicalOperator::Or | LogicalOperator::Coalesce => {
                            expr.right.node_id() == current_id
                        }
                    };
                    if !propagates_short_circuit {
                        return None;
                    }
                    current_id = parent_id;
                }
                AstKind::ConditionalExpression(expr)
                    if expr.consequent.node_id() == current_id
                        || expr.alternate.node_id() == current_id =>
                {
                    current_id = parent_id;
                }
                AstKind::SequenceExpression(expr)
                    if expr.expressions.last().is_some_and(|expr| expr.node_id() == current_id) =>
                {
                    current_id = parent_id;
                }
                AstKind::CallExpression(expr)
                    if !expr.optional && expr.callee.node_id() == current_id =>
                {
                    return Some(ErrorType::Usage);
                }
                AstKind::StaticMemberExpression(expr)
                    if !expr.optional && expr.object.node_id() == current_id =>
                {
                    return Some(ErrorType::Usage);
                }
                AstKind::ComputedMemberExpression(expr)
                    if !expr.optional && expr.object.node_id() == current_id =>
                {
                    return Some(ErrorType::Usage);
                }
                AstKind::PrivateFieldExpression(expr)
                    if !expr.optional && expr.object.node_id() == current_id =>
                {
                    return Some(ErrorType::Usage);
                }
                AstKind::TaggedTemplateExpression(expr) if expr.tag.node_id() == current_id => {
                    return Some(ErrorType::Usage);
                }
                AstKind::NewExpression(expr) if expr.callee.node_id() == current_id => {
                    return Some(ErrorType::Usage);
                }
                AstKind::AssignmentExpression(expr) if expr.right.node_id() == current_id => {
                    if matches!(expr.left, match_assignment_target_pattern!(AssignmentTarget)) {
                        return Some(ErrorType::Usage);
                    }
                    if self.disallow_arithmetic_operators && expr.operator.is_arithmetic() {
                        return Some(ErrorType::Arithmetic);
                    }
                    return None;
                }
                AstKind::BinaryExpression(expr) => {
                    if expr.operator.is_relational() && expr.right.node_id() == current_id {
                        return Some(ErrorType::Usage);
                    }
                    if self.disallow_arithmetic_operators
                        && expr.operator.is_arithmetic()
                        && (expr.left.node_id() == current_id || expr.right.node_id() == current_id)
                    {
                        return Some(ErrorType::Arithmetic);
                    }
                    return None;
                }
                AstKind::UnaryExpression(expr)
                    if self.disallow_arithmetic_operators
                        && expr.operator.is_arithmetic()
                        && expr.argument.node_id() == current_id =>
                {
                    return Some(ErrorType::Arithmetic);
                }
                AstKind::ForOfStatement(stmt) if stmt.right.node_id() == current_id => {
                    return Some(ErrorType::Usage);
                }
                AstKind::WithStatement(stmt) if stmt.object.node_id() == current_id => {
                    return Some(ErrorType::Usage);
                }
                AstKind::Class(class)
                    if class
                        .super_class
                        .as_ref()
                        .is_some_and(|expr| expr.node_id() == current_id) =>
                {
                    return Some(ErrorType::Usage);
                }
                AstKind::AssignmentPattern(pat)
                    if pat.left.is_destructuring_pattern() && pat.right.node_id() == current_id =>
                {
                    return Some(ErrorType::Usage);
                }
                AstKind::VariableDeclarator(decl)
                    if decl.id.is_destructuring_pattern()
                        && decl.init.as_ref().is_some_and(|expr| expr.node_id() == current_id) =>
                {
                    return Some(ErrorType::Usage);
                }
                AstKind::AssignmentTargetWithDefault(target)
                    if matches!(
                        target.binding,
                        match_assignment_target_pattern!(AssignmentTarget)
                    ) && target.init.node_id() == current_id =>
                {
                    return Some(ErrorType::Usage);
                }
                AstKind::SpreadElement(spread)
                    if spread.argument.node_id() == current_id
                        && matches!(
                            ctx.nodes().parent_kind(parent_id),
                            AstKind::ArrayExpression(_)
                        ) =>
                {
                    return Some(ErrorType::Usage);
                }
                _ => return None,
            }
        }

        None
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
        ("(obj?.foo || bar).baz;", None),
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
        ("x?.f<T>();", None),
        ("x?.f?.<T>();", None),
        ("f?.<Q>();", None),
        ("a?.c?.b<c>", None),
        ("const baz = {...obj?.foo };", None),
    ];

    let fail = vec![
        ("(obj?.foo).bar", None),
        ("(obj?.foo)();", None),
        ("new (obj?.foo)();", None),
        ("(obj?.foo)`template`", None),
        ("class A extends obj?.foo {}", None),
        ("const {foo} = obj?.bar;", None),
        ("({foo} = obj?.bar);", None),
        ("foo in obj?.bar;", None),
        ("for (foo of obj?.bar) {}", None),
        ("(obj?.foo && obj?.baz).bar", None),
        ("with (obj?.foo) {};", None),
        ("async function foo() { with ( await obj?.foo) {}; }", None),
        ("(foo ? obj?.foo : obj?.bar).bar", None),
        ("const a = [...obj?.foo];", None),
        ("const b = [...c, ...obj?.foo];", None),
        ("const s = [], t = [...obj?.foo];", None),
        ("const c = () => ([...(obj?.foo)]);", None),
        ("bar + obj?.foo;", Some(serde_json::json!([{ "disallowArithmeticOperators": true }]))),
    ];

    Tester::new(NoUnsafeOptionalChaining::NAME, NoUnsafeOptionalChaining::PLUGIN, pass, fail)
        .test_and_snapshot();
}
