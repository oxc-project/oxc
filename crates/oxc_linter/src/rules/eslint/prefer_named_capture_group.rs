use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::visit::{RegExpAstKind, Visit};
use oxc_span::Span;
use oxc_str::CompactStr;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{run_on_arguments, run_on_regex_node},
};

fn prefer_named_capture_group_diagnostic(
    span: Span,
    unnamed_count: usize,
    allowed: u32,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Capture group should be named.")
        .with_help(format!(
            "Use a named capture group like \"(?<name>...)\" \u{2014} this regex has {unnamed_count} unnamed group{} (allowed: {allowed}).",
            if unnamed_count == 1 { "" } else { "s" }
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct PreferNamedCaptureGroupConfig {
    /// Maximum number of unnamed capturing groups allowed per regex pattern.
    ///
    /// Default is `0`, which disallows any unnamed groups, matching ESLint's stock behavior.
    /// Set to `1` to permit one unnamed capture per regex, and so on.
    allow_unnamed_groups: u32,
    /// Additional function names to treat as RegExp constructors.
    ///
    /// oxlint-only extension. When a call or `new` expression uses one of these names as
    /// a bare identifier callee, its string or template-literal argument is parsed and
    /// checked for unnamed capturing groups, just like `RegExp(...)`.
    ///
    /// Example: `{ "additionalRegExpFunctions": ["regEx"] }` makes `regEx('(foo)')` reportable.
    additional_reg_exp_functions: FxHashSet<CompactStr>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PreferNamedCaptureGroup(Box<PreferNamedCaptureGroupConfig>);

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
    version = "next",
    config = PreferNamedCaptureGroup,
);

impl Rule for PreferNamedCaptureGroup {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let allow_unnamed = self.0.allow_unnamed_groups;
        run_on_regex_node(node, ctx, |pattern, _span| {
            check_pattern(pattern, allow_unnamed, ctx);
        });

        if self.0.additional_reg_exp_functions.is_empty() {
            return;
        }

        let (callee, arguments) = match node.kind() {
            AstKind::CallExpression(expr) => (&expr.callee, &expr.arguments),
            AstKind::NewExpression(expr) => (&expr.callee, &expr.arguments),
            _ => return,
        };

        let Expression::Identifier(ident) = callee.get_inner_expression() else { return };
        if !self.0.additional_reg_exp_functions.contains(ident.name.as_str()) {
            return;
        }

        let arg0 = arguments.first();
        // Skip regex literals — they're already handled as their own AST node by run_on_regex_node.
        if arg0.is_some_and(|a| matches!(a, oxc_ast::ast::Argument::RegExpLiteral(_))) {
            return;
        }

        run_on_arguments(arg0, arguments.get(1), ctx, |pattern, _span| {
            check_pattern(pattern, allow_unnamed, ctx);
        });
    }
}

fn check_pattern(
    pattern: &oxc_regular_expression::ast::Pattern<'_>,
    allow_unnamed: u32,
    ctx: &LintContext<'_>,
) {
    let mut collector = UnnamedGroupCollector::default();
    collector.visit_pattern(pattern);

    let count = collector.unnamed_spans.len();
    if count > allow_unnamed as usize {
        for span in collector.unnamed_spans {
            ctx.diagnostic(prefer_named_capture_group_diagnostic(span, count, allow_unnamed));
        }
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
        // No capturing groups
        ("/normal_regex/", None),
        // Non-capturing group — always fine
        ("/(?:[0-9]{4})/", None),
        // Named capturing group — the good pattern
        ("/(?<year>[0-9]{4})/", None),
        // No groups at all
        (r"/\u{1F680}/u", None),
        // RegExp constructor with no arguments / non-static argument → can't check
        ("new RegExp()", None),
        ("new RegExp(foo)", None),
        ("RegExp()", None),
        ("RegExp(foo)", None),
        // Empty pattern — no groups
        ("new RegExp('')", None),
        ("RegExp('')", None),
        // Named group via constructor
        ("new RegExp('(?<year>[0-9]{4})')", None),
        ("RegExp('(?<year>[0-9]{4})')", None),
        // Invalid regex (parse error) → silently ignored
        ("RegExp('(')", None),
        // Unicode escape, no groups
        (r"RegExp('\\u{1F680}', 'u')", None),
        // globalThis.RegExp with non-static argument → can't check
        ("new globalThis.RegExp()", None),    // ecmaVersion 2020
        ("new globalThis.RegExp(foo)", None), // ecmaVersion 2020
        ("globalThis.RegExp(foo)", None),     // ecmaVersion 2020
        // globalThis shadowed → not recognized as global RegExp constructor
        (
            "
            var globalThis = bar;
            globalThis.RegExp(foo);
            ",
            None,
        ),
        (
            "
            function foo () {
                var globalThis = bar;
                new globalThis.RegExp(baz);
            }
            ",
            None,
        ),
        // v-flag edge case: invalid escape inside character class in v mode → parse error → ignored
        (r"new RegExp('([\\q])', 'v')", None),
        // Named group in v-flag pattern
        ("new RegExp('(?<c>[[A--B]])', 'v')", None),
        // Inline flag groups (non-capturing, ecmaVersion 2025)
        ("/(?i:foo)bar/", None),
        ("new RegExp('(?i:foo)bar')", None),
        ("/(?-i:foo)bar/", None),
        ("new RegExp('(?-i:foo)bar')", None),
        // Concatenated strings — not statically resolvable, ignored
        ("new RegExp('(' + 'a)')", None),
        ("new RegExp('a(bc)d' + 'e')", None),
        (r#"new RegExp("foo" + "(a)" + "(b)");"#, None),
        (r#"new RegExp("foo" + "(?:a)" + "(b)");"#, None),
        ("RegExp('(a)'+'')", None),
        ("RegExp( '' + '(ab)')", None),
        // Template literal with substitution — not statically resolvable, ignored
        ("new RegExp(`(ab)${''}`)", None),
        // allowUnnamedGroups: 1 — one unnamed group is permitted
        ("/(foo)/", Some(serde_json::json!([{ "allowUnnamedGroups": 1 }]))),
        // allowUnnamedGroups: 1 — one unnamed alongside a named group is fine
        ("/(?<a>x)(b)/", Some(serde_json::json!([{ "allowUnnamedGroups": 1 }]))),
        // additionalRegExpFunctions — named group only, no diagnostic
        (
            "regEx('(?<x>foo)')",
            Some(serde_json::json!([{ "additionalRegExpFunctions": ["regEx"] }])),
        ),
        // string arg but factory name not in the list → not inspected
        ("regEx('(foo)')", None),
        // string arg but factory name doesn't match the configured name
        ("regEx('(foo)')", Some(serde_json::json!([{ "additionalRegExpFunctions": ["other"] }]))),
        // allowUnnamedGroups threshold met
        (
            "regEx('(foo)', 'g')",
            Some(
                serde_json::json!([{ "additionalRegExpFunctions": ["regEx"], "allowUnnamedGroups": 1 }]),
            ),
        ),
        // new form with named group
        (
            "new MyRe('(?<a>x)')",
            Some(serde_json::json!([{ "additionalRegExpFunctions": ["MyRe"] }])),
        ),
        // regex literal arg with named group — passes via literal arm, no false positive
        (
            "regEx(/(?<a>foo)/)",
            Some(serde_json::json!([{ "additionalRegExpFunctions": ["regEx"] }])),
        ),
    ];

    let fail = vec![
        // Single unnamed group — simplest case
        ("/([0-9]{4})/", None),
        // Constructor forms
        ("new RegExp('([0-9]{4})')", None),
        ("RegExp('([0-9]{4})')", None),
        // Template literal (no substitution)
        ("new RegExp(`a(bc)d`)", None),
        // Unicode prefix in string, unnamed group
        ("new RegExp('ሴ噸(?:a)(b)');", None),
        // Multiple unnamed groups
        (r"/([0-9]{4})-(\w{5})/", None),
        ("/([0-9]{4})-(5)/", None),
        // Named outer group containing unnamed inner group
        ("/(?<temp2>(a))/", None),
        ("/(?<temp2>(a)(?<temp5>b))/", None),
        // Mixed named and unnamed
        (r"/(?<temp1>[0-9]{4})-(\w{5})/", None),
        ("/(?<temp1>[0-9]{4})-(5)/", None),
        ("/(?<temp1>a)(?<temp2>a)(a)(?<temp3>a)/", None),
        // Multi-line template literal (no substitution)
        ("new RegExp(`(a)\n            `)", None),
        ("RegExp(`a(b\n            c)d`)", None),
        // Escape sequences in string patterns
        (r"new RegExp('a(b)\'')", None),
        (r"RegExp('(a)\\d')", None),
        (r"RegExp(`\a(b)`)", None),
        // globalThis.RegExp — always recognized in oxlint regardless of ecmaVersion
        ("new globalThis.RegExp('([0-9]{4})')", None),
        ("globalThis.RegExp('([0-9]{4})')", None),
        // globalThis NOT shadowed in the outer scope
        (
            "
            function foo() { var globalThis = bar; }
            new globalThis.RegExp('([0-9]{4})');
            ",
            None,
        ),
        // v-flag with unnamed group in set notation
        ("new RegExp('([[A--B]])', 'v')", None),
        // allowUnnamedGroups: 1 — two unnamed groups still fails (both are reported)
        ("/(a)(b)/", Some(serde_json::json!([{ "allowUnnamedGroups": 1 }]))),
        // allowUnnamedGroups: 1 — two unnamed + one named, still two unnamed → fail
        ("/(a)(?<b>c)(d)/", Some(serde_json::json!([{ "allowUnnamedGroups": 1 }]))),
        // additionalRegExpFunctions — string arg with unnamed group
        (
            "regEx('([0-9]+)')",
            Some(serde_json::json!([{ "additionalRegExpFunctions": ["regEx"] }])),
        ),
        // additionalRegExpFunctions — template literal (no substitution) with two unnamed groups
        (
            "regEx(`(foo)(bar)`)",
            Some(serde_json::json!([{ "additionalRegExpFunctions": ["regEx"] }])),
        ),
        // additionalRegExpFunctions — new form with unnamed group
        (
            "new MyRe('([0-9]+)')",
            Some(serde_json::json!([{ "additionalRegExpFunctions": ["MyRe"] }])),
        ),
        // regex literal arg inside a custom factory: reported once (via literal arm), not twice
        ("regEx(/(foo)/)", Some(serde_json::json!([{ "additionalRegExpFunctions": ["regEx"] }]))),
    ];

    Tester::new(PreferNamedCaptureGroup::NAME, PreferNamedCaptureGroup::PLUGIN, pass, fail)
        .test_and_snapshot();
}
