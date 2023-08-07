use itertools::Itertools;
use oxc_ast::{ast::Argument, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{LogicalOperator, UnaryOperator};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-extra-boolean-cast): Redundant double negation")]
#[diagnostic(
    severity(warning),
    help("Remove the double negation as it will already be coerced to a boolean")
)]
struct NoExtraDoubleNegationCastDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-extra-boolean-cast): Redundant Boolean call")]
#[diagnostic(
    severity(warning),
    help("Remove the Boolean call as it will already be coerced to a boolean")
)]
struct NoExtraBooleanCastDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoExtraBooleanCast {
    pub enforce_for_logical_operands: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// This rule disallows unnecessary boolean casts.
    ///
    /// ### Why is this bad?
    /// In contexts such as an if statement’s test where the result of the expression will already be coerced to a Boolean,
    /// casting to a Boolean via double negation (!!) or a Boolean call is unnecessary.
    ///
    /// ### Example
    /// ```javascript
    /// var foo = !!!bar;
    /// var foo = Boolean(!!bar);
    ///
    /// if (!!foo) {}
    /// if (Boolean(foo)) {}
    ///
    /// // with "enforceForLogicalOperands" option enabled
    /// if (!!foo || bar) {}
    /// ```
    NoExtraBooleanCast,
    correctness
);

impl Rule for NoExtraBooleanCast {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
            enforce_for_logical_operands: value
                .get(0)
                .and_then(|x| x.get("enforceForLogicalOperands"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(expr) = node.kind() {
            if expr
                .callee
                .without_parenthesized()
                .get_identifier_reference()
                .is_some_and(|ident| ident.name != "Boolean")
            {
                return;
            }
            if is_flagged_ctx(node, ctx, self.enforce_for_logical_operands) {
                ctx.diagnostic(NoExtraBooleanCastDiagnostic(expr.span));
            }
        }

        let Some(parent) = get_real_parent(node, ctx) else { return };

        if let (AstKind::UnaryExpression(expr), AstKind::UnaryExpression(parent_expr)) =
            (node.kind(), parent.kind())
        {
            match (expr.operator, parent_expr.operator) {
                (UnaryOperator::LogicalNot, UnaryOperator::LogicalNot)
                    if is_flagged_ctx(parent, ctx, self.enforce_for_logical_operands) =>
                {
                    ctx.diagnostic(NoExtraDoubleNegationCastDiagnostic(parent_expr.span));
                }
                _ => (),
            }
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
        AstKind::CallExpression(expr) => expr
            .callee
            .without_parenthesized()
            .get_identifier_reference()
            .is_some_and(|x| x.name == "Boolean"),
        AstKind::NewExpression(expr) => expr
            .callee
            .without_parenthesized()
            .get_identifier_reference()
            .is_some_and(|x| x.name == "Boolean"),
        _ => false,
    }
}

fn is_first_arg(node: &AstNode, parent: &AstNode) -> bool {
    match parent.kind() {
        AstKind::CallExpression(expr) => expr.arguments.first().map_or(false, |arg| {
            if let Argument::Expression(expr) = arg {
                expr.without_parenthesized().span() == node.kind().span()
            } else {
                false
            }
        }),
        AstKind::NewExpression(expr) => expr.arguments.first().map_or(false, |arg| {
            if let Argument::Expression(expr) = arg {
                expr.without_parenthesized().span() == node.kind().span()
            } else {
                false
            }
        }),
        _ => false,
    }
}

fn is_inside_test_condition(node: &AstNode, ctx: &LintContext) -> bool {
    get_real_parent(node, ctx).map_or(false, |parent| match parent.kind() {
        AstKind::IfStatement(stmt) => {
            let expr_span = stmt.test.get_inner_expression().without_parenthesized().span();
            expr_span == node.kind().span()
        }
        AstKind::DoWhileStatement(stmt) => {
            let expr_span = stmt.test.get_inner_expression().without_parenthesized().span();
            expr_span == node.kind().span()
        }
        AstKind::WhileStatement(stmt) => {
            let expr_span = stmt.test.get_inner_expression().without_parenthesized().span();
            expr_span == node.kind().span()
        }
        AstKind::ConditionalExpression(stmt) => {
            let expr_span = stmt.test.get_inner_expression().without_parenthesized().span();
            expr_span == node.kind().span()
        }
        AstKind::ForStatement(stmt) => stmt.test.as_ref().map_or(false, |expr| {
            let expr_span = expr.get_inner_expression().without_parenthesized().span();
            expr_span == node.kind().span()
        }),
        _ => false,
    })
}

fn is_unary_negation(node: &AstNode) -> bool {
    match node.kind() {
        AstKind::UnaryExpression(expr) => expr.operator == UnaryOperator::LogicalNot,
        _ => false,
    }
}

fn get_real_parent<'a, 'b>(node: &AstNode, ctx: &'a LintContext<'b>) -> Option<&'a AstNode<'b>> {
    for (_, parent) in
        ctx.nodes().iter_parents(node.id()).tuple_windows::<(&AstNode<'b>, &AstNode<'b>)>()
    {
        if let AstKind::Argument(_) | AstKind::ParenthesizedExpression(_) = parent.kind() {
            continue;
        }

        return Some(parent);
    }
    None
}

#[allow(clippy::too_many_lines)]
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
        ("if (!!foo ?? bar) {}", Some(serde_json::json!([{ "enforceForLogicalOperands": true }]))),
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
        ("!Boolean(...foo);", None),
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
        ("function *foo() { yield!!a ? b : c }", None),
        ("function *foo() { yield!! a ? b : c }", None),
        ("function *foo() { yield! !a ? b : c }", None),
        ("function *foo() { yield !!a ? b : c }", None),
        ("function *foo() { yield(!!a) ? b : c }", None),
        ("function *foo() { yield/**/!!a ? b : c }", None),
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
        ),
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
        ),
        (
            "function *foo() { yield(!! a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "function *foo() { yield(! !a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "function *foo() { yield (!!a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        (
            "function *foo() { yield/**/(!!a || d) ? b : c }",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
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
        ),
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
        ("!!!(a ** b)", None),
        ("!Boolean(a ** b)", None),
        ("async function f() { !!!(await a) }", None),
        ("async function f() { !Boolean(await a) }", None),
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
        ),
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
        ),
        ("if (Boolean?.(foo)) ;", None),
        (
            "if (Boolean?.(a ?? b) || c) {}",
            Some(serde_json::json!([{ "enforceForLogicalOperands": true }])),
        ),
        ("if (!Boolean(a as any)) { }", None),
    ];

    Tester::new(NoExtraBooleanCast::NAME, pass, fail).test_and_snapshot();
}
