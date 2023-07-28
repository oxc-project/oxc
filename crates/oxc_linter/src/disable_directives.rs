use oxc_ast::Trivias;
use oxc_span::Span;
use rust_lapper::{Interval, Lapper};
use rustc_hash::FxHashMap;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum DisabledRule<'a> {
    All,
    Single(&'a str),
}

pub struct DisableDirectives<'a> {
    /// All the disabled rules with their corresponding covering spans
    intervals: Lapper<u32, DisabledRule<'a>>,
}

impl<'a> DisableDirectives<'a> {
    pub fn contains(&self, rule_name: &'static str, start: u32) -> bool {
        self.intervals.find(start, start + 1).any(|interval| {
            interval.val == DisabledRule::All
                // Our rule name currently does not contain the prefix.
                // For example, this will match `@typescript-eslint/no-var-requires` given
                // our rule_name is `no-var-requires`.
                || matches!(interval.val, DisabledRule::Single(name) if name.contains(rule_name))
        })
    }
}

pub struct DisableDirectivesBuilder<'a, 'b> {
    source_text: &'a str,
    trivias: &'b Trivias,
    /// All the disabled rules with their corresponding covering spans
    intervals: Lapper<u32, DisabledRule<'a>>,
    /// Start of `eslint-disable`
    disable_all_start: Option<u32>,
    /// Start of `eslint-disable rule_name`
    disable_start_map: FxHashMap<&'a str, u32>,
}

impl<'a, 'b> DisableDirectivesBuilder<'a, 'b> {
    pub fn new(source_text: &'a str, trivias: &'b Trivias) -> Self {
        Self {
            source_text,
            trivias,
            intervals: Lapper::new(vec![]),
            disable_all_start: None,
            disable_start_map: FxHashMap::default(),
        }
    }

    pub fn build(mut self) -> DisableDirectives<'a> {
        self.build_impl();
        DisableDirectives { intervals: self.intervals }
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
        for (start, comment) in self.trivias.comments() {
            let span = Span::new(*start, comment.end());
            let text = span.source_text(self.source_text);
            let text = text.trim_start();

            if let Some(text) = text.strip_prefix("eslint-disable") {
                // `eslint-disable`
                if text.trim().is_empty() {
                    self.disable_all_start = Some(span.end);
                    continue;
                }

                // `eslint-disable-next-line`
                if let Some(text) = text.strip_prefix("-next-line") {
                    // Get the span up to the next new line
                    let stop = self.source_text[span.end as usize..]
                        .lines()
                        .take(if comment.is_single_line() { 1 } else { 2 })
                        .map(|line| span.end + line.len() as u32)
                        .sum();
                    if text.trim().is_empty() {
                        self.add_interval(span.end, stop, DisabledRule::All);
                    } else {
                        // `eslint-disable-next-line rule_name1, rule_name2`
                        Self::get_rule_names(text, |rule_name| {
                            self.add_interval(span.end, stop, DisabledRule::Single(rule_name));
                        });
                    }
                    continue;
                }

                // `eslint-disable-line`
                if let Some(text) = text.strip_prefix("-line") {
                    // Get the span between the preceding newline to this comment
                    let start = self.source_text[..=span.start as usize]
                        .lines()
                        .next_back()
                        .map_or(0, |line| span.start - (line.len() as u32 - 1));
                    let stop = span.start;

                    // `eslint-disable-line`
                    if text.trim().is_empty() {
                        self.add_interval(start, stop, DisabledRule::All);
                    } else {
                        // `eslint-disable-line rule-name1, rule-name2`
                        Self::get_rule_names(text, |rule_name| {
                            self.add_interval(start, stop, DisabledRule::Single(rule_name));
                        });
                    }
                    continue;
                }

                // `eslint-disable rule-name1, rule-name2`
                Self::get_rule_names(text, |rule_name| {
                    self.disable_start_map.insert(rule_name, span.end);
                });

                continue;
            }

            if let Some(text) = text.strip_prefix("eslint-enable") {
                // `eslint-enable`
                if text.trim().is_empty() {
                    if let Some(start) = self.disable_all_start.take() {
                        self.add_interval(start, span.start, DisabledRule::All);
                    }
                } else {
                    // `eslint-enable rule-name1, rule-name2`
                    Self::get_rule_names(text, |rule_name| {
                        if let Some(start) = self.disable_start_map.remove(rule_name) {
                            self.add_interval(start, span.start, DisabledRule::Single(rule_name));
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

    // [Disabling Rules](https://eslint.org/docs/latest/use/configure/rules#disabling-rules)
    // Using configuration comments
    let pass = vec![
        // To disable rule warnings in a part of a file, use block comments in the following format:
        "
        /* eslint-disable */
            debugger;
        /* eslint-enable */
        ",
        // You can also disable or enable warnings for specific rules:
        "
        /* eslint-disable no-debugger, no-console */
            debugger;
        /* eslint-enable no-debugger, no-console */
        ",
        // To disable rule warnings in an entire file, put a /* eslint-disable */ block comment at the top of the file:
        "
        /* eslint-disable */
            debugger;
        ",
        // You can also disable or enable specific rules for an entire file:
        "
        /* eslint-disable no-debugger */
            debugger;
        ",
        // To ensure that a rule is never applied (regardless of any future enable/disable lines):
        // This is not supported.
        // "
        // /* eslint no-debugger: \"off\" */
        //     debugger;
        // ",
        // To disable all rules on a specific line, use a line or block comment in one of the following formats:
        "debugger; // eslint-disable-line
            debugger; // eslint-disable-line

            // eslint-disable-next-line
            debugger;

            /* eslint-disable-next-line */
            debugger;

            debugger; /* eslint-disable-line */
        ",
        // To disable a specific rule on a specific line:
        "
            debugger; // eslint-disable-line no-debugger

            // eslint-disable-next-line no-debugger
            debugger;

            debugger; /* eslint-disable-line no-debugger */

            /* eslint-disable-next-line no-debugger */
            debugger;
        ",
        // To disable multiple rules on a specific line:
        "
            debugger; // eslint-disable-line no-debugger, quotes, semi

            // eslint-disable-next-line no-debugger, quotes, semi
            debugger;

            debugger; /* eslint-disable-line no-debugger, quotes, semi */

            /* eslint-disable-next-line no-debugger, quotes, semi */
            debugger;

            /* eslint-disable-next-line
              no-debugger,
              quotes,
              semi
            */
            debugger;
        ",
        // Comment descriptions
        "
            // eslint-disable-next-line no-debugger -- Here's a description about why this configuration is necessary.
            debugger;

            /* eslint-disable-next-line no-debugger --
             * Here's a very long description about why this configuration is necessary
             * along with some additional information
            **/
            debugger;
        "
    ];

    let fail = vec![
        "debugger",
        "
            debugger; // eslint-disable-line no-alert

            // eslint-disable-next-line no-alert
            debugger;

            debugger; /* eslint-disable-line no-alert */

            /* eslint-disable-next-line no-alert */
            debugger;
        ",
        "
            debugger; // eslint-disable-line no-alert, quotes, semi

            // eslint-disable-next-line no-alert, quotes, semi
            debugger;

            debugger; /* eslint-disable-line no-alert, quotes, semi */

            /* eslint-disable-next-line no-alert, quotes, semi */
            debugger;

            /* eslint-disable-next-line
              no-alert,
              quotes,
              semi
            */
            debugger;
        ",
    ];

    Tester::new_without_config("no-debugger", pass, fail).test();
}
