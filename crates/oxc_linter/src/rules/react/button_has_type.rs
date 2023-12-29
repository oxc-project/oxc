use oxc_ast::{
    ast::{
        Argument, Expression, JSXAttributeItem, JSXAttributeValue, JSXElementName, JSXExpression,
        JSXExpressionContainer, ObjectPropertyKind,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_prop_value, has_jsx_prop_lowercase, is_create_element_call},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
enum ButtonHasTypeDiagnostic {
    #[error("eslint-plugin-react(button-has-type): `button` elements must have an explicit `type` attribute.")]
    #[diagnostic(severity(warning), help("Add a `type` attribute to the `button` element."))]
    MissingTypeProp(#[label] Span),

    #[error("eslint-plugin-react(button-has-type): `button` elements must have a valid `type` attribute.")]
    #[diagnostic(severity(warning), help("Change the `type` attribute to one of the allowed values: `button`, `submit`, or `reset`."))]
    InvalidTypeProp(#[label] Span),
}

#[derive(Debug, Clone)]
pub struct ButtonHasType {
    button: bool,
    submit: bool,
    reset: bool,
}

impl Default for ButtonHasType {
    fn default() -> Self {
        Self { button: true, submit: true, reset: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces explicit `type` attribute for all the `button` HTML elements.
    ///
    /// ### Why is this bad?
    ///
    /// The default value of `type` attribute for `button` HTML element is `"submit"` which is often not the desired behavior and may lead to unexpected page reloads.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <button />
    /// <button type="foo" />
    ///
    /// // Good
    /// <button type="button" />
    /// <button type="submit" />
    /// ```
    ButtonHasType,
    restriction
);

impl Rule for ButtonHasType {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXOpeningElement(jsx_el) => {
                let JSXElementName::Identifier(identifier) = &jsx_el.name else {
                    return;
                };

                let name = identifier.name.as_str();
                if name != "button" {
                    return;
                }

                has_jsx_prop_lowercase(jsx_el, "type").map_or_else(
                    || {
                        ctx.diagnostic(ButtonHasTypeDiagnostic::MissingTypeProp(identifier.span));
                    },
                    |button_type_prop| {
                        if !self.is_valid_button_type_prop(button_type_prop) {
                            ctx.diagnostic(ButtonHasTypeDiagnostic::InvalidTypeProp(
                                button_type_prop.span(),
                            ));
                        }
                    },
                );
            }
            AstKind::CallExpression(call_expr) => {
                if is_create_element_call(call_expr) {
                    let Some(Argument::Expression(Expression::StringLiteral(str))) =
                        call_expr.arguments.first()
                    else {
                        return;
                    };

                    if str.value.as_str() != "button" {
                        return;
                    }

                    if let Some(Argument::Expression(Expression::ObjectExpression(obj_expr))) =
                        call_expr.arguments.get(1)
                    {
                        obj_expr
                            .properties
                            .iter()
                            .find_map(|prop| {
                                if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                                    if prop.key.is_specific_static_name("type") {
                                        return Some(prop);
                                    }
                                }

                                None
                            })
                            .map_or_else(
                                || {
                                    ctx.diagnostic(ButtonHasTypeDiagnostic::MissingTypeProp(
                                        obj_expr.span,
                                    ));
                                },
                                |type_prop| {
                                    if !self.is_valid_button_type_prop_expression(&type_prop.value)
                                    {
                                        ctx.diagnostic(ButtonHasTypeDiagnostic::InvalidTypeProp(
                                            type_prop.span,
                                        ));
                                    }
                                },
                            );
                    } else {
                        ctx.diagnostic(ButtonHasTypeDiagnostic::MissingTypeProp(call_expr.span));
                    }
                }
            }
            _ => {}
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let value = value.as_array().and_then(|arr| arr.first()).and_then(|val| val.as_object());

        Self {
            button: value
                .and_then(|val| val.get("button").and_then(serde_json::Value::as_bool))
                .unwrap_or(true),
            submit: value
                .and_then(|val| val.get("submit").and_then(serde_json::Value::as_bool))
                .unwrap_or(true),
            reset: value
                .and_then(|val| val.get("reset").and_then(serde_json::Value::as_bool))
                .unwrap_or(true),
        }
    }
}

impl ButtonHasType {
    fn is_valid_button_type_prop(&self, item: &JSXAttributeItem) -> bool {
        match get_prop_value(item) {
            Some(JSXAttributeValue::ExpressionContainer(JSXExpressionContainer {
                expression: JSXExpression::Expression(expr),
                ..
            })) => self.is_valid_button_type_prop_expression(expr),
            Some(JSXAttributeValue::StringLiteral(str)) => {
                self.is_valid_button_type_prop_string_literal(str.value.as_str())
            }
            _ => false,
        }
    }

    fn is_valid_button_type_prop_expression(&self, expr: &Expression) -> bool {
        match expr.without_parenthesized() {
            Expression::StringLiteral(str) => {
                self.is_valid_button_type_prop_string_literal(str.value.as_str())
            }
            Expression::TemplateLiteral(template_literal) => {
                if !template_literal.is_no_substitution_template() {
                    return false;
                }
                if let Some(quasi) = template_literal.quasi() {
                    return self.is_valid_button_type_prop_string_literal(quasi.as_str());
                }
                false
            }
            Expression::ConditionalExpression(conditional_expr) => {
                self.is_valid_button_type_prop_expression(&conditional_expr.consequent)
                    && self.is_valid_button_type_prop_expression(&conditional_expr.alternate)
            }
            _ => false,
        }
    }

    fn is_valid_button_type_prop_string_literal(&self, s: &str) -> bool {
        match s {
            "button" => self.button,
            "submit" => self.submit,
            "reset" => self.reset,
            _ => false,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"<span/>", None),
        (r#"<span type="foo"/>"#, None),
        (r#"<button type="button"/>"#, None),
        (r#"<button type="submit"/>"#, None),
        (r#"<button type="reset"/>"#, None),
        (r#"<button type={"button"}/>"#, None),
        (r"<button type={'button'}/>", None),
        (r"<button type={`button`}/>", None),
        (r#"<button type={condition ? "button" : "submit"}/>"#, None),
        (r"<button type={condition ? 'button' : 'submit'}/>", None),
        (r"<button type={condition ? `button` : `submit`}/>", None),
        (r#"<button type="button"/>"#, Some(serde_json::json!([{ "reset": false }]))),
        (r#"React.createElement("span")"#, None),
        (r#"React.createElement("span", {type: "foo"})"#, None),
        (r#"React.createElement("button", {type: "button"})"#, None),
        (r#"React.createElement("button", {type: 'button'})"#, None),
        (r#"React.createElement("button", {type: `button`})"#, None),
        (r#"React.createElement("button", {type: "submit"})"#, None),
        (r#"React.createElement("button", {type: 'submit'})"#, None),
        (r#"React.createElement("button", {type: `submit`})"#, None),
        (r#"React.createElement("button", {type: "reset"})"#, None),
        (r#"React.createElement("button", {type: 'reset'})"#, None),
        (r#"React.createElement("button", {type: `reset`})"#, None),
        (r#"React.createElement("button", {type: condition ? "button" : "submit"})"#, None),
        (r#"React.createElement("button", {type: condition ? 'button' : 'submit'})"#, None),
        (r#"React.createElement("button", {type: condition ? `button` : `submit`})"#, None),
        (
            r#"React.createElement("button", {type: "button"})"#,
            Some(serde_json::json!([{ "reset": false }])),
        ),
        (
            r#"
			        function MyComponent(): ReactElement {
			          const buttonProps: (Required<Attributes> & ButtonHTMLAttributes<HTMLButtonElement>)[] = [
			            {
			              children: 'test',
			              key: 'test',
			              onClick: (): void => {
			                return;
			              },
			            },
			          ];
			
			          return <>
			            {
			              buttonProps.map(
			                ({ key, ...props }: Required<Attributes> & ButtonHTMLAttributes<HTMLButtonElement>): ReactElement =>
			                  <button key={key} type="button" {...props} />
			              )
			            }
			          </>;
			        }
			      "#,
            None,
        ),
    ];

    let fail = vec![
        (r"<button/>", None),
        (r#"<button type="foo"/>"#, None),
        (r"<button type={foo}/>", None),
        (r#"<button type={"foo"}/>"#, None),
        (r"<button type={'foo'}/>", None),
        (r"<button type={`foo`}/>", None),
        (r"<button type={`button${foo}`}/>", None),
        (r#"<button type="reset"/>"#, Some(serde_json::json!([{ "reset": false }]))),
        (r#"<button type={condition ? "button" : foo}/>"#, None),
        (r#"<button type={condition ? "button" : "foo"}/>"#, None),
        (
            r#"<button type={condition ? "button" : "reset"}/>"#,
            Some(serde_json::json!([{ "reset": false }])),
        ),
        (r#"<button type={condition ? foo : "button"}/>"#, None),
        (r#"<button type={condition ? "foo" : "button"}/>"#, None),
        (r"button type/>", None),
        (
            r#"<button type={condition ? "reset" : "button"}/>"#,
            Some(serde_json::json!([{ "reset": false }])),
        ),
        (r#"React.createElement("button")"#, None),
        (r#"React.createElement("button", {type: foo})"#, None),
        (r#"React.createElement("button", {type: "foo"})"#, None),
        (
            r#"React.createElement("button", {type: "reset"})"#,
            Some(serde_json::json!([{ "reset": false }])),
        ),
        (r#"React.createElement("button", {type: condition ? "button" : foo})"#, None),
        (r#"React.createElement("button", {type: condition ? "button" : "foo"})"#, None),
        (
            r#"React.createElement("button", {type: condition ? "button" : "reset"})"#,
            Some(serde_json::json!([{ "reset": false }])),
        ),
        (r#"React.createElement("button", {type: condition ? foo : "button"})"#, None),
        (r#"React.createElement("button", {type: condition ? "foo" : "button"})"#, None),
        (
            r#"React.createElement("button", {type: condition ? "reset" : "button"})"#,
            Some(serde_json::json!([{ "reset": false }])),
        ),
        (r#"Foo.createElement("button")"#, None),
        (
            r"function Button({ type, ...extraProps }) { const button = type; return <button type={button} {...extraProps} />; }",
            None,
        ),
    ];

    Tester::new(ButtonHasType::NAME, pass, fail).test_and_snapshot();
}
