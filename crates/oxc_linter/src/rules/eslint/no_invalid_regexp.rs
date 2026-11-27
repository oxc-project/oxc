use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_allocator::Allocator;
use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, StringLiteral},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::{ConstructorParser, Options};
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

// Use the same prefix with `oxc_regular_expression` crate
fn duplicated_flag_diagnostic(span: Span, flag: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid regular expression: Duplicated flag")
        .with_help(format!("Remove the duplicated '{flag}' flag from the regular expression flags"))
        .with_label(span.label(format!("flag '{flag}' already specified")))
}

fn unknown_flag_diagnostic(span: Span, flag: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid regular expression: Unknown flag")
        .with_note("Valid flags are: d (indices), g (global), i (ignore case), m (multiline),\n             s (dot all), u (unicode), v (unicode sets), y (sticky)")
        .with_label(span.label(format!("flag '{flag}' is not a valid regular expression flag")))
}

fn invalid_unicode_flags_diagnostic(span: Span, is_u_specified: bool) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid regular expression: `u` and `v` flags should be used alone")
        .with_help("Specify only one of 'u' or 'v' flags")
        .with_label(span.label(if is_u_specified {
            "the 'v' flag cannot be used when the 'u' flag is specified"
        } else {
            "the 'u' flag cannot be used when the 'v' flag is specified"
        }))
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoInvalidRegexp(Box<NoInvalidRegexpConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow invalid regular expression strings in RegExp constructors.
    ///
    /// ### Why is this bad?
    ///
    /// An invalid pattern in a regular expression literal is a SyntaxError when the code is parsed,
    /// but an invalid string in RegExp constructors throws a SyntaxError only when the code is executed.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// RegExp('[')
    /// RegExp('.', 'z')
    /// new RegExp('\\')
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// RegExp('.')
    /// new RegExp
    /// this.RegExp('[')
    /// ```
    NoInvalidRegexp,
    eslint,
    correctness,
    recommended,
    config = NoInvalidRegexpConfig,
    version = "0.9.4",
    short_description = "Disallow invalid regular expression strings in RegExp constructors.",
);

#[derive(Debug, Clone, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NoInvalidRegexpConfig {
    /// Case-sensitive array of flags that will be allowed.
    allow_constructor_flags: Vec<char>,
}

impl Rule for NoInvalidRegexp {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (pattern_arg, flags_arg) = match node.kind() {
            AstKind::NewExpression(expr) if is_global_regexp(&expr.callee, ctx) => {
                parse_arguments_to_check(expr.arguments.first(), expr.arguments.get(1))
            }
            AstKind::CallExpression(expr) if is_global_regexp(&expr.callee, ctx) => {
                parse_arguments_to_check(expr.arguments.first(), expr.arguments.get(1))
            }
            // Other kinds, skip
            _ => return,
        };

        let (mut u_flag_found, mut v_flag_found) = (false, false);
        // Validate flags first if exists
        // `oxc_regular_expression` crate has a ability to validate flags.
        // But, it does not accept any `allow_constructor_flags` option.
        // And if we omit user defined flags here, `Span` may be incorrect on error reporting.
        if let Flags::Literal(flags) = flags_arg {
            let Some(flags_text) = flags.value.as_str() else {
                return ctx.diagnostic(unknown_flag_diagnostic(
                    flags.span,
                    &flags.value.display().to_string(),
                ));
            };
            let raw_flags_span = Span::new(flags.span.start + 1, flags.span.end - 1);
            let has_escapes = ctx.source_range(raw_flags_span) != flags_text;

            let mut unique_flags = FxHashSet::default();
            for (idx, ch) in flags_text.char_indices() {
                #[expect(clippy::cast_possible_truncation)]
                let span = if has_escapes {
                    flags.span
                } else {
                    Span::empty(raw_flags_span.start + idx as u32)
                };

                // Invalid combination: u+v
                if ch == 'u' {
                    if v_flag_found {
                        return ctx.diagnostic(invalid_unicode_flags_diagnostic(span, false));
                    }
                    u_flag_found = true;
                }
                if ch == 'v' {
                    if u_flag_found {
                        return ctx.diagnostic(invalid_unicode_flags_diagnostic(span, true));
                    }
                    v_flag_found = true;
                }

                // Duplicated: user defined, invalid or valid
                if !unique_flags.insert(ch) {
                    return ctx.diagnostic(duplicated_flag_diagnostic(span, &ch.to_string()));
                }

                // Unknown: not valid, not user defined
                if !(matches!(ch, 'd' | 'g' | 'i' | 'm' | 's' | 'u' | 'v' | 'y')
                    || self.0.allow_constructor_flags.contains(&ch))
                {
                    return ctx.diagnostic(unknown_flag_diagnostic(span, &ch.to_string()));
                }
            }
        }

