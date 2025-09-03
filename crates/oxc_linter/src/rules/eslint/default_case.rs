use lazy_regex::{Regex, RegexBuilder};
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

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
    /// Enforces that all `switch` statements include a `default` case,
    /// unless explicitly marked with a configured comment.
    ///
    /// ### Why is this bad?
    ///
    /// Without a `default` case, it is unclear whether the omission was
    /// intentional or an oversight. Adding a `default` or a special comment
    /// makes the code more explicit and reduces mistakes.
    ///
    /// You may optionally include a `// no default` after the last case if there is
    /// no default case. The comment may be in any desired case, such as `// No Default`.
    ///
    /// ### Options
    ///
    /// First option:
    /// - Type: `object`
    /// - Properties:
    ///     - `commentPattern`: `string` (default: `/^no default$/i`) - A regex pattern used to detect comments that mark the absence of a `default` case as intentional.
    ///
    /// Example configuration:
    ///   ```json
    ///   {
    ///       "default-case": ["error", { "commentPattern": "^skip\\sdefault" }]
    ///   }
    ///   ```
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /* default-case: ["error"] */
    ///
    /// switch (foo) {
    ///   case 1:
    ///     break;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /* default-case: ["error"] */
    ///
    /// switch (a) {
    ///   case 1:
    ///     break;
    ///   default:
    ///     break;
    /// }
    ///
    /// switch (a) {
    ///   case 1:
    ///     break;
    ///   // no default
    /// }
    /// ```
    ///
    /// #### `commentPattern`
    ///
    /// Examples of **incorrect** code for this rule with the `{ "commentPattern": "^skip\\sdefault" }` option:
    /// ```js
    /// /* default-case: ["error", { "commentPattern": "^skip\\sdefault" }] */
    ///
    /// switch (a) {
    ///   case 1:
    ///     break;
    ///   // no default
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "commentPattern": "^skip\\sdefault" }` option:
    /// ```js
    /// /* default-case: ["error", { "commentPattern": "^skip\\sdefault" }] */
    ///
    /// switch (a) {
    ///   case 1:
    ///     break;
    ///   // skip default
    /// }
    /// ```
    DefaultCase,
    eslint,
    restriction,
);

impl Rule for DefaultCase {
    fn from_configuration(value: serde_json::Value) -> Self {
        let comment_pattern = value
            .get(0)
            .and_then(|config| config.get("commentPattern"))
            .and_then(serde_json::Value::as_str)
            .and_then(|pattern| RegexBuilder::new(pattern).case_insensitive(true).build().ok());
        let case_config = DefaultCaseConfig { comment_pattern };

        Self(Box::new(case_config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SwitchStatement(switch) = node.kind() else {
            return;
        };

        let cases = &switch.cases;

        if cases.is_empty() || cases.iter().any(|case| case.test.is_none()) {
            return;
        }

        let Some(last_case) = cases.last() else {
            return;
        };

        if !has_default_comment(ctx, switch.span, last_case.span, self.comment_pattern.as_ref()) {
            ctx.diagnostic(default_case_diagnostic(switch.span));
        }
    }
}

fn has_default_comment(
    ctx: &LintContext,
    switch_span: Span,
    last_case_span: Span,
    comment_pattern: Option<&Regex>,
) -> bool {
    ctx.semantic().comments_range(last_case_span.start..switch_span.end).next_back().is_some_and(
        |comment| {
            let raw = ctx.source_range(comment.content_span()).trim();

            match comment_pattern {
                Some(re) => re.is_match(raw),
                None => raw.eq_ignore_ascii_case("no default"),
            }
        },
    )
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

    Tester::new(DefaultCase::NAME, DefaultCase::PLUGIN, pass, fail).test_and_snapshot();
}
