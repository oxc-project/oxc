use std::cell::RefCell;

use itertools::Itertools;
use oxc_ast::Comment;
use oxc_span::Span;
use rust_lapper::{Interval, Lapper};
use rustc_hash::FxHashMap;

use crate::{FixKind, fixer::Fix};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum DirectivePrefix {
    Eslint,
    Oxlint,
}

impl DirectivePrefix {
    #[must_use]
    pub const fn disable_directive_name(self) -> &'static str {
        match self {
            Self::Eslint => "eslint-disable",
            Self::Oxlint => "oxlint-disable",
        }
    }

    #[must_use]
    pub const fn enable_directive_name(self) -> &'static str {
        match self {
            Self::Eslint => "eslint-enable",
            Self::Oxlint => "oxlint-enable",
        }
    }

    #[must_use]
    pub fn unused_disable_message(self) -> String {
        format!("Unused {} directive (no problems were reported).", self.disable_directive_name())
    }

    #[must_use]
    pub fn unused_disable_rule_message(self, rule_name: &str) -> String {
        format!(
            "Unused {} directive (no problems were reported from {rule_name}).",
            self.disable_directive_name()
        )
    }

    #[must_use]
    pub fn unused_enable_message(self) -> String {
        format!(
            "Unused {} directive (no matching {} directives were found).",
            self.enable_directive_name(),
            self.disable_directive_name()
        )
    }

    #[must_use]
    pub fn unused_enable_rule_message(self, rule_name: &str) -> String {
        format!(
            "Unused {} directive (no matching {} directives were found for {rule_name}).",
            self.enable_directive_name(),
            self.disable_directive_name()
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum DisabledRule {
    /// Disables all linting rules for a span of code.
    /// Used by directives like `eslint-disable`, `eslint-disable-next-line`, or `eslint-disable-line` without specific rule names.
    ///
    /// # Example
    /// ```text
    /// /* eslint-disable */
    ///    ^^^^^^^^^^^^^^^ comment_span
    /// ```
    All {
        /// Prefix used by the directive comment (`eslint` or `oxlint`).
        directive_prefix: DirectivePrefix,
        /// Full outer span of the comment (including `//` or `/* */` delimiters).
        /// Used for diagnostic labels.
        comment_span: Span,
        /// Span used for the fix.  Extends to cover the whole line (including leading
        /// whitespace and the trailing newline) when the comment is the only content
        /// on that line; otherwise equals `comment_span`.
        fix_span: Span,
        /// Whether this is a line-specific directive (`-next-line` or `-line`).
        is_next_line: bool,
    },
    /// Disables a single specific linting rule for a span of code.
    /// Used by directives like `eslint-disable rule-name`, `eslint-disable-next-line rule-name`, or `eslint-disable-line rule-name`.
    ///
    /// # Example
    /// ```text
    /// /* eslint-disable no-debugger */
    ///   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ comment_span
    ///                   ^^^^^^^^^^^  name_span (for "no-debugger")
    /// ```
    Single {
        /// Prefix used by the directive comment (`eslint` or `oxlint`).
        directive_prefix: DirectivePrefix,
        /// Name of the disabled rule (e.g., "no-debugger", "no-console")
        rule_name: String,
        /// Span of the rule name within the comment.
        ///
        /// For `/* eslint-disable no-debugger */`, this points to "no-debugger".
        name_span: Span,
        /// Full outer span of the comment (including `//` or `/* */` delimiters).
        /// Used for diagnostic labels.
        comment_span: Span,
        /// Span used for the fix.  Extends to cover the whole line (including leading
        /// whitespace and the trailing newline) when the comment is the only content
        /// on that line; otherwise equals `comment_span`.
        fix_span: Span,
        /// Whether this is a line-specific directive (`-next-line` or `-line`).
        /// When true, only diagnostics starting within the interval are suppressed.
        /// When false, any diagnostic overlapping the interval is suppressed.
        is_next_line: bool,
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

    pub fn fix_span(&self) -> &Span {
        match self {
            DisabledRule::All { fix_span, .. } | DisabledRule::Single { fix_span, .. } => fix_span,
        }
    }

    pub fn is_next_line(&self) -> bool {
        match self {
            DisabledRule::All { is_next_line, .. } | DisabledRule::Single { is_next_line, .. } => {
                *is_next_line
            }
        }
    }

    pub fn directive_prefix(&self) -> DirectivePrefix {
        match self {
            DisabledRule::All { directive_prefix, .. }
            | DisabledRule::Single { directive_prefix, .. } => *directive_prefix,
        }
    }
}

/// Represents a single rule within a disable/enable comment directive.
///
/// Used when reporting unused disable directives or creating fixes to remove
/// specific rules from a comment.
///
/// # Example
/// ```text
/// /* eslint-disable no-debugger, no-console */
///                   ^^^^^^^^^^^              name_span (for this RuleCommentRule)
///                   no-debugger              rule_name
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RuleCommentRule {
    /// Prefix used by the directive comment (`eslint` or `oxlint`).
    pub directive_prefix: DirectivePrefix,
    /// Name of the rule (e.g., "no-debugger", "no-console")
    pub rule_name: String,
    /// Span of the rule name within the comment.
    ///
    /// For `/* eslint-disable no-debugger, no-console */`, the first
    /// `RuleCommentRule` would have a `name_span` pointing to "no-debugger".
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
            ))
            .with_kind(FixKind::Suggestion);
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
            ))
            .with_kind(FixKind::Suggestion);
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
    /// Prefix used by the directive comment (`eslint` or `oxlint`).
    pub directive_prefix: DirectivePrefix,
    /// Full outer span of the comment (including `//` or `/* */` delimiters).
    /// Used for diagnostic labels.
    pub span: Span,
    /// Span used for the fix.  Extends to cover the whole line (including leading
    /// whitespace and the trailing newline) when the comment is the only content
    /// on that line; otherwise equals `span`.
    pub fix_span: Span,
    /// Rules disabled by the comment
    pub r#type: RuleCommentType,
}