        // Then, validate pattern if exists
        // Pattern check is skipped when 1st argument is NOT a `StringLiteral`
        // e.g. `new RegExp(var)`, `RegExp("str" + var)`
        let Some(pattern) = pattern_arg else { return };
        let allocator = Allocator::default();
        let pattern_text = ctx.source_range(pattern.span);
        let parse = |flags_text: Option<&str>| {
            ConstructorParser::new(
                &allocator,
                pattern_text,
                flags_text,
                Options { pattern_span_offset: pattern.span.start, flags_span_offset: 0 },
            )
            .parse()
            .err()
        };

        let diagnostic = if matches!(flags_arg, Flags::Unknown) {
            if parse(Some("'u'")).is_none() || parse(Some("'v'")).is_none() {
                return;
            }
            parse(None)
        } else {
            parse(match (u_flag_found, v_flag_found) {
                (true, false) => Some("'u'"),
                (_, true) => Some("'v'"),
                (false, false) => None,
            })
        };
        if let Some(diagnostic) = diagnostic {
            ctx.diagnostic(diagnostic);
        }
    }
}

#[derive(Clone, Copy)]
enum Flags<'b, 'a> {
    Absent,
    Literal(&'b StringLiteral<'a>),
    Unknown,
}

fn parse_arguments_to_check<'b, 'a>(
    arg1: Option<&'b Argument<'a>>,
    arg2: Option<&'b Argument<'a>>,
) -> (Option<&'b StringLiteral<'a>>, Flags<'b, 'a>) {
    let pattern = arg1.and_then(string_literal);
    let flags = match arg2 {
        None => Flags::Absent,
        Some(arg) => string_literal(arg).map_or(Flags::Unknown, Flags::Literal),
    };
    (pattern, flags)
}

fn string_literal<'b, 'a>(arg: &'b Argument<'a>) -> Option<&'b StringLiteral<'a>> {
    match arg.as_expression()?.without_parentheses() {
        Expression::StringLiteral(literal) => Some(literal),
        _ => None,
    }
}

