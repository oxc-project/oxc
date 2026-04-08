use rustc_hash::FxHashMap;

use super::css_utils::is_css_file;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    css_parser::{CssDeclarationBlock, CssRule, CssStylesheet, parse_css},
    rule::Rule,
};

fn css_no_duplicate_properties_diagnostic(
    property: &str,
    span: Span,
    first_span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Duplicate property `{property}` in declaration block."))
        .with_help("Remove one of the duplicate declarations or use a different value strategy.")
        .with_labels([span.label("duplicate here"), first_span.label("first occurrence")])
}

#[derive(Debug, Default, Clone)]
pub struct CssNoDuplicateProperties;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects duplicate CSS properties within the same declaration block.
    ///
    /// ### Why is this bad?
    ///
    /// Duplicate properties are usually a mistake. The later declaration
    /// overrides the earlier one, which can hide intended styles. While
    /// sometimes used as a fallback strategy, explicit fallbacks should use
    /// different syntax (e.g., custom properties, `@supports`).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```css
    /// .a { color: red; color: blue; }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```css
    /// .a { color: red; background: blue; }
    /// ```
    CssNoDuplicateProperties,
    oxc,
    correctness
);

impl Rule for CssNoDuplicateProperties {
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
    let mut seen: FxHashMap<&str, Span> = FxHashMap::default();
    for decl in &block.declarations {
        if let Some(&first_span) = seen.get(decl.property) {
            ctx.diagnostic(css_no_duplicate_properties_diagnostic(
                decl.property,
                decl.property_span,
                first_span,
            ));
        } else {
            seen.insert(decl.property, decl.property_span);
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ".a { color: red; background: blue; }",
        ".a { color: red; } .b { color: blue; }",
        "@media (max-width: 768px) { .a { color: red; } }",
    ];

    let fail = vec![
        ".a { color: red; color: blue; }",
        "h1 { font-size: 16px; margin: 0; font-size: 20px; }",
    ];

    Tester::new(CssNoDuplicateProperties::NAME, CssNoDuplicateProperties::PLUGIN, pass, fail)
        .change_rule_path("test.css")
        .test_and_snapshot();
}
