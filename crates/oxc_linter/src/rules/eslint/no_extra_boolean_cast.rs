use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, NewExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    operator::{LogicalOperator, UnaryOperator},
    precedence::Precedence,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::get_precedence,
};

fn no_extra_double_negation_cast_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Redundant double negation")
        .with_help("Remove the double negation as it will already be coerced to a boolean")
        .with_label(span)
}

fn no_extra_boolean_cast_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Redundant Boolean call")
        .with_help("Remove the Boolean call as it will already be coerced to a boolean")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoExtraBooleanCast {
    /// when set to `true`, in addition to checking default contexts, checks
    /// whether extra boolean casts are present in expressions whose result is
    /// used in a boolean context. See examples below. Default is `false`,
    /// meaning that this rule by default does not warn about extra booleans
    /// cast inside inner expressions.
    #[serde(alias = "enforceForLogicalOperands")]
    pub enforce_for_inner_expressions: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows unnecessary boolean casts.
    ///
    /// ### Why is this bad?
    ///
    /// In contexts such as an if statement's test where the result of the expression will already be coerced to a Boolean,
    /// casting to a Boolean via double negation (`!!`) or a `Boolean` call is unnecessary.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var foo = !!!bar;
    /// var foo = Boolean(!!bar);
    ///
    /// if (!!foo) {}
    /// if (Boolean(foo)) {}
    ///
    /// // with "enforceForInnerExpressions" option enabled
    /// if (!!foo || bar) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// var foo = !bar;
    /// var foo = Boolean(bar);
    ///
    /// if (foo) {}
    /// if (foo) {}
    ///
    /// // with "enforceForInnerExpressions" option enabled
    /// if (foo || bar) {}
    /// ```
    NoExtraBooleanCast,
    eslint,
    correctness,
    conditional_fix_or_conditional_suggestion,
    config = NoExtraBooleanCast,
);