/// `RegExp` that refers to the built-in, not to a local binding of that name.
fn is_global_regexp(callee: &Expression, ctx: &LintContext) -> bool {
    matches!(
        callee.get_inner_expression(),
        Expression::Identifier(ident)
            if ident.name == "RegExp" && ctx.is_reference_to_global_variable(ident)
    )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"RegExp(".", "\u0067");"#, None),
        (r#"RegExp(".", "\x67\u{69}");"#, None),
        ("new RegExp('.', ('g'));", None),
        ("RegExp('[A--B]', flags);", None),
        ("RegExp('.', ...flags);", None),
        ("RegExp('')", None),
        ("RegExp()", None),
        ("RegExp('.', 'g')", None),
        ("new RegExp('.')", None),
        ("new RegExp", None),
        ("new RegExp('.', 'im')", None),
        (r"global.RegExp('\\')", None),
        ("new RegExp('.', y)", None),
        ("new RegExp('.', 'y')", None),
        ("new RegExp('.', 'u')", None),
        ("new RegExp('.', 'yu')", None),
        ("new RegExp('/', 'yu')", None),
        (r"new RegExp('\/', 'yu')", None),
        (r"new RegExp('\\u{65}', 'u')", None),
        (r"new RegExp('\\u{65}*', 'u')", None),
        (r"new RegExp('[\\u{0}-\\u{1F}]', 'u')", None),
        ("new RegExp('.', 's')", None),
        ("new RegExp('(?<=a)b')", None),
        ("new RegExp('(?<!a)b')", None),
        (r"new RegExp('(?<a>b)\k<a>')", None),
        (r"new RegExp('(?<a>b)\k<a>', 'u')", None),
        (r"new RegExp('\\p{Letter}', 'u')", None),
        // unknown flags
        ("RegExp('{', flags)", None),
        ("new RegExp('{', flags)", None),
        (r"RegExp('\\u{0}*', flags)", None),
        (r"new RegExp('\\u{0}*', flags)", None),
        ("RegExp('{', flags)", Some(serde_json::json!([{ "allowConstructorFlags": ["u"] }]))),
        (
            r"RegExp('\\u{0}*', flags)",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }])),
        ),
        // unknown pattern
        ("new RegExp(pattern, 'g')", None),
        ("new RegExp('.' + '', 'g')", None),
        ("new RegExp(pattern, '')", None),
        ("new RegExp(pattern)", None),
        // ES2020
        (r"new RegExp('(?<\\ud835\\udc9c>.)', 'g')", None),
        (r"new RegExp('(?<\\u{1d49c}>.)', 'g')", None),
        ("new RegExp('(?<𝒜>.)', 'g');", None),
        (r"new RegExp('\\p{Script=Nandinagari}', 'u');", None),
        // ES2022
        ("new RegExp('a+(?<Z>z)?', 'd')", None),
        (r"new RegExp('\\p{Script=Cpmn}', 'u')", None),
        (r"new RegExp('\\p{Script=Cypro_Minoan}', 'u')", None),
        (r"new RegExp('\\p{Script=Old_Uyghur}', 'u')", None),
        (r"new RegExp('\\p{Script=Ougr}', 'u')", None),
        (r"new RegExp('\\p{Script=Tangsa}', 'u')", None),
        (r"new RegExp('\\p{Script=Tnsa}', 'u')", None),
        (r"new RegExp('\\p{Script=Toto}', 'u')", None),
        (r"new RegExp('\\p{Script=Vith}', 'u')", None),
        (r"new RegExp('\\p{Script=Vithkuqi}', 'u')", None),
        // ES2024
        ("new RegExp('[A--B]', 'v')", None),
        ("new RegExp('[A&&B]', 'v')", None),
        ("new RegExp('[A--[0-9]]', 'v')", None),
        (r"new RegExp('[\\p{Basic_Emoji}--\\q{a|bc|def}]', 'v')", None),
        ("new RegExp('[A--B]', flags)", None),
        (r"new RegExp('[[]\\u{0}*', flags)", None),
        // ES2025
        ("new RegExp('((?<k>a)|(?<k>b))')", None),
        ("new RegExp('(?ims:foo)')", None),
        ("new RegExp('(?ims-:foo)')", None),
        ("new RegExp('(?-ims:foo)')", None),
        ("new RegExp('(?s-i:foo)')", None),
        // allowConstructorFlags
        ("new RegExp('.', 'g')", Some(serde_json::json!([{ "allowConstructorFlags": [] }]))),
        ("new RegExp('.', 'g')", Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }]))),
        ("new RegExp('.', 'a')", Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }]))),
        ("new RegExp('.', 'ag')", Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }]))),
        ("new RegExp('.', 'ga')", Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }]))),
        (
            "new RegExp(pattern, 'ga')",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }])),
        ),
        (
            "new RegExp('.' + '', 'ga')",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }])),
        ),
        (
            "new RegExp('.', 'a')",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a", "z"] }])),
        ),
        (
            "new RegExp('.', 'z')",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a", "z"] }])),
        ),
        (
            "new RegExp('.', 'az')",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a", "z"] }])),
        ),
        (
            "new RegExp('.', 'za')",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a", "z"] }])),
        ),
        (
            "new RegExp('.', 'agz')",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a", "z"] }])),
        ),
        ("function foo(RegExp) { RegExp('['); }", None),
        ("function foo(RegExp) { new RegExp('['); }", None),
        ("let RegExp; RegExp('.', 'z');", None),
        ("function f(RegExp) { (RegExp)('['); new (RegExp)('['); }", None),
        ("function f(RegExp: any) { (RegExp as any)('['); new (RegExp!)('['); }", None),
    ];

    let fail = vec![
        (r#"RegExp(".", "\u0067\u0067");"#, None),
        (r#"RegExp(".", "g\u007a");"#, None),
        ("new RegExp('.', ('gg'));", None),
        ("new RegExp(('['));", None),
        ("function make(flags) { return RegExp('[', flags); }", None),
        ("RegExp(')', ...flags);", None),
        ("RegExp('[');", None),
        ("RegExp('.', 'z');", None),
        ("RegExp('.', 'a');", Some(serde_json::json!([{}]))),
        ("new RegExp('.', 'a');", Some(serde_json::json!([{ "allowConstructorFlags": [] }]))),
        ("new RegExp('.', 'z');", Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }]))),
        ("RegExp('.', 'a');", Some(serde_json::json!([{ "allowConstructorFlags": ["A"] }]))),
        ("RegExp('.', 'A');", Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }]))),
        ("new RegExp('.', 'az');", Some(serde_json::json!([{ "allowConstructorFlags": ["z"] }]))),
        ("new RegExp('.', 'aa');", Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }]))),
        // Can remove this test once we block duplicate values in this array.
        (
            "new RegExp('.', 'aa');",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a", "a"] }])),
        ),
        ("new RegExp('.', 'aA');", Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }]))),
        (
            "new RegExp('.', 'aaz');",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a", "z"] }])),
        ),
        (
            "new RegExp('.', 'azz');",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a", "z"] }])),
        ),
        ("new RegExp('.', 'aga');", Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }]))),
        ("new RegExp('.', 'uu');", Some(serde_json::json!([{ "allowConstructorFlags": ["u"] }]))),
        ("new RegExp('.', 'ouo');", Some(serde_json::json!([{ "allowConstructorFlags": ["u"] }]))),
        ("new RegExp(')');", None),
        (r"new RegExp('\\a', 'u');", None),
        (r"new RegExp('\\a', 'u');", Some(serde_json::json!([{ "allowConstructorFlags": ["u"] }]))),
        (r"RegExp('\\u{0}*');", None),
        (r"new RegExp('\\u{0}*');", None),
        (r"new RegExp('\\u{0}*', '');", None),
        (
            r"new RegExp('\\u{0}*', 'a');",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }])),
        ),
        (r"RegExp('\\u{0}*');", Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }]))),
        (r"new RegExp('\\');", None),
        ("RegExp(')' + '', 'a');", None),
        (
            "new RegExp('.' + '', 'az');",
            Some(serde_json::json!([{ "allowConstructorFlags": ["z"] }])),
        ),
        (
            "new RegExp(pattern, 'az');",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }])),
        ),
        // ES2024
        ("new RegExp('[[]', 'v');", None),
        ("new RegExp('.', 'uv');", None),
        ("new RegExp(pattern, 'uv');", None),
        ("new RegExp('[A--B]' /* valid only with `v` flag */, 'u')", None),
        (r"new RegExp('[[]\\u{0}*' /* valid only with `u` flag */, 'v')", None),
        // ES2025
        ("new RegExp('(?<k>a)(?<k>b)')", None),
        ("new RegExp('(?ii:foo)')", None),
        ("new RegExp('(?-ii:foo)')", None),
        ("new RegExp('(?i-i:foo)')", None),
        ("new RegExp('(?-:foo)')", None),
        ("new RegExp('(?g:foo)')", None),
        ("new RegExp('(?-u:foo)')", None),
        ("(RegExp)('[');", None),
        ("new (RegExp)('[');", None),
        ("(RegExp as any)('[');", None),
        ("new (RegExp!)('[');", None),
    ];

    Tester::new(NoInvalidRegexp::NAME, NoInvalidRegexp::PLUGIN, pass, fail).test_and_snapshot();
}
