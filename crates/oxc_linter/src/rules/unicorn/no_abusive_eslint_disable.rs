use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    disable_directives::{DirectivePrefix, RuleCommentType},
    rule::Rule,
};

fn no_abusive_eslint_disable_diagnostic(
    span: Span,
    directive_prefix: DirectivePrefix,
) -> OxcDiagnostic {
    let message = match directive_prefix {
        DirectivePrefix::Eslint => {
            "Unexpected `eslint-disable` comment that does not specify any rules to disable."
        }
        DirectivePrefix::Oxlint => {
            "Unexpected `oxlint-disable` comment that does not specify any rules to disable."
        }
    };
    OxcDiagnostic::warn(message)
        .with_help("Specify the rules you want to disable.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAbusiveEslintDisable;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows `oxlint-disable` or `eslint-disable` comments without specifying rules.
    ///
    /// ### Why is this bad?
    ///
    /// A general `oxlint-disable` or `eslint-disable` comment suppresses all lint errors, not just the intended one,
    /// potentially hiding useful warnings and making debugging harder.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
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
    /// ```javascript
    /// /* oxlint-disable */
    /// console.log(message);
    ///
    /// console.log(message); // oxlint-disable-line
    ///
    /// // oxlint-disable-next-line
    /// console.log(message);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```javascript
    /// /* eslint-disable no-console */
    /// console.log(message);
    ///
    /// console.log(message); // eslint-disable-line no-console
    ///
    /// // eslint-disable-next-line no-console
    /// console.log(message);
    /// ```
    ///
    /// ```javascript
    /// /* oxlint-disable no-console */
    /// console.log(message);
    ///
    /// console.log(message); // oxlint-disable-line no-console
    ///
    /// // oxlint-disable-next-line no-console
    /// console.log(message);
    /// ```
    NoAbusiveEslintDisable,
    unicorn,
    restriction,
    recommended,
    version = "0.0.18",
    short_description = "Disallows `oxlint-disable` or `eslint-disable` comments without specifying rules.",
);

impl Rule for NoAbusiveEslintDisable {
    fn run_once(&self, ctx: &LintContext) {
        for comment in ctx.disable_directives().disable_rule_comments() {
            match &comment.r#type {
                RuleCommentType::All => {
                    ctx.diagnostic(no_abusive_eslint_disable_diagnostic(
                        comment.span,
                        comment.directive_prefix,
                    ));
                }
                RuleCommentType::Single(rules) => {
                    for rule in rules {
                        if !is_valid_rule_name(&rule.rule_name) {
                            ctx.diagnostic(no_abusive_eslint_disable_diagnostic(
                                comment.span,
                                comment.directive_prefix,
                            ));
                        }
                    }
                }
            }
        }
    }
}

/// Rule names may have any number of `/`-separated segments
/// (e.g. `n/no-unsupported-features/node-builtins`). A scoped name
/// (`@scope/...`) must name at least one segment after the scope.
fn is_valid_rule_name(rule_name: &str) -> bool {
    match rule_name.strip_prefix('@') {
        Some(scoped) => scoped.contains('/'),
        None => true,
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
        "eval(); //	oxlint-disable-line no-eval",
        "eval(); /* oxlint-disable-line no-eval */",
        "eval(); // eslint-disable-line plugin/rule",
        "eval(); // eslint-disable-line @scope/plugin/rule-name",
        "eval(); // oxlint-disable-line plugin/rule",
        "eval(); // oxlint-disable-line @scope/plugin/rule-name",
        "eval(); // eslint-disable-line no-eval, @scope/plugin/rule-name",
        "eval(); // eslint-disable-line @scope/rule-name",
        "eval(); // eslint-disable-line no-eval, @scope/rule-name",
        "eval(); // eslint-disable-line n/no-unsupported-features/node-builtins",
        "eval(); // oxlint-disable-line node/no-unsupported-features/node-builtins",
        "eval(); // eslint-disable-line @scope/plugin/category/rule-name",
        "/* eslint-disable n/no-unsupported-features/es-builtins */",
        "eval(); // eslint-line-disable",
        "eval(); // some comment",
        "/* eslint-disable no-eval */",
        "/* oxlint-disable no-eval */",
        r"
        /* eslint-disable no-abusive-eslint-disable */
        eval(); // eslint-disable-line
        ",
        r"
        /* oxlint-disable no-abusive-eslint-disable */
        eval(); // oxlint-disable-line
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
        r"
        foo();
        /* oxlint-disable-next-line no-eval */
        eval();
        ",
    ];

    let fail = vec![
        r"
        // eslint-disable-next-line @scopewithoutplugin
        eval();
        ",
        "// oxlint-disable-next-line @scopewithoutplugin\neval();",
        "eval(); // eslint-disable-line",
        "eval(); // oxlint-disable-line",
        r"
        foo();
        eval(); // eslint-disable-line
        ",
        "/* eslint-disable */",
        "/* oxlint-disable */",
        r"
        foo();
        /* eslint-disable */
        eval();
        ",
        r"
        foo();
        /* oxlint-disable */
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
        r"
        foo();
        /* oxlint-disable-next-line */
        eval();
        ",
        r"
        // oxlint-disable-next-line
        eval();
        ",
    ];

    Tester::new(NoAbusiveEslintDisable::NAME, NoAbusiveEslintDisable::PLUGIN, pass, fail)
        .test_and_snapshot();
}
