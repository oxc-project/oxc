use std::borrow::Cow;

use oxc_diagnostics::Error;

use super::Fix;

const BOM: char = '\u{FEFF}';

pub struct FixResult<'a> {
    pub fixed: bool,
    pub fixed_code: Cow<'a, str>,
    pub messages: Vec<Message<'a>>,
}

#[derive(Debug)]
pub struct Message<'a> {
    pub error: Error,
    fix: Option<Fix<'a>>,
    fixed: bool,
}

impl<'a> Message<'a> {
    pub fn new(error: Error, fix: Option<Fix<'a>>) -> Self {
        Self { error, fix, fixed: false }
    }
}

pub struct Fixer<'a> {
    source_text: &'a str,
    messages: Vec<Message<'a>>,
}

impl<'a> Fixer<'a> {
    #[must_use]
    pub fn new(source_text: &'a str, messages: Vec<Message<'a>>) -> Self {
        Self { source_text, messages }
    }

    #[must_use]
    /// # Panics
    pub fn fix(mut self) -> FixResult<'a> {
        if self.messages.iter().all(|m| matches!(m.fix, None)) {
            return FixResult {
                fixed: false,
                fixed_code: Cow::Borrowed(self.source_text),
                messages: self.messages,
            };
        }

        let (source_text, mut prepend_bom) = if self.source_text.starts_with(BOM) {
            (&self.source_text[3..], true)
        } else {
            (self.source_text, false)
        };

        self.messages.sort_by_key(|m| m.fix.as_ref().unwrap_or(&Fix::default()).span);
        let mut fixed = false;
        let mut output = String::with_capacity(source_text.len());
        let mut last_pos: i64 = -1;
        self.messages.iter_mut().filter(|m| matches!(m.fix, Some(_))).for_each(|m| {
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
            if start == 0 && content.starts_with(BOM) {
                prepend_bom = true;
                output.push_str(&content[3..]);
            } else {
                output.push_str(content);
            }
            last_pos = i64::from(end);
        });

        let offset = usize::try_from(last_pos.max(0)).ok().unwrap();
        output.push_str(&source_text[offset..]);

        if prepend_bom {
            output.insert(0, BOM);
        }

        let messages = self.messages.into_iter().filter(|m| !m.fixed).collect::<Vec<_>>();

        return FixResult { fixed, fixed_code: Cow::Owned(output), messages };
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use miette::{self, Diagnostic};
    use oxc_ast::Span;
    use oxc_diagnostics::{thiserror::Error, Error};

    use super::{FixResult, Fixer, Message};
    use crate::autofix::{fixer::BOM, Fix};

    const TEST_CODE: &str = "var answer = 6 * 7;";
    const TEST_CODE_WITH_BOM: &str = "\u{FEFF}var answer = 6 * 7;";

    #[derive(Debug, Error, Diagnostic)]
    #[error("End")]
    struct InsertAtEnd();
    const INSERT_AT_END: Fix =
        Fix { span: Span { start: 19, end: 19 }, content: Cow::Borrowed("// end") };

    #[derive(Debug, Error, Diagnostic)]
    #[error("Start")]
    struct InsertAtStart();
    const INSERT_AT_START: Fix =
        Fix { span: Span { start: 0, end: 0 }, content: Cow::Borrowed("// start") };

    #[derive(Debug, Error, Diagnostic)]
    #[error("Multiply")]
    struct InsertAtMiddle();
    const INSERT_AT_MIDDLE: Fix =
        Fix { span: Span { start: 13, end: 13 }, content: Cow::Borrowed("5 *") };

    #[derive(Debug, Error, Diagnostic)]
    #[error("foo")]
    struct ReplaceId();
    const REPLACE_ID: Fix = Fix { span: Span { start: 4, end: 10 }, content: Cow::Borrowed("foo") };
    #[derive(Debug, Error, Diagnostic)]
    #[error("let")]
    struct ReplaceVar();
    const REPLACE_VAR: Fix = Fix { span: Span { start: 0, end: 3 }, content: Cow::Borrowed("let") };

    #[derive(Debug, Error, Diagnostic)]
    #[error("5")]
    struct ReplaceNum();
    const REPLACE_NUM: Fix = Fix { span: Span { start: 13, end: 14 }, content: Cow::Borrowed("5") };