#[derive(Debug, Clone)]
pub struct DisableDirectives {
    /// All the disabled rules with their corresponding covering spans
    intervals: Lapper<u32, DisabledRule>,
    /// All comments that disable one or more specific rules
    disable_rule_comments: Box<[DisableRuleComment]>,
    /// Spans of unused enable directives
    unused_enable_comments: Box<[(DirectivePrefix, Option<String>, Span)]>,
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
        let mut has_match = false;
        for interval in self.intervals.find(span.start, span.end) {
            // Check if this rule should be disabled
            let rule_matches = match &interval.val {
                DisabledRule::All { .. } => true,
                // `rule_name` does not contain the plugin prefix.
                // - `vitest/foobar` will be just `foobar`.
                // - `@typescript-eslint/no-var-requires` will be just `no-var-requires`
                //
                // This enables matching rules across different plugins that share the same
                // rule name, such as jest<->vitest rules and eslint<->typescript rules.
                //
                // We strip the plugin prefix from the directive name and compare equality
                // rather than doing a substring match. Otherwise unrelated rules like
                // `canonical/no-re-export` would accidentally match oxlint's `export`
                // rule because `"no-re-export".contains("export")` is true.
                DisabledRule::Single { rule_name: name, .. } => {
                    let directive_rule_name =
                        name.rsplit_once('/').map_or(name.as_str(), |(_, rule)| rule);
                    directive_rule_name == rule_name
                }
            };

            if !rule_matches {
                continue;
            }

            // Check if the diagnostic span is covered by this interval
            let span_covered = if interval.val.is_next_line() {
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
            };

            if span_covered {
                self.mark_disable_directive_used(interval.val.clone());
                has_match = true;
            }
        }
        has_match
    }

    pub fn disable_rule_comments(&self) -> &[DisableRuleComment] {
        &self.disable_rule_comments
    }

    pub fn unused_enable_comments(&self) -> &[(DirectivePrefix, Option<String>, Span)] {
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

                if group_vec.is_empty() {
                    return None;
                }

                // All intervals in the group share the same comment, so they have the same fix_span.
                let fix_span = *group_vec[0].val.fix_span();

                let rules: Vec<RuleCommentRule> = group_vec
                    .iter()
                    .filter_map(|interval| {
                        if used.contains(&interval.val) {
                            return None;
                        }
                        match &interval.val {
                            DisabledRule::Single { rule_name, name_span, .. } => {
                                Some(RuleCommentRule {
                                    directive_prefix: interval.val.directive_prefix(),
                                    rule_name: rule_name.clone(),
                                    name_span: *name_span,
                                })
                            }
                            DisabledRule::All { .. } => Some(RuleCommentRule {
                                directive_prefix: interval.val.directive_prefix(),
                                rule_name: "all".to_string(),
                                name_span: *comment_span,
                            }),
                        }
                    })
                    .collect::<Vec<_>>();

                if rules.is_empty() {
                    return None;
                }

                if rules.len() == group_vec.len() {
                    return Some(DisableRuleComment {
                        directive_prefix: group_vec[0].val.directive_prefix(),
                        span: *comment_span,
                        fix_span,
                        r#type: RuleCommentType::All,
                    });
                }

                Some(DisableRuleComment {
                    directive_prefix: group_vec[0].val.directive_prefix(),
                    span: *comment_span,
                    fix_span,
                    r#type: RuleCommentType::Single(rules),
                })
            })
            .collect()
    }
}

