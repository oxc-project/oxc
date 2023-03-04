use std::{borrow::Cow, cmp::Ordering};

use oxc_ast::Span;

use super::Fix;

pub struct Fixer<'a> {
    source_text: &'a str,
    fixes: Vec<Fix<'a>>,
}

impl<'a> Fixer<'a> {
    pub fn new(source_text: &'a str, mut fixes: Vec<Fix<'a>>) -> Self {
        fixes.sort_by(
            |Fix { span: Span { start: a_start, end: a_end }, .. },
             Fix { span: Span { start: b_start, end: b_end }, .. }| {
                if a_start < b_start {
                    Ordering::Less
                } else if a_start > b_start {
                    Ordering::Greater
                } else {
                    if a_end < b_end {
                        Ordering::Less
                    } else if a_end > b_end {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                }
            },
        );
        Self { source_text, fixes }
    }

    pub fn fix(&self) -> Cow<'a, str> {
        if self.fixes.is_empty() {
            Cow::Borrowed(self.source_text)
        } else {
            let source_text = self.source_text;
            let mut output = String::with_capacity(source_text.len());
            // To record the position of the last fix.
            let mut last_pos = 0;
            self.fixes.iter().for_each(|Fix { content, span }| {
                let start = span.start;
                let end = span.end;
                if start > end {
                    return;
                }
                // Current fix may conflict with the last fix, so let's skip it.
                if start < last_pos {
                    return;
                }

                let offset = last_pos.max(0) as usize;
                output.push_str(&source_text[offset..start as usize]);
                output.push_str(content);
                last_pos = span.end;
            });

            let offset = last_pos.max(0) as usize;
            output.push_str(&source_text[offset..]);

            return Cow::Owned(output);
        }
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use oxc_ast::Span;

    use super::Fixer;
    use crate::autofix::Fix;

    const TEST_CODE: &str = "var answer = 6 * 7";
    const INSERT_AT_END: Fix =
        Fix { span: Span { start: 18, end: 18 }, content: Cow::Borrowed("// end") };
    const INSERT_AT_START: Fix =
        Fix { span: Span { start: 0, end: 0 }, content: Cow::Borrowed("// start") };
    const INSERT_AT_MIDDLE: Fix =
        Fix { span: Span { start: 13, end: 13 }, content: Cow::Borrowed("5 *") };
    const REVERSE_RANGE: Fix = Fix { span: Span { start: 3, end: 0 }, content: Cow::Borrowed(" ") };

    #[test]
    fn insert_at_the_end() {
        let fixer = Fixer::new(TEST_CODE, vec![INSERT_AT_END]);
        assert_eq!(fixer.fix(), TEST_CODE.to_string() + INSERT_AT_END.content.as_ref());
    }

    #[test]
    fn insert_at_the_beginning() {
        let fixer = Fixer::new(TEST_CODE, vec![INSERT_AT_START]);
        assert_eq!(fixer.fix(), INSERT_AT_START.content.to_string() + TEST_CODE);
    }

    #[test]
    fn insert_at_the_middle() {
        let fixer = Fixer::new(TEST_CODE, vec![INSERT_AT_MIDDLE]);
        assert_eq!(
            fixer.fix(),
            TEST_CODE.replace("6 *", &format!("{}{}", INSERT_AT_MIDDLE.content, "6 *"))
        );
    }

    #[test]
    fn insert_at_the_beginning_middle_end() {
        let fixer = Fixer::new(TEST_CODE, vec![INSERT_AT_MIDDLE, INSERT_AT_START, INSERT_AT_END]);
        assert_eq!(
            fixer.fix(),
            format!(
                "{}{}{}",
                INSERT_AT_START.content,
                TEST_CODE.replace("6 *", &format!("{}{}", INSERT_AT_MIDDLE.content, "6 *")),
                INSERT_AT_END.content
            )
        );
    }

    #[test]
    fn ignore_reverse_range() {
        let fixer = Fixer::new(TEST_CODE, vec![REVERSE_RANGE]);
        assert_eq!(fixer.fix(), TEST_CODE)
    }
}
