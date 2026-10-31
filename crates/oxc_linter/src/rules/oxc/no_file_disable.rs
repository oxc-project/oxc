use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, disable_directives::DirectivePrefix, rule::Rule};

fn no_file_disable_diagnostic(span: Span, prefix: DirectivePrefix) -> OxcDiagnostic {
    let message = match prefix {
        DirectivePrefix::Eslint => "Do not use file-scoped `eslint-disable` comments.",
        DirectivePrefix::Oxlint => "Do not use file-scoped `oxlint-disable` comments.",
    };
    OxcDiagnostic::warn(message)
        .with_help(
            "Use a disable-next-line or disable-line directive, or configure a file override.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoFileDisable;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows file-scoped `eslint-disable` and `oxlint-disable` comments,
    /// with or without a list of rules.
    ///
    /// These directives disable rules until a matching enable directive or the
    /// end of the file. This rule also reports directives paired with an enable
    /// directive. The `disable-line` and `disable-next-line` forms are allowed.
    ///
    /// ### Why is this bad?
    ///
    /// File-scoped disable comments can unintentionally hide warnings in code
    /// added later. Line-scoped directives make the exceptions explicit and keep
    /// them close to the affected code. Use configuration overrides when a rule
    /// should be disabled for an entire file.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```js
    /// /* eslint-disable no-console */
    /// console.log(message);
    ///
    /// /* oxlint-disable */
    /// debugger;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```js
    /// // eslint-disable-next-line no-console
    /// console.log(message);
    ///
    /// debugger; // oxlint-disable-line no-debugger
    /// ```
    NoFileDisable,
    oxc,
    restriction,
    version = "next",
    short_description = "Disallow file-scoped disable comments.",
);

impl Rule for NoFileDisable {
    fn run_once(&self, ctx: &LintContext) {
        for comment in ctx.disable_directives().disable_rule_comments() {
            let directive = ctx.source_range(comment.span).split_whitespace().next();
            if matches!(directive, Some("eslint-disable" | "oxlint-disable")) {
                ctx.diagnostic(no_file_disable_diagnostic(comment.span, comment.directive_prefix));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "debugger;",
        "// An eslint-disable comment would hide later warnings.",
        r#"const text = "/* oxlint-disable */";"#,
        "/* eslint-disablefoo no-debugger */",
        "/* oxlint-disable-lext-nine no-debugger */",
        "/* eslint-enable no-debugger */",
        "// oxlint-enable",
        "debugger; // eslint-disable-line",
        "debugger; // oxlint-disable-line no-debugger",
        "debugger; /* eslint-disable-line no-debugger */",
        "debugger; /* oxlint-disable-line */",
        "// eslint-disable-next-line no-debugger\ndebugger;",
        "// oxlint-disable-next-line\ndebugger;",
        "/* eslint-disable-next-line */\ndebugger;",
        "/* oxlint-disable-next-line no-debugger */\ndebugger;",
        "// eslint-disable-next-line oxc/no-file-disable\n/* eslint-disable no-debugger */",
        "// oxlint-disable-next-line oxc/no-file-disable\n/* oxlint-disable no-debugger */",
        "/* eslint-disable no-debugger */ // eslint-disable-line oxc/no-file-disable",
    ];

    let fail = vec![
        "/* eslint-disable */",
        "/* oxlint-disable */",
        "/*eslint-disable*/",
        "/*oxlint-disable*/",
        "/* eslint-disable no-debugger */\ndebugger;",
        "/* oxlint-disable no-debugger, no-console */\ndebugger;",
        "// eslint-disable no-debugger\ndebugger;",
        "// oxlint-disable\ndebugger;",
        "debugger; /* eslint-disable no-console */",
        "/*\n\toxlint-disable no-debugger\n*/\ndebugger;",
        "/*\u{a0}eslint-disable no-debugger */",
        "/* oxlint-disable\u{a0}no-debugger */",
        "/* eslint-disable\r\nno-debugger */",
        "/* eslint-disable no-debugger -- needed for debugging */",
        "/* oxlint-disable -- generated file */",
        "/* eslint-disable no-debugger */\ndebugger;\n/* eslint-enable no-debugger */",
        "/* oxlint-disable no-debugger */\ndebugger;\n/* oxlint-enable no-debugger */",
        "/* eslint-disable oxc/no-file-disable */",
        "/* oxlint-disable oxc/no-file-disable */",
        "/* eslint-disable no-debugger */\n/* oxlint-disable no-console */",
    ];

    Tester::new(NoFileDisable::NAME, NoFileDisable::PLUGIN, pass, fail).test_and_snapshot();
}
