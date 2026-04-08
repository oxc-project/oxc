use super::css_utils::is_css_file;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    css_parser::{CssRule, CssStylesheet, parse_css},
    rule::Rule,
};

fn css_no_empty_blocks_diagnostic(selector: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Empty declaration block for `{selector}`."))
        .with_help("Remove the empty rule or add declarations to it.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct CssNoEmptyBlocks;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects CSS rules with empty declaration blocks.
    ///
    /// ### Why is this bad?
    ///
    /// Empty rule blocks are dead code that increases file size and makes
    /// stylesheets harder to maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```css
    /// .a { }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```css
    /// .a { color: red; }
    /// ```
    CssNoEmptyBlocks,
    oxc,
    correctness
);

impl Rule for CssNoEmptyBlocks {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_css(source_text);
        let Some(stylesheet) = &result.stylesheet else {
            return;
        };

        check_stylesheet(stylesheet, ctx);
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host() && is_css_file(ctx.file_path())
    }
}

fn check_stylesheet(stylesheet: &CssStylesheet<'_>, ctx: &LintContext<'_>) {
    for rule in &stylesheet.rules {
        check_rule(rule, ctx);
    }
}

fn check_rule(rule: &CssRule<'_>, ctx: &LintContext<'_>) {
    match rule {
        CssRule::QualifiedRule(qr) => {
            if qr.block.declarations.is_empty() {
                ctx.diagnostic(css_no_empty_blocks_diagnostic(qr.selector, qr.block.span));
            }
        }
        CssRule::AtRule(ar) => {
            if let Some(block) = &ar.block {
                for nested in &block.rules {
                    check_rule(nested, ctx);
                }
            }
        }
        CssRule::Comment(_) => {}
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![".a { color: red; }", "@media (max-width: 768px) { .a { color: red; } }"];

    let fail = vec![".a { }", "h1 { } h2 { color: red; }"];

    Tester::new(CssNoEmptyBlocks::NAME, CssNoEmptyBlocks::PLUGIN, pass, fail)
        .change_rule_path("test.css")
        .test_and_snapshot();
}
