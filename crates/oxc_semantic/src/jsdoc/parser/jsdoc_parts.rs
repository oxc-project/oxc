use oxc_span::Span;

/// Used for `JSDoc.comment` and `JSDocTag.comment`
#[derive(Debug, Clone, Copy)]
pub struct JSDocCommentPart<'a> {
    raw: &'a str,
    pub span: Span,
}
impl<'a> JSDocCommentPart<'a> {
    pub fn new(part_content: &'a str, span: Span) -> Self {
        Self { raw: part_content, span }
    }

    // For example, `Span` for the following comment part:
    // ```
    // /**
    //  * @kind1 COMMENT
    //  * WILL BE ...
    //  * @kind2 C2
    //  * @kind3
    //  */
    // ```
    // is ` COMMENT\n * WILL BE ...\n * `.
    //
    // It includes whitespace and line breaks.
    // And it also includes leading `*` prefixes in every line, even in a single line tag.
    // (comment `Span` for `kind2` is ` C2\n * `)
    //
    // Since these are trimmed by `parsed()` output, this raw `Span` may not be suitable for linter diagnostics.
    //
    // And if the passed `Span` for miette diagnostics is multiline,
    // it will just render arrow markers which is not intuitive.
    // (It renders a nice undeline for single line span, but not for multiline)
    // ```
    // ╭─▶ * @kind1 COMMENT
    // │   * WILL BE ...
    // ╰─▶ * @kind2 C2
    // ```
    //
    // So instead, just indicate the first visible line of the comment part.
    // ```
    //     * @kind1 COMMENT
    //              ───────
    //     * WILL BE ...
    //     * @kind2 C2
    // ```
    // It may not be perfect for multiline, but for single line, which is probably the majority, it is enough.
    pub fn span_trimmed_first_line(&self) -> Span {
        if self.raw.trim().is_empty() {
            return Span::new(self.span.start, self.span.start);
        }

        let base_len = self.raw.len();
        if self.raw.lines().count() == 1 {
            let trimmed_start_offset = base_len - self.raw.trim_start().len();
            let trimmed_end_offset = base_len - self.raw.trim_end().len();

            return Span::new(
                self.span.start + u32::try_from(trimmed_start_offset).unwrap_or_default(),
                self.span.end - u32::try_from(trimmed_end_offset).unwrap_or_default(),
            );
        }

        let start_trimmed = self.raw.trim_start();
        let trimmed_start_offset = base_len - start_trimmed.len();
        let trimmed_end_offset = trimmed_start_offset + start_trimmed.find('\n').unwrap_or(0);
        Span::new(
            self.span.start + u32::try_from(trimmed_start_offset).unwrap_or_default(),
            self.span.start + u32::try_from(trimmed_end_offset).unwrap_or_default(),
        )
    }

    /// Returns the content of the comment part without leading `*` in each line.
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

    /// Returns `kind`, it can be any string like `param`, `type`, `whatever`, ...etc.
    pub fn parsed(&self) -> &'a str {
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

    /// Returns the type content without `{` and `}`.
    pub fn parsed(&self) -> &'a str {
        // +1 for `{`, -1 for `}`
        self.raw[1..self.raw.len() - 1].trim()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JSDocTagTypeNamePart<'a> {
    raw: &'a str,
    pub span: Span,
    pub optional: bool,
    pub default: bool,
}
impl<'a> JSDocTagTypeNamePart<'a> {
    pub fn new(part_content: &'a str, span: Span) -> Self {
        debug_assert!(part_content.trim() == part_content);

        let optional = part_content.starts_with('[') && part_content.ends_with(']');
        let default = optional && part_content.contains('=');

        Self { raw: part_content, span, optional, default }
    }

    /// Returns the type name itself.
    /// `.raw` may be like `[foo = var]`, so extract the name
    pub fn parsed(&self) -> &'a str {
        if self.optional {
            let inner = self.raw.trim_start_matches('[').trim_end_matches(']').trim();
            return inner.split_once('=').map_or(inner, |(v, _)| v.trim());
        }

        self.raw
    }
}

#[cfg(test)]
mod test {
    use oxc_span::{Span, SPAN};

    use super::{JSDocCommentPart, JSDocTagKindPart, JSDocTagTypeNamePart, JSDocTagTypePart};

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
    fn comment_part_span_trimmed() {
        for (actual, expect) in [
            ("", ""),
            ("\n", ""),
            ("\n\n\n", ""),
            ("...", "..."),
            ("c1\n", "c1"),
            ("\nc2\n", "c2"),
            (" c 3\n", "c 3"),
            ("\nc4\n * ...\n ", "c4"),
            (
                "
 extra text
*
",
                "extra text",
            ),
            (
                "
 * foo
 * bar
",
                "* foo",
            ),
        ] {
            let comment_part =
                JSDocCommentPart::new(actual, Span::new(0, u32::try_from(actual.len()).unwrap()));
            assert_eq!(comment_part.span_trimmed_first_line().source_text(actual), expect);
        }
    }

    #[test]
    fn kind_part_parsed() {
        for (actual, expect) in [("@foo", "foo"), ("@", ""), ("@かいんど", "かいんど")] {
            // `Span` is not used in this test
            let kind_part = JSDocTagKindPart::new(actual, SPAN);
            assert_eq!(kind_part.parsed(), expect);
        }
    }

    #[test]
    fn type_part_parsed() {
        for (actual, expect) in [
            ("{}", ""),
            ("{-}", "-"),
            ("{string}", "string"),
            ("{ string}", "string"),
            ("{ bool  }", "bool"),
            ("{{x:1}}", "{x:1}"),
            ("{[1,2,3]}", "[1,2,3]"),
        ] {
            // `Span` is not used in this test
            let type_part = JSDocTagTypePart::new(actual, SPAN);
            assert_eq!(type_part.parsed(), expect);
        }
    }

    #[test]
    fn type_name_part_parsed() {
        for (actual, expect) in [
            ("foo", "foo"),
            ("Bar", "Bar"),
            ("変数", "変数"),
            ("[opt]", "opt"),
            ("[ opt2 ]", "opt2"),
            ("[def1 = [ 1 ]]", "def1"),
            (r#"[def2 = "foo bar"]"#, "def2"),
        ] {
            // `Span` is not used in this test
            let type_name_part = JSDocTagTypeNamePart::new(actual, SPAN);
            assert_eq!(type_name_part.parsed(), expect);
        }
    }
}
