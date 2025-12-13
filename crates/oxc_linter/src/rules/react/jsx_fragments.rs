use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_jsx_fragment,
};

fn jsx_fragments_diagnostic(span: Span, mode: FragmentMode) -> OxcDiagnostic {
    let msg = if mode == FragmentMode::Element {
        "Standard form for React fragments is preferred."
    } else {
        "Shorthand form for React fragments is preferred."
    };
    let help = if mode == FragmentMode::Element {
        "Use `<React.Fragment></React.Fragment>` instead of `<></>`."
    } else {
        "Use `<></>` instead of `<React.Fragment></React.Fragment>`."
    };
    OxcDiagnostic::warn(msg).with_help(help).with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(untagged)]
pub enum JsxFragments {
    Mode(FragmentMode),
    Object { mode: FragmentMode },
}

impl Default for JsxFragments {
    fn default() -> Self {
        JsxFragments::Mode(FragmentMode::Syntax)
    }
}

impl JsxFragments {
    fn mode(&self) -> FragmentMode {
        match self {
            JsxFragments::Mode(m) => *m,
            JsxFragments::Object { mode } => *mode,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Copy, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FragmentMode {
    /// This is the default mode. It will enforce the shorthand syntax for React fragments, with one exception.
    /// Keys or attributes are not supported by the shorthand syntax, so the rule will not warn on standard-form fragments that use those.
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <React.Fragment><Foo /></React.Fragment>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <><Foo /></>
    /// ```
    ///
    /// ```jsx
    /// <React.Fragment key="key"><Foo /></React.Fragment>
    /// ```
    #[default]
    Syntax,
    /// This mode enforces the standard form for React fragments.
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <><Foo /></>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <React.Fragment><Foo /></React.Fragment>
    /// ```
    ///
    /// ```jsx
    /// <React.Fragment key="key"><Foo /></React.Fragment>
    /// ```
    Element,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the shorthand or standard form for React Fragments.
    ///
    /// ### Why is this bad?
    ///
    /// Makes code using fragments more consistent one way or the other.
    JsxFragments,
    react,
    style,
    fix,
    config = FragmentMode,
);

impl Rule for JsxFragments {
    // Generally we should prefer the string-only syntax for compatibility with the original ESLint rule,
    // but we originally implemented the rule with only the object syntax, so we support both now.
    fn from_configuration(value: Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<JsxFragments>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(jsx_elem) if self.mode() == FragmentMode::Syntax => {
                let Some(closing_element) = &jsx_elem.closing_element else {
                    return;
                };
                if !is_jsx_fragment(&jsx_elem.opening_element)
                    || !jsx_elem.opening_element.attributes.is_empty()
                {
                    return;
                }
                ctx.diagnostic_with_fix(
                    jsx_fragments_diagnostic(jsx_elem.opening_element.name.span(), self.mode()),
                    |fixer| {
                        let before_opening_tag = ctx.source_range(Span::new(
                            jsx_elem.span().start,
                            jsx_elem.opening_element.span().start,
                        ));
                        let between_opening_tag_and_closing_tag = ctx.source_range(Span::new(
                            jsx_elem.opening_element.span().end,
                            closing_element.span().start,
                        ));
                        let after_closing_tag = ctx.source_range(Span::new(
                            closing_element.span().end,
                            jsx_elem.span().end,
                        ));
                        let mut replacement = String::new();
                        replacement.push_str(before_opening_tag);
                        replacement.push_str("<>");
                        replacement.push_str(between_opening_tag_and_closing_tag);
                        replacement.push_str("</>");
                        replacement.push_str(after_closing_tag);
                        fixer.replace(jsx_elem.span(), replacement)
                    },
                );
            }
            AstKind::JSXFragment(jsx_frag) if self.mode() == FragmentMode::Element => {
                ctx.diagnostic_with_fix(
                    jsx_fragments_diagnostic(jsx_frag.opening_fragment.span(), self.mode()),
                    |fixer| {
                        let before_opening_tag = ctx.source_range(Span::new(
                            jsx_frag.span().start,
                            jsx_frag.opening_fragment.span().start,
                        ));
                        let between_opening_tag_and_closing_tag = ctx.source_range(Span::new(
                            jsx_frag.opening_fragment.span().end,
                            jsx_frag.closing_fragment.span().start,
                        ));
                        let after_closing_tag = ctx.source_range(Span::new(
                            jsx_frag.closing_fragment.span().end,
                            jsx_frag.span().end,
                        ));
                        let mut replacement = String::new();
                        replacement.push_str(before_opening_tag);
                        replacement.push_str("<React.Fragment>");
                        replacement.push_str(between_opening_tag_and_closing_tag);
                        replacement.push_str("</React.Fragment>");
                        replacement.push_str(after_closing_tag);
                        fixer.replace(jsx_frag.span(), replacement)
                    },
                );
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("<><Foo /></>", None),
        (r#"<Fragment key="key"><Foo /></Fragment>"#, None),
        (r#"<React.Fragment key="key"><Foo /></React.Fragment>"#, None),
        ("<Fragment />", None),
        ("<React.Fragment />", None),
        // Configuration can be done via a string directly, or an object with the `mode` field.
        ("<><Foo /></>", Some(json!(["syntax"]))),
        ("<><Foo /></>", Some(json!([{"mode": "syntax"}]))),
        ("<React.Fragment><Foo /></React.Fragment>", Some(json!(["element"]))),
        ("<React.Fragment><Foo /></React.Fragment>", Some(json!([{"mode": "element"}]))),
    ];

    let fail = vec![
        ("<Fragment><Foo /></Fragment>", None),
        ("<React.Fragment><Foo /></React.Fragment>", None),
        ("<><Foo /></>", Some(json!(["element"]))),
        ("<><Foo /></>", Some(json!([{"mode": "element"}]))),
    ];

    let fix = vec![
        ("<Fragment><Foo /></Fragment>", "<><Foo /></>", None),
        ("<React.Fragment><Foo /></React.Fragment>", "<><Foo /></>", None),
        ("<><Foo /></>", "<React.Fragment><Foo /></React.Fragment>", Some(json!(["element"]))),
        (
            "<><Foo /></>",
            "<React.Fragment><Foo /></React.Fragment>",
            Some(json!([{"mode": "element"}])),
        ),
    ];

    Tester::new(JsxFragments::NAME, JsxFragments::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
