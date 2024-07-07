use std::borrow::Cow;

use oxc_codegen::Codegen;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};

use crate::LintContext;

#[derive(Debug, Clone, Default)]
pub struct Fix<'a> {
    pub content: Cow<'a, str>,
    pub span: Span,
}

impl<'a> Fix<'a> {
    pub const fn delete(span: Span) -> Self {
        Self { content: Cow::Borrowed(""), span }
    }

    pub fn new<T: Into<Cow<'a, str>>>(content: T, span: Span) -> Self {
        Self { content: content.into(), span }
    }
}

pub enum CompositeFix<'a> {
    Single(Fix<'a>),
    Multiple(Vec<Fix<'a>>),
}

impl<'a> From<Fix<'a>> for CompositeFix<'a> {
    fn from(fix: Fix<'a>) -> Self {
        CompositeFix::Single(fix)
    }
}

impl<'a> From<Vec<Fix<'a>>> for CompositeFix<'a> {
    fn from(fixes: Vec<Fix<'a>>) -> Self {
        CompositeFix::Multiple(fixes)
    }
}

impl<'a> CompositeFix<'a> {
    /// Gets one fix from the fixes. If we retrieve multiple fixes, this merges those into one.
    /// <https://github.com/eslint/eslint/blob/main/lib/linter/report-translator.js#L181-L203>
    pub fn normalize_fixes(self, source_text: &str) -> Fix<'a> {
        match self {
            CompositeFix::Single(fix) => fix,
            CompositeFix::Multiple(fixes) => Self::merge_fixes(fixes, source_text),
        }
    }
    // Merges multiple fixes to one, returns an `Fix::default`(which will not fix anything) if:
    // 1. `fixes` is empty
    // 2. contains overlapped ranges
    // 3. contains negative ranges (span.start > span.end)
    // <https://github.com/eslint/eslint/blob/main/lib/linter/report-translator.js#L147-L179>
    fn merge_fixes(fixes: Vec<Fix<'a>>, source_text: &str) -> Fix<'a> {
        let mut fixes = fixes;
        let empty_fix = Fix::default();
        if fixes.is_empty() {
            // Do nothing
            return empty_fix;
        }
        if fixes.len() == 1 {
            return fixes.pop().unwrap();
        }

        fixes.sort_by(|a, b| a.span.cmp(&b.span));

        // safe, as fixes.len() > 1
        let start = fixes[0].span.start;
        let end = fixes[fixes.len() - 1].span.end;
        let mut last_pos = start;
        let mut output = String::new();

        for fix in fixes {
            let Fix { ref content, span } = fix;
            // negative range or overlapping ranges is invalid
            if span.start > span.end {
                debug_assert!(false, "Negative range is invalid: {span:?}");
                return empty_fix;
            }
            if last_pos > span.start {
                debug_assert!(
                    false,
                    "Fix must not be overlapped, last_pos: {}, span.start: {}",
                    last_pos, span.start
                );
                return empty_fix;
            }

            let Some(before) = source_text.get((last_pos) as usize..span.start as usize) else {
                debug_assert!(false, "Invalid range: {}, {}", last_pos, span.start);
                return empty_fix;
            };

            output.push_str(before);
            output.push_str(content);
            last_pos = span.end;
        }

        let Some(after) = source_text.get(last_pos as usize..end as usize) else {
            debug_assert!(false, "Invalid range: {:?}", last_pos as usize..end as usize);
            return empty_fix;
        };

        output.push_str(after);
        Fix::new(output, Span::new(start, end))
    }
}

/// Inspired by ESLint's [`RuleFixer`].
///
/// [`RuleFixer`]: https://github.com/eslint/eslint/blob/main/lib/linter/rule-fixer.js
#[derive(Clone, Copy)]
pub struct RuleFixer<'c, 'a: 'c> {
    ctx: &'c LintContext<'a>,
}

