use super::css_utils::is_css_file;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    css_parser::{CssDeclarationBlock, CssRule, CssStylesheet, parse_css},
    rule::Rule,
};

fn css_no_important_diagnostic(property: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`!important` used on `{property}`."))
        .with_help(
            "Avoid `!important` — it makes styles harder to override and maintain. \
             Use more specific selectors instead.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct CssNoImportant;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows use of `!important` in CSS declarations.
    ///
    /// ### Why is this bad?
    ///
    /// `!important` overrides the natural specificity cascade, making styles
    /// difficult to maintain and debug. It often leads to specificity wars
    /// where more `!important` declarations are added to override earlier ones.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```css
    /// .a { color: red !important; }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```css
    /// .a { color: red; }
    /// ```
    CssNoImportant,
    oxc,
    style
);

impl Rule for CssNoImportant {
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
        CssRule::QualifiedRule(qr) => check_block(&qr.block, ctx),
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

fn check_block(block: &CssDeclarationBlock<'_>, ctx: &LintContext<'_>) {
    for decl in &block.declarations {
        if decl.important {
            ctx.diagnostic(css_no_important_diagnostic(decl.property, decl.value_span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ".a { color: red; }",
        ".a { color: red; font-size: 16px; }",
        "@media (max-width: 768px) { .a { color: red; } }",
    ];

    let fail =
        vec![".a { color: red !important; }", ".a { color: red; font-size: 16px !important; }"];

    Tester::new(CssNoImportant::NAME, CssNoImportant::PLUGIN, pass, fail)
        .change_rule_path("test.css")
        .test_and_snapshot();
}
