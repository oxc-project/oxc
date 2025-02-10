use oxc_ast::{
    ast::{
        Argument, Expression, JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue,
        JSXElementName, ObjectPropertyKind, TSType,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};

use crate::{
    ast_util::get_declaration_of_variable,
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::is_create_element_call,
    AstNode,
};

fn style_prop_object_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Style prop value must be an object").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct StylePropObject(Box<StylePropObjectConfig>);

#[derive(Debug, Default, Clone)]
pub struct StylePropObjectConfig {
    allow: Vec<CompactStr>,
}

impl std::ops::Deref for StylePropObject {
    type Target = StylePropObjectConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Require that the value of the prop `style` be an object or a variable that is an object.
    ///
    /// ### Why is this bad?
    /// The `style` prop expects an object mapping from style properties to values when using JSX.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div style="color: 'red'" />
    /// <div style={true} />
    /// <Hello style={true} />
    /// const styles = true;
    /// <div style={styles} />
    ///
    /// React.createElement("div", { style: "color: 'red'" });
    /// React.createElement("div", { style: true });
    /// React.createElement("Hello", { style: true });
    /// const styles = true;
    /// React.createElement("div", { style: styles });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div style={{ color: "red" }} />
    /// <Hello style={{ color: "red" }} />
    /// const styles = { color: "red" };
    /// <div style={styles} />
    ///
    /// React.createElement("div", { style: { color: 'red' }});
    /// React.createElement("Hello", { style: { color: 'red' }});
    /// const styles = { height: '100px' };
    /// React.createElement("div", { style: styles });
    /// ```
    StylePropObject,
    react,
    suspicious
);

fn is_invalid_type(ty: &TSType) -> bool {
    match ty {
        TSType::TSNumberKeyword(_) | TSType::TSStringKeyword(_) | TSType::TSBooleanKeyword(_) => {
            true
        }
        TSType::TSUnionType(union) => union.types.iter().any(is_invalid_type),
        TSType::TSIntersectionType(intersect) => intersect.types.iter().any(is_invalid_type),
        _ => false,
    }
}

fn is_invalid_expression<'a>(expression: Option<&Expression<'a>>, ctx: &LintContext<'a>) -> bool {
    let Some(expression) = expression else {
        return false;
    };

    match expression {
        Expression::StringLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::TemplateLiteral(_) => true,
        Expression::Identifier(ident) => {
            let Some(node) = get_declaration_of_variable(ident, ctx) else {
                return false;
            };

            let AstKind::VariableDeclarator(var_decl) = node.kind() else {
                return false;
            };

            if let Some(asd) = var_decl.id.type_annotation.as_ref() {
                return is_invalid_type(&asd.type_annotation);
            };

            is_invalid_expression(var_decl.init.as_ref(), ctx)
        }
        _ => false,
    }
}

fn is_invalid_style_attribute<'a>(attribute: &JSXAttribute<'a>, ctx: &LintContext<'a>) -> bool {
    let JSXAttributeName::Identifier(attr_ident) = &attribute.name else {
        return false;
    };

    if attr_ident.name == "style" {
        if let Some(attr_value) = &attribute.value {
            return match attr_value {
                JSXAttributeValue::StringLiteral(_) => true,
                JSXAttributeValue::ExpressionContainer(container) => {
                    return is_invalid_expression(container.expression.as_expression(), ctx);
                }
                _ => false,
            };
        }
    }

    false
}

impl Rule for StylePropObject {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut allow = value
            .get(0)
            .and_then(|v| v.get("allow"))
            .and_then(serde_json::Value::as_array)
            .map(|v| {
                v.iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(CompactStr::from)
                    .collect::<Vec<CompactStr>>()
            })
            .unwrap_or_default();

        allow.sort();

