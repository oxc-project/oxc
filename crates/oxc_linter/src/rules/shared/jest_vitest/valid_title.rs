use itertools::Itertools;
use lazy_regex::{Regex, regex};
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{Argument, BinaryExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;

use crate::{
    context::LintContext,
    rule::DefaultRuleConfig,
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, is_string_raw_member_expression,
        parse_general_jest_fn_call,
    },
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

pub const DOCUMENTATION: &str = r"
### What it does

Checks that the titles of Jest and Vitest blocks are valid.

Titles must be:
- not empty,
- strings,
- not prefixed with their block name,
- have no leading or trailing spaces.

### Why is this bad?

Titles that are not valid can be misleading and make it harder to understand the purpose of the test.

### Examples

Examples of **incorrect** code for this rule:
```javascript
describe('', () => {});
describe('foo', () => {
  it('', () => {});
});
it('', () => {});
test('', () => {});
xdescribe('', () => {});
xit('', () => {});
xtest('', () => {});
```
Examples of **correct** code for this rule:
```javascript
describe('foo', () => {});
it('bar', () => {});
test('baz', () => {});
```
";

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ValidTitleOptions {
    /// Whether to ignore the type of the first argument to `test` and `it`
    /// blocks, so titles that are not strings are allowed.
    ignore_type_of_test_name: bool,
    /// Whether to ignore the type of the first argument to `describe` blocks,
    /// so titles that are not strings are allowed.
    ignore_type_of_describe_name: bool,
    /// Whether to allow identifiers (e.g. variables holding the title) as
    /// titles.
    allow_arguments: bool,
    /// Whether to allow leading and trailing spaces in titles.
    ignore_spaces: bool,
    /// A list of words that are not allowed to appear in titles.
    disallowed_words: Vec<CompactStr>,
    /// A pattern (or per-block patterns) that titles must **not** match,
    /// optionally paired with a custom message: `"pattern"`,
    /// `["pattern", "message"]`, or `{ "describe" | "test" | "it": pattern }`.
    must_not_match: Option<MatcherPatterns>,
    /// A pattern (or per-block patterns) that titles must match, optionally
    /// paired with a custom message: `"pattern"`, `["pattern", "message"]`,
    /// or `{ "describe" | "test" | "it": pattern }`.
    must_match: Option<MatcherPatterns>,
}

/// Patterns for `mustMatch`/`mustNotMatch`, either applied to all block types
/// or per block type.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum MatcherPatterns {
    /// A single pattern applied to `describe`, `test`, and `it` titles.
    All(MatcherPattern),
    /// Separate patterns per block type, e.g. `{ "describe": "pattern" }`.
    PerKind(FxHashMap<MatchKind, MatcherPattern>),
}

/// A single `mustMatch`/`mustNotMatch` pattern, with an optional custom
/// diagnostic message.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum MatcherPattern {
    /// A regex pattern, e.g. `"^[^#]+$"`.
    Pattern(CompactStr),
    /// A regex pattern followed by a custom message, e.g.
    /// `["^[^#]+$", "must not include a hash"]`.
    PatternAndMessage(Vec<CompactStr>),
}

#[derive(Debug, Default, Clone)]
pub struct ValidTitleConfig {
    /// Whether to ignore the type of the name passed to `test`.
    ignore_type_of_test_name: bool,
    /// Whether to ignore the type of the name passed to `describe`.
    ignore_type_of_describe_name: bool,
    /// Whether to allow arguments as titles.
    allow_arguments: bool,
    /// Matcher for disallowed words, which will not be allowed in titles.
    /// `None` when no disallowed words are configured.
    disallowed_words_reg: Option<Regex>,
    /// Whether to ignore leading and trailing spaces in titles.
    ignore_spaces: bool,
    /// Patterns for titles that must not match.
    must_not_match_patterns: FxHashMap<MatchKind, CompiledMatcherAndMessage>,
    /// Patterns for titles that must be matched for the title to be valid.
    must_match_patterns: FxHashMap<MatchKind, CompiledMatcherAndMessage>,
}

impl From<ValidTitleOptions> for ValidTitleConfig {
    fn from(options: ValidTitleOptions) -> Self {
        let disallowed_words_reg = (!options.disallowed_words.is_empty()).then(|| {
            let disallowed_words_pattern =
                options.disallowed_words.iter().map(|word| regex::escape(word)).join("|");
            Regex::new(&format!(r"(?iu)\b(?:{disallowed_words_pattern})\b"))
                .expect("escaped disallowed words should form a valid regex")
        });
        ValidTitleConfig {
            ignore_type_of_test_name: options.ignore_type_of_test_name,
            ignore_type_of_describe_name: options.ignore_type_of_describe_name,
            allow_arguments: options.allow_arguments,
            disallowed_words_reg,
            ignore_spaces: options.ignore_spaces,
            must_not_match_patterns: options
                .must_not_match
                .as_ref()
                .map(compile_matcher_patterns)
                .unwrap_or_default(),
            must_match_patterns: options
                .must_match
                .as_ref()
                .map(compile_matcher_patterns)
                .unwrap_or_default(),
        }
    }
}