pub struct DisableDirectivesBuilder {
    /// Which directive prefixes should be recognized.
    respect_eslint_disable_directives: bool,
    /// All the disabled rules with their corresponding covering spans
    intervals: Lapper<u32, DisabledRule>,
    /// Start of `eslint-disable` or `oxlint-disable`
    disable_all_start: Option<(u32, DirectivePrefix, Span, Span)>,
    /// Start of `eslint-disable` or `oxlint-disable` rule_name`
    disable_start_map: FxHashMap<String, (u32, DirectivePrefix, Span, Span, Span)>,
    /// All comments that disable one or more specific rules
    disable_rule_comments: Vec<DisableRuleComment>,
    /// Spans of unused enable directives
    unused_enable_comments: Vec<(DirectivePrefix, Option<String>, Span)>,
}

impl DisableDirectivesBuilder {
    pub fn new() -> Self {
        Self {
            respect_eslint_disable_directives: true,
            intervals: Lapper::new(vec![]),
            disable_all_start: None,
            disable_start_map: FxHashMap::default(),
            disable_rule_comments: vec![],
            unused_enable_comments: vec![],
        }
    }

    #[must_use]
    pub fn with_respect_eslint_disable_directives(
        mut self,
        respect_eslint_disable_directives: bool,
    ) -> Self {
        self.respect_eslint_disable_directives = respect_eslint_disable_directives;
        self
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

    /// Computes the fix span for a disable-directive comment.
    ///
    /// If the comment is the only non-whitespace content on its line, returns a
    /// span that covers the entire line (including leading whitespace and the
    /// trailing newline).  Otherwise returns the comment's full outer span
    /// (including `//` or `/* */` delimiters).
    ///
    /// This span is stored in [`DisabledRule`] so that `Fix::delete(span)`
    /// produces the correct edit without any extra post-processing in callers.
    #[expect(clippy::cast_possible_truncation)]
    pub(crate) fn compute_comment_fix_span(comment: &Comment, source_text: &str) -> Span {
        let outer_start = comment.span.start as usize;
        let outer_end = comment.span.end as usize;

        // Find the start of the current line (character after the preceding `\n`, or 0).
        let line_start = source_text[..outer_start].rfind('\n').map_or(0, |i| i + 1);

        // Find the end of the current line, including the newline character itself.
        let line_end =
            source_text[outer_end..].find('\n').map_or(source_text.len(), |i| outer_end + i + 1);

        let before_on_line = &source_text[line_start..outer_start];
        let after_on_line = source_text[outer_end..line_end].trim_end_matches(['\r', '\n']);

        if before_on_line.trim().is_empty() && after_on_line.trim().is_empty() {
            // The comment is the only meaningful content on the line – the fix
            // should delete the whole line.
            Span::new(line_start as u32, line_end as u32)
        } else {
            // There is other content on the same line – only delete the comment
            // itself (including its delimiters).
            comment.span
        }
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
        let mut unused_enable_directives: Vec<(DirectivePrefix, Option<String>, Span)> = vec![];

        for comment in comments {
            let comment_span = comment.content_span();
            // `comment.span` is the full outer span (including `//` or `/* */` delimiters).
            // It is used as the diagnostic span.
            let outer_span = comment.span;
            // Pre-compute the fix span for this comment:
            // - whole line (incl. leading whitespace + newline) if the comment is alone on the line
            // - outer comment span (incl. `//` / `/* */` delimiters) otherwise
            let comment_fix_span = Self::compute_comment_fix_span(comment, source_text);
            let text_source = comment_span.source_text(source_text);
            let text = text_source.trim_start();
            let mut rule_name_start = comment_span.start + (text_source.len() - text.len()) as u32;

            if let Some((directive_prefix, text)) = self.match_disable_directive(text) {
                rule_name_start += directive_prefix.disable_directive_name().len() as u32;
                // `eslint-disable`
                if text.trim().is_empty() {
                    if self.disable_all_start.is_none() {
                        self.disable_all_start = Some((
                            comment_span.end,
                            directive_prefix,
                            outer_span,
                            comment_fix_span,
                        ));
                    }
                    self.disable_rule_comments.push(DisableRuleComment {
                        directive_prefix,
                        span: comment_span,
                        fix_span: comment_fix_span,
                        r#type: RuleCommentType::All,
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
                                directive_prefix,
                                comment_span: outer_span,
                                fix_span: comment_fix_span,
                                is_next_line: true,
                            },
                        );
                        self.disable_rule_comments.push(DisableRuleComment {
                            directive_prefix,
                            span: comment_span,
                            fix_span: comment_fix_span,
                            r#type: RuleCommentType::All,
                        });
                    } else {
                        // `eslint-disable-next-line rule_name1, rule_name2`
                        let mut rules = vec![];
                        Self::get_rule_names(text, rule_name_start, |rule_name, name_span| {
                            self.add_interval(
                                comment_span.end,
                                stop,
                                DisabledRule::Single {
                                    directive_prefix,
                                    rule_name: rule_name.to_string(),
                                    name_span,
                                    comment_span: outer_span,
                                    fix_span: comment_fix_span,
                                    is_next_line: true,
                                },
                            );
                            rules.push(RuleCommentRule {
                                directive_prefix,
                                rule_name: rule_name.to_string(),
                                name_span,
                            });
                        });
                        self.disable_rule_comments.push(DisableRuleComment {
                            directive_prefix,
                            span: comment_span,
                            fix_span: comment_fix_span,
                            r#type: RuleCommentType::Single(rules),
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
                                directive_prefix,
                                comment_span: outer_span,
                                fix_span: comment_fix_span,
                                is_next_line: true,
                            },
                        );
                        self.disable_rule_comments.push(DisableRuleComment {
                            directive_prefix,
                            span: comment_span,
                            fix_span: comment_fix_span,
                            r#type: RuleCommentType::All,
                        });
                    } else {
                        // `eslint-disable-line rule-name1, rule-name2`
                        let mut rules = vec![];
                        Self::get_rule_names(text, rule_name_start, |rule_name, name_span| {
                            self.add_interval(
                                start,
                                stop,
                                DisabledRule::Single {
                                    directive_prefix,
                                    rule_name: rule_name.to_string(),
                                    name_span,
                                    comment_span: outer_span,
                                    fix_span: comment_fix_span,
                                    is_next_line: true,
                                },
                            );
                            rules.push(RuleCommentRule {
                                directive_prefix,
                                rule_name: rule_name.to_string(),
                                name_span,
                            });
                        });
                        self.disable_rule_comments.push(DisableRuleComment {
                            directive_prefix,
                            span: comment_span,
                            fix_span: comment_fix_span,
                            r#type: RuleCommentType::Single(rules),
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
                            directive_prefix,
                            name_span,
                            outer_span,
                            comment_fix_span,
                        ));
                        rules.push(RuleCommentRule {
                            directive_prefix,
                            rule_name: rule_name.to_string(),
                            name_span,
                        });
                    });
                    self.disable_rule_comments.push(DisableRuleComment {
                        directive_prefix,
                        span: comment_span,
                        fix_span: comment_fix_span,
                        r#type: RuleCommentType::Single(rules),
                    });
                    continue;
                }
            }

            if let Some((directive_prefix, text)) = self.match_enable_directive(text) {
                rule_name_start += directive_prefix.enable_directive_name().len() as u32;
                // `eslint-enable`
                if text.trim().is_empty() {
                    if let Some((start, disable_prefix, _, _)) = self.disable_all_start.take() {
                        self.add_interval(
                            start,
                            comment_span.start,
                            DisabledRule::All {
                                directive_prefix: disable_prefix,
                                comment_span: outer_span,
                                fix_span: comment_fix_span,
                                is_next_line: false,
                            },
                        );
                    } else {
                        // collect as unused enable (see more at note comments in beginning of this method)
                        unused_enable_directives.push((directive_prefix, None, comment_span));
                    }
                } else {
                    // `eslint-enable rule-name1, rule-name2`
                    Self::get_rule_names(text, rule_name_start, |rule_name, name_span| {
                        if let Some((start, disable_prefix, _, _, _)) =
                            self.disable_start_map.remove(rule_name)
                        {
                            self.add_interval(
                                start,
                                comment_span.start,
                                DisabledRule::Single {
                                    directive_prefix: disable_prefix,
                                    rule_name: rule_name.to_string(),
                                    name_span,
                                    comment_span: outer_span,
                                    fix_span: comment_fix_span,
                                    is_next_line: false,
                                },
                            );
                        } else {
                            // collect as unused enable (see more at note comments in beginning of this method)
                            unused_enable_directives.push((
                                directive_prefix,
                                Some(rule_name.to_string()),
                                name_span,
                            ));
                        }
                    });
                }
            }
        }

