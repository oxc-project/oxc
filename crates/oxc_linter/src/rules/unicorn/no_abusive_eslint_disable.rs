use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, disable_directives::DisableRuleComment, rule::Rule};

fn no_abusive_eslint_disable_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Unexpected `eslint-disable` comment that does not specify any rules to disable.",
    )
    .with_help("Specify the rules you want to disable.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAbusiveEslintDisable;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows `eslint-disable` comments that do not specify any rules to disable.
    ///
    /// ### Why is this bad?
    ///
    /// When only one rule should be disabled but the `eslint-disable` comment does not specify any rules, other useful errors will also be silently ignored.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /* eslint-disable */
    /// console.log(message);
    ///
    /// console.log(message); // eslint-disable-line
    ///
    /// // eslint-disable-next-line
    /// console.log(message);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /* eslint-disable no-console */
    /// console.log(message);
    ///
    /// console.log(message); // eslint-disable-line no-console
    ///
    /// // eslint-disable-next-line no-console
    /// console.log(message);
    /// ```
    NoAbusiveEslintDisable,
    unicorn,
    restriction
);

impl Rule for NoAbusiveEslintDisable {
    fn run_once(&self, ctx: &LintContext) {
        for span in ctx.disable_directives().disable_all_comments() {
            ctx.diagnostic(no_abusive_eslint_disable_diagnostic(*span));
        }

        for DisableRuleComment { span, rules } in ctx.disable_directives().disable_rule_comments() {
            if rules.is_empty() || !is_valid_rule_name(rules[0]) {
                ctx.diagnostic(no_abusive_eslint_disable_diagnostic(*span));
            }
        }
    }
}

fn is_valid_rule_name(rule_name: &str) -> bool {
    let segment_count = rule_name.split('/').count();
    if rule_name.starts_with('@') {
        segment_count == 2 || segment_count == 3
    } else {
        segment_count <= 2
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
        r"
        /* eslint-disable no-abusive-eslint-disable */
        eval(); // eslint-disable-line
        ",
        r"
        foo();
        // eslint-disable-line no-eval
        eval();
        ",
        r"
        foo();
        /* eslint-disable no-eval */
        eval();
        ",
        r"
        foo();
        /* eslint-disable-next-line no-eval */
        eval();
        ",
    ];

    let fail = vec![
        r"
        // eslint-disable-next-line @scopewithoutplugin
        eval();
        ",
        "eval(); // eslint-disable-line",
        r"
        foo();
        eval(); // eslint-disable-line
        ",
        "/* eslint-disable */",
        r"
        foo();
        /* eslint-disable */
        eval();
        ",
        r"
        foo();
        /* eslint-disable-next-line */
        eval();
        ",
        r"
        // eslint-disable-next-line
        eval();
        ",
    ];

    Tester::new(NoAbusiveEslintDisable::NAME, NoAbusiveEslintDisable::PLUGIN, pass, fail)
        .test_and_snapshot();
}
