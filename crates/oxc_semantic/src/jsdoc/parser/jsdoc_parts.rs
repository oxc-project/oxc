use oxc_span::Span;

#[derive(Debug, Clone, Copy)]
pub struct JSDocCommentPart<'a> {
    raw: &'a str,
    pub span: Span,
}
impl<'a> JSDocCommentPart<'a> {
    pub fn new(part_content: &'a str, span: Span) -> Self {
        Self { raw: part_content, span }
    }

    // TODO: span_trimmed

    pub fn parsed(&self) -> String {
        // If single line, there is no leading `*`
        if self.raw.lines().count() == 1 {
            return self.raw.trim().to_string();
        }

        self.raw
            .lines()
            // Trim leading the first `*` in each line
            .map(|line| line.trim().strip_prefix('*').unwrap_or(line).trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// `kind` can be any string like `param`, `type`, `whatever`, ...etc.
#[derive(Debug, Clone, Copy)]
pub struct JSDocTagKindPart<'a> {
    raw: &'a str,
    pub span: Span,
}
impl<'a> JSDocTagKindPart<'a> {
    pub fn new(part_content: &'a str, span: Span) -> Self {
        debug_assert!(part_content.starts_with('@'));
        debug_assert!(part_content.trim() == part_content);

        Self { raw: part_content, span }
    }

    pub fn parsed(&self) -> &str {
        // +1 for `@`
        &self.raw[1..]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JSDocTagTypePart<'a> {
    raw: &'a str,
    pub span: Span,
}
impl<'a> JSDocTagTypePart<'a> {
    pub fn new(part_content: &'a str, span: Span) -> Self {
        debug_assert!(part_content.starts_with('{'));
        debug_assert!(part_content.ends_with('}'));

        Self { raw: part_content, span }
    }

    pub fn parsed(&self) -> &str {
        // +1 for `{`, -1 for `}`
        &self.raw[1..self.raw.len() - 1]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JSDocTagTypeNamePart<'a> {
    raw: &'a str,
    pub span: Span,
}
impl<'a> JSDocTagTypeNamePart<'a> {
    pub fn new(part_content: &'a str, span: Span) -> Self {
        debug_assert!(part_content.trim() == part_content);

        Self { raw: part_content, span }
    }

    pub fn parsed(&self) -> &str {
        self.raw
    }
}

#[cfg(test)]
mod test {
    use super::{JSDocCommentPart, JSDocTagKindPart, JSDocTagTypeNamePart, JSDocTagTypePart};
    use oxc_span::SPAN;

    #[test]
    fn comment_part_parsed() {
        for (actual, expect) in [
            ("", ""),
            ("hello  ", "hello"),
            ("  * single line", "* single line"),
            (" * ", "*"),
            (" * * ", "* *"),
            ("***", "***"),
            (
                "
      trim
    ",
                "trim",
            ),
            (
                "

    ", "",
            ),
            (
                "
                    *
                    *
                    ",
                "",
            ),
            (
                "
     * asterisk
    ",
                "asterisk",
            ),
            (
                "
     * * li
     * * li
    ",
                "* li\n* li",
            ),
            (
                "
    * list
    * list
    ",
                "list\nlist",
            ),
            (
                "
     * * 1
     ** 2
    ",
                "* 1\n* 2",
            ),
            (
                "
    1

    2

    3
                ",
                "1\n2\n3",
            ),
        ] {
            // `Span` is not used in this test
            let comment_part = JSDocCommentPart::new(actual, SPAN);
            assert_eq!(comment_part.parsed(), expect);
        }
    }

    #[test]
    fn kind_part_parsed() {
        for (actual, expect) in [("@foo", "foo"), ("@", "")] {
            // `Span` is not used in this test
            let kind_part = JSDocTagKindPart::new(actual, SPAN);
            assert_eq!(kind_part.parsed(), expect);
        }
    }

    #[test]
    fn type_part_parsed() {
        for (actual, expect) in
            [("{string}", "string"), ("{{x:1}}", "{x:1}"), ("{[1,2,3]}", "[1,2,3]")]
        {
            // `Span` is not used in this test
            let type_part = JSDocTagTypePart::new(actual, SPAN);
            assert_eq!(type_part.parsed(), expect);
        }
    }

    #[test]
    fn type_name_part_parsed() {
        for (actual, expect) in [("foo", "foo"), ("Bar", "Bar")] {
            // `Span` is not used in this test
            let type_name_part = JSDocTagTypeNamePart::new(actual, SPAN);
            assert_eq!(type_name_part.parsed(), expect);
        }
    }
}
