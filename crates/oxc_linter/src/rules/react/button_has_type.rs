use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::{get_prop_value, has_jsx_prop_ignore_case, is_create_element_call},
};
use oxc_ast::{
    AstKind,
    ast::{
        Argument, Expression, JSXAttributeItem, JSXAttributeValue, JSXElementName,
        ObjectPropertyKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

fn missing_type_prop(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`button` elements must have an explicit `type` attribute.")
        .with_help("Add a `type` attribute to the `button` element.")
        .with_label(span)
}

fn invalid_type_prop(span: Span, allowed_types: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("`button` elements must have a valid `type` attribute.")
        .with_help(format!(
            "Change the `type` attribute to one of the allowed values: {allowed_types}."
        ))
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ButtonHasType {
    /// If true, allow `type="button"`.
    button: bool,
    /// If true, allow `type="submit"`.
    submit: bool,
    /// If true, allow `type="reset"`.
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
    /// The default value of `type` attribute for `button` HTML element is
    /// `"submit"` which is often not the desired behavior and may lead to
    /// unexpected page reloads.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <button />
    /// <button type="foo" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <button type="button" />
    /// <button type="submit" />
    /// ```
    ButtonHasType,
    react,
    restriction,
    config = ButtonHasType,
);

impl Rule for ButtonHasType {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<ButtonHasType>>(value)
            .unwrap_or_default()
            .into_inner()
    }

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

                has_jsx_prop_ignore_case(jsx_el, "type").map_or_else(
                    || {
                        ctx.diagnostic(missing_type_prop(identifier.span));
                    },
                    |button_type_prop| {
                        if !self.is_valid_button_type_prop(button_type_prop) {
                            let allowed_types = self.allowed_types_message();
                            ctx.diagnostic(invalid_type_prop(
                                button_type_prop.span(),
                                &allowed_types,
                            ));
                        }
                    },
                );
            }
            AstKind::CallExpression(call_expr) => {
                if is_create_element_call(call_expr) {
                    let Some(Argument::StringLiteral(str)) = call_expr.arguments.first() else {
                        return;
                    };

                    if str.value.as_str() != "button" {
                        return;
                    }

                    if let Some(Argument::ObjectExpression(obj_expr)) = call_expr.arguments.get(1) {
                        obj_expr
                            .properties
                            .iter()
                            .find_map(|prop| {
                                if let ObjectPropertyKind::ObjectProperty(prop) = prop
                                    && prop.key.is_specific_static_name("type")
                                {
                                    return Some(prop);
                                }

                                None
                            })
                            .map_or_else(
                                || {
                                    ctx.diagnostic(missing_type_prop(obj_expr.span));
                                },
                                |type_prop| {
                                    if !self.is_valid_button_type_prop_expression(&type_prop.value)
                                    {
                                        let allowed_types = self.allowed_types_message();
                                        ctx.diagnostic(invalid_type_prop(
                                            type_prop.span,
                                            &allowed_types,
                                        ));
                                    }
                                },
                            );
                    } else {
                        ctx.diagnostic(missing_type_prop(call_expr.span));
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

impl ButtonHasType {
    fn allowed_types_message(&self) -> String {
        let mut types = Vec::new();
        if self.button {
            types.push("`button`");
        }
        if self.submit {
            types.push("`submit`");
        }
        if self.reset {
            types.push("`reset`");
        }

        match types.len() {
            0 => String::new(),
            1 => types[0].to_string(),
            2 => format!("{} or {}", types[0], types[1]),
            _ => {
                let last = types.pop().unwrap();
                format!("{}, or {}", types.join(", "), last)
            }
        }
    }

    fn is_valid_button_type_prop(&self, item: &JSXAttributeItem) -> bool {
        match get_prop_value(item) {
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                if let Some(expr) = container.expression.as_expression() {
                    self.is_valid_button_type_prop_expression(expr)
                } else {
                    false
                }
            }
            Some(JSXAttributeValue::StringLiteral(str)) => {
                self.is_valid_button_type_prop_string_literal(str.value.as_str())
            }
            _ => false,
        }
    }

    fn is_valid_button_type_prop_expression(&self, expr: &Expression) -> bool {
        match expr.without_parentheses() {
            Expression::StringLiteral(str) => {
                self.is_valid_button_type_prop_string_literal(str.value.as_str())
            }
            Expression::TemplateLiteral(template_literal) => template_literal
                .single_quasi()
                .is_some_and(|quasi| self.is_valid_button_type_prop_string_literal(quasi.as_str())),
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
        (r#"createElement("span")"#, None),
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
            r#"React.createElement("button", {type: "button"})"#,
            Some(serde_json::json!([{ "reset": false, "submit": false }])),
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
        (r#"createElement("button")"#, None),
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
        (
            r#"React.createElement("button", {type: condition ? "reset" : "button"})"#,
            Some(serde_json::json!([{ "reset": false, "submit": false }])),
        ),
        (r#"Foo.createElement("button")"#, None),
        (
            r"function Button({ type, ...extraProps }) { const button = type; return <button type={button} {...extraProps} />; }",
            None,
        ),
    ];

    Tester::new(ButtonHasType::NAME, ButtonHasType::PLUGIN, pass, fail).test_and_snapshot();
}
