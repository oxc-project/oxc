use std::borrow::Cow;

use oxc_diagnostics::Error;
use oxc_span::Span;

#[derive(Debug, Default)]
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

#[derive(Debug)]
pub struct FixResult<'a> {
    pub fixed: bool,
    pub fixed_code: Cow<'a, str>,
    pub messages: Vec<Message<'a>>,
}

#[derive(Debug)]
pub struct Message<'a> {
    pub error: Error,
    start: u32,
    end: u32,
    pub fix: Option<Fix<'a>>,
    fixed: bool,
}

impl<'a> Message<'a> {
    #[allow(clippy::cast_possible_truncation)] // for `as u32`
    pub fn new(error: Error, fix: Option<Fix<'a>>) -> Self {
        let labels = error.labels().map_or(vec![], Iterator::collect);
        let start =
            labels.iter().min_by_key(|span| span.offset()).map_or(0, |span| span.offset() as u32);
        let end = labels
            .iter()
            .max_by_key(|span| span.offset() + span.len())
            .map_or(0, |span| (span.offset() + span.len()) as u32);
        Self { error, start, end, fix, fixed: false }
    }

    pub fn start(&self) -> u32 {
        self.start
    }

    pub fn end(&self) -> u32 {
        self.end
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
        return FixResult { fixed, fixed_code: Cow::Owned(output), messages };
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use miette::{self, Diagnostic};
    use oxc_diagnostics::{thiserror::Error, Error};
    use oxc_span::Span;

    use super::{Fix, FixResult, Fixer, Message};

    const TEST_CODE: &str = "var answer = 6 * 7;";

    #[derive(Debug, Error, Diagnostic)]
    #[error("End")]
    struct InsertAtEnd;
    const INSERT_AT_END: Fix = Fix { span: Span::new(19, 19), content: Cow::Borrowed("// end") };

    #[derive(Debug, Error, Diagnostic)]
    #[error("Start")]
    struct InsertAtStart;
    const INSERT_AT_START: Fix = Fix { span: Span::new(0, 0), content: Cow::Borrowed("// start") };

    #[derive(Debug, Error, Diagnostic)]
    #[error("Multiply")]
    struct InsertAtMiddle;
    const INSERT_AT_MIDDLE: Fix = Fix { span: Span::new(13, 13), content: Cow::Borrowed("5 *") };

    #[derive(Debug, Error, Diagnostic)]
    #[error("foo")]
    struct ReplaceId;
    const REPLACE_ID: Fix = Fix { span: Span::new(4, 10), content: Cow::Borrowed("foo") };
    #[derive(Debug, Error, Diagnostic)]
    #[error("let")]
    struct ReplaceVar;
    const REPLACE_VAR: Fix = Fix { span: Span::new(0, 3), content: Cow::Borrowed("let") };

    #[derive(Debug, Error, Diagnostic)]
    #[error("5")]
    struct ReplaceNum;
    const REPLACE_NUM: Fix = Fix { span: Span::new(13, 14), content: Cow::Borrowed("5") };

    #[derive(Debug, Error, Diagnostic)]
    #[error("removestart")]
    struct RemoveStart;
    const REMOVE_START: Fix = Fix::delete(Span::new(0, 4));

    #[derive(Debug, Error, Diagnostic)]
    #[error("removemiddle")]
    struct RemoveMiddle(#[label] pub Span);
    const REMOVE_MIDDLE: Fix = Fix::delete(Span::new(5, 10));

    #[derive(Debug, Error, Diagnostic)]
    #[error("removeend")]
    struct RemoveEnd;
    const REMOVE_END: Fix = Fix::delete(Span::new(14, 18));

    #[derive(Debug, Error, Diagnostic)]
    #[error("reversed range")]
    struct ReverseRange;
    const REVERSE_RANGE: Fix = Fix { span: Span::new(3, 0), content: Cow::Borrowed(" ") };

    #[derive(Debug, Error, Diagnostic)]
    #[error("nofix")]
    struct NoFix(#[label] pub Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error("nofix1")]
    struct NoFix1(#[label] pub Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error("nofix2")]
    struct NoFix2(#[label] pub Span);

    fn get_fix_result(messages: Vec<Message>) -> FixResult {
        Fixer::new(TEST_CODE, messages).fix()
    }

    fn create_message<T: Into<Error>>(error: T, fix: Option<Fix>) -> Message {
        Message::new(error.into(), fix)
    }

    #[test]
    fn insert_at_the_end() {
        let result = get_fix_result(vec![create_message(InsertAtEnd, Some(INSERT_AT_END))]);
        assert_eq!(result.fixed_code, TEST_CODE.to_string() + INSERT_AT_END.content.as_ref());
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn insert_at_the_start() {
        let result = get_fix_result(vec![create_message(InsertAtStart, Some(INSERT_AT_START))]);
        assert_eq!(result.fixed_code, INSERT_AT_START.content.to_string() + TEST_CODE);
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn insert_at_the_middle() {
        let result = get_fix_result(vec![create_message(InsertAtMiddle, Some(INSERT_AT_MIDDLE))]);
        assert_eq!(
            result.fixed_code,
            TEST_CODE.replace("6 *", &format!("{}{}", INSERT_AT_MIDDLE.content, "6 *"))
        );
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn insert_at_the_start_middle_end() {
        let messages = vec![
            create_message(InsertAtMiddle, Some(INSERT_AT_MIDDLE)),
            create_message(InsertAtStart, Some(INSERT_AT_START)),
            create_message(InsertAtEnd, Some(INSERT_AT_END)),
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
        let result = get_fix_result(vec![create_message(ReverseRange, Some(REVERSE_RANGE))]);
        assert_eq!(result.fixed_code, TEST_CODE);
    }

    #[test]
    fn replace_at_the_start() {
        let result = get_fix_result(vec![create_message(ReplaceVar, Some(REPLACE_VAR))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("var", "let"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_the_middle() {
        let result = get_fix_result(vec![create_message(ReplaceId, Some(REPLACE_ID))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("answer", "foo"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_the_end() {
        let result = get_fix_result(vec![create_message(ReplaceNum, Some(REPLACE_NUM))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace('6', "5"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_the_start_middle_end() {
        let messages = vec![
            create_message(ReplaceId, Some(REPLACE_ID)),
            create_message(ReplaceVar, Some(REPLACE_VAR)),
            create_message(ReplaceNum, Some(REPLACE_NUM)),
        ];
        let result = get_fix_result(messages);
        assert_eq!(result.fixed_code, "let foo = 5 * 7;");
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn remove_at_the_start() {
        let result = get_fix_result(vec![create_message(RemoveStart, Some(REMOVE_START))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("var ", ""));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn remove_at_the_middle() {
        let result = get_fix_result(vec![create_message(
            RemoveMiddle(Span::default()),
            Some(REMOVE_MIDDLE),
        )]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("answer", "a"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn remove_at_the_end() {
        let result = get_fix_result(vec![create_message(RemoveEnd, Some(REMOVE_END))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace(" * 7", ""));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_start_remove_at_middle_insert_at_end() {
        let result = get_fix_result(vec![
            create_message(InsertAtEnd, Some(INSERT_AT_END)),
            create_message(RemoveEnd, Some(REMOVE_END)),
            create_message(ReplaceVar, Some(REPLACE_VAR)),
        ]);
        assert_eq!(result.fixed_code, "let answer = 6;// end");
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn apply_one_fix_when_spans_overlap() {
        let result = get_fix_result(vec![
            create_message(RemoveMiddle(Span::default()), Some(REMOVE_MIDDLE)),
            create_message(ReplaceId, Some(REPLACE_ID)),
        ]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("answer", "foo"));
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].error.to_string(), "removemiddle");
        assert!(result.fixed);
    }

    #[test]
    fn apply_one_fix_when_the_start_the_same_as_the_previous_end() {
        let result = get_fix_result(vec![
            create_message(RemoveStart, Some(REMOVE_START)),
            create_message(ReplaceId, Some(REPLACE_ID)),
        ]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("var ", ""));
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].error.to_string(), "foo");
        assert!(result.fixed);
    }

    #[test]
    fn apply_one_fix_when_range_overlap_and_one_message_has_no_fix() {
        let result = get_fix_result(vec![
            create_message(RemoveMiddle(Span::default()), Some(REMOVE_MIDDLE)),
            create_message(ReplaceId, Some(REPLACE_ID)),
            create_message(NoFix(Span::default()), None),
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
            create_message(RemoveMiddle(Span::default()), Some(REMOVE_MIDDLE)),
            create_message(ReplaceId, Some(REPLACE_ID)),
        ]);
        let result2 = get_fix_result(vec![
            create_message(ReplaceId, Some(REPLACE_ID)),
            create_message(RemoveMiddle(Span::default()), Some(REMOVE_MIDDLE)),
        ]);
        assert_eq!(result1.fixed_code, result2.fixed_code);
    }

    #[test]
    fn should_not_apply_fix_with_one_no_fix() {
        let result = get_fix_result(vec![create_message(NoFix(Span::default()), None)]);
        assert_eq!(result.fixed_code, TEST_CODE);
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].error.to_string(), "nofix");
        assert!(!result.fixed);
    }

    #[test]
    fn sort_no_fix_messages_correctly() {
        let result = get_fix_result(vec![
            create_message(ReplaceId, Some(REPLACE_ID)),
            Message::new(NoFix2(Span { start: 1, end: 7 }).into(), None),
            Message::new(NoFix1(Span { start: 1, end: 3 }).into(), None),
        ]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("answer", "foo"));
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0].error.to_string(), "nofix1");
        assert_eq!(result.messages[1].error.to_string(), "nofix2");
        assert!(result.fixed);
    }
}