    #[derive(Debug, Error, Diagnostic)]
    #[error("removestart")]
    struct RemoveStart();
    const REMOVE_START: Fix = Fix::delete(Span { start: 0, end: 4 });

    #[derive(Debug, Error, Diagnostic)]
    #[error("removemiddle")]
    struct RemoveMiddle();
    const REMOVE_MIDDLE: Fix = Fix::delete(Span { start: 5, end: 10 });

    #[derive(Debug, Error, Diagnostic)]
    #[error("removeend")]
    struct RemoveEnd();
    const REMOVE_END: Fix = Fix::delete(Span { start: 14, end: 18 });

    #[derive(Debug, Error, Diagnostic)]
    #[error("reversed range")]
    struct ReverseRange();
    const REVERSE_RANGE: Fix = Fix { span: Span { start: 3, end: 0 }, content: Cow::Borrowed(" ") };

    #[derive(Debug, Error, Diagnostic)]
    #[error("nofix")]
    struct NoFix();

    #[derive(Debug, Error, Diagnostic)]
    #[error("insert-bom")]
    struct InsertBOM();
    const INSERT_BOM: Fix =
        Fix { span: Span { start: 0, end: 0 }, content: Cow::Borrowed("\u{FEFF}") };

    #[derive(Debug, Error, Diagnostic)]
    #[error("insert-bom")]
    struct InsertBOMWithText();
    const INSERT_BOM_WITH_TEXT: Fix =
        Fix { span: Span { start: 0, end: 0 }, content: Cow::Borrowed("\u{FEFF}// start\n") };

    #[derive(Debug, Error, Diagnostic)]
    #[error("nofix1")]
    struct NoFix1();

    #[derive(Debug, Error, Diagnostic)]
    #[error("nofix2")]
    struct NoFix2();

    fn get_fix_result(messages: Vec<Message>) -> FixResult {
        Fixer::new(TEST_CODE, messages).fix()
    }

    fn create_message<T: Into<Error>>(error: T, fix: Option<Fix>) -> Message {
        Message::new(error.into(), fix)
    }

