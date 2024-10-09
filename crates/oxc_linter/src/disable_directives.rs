use oxc_ast::Trivias;
use oxc_span::Span;
use rust_lapper::{Interval, Lapper};
use rustc_hash::FxHashMap;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum DisabledRule<'a> {
    All,
    Single(&'a str),
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
}

impl<'a> DisableDirectives<'a> {
    pub fn contains(&self, rule_name: &'static str, span: Span) -> bool {
        self.intervals.find(span.start, span.end).any(|interval| {
            interval.val == DisabledRule::All
                // Our rule name currently does not contain the prefix.
                // For example, this will match `@typescript-eslint/no-var-requires` given
                // our rule_name is `no-var-requires`.
                || matches!(interval.val, DisabledRule::Single(name) if name.contains(rule_name))
        })
    }

    pub fn disable_all_comments(&self) -> &[Span] {
        &self.disable_all_comments
    }

    pub fn disable_rule_comments(&self) -> &[DisableRuleComment<'a>] {
        &self.disable_rule_comments
    }
}

pub struct DisableDirectivesBuilder<'a> {
    source_text: &'a str,
    trivias: Trivias,
    /// All the disabled rules with their corresponding covering spans
    intervals: Lapper<u32, DisabledRule<'a>>,
    /// Start of `eslint-disable` or `oxlint-disable`
    disable_all_start: Option<u32>,
    /// Start of `eslint-disable` or `oxlint-disable` rule_name`
    disable_start_map: FxHashMap<&'a str, u32>,
    /// Spans of comments that disable all rules
    disable_all_comments: Vec<Span>,
    /// All comments that disable one or more specific rules
    disable_rule_comments: Vec<DisableRuleComment<'a>>,
}

impl<'a> DisableDirectivesBuilder<'a> {
    pub fn new(source_text: &'a str, trivias: Trivias) -> Self {
        Self {
            source_text,
            trivias,
            intervals: Lapper::new(vec![]),
            disable_all_start: None,
            disable_start_map: FxHashMap::default(),
            disable_all_comments: vec![],
            disable_rule_comments: vec![],
        }
    }

    pub fn build(mut self) -> DisableDirectives<'a> {
        self.build_impl();
        DisableDirectives {
            intervals: self.intervals,
            disable_all_comments: self.disable_all_comments.into_boxed_slice(),
            disable_rule_comments: self.disable_rule_comments.into_boxed_slice(),
        }
    }

    fn add_interval(&mut self, start: u32, stop: u32, val: DisabledRule<'a>) {
        self.intervals.insert(Interval { start, stop, val });
    }

    #[allow(clippy::cast_possible_truncation)] // for `as u32`
    fn build_impl(&mut self) {
        let source_len = self.source_text.len() as u32;
        // This algorithm iterates through the comments and builds all intervals
        // for matching disable and enable pairs.
        // Wrongly ordered matching pairs are not taken into consideration.
        for comment in self.trivias.clone().comments() {
            let text = comment.span.source_text(self.source_text);
            let text = text.trim_start();

            if let Some(text) =
                text.strip_prefix("eslint-disable").or_else(|| text.strip_prefix("oxlint-disable"))
            {
                // `eslint-disable`
                if text.trim().is_empty() {
                    if self.disable_all_start.is_none() {
                        self.disable_all_start = Some(comment.span.end);
                    }
                    self.disable_all_comments.push(comment.span);
                    continue;
                }
                // `eslint-disable-next-line`
                else if let Some(text) = text.strip_prefix("-next-line") {
                    // Get the span up to the next new line
                    let stop = self.source_text[comment.span.end as usize..]
                        .lines()
                        .take(2)
                        .fold(comment.span.end, |acc, line| acc + line.len() as u32);
                    if text.trim().is_empty() {
                        self.add_interval(comment.span.end, stop, DisabledRule::All);
                        self.disable_all_comments.push(comment.span);
                    } else {
                        // `eslint-disable-next-line rule_name1, rule_name2`
                        let mut rules = vec![];
                        Self::get_rule_names(text, |rule_name| {
                            self.add_interval(
                                comment.span.end,
                                stop,
                                DisabledRule::Single(rule_name),
                            );
                            rules.push(rule_name);
                        });
                        self.disable_rule_comments
                            .push(DisableRuleComment { span: comment.span, rules });
                    }
                    continue;
                }
                // `eslint-disable-line`
                else if let Some(text) = text.strip_prefix("-line") {
                    // Get the span between the preceding newline to this comment
                    let start = self.source_text[..=comment.span.start as usize]
                        .lines()
                        .next_back()
                        .map_or(0, |line| comment.span.start - (line.len() as u32 - 1));
                    let stop = comment.span.start;

                    // `eslint-disable-line`
                    if text.trim().is_empty() {
                        self.add_interval(start, stop, DisabledRule::All);
                        self.disable_all_comments.push(comment.span);
                    } else {
                        // `eslint-disable-line rule-name1, rule-name2`
                        let mut rules = vec![];
                        Self::get_rule_names(text, |rule_name| {
                            self.add_interval(start, stop, DisabledRule::Single(rule_name));
                            rules.push(rule_name);
                        });
                        self.disable_rule_comments
                            .push(DisableRuleComment { span: comment.span, rules });
                    }
                    continue;
                }
                // Remaining text should start with a space, else it's probably a typo of the correct syntax.
                // Like `eslint-disable-lext-nine` where `text` is `-lext-nine`, or directive is `eslint-disablefoo`
                else if text.starts_with(' ') {
                    // `eslint-disable rule-name1, rule-name2`
                    let mut rules = vec![];
                    Self::get_rule_names(text, |rule_name| {
                        self.disable_start_map.entry(rule_name).or_insert(comment.span.end);
                        rules.push(rule_name);
                    });
                    self.disable_rule_comments
                        .push(DisableRuleComment { span: comment.span, rules });
                    continue;
                }
            }

            if let Some(text) =
                text.strip_prefix("eslint-enable").or_else(|| text.strip_prefix("oxlint-enable"))
            {
                // `eslint-enable`
                if text.trim().is_empty() {
                    if let Some(start) = self.disable_all_start.take() {
                        self.add_interval(start, comment.span.start, DisabledRule::All);
                    }
                } else {
                    // `eslint-enable rule-name1, rule-name2`
                    Self::get_rule_names(text, |rule_name| {
                        if let Some(start) = self.disable_start_map.remove(rule_name) {
                            self.add_interval(
                                start,
                                comment.span.start,
                                DisabledRule::Single(rule_name),
                            );
                        }
                    });
                }
                continue;
            }
        }

        // Lone `eslint-disable`
        if let Some(start) = self.disable_all_start {
            self.add_interval(start, source_len, DisabledRule::All);
        }

        // Lone `eslint-disable rule_name`
        let disable_start_map = self.disable_start_map.drain().collect::<Vec<_>>();
        for (rule_name, start) in disable_start_map {
            self.add_interval(start, source_len, DisabledRule::Single(rule_name));
        }
    }

    fn get_rule_names<F: FnMut(&'a str)>(text: &'a str, cb: F) {
        if let Some(text) = text.split_terminator("--").next() {
            text.split(',').map(str::trim).for_each(cb);
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

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
            ")
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

        Tester::new("no-debugger", pass, fail).intentionally_allow_no_fix_tests().test();
    }
}
