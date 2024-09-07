use oxc_allocator::Allocator;
use oxc_ast::{ast::Argument, AstKind};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::{FlagsParser, ParserOptions, PatternParser};
use oxc_span::Span;
use rustc_hash::FxHashSet;
use serde::Deserialize;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoInvalidRegexp(Box<NoInvalidRegexpConfig>);

declare_oxc_lint!(
    /// ### What it does
    /// Disallow invalid regular expression strings in RegExp constructors.
    ///
    /// ### Why is this bad?
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
    correctness,
);

#[derive(Debug, Clone, Deserialize, Default)]
struct NoInvalidRegexpConfig {
    #[serde(default, rename = "allowConstructorFlags")]
    /// Case-sensitive array of flags.
    allow_constructor_flags: Vec<char>,
}

impl Rule for NoInvalidRegexp {
    fn from_configuration(value: serde_json::Value) -> Self {
        value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .map_or_else(Self::default, |value| Self(Box::new(value)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (pattern_arg, flags_arg) = match node.kind() {
            AstKind::NewExpression(expr) if expr.callee.is_specific_id("RegExp") => {
                parse_arguments_to_check(expr.arguments.first(), expr.arguments.get(1))
            }
            AstKind::CallExpression(expr) if expr.callee.is_specific_id("RegExp") => {
                parse_arguments_to_check(expr.arguments.first(), expr.arguments.get(1))
            }
            // Other kinds, skip
            _ => return,
        };

        // No arguments, skip
        if pattern_arg.is_none() && flags_arg.is_none() {
            return;
        }

        let allocator = Allocator::default();

        // Validate flags first if exists
        let mut parsed_flags = None;
        if let Some((flags_span_start, flags_text)) = flags_arg {
            // Check for duplicated flags
            // For compatibility with ESLint, we need to check "user-defined duplicated" flags here
            // "valid duplicated" flags are also checked
            let mut unique_flags = FxHashSet::default();
            let mut violations = vec![];
            for (idx, ch) in flags_text.char_indices() {
                if !unique_flags.insert(ch) {
                    violations.push(idx);
                }
            }
            if !violations.is_empty() {
                return ctx.diagnostic(
                    // Use the same prefix with `oxc_regular_expression`
                    OxcDiagnostic::warn("Invalid regular expression: Duplicated flag").with_labels(
                        violations
                            .iter()
                            .map(|&start| {
                                #[allow(clippy::cast_possible_truncation)]
                                let start = flags_span_start + start as u32;
                                LabeledSpan::new_with_span(None, Span::new(start, start))
                            })
                            .collect::<Vec<_>>(),
                    ),
                );
            }

            // Omit user defined invalid flags
            for flag in &self.0.allow_constructor_flags {
                match flag {
                    // Keep valid flags, even if they are defined
                    'd' | 'g' | 'i' | 'm' | 's' | 'u' | 'v' | 'y' => continue,
                    _ => {
                        unique_flags.remove(flag);
                    }
                }
            }

            // Use parser to check:
            // - Unknown invalid flags
            // - Invalid flags combination: u+v
            // - (Valid duplicated flags are already checked above)
            // It can be done without `FlagsParser`, though
            let flags_text = unique_flags.iter().collect::<String>();
            let options = ParserOptions::default().with_span_offset(flags_span_start);
            match FlagsParser::new(&allocator, flags_text.as_str(), options).parse() {
                Ok(flags) => parsed_flags = Some(flags),
                Err(diagnostic) => return ctx.diagnostic(diagnostic),
            }
        }

        // Then, validate pattern if exists
        // Pattern check is skipped when 1st argument is NOT a `StringLiteral`
        // e.g. `new RegExp(var)`, `RegExp("str" + var)`
        if let Some((pattern_span_start, pattern_text)) = pattern_arg {
            let mut options = ParserOptions::default().with_span_offset(pattern_span_start);
            if let Some(flags) = parsed_flags {
                if flags.unicode || flags.unicode_sets {
                    options = options.with_unicode_mode();
                }
                if flags.unicode_sets {
                    options = options.with_unicode_sets_mode();
                }
            }
            match PatternParser::new(&allocator, pattern_text, options).parse() {
                Ok(_) => {}
                Err(diagnostic) => ctx.diagnostic(diagnostic),
            }
        }
    }
}

/// Returns: (span_start, text)
/// span_start + 1 for opening string bracket.
type ParsedArgument<'a> = (u32, &'a str);
fn parse_arguments_to_check<'a>(
    arg1: Option<&Argument<'a>>,
    arg2: Option<&Argument<'a>>,
) -> (Option<ParsedArgument<'a>>, Option<ParsedArgument<'a>>) {
    match (arg1, arg2) {
        // ("pattern", "flags")
        (Some(Argument::StringLiteral(pattern)), Some(Argument::StringLiteral(flags))) => (
            Some((pattern.span.start + 1, pattern.value.as_str())),
            Some((flags.span.start + 1, flags.value.as_str())),
        ),
        // (pattern, "flags")
        (Some(_arg), Some(Argument::StringLiteral(flags))) => {
            (None, Some((flags.span.start + 1, flags.value.as_str())))
        }
        // ("pattern")
        (Some(Argument::StringLiteral(pattern)), None) => {
            (Some((pattern.span.start + 1, pattern.value.as_str())), None)
        }
        // (pattern), ()
        _ => (None, None),
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("[RegExp(''), /a/uv]", None),
        ("RegExp()", None),
        ("RegExp('.', 'g')", None),
        ("new RegExp('.')", None),
        ("new RegExp", None),
        ("new RegExp('.', 'im')", None),
        ("global.RegExp('\\\\')", None),
        ("new RegExp('.', y)", None),
        ("new RegExp('.', 'y')", None),
        ("new RegExp('.', 'u')", None),
        ("new RegExp('.', 'yu')", None),
        ("new RegExp('/', 'yu')", None),
        ("new RegExp('\\/', 'yu')", None),
        ("new RegExp('\\\\u{65}', 'u')", None),
        ("new RegExp('\\\\u{65}*', 'u')", None),
        ("new RegExp('[\\\\u{0}-\\\\u{1F}]', 'u')", None),
        ("new RegExp('.', 's')", None),
        ("new RegExp('(?<=a)b')", None),
        ("new RegExp('(?<!a)b')", None),
        ("new RegExp('(?<a>b)\\k<a>')", None),
        ("new RegExp('(?<a>b)\\k<a>', 'u')", None),
        ("new RegExp('\\\\p{Letter}', 'u')", None),
        // unknown flags
        ("RegExp('{', flags)", None),
        ("new RegExp('{', flags)", None),
        ("RegExp('\\\\u{0}*', flags)", None),
        ("new RegExp('\\\\u{0}*', flags)", None),
        ("RegExp('{', flags)", Some(serde_json::json!([{ "allowConstructorFlags": ["u"] }]))),
        (
            "RegExp('\\\\u{0}*', flags)",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }])),
        ),
        // unknown pattern
        ("new RegExp(pattern, 'g')", None),
        ("new RegExp('.' + '', 'g')", None),
        ("new RegExp(pattern, '')", None),
        ("new RegExp(pattern)", None),
        // ES2020
        ("new RegExp('(?<\\\\ud835\\\\udc9c>.)', 'g')", None),
        ("new RegExp('(?<\\\\u{1d49c}>.)', 'g')", None),
        ("new RegExp('(?<𝒜>.)', 'g');", None),
        ("new RegExp('\\\\p{Script=Nandinagari}', 'u');", None),
        // ES2022
        ("new RegExp('a+(?<Z>z)?', 'd')", None),
        ("new RegExp('\\\\p{Script=Cpmn}', 'u')", None),
        ("new RegExp('\\\\p{Script=Cypro_Minoan}', 'u')", None),
        ("new RegExp('\\\\p{Script=Old_Uyghur}', 'u')", None),
        ("new RegExp('\\\\p{Script=Ougr}', 'u')", None),
        ("new RegExp('\\\\p{Script=Tangsa}', 'u')", None),
        ("new RegExp('\\\\p{Script=Tnsa}', 'u')", None),
        ("new RegExp('\\\\p{Script=Toto}', 'u')", None),
        ("new RegExp('\\\\p{Script=Vith}', 'u')", None),
        ("new RegExp('\\\\p{Script=Vithkuqi}', 'u')", None),
        // ES2024
        ("new RegExp('[A--B]', 'v')", None),
        ("new RegExp('[A&&B]', 'v')", None),
        ("new RegExp('[A--[0-9]]', 'v')", None),
        ("new RegExp('[\\\\p{Basic_Emoji}--\\\\q{a|bc|def}]', 'v')", None),
        ("new RegExp('[A--B]', flags)", None),
        ("new RegExp('[[]\\\\u{0}*', flags)", None),
        // ES2025
        // ("new RegExp('((?<k>a)|(?<k>b))')", None),
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
    ];

    let fail = vec![
        ("RegExp('[');", None),
        ("RegExp('.', 'z');", None),
        ("RegExp('.', 'a');", Some(serde_json::json!([{}]))),
        ("new RegExp('.', 'a');", Some(serde_json::json!([{ "allowConstructorFlags": [] }]))),
        ("new RegExp('.', 'z');", Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }]))),
        ("RegExp('.', 'a');", Some(serde_json::json!([{ "allowConstructorFlags": ["A"] }]))),
        ("RegExp('.', 'A');", Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }]))),
        ("new RegExp('.', 'az');", Some(serde_json::json!([{ "allowConstructorFlags": ["z"] }]))),
        ("new RegExp('.', 'aa');", Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }]))),
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
        ("new RegExp('\\\\a', 'u');", None),
        (
            "new RegExp('\\\\a', 'u');",
            Some(serde_json::json!([{ "allowConstructorFlags": ["u"] }])),
        ),
        ("RegExp('\\\\u{0}*');", None),
        ("new RegExp('\\\\u{0}*');", None),
        ("new RegExp('\\\\u{0}*', '');", None),
        (
            "new RegExp('\\\\u{0}*', 'a');",
            Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }])),
        ),
        ("RegExp('\\\\u{0}*');", Some(serde_json::json!([{ "allowConstructorFlags": ["a"] }]))),
        ("new RegExp('\\\\');", None),
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
        ("new RegExp('[[]\\\\u{0}*' /* valid only with `u` flag */, 'v')", None),
        // ES2025
        ("new RegExp('(?<k>a)(?<k>b)')", None),
    ];

    Tester::new(NoInvalidRegexp::NAME, pass, fail).test_and_snapshot();
}
