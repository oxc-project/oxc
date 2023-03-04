use std::borrow::Cow;

use oxc_diagnostics::Error;

use super::Fix;

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
        let source_text = self.source_text;
        if self.messages.iter().all(|m| matches!(m.fix, None)) {
            return FixResult {
                fixed: false,
                fixed_code: Cow::Borrowed(source_text),
                messages: self.messages,
            };
        }

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
            output.push_str(content);
            last_pos = i64::from(end);
        });

        let offset = usize::try_from(last_pos.max(0)).ok().unwrap();
        output.push_str(&source_text[offset..]);

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
    use crate::autofix::Fix;

    const TEST_CODE: &str = "var answer = 6 * 7;";

    #[derive(Debug, Error, Diagnostic)]
    #[error("End")]
    struct InsertAtEnd();
    const INSERT_AT_END: Fix =
        Fix { span: Span { start: 19, end: 19 }, content: Cow::Borrowed("// end") };

    // const INSERT_AT_START: Fix =
    //     Fix { span: Span { start: 0, end: 0 }, content: Cow::Borrowed("// start") };
    // const INSERT_AT_MIDDLE: Fix =
    //     Fix { span: Span { start: 13, end: 13 }, content: Cow::Borrowed("5 *") };
    // const REPLACE_ID: Fix = Fix { span: Span { start: 4, end: 10 }, content: Cow::Borrowed("foo") };
    // const REPLACE_VAR: Fix = Fix { span: Span { start: 0, end: 3 }, content: Cow::Borrowed("let") };
    // const REPLACE_NUM: Fix = Fix { span: Span { start: 13, end: 14 }, content: Cow::Borrowed("5") };
    // const REMOVE_START: Fix = Fix::delete(Span { start: 0, end: 4 });
    // const REMOVE_MIDDLE: Fix = Fix::delete(Span { start: 5, end: 10 });
    // const REMOVE_END: Fix = Fix::delete(Span { start: 14, end: 18 });
    // const REVERSE_RANGE: Fix = Fix { span: Span { start: 3, end: 0 }, content: Cow::Borrowed(" ") };

    fn get_fix_result(messages: Vec<Message>) -> FixResult {
        Fixer::new(TEST_CODE, messages).fix()
    }

    #[test]
    fn insert_at_the_end() {
        let foo: Error = InsertAtEnd().into();
        let msg = Message { error: foo, fixed: false, fix: Some(INSERT_AT_END) };
        let result = get_fix_result(vec![msg]);
        assert_eq!(result.fixed_code, TEST_CODE.to_string() + INSERT_AT_END.content.as_ref());
        assert_eq!(result.messages.len(), 0);
    }

    //     #[test]
    //     fn insert_at_the_start() {
    //         let fixer = get_fix_result(vec![INSERT_AT_START]);
    //         assert_eq!(fixer.fix(), INSERT_AT_START.content.to_string() + TEST_CODE);
    //     }

    //     #[test]
    //     fn insert_at_the_middle() {
    //         let fixer = get_fix_result(vec![INSERT_AT_MIDDLE]);
    //         assert_eq!(
    //             fixer.fix(),
    //             TEST_CODE.replace("6 *", &format!("{}{}", INSERT_AT_MIDDLE.content, "6 *"))
    //         );
    //     }

    //     #[test]
    //     fn insert_at_the_start_middle_end() {
    //         let fixer = get_fix_result(vec![INSERT_AT_MIDDLE, INSERT_AT_START, INSERT_AT_END]);
    //         assert_eq!(
    //             fixer.fix(),
    //             format!(
    //                 "{}{}{}",
    //                 INSERT_AT_START.content,
    //                 TEST_CODE.replace("6 *", &format!("{}{}", INSERT_AT_MIDDLE.content, "6 *")),
    //                 INSERT_AT_END.content
    //             )
    //         );
    //     }

    //     #[test]
    //     fn ignore_reverse_range() {
    //         let fixer = get_fix_result(vec![REVERSE_RANGE]);
    //         assert_eq!(fixer.fix(), TEST_CODE);
    //     }

    //     #[test]
    //     fn replace_at_the_start() {
    //         let fixer = get_fix_result(vec![REPLACE_VAR]);
    //         assert_eq!(fixer.fix(), TEST_CODE.replace("var", "let"));
    //     }

    //     #[test]
    //     fn replace_at_the_middle() {
    //         let fixer = get_fix_result(vec![REPLACE_ID]);
    //         assert_eq!(fixer.fix(), TEST_CODE.replace("answer", "foo"));
    //     }

    //     #[test]
    //     fn replace_at_the_end() {
    //         let fixer = get_fix_result(vec![REPLACE_NUM]);
    //         assert_eq!(fixer.fix(), TEST_CODE.replace('6', "5"));
    //     }

    //     #[test]
    //     fn replace_at_the_start_middle_end() {
    //         let fixer = get_fix_result(vec![REPLACE_ID, REPLACE_VAR, REPLACE_NUM]);
    //         assert_eq!(fixer.fix(), "let foo = 5 * 7;");
    //     }

    //     #[test]
    //     fn remove_at_the_start() {
    //         let fixer = get_fix_result(vec![REMOVE_START]);
    //         assert_eq!(fixer.fix(), TEST_CODE.replace("var ", ""));
    //     }

    //     #[test]
    //     fn remove_at_the_middle() {
    //         let fixer = get_fix_result(vec![REMOVE_MIDDLE]);
    //         assert_eq!(fixer.fix(), TEST_CODE.replace("answer", "a"));
    //     }

    //     #[test]
    //     fn remove_at_the_end() {
    //         let fixer = get_fix_result(vec![REMOVE_END]);
    //         assert_eq!(fixer.fix(), TEST_CODE.replace(" * 7", ""));
    //     }

    //     #[test]
    //     fn replace_at_start_remove_at_middle_insert_at_end() {
    //         let fixer = get_fix_result(vec![INSERT_AT_END, REMOVE_END, REPLACE_VAR]);
    //         assert_eq!(fixer.fix(), "let answer = 6;// end");
    //     }

    //     #[test]
    //     fn apply_one_fix_when_spans_overlap() {
    //         let fixer = get_fix_result(vec![REMOVE_MIDDLE, REPLACE_ID]);
    //         assert_eq!(fixer.fix(), TEST_CODE.replace("answer", "foo"));
    //     }

    //     #[test]
    //     fn apply_one_fix_when_the_start_the_same_as_the_previous_end() {
    //         let fixer = get_fix_result(vec![REMOVE_START, REPLACE_ID]);
    //         assert_eq!(fixer.fix(), TEST_CODE.replace("var ", ""));
    //     }

    //     #[ignore]
    //     #[test]
    //     fn apply_one_fix_when_range_overlap_and_one_message_has_no_fix() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[test]
    //     fn apply_same_fix_when_span_overlap_regardless_of_order() {
    //         let fixer1 = get_fix_result(vec![REMOVE_MIDDLE, REPLACE_ID]);
    //         let fixer2 = get_fix_result(vec![REPLACE_ID, REMOVE_MIDDLE]);
    //         assert_eq!(fixer1.fix(), fixer2.fix());
    //     }

    //     #[ignore]
    //     #[test]
    //     fn should_not_apply_fix_with_one_no_fix() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn sort_no_fix_messages_correctly() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn insert_bom_at_0() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn insert_bom_with_text_at_0() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn remove_bom_with_negative_range() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn replace_bom_with_negative_range_and_foobar() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     // With BOM
    //     #[ignore]
    //     #[test]
    //     fn insert_at_the_end_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn insert_at_the_start_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn insert_at_the_middle_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn insert_at_the_start_middle_end_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn ignore_reverse_range_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn replace_at_the_end_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn replace_at_the_start_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn replace_at_the_middle_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn replace_at_the_start_middle_end_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn remove_at_the_end_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn remove_at_the_start_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn remove_at_the_middle_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn remove_at_the_start_middle_end_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn replace_at_start_remove_at_middle_insert_at_end_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn apply_one_fix_when_spans_overlap_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn apply_one_fix_when_the_start_the_same_as_the_previous_end_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn apply_one_fix_when_range_overlap_and_one_message_has_no_fix_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn apply_same_fix_when_span_overlap_regardless_of_order_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn should_not_apply_fix_with_one_no_fix_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn insert_bom_at_0_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn insert_bom_with_text_at_0_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn remove_bom_with_negative_range_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }

    //     #[ignore]
    //     #[test]
    //     fn replace_bom_with_negative_range_and_foobar_with_bom() {
    //         let _fixer = get_fix_result(vec![]);
    //     }
}