impl<'c, 'a: 'c> RuleFixer<'c, 'a> {
    pub fn new(ctx: &'c LintContext<'a>) -> Self {
        Self { ctx }
    }

    pub fn source_range(self, span: Span) -> &'a str {
        self.ctx.source_range(span)
    }

    /// Create a [`Fix`] that deletes the text covered by the given [`Span`] or
    /// AST node.
    pub fn delete<S: GetSpan>(self, spanned: &S) -> Fix<'a> {
        self.delete_range(spanned.span())
    }

    #[allow(clippy::unused_self)]
    pub fn delete_range(self, span: Span) -> Fix<'a> {
        Fix::delete(span)
    }

    /// Replace a `target` AST node with the source code of a `replacement` node..
    pub fn replace_with<T: GetSpan, S: GetSpan>(self, target: &T, replacement: &S) -> Fix<'a> {
        let replacement_text = self.ctx.source_range(replacement.span());
        Fix::new(replacement_text, target.span())
    }

    /// Replace a `target` AST node with a `replacement` string.
    #[allow(clippy::unused_self)]
    pub fn replace<S: Into<Cow<'a, str>>>(self, target: Span, replacement: S) -> Fix<'a> {
        Fix::new(replacement, target)
    }

    #[allow(clippy::unused_self)]
    pub fn codegen(self) -> Codegen<'a, false> {
        Codegen::<false>::new()
    }
}

pub struct FixResult<'a> {
    #[allow(unused)]
    pub fixed: bool,
    pub fixed_code: Cow<'a, str>,
    pub messages: Vec<Message<'a>>,
}

#[derive(Clone)]
pub struct Message<'a> {
    pub error: OxcDiagnostic,
    pub start: u32,
    pub end: u32,
    pub fix: Option<Fix<'a>>,
    fixed: bool,
}

impl<'a> Message<'a> {
    #[allow(clippy::cast_possible_truncation)] // for `as u32`
    pub fn new(error: OxcDiagnostic, fix: Option<Fix<'a>>) -> Self {
        let (start, end) = if let Some(labels) = &error.labels {
            let start = labels
                .iter()
                .min_by_key(|span| span.offset())
                .map_or(0, |span| span.offset() as u32);
            let end = labels
                .iter()
                .max_by_key(|span| span.offset() + span.len())
                .map_or(0, |span| (span.offset() + span.len()) as u32);
            (start, end)
        } else {
            (0, 0)
        };
        Self { error, start, end, fix, fixed: false }
    }

    #[inline]
    pub fn start(&self) -> u32 {
        self.start
    }

    #[inline]
    pub fn end(&self) -> u32 {
        self.end
    }
}

impl<'a> GetSpan for Message<'a> {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.start(), self.end())
    }
}

/// The fixer of the code.
/// Note that our parser has handled the BOM, so we don't need to port the BOM test cases from `ESLint`.
pub struct Fixer<'a> {
    source_text: &'a str,
    messages: Vec<Message<'a>>,
}

impl<'a> Fixer<'a> {
    pub fn new(source_text: &'a str, messages: Vec<Message<'a>>) -> Self {
        Self { source_text, messages }
    }

    /// # Panics
    pub fn fix(mut self) -> FixResult<'a> {
        let source_text = self.source_text;
        if self.messages.iter().all(|m| m.fix.is_none()) {
            return FixResult {
                fixed: false,
                fixed_code: Cow::Borrowed(source_text),
                messages: self.messages,
            };
        }

        self.messages.sort_by_key(|m| m.fix.as_ref().unwrap_or(&Fix::default()).span);
        let mut fixed = false;
        let mut output = String::with_capacity(source_text.len());
        let mut last_pos: i64 = -1;
        self.messages.iter_mut().filter(|m| m.fix.is_some()).for_each(|m| {
            let Fix { content, span } = m.fix.as_ref().unwrap();
            let start = span.start;
            let end = span.end;
            if start > end {
                return;
            }
            if i64::from(start) <= last_pos {
                return;
            }

            m.fixed = true;
            fixed = true;
            let offset = usize::try_from(last_pos.max(0)).ok().unwrap();
            output.push_str(&source_text[offset..start as usize]);
            output.push_str(content);
            last_pos = i64::from(end);
        });

        let offset = usize::try_from(last_pos.max(0)).ok().unwrap();
        output.push_str(&source_text[offset..]);

        let mut messages = self.messages.into_iter().filter(|m| !m.fixed).collect::<Vec<_>>();
        messages.sort_by_key(|m| (m.start, m.end));
        FixResult { fixed, fixed_code: Cow::Owned(output), messages }
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use oxc_diagnostics::OxcDiagnostic;
    use oxc_span::Span;

    use super::{CompositeFix, Fix, FixResult, Fixer, Message};

    fn insert_at_end() -> OxcDiagnostic {
        OxcDiagnostic::warn("End")
    }

    fn insert_at_start() -> OxcDiagnostic {
        OxcDiagnostic::warn("Start")
    }

    fn insert_at_middle() -> OxcDiagnostic {
        OxcDiagnostic::warn("Multiply")
    }

    fn replace_id() -> OxcDiagnostic {
        OxcDiagnostic::warn("foo")
    }

