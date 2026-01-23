use std::hash::Hash;

use cow_utils::CowUtils;
use lazy_regex::Regex;
use oxc_ast::{
    AstKind,
    ast::{Argument, BinaryExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{CompactStr, GetSpan, Span};
use rustc_hash::FxHashMap;

use crate::{
    context::LintContext,
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, parse_general_jest_fn_call},
};

fn title_must_be_string_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Title must be a string")
        .with_help("Replace your title with a string")
        .with_label(span)
}

fn empty_title_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Should not have an empty title")
        .with_help("Write a meaningful title for your test")
        .with_label(span)
}

fn duplicate_prefix_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Should not have duplicate prefix")
        .with_help("The function name already has the prefix, try to remove the duplicate prefix")
        .with_label(span)
}

fn accidental_space_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Should not have leading or trailing spaces")
        .with_help("Remove the leading or trailing spaces")
        .with_label(span)
}

fn disallowed_word_diagnostic(word: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{word} is not allowed in test title"))
        .with_help("It is included in the `disallowedWords` of your config file, try to remove it from your title")
        .with_label(span)
}

pub fn must_match_diagnostic(message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(message.to_string())
        .with_help("Make sure the title matches the `mustMatch` of your config file")
        .with_label(span)
}

pub fn must_not_match_diagnostic(message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(message.to_string())
        .with_help("Make sure the title does not match the `mustNotMatch` of your config file")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ValidTitleConfig {
    /// Whether to ignore the type of the name passed to `test`.
    ignore_type_of_test_name: bool,
    /// Whether to ignore the type of the name passed to `describe`.
    ignore_type_of_describe_name: bool,
    /// Whether to allow arguments as titles.
    allow_arguments: bool,
    /// A list of disallowed words, which will not be allowed in titles.
    disallowed_words: Vec<CompactStr>,
    /// Whether to ignore leading and trailing spaces in titles.
    ignore_space: bool,
    /// Patterns for titles that must not match.
    must_not_match_patterns: FxHashMap<MatchKind, CompiledMatcherAndMessage>,
    /// Patterns for titles that must be matched for the title to be valid.
    must_match_patterns: FxHashMap<MatchKind, CompiledMatcherAndMessage>,
}

impl ValidTitleConfig {
    #[expect(clippy::unnecessary_wraps)] // TODO: handle error
    pub fn from_configuration(
        value: &serde_json::Value,
    ) -> Result<ValidTitleConfig, serde_json::error::Error> {
        let config = value.get(0);
        let get_as_bool = |name: &str| -> bool {
            config
                .and_then(|v| v.get(name))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_default()
        };

        let ignore_type_of_test_name = get_as_bool("ignoreTypeOfTestName");
        let ignore_type_of_describe_name = get_as_bool("ignoreTypeOfDescribeName");
        let allow_arguments = get_as_bool("allowArguments");
        let ignore_space = get_as_bool("ignoreSpaces");
        let disallowed_words = config
            .and_then(|v| v.get("disallowedWords"))
            .and_then(|v| v.as_array())
            .map(|v| v.iter().filter_map(|v| v.as_str().map(CompactStr::from)).collect())
            .unwrap_or_default();
        let must_not_match_patterns = config
            .and_then(|v| v.get("mustNotMatch"))
            .and_then(compile_matcher_patterns)
            .unwrap_or_default();
        let must_match_patterns = config
            .and_then(|v| v.get("mustMatch"))
            .and_then(compile_matcher_patterns)
            .unwrap_or_default();

        Ok(ValidTitleConfig {
            ignore_type_of_test_name,
            ignore_type_of_describe_name,
            allow_arguments,
            disallowed_words,
            ignore_space,
            must_not_match_patterns,
            must_match_patterns,
        })
    }

    pub fn run_rule<'a>(
        config: &ValidTitleConfig,
        possible_jest_fn_node: &PossibleJestNode<'a, '_>,
        ctx: &LintContext<'a>,
    ) {
        let node = possible_jest_fn_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, possible_jest_fn_node, ctx)
        else {
            return;
        };

        if !matches!(
            jest_fn_call.kind,
            JestFnKind::General(JestGeneralFnKind::Describe | JestGeneralFnKind::Test)
        ) {
            return;
        }

        // Check if extend keyword has been used (vitest feature)
        if let Some(member) = jest_fn_call.members.first()
            && member.is_name_equal("extend")
        {
            return;
        }

        let Some(arg) = call_expr.arguments.first() else {
            return;
        };

        // Handle typecheck settings - skip for describe when enabled (vitest feature)
        if ctx.settings().vitest.typecheck
            && matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Describe))
        {
            return;
        }

        // Handle allowArguments option (vitest feature)
        if config.allow_arguments && matches!(arg, Argument::Identifier(_)) {
            return;
        }

        let need_report_name = match jest_fn_call.kind {
            JestFnKind::General(JestGeneralFnKind::Test) => !config.ignore_type_of_test_name,
            JestFnKind::General(JestGeneralFnKind::Describe) => {
                !config.ignore_type_of_describe_name
            }
            _ => unreachable!(),
        };

        match arg {
            Argument::StringLiteral(string_literal) => {
                validate_title(
                    &string_literal.value,
                    string_literal.span,
                    config,
                    &jest_fn_call.name,
                    ctx,
                );
            }
            Argument::TemplateLiteral(template_literal) => {
                if let Some(quasi) = template_literal.single_quasi() {
                    validate_title(
                        quasi.as_str(),
                        template_literal.span,
                        config,
                        &jest_fn_call.name,
                        ctx,
                    );
                }
            }
            Argument::BinaryExpression(binary_expr) => {
                if does_binary_expression_contain_string_node(binary_expr) {
                    return;
                }
                if need_report_name {
                    ctx.diagnostic(title_must_be_string_diagnostic(arg.span()));
                }
            }
            _ => {
                if need_report_name {
                    ctx.diagnostic(title_must_be_string_diagnostic(arg.span()));
                }
            }
        }
    }
}

