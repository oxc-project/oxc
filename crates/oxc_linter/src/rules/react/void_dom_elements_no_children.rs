use oxc_ast::{
    ast::{
        Argument, JSXAttributeItem, JSXAttributeName, JSXElementName, ObjectPropertyKind,
        PropertyKey,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::phf_set;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::is_create_element_call,
    AstNode,
};

fn void_dom_elements_no_children_diagnostic(tag: &str, span: Span) -> OxcDiagnostic {
    // TODO: use imperative phrasing
    OxcDiagnostic::warn(format!("Void DOM element <{tag:?} /> cannot receive children."))
        .with_help("Remove this element's children or use a non-void element.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct VoidDomElementsNoChildren;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow void DOM elements (e.g. `<img />`, `<br />`) from receiving children.
    ///
    /// ### Why is this bad?
    /// There are some HTML elements that are only self-closing (e.g. img, br, hr). These are collectively known as void DOM elements.
    /// This rule checks that children are not passed to void DOM elements.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <br>Children</br>
    /// <br children='Children' />
    /// <br dangerouslySetInnerHTML={{ __html: 'HTML' }} />
    /// React.createElement('br', undefined, 'Children')
    /// React.createElement('br', { children: 'Children' })
    /// React.createElement('br', { dangerouslySetInnerHTML: { __html: 'HTML' } })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div>Children</div>
    /// <div children='Children' />
    /// <div dangerouslySetInnerHTML={{ __html: 'HTML' }} />
    /// React.createElement('div', undefined, 'Children')
    /// React.createElement('div', { children: 'Children' })
    /// React.createElement('div', { dangerouslySetInnerHTML: { __html: 'HTML' } })
    /// ```
    VoidDomElementsNoChildren,
    react,
    correctness
);

const VOID_DOM_ELEMENTS: phf::Set<&'static str> = phf_set![
    "area", "base", "br", "col", "embed", "hr", "img", "input", "keygen", "link", "menuitem",
    "meta", "param", "source", "track", "wbr",
];

pub fn is_void_dom_element(element_name: &str) -> bool {
    VOID_DOM_ELEMENTS.contains(element_name)
}

impl Rule for VoidDomElementsNoChildren {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(jsx_el) => {
                let jsx_opening_el = &jsx_el.opening_element;
                let JSXElementName::Identifier(identifier) = &jsx_opening_el.name else {
                    return;
                };

                if !is_void_dom_element(&identifier.name) {
                    return;
                }

                let has_children_attribute_or_danger =
                    jsx_opening_el.attributes.iter().any(|attribute| match attribute {
                        JSXAttributeItem::Attribute(attr) => {
                            let JSXAttributeName::Identifier(iden) = &attr.name else {
                                return false;
                            };
                            iden.name == "children" || iden.name == "dangerouslySetInnerHTML"
                        }
                        JSXAttributeItem::SpreadAttribute(_) => false,
                    });

                if !jsx_el.children.is_empty() || has_children_attribute_or_danger {
                    ctx.diagnostic(void_dom_elements_no_children_diagnostic(
                        &identifier.name,
                        identifier.span,
                    ));
                }
            }
            AstKind::CallExpression(call_expr) => {
                if !is_create_element_call(call_expr) {
                    return;
                }

                if call_expr.arguments.is_empty() {
                    return;
                }

                let Some(Argument::StringLiteral(element_name)) = call_expr.arguments.first()
                else {
                    return;
                };

                if !is_void_dom_element(element_name.value.as_str()) {
                    return;
                }

                if call_expr.arguments.len() < 2 {
                    return;
                }

                let Some(Argument::ObjectExpression(obj_expr)) = call_expr.arguments.get(1) else {
                    return;
                };

                let has_children_prop_or_danger =
                    obj_expr.properties.iter().any(|property| match property {
                        ObjectPropertyKind::ObjectProperty(prop) => match &prop.key {
                            PropertyKey::StaticIdentifier(iden) => {
                                iden.name == "children" || iden.name == "dangerouslySetInnerHTML"
                            }
                            _ => false,
                        },
                        ObjectPropertyKind::SpreadProperty(_) => false,
                    });

                if call_expr.arguments.get(2).is_some() || has_children_prop_or_danger {
                    ctx.diagnostic(void_dom_elements_no_children_diagnostic(
                        &element_name.value,
                        element_name.span,
                    ));
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"<div>Foo</div>;", None),
        (r"<div children='Foo' />;", None),
        (r"<div dangerouslySetInnerHTML={{ __html: 'Foo' }} />;", None),
        (r"React.createElement('div', {}, 'Foo');", None),
        (r"React.createElement('div', { children: 'Foo' });", None),
        (r"React.createElement('div', { dangerouslySetInnerHTML: { __html: 'Foo' } });", None),
        (r"React.createElement('img');", None),
        (r"React.createElement();", None),
        (
            r"
                const props = {};
                React.createElement('img', props);
            ",
            None,
        ),
        (
            r"
                import React, {createElement} from 'react';
                createElement('div');
            ",
            None,
        ),
        (
            r"
                import React, {createElement} from 'react';
                createElement('img');
            ",
            None,
        ),
        (
            r"
                import React, {createElement, PureComponent} from 'react';
                class Button extends PureComponent {
                    handleClick(ev) {
                        ev.preventDefault();
                    }
                    render() {
                        return <div onClick={this.handleClick}>Hello</div>;
                    }
                }
            ",
            None,
        ),
    ];

    let fail = vec![
        (r"<br>Foo</br>;", None),
        (r"<br children='Foo' />;", None),
        (r"<img {...props} children='Foo' />;", None),
        (r"<br dangerouslySetInnerHTML={{ __html: 'Foo' }} />;", None),
        (r"React.createElement('br', {}, 'Foo');", None),
        (r"React.createElement('br', { children: 'Foo' });", None),
        (r"React.createElement('br', { dangerouslySetInnerHTML: { __html: 'Foo' } });", None),
        (
            r"
                import React, {createElement} from 'react';
                createElement('img', {}, 'Foo');
            ",
            None,
        ),
        (
            r"
                import React, {createElement} from 'react';
                createElement('img', { children: 'Foo' });
            ",
            None,
        ),
        (
            r"
                import React, {createElement} from 'react';
                createElement('img', { dangerouslySetInnerHTML: { __html: 'Foo' } });
            ",
            None,
        ),
    ];

    Tester::new(VoidDomElementsNoChildren::NAME, VoidDomElementsNoChildren::PLUGIN, pass, fail)
        .test_and_snapshot();
}
