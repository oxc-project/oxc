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

    // Consider the following JSDoc:
    //
    // ```
    // /**
    //  * @kind1 com
    //  * ment...
    //  */
    // /** @kind2 com ment */
    // ```
    //
    // In this case, `comment_part.span` will be:
    // - `@kind1`: `com\n * ment...\n * `
    // - `@kind2`: `com ment `
    //
    // The problem is...
    //
    // If passed `Span` for Miette's `Diagnostic` is single line,
    // it will render nice underline for the span range.
    // But if the span is multiline, it will just render arrow mark at the start of each line.
    //
    // ```
    // ╭─▶ * @kind1 com
    // |   * ment...
    // ╰─▶ */
    // ```
    //
    // It's too verbose and may not fit for linter diagnostics.
    //
    // Even if with single line, the underline is not the same as `parsed()` range.
    // `parsed()` does not include leading and trailing whitespaces.
    //
    // To solve these problems, just indicate the trimmed first line of the comment.
    pub fn span_first_line(&self) -> Span {
        // Multiline
        if let Some(first_line_end) = self.raw.find('\n') {
            // -1 for `\n`
            let span_end = first_line_end.checked_sub(1).unwrap_or_default();
            return Span::new(
                self.span.start,
                self.span.start + u32::try_from(span_end).unwrap_or_default(),
            );
        }

        // Single line
        self.span
    }

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

        Self { raw: part_content, span }
    }

    pub fn parsed(&self) -> &str {
        // +1 for `@`
        &self.raw[1..]
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub struct JSDocTagTypeNamePart<'a> {
    raw: &'a str,
    pub span: Span,
}
impl<'a> JSDocTagTypeNamePart<'a> {
    pub fn new(part_content: &'a str, span: Span) -> Self {
        Self { raw: part_content, span }
    }

    pub fn parsed(&self) -> &str {
        self.raw
    }
}

#[cfg(test)]
mod test {
    use super::JSDocCommentPart;
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
            let comment_part = JSDocCommentPart::new(actual, SPAN);
            assert_eq!(comment_part.parsed(), expect);
        }
    }
}