impl ValidTitleConfig {
    pub fn from_configuration(
        value: serde_json::Value,
    ) -> Result<ValidTitleConfig, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<ValidTitleOptions>>(value)
            .map(|options| options.into_inner().into())
    }

    pub fn run_rule<'a>(
        &self,
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
        if self.allow_arguments && matches!(arg, Argument::Identifier(_)) {
            return;
        }

        let need_report_name = match jest_fn_call.kind {
            JestFnKind::General(JestGeneralFnKind::Test) => !self.ignore_type_of_test_name,
            JestFnKind::General(JestGeneralFnKind::Describe) => !self.ignore_type_of_describe_name,
            _ => unreachable!(),
        };

        match arg {
            Argument::StringLiteral(string_literal) => {
                validate_title(
                    &string_literal.value,
                    string_literal.span,
                    self,
                    &jest_fn_call.name,
                    ctx,
                );
            }
            // Handle String.raw`foo`
            Argument::TaggedTemplateExpression(tagged_template) => {
                if !is_string_raw_member_expression(&tagged_template.tag, ctx.scoping()) {
                    if need_report_name {
                        ctx.diagnostic(title_must_be_string_diagnostic(arg.span()));
                    }
                    return;
                }

                if let Some(quasi) = tagged_template.quasi.single_quasi() {
                    validate_title(
                        quasi.as_str(),
                        tagged_template.span,
                        self,
                        &jest_fn_call.name,
                        ctx,
                    );
                }
            }
            Argument::TemplateLiteral(template_literal) => {
                if let Some(quasi) = template_literal.single_quasi() {
                    validate_title(
                        quasi.as_str(),
                        template_literal.span,
                        self,
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

/// The kind of Jest/Vitest block a `mustMatch`/`mustNotMatch` pattern applies to.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MatchKind {
    Describe,
    It,
    Test,
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
    matcher_patterns: &MatcherPatterns,
) -> FxHashMap<MatchKind, CompiledMatcherAndMessage> {
    let mut map: FxHashMap<MatchKind, CompiledMatcherAndMessage> = FxHashMap::default();
    match matcher_patterns {
        MatcherPatterns::All(pattern) => {
            if let Some(compiled) = compile_matcher_pattern(pattern) {
                map.insert(MatchKind::Describe, compiled.clone());
                map.insert(MatchKind::Test, compiled.clone());
                map.insert(MatchKind::It, compiled);
            }
        }
        MatcherPatterns::PerKind(patterns) => {
            for (kind, pattern) in patterns {
                if let Some(compiled) = compile_matcher_pattern(pattern) {
                    map.insert(kind.clone(), compiled);
                }
            }
        }
    }
    map
}

/// Compiles a matcher pattern, returning `None` if the pattern is not a valid
/// Rust regex (e.g. it uses JS-only features such as lookaheads).
fn compile_matcher_pattern(pattern: &MatcherPattern) -> Option<CompiledMatcherAndMessage> {
    let (pattern_str, message) = match pattern {
        MatcherPattern::Pattern(pattern) => (pattern.as_str(), None),
        MatcherPattern::PatternAndMessage(parts) => {
            (parts.first()?.as_str(), parts.get(1).cloned())
        }
    };

    // Check for JS regex literal: /pattern/flags
    let regex = if let Some(stripped) = pattern_str.strip_prefix('/')
        && let Some(end) = stripped.rfind('/')
    {
        let (pat, _flags) = stripped.split_at(end);
        // For now, ignore flags and just use the pattern
        Regex::new(pat).ok()?
    } else {
        // Fallback: treat as a normal Rust regex with Unicode support
        Regex::new(&format!("(?u){pattern_str}")).ok()?
    };

    Some((regex, message))
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

    if let Some(disallowed_words_reg) = &config.disallowed_words_reg
        && let Some(matched) = disallowed_words_reg.find(title)
    {
        ctx.diagnostic(disallowed_word_diagnostic(matched.as_str(), span));
        return;
    }

    let trimmed_title = title.trim();
    if !config.ignore_spaces && trimmed_title != title {
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