impl Rule for NoExtraBooleanCast {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(expr)
                if expr.callee.is_specific_id("Boolean")
                    && is_flagged_ctx(node, ctx, self.enforce_for_inner_expressions) =>
            {
                ctx.diagnostic_with_fix(no_extra_boolean_cast_diagnostic(expr.span), |fixer| {
                    if expr.arguments.len() != 1 {
                        return fixer.noop();
                    }
                    let Some(arg) = expr.arguments[0]
                        .as_expression()
                        .map(remove_double_not)
                        .map(Expression::without_parentheses)
                    else {
                        return fixer.noop();
                    };

                    // When the parent is a unary `!` and the argument has lower precedence than
                    // unary negation, we need to wrap in parentheses to preserve correct semantics.
                    // e.g., `!Boolean(a ?? b)` should become `!(a ?? b)`, not `!a ?? b`
                    // e.g., `!Boolean(a = b)` should become `!(a = b)`, not `!a = b`
                    // e.g., `!Boolean(a ? b : c)` should become `!(a ? b : c)`, not `!a ? b : c`
                    let parent = get_real_parent(node, ctx);
                    let needs_parens = parent.is_some_and(|p| is_unary_negation(p))
                        && get_precedence(arg).is_some_and(|p| p < Precedence::Prefix);

                    if needs_parens {
                        let arg_text = fixer.source_range(arg.span());
                        fixer.replace(expr.span, format!("({arg_text})"))
                    } else {
                        fixer.replace_with(expr, arg)
                    }
                });
            }
            AstKind::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
                let Some(parent) = get_real_parent(node, ctx) else {
                    return;
                };
                match parent.kind() {
                    AstKind::UnaryExpression(p)
                        if p.operator == UnaryOperator::LogicalNot
                            && is_flagged_ctx(parent, ctx, self.enforce_for_inner_expressions) =>
                    {
                        ctx.diagnostic_with_suggestion(
                            no_extra_double_negation_cast_diagnostic(parent.kind().span()),
                            |fixer| fixer.replace_with(p, &unary.argument),
                        );
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

// Checks whether the node is a context that should report an error
// Acts recursively if it is in a logical context
fn is_flagged_ctx(node: &AstNode, ctx: &LintContext, enforce_for_logical_operands: bool) -> bool {
    let parent = get_real_parent(node, ctx);
    if is_bool_context(node, parent, ctx) {
        return true;
    }

    parent.is_some_and(|parent| {
        is_logical_ctx(parent, enforce_for_logical_operands)
            && is_flagged_ctx(parent, ctx, enforce_for_logical_operands)
    })
}

// Check if a node is in a context where its value would be coerced to a boolean at runtime
fn is_bool_context(node: &AstNode, parent: Option<&AstNode>, ctx: &LintContext) -> bool {
    parent.is_some_and(|parent| {
        (is_bool_fn_or_constructor_call(parent) && is_first_arg(node, parent))
            || is_inside_test_condition(node, ctx)
            || is_unary_negation(parent)
    })
}

// Checks whether the node is a logical expression and that the option is enabled
fn is_logical_ctx(node: &AstNode, enforce_for_logical_operands: bool) -> bool {
    match node.kind() {
        AstKind::LogicalExpression(expr) => {
            (expr.operator == LogicalOperator::And || expr.operator == LogicalOperator::Or)
                && enforce_for_logical_operands
        }
        _ => false,
    }
}

fn is_bool_fn_or_constructor_call(node: &AstNode) -> bool {
    match node.kind() {
        AstKind::CallExpression(CallExpression { callee, .. })
        | AstKind::NewExpression(NewExpression { callee, .. }) => callee.is_specific_id("Boolean"),
        _ => false,
    }
}

fn is_first_arg(node: &AstNode, parent: &AstNode) -> bool {
    match parent.kind() {
        AstKind::CallExpression(CallExpression { arguments, .. })
        | AstKind::NewExpression(NewExpression { arguments, .. }) => {
            arguments.first().is_some_and(|arg| {
                arg.as_expression()
                    .is_some_and(|expr| expr.without_parentheses().span() == node.kind().span())
            })
        }
        _ => false,
    }
}

fn is_inside_test_condition(node: &AstNode, ctx: &LintContext) -> bool {
    get_real_parent(node, ctx).is_some_and(|parent| match parent.kind() {
        AstKind::IfStatement(stmt) => is_same_test_condition(&stmt.test, node),
        AstKind::DoWhileStatement(stmt) => is_same_test_condition(&stmt.test, node),
        AstKind::WhileStatement(stmt) => is_same_test_condition(&stmt.test, node),
        AstKind::ConditionalExpression(stmt) => is_same_test_condition(&stmt.test, node),
        AstKind::ForStatement(stmt) => {
            stmt.test.as_ref().is_some_and(|test| is_same_test_condition(test, node))
        }
        _ => false,
    })
}
fn is_same_test_condition<'a>(test: &Expression<'a>, node: &AstNode<'a>) -> bool {
    test.get_inner_expression().without_parentheses().span() == node.kind().span()
}

fn is_unary_negation(node: &AstNode) -> bool {
    match node.kind() {
        AstKind::UnaryExpression(expr) => expr.operator == UnaryOperator::LogicalNot,
        _ => false,
    }
}

fn get_real_parent<'a, 'b>(node: &AstNode, ctx: &'a LintContext<'b>) -> Option<&'a AstNode<'b>> {
    ctx.nodes().ancestors(node.id()).find(|parent| {
        !matches!(parent.kind(), AstKind::ParenthesizedExpression(_) | AstKind::ChainExpression(_))
    })
}

/// Remove `!!` from `expr` if present.
fn remove_double_not<'a, 'b>(expr: &'b Expression<'a>) -> &'b Expression<'a> {
    without_not(expr).and_then(|inner| without_not(inner)).unwrap_or(expr)
}

fn without_not<'a, 'b>(expr: &'b Expression<'a>) -> Option<&'b Expression<'a>> {
    match expr.without_parentheses() {
        Expression::UnaryExpression(expr) if expr.operator == UnaryOperator::LogicalNot => {
            Some(&expr.argument)
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("Boolean(bar, !!baz);", None),
        ("var foo = !!bar;", None),
        ("function foo() { return !!bar; }", None),
        ("var foo = bar() ? !!baz : !!bat", None),
        ("for(!!foo;;) {}", None),
        ("for(;; !!foo) {}", None),
        ("var foo = Boolean(bar);", None),
        ("function foo() { return Boolean(bar); }", None),
        ("var foo = bar() ? Boolean(baz) : Boolean(bat)", None),
        ("for(Boolean(foo);;) {}", None),
        ("for(;; Boolean(foo)) {}", None),
        ("if (new Boolean(foo)) {}", None),
        ("if ((Boolean(1), 2)) {}", None),
        (
            "var foo = bar || !!baz",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "var foo = bar && !!baz",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "var foo = bar || (baz && !!bat)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "function foo() { return (!!bar || baz); }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "var foo = bar() ? (!!baz && bat) : (!!bat && qux)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "for(!!(foo && bar);;) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "for(;; !!(foo || bar)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "var foo = Boolean(bar) || baz;",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "var foo = bar || Boolean(baz);",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "var foo = Boolean(bar) || Boolean(baz);",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "function foo() { return (Boolean(bar) || baz); }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "var foo = bar() ? Boolean(baz) || bat : Boolean(bat)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "for(Boolean(foo) || bar;;) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "for(;; Boolean(foo) || bar) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (new Boolean(foo) || bar) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        ("if (!!foo || bar) {}", None),
        ("if (!!foo || bar) {}", Some(serde_json::json!([{}]))),
        ("if (!!foo || bar) {}", Some(serde_json::json!([{ "enforceForLogicalOperands": false }]))),
        (
            "if ((!!foo || bar) === baz) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        ("if (!!foo ?? bar) {}", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))), // { "ecmaVersion": 2020 },
        (
            "var foo = bar || !!baz",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "var foo = bar && !!baz",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "var foo = bar || (baz && !!bat)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "function foo() { return (!!bar || baz); }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "var foo = bar() ? (!!baz && bat) : (!!bat && qux)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "for(!!(foo && bar);;) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "for(;; !!(foo || bar)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "var foo = Boolean(bar) || baz;",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "var foo = bar || Boolean(baz);",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "var foo = Boolean(bar) || Boolean(baz);",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "function foo() { return (Boolean(bar) || baz); }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "var foo = bar() ? Boolean(baz) || bat : Boolean(bat)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "for(Boolean(foo) || bar;;) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "for(;; Boolean(foo) || bar) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (new Boolean(foo) || bar) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!foo || bar) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": false }])),
        ),
        (
            "if ((!!foo || bar) === baz) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        ("if (!!foo ?? bar) {}", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))), // { "ecmaVersion": 2020 },
        (
            "if ((1, Boolean(2), 3)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
    ];

    let fail = vec![
        ("if (!!foo) {}", None),
        ("do {} while (!!foo)", None),
        ("while (!!foo) {}", None),
        ("!!foo ? bar : baz", None),
        ("for (; !!foo;) {}", None),
        ("!!!foo", None),
        ("Boolean(!!foo)", None),
        ("new Boolean(!!foo)", None),
        ("if (Boolean(foo)) {}", None),
        ("do {} while (Boolean(foo))", None),
        ("while (Boolean(foo)) {}", None),
        ("Boolean(foo) ? bar : baz", None),
        ("for (; Boolean(foo);) {}", None),
        ("!Boolean(foo)", None),
        ("!Boolean(foo && bar)", None),
        ("!Boolean(foo + bar)", None),
        ("!Boolean(+foo)", None),
        ("!Boolean(foo())", None),
        ("!Boolean(foo = bar)", None),
        ("!Boolean(...foo);", None), // { "ecmaVersion": 2015 },
        ("!Boolean(foo, bar());", None),
        ("!Boolean((foo, bar()));", None),
        ("!Boolean();", None),
        ("!(Boolean());", None),
        ("if (!Boolean()) { foo() }", None),
        ("while (!Boolean()) { foo() }", None),
        ("var foo = Boolean() ? bar() : baz()", None),
        ("if (Boolean()) { foo() }", None),
        ("while (Boolean()) { foo() }", None),
        ("Boolean(Boolean(foo))", None),
        ("Boolean(!!foo, bar)", None),
        ("function *foo() { yield!!a ? b : c }", None), // { "ecmaVersion": 2015 },
        ("function *foo() { yield!! a ? b : c }", None), // { "ecmaVersion": 2015 },
        ("function *foo() { yield! !a ? b : c }", None), // { "ecmaVersion": 2015 },
        ("function *foo() { yield !!a ? b : c }", None), // { "ecmaVersion": 2015 },
        ("function *foo() { yield(!!a) ? b : c }", None), // { "ecmaVersion": 2015 },
        ("function *foo() { yield/**/!!a ? b : c }", None), // { "ecmaVersion": 2015 },
        ("x=!!a ? b : c ", None),
        ("void!Boolean()", None),
        ("void! Boolean()", None),
        ("typeof!Boolean()", None),
        ("(!Boolean())", None),
        ("+!Boolean()", None),
        ("void !Boolean()", None),
        ("void(!Boolean())", None),
        ("void/**/!Boolean()", None),
        ("!/**/!!foo", None),
        ("!!/**/!foo", None),
        ("!!!/**/foo", None),
        ("!!!foo/**/", None),
        ("if(!/**/!foo);", None),
        ("(!!/**/foo ? 1 : 2)", None),
        ("!/**/Boolean(foo)", None),
        ("!Boolean/**/(foo)", None),
        ("!Boolean(/**/foo)", None),
        ("!Boolean(foo/**/)", None),
        ("!Boolean(foo)/**/", None),
        ("if(Boolean/**/(foo));", None),
        ("(Boolean(foo/**/) ? 1 : 2)", None),
        ("/**/!Boolean()", None),
        ("!/**/Boolean()", None),
        ("!Boolean/**/()", None),
        ("!Boolean(/**/)", None),
        ("!Boolean()/**/", None),
        ("if(!/**/Boolean());", None),
        ("(!Boolean(/**/) ? 1 : 2)", None),
        ("if(/**/Boolean());", None),
        ("if(Boolean/**/());", None),
        ("if(Boolean(/**/));", None),
        ("if(Boolean()/**/);", None),
        ("(Boolean/**/() ? 1 : 2)", None),
        ("if (!!foo || bar) {}", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        ("if (!!foo && bar) {}", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        (
            "if ((!!foo || bar) && bat) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        ("if (foo && !!bar) {}", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        (
            "do {} while (!!foo || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "while (!!foo || bar) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!!foo && bat ? bar : baz",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "for (; !!foo || bar;) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        ("!!!foo || bar", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        ("Boolean(!!foo || bar)", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        (
            "new Boolean(!!foo || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean(foo) || bar) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "do {} while (Boolean(foo) || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "while (Boolean(foo) || bar) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "Boolean(foo) || bat ? bar : baz",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "for (; Boolean(foo) || bar;) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        ("!Boolean(foo) || bar", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        (
            "!Boolean(foo && bar) || bat",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(foo + bar) || bat",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(+foo)  || bar",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(foo()) || bar",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(foo() || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(foo = bar) || bat",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(...foo) || bar;",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "!Boolean(foo, bar()) || bar;",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean((foo, bar()) || bat);",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        ("!Boolean() || bar;", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        ("!(Boolean()) || bar;", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        (
            "if (!Boolean() || bar) { foo() }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "while (!Boolean() || bar) { foo() }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "var foo = Boolean() || bar ? bar() : baz()",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean() || bar) { foo() }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "while (Boolean() || bar) { foo() }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "function *foo() { yield(!!a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "function *foo() { yield(!! a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "function *foo() { yield(! !a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "function *foo() { yield (!!a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "function *foo() { yield/**/(!!a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ), // { "ecmaVersion": 2015 },
        ("x=!!a || d ? b : c ", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        (
            "void(!Boolean() || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "void(! Boolean() || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "typeof(!Boolean() || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        ("(!Boolean() || bar)", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        (
            "void/**/(!Boolean() || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        ("!/**/(!!foo || bar)", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        ("!!/**/!foo || bar", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        ("!!!/**/foo || bar", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        ("!(!!foo || bar)/**/", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        ("if(!/**/!foo || bar);", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        (
            "(!!/**/foo || bar ? 1 : 2)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!/**/(Boolean(foo) || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean/**/(foo) || bar",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(/**/foo) || bar",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(foo/**/) || bar",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!(Boolean(foo)|| bar)/**/",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if(Boolean/**/(foo) || bar);",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "(Boolean(foo/**/)|| bar ? 1 : 2)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        ("/**/!Boolean()|| bar", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        ("!/**/Boolean()|| bar", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        ("!Boolean/**/()|| bar", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        ("!Boolean(/**/)|| bar", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
        (
            "(!Boolean()|| bar)/**/",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if(!/**/Boolean()|| bar);",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "(!Boolean(/**/) || bar ? 1 : 2)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if(/**/Boolean()|| bar);",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if(Boolean/**/()|| bar);",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if(Boolean(/**/)|| bar);",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if(Boolean()|| bar/**/);",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "(Boolean/**/()|| bar ? 1 : 2)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (a && !!(b ? c : d)){}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "function *foo() { yield!!a || d ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ), // { "ecmaVersion": 6 },
        ("if (!!foo || bar) {}", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        ("if (!!foo && bar) {}", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        (
            "if ((!!foo || bar) && bat) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        ("if (foo && !!bar) {}", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        (
            "do {} while (!!foo || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "while (!!foo || bar) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!!foo && bat ? bar : baz",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "for (; !!foo || bar;) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        ("!!!foo || bar", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        (
            "Boolean(!!foo || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "new Boolean(!!foo || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean(foo) || bar) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "do {} while (Boolean(foo) || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "while (Boolean(foo) || bar) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "Boolean(foo) || bat ? bar : baz",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "for (; Boolean(foo) || bar;) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        ("!Boolean(foo) || bar", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        (
            "!Boolean(foo && bar) || bat",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(foo + bar) || bat",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(+foo)  || bar",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(foo()) || bar",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(foo() || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(foo = bar) || bat",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(...foo) || bar;",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "!Boolean(foo, bar()) || bar;",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean((foo, bar()) || bat);",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        ("!Boolean() || bar;", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        ("!(Boolean()) || bar;", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        (
            "if (!Boolean() || bar) { foo() }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "while (!Boolean() || bar) { foo() }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "var foo = Boolean() || bar ? bar() : baz()",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean() || bar) { foo() }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "while (Boolean() || bar) { foo() }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "function *foo() { yield(!!a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "function *foo() { yield(!! a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "function *foo() { yield(! !a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "function *foo() { yield (!!a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "function *foo() { yield/**/(!!a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ), // { "ecmaVersion": 2015 },
        ("x=!!a || d ? b : c ", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        (
            "void(!Boolean() || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "void(! Boolean() || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "typeof(!Boolean() || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        ("(!Boolean() || bar)", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        (
            "void/**/(!Boolean() || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        ("!/**/(!!foo || bar)", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        ("!!/**/!foo || bar", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        ("!!!/**/foo || bar", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        ("!(!!foo || bar)/**/", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        (
            "if(!/**/!foo || bar);",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "(!!/**/foo || bar ? 1 : 2)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!/**/(Boolean(foo) || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean/**/(foo) || bar",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(/**/foo) || bar",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(foo/**/) || bar",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!(Boolean(foo)|| bar)/**/",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if(Boolean/**/(foo) || bar);",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "(Boolean(foo/**/)|| bar ? 1 : 2)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        ("/**/!Boolean()|| bar", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        ("!/**/Boolean()|| bar", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        ("!Boolean/**/()|| bar", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        ("!Boolean(/**/)|| bar", Some(serde_json::json!([{ "enforceForInnerExpressions": true }]))),
        (
            "(!Boolean()|| bar)/**/",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if(!/**/Boolean()|| bar);",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "(!Boolean(/**/) || bar ? 1 : 2)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if(/**/Boolean()|| bar);",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if(Boolean/**/()|| bar);",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if(Boolean(/**/)|| bar);",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if(Boolean()|| bar/**/);",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "(Boolean/**/()|| bar ? 1 : 2)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (a && !!(b ? c : d)){}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "function *foo() { yield!!a || d ? b : c }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ), // { "ecmaVersion": 6 },
        ("Boolean(!!(a, b))", None),
        ("Boolean(Boolean((a, b)))", None),
        ("Boolean((!!(a, b)))", None),
        ("Boolean((Boolean((a, b))))", None),
        ("Boolean(!(!(a, b)))", None),
        ("Boolean((!(!(a, b))))", None),
        ("Boolean(!!(a = b))", None),
        ("Boolean((!!(a = b)))", None),
        ("Boolean(Boolean(a = b))", None),
        ("Boolean(Boolean((a += b)))", None),
        ("Boolean(!!(a === b))", None),
        ("Boolean(!!((a !== b)))", None),
        ("Boolean(!!a.b)", None),
        ("Boolean(Boolean((a)))", None),
        ("Boolean((!!(a)))", None),
        ("new Boolean(!!(a, b))", None),
        ("new Boolean(Boolean((a, b)))", None),
        ("new Boolean((!!(a, b)))", None),
        ("new Boolean((Boolean((a, b))))", None),
        ("new Boolean(!(!(a, b)))", None),
        ("new Boolean((!(!(a, b))))", None),
        ("new Boolean(!!(a = b))", None),
        ("new Boolean((!!(a = b)))", None),
        ("new Boolean(Boolean(a = b))", None),
        ("new Boolean(Boolean((a += b)))", None),
        ("new Boolean(!!(a === b))", None),
        ("new Boolean(!!((a !== b)))", None),
        ("new Boolean(!!a.b)", None),
        ("new Boolean(Boolean((a)))", None),
        ("new Boolean((!!(a)))", None),
        ("if (!!(a, b));", None),
        ("if (Boolean((a, b)));", None),
        ("if (!(!(a, b)));", None),
        ("if (!!(a = b));", None),
        ("if (Boolean(a = b));", None),
        ("if (!!(a > b));", None),
        ("if (Boolean(a === b));", None),
        ("if (!!f(a));", None),
        ("if (Boolean(f(a)));", None),
        ("if (!!(f(a)));", None),
        ("if ((!!f(a)));", None),
        ("if ((Boolean(f(a))));", None),
        ("if (!!a);", None),
        ("if (Boolean(a));", None),
        ("while (!!(a, b));", None),
        ("while (Boolean((a, b)));", None),
        ("while (!(!(a, b)));", None),
        ("while (!!(a = b));", None),
        ("while (Boolean(a = b));", None),
        ("while (!!(a > b));", None),
        ("while (Boolean(a === b));", None),
        ("while (!!f(a));", None),
        ("while (Boolean(f(a)));", None),
        ("while (!!(f(a)));", None),
        ("while ((!!f(a)));", None),
        ("while ((Boolean(f(a))));", None),
        ("while (!!a);", None),
        ("while (Boolean(a));", None),
        ("do {} while (!!(a, b));", None),
        ("do {} while (Boolean((a, b)));", None),
        ("do {} while (!(!(a, b)));", None),
        ("do {} while (!!(a = b));", None),
        ("do {} while (Boolean(a = b));", None),
        ("do {} while (!!(a > b));", None),
        ("do {} while (Boolean(a === b));", None),
        ("do {} while (!!f(a));", None),
        ("do {} while (Boolean(f(a)));", None),
        ("do {} while (!!(f(a)));", None),
        ("do {} while ((!!f(a)));", None),
        ("do {} while ((Boolean(f(a))));", None),
        ("do {} while (!!a);", None),
        ("do {} while (Boolean(a));", None),
        ("for (; !!(a, b););", None),
        ("for (; Boolean((a, b)););", None),
        ("for (; !(!(a, b)););", None),
        ("for (; !!(a = b););", None),
        ("for (; Boolean(a = b););", None),
        ("for (; !!(a > b););", None),
        ("for (; Boolean(a === b););", None),
        ("for (; !!f(a););", None),
        ("for (; Boolean(f(a)););", None),
        ("for (; !!(f(a)););", None),
        ("for (; (!!f(a)););", None),
        ("for (; (Boolean(f(a))););", None),
        ("for (; !!a;);", None),
        ("for (; Boolean(a););", None),
        ("!!(a, b) ? c : d", None),
        ("(!!(a, b)) ? c : d", None),
        ("Boolean((a, b)) ? c : d", None),
        ("!!(a = b) ? c : d", None),
        ("Boolean(a -= b) ? c : d", None),
        ("(Boolean((a *= b))) ? c : d", None),
        ("!!(a ? b : c) ? d : e", None),
        ("Boolean(a ? b : c) ? d : e", None),
        ("!!(a || b) ? c : d", None),
        ("Boolean(a && b) ? c : d", None),
        ("!!(a === b) ? c : d", None),
        ("Boolean(a < b) ? c : d", None),
        ("!!((a !== b)) ? c : d", None),
        ("Boolean((a >= b)) ? c : d", None),
        ("!!+a ? b : c", None),
        ("!!+(a) ? b : c", None),
        ("Boolean(!a) ? b : c", None),
        ("!!f(a) ? b : c", None),
        ("(!!f(a)) ? b : c", None),
        ("Boolean(a.b) ? c : d", None),
        ("!!a ? b : c", None),
        ("Boolean(a) ? b : c", None),
        ("!!!(a, b)", None),
        ("!Boolean((a, b))", None),
        ("!!!(a = b)", None),
        ("!!(!(a += b))", None),
        ("!(!!(a += b))", None),
        ("!Boolean(a -= b)", None),
        ("!Boolean((a -= b))", None),
        ("!(Boolean(a -= b))", None),
        ("!!!(a || b)", None),
        ("!Boolean(a || b)", None),
        ("!!!(a && b)", None),
        ("!Boolean(a && b)", None),
        ("!!!(a != b)", None),
        ("!!!(a === b)", None),
        ("var x = !Boolean(a > b)", None),
        ("!!!(a - b)", None),
        ("!!!(a ** b)", None),      // { "ecmaVersion": 2016 },
        ("!Boolean(a ** b)", None), // { "ecmaVersion": 2016 },
        ("async function f() { !!!(await a) }", None), // { "ecmaVersion": 2017 },
        ("async function f() { !Boolean(await a) }", None), // { "ecmaVersion": 2017 },
        ("!!!!a", None),
        ("!!(!(!a))", None),
        ("!Boolean(!a)", None),
        ("!Boolean((!a))", None),
        ("!Boolean(!(a))", None),
        ("!(Boolean(!a))", None),
        ("!!!+a", None),
        ("!!!(+a)", None),
        ("!!(!+a)", None),
        ("!(!!+a)", None),
        ("!Boolean((-a))", None),
        ("!Boolean(-(a))", None),
        ("!!!(--a)", None),
        ("!Boolean(a++)", None),
        ("!!!f(a)", None),
        ("!!!(f(a))", None),
        ("!!!a", None),
        ("!Boolean(a)", None),
        (
            "if (!!(a, b) || !!(c, d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean((a, b)) || Boolean((c, d))) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if ((!!((a, b))) || (!!((c, d)))) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (!!(a, b) && !!(c, d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean((a, b)) && Boolean((c, d))) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if ((!!((a, b))) && (!!((c, d)))) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (!!(a = b) || !!(c = d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean(a /= b) || Boolean(c /= d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (!!(a >>= b) && !!(c >>= d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean(a **= b) && Boolean(c **= d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ), // { "ecmaVersion": 2016 },
        (
            "if (!!(a ? b : c) || !!(d ? e : f)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean(a ? b : c) || Boolean(d ? e : f)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (!!(a ? b : c) && !!(d ? e : f)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean(a ? b : c) && Boolean(d ? e : f)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (!!(a || b) || !!(c || d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean(a || b) || Boolean(c || d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (!!(a || b) && !!(c || d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean(a || b) && Boolean(c || d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (!!(a && b) || !!(c && d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean(a && b) || Boolean(c && d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (!!(a && b) && !!(c && d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean(a && b) && Boolean(c && d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (!!(a !== b) || !!(c !== d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean(a != b) || Boolean(c != d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (!!(a === b) && !!(c === d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (!!(a > b) || !!(c < d)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean(!a) || Boolean(+b)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (!!f(a) && !!b.c) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean(a) || !!b) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (!!a && Boolean(b)) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if ((!!a) || (Boolean(b))) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean(a ?? b) || c) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ), // { "ecmaVersion": 2020 },
        ("if (Boolean?.(foo)) {};", None), // { "ecmaVersion": 2020 },
        (
            "if (Boolean?.(a ?? b) || c) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ), // { "ecmaVersion": 2020 },
        ("if (!Boolean(a as any)) { }", None), // {  "parser": require(  parser("typescript-parsers/boolean-cast-with-assertion"),  ),  "ecmaVersion": 2020,  },
        // (
        //     "if ((1, 2, Boolean(3))) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (a ?? Boolean(b)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if ((a, b, c ?? (d, e, f ?? Boolean(g)))) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        (
            "if (!!(a, b) || !!(c, d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean((a, b)) || Boolean((c, d))) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if ((!!((a, b))) || (!!((c, d)))) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!(a, b) && !!(c, d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean((a, b)) && Boolean((c, d))) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if ((!!((a, b))) && (!!((c, d)))) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!(a = b) || !!(c = d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean(a /= b) || Boolean(c /= d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!(a >>= b) && !!(c >>= d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean(a **= b) && Boolean(c **= d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ), // { "ecmaVersion": 2016 },
        (
            "if (!!(a ? b : c) || !!(d ? e : f)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean(a ? b : c) || Boolean(d ? e : f)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!(a ? b : c) && !!(d ? e : f)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean(a ? b : c) && Boolean(d ? e : f)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!(a || b) || !!(c || d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean(a || b) || Boolean(c || d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!(a || b) && !!(c || d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean(a || b) && Boolean(c || d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!(a && b) || !!(c && d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean(a && b) || Boolean(c && d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!(a && b) && !!(c && d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean(a && b) && Boolean(c && d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!(a !== b) || !!(c !== d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean(a != b) || Boolean(c != d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!(a === b) && !!(c === d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!(a > b) || !!(c < d)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean(!a) || Boolean(+b)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!f(a) && !!b.c) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean(a) || !!b) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!a && Boolean(b)) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if ((!!a) || (Boolean(b))) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean(a ?? b) || c) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ), // { "ecmaVersion": 2020 },
           // (
           //     "if (Boolean?.(a ?? b) || c) {}",
           //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
           // ), // { "ecmaVersion": 2020 },
           // (
           //     "if (a ? Boolean(b) : c) {}",
           //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
           // ),
           // (
           //     "if (a ? b : Boolean(c)) {}",
           //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
           // ),
           // (
           //     "if (a ? b : Boolean(c ? d : e)) {}",
           //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
           // ),
           // (
           //     "const ternary = Boolean(bar ? !!baz : bat);",
           //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
           // ),
           // (
           //     "const commaOperator = Boolean((bar, baz, !!bat));",
           //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
           // ),
           // (
           //     "
           // 	for (let i = 0; (console.log(i), Boolean(i < 10)); i++) {
           // 	    // ...
           // 	}",
           //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
           // ),
           // (
           //     "const nullishCoalescingOperator = Boolean(bar ?? Boolean(baz));",
           //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
           // ),
           // (
           //     "if (a ? Boolean(b = c) : Boolean(d = e));",
           //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
           // ),
           // (
           //     "if (a ? Boolean((b, c)) : Boolean((d, e)));",
           //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
           // ),
           // (
           //     "
           // 	function * generator() {
           // 	    if (a ? Boolean(yield y) : x) {
           // 	        return a;
           // 	    };
           // 	}
           // 	",
           //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
           // ),
    ];

    let fix = vec![
        ("if (!!foo) {}", "if (foo) {}", None),
        ("do {} while (!!foo)", "do {} while (foo)", None),
        ("while (!!foo) {}", "while (foo) {}", None),
        ("!!foo ? bar : baz", "foo ? bar : baz", None),
        ("for (; !!foo;) {}", "for (; foo;) {}", None),
        ("!!!foo", "!foo", None),
        ("Boolean(!!foo)", "Boolean(foo)", None),
        ("new Boolean(!!foo)", "new Boolean(foo)", None),
        ("if (Boolean(foo)) {}", "if (foo) {}", None),
        ("do {} while (Boolean(foo))", "do {} while (foo)", None),
        ("while (Boolean(foo)) {}", "while (foo) {}", None),
        ("Boolean(foo) ? bar : baz", "foo ? bar : baz", None),
        ("for (; Boolean(foo);) {}", "for (; foo;) {}", None),
        ("!Boolean(foo)", "!foo", None),
        ("!Boolean(foo && bar)", "!(foo && bar)", None),
        ("!Boolean(foo + bar)", "!(foo + bar)", None),
        ("!Boolean(+foo)", "!+foo", None),
        ("!Boolean(foo())", "!foo()", None),
        ("!Boolean(foo = bar)", "!(foo = bar)", None),
        ("!Boolean((foo, bar()));", "!(foo, bar());", None),
        // ("!Boolean();", "true;", None),
        // ("!(Boolean());", "true;", None),
        // ("if (!Boolean()) { foo() }", "if (true) { foo() }", None),
        // ("while (!Boolean()) { foo() }", "while (true) { foo() }", None),
        // ("var foo = Boolean() ? bar() : baz()", "var foo = false ? bar() : baz()", None),
        // ("if (Boolean()) { foo() }", "if (false) { foo() }", None),
        // ("while (Boolean()) { foo() }", "while (false) { foo() }", None),
        ("Boolean(Boolean(foo))", "Boolean(foo)", None),
        ("Boolean(!!foo, bar)", "Boolean(foo, bar)", None),
        // ("function *foo() { yield!!a ? b : c }", "function *foo() { yield a ? b : c }", None),
        // ("function *foo() { yield!! a ? b : c }", "function *foo() { yield a ? b : c }", None),
        // ("function *foo() { yield! !a ? b : c }", "function *foo() { yield a ? b : c }", None),
        ("function *foo() { yield !!a ? b : c }", "function *foo() { yield a ? b : c }", None),
        ("function *foo() { yield(!!a) ? b : c }", "function *foo() { yield(a) ? b : c }", None),
        (
            "function *foo() { yield/**/!!a ? b : c }",
            "function *foo() { yield/**/a ? b : c }",
            None,
        ),
        ("x=!!a ? b : c ", "x=a ? b : c ", None),
        // ("void!Boolean()", "void true", None),
        // ("void! Boolean()", "void true", None),
        // ("typeof!Boolean()", "typeof true", None),
        // ("(!Boolean())", "(true)", None),
        // ("+!Boolean()", "+true", None),
        // ("void !Boolean()", "void true", None),
        // ("void(!Boolean())", "void(true)", None),
        // ("void/**/!Boolean()", "void/**/true", None),
        ("!/**/!!foo", "!/**/foo", None),
        ("!!!foo/**/", "!foo/**/", None),
        ("!/**/Boolean(foo)", "!/**/foo", None),
        ("!Boolean(foo)/**/", "!foo/**/", None),
        // ("/**/!Boolean()", "/**/true", None),
        // ("!Boolean()/**/", "true/**/", None),
        // ("if(/**/Boolean());", "if(/**/false);", None),
        // ("if(Boolean()/**/);", "if(false/**/);", None),
        (
            "if (!!foo || bar) {}",
            "if (foo || bar) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (!!foo && bar) {}",
            "if (foo && bar) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if ((!!foo || bar) && bat) {}",
            "if ((foo || bar) && bat) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (foo && !!bar) {}",
            "if (foo && bar) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "do {} while (!!foo || bar)",
            "do {} while (foo || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "while (!!foo || bar) {}",
            "while (foo || bar) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!!foo && bat ? bar : baz",
            "foo && bat ? bar : baz",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "for (; !!foo || bar;) {}",
            "for (; foo || bar;) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!!!foo || bar",
            "!foo || bar",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "Boolean(!!foo || bar)",
            "Boolean(foo || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "new Boolean(!!foo || bar)",
            "new Boolean(foo || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "if (Boolean(foo) || bar) {}",
            "if (foo || bar) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "do {} while (Boolean(foo) || bar)",
            "do {} while (foo || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "while (Boolean(foo) || bar) {}",
            "while (foo || bar) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "Boolean(foo) || bat ? bar : baz",
            "foo || bat ? bar : baz",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "for (; Boolean(foo) || bar;) {}",
            "for (; foo || bar;) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(foo) || bar",
            "!foo || bar",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(foo && bar) || bat",
            "!(foo && bar) || bat",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(foo + bar) || bat",
            "!(foo + bar) || bat",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(+foo)  || bar",
            "!+foo  || bar",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(foo()) || bar",
            "!foo() || bar",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(foo() || bar)",
            "!(foo() || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean(foo = bar) || bat",
            "!(foo = bar) || bat",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!Boolean((foo, bar()) || bat);",
            "!((foo, bar()) || bat);",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        // (
        //     "!Boolean() || bar;",
        //     "true || bar;",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "!(Boolean()) || bar;",
        //     "true || bar;",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!Boolean() || bar) { foo() }",
        //     "if (true || bar) { foo() }",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "while (!Boolean() || bar) { foo() }",
        //     "while (true || bar) { foo() }",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "var foo = Boolean() || bar ? bar() : baz()",
        //     "var foo = false || bar ? bar() : baz()",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean() || bar) { foo() }",
        //     "if (false || bar) { foo() }",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "while (Boolean() || bar) { foo() }",
        //     "while (false || bar) { foo() }",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        (
            "function *foo() { yield(!!a || d) ? b : c }",
            "function *foo() { yield(a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "function *foo() { yield(!! a || d) ? b : c }",
            "function *foo() { yield(a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "function *foo() { yield(! !a || d) ? b : c }",
            "function *foo() { yield(a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "function *foo() { yield (!!a || d) ? b : c }",
            "function *foo() { yield (a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "function *foo() { yield/**/(!!a || d) ? b : c }",
            "function *foo() { yield/**/(a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "x=!!a || d ? b : c ",
            "x=a || d ? b : c ",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        // (
        //     "void(!Boolean() || bar)",
        //     "void(true || bar)",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "void(! Boolean() || bar)",
        //     "void(true || bar)",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "typeof(!Boolean() || bar)",
        //     "typeof(true || bar)",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "(!Boolean() || bar)",
        //     "(true || bar)",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "void/**/(!Boolean() || bar)",
        //     "void/**/(true || bar)",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        (
            "!/**/(!!foo || bar)",
            "!/**/(foo || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!(!!foo || bar)/**/",
            "!(foo || bar)/**/",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!/**/(Boolean(foo) || bar)",
            "!/**/(foo || bar)",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "!(Boolean(foo)|| bar)/**/",
            "!(foo|| bar)/**/",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        // (
        //     "/**/!Boolean()|| bar",
        //     "/**/true|| bar",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "(!Boolean()|| bar)/**/",
        //     "(true|| bar)/**/",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if(/**/Boolean()|| bar);",
        //     "if(/**/false|| bar);",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if(Boolean()|| bar/**/);",
        //     "if(false|| bar/**/);",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        (
            "if (a && !!(b ? c : d)){}",
            "if (a && (b ? c : d)){}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        // (
        //     "function *foo() { yield!!a || d ? b : c }",
        //     "function *foo() { yield a || d ? b : c }",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        (
            "if (!!foo || bar) {}",
            "if (foo || bar) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (!!foo && bar) {}",
            "if (foo && bar) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if ((!!foo || bar) && bat) {}",
            "if ((foo || bar) && bat) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (foo && !!bar) {}",
            "if (foo && bar) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "do {} while (!!foo || bar)",
            "do {} while (foo || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "while (!!foo || bar) {}",
            "while (foo || bar) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!!foo && bat ? bar : baz",
            "foo && bat ? bar : baz",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "for (; !!foo || bar;) {}",
            "for (; foo || bar;) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!!!foo || bar",
            "!foo || bar",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "Boolean(!!foo || bar)",
            "Boolean(foo || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "new Boolean(!!foo || bar)",
            "new Boolean(foo || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "if (Boolean(foo) || bar) {}",
            "if (foo || bar) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "do {} while (Boolean(foo) || bar)",
            "do {} while (foo || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "while (Boolean(foo) || bar) {}",
            "while (foo || bar) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "Boolean(foo) || bat ? bar : baz",
            "foo || bat ? bar : baz",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "for (; Boolean(foo) || bar;) {}",
            "for (; foo || bar;) {}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(foo) || bar",
            "!foo || bar",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(foo && bar) || bat",
            "!(foo && bar) || bat",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(foo + bar) || bat",
            "!(foo + bar) || bat",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(+foo)  || bar",
            "!+foo  || bar",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(foo()) || bar",
            "!foo() || bar",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(foo() || bar)",
            "!(foo() || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean(foo = bar) || bat",
            "!(foo = bar) || bat",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!Boolean((foo, bar()) || bat);",
            "!((foo, bar()) || bat);",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        // (
        //     "!Boolean() || bar;",
        //     "true || bar;",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "!(Boolean()) || bar;",
        //     "true || bar;",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!Boolean() || bar) { foo() }",
        //     "if (true || bar) { foo() }",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "while (!Boolean() || bar) { foo() }",
        //     "while (true || bar) { foo() }",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "var foo = Boolean() || bar ? bar() : baz()",
        //     "var foo = false || bar ? bar() : baz()",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean() || bar) { foo() }",
        //     "if (false || bar) { foo() }",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "while (Boolean() || bar) { foo() }",
        //     "while (false || bar) { foo() }",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        (
            "function *foo() { yield(!!a || d) ? b : c }",
            "function *foo() { yield(a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "function *foo() { yield(!! a || d) ? b : c }",
            "function *foo() { yield(a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "function *foo() { yield(! !a || d) ? b : c }",
            "function *foo() { yield(a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "function *foo() { yield (!!a || d) ? b : c }",
            "function *foo() { yield (a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "function *foo() { yield/**/(!!a || d) ? b : c }",
            "function *foo() { yield/**/(a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "x=!!a || d ? b : c ",
            "x=a || d ? b : c ",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        // (
        //     "void(!Boolean() || bar)",
        //     "void(true || bar)",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "void(! Boolean() || bar)",
        //     "void(true || bar)",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "typeof(!Boolean() || bar)",
        //     "typeof(true || bar)",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "(!Boolean() || bar)",
        //     "(true || bar)",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "void/**/(!Boolean() || bar)",
        //     "void/**/(true || bar)",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        (
            "!/**/(!!foo || bar)",
            "!/**/(foo || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!(!!foo || bar)/**/",
            "!(foo || bar)/**/",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!/**/(Boolean(foo) || bar)",
            "!/**/(foo || bar)",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        (
            "!(Boolean(foo)|| bar)/**/",
            "!(foo|| bar)/**/",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        // (
        //     "/**/!Boolean()|| bar",
        //     "/**/true|| bar",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "(!Boolean()|| bar)/**/",
        //     "(true|| bar)/**/",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if(/**/Boolean()|| bar);",
        //     "if(/**/false|| bar);",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if(Boolean()|| bar/**/);",
        //     "if(false|| bar/**/);",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        (
            "if (a && !!(b ? c : d)){}",
            "if (a && (b ? c : d)){}",
            Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        ),
        // (
        //     "function *foo() { yield!!a || d ? b : c }",
        //     "function *foo() { yield a || d ? b : c }",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        ("Boolean(!!(a, b))", "Boolean((a, b))", None),
        // ("Boolean(Boolean((a, b)))", "Boolean((a, b))", None),
        // ("Boolean((!!(a, b)))", "Boolean((a, b))", None),
        ("Boolean((Boolean((a, b))))", "Boolean((a, b))", None),
        ("Boolean(!(!(a, b)))", "Boolean((a, b))", None),
        // ("Boolean((!(!(a, b))))", "Boolean((a, b))", None),
        // ("Boolean(!!(a = b))", "Boolean(a = b)", None),
        // ("Boolean((!!(a = b)))", "Boolean((a = b))", None),
        ("Boolean(Boolean(a = b))", "Boolean(a = b)", None),
        ("Boolean(Boolean((a += b)))", "Boolean(a += b)", None),
        // ("Boolean(!!(a === b))", "Boolean(a === b)", None),
        // ("Boolean(!!((a !== b)))", "Boolean(a !== b)", None),
        ("Boolean(!!a.b)", "Boolean(a.b)", None),
        ("Boolean(Boolean((a)))", "Boolean(a)", None),
        // ("Boolean((!!(a)))", "Boolean((a))", None),
        ("new Boolean(!!(a, b))", "new Boolean((a, b))", None),
        // ("new Boolean(Boolean((a, b)))", "new Boolean((a, b))", None),
        // ("new Boolean((!!(a, b)))", "new Boolean((a, b))", None),
        ("new Boolean((Boolean((a, b))))", "new Boolean((a, b))", None),
        ("new Boolean(!(!(a, b)))", "new Boolean((a, b))", None),
        // ("new Boolean((!(!(a, b))))", "new Boolean((a, b))", None),
        // ("new Boolean(!!(a = b))", "new Boolean(a = b)", None),
        // ("new Boolean((!!(a = b)))", "new Boolean((a = b))", None),
        ("new Boolean(Boolean(a = b))", "new Boolean(a = b)", None),
        ("new Boolean(Boolean((a += b)))", "new Boolean(a += b)", None),
        // ("new Boolean(!!(a === b))", "new Boolean(a === b)", None),
        // ("new Boolean(!!((a !== b)))", "new Boolean(a !== b)", None),
        ("new Boolean(!!a.b)", "new Boolean(a.b)", None),
        ("new Boolean(Boolean((a)))", "new Boolean(a)", None),
        // ("new Boolean((!!(a)))", "new Boolean((a))", None),
        // ("if (!!(a, b));", "if (a, b);", None),
        ("if (Boolean((a, b)));", "if (a, b);", None),
        // ("if (!(!(a, b)));", "if (a, b);", None),
        // ("if (!!(a = b));", "if (a = b);", None),
        ("if (Boolean(a = b));", "if (a = b);", None),
        // ("if (!!(a > b));", "if (a > b);", None),
        ("if (Boolean(a === b));", "if (a === b);", None),
        ("if (!!f(a));", "if (f(a));", None),
        ("if (Boolean(f(a)));", "if (f(a));", None),
        // ("if (!!(f(a)));", "if (f(a));", None),
        ("if ((!!f(a)));", "if ((f(a)));", None),
        ("if ((Boolean(f(a))));", "if ((f(a)));", None),
        ("if (!!a);", "if (a);", None),
        ("if (Boolean(a));", "if (a);", None),
        // ("while (!!(a, b));", "while (a, b);", None),
        ("while (Boolean((a, b)));", "while (a, b);", None),
        // ("while (!(!(a, b)));", "while (a, b);", None),
        // ("while (!!(a = b));", "while (a = b);", None),
        ("while (Boolean(a = b));", "while (a = b);", None),
        // ("while (!!(a > b));", "while (a > b);", None),
        ("while (Boolean(a === b));", "while (a === b);", None),
        ("while (!!f(a));", "while (f(a));", None),
        ("while (Boolean(f(a)));", "while (f(a));", None),
        // ("while (!!(f(a)));", "while (f(a));", None),
        ("while ((!!f(a)));", "while ((f(a)));", None),
        ("while ((Boolean(f(a))));", "while ((f(a)));", None),
        ("while (!!a);", "while (a);", None),
        ("while (Boolean(a));", "while (a);", None),
        // ("do {} while (!!(a, b));", "do {} while (a, b);", None),
        ("do {} while (Boolean((a, b)));", "do {} while (a, b);", None),
        // ("do {} while (!(!(a, b)));", "do {} while (a, b);", None),
        // ("do {} while (!!(a = b));", "do {} while (a = b);", None),
        ("do {} while (Boolean(a = b));", "do {} while (a = b);", None),
        // ("do {} while (!!(a > b));", "do {} while (a > b);", None),
        // ("do {} while (Boolean(a === b));", "do {} while (a === b);", None),
        // ("do {} while (!!f(a));", "do {} while (f(a));", None),
        // ("do {} while (Boolean(f(a)));", "do {} while (f(a));", None),
        // ("do {} while (!!(f(a)));", "do {} while (f(a));", None),
        // ("do {} while ((!!f(a)));", "do {} while ((f(a)));", None),
        // ("do {} while ((Boolean(f(a))));", "do {} while ((f(a)));", None),
        // ("do {} while (!!a);", "do {} while (a);", None),
        // ("do {} while (Boolean(a));", "do {} while (a);", None),
        // ("for (; !!(a, b););", "for (; a, b;);", None),
        // ("for (; Boolean((a, b)););", "for (; a, b;);", None),
        // ("for (; !(!(a, b)););", "for (; a, b;);", None),
        // ("for (; !!(a = b););", "for (; a = b;);", None),
        // ("for (; Boolean(a = b););", "for (; a = b;);", None),
        // ("for (; !!(a > b););", "for (; a > b;);", None),
        // ("for (; Boolean(a === b););", "for (; a === b;);", None),
        // ("for (; !!f(a););", "for (; f(a););", None),
        // ("for (; Boolean(f(a)););", "for (; f(a););", None),
        // ("for (; !!(f(a)););", "for (; f(a););", None),
        // ("for (; (!!f(a)););", "for (; (f(a)););", None),
        // ("for (; (Boolean(f(a))););", "for (; (f(a)););", None),
        // ("for (; !!a;);", "for (; a;);", None),
        // ("for (; Boolean(a););", "for (; a;);", None),
        // ("!!(a, b) ? c : d", "(a, b) ? c : d", None),
        // ("(!!(a, b)) ? c : d", "(a, b) ? c : d", None),
        // ("Boolean((a, b)) ? c : d", "(a, b) ? c : d", None),
        // ("!!(a = b) ? c : d", "(a = b) ? c : d", None),
        // ("Boolean(a -= b) ? c : d", "(a -= b) ? c : d", None),
        // ("(Boolean((a *= b))) ? c : d", "(a *= b) ? c : d", None),
        // ("!!(a ? b : c) ? d : e", "(a ? b : c) ? d : e", None),
        // ("Boolean(a ? b : c) ? d : e", "(a ? b : c) ? d : e", None),
        // ("!!(a || b) ? c : d", "a || b ? c : d", None),
        // ("Boolean(a && b) ? c : d", "a && b ? c : d", None),
        // ("!!(a === b) ? c : d", "a === b ? c : d", None),
        // ("Boolean(a < b) ? c : d", "a < b ? c : d", None),
        // ("!!((a !== b)) ? c : d", "a !== b ? c : d", None),
        // ("Boolean((a >= b)) ? c : d", "a >= b ? c : d", None),
        // ("!!+a ? b : c", "+a ? b : c", None),
        // ("!!+(a) ? b : c", "+(a) ? b : c", None),
        // ("Boolean(!a) ? b : c", "!a ? b : c", None),
        // ("!!f(a) ? b : c", "f(a) ? b : c", None),
        // ("(!!f(a)) ? b : c", "(f(a)) ? b : c", None),
        // ("Boolean(a.b) ? c : d", "a.b ? c : d", None),
        // ("!!a ? b : c", "a ? b : c", None),
        // ("Boolean(a) ? b : c", "a ? b : c", None),
        // ("!!!(a, b)", "!(a, b)", None),
        // ("!Boolean((a, b))", "!(a, b)", None),
        // ("!!!(a = b)", "!(a = b)", None),
        // ("!!(!(a += b))", "!(a += b)", None),
        // ("!(!!(a += b))", "!(a += b)", None),
        // ("!Boolean(a -= b)", "!(a -= b)", None),
        // ("!Boolean((a -= b))", "!(a -= b)", None),
        // ("!(Boolean(a -= b))", "!(a -= b)", None),
        // ("!!!(a || b)", "!(a || b)", None),
        // ("!Boolean(a || b)", "!(a || b)", None),
        // ("!!!(a && b)", "!(a && b)", None),
        // ("!Boolean(a && b)", "!(a && b)", None),
        // ("!!!(a != b)", "!(a != b)", None),
        // ("!!!(a === b)", "!(a === b)", None),
        // ("var x = !Boolean(a > b)", "var x = !(a > b)", None),
        // ("!!!(a - b)", "!(a - b)", None),
        // ("!!!(a ** b)", "!(a ** b)", None),
        // ("!Boolean(a ** b)", "!(a ** b)", None),
        // ("async function f() { !!!(await a) }", "async function f() { !await a }", None),
        // ("async function f() { !Boolean(await a) }", "async function f() { !await a }", None),
        // ("!!!!a", "!!a", None),
        // ("!!(!(!a))", "!!a", None),
        // ("!Boolean(!a)", "!!a", None),
        // ("!Boolean((!a))", "!!a", None),
        // ("!Boolean(!(a))", "!!(a)", None),
        // ("!(Boolean(!a))", "!(!a)", None),
        // ("!!!+a", "!+a", None),
        // // ("!!!(+a)", "!+a", None),
        // ("!!(!+a)", "!+a", None),
        // ("!(!!+a)", "!(+a)", None),
        // ("!Boolean((-a))", "!-a", None),
        // ("!Boolean(-(a))", "!-(a)", None),
        // // ("!!!(--a)", "!--a", None),
        // ("!Boolean(a++)", "!a++", None),
        // ("!!!f(a)", "!f(a)", None),
        // ("!!!(f(a))", "!f(a)", None),
        // ("!!!a", "!a", None),
        // ("!Boolean(a)", "!a", None),
        // (
        //     "if (!!(a, b) || !!(c, d)) {}",
        //     "if ((a, b) || (c, d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean((a, b)) || Boolean((c, d))) {}",
        //     "if ((a, b) || (c, d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if ((!!((a, b))) || (!!((c, d)))) {}",
        //     "if ((a, b) || (c, d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!!(a, b) && !!(c, d)) {}",
        //     "if ((a, b) && (c, d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean((a, b)) && Boolean((c, d))) {}",
        //     "if ((a, b) && (c, d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if ((!!((a, b))) && (!!((c, d)))) {}",
        //     "if ((a, b) && (c, d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!!(a = b) || !!(c = d)) {}",
        //     "if ((a = b) || (c = d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean(a /= b) || Boolean(c /= d)) {}",
        //     "if ((a /= b) || (c /= d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!!(a >>= b) && !!(c >>= d)) {}",
        //     "if ((a >>= b) && (c >>= d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean(a **= b) && Boolean(c **= d)) {}",
        //     "if ((a **= b) && (c **= d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!!(a ? b : c) || !!(d ? e : f)) {}",
        //     "if ((a ? b : c) || (d ? e : f)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean(a ? b : c) || Boolean(d ? e : f)) {}",
        //     "if ((a ? b : c) || (d ? e : f)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!!(a ? b : c) && !!(d ? e : f)) {}",
        //     "if ((a ? b : c) && (d ? e : f)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean(a ? b : c) && Boolean(d ? e : f)) {}",
        //     "if ((a ? b : c) && (d ? e : f)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!!(a || b) || !!(c || d)) {}",
        //     "if (a || b || (c || d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean(a || b) || Boolean(c || d)) {}",
        //     "if (a || b || (c || d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!!(a || b) && !!(c || d)) {}",
        //     "if ((a || b) && (c || d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean(a || b) && Boolean(c || d)) {}",
        //     "if ((a || b) && (c || d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!!(a && b) || !!(c && d)) {}",
        //     "if (a && b || c && d) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean(a && b) || Boolean(c && d)) {}",
        //     "if (a && b || c && d) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!!(a && b) && !!(c && d)) {}",
        //     "if (a && b && (c && d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean(a && b) && Boolean(c && d)) {}",
        //     "if (a && b && (c && d)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!!(a !== b) || !!(c !== d)) {}",
        //     "if (a !== b || c !== d) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean(a != b) || Boolean(c != d)) {}",
        //     "if (a != b || c != d) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!!(a === b) && !!(c === d)) {}",
        //     "if (a === b && c === d) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!!(a > b) || !!(c < d)) {}",
        //     "if (a > b || c < d) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean(!a) || Boolean(+b)) {}",
        //     "if (!a || +b) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!!f(a) && !!b.c) {}",
        //     "if (f(a) && b.c) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean(a) || !!b) {}",
        //     "if (a || b) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (!!a && Boolean(b)) {}",
        //     "if (a && b) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if ((!!a) || (Boolean(b))) {}",
        //     "if ((a) || (b)) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // (
        //     "if (Boolean(a ?? b) || c) {}",
        //     "if ((a ?? b) || c) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // ("if (Boolean?.(foo)) {};", "if (foo) {};", None),
        // (
        //     "if (Boolean?.(a ?? b) || c) {}",
        //     "if ((a ?? b) || c) {}",
        //     Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        // ),
        // ("if (!Boolean(a as any)) { }", "if (!(a as any)) { }", None),
        // (
        //     "if ((1, 2, Boolean(3))) {}",
        //     "if ((1, 2, 3)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (a ?? Boolean(b)) {}",
        //     "if (a ?? b) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if ((a, b, c ?? (d, e, f ?? Boolean(g)))) {}",
        //     "if ((a, b, c ?? (d, e, f ?? g))) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!(a, b) || !!(c, d)) {}",
        //     "if ((a, b) || (c, d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean((a, b)) || Boolean((c, d))) {}",
        //     "if ((a, b) || (c, d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if ((!!((a, b))) || (!!((c, d)))) {}",
        //     "if ((a, b) || (c, d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!(a, b) && !!(c, d)) {}",
        //     "if ((a, b) && (c, d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean((a, b)) && Boolean((c, d))) {}",
        //     "if ((a, b) && (c, d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if ((!!((a, b))) && (!!((c, d)))) {}",
        //     "if ((a, b) && (c, d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!(a = b) || !!(c = d)) {}",
        //     "if ((a = b) || (c = d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean(a /= b) || Boolean(c /= d)) {}",
        //     "if ((a /= b) || (c /= d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!(a >>= b) && !!(c >>= d)) {}",
        //     "if ((a >>= b) && (c >>= d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean(a **= b) && Boolean(c **= d)) {}",
        //     "if ((a **= b) && (c **= d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!(a ? b : c) || !!(d ? e : f)) {}",
        //     "if ((a ? b : c) || (d ? e : f)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean(a ? b : c) || Boolean(d ? e : f)) {}",
        //     "if ((a ? b : c) || (d ? e : f)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!(a ? b : c) && !!(d ? e : f)) {}",
        //     "if ((a ? b : c) && (d ? e : f)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean(a ? b : c) && Boolean(d ? e : f)) {}",
        //     "if ((a ? b : c) && (d ? e : f)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!(a || b) || !!(c || d)) {}",
        //     "if (a || b || (c || d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean(a || b) || Boolean(c || d)) {}",
        //     "if (a || b || (c || d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!(a || b) && !!(c || d)) {}",
        //     "if ((a || b) && (c || d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean(a || b) && Boolean(c || d)) {}",
        //     "if ((a || b) && (c || d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!(a && b) || !!(c && d)) {}",
        //     "if (a && b || c && d) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean(a && b) || Boolean(c && d)) {}",
        //     "if (a && b || c && d) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!(a && b) && !!(c && d)) {}",
        //     "if (a && b && (c && d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean(a && b) && Boolean(c && d)) {}",
        //     "if (a && b && (c && d)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!(a !== b) || !!(c !== d)) {}",
        //     "if (a !== b || c !== d) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean(a != b) || Boolean(c != d)) {}",
        //     "if (a != b || c != d) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!(a === b) && !!(c === d)) {}",
        //     "if (a === b && c === d) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!(a > b) || !!(c < d)) {}",
        //     "if (a > b || c < d) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean(!a) || Boolean(+b)) {}",
        //     "if (!a || +b) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!f(a) && !!b.c) {}",
        //     "if (f(a) && b.c) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean(a) || !!b) {}",
        //     "if (a || b) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (!!a && Boolean(b)) {}",
        //     "if (a && b) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if ((!!a) || (Boolean(b))) {}",
        //     "if ((a) || (b)) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean(a ?? b) || c) {}",
        //     "if ((a ?? b) || c) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (Boolean?.(a ?? b) || c) {}",
        //     "if ((a ?? b) || c) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (a ? Boolean(b) : c) {}",
        //     "if (a ? b : c) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (a ? b : Boolean(c)) {}",
        //     "if (a ? b : c) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (a ? b : Boolean(c ? d : e)) {}",
        //     "if (a ? b : c ? d : e) {}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "const ternary = Boolean(bar ? !!baz : bat);",
        //     "const ternary = Boolean(bar ? baz : bat);",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "const commaOperator = Boolean((bar, baz, !!bat));",
        //     "const commaOperator = Boolean((bar, baz, bat));",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "
        // 	for (let i = 0; (console.log(i), Boolean(i < 10)); i++) {
        // 	    // ...
        // 	}",
        //     "
        // 	for (let i = 0; (console.log(i), i < 10); i++) {
        // 	    // ...
        // 	}",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "const nullishCoalescingOperator = Boolean(bar ?? Boolean(baz));",
        //     "const nullishCoalescingOperator = Boolean(bar ?? baz);",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (a ? Boolean(b = c) : Boolean(d = e));",
        //     "if (a ? b = c : d = e);",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "if (a ? Boolean((b, c)) : Boolean((d, e)));",
        //     "if (a ? (b, c) : (d, e));",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
        // (
        //     "
        // 	function * generator() {
        // 	    if (a ? Boolean(yield y) : x) {
        // 	        return a;
        // 	    };
        // 	}
        // 	",
        //     "
        // 	function * generator() {
        // 	    if (a ? yield y : x) {
        // 	        return a;
        // 	    };
        // 	}
        // 	",
        //     Some(serde_json::json!([{ "enforceForInnerExpressions": true }])),
        // ),
    ];

    Tester::new(NoExtraBooleanCast::NAME, NoExtraBooleanCast::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