        Self(Box::new(StylePropObjectConfig { allow }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(jsx_elem) => {
                let name = match &jsx_elem.opening_element.name {
                    JSXElementName::Identifier(id) => id.name.as_str(),
                    JSXElementName::IdentifierReference(id) => id.name.as_str(),
                    _ => return,
                };

                if self.allow.iter().any(|s| s == name) {
                    return;
                }

                jsx_elem.opening_element.attributes.iter().for_each(|attribute| {
                    if let JSXAttributeItem::Attribute(attribute) = attribute {
                        if is_invalid_style_attribute(attribute, ctx) {
                            let Some(value) = &attribute.value else {
                                return;
                            };

                            ctx.diagnostic(style_prop_object_diagnostic(value.span()));
                        }
                    }
                });
            }
            AstKind::CallExpression(call_expr) => {
                if !is_create_element_call(call_expr) {
                    return;
                }

                let Some(component) = call_expr.arguments.first() else {
                    return;
                };

                let Some(expr) = component.as_expression() else {
                    return;
                };

                let name = match expr {
                    Expression::StringLiteral(literal) => literal.value.as_str(),
                    Expression::Identifier(identifier) => identifier.name.as_str(),
                    _ => return,
                };

                if self.allow.binary_search(&name.into()).is_ok() {
                    return;
                }

                let Some(props) = call_expr.arguments.get(1) else {
                    return;
                };

                let Argument::ObjectExpression(obj_expr) = props else {
                    return;
                };

                for prop in &obj_expr.properties {
                    if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
                        if let Some(prop_name) = obj_prop.key.static_name() {
                            if prop_name == "style"
                                && is_invalid_expression(Some(&obj_prop.value), ctx)
                            {
                                ctx.diagnostic(style_prop_object_diagnostic(obj_prop.value.span()));
                            }
                        }
                    }
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
        (r#"<div style={{ color: "blue" }} />"#, None),
        (r#"<div style={{ color: "ASD" }} />"#, None),
        (r#"<Hello style={{ color: "red" }} />"#, None),
        (
            r#"
              function redDiv() {
                const styles = { color: "red" };
                return <div style={styles} />;
              }
            "#,
            None,
        ),
        (
            r#"
              function redDiv() {
                const styles = { color: "red" };
                return <Hello style={styles} />;
              }
            "#,
            None,
        ),
        (
            r#"
              const styles = { color: "red" };
              function redDiv() {
                return <div style={styles} />;
              }
            "#,
            None,
        ),
        (
            r"
              function redDiv(props) {
                return <div style={props.styles} />;
              }
            ",
            None,
        ),
        (
            r"
              import styles from './styles';
              function redDiv() {
                return <div style={styles} />;
              }
            ",
            None,
        ),
        (
            r"
              import mystyles from './styles';
              const styles = Object.assign({ color: 'red' }, mystyles);
              function redDiv() {
                return <div style={styles} />;
              }
            ",
            None,
        ),
        (
            r#"
              const otherProps = { style: { color: "red" } };
              const { a, b, ...props } = otherProps;
              <div {...props} />
            "#,
            None,
        ),
        (
            r#"
              const styles = Object.assign({ color: 'red' }, mystyles);
              React.createElement("div", { style: styles });
            "#,
            None,
        ),
        (r"<div style></div>", None),
        (
            r"
              React.createElement(MyCustomElem, {
                [style]: true
              }, 'My custom Elem')
            ",
            None,
        ),
        (
            r"
              let style;
              <div style={style}></div>
            ",
            None,
        ),
        (
            r"
              let style = null;
              <div style={style}></div>
            ",
            None,
        ),
        (
            r"
              let style = undefined;
              <div style={style}></div>
            ",
            None,
        ),
        (
            r"
              const otherProps = { style: undefined };
              const { a, b, ...props } = otherProps;
              <div {...props} />
            ",
            None,
        ),
        (
            r#"
              React.createElement("div", {
                style: undefined
              })
            "#,
            None,
        ),
        (
            r#"
              let style;
              React.createElement("div", {
                style
              })
            "#,
            None,
        ),
        ("<div style={null}></div>", None),
        (
            r"
              const props = { style: null };
              <div {...props} />
            ",
            None,
        ),
        (
            r"
              const otherProps = { style: null };
              const { a, b, ...props } = otherProps;
              <div {...props} />
            ",
            None,
        ),
        (
            r#"
              React.createElement("div", {
                style: null
              })
            "#,
            None,
        ),
        (
            r"
              const MyComponent = (props) => {
                React.createElement(MyCustomElem, {
                  ...props
                });
              };
            ",
            None,
        ),
        (
            r#"<MyComponent style="myStyle" />"#,
            Some(serde_json::json!([{ "allow": ["MyComponent"] }])),
        ),
        (
            r#"React.createElement(MyComponent, { style: "mySpecialStyle" })"#,
            Some(serde_json::json!([{ "allow": ["MyComponent"] }])),
        ),
        (
            r"
            let styles: object | undefined
            return <div style={styles} />
          ",
            None,
        ),
        (
            r"
            let styles: CSSProperties | undefined
            return <div style={styles} />
          ",
            None,
        ),
    ];

    let fail = vec![
        (r#"<div style="color: 'red'" />"#, None),
        (r#"<Hello style="color: 'green'" />"#, None),
        (r"<div style={true} />", None),
        (
            r#"
              const styles = `color: "red"`;
              function redDiv2() {
                return <div style={styles} />;
              }
            "#,
            None,
        ),
        (
            r#"
              const styles = 'color: "red"';
              function redDiv2() {
                return <div style={styles} />;
              }
            "#,
            None,
        ),
        (
            r#"
              const styles = 'color: "blue"';
              function redDiv2() {
                return <Hello style={styles} />;
              }
            "#,
            None,
        ),
        (
            r"
              const styles = true;
              function redDiv() {
                return <div style={styles} />;
              }
            ",
            None,
        ),
        (
            r#"<MyComponent style="myStyle" />"#,
            Some(serde_json::json!([{ "allow": ["MyOtherComponent"] }])),
        ),
        (
            r#"React.createElement(MyComponent2, { style: "mySpecialStyle" })"#,
            Some(serde_json::json!([{ "allow": ["MyOtherComponent"] }])),
        ),
        (
            r"
            let styles: string | undefined
            return <div style={styles} />
          ",
            None,
        ),
        (
            r"
            let styles: number | undefined
            return <div style={styles} />
          ",
            None,
        ),
        (
            r"
            let styles: boolean | undefined
            return <div style={styles} />
          ",
            None,
        ),
    ];

    Tester::new(StylePropObject::NAME, StylePropObject::PLUGIN, pass, fail).test_and_snapshot();
}
