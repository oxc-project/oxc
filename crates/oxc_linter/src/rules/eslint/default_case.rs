use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::{Regex, RegexBuilder};

use crate::{context::LintContext, rule::Rule, AstNode};

fn default_case_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Require default cases in switch statements.")
        .with_help("Add a default case.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DefaultCase(Box<DefaultCaseConfig>);

#[derive(Debug, Default, Clone)]
pub struct DefaultCaseConfig {
    comment_pattern: Option<Regex>,
}

impl std::ops::Deref for DefaultCase {
    type Target = DefaultCaseConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require default cases in switch statements
    ///
    /// ### Why is this bad?
    ///
    /// Some code conventions require that all switch statements have a default case, even if the
    /// default case is empty.
    ///
    /// ### Example
    /// ```javascript
    /// switch (foo) {
    ///   case 1:
    ///     break;
    /// }
    /// ```
    DefaultCase,
    restriction,
);

impl Rule for DefaultCase {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut cfg = DefaultCaseConfig::default();

        if let Some(config) = value.get(0) {
            if let Some(val) = config.get("commentPattern").and_then(serde_json::Value::as_str) {
                cfg.comment_pattern = RegexBuilder::new(val).case_insensitive(true).build().ok();
            }
        }

        Self(Box::new(cfg))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::SwitchStatement(switch) = node.kind() {
            let cases = &switch.cases;

            if cases.is_empty() || cases.iter().any(|case| case.test.is_none()) {
                return;
            }

            let Some(last_case) = cases.last() else {
                return;
            };

            let has_default_comment = ctx
                .semantic()
                .comments_range(last_case.span.start..switch.span.end)
                .last()
                .is_some_and(|comment| {
                    let raw = comment.span.source_text(ctx.semantic().source_text()).trim();
                    match &self.comment_pattern {
                        Some(comment_pattern) => comment_pattern.is_match(raw),
                        None => raw.eq_ignore_ascii_case("no default"),
                    }
                });

            if !has_default_comment {
                ctx.diagnostic(default_case_diagnostic(switch.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("switch (a) { case 1: break; default: break; }", None),
        ("switch (a) { case 1: break; case 2: default: break; }", None),
        (
            "switch (a) { case 1: break; default: break;
			 //no default
			 }",
            None,
        ),
        (
            "switch (a) {
			    case 1: break;

			//oh-oh
			 // no default
			 }",
            None,
        ),
        (
            "switch (a) {
			    case 1:

			// no default
			 }",
            None,
        ),
        (
            "switch (a) {
			    case 1:

			// No default
			 }",
            None,
        ),
        (
            "switch (a) {
			    case 1:

			// no deFAUlt
			 }",
            None,
        ),
        (
            "switch (a) {
			    case 1:

			// NO DEFAULT
			 }",
            None,
        ),
        (
            "switch (a) {
			    case 1: a = 4;

			// no default
			 }",
            None,
        ),
        (
            "switch (a) {
			    case 1: a = 4;

			/* no default */
			 }",
            None,
        ),
        (
            "switch (a) {
			    case 1: a = 4; break; break;

			// no default
			 }",
            None,
        ),
        (
            "switch (a) { // no default
			 }",
            None,
        ),
        ("switch (a) { }", None),
        (
            "switch (a) { case 1: break; default: break; }",
            Some(serde_json::json!([{
                "commentPattern": "default case omitted"
            }])),
        ),
        (
            "switch (a) { case 1: break;
			 // skip default case
			 }",
            Some(serde_json::json!([{
                "commentPattern": "^skip default"
            }])),
        ),
        (
            "switch (a) { case 1: break;
			 /*
			TODO:
			 throw error in default case
			*/
			 }",
            Some(serde_json::json!([{
                "commentPattern": "default"
            }])),
        ),
        (
            "switch (a) { case 1: break;
			//
			 }",
            Some(serde_json::json!([{
                "commentPattern": ".?"
            }])),
        ),
    ];

    let fail = vec![
        ("switch (a) { case 1: break; }", None),
        (
            "switch (a) {
			 // no default
			 case 1: break;  }",
            None,
        ),
        (
            "switch (a) { case 1: break;
			 // no default
			 // nope
			  }",
            None,
        ),
        (
            "switch (a) { case 1: break;
			 // no default
			 }",
            Some(serde_json::json!([{
                "commentPattern": "skipped default case"
            }])),
        ),
        (
            "switch (a) {
			case 1: break;
			// default omitted intentionally
			// TODO: add default case
			}",
            Some(serde_json::json!([{
                "commentPattern": "default omitted"
            }])),
        ),
        (
            "switch (a) {
			case 1: break;
			}",
            Some(serde_json::json!([{
                "commentPattern": ".?"
            }])),
        ),
    ];

    Tester::new(DefaultCase::NAME, pass, fail).test_and_snapshot();
}
