use std::cell::RefCell;

use oxc_ast::Comment;
use oxc_span::Span;
use rust_lapper::{Interval, Lapper};
use rustc_hash::FxHashMap;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum DisabledRule<'a> {
    All { comment_span: Span },
    Single { rule_name: &'a str, comment_span: Span },
}

/// A comment which disables one or more specific rules
#[derive(Debug)]
pub struct DisableRuleComment<'a> {
    /// Span of the comment
    pub span: Span,
    /// Rules disabled by the comment
    pub rules: Vec<&'a str>,
}

pub struct DisableDirectives<'a> {
    /// All the disabled rules with their corresponding covering spans
    intervals: Lapper<u32, DisabledRule<'a>>,
    /// Spans of comments that disable all rules
    disable_all_comments: Box<[Span]>,
    /// All comments that disable one or more specific rules
    disable_rule_comments: Box<[DisableRuleComment<'a>]>,
    /// Spans of unused enable directives
    unused_enable_comments: Box<[(Option<&'a str>, Span)]>,
    /// Spans of used enable directives, to filter out unused
    used_disable_comments: RefCell<Vec<DisabledRule<'a>>>,
}

impl<'a> DisableDirectives<'a> {
    fn mark_disable_directive_used(&self, disable_directive: DisabledRule<'a>) {
        self.used_disable_comments.borrow_mut().push(disable_directive);
    }

    pub fn contains(&self, rule_name: &'static str, span: Span) -> bool {
        let matched_intervals = self
            .intervals
            .find(span.start, span.end)
            .filter(|interval| {
                match interval.val {
                    DisabledRule::All { .. } => true,
                    // Our rule name currently does not contain the prefix.
                    // For example, this will match `@typescript-eslint/no-var-requires` given
                    // our rule_name is `no-var-requires`.
                    DisabledRule::Single { rule_name: name, .. } => name.contains(rule_name),
                }
            })
            .map(|interval| interval.val)
            .collect::<Vec<DisabledRule<'a>>>();

        for disable in &matched_intervals {
            self.mark_disable_directive_used(*disable);
        }

        !matched_intervals.is_empty()
    }

    pub fn disable_all_comments(&self) -> &[Span] {
        &self.disable_all_comments
    }

    pub fn disable_rule_comments(&self) -> &[DisableRuleComment<'a>] {
        &self.disable_rule_comments
    }

    pub fn unused_enable_comments(&self) -> &[(Option<&'a str>, Span)] {
        &self.unused_enable_comments
    }

    pub fn collect_unused_disable_comments(&self) -> Vec<(Option<&'a str>, Span)> {
        let used = self.used_disable_comments.borrow();

        self.intervals
            .iter()
            .filter(|interval| !used.contains(&interval.val))
            .map(|interval| match interval.val {
                DisabledRule::All { comment_span } => (None, comment_span),
                DisabledRule::Single { comment_span, rule_name } => (Some(rule_name), comment_span),
            })
            .collect()
    }
}

pub struct DisableDirectivesBuilder<'a> {
    /// All the disabled rules with their corresponding covering spans
    intervals: Lapper<u32, DisabledRule<'a>>,
    /// Start of `eslint-disable` or `oxlint-disable`
    disable_all_start: Option<(u32, Span)>,
    /// Start of `eslint-disable` or `oxlint-disable` rule_name`
    disable_start_map: FxHashMap<&'a str, (u32, Span)>,
    /// Spans of comments that disable all rules
    disable_all_comments: Vec<Span>,
    /// All comments that disable one or more specific rules
    disable_rule_comments: Vec<DisableRuleComment<'a>>,
    /// Spans of unused enable directives
    unused_enable_comments: Vec<(Option<&'a str>, Span)>,
}

impl<'a> DisableDirectivesBuilder<'a> {
    pub fn new() -> Self {
        Self {
            intervals: Lapper::new(vec![]),
            disable_all_start: None,
            disable_start_map: FxHashMap::default(),
            disable_all_comments: vec![],
            disable_rule_comments: vec![],
            unused_enable_comments: vec![],
        }
    }

    pub fn build(mut self, source_text: &'a str, comments: &[Comment]) -> DisableDirectives<'a> {
        self.build_impl(source_text, comments);

        DisableDirectives {
            intervals: self.intervals,
            disable_all_comments: self.disable_all_comments.into_boxed_slice(),
            disable_rule_comments: self.disable_rule_comments.into_boxed_slice(),
            unused_enable_comments: self.unused_enable_comments.into_boxed_slice(),
            used_disable_comments: RefCell::new(Vec::new()),
        }
    }

    fn add_interval(&mut self, start: u32, stop: u32, val: DisabledRule<'a>) {
        self.intervals.insert(Interval { start, stop, val });
    }

    #[expect(clippy::cast_possible_truncation)] // for `as u32`
    fn build_impl(&mut self, source_text: &'a str, comments: &[Comment]) {
        let source_len = source_text.len() as u32;
        // This algorithm iterates through the comments and builds all intervals
        // for matching disable and enable pairs.
        // Wrongly ordered matching pairs are not taken into consideration.

        // NOTE: oxlint apply directive's logic not exactly same to eslint
        // only `disable-all` & `enable-all`, or `disable-rule` & `enable-rule` will add to intervals for disable directives
        // `disable-all` & `enable-rule` -> lone `disable-all` (enable-rule find not disable-rule before itself)
        // https://github.com/eslint/eslint/blob/f67d5e875324a9d899598b11807a9c7624021432/lib/linter/apply-disable-directives.js#L308

        // enable directive keep the same logic for checking unused
        let mut unused_enable_directives: Vec<(Option<&str>, Span)> = vec![];

        for comment in comments {
            let comment_span = comment.content_span();
            let text = comment_span.source_text(source_text);
            let text = text.trim_start();

            if let Some(text) =
                text.strip_prefix("eslint-disable").or_else(|| text.strip_prefix("oxlint-disable"))
            {
                // `eslint-disable`
                if text.trim().is_empty() {
                    if self.disable_all_start.is_none() {
                        self.disable_all_start = Some((comment_span.end, comment_span));
                    }
                    self.disable_all_comments.push(comment_span);
                    continue;
                }
                // `eslint-disable-next-line`
                else if let Some(text) = text.strip_prefix("-next-line") {
                    // Get the span up to the next new line
                    let mut stop = comment_span.end;
                    let mut lines_after_comment_end =
                        source_text[comment_span.end as usize..].split_inclusive('\n');

                    if let Some(rest_of_line) = lines_after_comment_end.next() {
                        stop += rest_of_line.len() as u32;
                    }

                    if let Some(next_line) = lines_after_comment_end.next() {
                        let next_line = next_line.strip_suffix('\n').unwrap_or(next_line);
                        let next_line = next_line.strip_suffix('\r').unwrap_or(next_line);
                        stop += next_line.len() as u32;
                    }

                    if text.trim().is_empty() {
                        self.add_interval(
                            comment_span.end,
                            stop,
                            DisabledRule::All { comment_span },
                        );
                        self.disable_all_comments.push(comment_span);
                    } else {
                        // `eslint-disable-next-line rule_name1, rule_name2`
                        let mut rules = vec![];
                        Self::get_rule_names(text, |rule_name| {
                            self.add_interval(
                                comment_span.end,
                                stop,
                                DisabledRule::Single { rule_name, comment_span },
                            );
                            rules.push(rule_name);
                        });
                        self.disable_rule_comments
                            .push(DisableRuleComment { span: comment_span, rules });
                    }
                    continue;
                }
                // `eslint-disable-line`
                else if let Some(text) = text.strip_prefix("-line") {
                    // Get the span between the preceding newline to this comment
                    let start = source_text[..comment_span.start as usize]
                        .lines()
                        .next_back()
                        .map_or(0, |line| comment_span.start - line.len() as u32);
                    let stop = comment_span.start;

                    // `eslint-disable-line`
                    if text.trim().is_empty() {
                        self.add_interval(start, stop, DisabledRule::All { comment_span });
                        self.disable_all_comments.push(comment_span);
                    } else {
                        // `eslint-disable-line rule-name1, rule-name2`
                        let mut rules = vec![];
                        Self::get_rule_names(text, |rule_name| {
                            self.add_interval(
                                start,
                                stop,
                                DisabledRule::Single { rule_name, comment_span },
                            );
                            rules.push(rule_name);
                        });
                        self.disable_rule_comments
                            .push(DisableRuleComment { span: comment_span, rules });
                    }
                    continue;
                }
                // Remaining text should start with a space, else it's probably a typo of the correct syntax.
                // Like `eslint-disable-lext-nine` where `text` is `-lext-nine`, or directive is `eslint-disablefoo`
                else if text.starts_with(' ') {
                    // `eslint-disable rule-name1, rule-name2`
                    let mut rules = vec![];
                    Self::get_rule_names(text, |rule_name| {
                        self.disable_start_map
                            .entry(rule_name)
                            .or_insert((comment_span.end, comment_span));
                        rules.push(rule_name);
                    });
                    self.disable_rule_comments
                        .push(DisableRuleComment { span: comment_span, rules });
                    continue;
                }
            }

            if let Some(text) =
                text.strip_prefix("eslint-enable").or_else(|| text.strip_prefix("oxlint-enable"))
            {
                // `eslint-enable`
                if text.trim().is_empty() {
                    if let Some((start, _)) = self.disable_all_start.take() {
                        self.add_interval(
                            start,
                            comment_span.start,
                            DisabledRule::All { comment_span },
                        );
                    } else {
                        // collect as unused enable (see more at note comments in beginning of this method)
                        unused_enable_directives.push((None, comment_span));
                    }
                } else {
                    // `eslint-enable rule-name1, rule-name2`
                    Self::get_rule_names(text, |rule_name| {
                        if let Some((start, _)) = self.disable_start_map.remove(rule_name) {
                            self.add_interval(
                                start,
                                comment_span.start,
                                DisabledRule::Single { rule_name, comment_span },
                            );
                        } else {
                            // collect as unused enable (see more at note comments in beginning of this method)
                            unused_enable_directives.push((Some(rule_name), comment_span));
                        }
                    });
                }
                continue;
            }
        }

        // Lone `eslint-disable`
        if let Some((start, comment_span)) = self.disable_all_start {
            self.add_interval(start, source_len, DisabledRule::All { comment_span });
        }

        // Lone `eslint-disable rule_name`
        let disable_start_map = self.disable_start_map.drain().collect::<Vec<_>>();
        for (rule_name, (start, comment_span)) in disable_start_map {
            self.add_interval(start, source_len, DisabledRule::Single { rule_name, comment_span });
        }

        // Collect unused `enable` directives
        self.unused_enable_comments = unused_enable_directives;
    }

    fn get_rule_names<F: FnMut(&'a str)>(text: &'a str, cb: F) {
        if let Some(text) = text.split_terminator("--").next() {
            text.split(',').map(str::trim).for_each(cb);
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

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_ast::Comment;
    use oxc_parser::Parser;
    use oxc_semantic::{Semantic, SemanticBuilder};
    use oxc_span::SourceType;

    use super::{DisableDirectives, DisableDirectivesBuilder, DisabledRule};

    fn process_source<'a>(allocator: &'a Allocator, source_text: &'a str) -> Semantic<'a> {
        let source_type = SourceType::default();
        let parser_ret = Parser::new(allocator, source_text, source_type).parse();
        let semantic_ret = SemanticBuilder::new().build(allocator.alloc(parser_ret.program));
        semantic_ret.semantic
    }

    fn test_directives(
        create_source_text: impl Fn(&str) -> String,
        test: impl Fn(&[Comment], DisableDirectives),
    ) {
        let allocator = Allocator::default();
        for prefix in ["eslint", "oxlint"] {
            let source_text = create_source_text(prefix);
            let semantic = process_source(&allocator, &source_text);
            let comments = semantic.comments();
            let directives =
                DisableDirectivesBuilder::new().build(semantic.source_text(), comments);
            test(comments, directives);
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
            |comments, directives| {
                let unused = directives.unused_enable_comments();

                assert_eq!(unused.len(), 1);

                let (unused_rule_name, unused_span) = unused.first().unwrap();
                let comment_span = comments.first().unwrap().content_span();
                assert_eq!(*unused_rule_name, None);
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
            |comments, directives| {
                let unused = directives.unused_enable_comments();

                assert_eq!(unused.len(), 2);

                let (unused_rule_name_no_debugger, unused_span_no_debugger) =
                    unused.first().unwrap();
                let comment_span_no_debugger = comments.first().unwrap().content_span();
                assert_eq!(*unused_rule_name_no_debugger, Some("no-debugger"));
                assert_eq!(*unused_span_no_debugger, comment_span_no_debugger);

                let (unused_rule_name_no_console, unused_span_no_console) = unused.last().unwrap();
                let comment_span_no_console = comments.last().unwrap().content_span();
                assert_eq!(*unused_rule_name_no_console, Some("no-console"));
                assert_eq!(*unused_span_no_console, comment_span_no_console);
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
            |_, directives| {
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
            |comments, directives| {
                // no mark unused

                let unused = directives.collect_unused_disable_comments();

                assert_eq!(unused.len(), 1);

                let (unused_rule_name, unused_span) = unused.first().unwrap();
                let comment_span = comments.first().unwrap().content_span();
                assert_eq!(*unused_rule_name, None);
                assert_eq!(*unused_span, comment_span);
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
                    console.log();
                    "
                )
            },
            |comments, directives| {
                // no mark unused

                let unused = directives.collect_unused_disable_comments();

                assert_eq!(unused.len(), 2);

                let (unused_rule_name_no_debugger, unused_span_no_debugger) =
                    unused.first().unwrap();
                let comment_span_no_debugger = comments.first().unwrap().content_span();
                assert_eq!(*unused_rule_name_no_debugger, Some("no-debugger"));
                assert_eq!(*unused_span_no_debugger, comment_span_no_debugger);

                let (unused_rule_name_no_console, unused_span_no_console) = unused.last().unwrap();
                let comment_span_no_console = comments.last().unwrap().content_span();
                assert_eq!(*unused_rule_name_no_console, Some("no-console"));
                assert_eq!(*unused_span_no_console, comment_span_no_console);
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
            |comments, directives| {
                directives.mark_disable_directive_used(DisabledRule::Single {
                    rule_name: "no-console",
                    comment_span: comments[0].content_span(),
                });
                directives.mark_disable_directive_used(DisabledRule::Single {
                    rule_name: "no-debugger",
                    comment_span: comments[1].content_span(),
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
}
