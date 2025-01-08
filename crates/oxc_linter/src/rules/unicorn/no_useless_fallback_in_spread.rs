use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::LogicalOperator;

use crate::{ast_util::outermost_paren_parent, context::LintContext, rule::Rule, AstNode};

fn no_useless_fallback_in_spread_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Empty fallbacks in spreads are unnecessary")
        .with_help("Spreading falsy values in object literals won't add any unexpected properties, so it's unnecessary to add an empty object as fallback.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessFallbackInSpread;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow useless fallback when spreading in object literals.
    ///
    /// ### Why is this bad?
    ///
    /// Spreading [falsy values](https://developer.mozilla.org/en-US/docs/Glossary/Falsy) in object literals won't add any unexpected properties, so it's unnecessary to add an empty object as fallback.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const object = { ...(foo || {}) }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const object = { ...foo }
    /// const object = { ...(foo || { not: "empty" }) }
    /// ```
    NoUselessFallbackInSpread,
    unicorn,
    correctness,
    conditional_fix
);

impl Rule for NoUselessFallbackInSpread {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::LogicalExpression(logical_expression) = node.kind() else {
            return;
        };

        if !matches!(logical_expression.operator, LogicalOperator::Or | LogicalOperator::Coalesce) {
            return;
        }

        let Expression::ObjectExpression(object_expression) =
            &logical_expression.right.without_parentheses()
        else {
            return;
        };

        if object_expression.properties.len() != 0 {
            return;
        }

        let Some(parent) = outermost_paren_parent(node, ctx) else {
            return;
        };

        let AstKind::SpreadElement(spread_element) = parent.kind() else {
            return;
        };

        let Some(parent) = outermost_paren_parent(parent, ctx) else {
            return;
        };

        if !matches!(parent.kind(), AstKind::ObjectExpression(_)) {
            return;
        }

        let diagnostic = no_useless_fallback_in_spread_diagnostic(spread_element.span);

        if can_fix(&logical_expression.left) {
            ctx.diagnostic_with_fix(diagnostic, |fixer| {
                let left_text = fixer.source_range(logical_expression.left.span());
                fixer.replace(spread_element.span, format!("...{left_text}"))
            });
        } else {
            ctx.diagnostic(diagnostic);
        }
    }
}

fn can_fix(left: &Expression<'_>) -> bool {
    const BANNED_IDENTIFIERS: [&str; 3] = ["undefined", "NaN", "Infinity"];
    match left.without_parentheses() {
        Expression::Identifier(ident) => !BANNED_IDENTIFIERS.contains(&ident.name.as_str()),
        Expression::LogicalExpression(expr) => can_fix(&expr.left),
        Expression::ObjectExpression(_)
        | Expression::ChainExpression(_)
        | Expression::CallExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_) => true,
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const array = [...(foo || [])]",
        r"const array = [...(foo || {})]",
        r"const array = [...(foo && {})]",
        r"const object = {...(foo && {})}",
        r"const object = {...({} || foo)}",
        r"const object = {...({} && foo)}",
        r"const object = {...({} ?? foo)}",
        r"const object = {...(foo ? foo : {})}",
        r"const object = {...foo}",
        r"const object = {...(foo ?? ({} || {}))}",
        r"const {...foo} = object",
        r"function foo({...bar}){}",
        r"const object = {...(foo || {}).toString()}",
        r"const object = {...fn(foo || {})}",
        r"const object = call({}, ...(foo || {}))",
        r#"const object = {...(foo || {not: "empty"})}"#,
        r"const object = {...(foo || {...{}})}",
    ];

    let fail = vec![
        r"const object = {...(foo || {})}",
        r"const object = {...(foo ?? {})}",
        r"const object = {...(foo ?? (( {} )))}",
        r"const object = {...((( foo )) ?? (( {} )))}",
        r"const object = {...(( (( foo )) ?? (( {} )) ))}",
        r"async ()=> ({...((await foo) || {})})",
        r"const object = {...(0 || {})}",
        r"const object = {...((-0) || {})}",
        r"const object = {...(.0 || {})}",
        r"const object = {...(0n || {})}",
        r"const object = {...(false || {})}",
        r"const object = {...(null || {})}",
        r"const object = {...(undefined || {})}",
        r"const object = {...((a && b) || {})}",
        r"const object = {...(NaN || {})}",
        r#"const object = {...("" || {})}"#,
        r"const object = {...([] || {})}",
        r"const object = {...({} || {})}",
        r"const object = {...(foo || {}),}",
        r"const object = {...((foo ?? {}) || {})}",
        r"const object = {...((foo && {}) || {})}",
        r"const object = {...(foo && {} || {})}",
        r"const object = {...({...(foo || {})})}",
        r"function foo(a = {...(bar || {})}){}",
        r"const object = {...(document.all || {})}",
    ];

    let fix = vec![
        //
        (r"const object = {...(foo || {})}", r"const object = {...foo}"),
        (r"const object = {...(foo?.bar || {})}", r"const object = {...foo?.bar}"),
        (r"const object = {...(foo() || {})}", r"const object = {...foo()}"),
        (r"const object = {...(foo?.bar() || {})}", r"const object = {...foo?.bar()}"),
        (r"const object = {...((foo && {}) || {})}", "const object = {...(foo && {})}"),
        (r"const object = {...(0 || {})}", r"const object = {...(0 || {})}"),
        (r"const object = {...(NaN || {})}", r"const object = {...(NaN || {})}"),
        (r"const object = {...(Infinity || {})}", r"const object = {...(Infinity || {})}"),
    ];

    Tester::new(NoUselessFallbackInSpread::NAME, NoUselessFallbackInSpread::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
