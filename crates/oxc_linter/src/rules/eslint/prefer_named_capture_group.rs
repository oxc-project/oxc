use std::borrow::Cow;

use oxc_allocator::Allocator;
use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
    builder::AstBuilder,
};
use oxc_codegen::Codegen;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::{
    ConstructorParser, LiteralParser, Options,
    ast::Pattern,
    visit::{RegExpAstKind, Visit},
};
use oxc_span::{GetSpan, Span};
use oxc_str::JSStr;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{is_regexp_callee, run_on_regex_node, static_string_value},
};

fn prefer_named_capture_group_diagnostic(span: Span, unnamed_count: usize) -> OxcDiagnostic {
    OxcDiagnostic::warn("Capture group should be named.")
        .with_help(format!(
            "Use a named capture group like \"(?<name>...)\" — this regex has {unnamed_count} unnamed group{}.",
            if unnamed_count == 1 { "" } else { "s" }
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferNamedCaptureGroup;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of named capture groups in regular expressions.
    ///
    /// ### Why is this bad?
    ///
    /// Unnamed capturing groups (`(...)`) are referenced only by position, which
    /// makes the regex harder to read and maintain. When the pattern changes, index-based
    /// references silently break. Named groups (`(?<name>...)`) make the intent explicit
    /// and allow references by name (e.g. `match.groups.year`), which is more robust.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const re = /([0-9]{4})-([0-9]{2})/;
    /// const match = re.exec(str);
    /// const year = match[1]; // fragile index
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const re = /(?<year>[0-9]{4})-(?<month>[0-9]{2})/;
    /// const match = re.exec(str);
    /// const year = match.groups.year; // explicit name
    ///
    /// // Non-capturing groups are always fine
    /// const parts = /(?:[0-9]{4})/;
    /// ```
    PreferNamedCaptureGroup,
    eslint,
    style,
    pending,
    version = "1.68.0",
    short_description = "Enforces the use of named capture groups in regular expressions.",
);

impl Rule for PreferNamedCaptureGroup {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::RegExpLiteral(_) | AstKind::CallExpression(_) | AstKind::NewExpression(_) => {}
            _ => return,
        }

        run_on_regex_node(node, ctx, |pattern, _span| {
            check_pattern(pattern, ctx, None);
        });

        let (callee, arguments) = match node.kind() {
            AstKind::CallExpression(expr) => (&expr.callee, &expr.arguments),
            AstKind::NewExpression(expr) => (&expr.callee, &expr.arguments),
            _ => return,
        };
        if is_regexp_callee(callee, ctx) {
            check_static_arguments(arguments.first(), arguments.get(1), ctx);
        }
    }
}

fn check_pattern(pattern: &Pattern<'_>, ctx: &LintContext<'_>, span_override: Option<Span>) {
    let mut collector = UnnamedGroupCollector::default();
    collector.visit_pattern(pattern);

    let count = collector.unnamed_spans.len();
    for span in collector.unnamed_spans {
        ctx.diagnostic(prefer_named_capture_group_diagnostic(span_override.unwrap_or(span), count));
    }
}

fn check_static_arguments(arg0: Option<&Argument>, arg1: Option<&Argument>, ctx: &LintContext<'_>) {
    let Some(pattern_expr) = arg0
        .and_then(Argument::as_expression)
        .map(Expression::get_inner_expression)
        .filter(|expr| !is_directly_supported_regex_argument(expr))
    else {
        return;
    };

    let Some(pattern_value) = static_string_value(pattern_expr, ctx.allocator()) else {
        return;
    };
    let flags_value = arg1
        .and_then(Argument::as_expression)
        .map(Expression::get_inner_expression)
        .and_then(|expr| static_string_value(expr, ctx.allocator()));
    let flags_text = match flags_value {
        Some(value) => {
            let Some(text) = value.as_str() else { return };
            Some(text)
        }
        None => None,
    };

    let allocator = Allocator::default();
    let options = Options { pattern_span_offset: pattern_expr.span().start, flags_span_offset: 0 };
    let pattern_text = pattern_value.as_str().map_or_else(
        || Cow::Owned(string_literal_source(pattern_value, &allocator)),
        Cow::Borrowed,
    );
    let flags_source = pattern_value
        .has_lone_surrogate()
        .then(|| flags_text.map(|text| string_literal_source(text.into(), &allocator)))
        .flatten();
    let parsed = if pattern_value.has_lone_surrogate() {
        // Parse the generated string literal to retain the original UTF-16 values.
        ConstructorParser::new(&allocator, &pattern_text, flags_source.as_deref(), options).parse()
    } else {
        LiteralParser::new(&allocator, &pattern_text, flags_text, options).parse()
    };
    let Ok(pattern) = parsed else { return };

    check_pattern(&pattern, ctx, Some(pattern_expr.span()));
}