    fn replace_var() -> OxcDiagnostic {
        OxcDiagnostic::warn("let")
    }

    fn replace_num() -> OxcDiagnostic {
        OxcDiagnostic::warn("5")
    }

    fn remove_start() -> OxcDiagnostic {
        OxcDiagnostic::warn("removestart")
    }

    fn remove_middle(span: Span) -> OxcDiagnostic {
        OxcDiagnostic::warn("removemiddle").with_label(span)
    }

    fn remove_end() -> OxcDiagnostic {
        OxcDiagnostic::warn("removeend")
    }

    fn reverse_range() -> OxcDiagnostic {
        OxcDiagnostic::warn("reversed range")
    }

    fn no_fix(span: Span) -> OxcDiagnostic {
        OxcDiagnostic::warn("nofix").with_label(span)
    }

    fn no_fix_1(span: Span) -> OxcDiagnostic {
        OxcDiagnostic::warn("nofix1").with_label(span)
    }

    fn no_fix_2(span: Span) -> OxcDiagnostic {
        OxcDiagnostic::warn("nofix2").with_label(span)
    }

    const TEST_CODE: &str = "var answer = 6 * 7;";
    const INSERT_AT_END: Fix = Fix { span: Span::new(19, 19), content: Cow::Borrowed("// end") };
    const INSERT_AT_START: Fix = Fix { span: Span::new(0, 0), content: Cow::Borrowed("// start") };
    const INSERT_AT_MIDDLE: Fix = Fix { span: Span::new(13, 13), content: Cow::Borrowed("5 *") };
    const REPLACE_ID: Fix = Fix { span: Span::new(4, 10), content: Cow::Borrowed("foo") };
    const REPLACE_VAR: Fix = Fix { span: Span::new(0, 3), content: Cow::Borrowed("let") };
    const REPLACE_NUM: Fix = Fix { span: Span::new(13, 14), content: Cow::Borrowed("5") };
    const REMOVE_START: Fix = Fix::delete(Span::new(0, 4));
    const REMOVE_MIDDLE: Fix = Fix::delete(Span::new(5, 10));
    const REMOVE_END: Fix = Fix::delete(Span::new(14, 18));
    const REVERSE_RANGE: Fix = Fix { span: Span::new(3, 0), content: Cow::Borrowed(" ") };

    fn get_fix_result(messages: Vec<Message>) -> FixResult {
        Fixer::new(TEST_CODE, messages).fix()
    }

    fn create_message(error: OxcDiagnostic, fix: Option<Fix>) -> Message {
        Message::new(error, fix)
    }