    #[test]
    fn insert_at_the_end() {
        let result = get_fix_result(vec![create_message(InsertAtEnd(), Some(INSERT_AT_END))]);
        assert_eq!(result.fixed_code, TEST_CODE.to_string() + INSERT_AT_END.content.as_ref());
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn insert_at_the_start() {
        let result = get_fix_result(vec![create_message(InsertAtStart(), Some(INSERT_AT_START))]);
        assert_eq!(result.fixed_code, INSERT_AT_START.content.to_string() + TEST_CODE);
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn insert_at_the_middle() {
        let result = get_fix_result(vec![create_message(InsertAtMiddle(), Some(INSERT_AT_MIDDLE))]);
        assert_eq!(
            result.fixed_code,
            TEST_CODE.replace("6 *", &format!("{}{}", INSERT_AT_MIDDLE.content, "6 *"))
        );
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn insert_at_the_start_middle_end() {
        let messages = vec![
            create_message(InsertAtMiddle(), Some(INSERT_AT_MIDDLE)),
            create_message(InsertAtStart(), Some(INSERT_AT_START)),
            create_message(InsertAtEnd(), Some(INSERT_AT_END)),
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
        let result = get_fix_result(vec![create_message(ReverseRange(), Some(REVERSE_RANGE))]);
        assert_eq!(result.fixed_code, TEST_CODE);
    }

    #[test]
    fn replace_at_the_start() {
        let result = get_fix_result(vec![create_message(ReplaceVar(), Some(REPLACE_VAR))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("var", "let"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_the_middle() {
        let result = get_fix_result(vec![create_message(ReplaceId(), Some(REPLACE_ID))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("answer", "foo"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_the_end() {
        let result = get_fix_result(vec![create_message(ReplaceNum(), Some(REPLACE_NUM))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace('6', "5"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_the_start_middle_end() {
        let messages = vec![
            create_message(ReplaceId(), Some(REPLACE_ID)),
            create_message(ReplaceVar(), Some(REPLACE_VAR)),
            create_message(ReplaceNum(), Some(REPLACE_NUM)),
        ];
        let result = get_fix_result(messages);
        assert_eq!(result.fixed_code, "let foo = 5 * 7;");
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn remove_at_the_start() {
        let result = get_fix_result(vec![create_message(RemoveStart(), Some(REMOVE_START))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("var ", ""));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn remove_at_the_middle() {
        let result = get_fix_result(vec![create_message(RemoveMiddle(), Some(REMOVE_MIDDLE))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("answer", "a"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn remove_at_the_end() {
        let result = get_fix_result(vec![create_message(RemoveEnd(), Some(REMOVE_END))]);
        assert_eq!(result.fixed_code, TEST_CODE.replace(" * 7", ""));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_start_remove_at_middle_insert_at_end() {
        let result = get_fix_result(vec![
            create_message(InsertAtEnd(), Some(INSERT_AT_END)),
            create_message(RemoveEnd(), Some(REMOVE_END)),
            create_message(ReplaceVar(), Some(REPLACE_VAR)),
        ]);
        assert_eq!(result.fixed_code, "let answer = 6;// end");
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn apply_one_fix_when_spans_overlap() {
        let result = get_fix_result(vec![
            create_message(RemoveMiddle(), Some(REMOVE_MIDDLE)),
            create_message(ReplaceId(), Some(REPLACE_ID)),
        ]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("answer", "foo"));
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].error.to_string(), "removemiddle");
        assert!(result.fixed);
    }

    #[test]
    fn apply_one_fix_when_the_start_the_same_as_the_previous_end() {
        let result = get_fix_result(vec![
            create_message(RemoveStart(), Some(REMOVE_START)),
            create_message(ReplaceId(), Some(REPLACE_ID)),
        ]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("var ", ""));
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].error.to_string(), "foo");
        assert!(result.fixed);
    }

    #[test]
    fn apply_one_fix_when_range_overlap_and_one_message_has_no_fix() {
        let result = get_fix_result(vec![
            create_message(RemoveMiddle(), Some(REMOVE_MIDDLE)),
            create_message(ReplaceId(), Some(REPLACE_ID)),
            create_message(NoFix(), None),
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
            create_message(RemoveMiddle(), Some(REMOVE_MIDDLE)),
            create_message(ReplaceId(), Some(REPLACE_ID)),
        ]);
        let result2 = get_fix_result(vec![
            create_message(ReplaceId(), Some(REPLACE_ID)),
            create_message(RemoveMiddle(), Some(REMOVE_MIDDLE)),
        ]);
        assert_eq!(result1.fixed_code, result2.fixed_code);
    }

    #[test]
    fn should_not_apply_fix_with_one_no_fix() {
        let result = get_fix_result(vec![create_message(NoFix(), None)]);
        assert_eq!(result.fixed_code, TEST_CODE);
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].error.to_string(), "nofix");
        assert!(!result.fixed);
    }

    #[ignore]
    #[test]
    fn sort_no_fix_messages_correctly() {
        let result = get_fix_result(vec![
            create_message(ReplaceId(), Some(REPLACE_ID)),
            create_message(NoFix2(), None),
            create_message(NoFix1(), None),
        ]);
        assert_eq!(result.fixed_code, TEST_CODE.replace("answer", "foo"));
        assert_eq!(result.messages.len(), 2);
        // We should sort the error to pass the following assertion.
        assert_eq!(result.messages[0].error.to_string(), "nofix1");
        assert_eq!(result.messages[0].error.to_string(), "nofix2");
        assert!(result.fixed);
    }

    #[test]
    fn insert_bom_at_0() {
        let result = get_fix_result(vec![create_message(InsertBOM(), Some(INSERT_BOM))]);
        assert_eq!(result.fixed_code, TEST_CODE_WITH_BOM);
        assert!(result.fixed);
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn insert_bom_with_text_at_0() {
        let result =
            get_fix_result(vec![create_message(InsertBOMWithText(), Some(INSERT_BOM_WITH_TEXT))]);
        assert_eq!(result.fixed_code, format!("\u{FEFF}// start\n{TEST_CODE}"));
        assert!(result.fixed);
        assert_eq!(result.messages.len(), 0);
    }

    #[ignore]
    #[test]
    fn remove_bom_with_negative_range() {
        let _result = get_fix_result(vec![]);
    }

    #[ignore]
    #[test]
    fn replace_bom_with_negative_range_and_foobar() {
        let _result = get_fix_result(vec![]);
    }

    fn get_fix_result_with_bom(messages: Vec<Message>) -> FixResult {
        Fixer::new(TEST_CODE_WITH_BOM, messages).fix()
    }

    // With BOM
    #[test]
    fn insert_at_the_end_with_bom() {
        let result =
            get_fix_result_with_bom(vec![create_message(InsertAtEnd(), Some(INSERT_AT_END))]);
        assert_eq!(result.fixed_code, format!("{}{}", TEST_CODE_WITH_BOM, INSERT_AT_END.content));
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn insert_at_the_start_with_bom() {
        let result =
            get_fix_result_with_bom(vec![create_message(InsertAtStart(), Some(INSERT_AT_START))]);
        assert_eq!(result.fixed_code, format!("{}{}{}", BOM, INSERT_AT_START.content, TEST_CODE));
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn insert_at_the_middle_with_bom() {
        let result =
            get_fix_result_with_bom(vec![create_message(InsertAtMiddle(), Some(INSERT_AT_MIDDLE))]);
        let insert_in_middle =
            TEST_CODE.replace("6 *", &format!("{}6 *", INSERT_AT_MIDDLE.content));
        assert_eq!(result.fixed_code, format!("{BOM}{insert_in_middle}"));
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn insert_at_the_start_middle_end_with_bom() {
        let result = get_fix_result_with_bom(vec![
            create_message(InsertAtMiddle(), Some(INSERT_AT_MIDDLE)),
            create_message(InsertAtStart(), Some(INSERT_AT_START)),
            create_message(InsertAtEnd(), Some(INSERT_AT_END)),
        ]);
        let insert_in_middle =
            TEST_CODE.replace("6 *", &format!("{}6 *", INSERT_AT_MIDDLE.content));
        assert_eq!(
            result.fixed_code,
            format!(
                "{}{}{}{}",
                BOM, INSERT_AT_START.content, insert_in_middle, INSERT_AT_END.content
            )
        );
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn ignore_reverse_range_with_bom() {
        let result =
            get_fix_result_with_bom(vec![create_message(ReverseRange(), Some(REVERSE_RANGE))]);
        assert_eq!(result.fixed_code, TEST_CODE_WITH_BOM);
    }

    #[test]
    fn replace_at_the_end_with_bom() {
        let result = get_fix_result_with_bom(vec![create_message(ReplaceVar(), Some(REPLACE_VAR))]);
        assert_eq!(result.fixed_code, TEST_CODE_WITH_BOM.replace("var", "let"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_the_start_with_bom() {
        let result = get_fix_result_with_bom(vec![create_message(ReplaceId(), Some(REPLACE_ID))]);
        assert_eq!(result.fixed_code, TEST_CODE_WITH_BOM.replace("answer", "foo"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_the_middle_with_bom() {
        let result = get_fix_result_with_bom(vec![create_message(ReplaceNum(), Some(REPLACE_NUM))]);
        assert_eq!(result.fixed_code, TEST_CODE_WITH_BOM.replace('6', "5"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_the_start_middle_end_with_bom() {
        let result = get_fix_result_with_bom(vec![
            create_message(ReplaceId(), Some(REPLACE_ID)),
            create_message(ReplaceVar(), Some(REPLACE_VAR)),
            create_message(ReplaceNum(), Some(REPLACE_NUM)),
        ]);
        assert_eq!(result.fixed_code, format!("{BOM}let foo = 5 * 7;"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn remove_at_the_end_with_bom() {
        let result = get_fix_result_with_bom(vec![create_message(RemoveEnd(), Some(REMOVE_END))]);
        assert_eq!(result.fixed_code, TEST_CODE_WITH_BOM.replace(" * 7", ""));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn remove_at_the_start_with_bom() {
        let result =
            get_fix_result_with_bom(vec![create_message(RemoveStart(), Some(REMOVE_START))]);
        assert_eq!(result.fixed_code, format!("{}{}", BOM, TEST_CODE.replace("var ", "")));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn remove_at_the_middle_with_bom() {
        let result =
            get_fix_result_with_bom(vec![create_message(RemoveMiddle(), Some(REMOVE_MIDDLE))]);
        assert_eq!(result.fixed_code, format!("{}{}", BOM, TEST_CODE.replace("answer", "a")));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn remove_at_the_start_middle_end_with_bom() {
        let result = get_fix_result_with_bom(vec![
            create_message(RemoveEnd(), Some(REMOVE_END)),
            create_message(RemoveStart(), Some(REMOVE_START)),
            create_message(RemoveMiddle(), Some(REMOVE_MIDDLE)),
        ]);
        assert_eq!(result.fixed_code, format!("{BOM}a = 6;"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn replace_at_start_remove_at_middle_insert_at_end_with_bom() {
        let result = get_fix_result_with_bom(vec![
            create_message(InsertAtEnd(), Some(INSERT_AT_END)),
            create_message(RemoveEnd(), Some(REMOVE_END)),
            create_message(ReplaceVar(), Some(REPLACE_VAR)),
        ]);
        assert_eq!(result.fixed_code, format!("{BOM}let answer = 6;// end"));
        assert_eq!(result.messages.len(), 0);
        assert!(result.fixed);
    }

    #[test]
    fn apply_one_fix_when_spans_overlap_with_bom() {
        let result = get_fix_result_with_bom(vec![
            create_message(RemoveMiddle(), Some(REMOVE_MIDDLE)),
            create_message(ReplaceId(), Some(REPLACE_ID)),
        ]);
        assert_eq!(result.fixed_code, TEST_CODE_WITH_BOM.replace("answer", "foo"));
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].error.to_string(), "removemiddle");
        assert!(result.fixed);
    }

    #[test]
    fn apply_one_fix_when_the_start_the_same_as_the_previous_end_with_bom() {
        let result = get_fix_result_with_bom(vec![
            create_message(RemoveStart(), Some(REMOVE_START)),
            create_message(ReplaceId(), Some(REPLACE_ID)),
        ]);
        assert_eq!(result.fixed_code, TEST_CODE_WITH_BOM.replace("var ", ""));
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].error.to_string(), "foo");
        assert!(result.fixed);
    }

    #[test]
    fn apply_one_fix_when_range_overlap_and_one_message_has_no_fix_with_bom() {
        let result = get_fix_result_with_bom(vec![
            create_message(RemoveMiddle(), Some(REMOVE_MIDDLE)),
            create_message(ReplaceId(), Some(REPLACE_ID)),
            create_message(NoFix(), None),
        ]);
        assert_eq!(result.fixed_code, TEST_CODE_WITH_BOM.replace("answer", "foo"));
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0].error.to_string(), "nofix");
        assert_eq!(result.messages[1].error.to_string(), "removemiddle");
        assert!(result.fixed);
    }

    #[test]
    fn apply_same_fix_when_span_overlap_regardless_of_order_with_bom() {
        let result1 = get_fix_result_with_bom(vec![
            create_message(RemoveMiddle(), Some(REMOVE_MIDDLE)),
            create_message(ReplaceId(), Some(REPLACE_ID)),
        ]);
        let result2 = get_fix_result_with_bom(vec![
            create_message(ReplaceId(), Some(REPLACE_ID)),
            create_message(RemoveMiddle(), Some(REMOVE_MIDDLE)),
        ]);
        assert_eq!(result1.fixed_code, result2.fixed_code);
    }

    #[test]
    fn should_not_apply_fix_with_one_no_fix_with_bom() {
        let result = get_fix_result_with_bom(vec![create_message(NoFix(), None)]);
        assert_eq!(result.fixed_code, TEST_CODE_WITH_BOM);
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].error.to_string(), "nofix");
        assert!(!result.fixed);
    }

    #[test]
    fn insert_bom_at_0_with_bom() {
        let result = get_fix_result_with_bom(vec![create_message(InsertBOM(), Some(INSERT_BOM))]);
        assert_eq!(result.fixed_code, TEST_CODE_WITH_BOM);
        assert!(result.fixed);
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn insert_bom_with_text_at_0_with_bom() {
        let result = get_fix_result_with_bom(vec![create_message(
            InsertBOMWithText(),
            Some(INSERT_BOM_WITH_TEXT),
        )]);
        assert_eq!(result.fixed_code, format!("\u{FEFF}// start\n{TEST_CODE}"));
        assert!(result.fixed);
        assert_eq!(result.messages.len(), 0);
    }

    #[ignore]
    #[test]
    fn remove_bom_with_negative_range_with_bom() {
        let _result = get_fix_result(vec![]);
    }

    #[ignore]
    #[test]
    fn replace_bom_with_negative_range_and_foobar_with_bom() {
        let _result = get_fix_result(vec![]);
    }
}
