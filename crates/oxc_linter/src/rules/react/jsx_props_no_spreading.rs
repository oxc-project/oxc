use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{Expression, JSXElementName, JSXMemberExpression, JSXMemberExpressionObject},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_react_component_name,
};

fn jsx_props_no_spreading_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prop spreading is forbidden").with_label(span)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
enum IgnoreEnforceOption {
    Ignore,
    #[default]
    Enforce,
}

#[derive(Debug, Clone, Default, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct JsxPropsNoSpreadingConfig {
    /// `html` set to `ignore` will ignore all html jsx tags like `div`, `img` etc. Default is set to `enforce`.
    html: IgnoreEnforceOption,
    /// `custom` set to `ignore` will ignore all custom jsx tags like `App`, `MyCustomComponent` etc. Default is set to `enforce`.
    custom: IgnoreEnforceOption,
    /// `explicitSpread` set to `ignore` will ignore spread operators that are explicitly listing all object properties within that spread. Default is set to `enforce`.
    explicit_spread: IgnoreEnforceOption,
    /// Exceptions flip the enforcement behavior for specific components.
    /// For example:
    /// - If `html` is set to `ignore`, an exception for `div` will enforce the rule on `<div>` elements.
    /// - If `custom` is set to `enforce`, an exception for `Foo` will ignore the rule on `<Foo>` components.
    ///
    /// This allows you to override the general setting for individual components.
    exceptions: Vec<CompactStr>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct JsxPropsNoSpreading(Box<JsxPropsNoSpreadingConfig>);

impl std::ops::Deref for JsxPropsNoSpreading {
    type Target = JsxPropsNoSpreadingConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow JSX prop spreading
    ///
    /// ### Why is this bad?
    ///
    /// Enforces that there is no spreading for any JSX attribute. This enhances readability of code by being more explicit about what props are received by the component.
    /// It is also good for maintainability by avoiding passing unintentional extra props and allowing react to emit warnings when invalid HTML props are passed to HTML elements.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <App {...props} />
    /// <MyCustomComponent {...props} some_other_prop={some_other_prop} />
    /// <img {...props} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// const {src, alt} = props;
    /// const {one_prop, two_prop} = otherProps;
    /// <MyCustomComponent one_prop={one_prop} two_prop={two_prop} />
    /// <img src={src} alt={alt} />
    /// ```
    JsxPropsNoSpreading,
    react,
    style,
    config = JsxPropsNoSpreadingConfig
);

impl Rule for JsxPropsNoSpreading {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<JsxPropsNoSpreading>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXSpreadAttribute(spread_attr) = node.kind() else {
            return;
        };

        let AstKind::JSXOpeningElement(jsx_opening_element) =
            ctx.nodes().parent_node(node.id()).kind()
        else {
            return;
        };

        let tag_name = get_tag_name(&jsx_opening_element.name);

        // Check if first character is lowercase (HTML tag convention)
        let is_html_tag = !is_react_component_name(&tag_name);
        // Custom tags: uppercase first char OR contains '.' (member expressions like Nav.Item)
        let is_custom_tag = !is_html_tag || tag_name.contains('.');

        let is_exception = is_exception(&tag_name, &self.exceptions);
        let ignore_html_tags = self.html == IgnoreEnforceOption::Ignore;
        let ignore_custom_tags = self.custom == IgnoreEnforceOption::Ignore;

        // XOR logic: ignore if (setting is ignore AND not exception) OR (setting is enforce AND is exception)
        let should_ignore_html = ignore_html_tags != is_exception;
        if is_html_tag && should_ignore_html {
            return;
        }

        let should_ignore_custom = ignore_custom_tags != is_exception;
        if is_custom_tag && should_ignore_custom {
            return;
        }

        if self.explicit_spread == IgnoreEnforceOption::Ignore
            && let Expression::ObjectExpression(obj_expr) = &spread_attr.argument
            && obj_expr.properties.iter().all(|prop| !prop.is_spread())
        {
            return;
        }

        ctx.diagnostic(jsx_props_no_spreading_diagnostic(spread_attr.span));
    }
}

fn is_exception(tag: &CompactStr, exceptions: &[CompactStr]) -> bool {
    exceptions.contains(tag)
}

fn get_tag_name(name: &JSXElementName<'_>) -> CompactStr {
    match name {
        JSXElementName::Identifier(ident) => ident.name.as_str().into(),
        JSXElementName::IdentifierReference(ident) => ident.name.as_str().into(),
        JSXElementName::MemberExpression(member_expr) => get_member_expr_tag_name(member_expr),
        JSXElementName::NamespacedName(namespaced_name) => format!(
            "{}:{}",
            namespaced_name.namespace.name.as_str(),
            namespaced_name.name.name.as_str()
        )
        .into(),
        JSXElementName::ThisExpression(_) => "this".into(),
    }
}