    #[test]
    fn insert_at_the_end() {
        let result = get_fix_result(vec![create_message(insert_at_end(), Some(INSERT_AT_END))]);
        assert_eq!(result.fixed_code, TEST_CODE.to_string() + INSERT_AT_END.content.as_ref());
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn insert_at_the_start() {
        let result = get_fix_result(vec![create_message(insert_at_start(), Some(INSERT_AT_START))]);
        assert_eq!(result.fixed_code, INSERT_AT_START.content.to_string() + TEST_CODE);
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn insert_at_the_middle() {
        let result =
            get_fix_result(vec![create_message(insert_at_middle(), Some(INSERT_AT_MIDDLE))]);
        assert_eq!(
            result.fixed_code,
            TEST_CODE.replace("6 *", &format!("{}{}", INSERT_AT_MIDDLE.content, "6 *"))
        );
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn insert_at_the_start_middle_end() {
        let messages = vec![
            create_message(insert_at_middle(), Some(INSERT_AT_MIDDLE)),
            create_message(insert_at_start(), Some(INSERT_AT_START)),
            create_message(insert_at_end(), Some(INSERT_AT_END)),
        ];
        let result = get_fix_result(messages);
        assert_eq!(
            result.fixed_code,
            format!(
                "{}{}{}",
                INSERT_AT_START.content,
                TEST_CODE.replace("6 *", &format!("{}{}", INSERT_AT_MIDDLE.content, "6 *")),
                INSERT_AT_END.content
            )
        );
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn ignore_reverse_range() {
        let result = get_fix_result(vec![create_message(reverse_range(), Some(REVERSE_RANGE))]);
        assert_eq!(result.fixed_code, TEST_CODE);
    }

    #[test]
    fn replace_at_the_start() {
        let result = get_fix_result(vec![create_message(replace_var(), Some(REPLACE_VAR))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("var", "let"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_the_middle() {
        let result = get_fix_result(vec![create_message(replace_id(), Some(REPLACE_ID))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("answer", "foo"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_the_end() {
        let result = get_fix_result(vec![create_message(replace_num(), Some(REPLACE_NUM))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace('6', "5"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_the_start_middle_end() {
        let messages = vec![
            create_message(replace_id(), Some(REPLACE_ID)),
            create_message(replace_var(), Some(REPLACE_VAR)),
            create_message(replace_num(), Some(REPLACE_NUM)),
        ];
        let result = get_fix_result(messages);
        assert_eq!(result.fixed_code, "let foo = 5 * 7;");
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn remove_at_the_start() {
        let result = get_fix_result(vec![create_message(remove_start(), Some(REMOVE_START))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("var ", ""));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn remove_at_the_middle() {
        let result = get_fix_result(vec![create_message(
            remove_middle(Span::default()),
            Some(REMOVE_MIDDLE),
        )]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("answer", "a"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn remove_at_the_end() {
        let result = get_fix_result(vec![create_message(remove_end(), Some(REMOVE_END))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace(" * 7", ""));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_start_remove_at_middle_insert_at_end() {
        let result = get_fix_result(vec![
            create_message(insert_at_end(), Some(INSERT_AT_END)),
            create_message(remove_end(), Some(REMOVE_END)),
            create_message(replace_var(), Some(REPLACE_VAR)),
        ]);
        assert_eq!(result.fixed_code, "let answer = 6;// end");
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn apply_one_fix_when_spans_overlap() {
        let result = get_fix_result(vec![
            create_message(remove_middle(Span::default()), Some(REMOVE_MIDDLE)),
            create_message(replace_id(), Some(REPLACE_ID)),
        ]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("answer", "foo"));
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].error.to_string(), "removemiddle");
        assert!(result.fixed);
    }

    #[test]
    fn apply_one_fix_when_the_start_the_same_as_the_previous_end() {
        let result = get_fix_result(vec![
            create_message(remove_start(), Some(REMOVE_START)),
            create_message(replace_id(), Some(REPLACE_ID)),
        ]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("var ", ""));
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].error.to_string(), "foo");
        assert!(result.fixed);
    }

    #[test]
    fn apply_one_fix_when_range_overlap_and_one_message_has_no_fix() {
        let result = get_fix_result(vec![
            create_message(remove_middle(Span::default()), Some(REMOVE_MIDDLE)),
            create_message(replace_id(), Some(REPLACE_ID)),
            create_message(no_fix(Span::default()), None),
        ]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("answer", "foo"));
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0].error.to_string(), "nofix");
        assert_eq!(result.messages[1].error.to_string(), "removemiddle");
        assert!(result.fixed);
    }

    #[test]
    fn apply_same_fix_when_span_overlap_regardless_of_order() {
        let result1 = get_fix_result(vec![
            create_message(remove_middle(Span::default()), Some(REMOVE_MIDDLE)),
            create_message(replace_id(), Some(REPLACE_ID)),
        ]);
        let result2 = get_fix_result(vec![
            create_message(replace_id(), Some(REPLACE_ID)),
            create_message(remove_middle(Span::default()), Some(REMOVE_MIDDLE)),
        ]);
        assert_eq!(result1.fixed_code, result2.fixed_code);
    }

    #[test]
    fn should_not_apply_fix_with_one_no_fix() {
        let result = get_fix_result(vec![create_message(no_fix(Span::default()), None)]);
        assert_eq!(result.fixed_code, TEST_CODE);
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].error.to_string(), "nofix");
        assert!(!result.fixed);
    }

    #[test]
    fn sort_no_fix_messages_correctly() {
        let result = get_fix_result(vec![
            create_message(replace_id(), Some(REPLACE_ID)),
            Message::new(no_fix_2(Span::new(1, 7)), None),
            Message::new(no_fix_1(Span::new(1, 3)), None),
        ]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("answer", "foo"));
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0].error.to_string(), "nofix1");
        assert_eq!(result.messages[1].error.to_string(), "nofix2");
        assert!(result.fixed);
    }

    fn assert_fixed_corrected(source_text: &str, expected: &str, composite_fix: CompositeFix) {
        let mut source_text = source_text.to_string();
        let fix = composite_fix.normalize_fixes(&source_text);
        let start = fix.span.start as usize;
        let end = fix.span.end as usize;
        source_text.replace_range(start..end, fix.content.to_string().as_str());
        assert_eq!(source_text, expected);
    }

    #[test]
    fn merge_fixes_in_composite_fix() {
        let source_text = "foo bar baz";
        let fixes = vec![Fix::new("quux", Span::new(0, 3)), Fix::new("qux", Span::new(4, 7))];
        let composite_fix = CompositeFix::Multiple(fixes);
        assert_fixed_corrected(source_text, "quux qux baz", composite_fix);
    }

    #[test]
    fn one_fix_in_composite_fix() {
        let source_text = "foo bar baz";
        let fix = Fix::new("quxx", Span::new(4, 7));
        let composite_fix = CompositeFix::Single(fix.clone());
        assert_fixed_corrected(source_text, "foo quxx baz", composite_fix);

        let composite_fix = CompositeFix::Multiple(vec![fix]);
        assert_fixed_corrected(source_text, "foo quxx baz", composite_fix);
    }

    #[test]
    fn zero_fixes_in_composite_fix() {
        let source_text = "foo bar baz";
        let composite_fix = CompositeFix::Multiple(vec![]);
        assert_fixed_corrected(source_text, source_text, composite_fix);
    }

    #[test]
    #[should_panic(expected = "Fix must not be overlapped, last_pos: 3, span.start: 2")]
    fn overlapping_ranges_in_composite_fix() {
        let source_text = "foo bar baz";
        let fixes = vec![Fix::new("quux", Span::new(0, 3)), Fix::new("qux", Span::new(2, 5))];
        let composite_fix = CompositeFix::Multiple(fixes);
        assert_fixed_corrected(source_text, source_text, composite_fix);
    }

    #[test]
    #[should_panic(expected = "Negative range is invalid: Span { start: 5, end: 2 }")]
    fn negative_ranges_in_composite_fix() {
        let source_text = "foo bar baz";
        let fixes = vec![Fix::new("quux", Span::new(0, 3)), Fix::new("qux", Span::new(5, 2))];
        let composite_fix = CompositeFix::Multiple(fixes);
        assert_fixed_corrected(source_text, source_text, composite_fix);
    }

    fn assert_fixes_merged(fixes: Vec<Fix>, fix: &Fix, source_text: &str) {
        let composite_fix = CompositeFix::from(fixes);
        let merged_fix = composite_fix.normalize_fixes(source_text);
        assert_eq!(merged_fix.content, fix.content);
        assert_eq!(merged_fix.span, fix.span);
    }

    // Remain test caces picked from eslint
    // <https://github.com/eslint/eslint/blob/main/tests/lib/linter/report-translator.js>
    // 1. Combining autofixes
    #[test]
    fn merge_fixes_into_one() {
        let source_text = "foo\nbar";
        let fixes = vec![Fix::new("foo", Span::new(1, 2)), Fix::new("bar", Span::new(4, 5))];
        assert_fixes_merged(fixes, &Fix::new("fooo\nbar", Span::new(1, 5)), source_text);
    }

    #[test]
    fn respect_ranges_of_empty_insertions() {
        let source_text = "foo\nbar";
        let fixes = vec![
            Fix::new("cd", Span::new(4, 5)),
            Fix::new("", Span::new(2, 2)),
            Fix::new("", Span::new(7, 7)),
        ];
        assert_fixes_merged(fixes, &Fix::new("o\ncdar", Span::new(2, 7)), source_text);
    }

    #[test]
    fn pass_through_fixes_if_only_one_present() {
        let source_text = "foo\nbar";
        let fix = Fix::new("foo", Span::new(1, 2));
        assert_fixes_merged(vec![fix.clone()], &fix, source_text);
    }

    #[test]
    #[should_panic(expected = "Fix must not be overlapped, last_pos: 3, span.start: 2")]
    fn throw_error_when_ranges_overlap() {
        let source_text = "foo\nbar";
        let fixes = vec![Fix::new("foo", Span::new(0, 3)), Fix::new("x", Span::new(2, 5))];
        assert_fixes_merged(fixes, &Fix::default(), source_text);
    }

    // 2. unique `fix` and `fix.range` objects
    #[test]
    fn return_new_fix_when_fixes_is_one() {
        let source_text = "foo\nbar";
        let fix = Fix::new("baz", Span::new(0, 3));
        let fixes = vec![fix.clone()];

        assert_fixes_merged(fixes, &fix, source_text);
    }

    #[test]
    fn create_new_fix_with_new_range_when_fixes_is_multiple() {
        let source_text = "foo\nbar";
        let fixes = vec![Fix::new("baz", Span::new(0, 3)), Fix::new("qux", Span::new(4, 7))];

        assert_fixes_merged(fixes, &Fix::new("baz\nqux", Span::new(0, 7)), source_text);
    }
}