        // Lone `eslint-disable`
        if let Some((start, directive_prefix, comment_span, fix_span)) = self.disable_all_start {
            self.add_interval(
                start,
                source_len,
                DisabledRule::All { directive_prefix, comment_span, fix_span, is_next_line: false },
            );
        }

        // Lone `eslint-disable rule_name`
        let disable_start_map = self.disable_start_map.drain().collect::<Vec<_>>();
        for (rule_name, (start, directive_prefix, name_span, comment_span, fix_span)) in
            disable_start_map
        {
            self.add_interval(
                start,
                source_len,
                DisabledRule::Single {
                    directive_prefix,
                    rule_name,
                    name_span,
                    comment_span,
                    fix_span,
                    is_next_line: false,
                },
            );
        }

        // Collect unused `enable` directives
        self.unused_enable_comments = unused_enable_directives;
    }

    fn match_disable_directive<'a>(&self, text: &'a str) -> Option<(DirectivePrefix, &'a str)> {
        if let Some(rest) = text.strip_prefix(DirectivePrefix::Oxlint.disable_directive_name()) {
            Some((DirectivePrefix::Oxlint, rest))
        } else if self.respect_eslint_disable_directives {
            text.strip_prefix(DirectivePrefix::Eslint.disable_directive_name())
                .map(|rest| (DirectivePrefix::Eslint, rest))
        } else {
            None
        }
    }

    fn match_enable_directive<'a>(&self, text: &'a str) -> Option<(DirectivePrefix, &'a str)> {
        if let Some(rest) = text.strip_prefix(DirectivePrefix::Oxlint.enable_directive_name()) {
            Some((DirectivePrefix::Oxlint, rest))
        } else if self.respect_eslint_disable_directives {
            text.strip_prefix(DirectivePrefix::Eslint.enable_directive_name())
                .map(|rest| (DirectivePrefix::Eslint, rest))
        } else {
            None
        }
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
        match unused_comment.r#type {
            RuleCommentType::All => {
                diagnostics.push(
                    OxcDiagnostic::warn(unused_comment.directive_prefix.unused_disable_message())
                        .with_label(span)
                        .with_severity(severity),
                );
            }
            RuleCommentType::Single(rules) => {
                for rule in rules {
                    let rule_message =
                        rule.directive_prefix.unused_disable_rule_message(&rule.rule_name);
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
    for (directive_prefix, rule_name, span) in unused_enable {
        let message = if let Some(rule_name) = rule_name {
            directive_prefix.unused_enable_rule_message(rule_name)
        } else {
            directive_prefix.unused_enable_message()
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

    use crate::disable_directives::{
        DirectivePrefix, DisabledRule, RuleCommentRule, RuleCommentType,
    };

    use super::{DisableDirectives, DisableDirectivesBuilder};

    fn process_source<'a>(allocator: &'a Allocator, source_text: &'a str) -> Semantic<'a> {
        let source_type = SourceType::default();
        let parser_ret = Parser::new(allocator, source_text, source_type).parse();
        assert!(parser_ret.errors.is_empty());
        let semantic_ret = SemanticBuilder::new().build(allocator.alloc(parser_ret.program));
        semantic_ret.semantic
    }

    /// Replicates the `compute_comment_fix_span` logic for use in tests.
    fn comment_fix_span(comment: &Comment, source_text: &str) -> Span {
        DisableDirectivesBuilder::compute_comment_fix_span(comment, source_text)
    }

    fn directive_prefix_for_comment(comment: &Comment, source_text: &str) -> DirectivePrefix {
        let text = comment.content_span().source_text(source_text).trim_start();
        if text.starts_with("oxlint-") { DirectivePrefix::Oxlint } else { DirectivePrefix::Eslint }
    }

    fn test_directives(
        create_source_text: impl Fn(&str) -> String,
        test: impl Fn(&str, &[Comment], DisableDirectives),
    ) {
        let allocator = Allocator::default();
        for prefix in ["eslint", "oxlint"] {
            let source_text = create_source_text(prefix);
            let semantic = process_source(&allocator, &source_text);
            let comments = semantic.comments();
            let directives =
                DisableDirectivesBuilder::new().build(semantic.source_text(), comments);
            test(semantic.source_text(), comments, directives);
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
            |_source_text, comments, directives| {
                let unused = directives.unused_enable_comments();

                assert_eq!(unused.len(), 1);

                let (_unused_prefix, unused_rule_name, unused_span) = unused.first().unwrap();
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
            |_source_text, comments, directives| {
                let unused = directives.unused_enable_comments();

                assert_eq!(unused.len(), 2);

                let (
                    _unused_prefix_no_debugger,
                    unused_rule_name_no_debugger,
                    unused_span_no_debugger,
                ) = unused.first().unwrap();
                assert_eq!(unused_rule_name_no_debugger.as_deref(), Some("no-debugger"));
                assert_eq!(
                    *unused_span_no_debugger,
                    Span::sized(comments[0].content_span().start + 15, 11)
                );

                let (
                    _unused_prefix_no_console,
                    unused_rule_name_no_console,
                    unused_span_no_console,
                ) = unused.last().unwrap();
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
            |_source_text, _, directives| {
                // no mark unused

                let unused = directives.unused_enable_comments();

                assert!(unused.is_empty());
            },
        );
    }

    #[test]
    #[expect(clippy::cast_possible_truncation)]
    fn only_configured_prefixes_are_recognized() {
        let allocator = Allocator::default();
        let source_text = r"
            /* eslint-disable no-console */
            console.log('eslint');
            /* oxlint-disable no-debugger */
            debugger;
            /* eslint-enable no-console */
            /* oxlint-enable no-debugger */
        ";
        let semantic = process_source(&allocator, source_text);
        let directives = DisableDirectivesBuilder::new()
            .with_respect_eslint_disable_directives(false)
            .build(semantic.source_text(), semantic.comments());

        let console_start = source_text.find("console.log").unwrap() as u32;
        let debugger_start = source_text.find("debugger;").unwrap() as u32;

        assert!(!directives.contains("no-console", Span::sized(console_start, 11)));
        assert!(directives.contains("no-debugger", Span::sized(debugger_start, 8)));
        assert!(directives.unused_enable_comments().is_empty());
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
            |source_text, comments, directives| {
                let unused = directives.collect_unused_disable_comments();

                assert_eq!(unused.len(), 1);

                let comment = unused.first().unwrap();
                let outer_span = comments.first().unwrap().span;

                // Diagnostic span must be the original comment outer span (no line extension).
                assert_eq!(
                    comment.span, outer_span,
                    "diagnostic span must be the original comment span"
                );

                // Fix span must extend to cover the whole line when the comment is alone on it.
                let line_has_only_comment = source_text[..outer_span.start as usize]
                    .rsplit_once('\n')
                    .is_none_or(|(_, before)| before.trim().is_empty());
                if line_has_only_comment {
                    assert!(
                        comment.fix_span.start < outer_span.start
                            || comment.fix_span.end > outer_span.end,
                        "fix span should extend beyond the comment when alone on a line"
                    );
                }

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
            |source_text, comments, directives| {
                let unused = directives.collect_unused_disable_comments();

                assert_eq!(unused.len(), 1);

                let comment = unused.first().unwrap();
                let outer_span = comments.first().unwrap().span;

                // Diagnostic span must be the original comment outer span (no line extension).
                assert_eq!(
                    comment.span, outer_span,
                    "diagnostic span must be the original comment span"
                );

                // Fix span must extend to cover the whole line when the comment is alone on it.
                let line_has_only_comment = source_text[..outer_span.start as usize]
                    .rsplit_once('\n')
                    .is_none_or(|(_, before)| before.trim().is_empty());
                if line_has_only_comment {
                    assert!(
                        comment.fix_span.start < outer_span.start
                            || comment.fix_span.end > outer_span.end,
                        "fix span should extend beyond the comment when alone on a line"
                    );
                }

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
            |source_text, comments, directives| {
                // Mark each directive as used by constructing the matching `DisabledRule` with the
                // correct spans (same as what `build_impl` stores).
                let fix_span_0 = comment_fix_span(&comments[0], source_text);
                let fix_span_1 = comment_fix_span(&comments[1], source_text);

                directives.mark_disable_directive_used(DisabledRule::Single {
                    directive_prefix: directive_prefix_for_comment(&comments[0], source_text),
                    rule_name: "no-console".to_string(),
                    name_span: Span::sized(comments[0].content_span().start + 16, 10),
                    comment_span: comments[0].span,
                    fix_span: fix_span_0,
                    is_next_line: false,
                });
                directives.mark_disable_directive_used(DisabledRule::Single {
                    directive_prefix: directive_prefix_for_comment(&comments[1], source_text),
                    rule_name: "no-debugger".to_string(),
                    name_span: Span::sized(comments[1].content_span().start + 16, 11),
                    comment_span: comments[1].span,
                    fix_span: fix_span_1,
                    is_next_line: false,
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
        // comment_span now represents the fix span (outer span including `//`, whole line if alone).
        // Since this string has no newline, the fix span = outer span = Span::new(0, len).
        let comment_span = Span::new(0, source_text.len() as u32);

        let max_params_fix = RuleCommentRule {
            directive_prefix: DirectivePrefix::Eslint,
            rule_name: "max-params".to_string(),
            name_span: Span::sized(28, 10),
        }
        .create_fix(source_text, comment_span);

        assert_eq!(&source_text[28..38], "max-params");
        assert_eq!(max_params_fix.span, Span::sized(28, 11)); // max-params is 10 + 1 for the comma

        let no_console_fix = RuleCommentRule {
            directive_prefix: DirectivePrefix::Eslint,
            rule_name: "no-console".to_string(),
            name_span: Span::sized(40, 10),
        }
        .create_fix(source_text, comment_span);

        assert_eq!(&source_text[40..50], "no-console");
        assert_eq!(no_console_fix.span, Span::sized(38, 12)); // no-console is 10 + 2 for the comma before and the space

        let no_debugger_fix = RuleCommentRule {
            directive_prefix: DirectivePrefix::Eslint,
            rule_name: "no-debugger".to_string(),
            name_span: Span::sized(52, 11),
        }
        .create_fix(source_text, comment_span);

        assert_eq!(&source_text[52..63], "no-debugger");
        assert_eq!(no_debugger_fix.span, Span::sized(50, 13)); // no-debugger is 11 + 2 for the comma before and the space
    }

    #[test]
    #[should_panic(
        expected = "A `RuleCommentRule` should have a comma, because only one rule should be RuleCommentType::All"
    )]
    #[expect(clippy::cast_possible_truncation)] // for `as u32`
    fn test_rule_comment_rule_create_fix_panic() {
        // This test is expected to panic because it is a standalone rule.
        // Standalone rules should be `RuleCommentType::All`.
        let source_text = "// eslint-disable-next-line max-params";
        let comment_span = Span::new(0, source_text.len() as u32);

        RuleCommentRule {
            directive_prefix: DirectivePrefix::Eslint,
            rule_name: "max-params".to_string(),
            name_span: Span::sized(28, 10),
        }
        .create_fix(source_text, comment_span);
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

    /// Helper: apply a `Fix::delete` on `span` to `source_text` and return the resulting string.
    fn apply_delete(source_text: &str, span: Span) -> String {
        let start = span.start as usize;
        let end = span.end as usize;
        format!("{}{}", &source_text[..start], &source_text[end..])
    }

    #[test]
    fn fix_span_line_comment_alone_on_line() {
        // A `// eslint-disable` on its own line.
        // - diagnostic `span` = outer comment span (no line extension)
        // - `fix_span` = the whole line (including the newline)
        let source_text = "const x = 1;\n// eslint-disable no-console\nconsole.log(x);\n";
        let allocator = Allocator::default();
        let semantic = process_source(&allocator, source_text);
        let directives =
            DisableDirectivesBuilder::new().build(semantic.source_text(), semantic.comments());

        let unused = directives.collect_unused_disable_comments();
        assert_eq!(unused.len(), 1);

        // span must be the outer comment span only (no line extension).
        let comment_outer_span = semantic.comments()[0].span;
        assert_eq!(
            unused[0].span, comment_outer_span,
            "diagnostic span must be the outer comment span"
        );

        // fix_span must delete the whole line.
        let result = apply_delete(source_text, unused[0].fix_span);
        assert_eq!(result, "const x = 1;\nconsole.log(x);\n");
    }

    #[test]
    fn fix_span_block_comment_alone_on_line() {
        // A `/* eslint-disable */` on its own line.
        // - diagnostic `span` = outer comment span (no line extension)
        // - `fix_span` = the whole line (including the newline)
        let source_text = "const x = 1;\n/* eslint-disable no-console */\nconsole.log(x);\n";
        let allocator = Allocator::default();
        let semantic = process_source(&allocator, source_text);
        let directives =
            DisableDirectivesBuilder::new().build(semantic.source_text(), semantic.comments());

        let unused = directives.collect_unused_disable_comments();
        assert_eq!(unused.len(), 1);

        // span must be the outer comment span only.
        let comment_outer_span = semantic.comments()[0].span;
        assert_eq!(
            unused[0].span, comment_outer_span,
            "diagnostic span must be the outer comment span"
        );

        // fix_span must delete the whole line.
        let result = apply_delete(source_text, unused[0].fix_span);
        assert_eq!(result, "const x = 1;\nconsole.log(x);\n");
    }

    #[test]
    fn fix_span_comment_on_line_with_code() {
        // An `eslint-disable-line` style comment after code.
        // Both `span` and `fix_span` should cover just the comment (no line extension since
        // there is code before it on the same line).
        let source_text = "const x = 1; // eslint-disable-line no-unused-vars\n";
        let allocator = Allocator::default();
        let semantic = process_source(&allocator, source_text);
        let directives =
            DisableDirectivesBuilder::new().build(semantic.source_text(), semantic.comments());

        let unused = directives.collect_unused_disable_comments();
        assert_eq!(unused.len(), 1);

        // fix_span deletes only the comment; code on the same line is preserved.
        let result = apply_delete(source_text, unused[0].fix_span);
        assert!(result.contains("const x = 1;"), "code on the line must not be removed");
        assert!(!result.contains("eslint-disable"), "directive must be removed");
        // Must not leave `//` behind.
        assert!(!result.contains("//"), "must not leave an empty `//` behind");
    }

    #[test]
    fn fix_span_indented_line_comment() {
        // An indented directive on its own line.
        // - `span` = outer comment span (no indentation)
        // - `fix_span` = whole line including indentation and newline
        let source_text =
            "function f() {\n    // eslint-disable-next-line no-console\n    console.log();\n}\n";
        let allocator = Allocator::default();
        let semantic = process_source(&allocator, source_text);
        let directives =
            DisableDirectivesBuilder::new().build(semantic.source_text(), semantic.comments());

        let unused = directives.collect_unused_disable_comments();
        assert_eq!(unused.len(), 1);

        // span must be the outer comment span (starts at `//`, not at the indentation).
        let comment_outer_span = semantic.comments()[0].span;
        assert_eq!(
            unused[0].span, comment_outer_span,
            "diagnostic span must be the outer comment span"
        );

        // fix_span must delete the whole line (including the 4-space indentation).
        let result = apply_delete(source_text, unused[0].fix_span);
        assert_eq!(result, "function f() {\n    console.log();\n}\n");
    }

    #[test]
    fn directive_rule_name_is_matched_on_full_rule_name_not_substring() {
        // Regression test: a directive for an unrelated plugin rule whose name happens
        // to contain an oxlint rule name as a substring (e.g. `canonical/no-re-export`
        // vs the oxlint `export` rule) must NOT suppress the oxlint rule.
        let source_text = r"
            // eslint-disable-next-line canonical/no-re-export
            export * from './foo';
        ";
        let allocator = Allocator::default();
        let semantic = process_source(&allocator, source_text);
        let directives =
            DisableDirectivesBuilder::new().build(semantic.source_text(), semantic.comments());

        #[expect(clippy::cast_possible_truncation)]
        let export_start = source_text.find("export *").unwrap() as u32;
        let export_span = Span::sized(export_start, 21);
        assert!(
            !directives.contains("export", export_span),
            "`canonical/no-re-export` directive must not suppress the `export` rule"
        );

        // But a directive that names the rule exactly (with or without a plugin prefix)
        // still matches.
        let source_text_exact = r"
            // eslint-disable-next-line import/export
            export * from './foo';
        ";
        let semantic = process_source(&allocator, source_text_exact);
        let directives =
            DisableDirectivesBuilder::new().build(semantic.source_text(), semantic.comments());
        #[expect(clippy::cast_possible_truncation)]
        let export_start = source_text_exact.find("export *").unwrap() as u32;
        let export_span = Span::sized(export_start, 21);
        assert!(
            directives.contains("export", export_span),
            "`import/export` directive must suppress the `export` rule"
        );
    }
}
