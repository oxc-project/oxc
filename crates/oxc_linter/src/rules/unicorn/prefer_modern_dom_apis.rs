use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::phf_map;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

fn prefer_modern_dom_apis_diagnostic(
    good_method: &str,
    bad_method: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer using `{good_method}` over `{bad_method}`."))
        .with_label(span)
}

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
    /// - `childNode.replaceWith(newNode)` over `parentNode.replaceChild(newNode, oldNode)`
    /// - `referenceNode.before(newNode)` over `parentNode.insertBefore(newNode, referenceNode)`
    /// - `referenceNode.before('text')` over `referenceNode.insertAdjacentText('beforebegin', 'text')`
    /// - `referenceNode.before(newNode)` over `referenceNode.insertAdjacentElement('beforebegin', newNode)`
    ///
    /// ### Why is this bad?
    ///
    /// There are some advantages of using the newer DOM APIs, like:
    /// - Traversing to the parent node is not necessary.
    /// - Appending multiple nodes at once.
    /// - Both `DOMString` and DOM node objects can be manipulated.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// oldChildNode.replaceWith(newChildNode);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// parentNode.replaceChild(newChildNode, oldChildNode);
    /// ```
    PreferModernDomApis,
    unicorn,
    style,
    pending
);

impl Rule for PreferModernDomApis {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Expression::StaticMemberExpression(member_expr) = &call_expr.callee else {
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
            .all(|argument| matches!(argument.as_expression(), Some(expr) if !expr.is_undefined()))
            && matches!(member_expr.object, Expression::Identifier(_))
            && !call_expr.optional
        {
            if let Some(preferred_method) = DISALLOWED_METHODS.get(method) {
                ctx.diagnostic(prefer_modern_dom_apis_diagnostic(
                    preferred_method,
                    method,
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
            if let Argument::StringLiteral(lit) = &call_expr.arguments[0] {
                for (position, replacer) in &POSITION_REPLACERS {
                    if lit.value == position {
                        ctx.diagnostic(prefer_modern_dom_apis_diagnostic(
                            replacer,
                            method,
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
        (
            "const foo = () => { return referenceNode.insertAdjacentElement(\"beforebegin\", newNode); }",
            None,
        ),
        ("if (referenceNode.insertAdjacentElement(\"beforebegin\", newNode)) {}", None),
        (
            "const foo = { bar: referenceNode.insertAdjacentElement(\"beforebegin\", newNode) }",
            None,
        ),
    ];

    Tester::new(PreferModernDomApis::NAME, PreferModernDomApis::PLUGIN, pass, fail)
        .test_and_snapshot();
}
