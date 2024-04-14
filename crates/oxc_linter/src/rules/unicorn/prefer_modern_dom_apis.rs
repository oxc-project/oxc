use oxc_ast::{
    ast::{Argument, Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use phf::phf_map;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-modern-dom-apis): Prefer using `{0}` over `{1}`.")]
#[diagnostic(severity(warning))]
struct PreferModernDomApisDiagnostic(pub &'static str, CompactStr, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferModernDomApis;

const DISALLOWED_METHODS: phf::Map<&'static str, &'static str> = phf_map!(
    "replaceChild" => "replaceWith",
    "insertBefore" => "before",
);

const POSITION_REPLACERS: phf::Map<&'static str, &'static str> = phf_map!(
    "beforebegin" => "before",
    "afterbegin" => "prepend",
    "beforeend" => "append",
    "afterend" => "after",
);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of:
    /// - childNode.replaceWith(newNode) over parentNode.replaceChild(newNode, oldNode)
    /// - referenceNode.before(newNode) over parentNode.insertBefore(newNode, referenceNode)
    /// - referenceNode.before('text') over referenceNode.insertAdjacentText('beforebegin', 'text')
    /// - referenceNode.before(newNode) over referenceNode.insertAdjacentElement('beforebegin', newNode)
    ///
    /// ### Why is this bad?
    ///
    /// There are some advantages of using the newer DOM APIs, like:
    /// - Traversing to the parent node is not necessary.
    /// - Appending multiple nodes at once.
    /// - Both DOMString and DOM node objects can be manipulated.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// ("oldChildNode.replaceWith(newChildNode);", None),
    ///
    /// // Good
    /// ("parentNode.replaceChild(newChildNode, oldChildNode);", None),
    /// ```
    PreferModernDomApis,
    style
);

impl Rule for PreferModernDomApis {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Expression::MemberExpression(member_expr) = &call_expr.callee else {
            return;
        };

        let MemberExpression::StaticMemberExpression(member_expr) = &**member_expr else {
            return;
        };
        let method = member_expr.property.name.as_str();

        if is_method_call(
            call_expr,
            None,
            Some(&["replaceChild", "insertBefore"]),
            Some(2),
            Some(2),
        ) && call_expr
            .arguments
            .iter()
            .all(|argument| matches!(argument, Argument::Expression(expr) if !expr.is_undefined()))
            && matches!(member_expr.object, Expression::Identifier(_))
            && !call_expr.optional
        {
            if let Some(preferred_method) = DISALLOWED_METHODS.get(method) {
                ctx.diagnostic(PreferModernDomApisDiagnostic(
                    preferred_method,
                    CompactStr::from(method),
                    member_expr.property.span,
                ));

                return;
            }
        }

        if is_method_call(
            call_expr,
            None,
            Some(&["insertAdjacentText", "insertAdjacentElement"]),
            Some(2),
            Some(2),
        ) {
            if let Argument::Expression(Expression::StringLiteral(lit)) = &call_expr.arguments[0] {
                for (position, replacer) in &POSITION_REPLACERS {
                    if lit.value == position {
                        ctx.diagnostic(PreferModernDomApisDiagnostic(
                            replacer,
                            CompactStr::from(method),
                            member_expr.property.span,
                        ));
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("oldChildNode.replaceWith(newChildNode);", None),
        ("referenceNode.before(newNode);", None),
        ("referenceNode.before(\"text\");", None),
        ("referenceNode.prepend(newNode);", None),
        ("referenceNode.prepend(\"text\");", None),
        ("referenceNode.append(newNode);", None),
        ("referenceNode.append(\"text\");", None),
        ("referenceNode.after(newNode);", None),
        ("referenceNode.after(\"text\");", None),
        ("oldChildNode.replaceWith(undefined, oldNode);", None),
        ("oldChildNode.replaceWith(newNode, undefined);", None),
        ("new parentNode.replaceChild(newNode, oldNode);", None),
        ("new parentNode.insertBefore(newNode, referenceNode);", None),
        ("new referenceNode.insertAdjacentText('beforebegin', 'text');", None),
        ("new referenceNode.insertAdjacentElement('beforebegin', newNode);", None),
        ("replaceChild(newNode, oldNode);", None),
        ("insertBefore(newNode, referenceNode);", None),
        ("insertAdjacentText('beforebegin', 'text');", None),
        ("insertAdjacentElement('beforebegin', newNode);", None),
        ("parentNode['replaceChild'](newNode, oldNode);", None),
        ("parentNode['insertBefore'](newNode, referenceNode);", None),
        ("referenceNode['insertAdjacentText']('beforebegin', 'text');", None),
        ("referenceNode['insertAdjacentElement']('beforebegin', newNode);", None),
        ("parentNode[replaceChild](newNode, oldNode);", None),
        ("parentNode[insertBefore](newNode, referenceNode);", None),
        ("referenceNode[insertAdjacentText]('beforebegin', 'text');", None),
        ("referenceNode[insertAdjacentElement]('beforebegin', newNode);", None),
        ("parent.foo(a, b);", None),
        ("parentNode.replaceChild(newNode);", None),
        ("parentNode.insertBefore(newNode);", None),
        ("referenceNode.insertAdjacentText('beforebegin');", None),
        ("referenceNode.insertAdjacentElement('beforebegin');", None),
        ("parentNode.replaceChild(newNode, oldNode, extra);", None),
        ("parentNode.insertBefore(newNode, referenceNode, extra);", None),
        ("referenceNode.insertAdjacentText('beforebegin', 'text', extra);", None),
        ("referenceNode.insertAdjacentElement('beforebegin', newNode, extra);", None),
        ("parentNode.replaceChild(...argumentsArray1, ...argumentsArray2);", None),
        ("parentNode.insertBefore(...argumentsArray1, ...argumentsArray2);", None),
        ("referenceNode.insertAdjacentText(...argumentsArray1, ...argumentsArray2);", None),
        ("referenceNode.insertAdjacentElement(...argumentsArray1, ...argumentsArray2);", None),
        ("referenceNode.insertAdjacentText('foo', 'text');", None),
        ("referenceNode.insertAdjacentElement('foo', newNode);", None),
    ];

    let fail = vec![
        ("parentNode.replaceChild(newChildNode, oldChildNode);", None),
        ("const foo = parentNode.replaceChild(newChildNode, oldChildNode);", None),
        ("foo = parentNode.replaceChild(newChildNode, oldChildNode);", None),
        ("parentNode.insertBefore(newNode, referenceNode);", None),
        ("parentNode.insertBefore(alfa, beta).insertBefore(charlie, delta);", None),
        ("const foo = parentNode.insertBefore(alfa, beta);", None),
        ("foo = parentNode.insertBefore(alfa, beta);", None),
        ("new Dom(parentNode.insertBefore(alfa, beta))", None),
        ("`${parentNode.insertBefore(alfa, beta)}`", None),
        ("referenceNode.insertAdjacentText(\"beforebegin\", \"text\");", None),
        ("referenceNode.insertAdjacentText(\"afterbegin\", \"text\");", None),
        ("referenceNode.insertAdjacentText(\"beforeend\", \"text\");", None),
        ("referenceNode.insertAdjacentText(\"afterend\", \"text\");", None),
        ("const foo = referenceNode.insertAdjacentText(\"beforebegin\", \"text\");", None),
        ("foo = referenceNode.insertAdjacentText(\"beforebegin\", \"text\");", None),
        ("referenceNode.insertAdjacentElement(\"beforebegin\", newNode);", None),
        ("referenceNode.insertAdjacentElement(\"afterbegin\", \"text\");", None),
        ("referenceNode.insertAdjacentElement(\"beforeend\", \"text\");", None),
        ("referenceNode.insertAdjacentElement(\"afterend\", newNode);", None),
        ("const foo = referenceNode.insertAdjacentElement(\"beforebegin\", newNode);", None),
        ("foo = referenceNode.insertAdjacentElement(\"beforebegin\", newNode);", None),
        ("const foo = [referenceNode.insertAdjacentElement(\"beforebegin\", newNode)]", None),
        ("foo(bar = referenceNode.insertAdjacentElement(\"beforebegin\", newNode))", None),
        ("const foo = () => { return referenceNode.insertAdjacentElement(\"beforebegin\", newNode); }", None),
        ("if (referenceNode.insertAdjacentElement(\"beforebegin\", newNode)) {}", None),
        ("const foo = { bar: referenceNode.insertAdjacentElement(\"beforebegin\", newNode) }", None),
    ];

    Tester::new(PreferModernDomApis::NAME, pass, fail).test_and_snapshot();
}