fn string_literal_source(value: JSStr<'_>, allocator: &Allocator) -> String {
    let expression =
        Expression::new_string_literal(Span::default(), value, None, &AstBuilder::new(allocator));
    let mut codegen = Codegen::new();
    codegen.print_expression(&expression);
    codegen.into_source_text()
}

fn is_directly_supported_regex_argument(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::RegExpLiteral(_) | Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.is_no_substitution_template(),
        _ => false,
    }
}

#[derive(Default)]
struct UnnamedGroupCollector {
    unnamed_spans: Vec<Span>,
}

impl<'a> Visit<'a> for UnnamedGroupCollector {
    fn enter_node(&mut self, kind: RegExpAstKind<'a>) {
        if let RegExpAstKind::CapturingGroup(group) = kind
            && group.name.is_none()
        {
            self.unnamed_spans.push(group.span);
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"new RegExp("(a)[" + "\uD801" + "-" + "\uD800" + "]");"#,
        r#"new RegExp("(a)[" + "\uD801" + "-" + "\uD800" + "]", "u");"#,
        r#"new RegExp("(a)" + "", "u\uD800");"#,
        "/normal_regex/",
        "/(?:[0-9]{4})/",
        "/(?<year>[0-9]{4})/",
        r"/\u{1F680}/u",
        "new RegExp()",
        "new RegExp(foo)",
        "new RegExp('')",
        "new RegExp('(?<year>[0-9]{4})')",
        "RegExp()",
        "RegExp(foo)",
        "RegExp('')",
        "RegExp('(?<year>[0-9]{4})')",
        "RegExp('(')",
        r"RegExp('\\u{1F680}', 'u')",
        // Oxc recognizes globalThis.RegExp regardless of ecmaVersion, so upstream's pre-2020
        // globalThis capture-group cases are intentionally covered as invalid below.
        "new globalThis.RegExp()",    // { "ecmaVersion": 2020 },
        "new globalThis.RegExp(foo)", // { "ecmaVersion": 2020 },
        "globalThis.RegExp(foo)",     // { "ecmaVersion": 2020 },
        // { "ecmaVersion": 2020 }
        "var globalThis = bar; globalThis.RegExp(foo);",
        // { "ecmaVersion": 2020 }
        "function foo () { var globalThis = bar; new globalThis.RegExp(baz); }",
        "new RegExp('(?<c>[[A--B]])', 'v')",
        r"new RegExp('([\\q])', 'v')",
        "/(?i:foo)bar/", // { "ecmaVersion": 2025 },
        "new RegExp('(?i:foo)bar')",
        "/(?-i:foo)bar/", // { "ecmaVersion": 2025 },
        "new RegExp('(?-i:foo)bar')",
    ];

    let fail = vec![
        r#"new RegExp("(a)[" + "\uD800" + "-" + "\uE000" + "]");"#,
        r#"new RegExp("(a)[" + "\uD800" + "-" + "\uE000" + "]", "u");"#,
        "/([0-9]{4})/",
        "new RegExp('([0-9]{4})')",
        "RegExp('([0-9]{4})')",
        "new RegExp(`a(bc)d`)",
        "new RegExp('ሴ噸(?:a)(b)');",
        r"new RegExp('\u1234\u5678(?:a)(b)');",
        r"/([0-9]{4})-(\w{5})/",
        "/([0-9]{4})-(5)/",
        "/(?<temp2>(a))/",
        "/(?<temp2>(a)(?<temp5>b))/",
        r"/(?<temp1>[0-9]{4})-(\w{5})/",
        "/(?<temp1>[0-9]{4})-(5)/",
        "/(?<temp1>a)(?<temp2>a)(a)(?<temp3>a)/",
        "new RegExp('(' + 'a)')",
        "new RegExp('a(bc)d' + 'e')",
        r#"new RegExp("foo" + "(a)" + "(b)");"#,
        r#"new RegExp("foo" + "(?:a)" + "(b)");"#,
        r#"new RegExp("(a)" + "\uD800");"#,
        "RegExp('(a)'+'')",
        "RegExp( '' + '(ab)')",
        "new RegExp(`(ab)${''}`)",
        "new RegExp(`(a)
            `)",
        "RegExp(`a(b
            c)d`)",
        r"new RegExp('a(b)\'')",
        r"RegExp('(a)\\d')",
        r"RegExp(`\a(b)`)",
        "new globalThis.RegExp('([0-9]{4})')", // { "ecmaVersion": 2020 },
        "globalThis.RegExp('([0-9]{4})')",     // { "ecmaVersion": 2020 },
        // { "ecmaVersion": 2020 }
        "function foo() { var globalThis = bar; } new globalThis.RegExp('([0-9]{4})');",
        "new RegExp('([[A--B]])', 'v')",
    ];

    Tester::new(PreferNamedCaptureGroup::NAME, PreferNamedCaptureGroup::PLUGIN, pass, fail)
        .test_and_snapshot();
}
