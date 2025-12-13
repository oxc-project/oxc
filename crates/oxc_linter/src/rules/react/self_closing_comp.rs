use oxc_ast::{
    AstKind,
    ast::{JSXChild, JSXElementName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    globals::HTML_TAG,
    rule::{DefaultRuleConfig, Rule},
};

fn self_closing_comp_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary closing tag")
        .with_help("Make the component self closing")
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SelfClosingComp {
    /// Whether to enforce self-closing for custom components.
    component: bool,
    /// Whether to enforce self-closing for native HTML elements.
    html: bool,
}

impl Default for SelfClosingComp {
    fn default() -> Self {
        Self { component: true, html: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects components without children which can be self-closed to avoid
    /// unnecessary extra closing tags.
    ///
    /// ### Why is this bad?
    ///
    /// Components without children don't need explicit closing tags. Using
    /// self-closing syntax makes code more concise and reduces visual clutter.
    /// It also follows common React and JSX conventions for empty elements.
    ///
    /// A self-closing component which contains whitespace is allowed except
    /// when it also contains a newline.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// const elem = <Component linter="oxlint"></Component>
    /// const dom_elem = <div id="oxlint"></div>
    /// const welem = <div id="oxlint">
    ///
    /// </div>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// const elem = <Component linter="oxlint" />
    /// const welem = <Component linter="oxlint" > </Component>
    /// const dom_elem = <div id="oxlint" />
    /// ```
    SelfClosingComp,
    react,
    style,
    fix,
    config = SelfClosingComp,
);

impl Rule for SelfClosingComp {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<SelfClosingComp>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXElement(jsx_el) = node.kind() else {
            return;
        };

        if jsx_el.closing_element.is_none() {
            return;
        }

        if jsx_el.children.len() > 1 {
            return;
        }

        // The eslint react rule disallows multiline whitespace lines, but allows lines with
        // whitespace
        if jsx_el.children.len() == 1 {
            let JSXChild::Text(jsx_text) = &jsx_el.children[0] else {
                return;
            };

            if !(jsx_text.value.contains('\n') && jsx_text.value.chars().all(char::is_whitespace)) {
                return;
            }
        }

        let Some(jsx_closing_elem) = &jsx_el.closing_element else {
            return;
        };

        let is_comp = matches!(
            jsx_el.opening_element.name,
            JSXElementName::MemberExpression(_) | JSXElementName::NamespacedName(_)
        );

        let mut is_dom_comp = false;
        if !is_comp && let Some(tag_name) = jsx_el.opening_element.name.get_identifier_name() {
            is_dom_comp = HTML_TAG.contains(tag_name.as_str());
        }

        if self.html && is_dom_comp || self.component && !is_dom_comp {
            ctx.diagnostic_with_fix(self_closing_comp_diagnostic(jsx_closing_elem.span), |fixer| {
                fixer.replace(
                    Span::new(jsx_el.opening_element.span.end - 1, jsx_closing_elem.span.end),
                    " />",
                )
            });
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
        (r#"var HelloJohn = <Hello name="John" />;"#, None),
        (r#"var HelloJohn = <Hello.Compound name="John" />;"#, None),
        (r#"var Profile = <Hello name="John"><img src="picture.png" /></Hello>;"#, None),
        (
            r#"var Profile = <Hello.Compound name="John"><img src="picture.png" /></Hello.Compound>;"#,
            None,
        ),
        (
            r#"
			        <Hello>
			          <Hello name="John" />
			        </Hello>
			      "#,
            None,
        ),
        (
            r#"
			        <Hello.Compound>
			          <Hello.Compound name="John" />
			        </Hello.Compound>
			      "#,
            None,
        ),
        (r#"var HelloJohn = <Hello name="John"> </Hello>;"#, None),
        (r#"var HelloJohn = <Hello.Compound name="John"> </Hello.Compound>;"#, None),
        (r#"var HelloJohn = <Hello name="John">        </Hello>;"#, None),
        (r#"var HelloJohn = <Hello.Compound name="John">        </Hello.Compound>;"#, None),
        ("var HelloJohn = <div>&nbsp;</div>;", None),
        ("var HelloJohn = <div>{' '}</div>;", None),
        (r#"var HelloJohn = <Hello name="John">&nbsp;</Hello>;"#, None),
        (r#"var HelloJohn = <Hello.Compound name="John">&nbsp;</Hello.Compound>;"#, None),
        (r#"var HelloJohn = <Hello name="John" />;"#, Some(serde_json::json!([]))),
        (r#"var HelloJohn = <Hello.Compound name="John" />;"#, Some(serde_json::json!([]))),
        (
            r#"var Profile = <Hello name="John"><img src="picture.png" /></Hello>;"#,
            Some(serde_json::json!([])),
        ),
        (
            r#"var Profile = <Hello.Compound name="John"><img src="picture.png" /></Hello.Compound>;"#,
            Some(serde_json::json!([])),
        ),
        (
            r#"<Hello>
		    <Hello name="John" />
		    </Hello>
	    "#,
            Some(serde_json::json!([])),
        ),
        (
            r#"<Hello.Compound>
		<Hello.Compound name="John" />
		</Hello.Compound>
	    "#,
            Some(serde_json::json!([])),
        ),
        ("var HelloJohn = <div> </div>;", Some(serde_json::json!([]))),
        ("var HelloJohn = <div>        </div>;", Some(serde_json::json!([]))),
        ("var HelloJohn = <div>&nbsp;</div>;", Some(serde_json::json!([]))),
        ("var HelloJohn = <div>{' '}</div>;", Some(serde_json::json!([]))),
        (r#"var HelloJohn = <Hello name="John">&nbsp;</Hello>;"#, Some(serde_json::json!([]))),
        (
            r#"var HelloJohn = <Hello.Compound name="John">&nbsp;</Hello.Compound>;"#,
            Some(serde_json::json!([])),
        ),
        (
            r#"var HelloJohn = <Hello name="John"></Hello>;"#,
            Some(serde_json::json!([{ "component": false }])),
        ),
        (
            r#"var HelloJohn = <Hello.Compound name="John"></Hello.Compound>;"#,
            Some(serde_json::json!([{ "component": false }])),
        ),
        (
            r#"var HelloJohn = <Hello name="John">
			</Hello>;"#,
            Some(serde_json::json!([{ "component": false }])),
        ),
        (
            r#"var HelloJohn = <Hello.Compound name="John">
			</Hello.Compound>;"#,
            Some(serde_json::json!([{ "component": false }])),
        ),
        (
            r#"var HelloJohn = <Hello name="John"> </Hello>;"#,
            Some(serde_json::json!([{ "component": false }])),
        ),
        (
            r#"var HelloJohn = <Hello.Compound name="John"> </Hello.Compound>;"#,
            Some(serde_json::json!([{ "component": false }])),
        ),
        (
            r#"var contentContainer = <div className="content" />;"#,
            Some(serde_json::json!([{ "html": true }])),
        ),
        (
            r#"var contentContainer = <div className="content"><img src="picture.png" /></div>;"#,
            Some(serde_json::json!([{ "html": true }])),
        ),
        (
            r#"
			        <div>
			          <div className="content" />
			        </div>
			      "#,
            Some(serde_json::json!([{ "html": true }])),
        ),
    ];

    let fail = vec![
        (r#"var contentContainer = <div className="content"></div>;"#, None),
        (r#"var contentContainer = <div className="content"></div>;"#, Some(serde_json::json!([]))),
        (r#"var HelloJohn = <Hello name="John"></Hello>;"#, None),
        (r#"var CompoundHelloJohn = <Hello.Compound name="John"></Hello.Compound>;"#, None),
        (
            r#"const HelloJohn = <Hello name="John">
			</Hello>;"#,
            None,
        ),
        (
            r#"var HelloJohn = <Hello.Compound name="John">
			</Hello.Compound>;"#,
            None,
        ),
        (r#"var HelloJohn = <Hello name="John"></Hello>;"#, Some(serde_json::json!([]))),
        (
            r#"var HelloJohn = <Hello.Compound name="John"></Hello.Compound>;"#,
            Some(serde_json::json!([])),
        ),
        (
            r#"var HelloJohn = <Hello name="John">
			</Hello>;"#,
            Some(serde_json::json!([])),
        ),
        (
            r#"var HelloJohn = <Hello.Compound name="John">
			</Hello.Compound>;"#,
            Some(serde_json::json!([])),
        ),
        (
            r#"var contentContainer = <div className="content"></div>;"#,
            Some(serde_json::json!([{ "html": true }])),
        ),
        (
            r#"var contentContainer = <div className="content">
			</div>;"#,
            Some(serde_json::json!([{ "html": true }])),
        ),
    ];

    let fix = vec![
        (
            r#"var contentContainer = <div className="content"></div>;"#,
            r#"var contentContainer = <div className="content" />;"#,
            None,
        ),
        (
            r#"var contentContainer = <div className="content"></div>;"#,
            r#"var contentContainer = <div className="content" />;"#,
            Some(serde_json::json!([])),
        ),
        (
            r#"var HelloJohn = <Hello name="John"></Hello>;"#,
            r#"var HelloJohn = <Hello name="John" />;"#,
            None,
        ),
        (
            r#"var CompoundHelloJohn = <Hello.Compound name="John"></Hello.Compound>;"#,
            r#"var CompoundHelloJohn = <Hello.Compound name="John" />;"#,
            None,
        ),
        (
            r#"var HelloJohn = <Hello name="John">
			</Hello>;"#,
            r#"var HelloJohn = <Hello name="John" />;"#,
            None,
        ),
        (
            r#"var HelloJohn = <Hello.Compound name="John">
			</Hello.Compound>;"#,
            r#"var HelloJohn = <Hello.Compound name="John" />;"#,
            None,
        ),
        (
            r#"var HelloJohn = <Hello name="John"></Hello>;"#,
            r#"var HelloJohn = <Hello name="John" />;"#,
            Some(serde_json::json!([])),
        ),
        (
            r#"var HelloJohn = <Hello.Compound name="John"></Hello.Compound>;"#,
            r#"var HelloJohn = <Hello.Compound name="John" />;"#,
            Some(serde_json::json!([])),
        ),
        (
            r#"var HelloJohn = <Hello name="John">
			</Hello>;"#,
            r#"var HelloJohn = <Hello name="John" />;"#,
            Some(serde_json::json!([])),
        ),
        (
            r#"var HelloJohn = <Hello.Compound name="John">
			</Hello.Compound>;"#,
            r#"var HelloJohn = <Hello.Compound name="John" />;"#,
            Some(serde_json::json!([])),
        ),
        (
            r#"var contentContainer = <div className="content"></div>;"#,
            r#"var contentContainer = <div className="content" />;"#,
            Some(serde_json::json!([{ "html": true }])),
        ),
        (
            r#"var contentContainer = <div className="content">
			</div>;"#,
            r#"var contentContainer = <div className="content" />;"#,
            Some(serde_json::json!([{ "html": true }])),
        ),
    ];
    Tester::new(SelfClosingComp::NAME, SelfClosingComp::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
