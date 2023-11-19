use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::LogicalOperator;

use crate::{ast_util::outermost_paren_parent, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(no-useless-fallback-in-spread): Disallow useless fallback when spreading in object literals")]
#[diagnostic(severity(warning), help("Spreading falsy values in object literals won't add any unexpected properties, so it's unnecessary to add an empty object as fallback."))]
struct NoUselessFallbackInSpreadDiagnostic(#[label] pub Span);

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
    /// ### Example
    /// ```javascript
    /// // bad
    /// const object = { ...(foo || {}) }
    ///
    /// // good
    /// const object = { ...foo }
    /// const object = { ...(foo || { not: "empty" }) }
    ///
    /// ```
    NoUselessFallbackInSpread,
    correctness
);

impl Rule for NoUselessFallbackInSpread {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::LogicalExpression(logical_expression) = node.kind() else { return };

        if !matches!(logical_expression.operator, LogicalOperator::Or | LogicalOperator::Coalesce) {
            return;
        }

        let Expression::ObjectExpression(object_expression) =
            &logical_expression.right.without_parenthesized()
        else {
            return;
        };

        if object_expression.properties.len() != 0 {
            return;
        }

        let Some(parent) = outermost_paren_parent(node, ctx) else { return };

        let AstKind::SpreadElement(spread_element) = parent.kind() else { return };

        let Some(parent) = outermost_paren_parent(parent, ctx) else { return };

        if !matches!(parent.kind(), AstKind::ObjectExpression(_)) {
            return;
        }

        ctx.diagnostic(NoUselessFallbackInSpreadDiagnostic(spread_element.span));
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

    Tester::new_without_config(NoUselessFallbackInSpread::NAME, pass, fail).test_and_snapshot();
}
