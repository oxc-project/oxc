use lazy_static::lazy_static;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::Regex;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(no-abusive-eslint-disable): Unexpected `eslint-disable` comment that does not specify any rules to disable.")]
#[diagnostic(severity(warning), help("Specify the rules you want to disable."))]
struct NoAbusiveEslintDisableDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoAbusiveEslintDisable;

declare_oxc_lint!(
    /// ### What it does
    /// This rule disallows `eslint-disable` comments that do not specify any rules to disable.
    ///
    /// ### Why is this bad?
    /// When only one rule should be disabled but the `eslint-disable` comment does not specify any rules, other useful errors will also be silently ignored.
    ///
    /// ### Example
    /// ```javascript
    /// // Fail
    /// /* eslint-disable */
    /// console.log(message);
    ///
    /// console.log(message); // eslint-disable-line
    ///
    /// // eslint-disable-next-line
    /// console.log(message);
    ///
    /// // Pass
    /// /* eslint-disable no-console */
    /// console.log(message);
    ///
    /// console.log(message); // eslint-disable-line no-console
    ///
    /// // eslint-disable-next-line no-console
    /// console.log(message);
    /// ```
    NoAbusiveEslintDisable,
    restriction
);

impl Rule for NoAbusiveEslintDisable {
    fn run_once(&self, ctx: &LintContext) {
        lazy_static! {
            static ref ESLINT_DISABLE_PATTERN: Regex = Regex::new(
                r"(?x)
                ^eslint-disable(?:-next-line|-line)?
                (?<rule_id>
                    $
                    |
                    (?:\s+(?:@(?:[0-9A-Za-z_-]+\/){1,2})?[0-9A-Za-z_-]+)?
                )
                ",
            )
            .unwrap();
        }

        let comments = ctx.semantic().trivias().comments();
        for (start, comment) in comments {
            let raw = &ctx.semantic().source_text()[*start as usize..comment.end() as usize];
            let Some(caps) = ESLINT_DISABLE_PATTERN.captures(raw.trim()) else { continue };
            let rule_id = caps.name("rule_id").unwrap().as_str();
            if rule_id.is_empty() {
                ctx.diagnostic(NoAbusiveEslintDisableDiagnostic(Span {
                    start: *start,
                    end: comment.end(),
                }));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "eval();",
        "eval(); // eslint-disable-line no-eval",
        "eval(); // eslint-disable-line no-eval, no-console",
        "eval(); //eslint-disable-line no-eval",
        "eval(); //     eslint-disable-line no-eval",
        "eval(); //	eslint-disable-line no-eval",
        "eval(); /* eslint-disable-line no-eval */",
        "eval(); // eslint-disable-line plugin/rule",
        "eval(); // eslint-disable-line @scope/plugin/rule-name",
        "eval(); // eslint-disable-line no-eval, @scope/plugin/rule-name",
        "eval(); // eslint-disable-line @scope/rule-name",
        "eval(); // eslint-disable-line no-eval, @scope/rule-name",
        "eval(); // eslint-line-disable",
        "eval(); // some comment",
        "/* eslint-disable no-eval */",
        r#"
        /* eslint-disable no-abusive-eslint-disable */
        eval(); // eslint-disable-line
        "#,
        r#"
        foo();
        // eslint-disable-line no-eval
        eval();
        "#,
        r#"
        foo();
        /* eslint-disable no-eval */
        eval();
        "#,
        r#"
        foo();
        /* eslint-disable-next-line no-eval */
        eval();
        "#,
    ];

    let fail = vec![
        r#"
        // eslint-disable-next-line @scopewithoutplugin
        eval();
        "#,
        "eval(); // eslint-disable-line",
        r#"
        foo();
        eval(); // eslint-disable-line
        "#,
        "/* eslint-disable */",
        r#"
        foo();
        /* eslint-disable */
        eval();
        "#,
        r#"
        foo();
        /* eslint-disable-next-line */
        eval();
        "#,
        r#"
        // eslint-disable-next-line
        eval();
        "#,
    ];

    Tester::new_without_config(NoAbusiveEslintDisable::NAME, pass, fail).test_and_snapshot();
}