/// gets full component name, e.g. "components.Group" in <components.Group />
fn get_member_expr_tag_name(member_expr: &JSXMemberExpression) -> CompactStr {
    let object_name = match &member_expr.object {
        JSXMemberExpressionObject::IdentifierReference(ident) => ident.name.as_str(),
        JSXMemberExpressionObject::ThisExpression(_) => "this",
        JSXMemberExpressionObject::MemberExpression(next_expr) => {
            &get_member_expr_tag_name(next_expr)
        }
    };

    format!("{}.{}", object_name, member_expr.property.name.as_str()).into()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			        const {one_prop, two_prop} = props;
			        <App one_prop={one_prop} two_prop={two_prop}/>
			      ",
            None,
        ),
        (
            "
			        const {one_prop, two_prop} = props;
			        <div one_prop={one_prop} two_prop={two_prop}></div>
			      ",
            None,
        ),
        (
            "
			        const newProps = {...props};
			        <App one_prop={newProps.one_prop} two_prop={newProps.two_prop} style={{...styles}}/>
			      ",
            None,
        ),
        (
            r#"
			        const props = {src: "dummy.jpg", alt: "dummy"};
			        <App>
			           <Image {...props}/>
			           <img {...props}/>
			        </App>
			      "#,
            Some(serde_json::json!([{ "exceptions": ["Image", "img"] }])),
        ),
        (
            r#"
			        const props = {src: "dummy.jpg", alt: "dummy"};
			        const { src, alt } = props;
			        <App>
			           <Image {...props}/>
			           <img src={src} alt={alt}/>
			        </App>
			      "#,
            Some(serde_json::json!([{ "custom": "ignore" }])),
        ),
        (
            r#"
			        const props = {src: "dummy.jpg", alt: "dummy"};
			        const { src, alt } = props;
			        <App>
			           <Image {...props}/>
			           <img {...props}/>
			        </App>
			      "#,
            Some(
                serde_json::json!([{ "custom": "enforce", "html": "ignore", "exceptions": ["Image"] }]),
            ),
        ),
        (
            r#"
			        const props = {src: "dummy.jpg", alt: "dummy"};
			        const { src, alt } = props;
			        <App>
			           <img {...props}/>
			           <Image src={src} alt={alt}/>
			           <div {...someOtherProps}/>
			        </App>
			      "#,
            Some(serde_json::json!([{ "html": "ignore" }])),
        ),
        (
            "
			        <App>
			          <Foo {...{ prop1, prop2, prop3 }} />
			        </App>
			      ",
            Some(serde_json::json!([{ "explicitSpread": "ignore" }])),
        ),
        (
            "
			        const props = {};
			        <App>
			           <components.Group {...props}/>
			           <Nav.Item {...props}/>
			        </App>
			      ",
            Some(serde_json::json!([{ "exceptions": ["components.Group", "Nav.Item"] }])),
        ),
        (
            "
			        const props = {};
			        <App>
			           <components.Group {...props}/>
			           <Nav.Item {...props}/>
			        </App>
			      ",
            Some(serde_json::json!([{ "custom": "ignore" }])),
        ),
        (
            "
			        const props = {};
			        <App>
			           <components.Group {...props}/>
			           <Nav.Item {...props}/>
			        </App>
			      ",
            Some(
                serde_json::json!([        {          "custom": "enforce",          "html": "ignore",          "exceptions": ["components.Group", "Nav.Item"],        },      ]),
            ),
        ),
    ];

    let fail = vec![
        (
            "
			        <App {...props}/>
			      ",
            None,
        ),
        (
            "
			        <div {...props}></div>
			      ",
            None,
        ),
        (
            "
			        <App {...props} some_other_prop={some_other_prop}/>
			      ",
            None,
        ),
        (
            r#"
			        const props = {src: "dummy.jpg", alt: "dummy"};
			        <App>
			           <Image {...props}/>
			           <span {...props}/>
			        </App>
			      "#,
            Some(serde_json::json!([{ "exceptions": ["Image", "img"] }])),
        ),
        (
            r#"
			        const props = {src: "dummy.jpg", alt: "dummy"};
			        const { src, alt } = props;
			        <App>
			           <Image {...props}/>
			           <img {...props}/>
			        </App>
			      "#,
            Some(serde_json::json!([{ "custom": "ignore" }])),
        ),
        (
            r#"
			        const props = {src: "dummy.jpg", alt: "dummy"};
			        const { src, alt } = props;
			        <App>
			           <Image {...props}/>
			           <img {...props}/>
			        </App>
			      "#,
            Some(serde_json::json!([{ "html": "ignore", "exceptions": ["Image", "img"] }])),
        ),
        (
            r#"
			        const props = {src: "dummy.jpg", alt: "dummy"};
			        const { src, alt } = props;
			        <App>
			           <Image {...props}/>
			           <img {...props}/>
			           <div {...props}/>
			        </App>
			      "#,
            Some(
                serde_json::json!([{ "custom": "ignore", "html": "ignore", "exceptions": ["Image", "img"] }]),
            ),
        ),
        (
            r#"
			        const props = {src: "dummy.jpg", alt: "dummy"};
			        const { src, alt } = props;
			        <App>
			           <img {...props}/>
			           <Image {...props}/>
			        </App>
			      "#,
            Some(serde_json::json!([{ "html": "ignore" }])),
        ),
        (
            "
			        <App>
			          <Foo {...{ prop1, prop2, prop3 }} />
			        </App>
			      ",
            None,
        ),
        (
            "
			        <App>
			          <Foo {...{ prop1, ...rest }} />
			        </App>
			      ",
            Some(serde_json::json!([{ "explicitSpread": "ignore" }])),
        ),
        (
            "
			        <App>
			          <Foo {...{ ...props }} />
			        </App>
			      ",
            Some(serde_json::json!([{ "explicitSpread": "ignore" }])),
        ),
        (
            "
			        <App>
			          <Foo {...props } />
			        </App>
			      ",
            Some(serde_json::json!([{ "explicitSpread": "ignore" }])),
        ),
        (
            "
			        const props = {};
			        <App>
			           <components.Group {...props}/>
			           <Nav.Item {...props}/>
			        </App>
			      ",
            Some(
                serde_json::json!([{ "exceptions": ["components.DropdownIndicator", "Nav.Item"] }]),
            ),
        ),
    ];

    Tester::new(JsxPropsNoSpreading::NAME, JsxPropsNoSpreading::PLUGIN, pass, fail)
        .test_and_snapshot();
}
