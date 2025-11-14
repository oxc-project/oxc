use std::cell::RefCell;
use std::sync::OnceLock;

use itertools::Itertools;
use oxc_ast::Comment;
use oxc_span::Span;
use rust_lapper::{Interval, Lapper};
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

use crate::fixer::Fix;

/// Check if a rule name corresponds to a known oxlint rule.
///
/// Unknown rules (e.g., from custom plugins) should not be reported as
/// unused directives since we cannot determine if they were actually needed.
/// This matches ESLint's behavior of only reporting unused directives for
/// rules from loaded plugins.
///
/// Supports remapped rule names:
/// - `vitest/*` rules map to `jest/*` equivalents
/// - `typescript/*` rules map to `eslint/*` equivalents
///
/// Uses a lazy-initialized cache of known rule names for O(1) lookup performance.
fn is_known_rule(rule_name: &str) -> bool {
    static KNOWN_RULES: OnceLock<FxHashSet<String>> = OnceLock::new();

    let known_rules = KNOWN_RULES.get_or_init(|| {
        use crate::rules::RULES;
        use crate::utils::{is_eslint_rule_adapted_to_typescript, is_jest_rule_adapted_to_vitest};

        let mut rules = FxHashSet::default();

        // Add all registered rules with their variants
        for rule in RULES.iter() {
            let plugin_name = rule.plugin_name();
            let name = rule.name();

            rules.insert(name.to_string()); // "no-debugger"
            rules.insert(format!("{plugin_name}/{name}")); // "eslint/no-debugger"
            rules.insert(format!("@{plugin_name}/{name}")); // "@typescript-eslint/no-explicit-any"

            // Add @typescript-eslint/* variant for typescript plugin rules
            // ESLint users use @typescript-eslint/rule-name format
            if plugin_name == "typescript" {
                rules.insert(format!("@typescript-eslint/{name}"));
            }

            // Add remapped rule names to cache
            // For jest rules that have vitest equivalents, add vitest/* variants
            if plugin_name == "jest" && is_jest_rule_adapted_to_vitest(name) {
                rules.insert(format!("vitest/{name}"));
                rules.insert(format!("@vitest/{name}"));
            }

            // For eslint rules that have typescript equivalents, add typescript/* variants
            if plugin_name == "eslint" && is_eslint_rule_adapted_to_typescript(name) {
                rules.insert(format!("typescript/{name}"));
                rules.insert(format!("@typescript/{name}"));
            }
        }

        rules
    });

    known_rules.contains(rule_name)
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum DisabledRule {
    All {
        comment_span: Span,
        is_next_line: bool,
        directive_prefix: &'static str,
    },
    Single {
        rule_name: String,
        name_span: Span,
        comment_span: Span,
        is_next_line: bool,
        directive_prefix: &'static str,
    },
}

impl DisabledRule {
    pub fn comment_span(&self) -> &Span {
        match self {
            DisabledRule::All { comment_span, .. } | DisabledRule::Single { comment_span, .. } => {
                comment_span
            }
        }
    }

    pub fn is_next_line(&self) -> bool {
        match self {
            DisabledRule::All { is_next_line, .. } | DisabledRule::Single { is_next_line, .. } => {
                *is_next_line
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RuleCommentRule {
    pub rule_name: String,
    pub name_span: Span,
}

impl RuleCommentRule {
    #[expect(clippy::cast_possible_truncation)] // for `as u32`
    pub fn create_fix(&self, source_text: &str, comment_span: Span) -> Fix {
        let before_source =
            &source_text[comment_span.start as usize..self.name_span.start as usize];

        // check if there is a comma before the rule name
        // if there is, remove the comma, whitespace and the rule name
        let mut comma_before_offset = None;
        for (i, c) in before_source.chars().rev().enumerate() {
            if c.is_whitespace() {
                continue;
            }
            if c == ',' {
                comma_before_offset = Some(1 + i as u32);
            }
            break;
        }

        if let Some(comma_before_offset) = comma_before_offset {
            return Fix::delete(Span::new(
                self.name_span.start - comma_before_offset,
                self.name_span.end,
            ));
        }

        let after_source = &source_text[self.name_span.end as usize..comment_span.end as usize];

        // check if there is a comma after the rule name
        // if there is, remove the comma, whitespace and the rule name
        let mut comma_after_offset = None;
        for (i, c) in after_source.char_indices() {
            if c.is_whitespace() {
                continue;
            }
            if c == ',' {
                comma_after_offset = Some(1 + i as u32);
            }
            break;
        }

        if let Some(comma_after_offset) = comma_after_offset {
            return Fix::delete(Span::new(
                self.name_span.start,
                self.name_span.end + comma_after_offset,
            ));
        }

        unreachable!(
            "A `RuleCommentRule` should have a comma, because only one rule should be RuleCommentType::All"
        );
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RuleCommentType {
    // disable/enable all the rules
    All,
    // disable/enable only a handful of rules
    Single(Vec<RuleCommentRule>),
}

/// A comment which disables one or more specific rules
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DisableRuleComment {
    /// Span of the comment
    pub span: Span,
    /// Rules disabled by the comment
    pub r#type: RuleCommentType,
    /// Directive prefix used ("eslint" or "oxlint")
    pub directive_prefix: &'static str,
}

#[derive(Debug, Clone)]
pub struct DisableDirectives {
    /// All the disabled rules with their corresponding covering spans
    intervals: Lapper<u32, DisabledRule>,
    /// All comments that disable one or more specific rules
    disable_rule_comments: Box<[DisableRuleComment]>,
    /// Spans of unused enable directives
    unused_enable_comments: Box<[(Option<String>, Span)]>,
    /// Spans of used enable directives, to filter out unused
    used_disable_comments: RefCell<Vec<DisabledRule>>,
}

impl DisableDirectives {
    fn mark_disable_directive_used(&self, disable_directive: DisabledRule) {
        self.used_disable_comments.borrow_mut().push(disable_directive);
    }

    pub fn contains(&self, rule_name: &str, span: Span) -> bool {
        // For `eslint-disable-next-line` and `eslint-disable-line` directives, we only check
        // if the diagnostic's starting position falls within the disabled interval.
        // This prevents suppressing diagnostics for larger constructs (like functions) that
        // contain disabled lines.
        //
        // For regular `eslint-disable` directives (which disable rules for the rest of the file),
        // we check if any part of the diagnostic span overlaps with the disabled interval.
        // This ensures that diagnostics starting before the disable comment (like no-empty-file)
        // are still suppressed.
        let matched_intervals = self
            .intervals
            .find(span.start, span.end)
            .filter(|interval| {
                // Check if this rule should be disabled
                let rule_matches = match &interval.val {
                    DisabledRule::All { .. } => true,
                    // Our rule name currently does not contain the prefix.
                    // For example, this will match `@typescript-eslint/no-var-requires` given
                    // our rule_name is `no-var-requires`.
                    DisabledRule::Single { rule_name: name, .. } => name.contains(rule_name),
                };

                if !rule_matches {
                    return false;
                }

                // Check if the diagnostic span is covered by this interval
                if interval.val.is_next_line() {
                    // For next-line directives, only check if the diagnostic starts within the interval
                    // We intentionally only check span.start (not span.end) to avoid suppressing
                    // diagnostics for large constructs that merely contain the disabled line
                    #[expect(clippy::suspicious_operation_groupings)]
                    {
                        span.start >= interval.start && span.start < interval.stop
                    }
                } else {
                    // For regular disable directives, check if there's any overlap
                    span.start < interval.stop && span.end > interval.start
                }
            })
            .map(|interval| interval.val.clone())
            .collect::<Vec<DisabledRule>>();

        for disable in &matched_intervals {
            self.mark_disable_directive_used(disable.clone());
        }

        !matched_intervals.is_empty()
    }

    pub fn disable_rule_comments(&self) -> &[DisableRuleComment] {
        &self.disable_rule_comments
    }

    pub fn unused_enable_comments(&self) -> &[(Option<String>, Span)] {
        &self.unused_enable_comments
    }

    pub fn collect_unused_disable_comments(&self) -> Vec<DisableRuleComment> {
        let used = self.used_disable_comments.borrow();

        self.intervals
            .iter()
            // 1. group intervals with the same interval.val.comment_span() together
            .chunk_by(|interval| interval.val.comment_span())
            .into_iter()
            // 2. iterate over all groups
            // 3. check if the group has only one , ore all entries with the comment span are used with `used.contains(&interval.val))`
            // 4. if all entries are used, map to RuleCommentType::All comment, otherwise map to RuleCommentType::Single comment.
            .filter_map(|(comment_span, group)| {
                let group_vec: Vec<_> = group.collect();

                // Extract directive_prefix from the first interval (all in group have same prefix)
                let first_interval = group_vec.first()?;
                let directive_prefix = match &first_interval.val {
                    DisabledRule::All { directive_prefix, .. }
                    | DisabledRule::Single { directive_prefix, .. } => *directive_prefix,
                };

                // Check if this directive disables all rules (e.g., `eslint-disable` with no specific rules)
                let is_disable_all_directive = group_vec
                    .iter()
                    .any(|interval| matches!(interval.val, DisabledRule::All { .. }));

                let rules: Vec<RuleCommentRule> = group_vec
                    .iter()
                    .filter_map(|interval| {
                        if used.contains(&interval.val) {
                            return None;
                        }
                        match &interval.val {
                            DisabledRule::Single {
                                rule_name, name_span, directive_prefix, ..
                            } => {
                                // Report rule as unused if:
                                // 1. It's a known rule (always report these)
                                // 2. It's an unknown rule AND directive_prefix is "oxlint"
                                //    (oxlint-disable with unknown rules should be reported as errors)
                                // Don't report unknown rules with "eslint" prefix (user might be running both linters)
                                if *directive_prefix == "oxlint" || is_known_rule(rule_name) {
                                    Some(RuleCommentRule {
                                        rule_name: rule_name.clone(),
                                        name_span: *name_span,
                                    })
                                } else {
                                    // Silently ignore unknown rules for eslint-disable directives
                                    None
                                }
                            }
                            DisabledRule::All { .. } => Some(RuleCommentRule {
                                rule_name: "all".to_string(),
                                name_span: *comment_span,
                            }),
                        }
                    })
                    .collect::<Vec<_>>();

                if rules.is_empty() {
                    return None;
                }

                // Use RuleCommentType::All when:
                // 1. The directive disables all rules (e.g., `eslint-disable` with no specific rules)
                // 2. The directive has only 1 rule total (represents the whole directive)
                // 3. All rules in the directive are unused (group_vec.len() == rules.len())
                // Otherwise use Single to show which specific rules are problematic
                if is_disable_all_directive
                    || group_vec.len() == 1
                    || group_vec.len() == rules.len()
                {
                    return Some(DisableRuleComment {
                        span: *comment_span,
                        r#type: RuleCommentType::All,
                        directive_prefix,
                    });
                }

                Some(DisableRuleComment {
                    span: *comment_span,
                    r#type: RuleCommentType::Single(rules),
                    directive_prefix,
                })
            })
            .collect()
    }
}

pub struct DisableDirectivesBuilder {
    /// All the disabled rules with their corresponding covering spans
    intervals: Lapper<u32, DisabledRule>,
    /// Start of `eslint-disable` or `oxlint-disable`
    disable_all_start: Option<(u32, Span, &'static str)>,
    /// Start of `eslint-disable` or `oxlint-disable` rule_name`
    disable_start_map: FxHashMap<String, (u32, Span, Span, &'static str)>,
    /// All comments that disable one or more specific rules
    disable_rule_comments: Vec<DisableRuleComment>,
    /// Spans of unused enable directives
    unused_enable_comments: Vec<(Option<String>, Span)>,
}

impl DisableDirectivesBuilder {
    pub fn new() -> Self {
        Self {
            intervals: Lapper::new(vec![]),
            disable_all_start: None,
            disable_start_map: FxHashMap::default(),
            disable_rule_comments: vec![],
            unused_enable_comments: vec![],
        }
    }

    pub fn build(mut self, source_text: &str, comments: &[Comment]) -> DisableDirectives {
        self.build_impl(source_text, comments);

        DisableDirectives {
            intervals: self.intervals,
            disable_rule_comments: self.disable_rule_comments.into_boxed_slice(),
            unused_enable_comments: self.unused_enable_comments.into_boxed_slice(),
            used_disable_comments: RefCell::new(Vec::new()),
        }
    }

    fn add_interval(&mut self, start: u32, stop: u32, val: DisabledRule) {
        self.intervals.insert(Interval { start, stop, val });
    }

    #[expect(clippy::cast_possible_truncation)] // for `as u32`
    fn build_impl(&mut self, source_text: &str, comments: &[Comment]) {
        let source_len = source_text.len() as u32;
        // This algorithm iterates through the comments and builds all intervals
        // for matching disable and enable pairs.
        // Wrongly ordered matching pairs are not taken into consideration.

        // NOTE: oxlint apply directive's logic not exactly same to eslint
        // only `disable-all` & `enable-all`, or `disable-rule` & `enable-rule` will add to intervals for disable directives
        // `disable-all` & `enable-rule` -> lone `disable-all` (enable-rule find not disable-rule before itself)
        // https://github.com/eslint/eslint/blob/f67d5e875324a9d899598b11807a9c7624021432/lib/linter/apply-disable-directives.js#L308

        // enable directive keep the same logic for checking unused
        let mut unused_enable_directives: Vec<(Option<String>, Span)> = vec![];

        for comment in comments {
            let comment_span = comment.content_span();
            let text_source = comment_span.source_text(source_text);
            let text = text_source.trim_start();
            let mut rule_name_start = comment_span.start + (text_source.len() - text.len()) as u32;

            // Detect which directive prefix is being used
            let directive_prefix: &'static str =
                if text_source.contains("oxlint-") { "oxlint" } else { "eslint" };

            if let Some(text) =
                text.strip_prefix("eslint-disable").or_else(|| text.strip_prefix("oxlint-disable"))
            {
                rule_name_start += 14; // eslint-disable is 14 bytes
                // `eslint-disable`
                if text.trim().is_empty() {
                    if self.disable_all_start.is_none() {
                        self.disable_all_start =
                            Some((comment_span.end, comment_span, directive_prefix));
                    }
                    self.disable_rule_comments.push(DisableRuleComment {
                        span: comment_span,
                        r#type: RuleCommentType::All,
                        directive_prefix,
                    });
                    continue;
                }
                // `eslint-disable-next-line`
                else if let Some(text) = text.strip_prefix("-next-line") {
                    rule_name_start += 10; // -next-line is 10 bytes
                    // Get the span up to the next new line
                    let mut stop = comment_span.end;
                    let mut lines_after_comment_end =
                        source_text[comment_span.end as usize..].split_inclusive('\n');

                    if let Some(rest_of_line) = lines_after_comment_end.next() {
                        stop += rest_of_line.len() as u32;
                    }

                    if let Some(next_line) = lines_after_comment_end.next() {
                        let next_line_trimmed = next_line.trim_end_matches(['\n', '\r']);
                        stop += next_line_trimmed.len() as u32;
                    }

                    if text.trim().is_empty() {
                        self.add_interval(
                            comment_span.end,
                            stop,
                            DisabledRule::All {
                                comment_span,
                                is_next_line: true,
                                directive_prefix,
                            },
                        );
                        self.disable_rule_comments.push(DisableRuleComment {
                            span: comment_span,
                            r#type: RuleCommentType::All,
                            directive_prefix,
                        });
                    } else {
                        // `eslint-disable-next-line rule_name1, rule_name2`
                        let mut rules = vec![];
                        Self::get_rule_names(text, rule_name_start, |rule_name, name_span| {
                            self.add_interval(
                                comment_span.end,
                                stop,
                                DisabledRule::Single {
                                    rule_name: rule_name.to_string(),
                                    name_span,
                                    comment_span,
                                    is_next_line: true,
                                    directive_prefix,
                                },
                            );
                            rules.push(RuleCommentRule {
                                rule_name: rule_name.to_string(),
                                name_span,
                            });
                        });
                        self.disable_rule_comments.push(DisableRuleComment {
                            span: comment_span,
                            r#type: RuleCommentType::Single(rules),
                            directive_prefix,
                        });
                    }
                    continue;
                }
                // `eslint-disable-line`
                else if let Some(text) = text.strip_prefix("-line") {
                    rule_name_start += 5; // -line is 5 bytes

                    // Get the span between the preceding newline to this comment
                    let start = source_text[..comment_span.start as usize]
                        .lines()
                        .next_back()
                        .map_or(0, |line| comment_span.start - line.len() as u32);
                    let stop = comment_span.start;

                    // `eslint-disable-line`
                    if text.trim().is_empty() {
                        self.add_interval(
                            start,
                            stop,
                            DisabledRule::All {
                                comment_span,
                                is_next_line: true,
                                directive_prefix,
                            },
                        );
                        self.disable_rule_comments.push(DisableRuleComment {
                            span: comment_span,
                            r#type: RuleCommentType::All,
                            directive_prefix,
                        });
                    } else {
                        // `eslint-disable-line rule-name1, rule-name2`
                        let mut rules = vec![];
                        Self::get_rule_names(text, rule_name_start, |rule_name, name_span| {
                            self.add_interval(
                                start,
                                stop,
                                DisabledRule::Single {
                                    rule_name: rule_name.to_string(),
                                    name_span,
                                    comment_span,
                                    is_next_line: true,
                                    directive_prefix,
                                },
                            );
                            rules.push(RuleCommentRule {
                                rule_name: rule_name.to_string(),
                                name_span,
                            });
                        });
                        self.disable_rule_comments.push(DisableRuleComment {
                            span: comment_span,
                            r#type: RuleCommentType::Single(rules),
                            directive_prefix,
                        });
                    }
                    continue;
                }
                // Remaining text should start with a whitespace character, else it's probably a typo of the correct syntax.
                // Like `eslint-disable-lext-nine` where `text` is `-lext-nine`, or directive is `eslint-disablefoo`
                else if text.starts_with(char::is_whitespace) {
                    // `eslint-disable rule-name1, rule-name2`
                    let mut rules = vec![];
                    Self::get_rule_names(text, rule_name_start, |rule_name, name_span| {
                        self.disable_start_map.entry(rule_name.to_string()).or_insert((
                            comment_span.end,
                            name_span,
                            comment_span,
                            directive_prefix,
                        ));
                        rules.push(RuleCommentRule { rule_name: rule_name.to_string(), name_span });
                    });
                    self.disable_rule_comments.push(DisableRuleComment {
                        span: comment_span,
                        r#type: RuleCommentType::Single(rules),
                        directive_prefix,
                    });
                    continue;
                }
            }

            if let Some(text) =
                text.strip_prefix("eslint-enable").or_else(|| text.strip_prefix("oxlint-enable"))
            {
                rule_name_start += 13; // eslint-enable is 13 bytes
                // `eslint-enable`
                if text.trim().is_empty() {
                    if let Some((start, _, stored_prefix)) = self.disable_all_start.take() {
                        self.add_interval(
                            start,
                            comment_span.start,
                            DisabledRule::All {
                                comment_span,
                                is_next_line: false,
                                directive_prefix: stored_prefix,
                            },
                        );
                    } else {
                        // collect as unused enable (see more at note comments in beginning of this method)
                        unused_enable_directives.push((None, comment_span));
                    }
                } else {
                    // `eslint-enable rule-name1, rule-name2`
                    Self::get_rule_names(text, rule_name_start, |rule_name, name_span| {
                        if let Some((start, _, _, stored_prefix)) =
                            self.disable_start_map.remove(rule_name)
                        {
                            self.add_interval(
                                start,
                                comment_span.start,
                                DisabledRule::Single {
                                    rule_name: rule_name.to_string(),
                                    name_span,
                                    comment_span,
                                    is_next_line: false,
                                    directive_prefix: stored_prefix,
                                },
                            );
                        } else {
                            // collect as unused enable (see more at note comments in beginning of this method)
                            unused_enable_directives.push((Some(rule_name.to_string()), name_span));
                        }
                    });
                }
            }
        }

        // Lone `eslint-disable`
        if let Some((start, comment_span, directive_prefix)) = self.disable_all_start {
            self.add_interval(
                start,
                source_len,
                DisabledRule::All { comment_span, is_next_line: false, directive_prefix },
            );
        }

        // Lone `eslint-disable rule_name`
        let disable_start_map = self.disable_start_map.drain().collect::<Vec<_>>();
        for (rule_name, (start, name_span, comment_span, directive_prefix)) in disable_start_map {
            self.add_interval(
                start,
                source_len,
                DisabledRule::Single {
                    rule_name: rule_name.clone(),
                    name_span,
                    comment_span,
                    is_next_line: false,
                    directive_prefix,
                },
            );
        }

        // Collect unused `enable` directives
        self.unused_enable_comments = unused_enable_directives;
    }

    #[expect(clippy::cast_possible_truncation)] // for `as u32`
    fn get_rule_names<F: FnMut(&str, Span)>(text: &str, rule_name_start: u32, mut cb: F) {
        if let Some(text) = text.split_terminator("--").next() {
            let mut rule_name_start: u32 = rule_name_start;

            for part in text.split(',') {
                let trimmed = part.trim();
                cb(
                    trimmed,
                    Span::sized(
                        rule_name_start + (part.len() - part.trim_start().len()) as u32,
                        trimmed.len() as u32,
                    ),
                );

                rule_name_start += 1 + part.len() as u32; // +1 for the next ","
            }
        }
    }
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::EslintNoDebugger, tester::Tester};

    for prefix in ["eslint", "oxlint"] {
        // [Disabling Rules](https://eslint.org/docs/latest/use/configure/rules#disabling-rules)
        // Using configuration comments
        let pass = vec![
            // To disable rule warnings in a part of a file, use block comments in the following format:
            format!(
                "
        /* {prefix}-disable */
            debugger;
        /* {prefix}-enable */
        "
            ),
            // You can also disable or enable warnings for specific rules:
            format!(
                "
        /* {prefix}-disable no-debugger, no-console */
            debugger;
        /* {prefix}-enable no-debugger, no-console */
        "
            ),
            // To disable rule warnings in an entire file, put a /* eslint-disable */ block comment at the top of the file:
            format!(
                "
        /* {prefix}-disable */
            debugger;
        "
            ),
            // You can also disable or enable specific rules for an entire file:
            format!(
                "
        /* {prefix}-disable no-debugger */
            debugger;
        "
            ),
            // To ensure that a rule is never applied (regardless of any future enable/disable lines):
            // This is not supported.
            // "
            // /* eslint no-debugger: \"off\" */
            //     debugger;
            // "),
            // To disable all rules on a specific line, use a line or block comment in one of the following formats:
            format!(
                "debugger; // {prefix}-disable-line
            debugger; // {prefix}-disable-line

            // {prefix}-disable-next-line
            debugger;

            /* {prefix}-disable-next-line */
            debugger;

            debugger; /* {prefix}-disable-line */
        "
            ),
            // To disable a specific rule on a specific line:
            format!(
                "
            debugger; // {prefix}-disable-line no-debugger

            // {prefix}-disable-next-line no-debugger
            debugger;

            debugger; /* {prefix}-disable-line no-debugger */

            /* {prefix}-disable-next-line no-debugger */
            debugger;
        "
            ),
            // To disable multiple rules on a specific line:
            format!(
                "
            debugger; // {prefix}-disable-line no-debugger, quotes, semi

            // {prefix}-disable-next-line no-debugger, quotes, semi
            debugger;

            debugger; /* {prefix}-disable-line no-debugger, quotes, semi */

            /* {prefix}-disable-next-line no-debugger, quotes, semi */
            debugger;

            /* {prefix}-disable-next-line
              no-debugger,
              quotes,
              semi
            */
            debugger;
        "
            ),
            // To disable all rules twice:
            format!(
                "
        /* {prefix}-disable */
            debugger;
        /* {prefix}-disable */
            debugger;
        "
            ),
            // To disable a rule twice:
            format!(
                "
        /* {prefix}-disable no-debugger */
            debugger;
        /* {prefix}-disable no-debugger */
            debugger;
        "
            ),
            // Comment descriptions
            format!(
                "
            // {prefix}-disable-next-line no-debugger -- Here's a description about why this configuration is necessary.
            debugger;

            /* {prefix}-disable-next-line no-debugger --
             * Here's a very long description about why this configuration is necessary
             * along with some additional information
            **/
            debugger;
        "
            ),
            // Should only match `eslint-enable` comments, not `eslint-enablefoo`
            format!("
            /* {prefix}-disable */
                debugger;
            /* {prefix}-enablefoo */
                debugger;
            "
            ),
            format!("
            /* {prefix}-disable no-debugger, no-console */
                debugger;
            /* {prefix}-enablefoo no-debugger, no-console */
                debugger;
            "
            ),
            // Handles no spaces in comment
            format!(
                "debugger; //{prefix}-disable-line
            debugger; //{prefix}-disable-line

            //{prefix}-disable-next-line
            debugger;

            /*{prefix}-disable-next-line*/
            debugger;

            debugger; /*{prefix}-disable-line*/

            debugger; //{prefix}-disable-line no-debugger

            //{prefix}-disable-next-line no-debugger
            debugger;

            debugger; /*{prefix}-disable-line no-debugger*/

            /*{prefix}-disable-next-line no-debugger*/
            debugger;
        "
            ),
            // Handles extra spaces in comment
            format!(
                "debugger; //       {prefix}-disable-line
            debugger; // \t\t {prefix}-disable-line

            //         {prefix}-disable-next-line
            debugger;

            /*      {prefix}-disable-next-line        */
            debugger;

            debugger; /*    {prefix}-disable-line       */

            debugger; //            {prefix}-disable-line no-debugger

            //          {prefix}-disable-next-line no-debugger
            debugger;

            debugger; /*     \t   {prefix}-disable-line no-debugger*/

            /*    \t   {prefix}-disable-next-line no-debugger       */
            debugger;
        "
            ),
            // Handles whitespace character before rule name in comment
            format!(
                "/*{prefix}-disable
no-debugger
*/
            debugger;
        "
            ),
            // Extra commas
            format!(
                "
            debugger // {prefix}-disable-line no-debugger,
            debugger // {prefix}-disable-line ,no-debugger
            debugger // {prefix}-disable-line no-debugger,,
            debugger // {prefix}-disable-line ,,no-debugger,,
            debugger // {prefix}-disable-line ,,no-debugger,,semi,,
            debugger // {prefix}-disable-line ,,no-debugger,,no-debugger,,
            debugger // {prefix}-disable-line ,  , ,,no-debugger, , ,

            // {prefix}-disable-next-line no-debugger,
            debugger
            // {prefix}-disable-next-line ,no-debugger,
            debugger
            // {prefix}-disable-next-line no-debugger,,
            debugger
            // {prefix}-disable-next-line ,,no-debugger,,
            debugger
            // {prefix}-disable-next-line ,,no-debugger,,semi,,
            debugger
            // {prefix}-disable-next-line ,,no-debugger,,no-debugger,,
            debugger
            // {prefix}-disable-next-line ,  , ,,no-debugger, , ,
        "
            ),
            format!("
                /* {prefix}-disable , ,no-debugger, , */
                debugger;
            "),
            format!("debugger;//{prefix}-disable-line")
        ];

        let fail = vec![
            "debugger".to_string(),
            format!(
                "
            debugger; // {prefix}-disable-line no-alert

            // {prefix}-disable-next-line no-alert
            debugger;

            debugger; /* {prefix}-disable-line no-alert */

            /* {prefix}-disable-next-line no-alert */
            debugger;
        "
            ),
            format!(
                "
            debugger; // {prefix}-disable-line no-alert, quotes, semi

            // {prefix}-disable-next-line no-alert, quotes, semi
            debugger;

            debugger; /* {prefix}-disable-line no-alert, quotes, semi */

            /* {prefix}-disable-next-line no-alert, quotes, semi */
            debugger;

            /* {prefix}-disable-next-line
              no-alert,
              quotes,
              semi
            */
            debugger;
        "
            ),
            format!(
                "
            /* {prefix}-disable-next-line no-debugger --
             * Here's a very long description about why this configuration is necessary
             * along with some additional information
            **/
            debugger;
            debugger;
        "
            ),
            format!(
                "
            // {prefix}-disable-next-line no-debugger
            debugger;
            debugger;
        "
            ),
            // Should not match invalid directives
            // https://github.com/oxc-project/oxc/issues/6041
            format!(
                "// {prefix}-disable-lext-nine no-debugger
                debugger;
                "
            ),
            format!(
                "// {prefix}-disabled no-debugger
                debugger;
                "
            ),
            format!(
                "// {prefix}-disabled
                debugger;
                "
            ),
            format!(
                "
            debugger; // {prefix}-disable-lext-nine no-debugger

            // {prefix}-disable-lext-nine no-debugger
            debugger;

            debugger; /* {prefix}-disable-lin no-debugger */

            /* {prefix}-disable-next-lin no-debugger */
            debugger;
        "
            ),
            format!(
                "debugger; // {prefix}-disable-linefoo
            debugger; // {prefix}-disable-linefoo

            // {prefix}-disable-next-linefoo
            debugger;

            /* {prefix}-disable-next-linefoo */
            debugger;

            debugger; /* {prefix}-disable-linefoo */
        "
            ),
            format!(
                "
            /* {prefix}-disable */
                debugger;
            /* {prefix}-enable */
                debugger;
            "
            ),
            format!(
                "
            /* {prefix}-disable no-debugger, no-console */
                debugger;
            /* {prefix}-enable no-debugger, no-console */
                debugger;
            "
            ),
            // Handles no spaces in comment
            format!(
                "
            /*{prefix}-disable*/
                debugger;
            /*{prefix}-enable*/
                debugger;
            "
            ),
            format!(
                "
            /*{prefix}-disable no-debugger,no-console*/
                debugger;
            /*{prefix}-enable no-debugger,no-console*/
                debugger;
            "
            ),
            format!(
                "debugger; //{prefix}-disable-line no-alert,quotes,semi
            //{prefix}-disable-next-line no-alert,quotes,semi
            debugger;
            debugger; /*{prefix}-disable-line no-alert,quotes,semi */
            /*{prefix}-disable-next-line no-alert,quotes,semi */
            debugger;
            /*{prefix}-disable-next-line
no-alert,
quotes,
semi*/
            debugger;
        "
            ),
            // Handles extra spaces in comment
            format!(
                "
            /*   \t\t {prefix}-disable   \t\t*/
                debugger;
            /*   \t\t {prefix}-enable   \t\t*/
                debugger;
            "
            ),
            format!(
                "
            /*   \t\t {prefix}-disable    \t\t no-debugger,   \t\t no-console   \t\t */
                debugger;
            /*   \t\t {prefix}-enable    \t\t no-debugger,   \t\t no-console   \t\t */
                debugger;
            "
            ),
            format!(
                "debugger; //   \t\t {prefix}-disable-line   \t\t  no-alert,   \t\t quotes,   \t\t semi   \t\t
            //   \t\t {prefix}-disable-next-line   \t\t  no-alert,   \t\t quotes,   \t\t semi
            debugger;
            debugger; /*   \t\t {prefix}-disable-line    \t\t no-alert,   \t\t quotes,   \t\t semi   \t\t  */
            /*   \t\t {prefix}-disable-next-line   \t\t  no-alert,   \t\t quotes,   \t\t semi */
            debugger;
            /*  \t\t {prefix}-disable-next-line
  \t\t no-alert,  \t\t
  \t\t quotes,  \t\t
  \t\t semi  \t\t */
            debugger;
        "
            ),
        ];

        Tester::new(EslintNoDebugger::NAME, EslintNoDebugger::PLUGIN, pass, fail)
            .intentionally_allow_no_fix_tests()
            .test();
    }
}

/// Create diagnostics for unused disable directives.
///
/// This utility function generates `OxcDiagnostic` instances for:
/// - Unused disable directives (no problems were reported)
/// - Unused enable directives (no matching disable directives)
///
/// # Arguments
/// * `directives` - The disable directives to check for unused comments
/// * `severity` - The severity level (Warn or Deny) for the diagnostics
///
/// # Returns
/// A vector of diagnostics for all unused directives
pub fn create_unused_directives_diagnostics(
    directives: &DisableDirectives,
    severity: crate::AllowWarnDeny,
) -> Vec<oxc_diagnostics::OxcDiagnostic> {
    use oxc_diagnostics::OxcDiagnostic;

    let mut diagnostics = Vec::new();

    let severity = if severity == crate::AllowWarnDeny::Deny {
        oxc_diagnostics::Severity::Error
    } else {
        oxc_diagnostics::Severity::Warning
    };

    // Report unused disable comments
    let unused_disable = directives.collect_unused_disable_comments();
    for unused_comment in unused_disable {
        let span = unused_comment.span;
        let prefix = unused_comment.directive_prefix;
        match unused_comment.r#type {
            RuleCommentType::All => {
                let message =
                    format!("Unused {prefix}-disable directive (no problems were reported).");
                diagnostics
                    .push(OxcDiagnostic::warn(message).with_label(span).with_severity(severity));
            }
            RuleCommentType::Single(rules) => {
                for rule in rules {
                    let rule_name = &rule.rule_name;
                    let rule_message = format!(
                        "Unused {prefix}-disable directive (no problems were reported from {rule_name})."
                    );
                    diagnostics.push(
                        OxcDiagnostic::warn(rule_message)
                            .with_label(rule.name_span)
                            .with_severity(severity),
                    );
                }
            }
        }
    }

    // Report unused enable comments
    let unused_enable = directives.unused_enable_comments();
    for (rule_name, span) in unused_enable {
        let message = if let Some(rule_name) = rule_name {
            format!(
                "Unused eslint-enable directive (no matching eslint-disable directives were found for {rule_name})."
            )
        } else {
            "Unused eslint-enable directive (no matching eslint-disable directives were found)."
                .to_string()
        };
        diagnostics.push(OxcDiagnostic::warn(message).with_label(*span).with_severity(severity));
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_ast::Comment;
    use oxc_parser::Parser;
    use oxc_semantic::{Semantic, SemanticBuilder};
    use oxc_span::{SourceType, Span};

    use crate::disable_directives::{DisabledRule, RuleCommentRule, RuleCommentType};

    use super::{DisableDirectives, DisableDirectivesBuilder};

    fn process_source<'a>(allocator: &'a Allocator, source_text: &'a str) -> Semantic<'a> {
        let source_type = SourceType::default();
        let parser_ret = Parser::new(allocator, source_text, source_type).parse();
        let semantic_ret = SemanticBuilder::new().build(allocator.alloc(parser_ret.program));
        semantic_ret.semantic
    }

    fn test_directives(
        create_source_text: impl Fn(&'static str) -> String,
        test: impl Fn(&'static str, &[Comment], DisableDirectives),
    ) {
        let allocator = Allocator::default();
        for prefix in ["eslint", "oxlint"] {
            let source_text = create_source_text(prefix);
            let semantic = process_source(&allocator, &source_text);
            let comments = semantic.comments();
            let directives =
                DisableDirectivesBuilder::new().build(semantic.source_text(), comments);
            test(prefix, comments, directives);
        }
    }

    fn test_directive_span(source_text: &str, expected_start: u32, expected_stop: u32) {
        let allocator = Allocator::default();
        let semantic = process_source(&allocator, source_text);
        let directives =
            DisableDirectivesBuilder::new().build(semantic.source_text(), semantic.comments());
        let interval = &directives.intervals.intervals[0];

        assert_eq!(interval.start, expected_start);
        assert_eq!(interval.stop, expected_stop);
    }

    #[test]
    fn unused_enable_all() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    console.log();
                    /* {prefix}-enable */
                    console.log();
                    "
                )
            },
            |_prefix, comments, directives| {
                let unused = directives.unused_enable_comments();

                assert_eq!(unused.len(), 1);

                let (unused_rule_name, unused_span) = unused.first().unwrap();
                let comment_span = comments.first().unwrap().content_span();
                assert_eq!(unused_rule_name.as_deref(), None);
                assert_eq!(*unused_span, comment_span);
            },
        );
    }

    #[test]
    fn unused_enable_rules() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    console.log();
                    /* {prefix}-enable no-debugger, no-console */
                    console.log();
                    "
                )
            },
            |_prefix, comments, directives| {
                let unused = directives.unused_enable_comments();

                assert_eq!(unused.len(), 2);

                let (unused_rule_name_no_debugger, unused_span_no_debugger) =
                    unused.first().unwrap();
                assert_eq!(unused_rule_name_no_debugger.as_deref(), Some("no-debugger"));
                assert_eq!(
                    *unused_span_no_debugger,
                    Span::sized(comments[0].content_span().start + 15, 11)
                );

                let (unused_rule_name_no_console, unused_span_no_console) = unused.last().unwrap();
                assert_eq!(unused_rule_name_no_console.as_deref(), Some("no-console"));
                assert_eq!(
                    *unused_span_no_console,
                    Span::sized(comments[0].content_span().start + 28, 10)
                );
            },
        );
    }

    #[test]
    fn no_unused_enable() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    /* {prefix}-disable no-console */
                    console.log();
                    /* {prefix}-enable no-console */
                    console.log();
                    "
                )
            },
            |_, _, directives| {
                // no mark unused

                let unused = directives.unused_enable_comments();

                assert!(unused.is_empty());
            },
        );
    }

    #[test]
    fn unused_disable_all() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    /* {prefix}-disable */
                    console.log();
                    "
                )
            },
            |_prefix, comments, directives| {
                // no mark unused

                let unused = directives.collect_unused_disable_comments();

                assert_eq!(unused.len(), 1);

                let comment = unused.first().unwrap();
                assert_eq!(comment.span, comments.first().unwrap().content_span());
                assert_eq!(comment.r#type, RuleCommentType::All);
            },
        );
    }

    #[test]
    fn unused_disable_rules() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    /* {prefix}-disable no-debugger, no-console */
                    for (let i = 0; i < 10; i++) {{ const x = 0; }}
                    "
                )
            },
            |_prefix, comments, directives| {
                // no mark unused

                let unused = directives.collect_unused_disable_comments();

                assert_eq!(unused.len(), 1);

                let comment = unused.first().unwrap();
                assert_eq!(comment.span, comments.first().unwrap().content_span());
                // 2 unused rules -> uses All (rules.len() > 1)
                assert_eq!(comment.r#type, RuleCommentType::All);
            },
        );
    }

    #[test]
    fn no_unused_disable() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    /* {prefix}-disable no-console */
                    console.log();
                    /* {prefix}-disable no-debugger */
                    debugger;
                    "
                )
            },
            |prefix, comments, directives| {
                directives.mark_disable_directive_used(DisabledRule::Single {
                    rule_name: "no-console".to_string(),
                    name_span: Span::sized(comments[0].content_span().start + 16, 10),
                    comment_span: comments[0].content_span(),
                    is_next_line: false,
                    directive_prefix: prefix,
                });
                directives.mark_disable_directive_used(DisabledRule::Single {
                    rule_name: "no-debugger".to_string(),
                    name_span: Span::sized(comments[1].content_span().start + 16, 11),
                    comment_span: comments[1].content_span(),
                    is_next_line: false,
                    directive_prefix: prefix,
                });

                assert!(directives.collect_unused_disable_comments().is_empty());
            },
        );
    }

    #[test]
    fn next_line_span_of_line_comment() {
        test_directive_span("// eslint-disable-next-line max-params", 38, 38);
        test_directive_span("// eslint-disable-next-line max-params\n", 38, 39);
        test_directive_span("// eslint-disable-next-line max-params\r\n", 38, 40);
        test_directive_span("// eslint-disable-next-line max-params    \n", 42, 43);
        test_directive_span("// eslint-disable-next-line max-params    \r\n", 42, 44);
        test_directive_span("// eslint-disable-next-line max-params    \n ABC", 42, 47);
        test_directive_span("// eslint-disable-next-line max-params    \r\n ABC", 42, 48);
        test_directive_span("// eslint-disable-next-line max-params    \n ABC \n", 42, 48);
        test_directive_span("// eslint-disable-next-line max-params    \r\n ABC \r\n", 42, 49);
    }

    #[test]
    #[expect(clippy::cast_possible_truncation)] // for `as u32`
    fn test_rule_comment_rule_create_fix() {
        let source_text = "// eslint-disable-next-line max-params, no-console, no-debugger";
        let comment_span = Span::sized(3, source_text.len() as u32 - 3);

        let max_params_fix =
            RuleCommentRule { rule_name: "max-params".to_string(), name_span: Span::sized(28, 10) }
                .create_fix(source_text, comment_span);

        assert_eq!(&source_text[28..38], "max-params");
        assert_eq!(max_params_fix.span, Span::sized(28, 11)); // max-params is 10 + 1 for the comma

        let no_console_fix =
            RuleCommentRule { rule_name: "no-console".to_string(), name_span: Span::sized(40, 10) }
                .create_fix(source_text, comment_span);

        assert_eq!(&source_text[40..50], "no-console");
        assert_eq!(no_console_fix.span, Span::sized(38, 12)); // no-console is 10 + 2 for the comma before and the space

        let no_debugger_fix = RuleCommentRule {
            rule_name: "no-debugger".to_string(),
            name_span: Span::sized(52, 11),
        }
        .create_fix(source_text, comment_span);

        assert_eq!(&source_text[52..63], "no-debugger");
        assert_eq!(no_debugger_fix.span, Span::sized(50, 13)); // no-debugger is 11 + 2 for the comma before and the space
    }

    #[test]
    fn test_disable_next_line_should_not_disable_large_span_diagnostics() {
        // This test demonstrates that eslint-disable-next-line should NOT suppress
        // diagnostics for larger constructs (like functions) that contain the disabled line.
        // It should only suppress diagnostics that START on the disabled line.
        let source_text = r"
function test() {
    // eslint-disable-next-line
    console.log('this line is disabled');
    console.warn('this line is not disabled');
}
";
        let allocator = Allocator::default();
        let semantic = process_source(&allocator, source_text);
        let directives =
            DisableDirectivesBuilder::new().build(semantic.source_text(), semantic.comments());

        // The function spans from line 2 to line 6 (positions 1 to 138)
        let function_span = Span::new(1, 138);

        // The diagnostic for the entire function should NOT be suppressed
        // even though it contains a disable-next-line directive
        assert!(
            !directives.contains("max-lines-per-function", function_span),
            "eslint-disable-next-line should not suppress diagnostics for the entire function"
        );

        // A diagnostic that starts on the disabled line (line 4) SHOULD be suppressed
        // The first console.log on line 4 starts at position 59
        let first_console_log_span = Span::new(55, 66);
        assert_eq!(first_console_log_span.source_text(source_text), "console.log");
        assert!(
            directives.contains("no-console", first_console_log_span),
            "eslint-disable-next-line should suppress diagnostics on the next line"
        );

        // A diagnostic that starts on a non-disabled line (line 5) should NOT be suppressed
        // The second console.log on line 5 starts at position 102
        let second_console_log_span = Span::new(97, 109);
        assert_eq!(second_console_log_span.source_text(source_text), "console.warn");
        assert!(
            !directives.contains("no-console", second_console_log_span),
            "eslint-disable-next-line should NOT suppress diagnostics on lines after the next line"
        );
    }

    // Tests for issue #11983: Unknown ESLint rules should not be reported as unused

    #[test]
    fn unknown_plugin_rules_not_reported_as_unused() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    /* {prefix}-disable custom-plugin/custom-rule */
                    console.log();
                    "
                )
            },
            |prefix, _comments, directives| {
                let unused = directives.collect_unused_disable_comments();
                // eslint-disable: Should NOT report unknown rules (user might be using both linters)
                // oxlint-disable: Should report unknown rules (user is explicitly targeting oxlint)
                if prefix == "eslint" {
                    assert_eq!(
                        unused.len(),
                        0,
                        "eslint-disable with unknown plugin rules should not be reported"
                    );
                } else {
                    assert_eq!(
                        unused.len(),
                        1,
                        "oxlint-disable with unknown plugin rules should be reported"
                    );
                }
            },
        );
    }

    #[test]
    fn known_rules_reported_as_unused() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    /* {prefix}-disable no-debugger */
                    console.log();
                    "
                )
            },
            |_prefix, comments, directives| {
                let unused = directives.collect_unused_disable_comments();
                // Should report no-debugger as unused (it's a known rule with no violation)
                assert_eq!(unused.len(), 1, "Known unused rules should be reported");
                assert_eq!(unused[0].span, comments.first().unwrap().content_span());
                // Only 1 rule in directive → use All (represents whole directive)
                assert_eq!(unused[0].r#type, RuleCommentType::All);
            },
        );
    }

    #[test]
    fn mixed_known_and_unknown_rules() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    /* {prefix}-disable no-debugger, custom-plugin/my-rule, no-console */
                    console.log();
                    "
                )
            },
            |prefix, _comments, directives| {
                let unused = directives.collect_unused_disable_comments();

                assert_eq!(unused.len(), 1, "Should report comment with unused rules");

                if prefix == "eslint" {
                    // eslint-disable: 3 total rules, 2 unused (unknown ignored)
                    // group_vec.len() = 3, rules.len() = 2 → NOT all unused → use Single
                    assert!(
                        matches!(unused[0].r#type, RuleCommentType::Single(_)),
                        "Expected RuleCommentType::Single but got {:?}",
                        unused[0].r#type
                    );
                } else {
                    // oxlint-disable: 3 total rules, all 3 unused (2 known + 1 unknown)
                    // group_vec.len() = 3, rules.len() = 3 → all unused → use All
                    assert_eq!(
                        unused[0].r#type,
                        RuleCommentType::All,
                        "Expected RuleCommentType::All when all rules are unused"
                    );
                }
            },
        );
    }

    #[test]
    fn typescript_eslint_plugin_unknown_rules() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    /* {prefix}-disable @custom-org/eslint-plugin/some-rule */
                    const x = 1;
                    "
                )
            },
            |prefix, _comments, directives| {
                let unused = directives.collect_unused_disable_comments();

                if prefix == "eslint" {
                    // eslint-disable: Custom org plugins should not be reported as unused
                    assert_eq!(
                        unused.len(),
                        0,
                        "eslint-disable with unknown @-prefixed plugin rules should not be reported"
                    );
                } else {
                    // oxlint-disable: Should report unknown rules
                    assert_eq!(
                        unused.len(),
                        1,
                        "oxlint-disable with unknown @-prefixed plugin rules should be reported"
                    );
                }
            },
        );
    }

    #[test]
    fn all_unknown_rules_results_in_no_unused_report() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    /* {prefix}-disable custom-plugin/rule1, another-plugin/rule2 */
                    console.log();
                    "
                )
            },
            |prefix, _comments, directives| {
                let unused = directives.collect_unused_disable_comments();

                if prefix == "eslint" {
                    // eslint-disable: If all rules are unknown, should not report anything
                    assert_eq!(
                        unused.len(),
                        0,
                        "eslint-disable with all unknown rules should not report any unused directives"
                    );
                } else {
                    // oxlint-disable: Should report all unknown rules
                    assert_eq!(
                        unused.len(),
                        1,
                        "oxlint-disable with all unknown rules should report them"
                    );
                }
            },
        );
    }

    // Tests for remapped rule names (issue #11983)

    #[test]
    fn remapped_vitest_rules_recognized_as_known() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    // {prefix}-disable-next-line vitest/expect-expect
                    console.log();
                    "
                )
            },
            |_, _, directives| {
                let unused = directives.collect_unused_disable_comments();
                // Should report vitest/expect-expect as unused since it's a known rule
                // (remapped from jest/expect-expect) and there's no violation
                assert_eq!(
                    unused.len(),
                    1,
                    "Remapped Vitest rules should be recognized as known rules"
                );
            },
        );
    }

    #[test]
    fn remapped_typescript_rules_recognized_as_known() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    // {prefix}-disable-next-line typescript/max-params
                    console.log();
                    "
                )
            },
            |_, _, directives| {
                let unused = directives.collect_unused_disable_comments();
                // Should report typescript/max-params as unused since it's a known rule
                // (remapped from eslint/max-params) and there's no violation
                assert_eq!(
                    unused.len(),
                    1,
                    "Remapped TypeScript rules should be recognized as known rules"
                );
            },
        );
    }

    #[test]
    fn remapped_typescript_rules_not_in_compatibility_list_treated_as_unknown() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    // {prefix}-disable-next-line typescript/no-var
                    console.log();
                    "
                )
            },
            |prefix, _comments, directives| {
                let unused = directives.collect_unused_disable_comments();

                if prefix == "eslint" {
                    // Should NOT report typescript/no-var as unused because
                    // even though eslint/no-var exists, no-var is NOT in the
                    // TYPESCRIPT_COMPATIBLE_ESLINT_RULES list, so we treat it as unknown
                    assert_eq!(
                        unused.len(),
                        0,
                        "eslint-disable with TypeScript-prefixed rules not in compatibility list should not be reported"
                    );
                } else {
                    // oxlint-disable: Should report unknown rules
                    assert_eq!(
                        unused.len(),
                        1,
                        "oxlint-disable with TypeScript-prefixed rules not in compatibility list should be reported"
                    );
                }
            },
        );
    }

    #[test]
    fn remapped_vitest_rules_not_in_compatibility_list_treated_as_unknown() {
        test_directives(
            |prefix| {
                format!(
                    r"
                    // {prefix}-disable-next-line vitest/no-disabled-tests
                    console.log();
                    "
                )
            },
            |_, _, directives| {
                let unused = directives.collect_unused_disable_comments();
                // vitest/no-disabled-tests should be recognized as known (it IS in the list)
                // This test verifies our remapping works for an actual implemented rule
                assert_eq!(
                    unused.len(),
                    1,
                    "Vitest rules in compatibility list should be recognized when implemented"
                );
            },
        );
    }
}
