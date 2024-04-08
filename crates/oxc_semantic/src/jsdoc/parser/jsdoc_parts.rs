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

    // For example, `comment_part.span` in the following JSDoc will be:
    // ```
    // /**
    //  * @kind1 bar
    //  * baz...
    //  * @kind2
    //  */
    // ```
    // for `@kind1`: `bar\n * baz...\n * `
    // for `@kind2`: `\n `
    //
    // If passed `Span` for Miette's `Diagnostic` is single line,
    // it will render nice underline for the span range.
    // ```
    //     * @kind bar... @kind2
    //             ------
    // ````
    //
    // But if the span is multiline, it will just render arrow mark at the start of each line.
    // ```
    // ╭─▶ * @kind1 bar
    // ╰─▶ * @kind2
    // ```
    //
    // It's too verbose and may not fit for linter diagnostics.
    // So instead, provide the first line span to indicate the span range.
    pub fn span_first_line(&self) -> Span {
        if let Some(first_line_end) = self.raw.find('\n') {
            // -1 for `\n`
            let span_end = first_line_end.checked_sub(1).unwrap_or_default();
            return Span::new(
                self.span.start,
                self.span.start + u32::try_from(span_end).unwrap_or_default(),
            );
        }

        self.span
    }

    pub fn parsed(&self) -> String {
        let lines = self.raw.lines();

        // If single line, there is no leading `*`
        if lines.clone().count() == 1 {
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