type CompiledMatcherAndMessage = (Regex, Option<CompactStr>);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum MatchKind {
    Describe,
    It,
    Test,
}

#[derive(Copy, Clone)]
enum MatcherPattern<'a> {
    String(&'a serde_json::Value),
    Vec(&'a Vec<serde_json::Value>),
}

impl MatchKind {
    fn from(name: &str) -> Option<Self> {
        match name {
            "describe" => Some(Self::Describe),
            "it" => Some(Self::It),
            "test" => Some(Self::Test),
            _ => None,
        }
    }
}

fn compile_matcher_patterns(
    matcher_patterns: &serde_json::Value,
) -> Option<FxHashMap<MatchKind, CompiledMatcherAndMessage>> {
    matcher_patterns
        .as_array()
        .map_or_else(
            || {
                // for `{ "describe": "/pattern/" }`
                let obj = matcher_patterns.as_object()?;
                let mut map: FxHashMap<MatchKind, CompiledMatcherAndMessage> = FxHashMap::default();
                for (key, value) in obj {
                    let Some(v) = compile_matcher_pattern(MatcherPattern::String(value)) else {
                        continue;
                    };
                    if let Some(kind) = MatchKind::from(key) {
                        map.insert(kind, v);
                    }
                }

                Some(map)
            },
            |value| {
                // for `["/pattern/", "message"]`
                let mut map: FxHashMap<MatchKind, CompiledMatcherAndMessage> = FxHashMap::default();
                let v = &compile_matcher_pattern(MatcherPattern::Vec(value))?;
                map.insert(MatchKind::Describe, v.clone());
                map.insert(MatchKind::Test, v.clone());
                map.insert(MatchKind::It, v.clone());
                Some(map)
            },
        )
        .map_or_else(
            || {
                // for `"/pattern/"`
                let string = matcher_patterns.as_str()?;
                let mut map: FxHashMap<MatchKind, CompiledMatcherAndMessage> = FxHashMap::default();
                let v = &compile_matcher_pattern(MatcherPattern::String(
                    &serde_json::Value::String(string.to_string()),
                ))?;
                map.insert(MatchKind::Describe, v.clone());
                map.insert(MatchKind::Test, v.clone());
                map.insert(MatchKind::It, v.clone());
                Some(map)
            },
            Some,
        )
}

fn compile_matcher_pattern(pattern: MatcherPattern) -> Option<CompiledMatcherAndMessage> {
    match pattern {
        MatcherPattern::String(pattern) => {
            let pattern_str = pattern.as_str()?;

            // Check for JS regex literal: /pattern/flags
            if let Some(stripped) = pattern_str.strip_prefix('/')
                && let Some(end) = stripped.rfind('/')
            {
                let (pat, _flags) = stripped.split_at(end);
                // For now, ignore flags and just use the pattern
                let regex = Regex::new(pat).ok()?;
                return Some((regex, None));
            }

            // Fallback: treat as a normal Rust regex with Unicode support
            let reg_str = format!("(?u){pattern_str}");
            let reg = Regex::new(&reg_str).ok()?;
            Some((reg, None))
        }
        MatcherPattern::Vec(pattern) => {
            let pattern_str = pattern.first().and_then(|v| v.as_str())?;

            // Check for JS regex literal: /pattern/flags
            let regex = if let Some(stripped) = pattern_str.strip_prefix('/')
                && let Some(end) = stripped.rfind('/')
            {
                let (pat, _flags) = stripped.split_at(end);
                Regex::new(pat).ok()?
            } else {
                let reg_str = format!("(?u){pattern_str}");
                Regex::new(&reg_str).ok()?
            };

            let message = pattern.get(1).and_then(serde_json::Value::as_str).map(CompactStr::from);
            Some((regex, message))
        }
    }
}

fn validate_title(
    title: &str,
    span: Span,
    config: &ValidTitleConfig,
    name: &str,
    ctx: &LintContext,
) {
    if title.is_empty() {
        ctx.diagnostic(empty_title_diagnostic(span));
        return;
    }

    if !config.disallowed_words.is_empty() {
        let Ok(disallowed_words_reg) = Regex::new(&format!(
            r"(?iu)\b(?:{})\b",
            config.disallowed_words.join("|").cow_replace('.', r"\.")
        )) else {
            return;
        };

        if let Some(matched) = disallowed_words_reg.find(title) {
            ctx.diagnostic(disallowed_word_diagnostic(matched.as_str(), span));
        }
        return;
    }

    let trimmed_title = title.trim();
    if !config.ignore_space && trimmed_title != title {
        ctx.diagnostic_with_fix(accidental_space_diagnostic(span), |fixer| {
            let inner_span = span.shrink(1);
            let raw_text = fixer.source_range(inner_span);
            let trimmed_raw = raw_text.trim().to_string();
            fixer.replace(inner_span, trimmed_raw)
        });
    }

    let un_prefixed_name = name.trim_start_matches(['f', 'x']);
    let Some(first_word) = title.split(' ').next() else {
        return;
    };

    if first_word == un_prefixed_name {
        ctx.diagnostic_with_fix(duplicate_prefix_diagnostic(span), |fixer| {
            // Use raw source text to preserve escape sequences
            let inner_span = span.shrink(1);
            let raw_text = fixer.source_range(inner_span);
            // Find the first space in raw text to avoid byte offset issues
            // if the prefix word ever contains escapable characters
            let space_pos = raw_text.find(' ').unwrap_or(raw_text.len());
            let replaced_raw = raw_text[space_pos..].trim().to_string();
            fixer.replace(inner_span, replaced_raw)
        });
        return;
    }

    let Some(jest_fn_name) = MatchKind::from(un_prefixed_name) else {
        return;
    };

    if let Some((regex, message)) = config.must_match_patterns.get(&jest_fn_name)
        && !regex.is_match(title)
    {
        let raw_pattern = regex.as_str();
        let message = match message.as_ref() {
            Some(message) => message.as_str(),
            None => &format!("{un_prefixed_name} should match {raw_pattern}"),
        };
        ctx.diagnostic(must_match_diagnostic(message, span));
    }

    if let Some((regex, message)) = config.must_not_match_patterns.get(&jest_fn_name)
        && regex.is_match(title)
    {
        let raw_pattern = regex.as_str();
        let message = match message.as_ref() {
            Some(message) => message.as_str(),
            None => &format!("{un_prefixed_name} should not match {raw_pattern}"),
        };

        ctx.diagnostic(must_not_match_diagnostic(message, span));
    }
}

fn does_binary_expression_contain_string_node(expr: &BinaryExpression) -> bool {
    if expr.left.is_string_literal() || expr.right.is_string_literal() {
        return true;
    }

    match &expr.left {
        Expression::BinaryExpression(left) => does_binary_expression_contain_string_node(left),
        _ => false,
    }
}
