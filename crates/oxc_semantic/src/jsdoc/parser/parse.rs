use oxc_span::Span;

use super::{
    jsdoc_parts::{JSDocCommentPart, JSDocTagKindPart},
    jsdoc_tag::JSDocTag,
    utils,
};

/// source_text: Inside of /**HERE*/, NOT includes `/**` and `*/`
/// span_start: Global positioned `Span` start for this JSDoc comment
pub fn parse_jsdoc(
    source_text: &str,
    jsdoc_span_start: u32,
) -> (JSDocCommentPart<'_>, Vec<JSDocTag<'_>>) {
    debug_assert!(!source_text.starts_with("/*"));
    debug_assert!(!source_text.ends_with("*/"));

    // JSDoc consists of comment and tags.
    // - Comment goes first, and tags(`@xxx`) follow
    // - Both can be optional
    // - Each tag is also separated by whitespace + `@`
    let mut comment = None;

    // This will collect all the @tags found in the JSDoc
    let mut tags = vec![];

    // Tracks how deeply nested we are inside curly braces `{}`.
    // Used to ignore `@` characters inside objects or inline tag syntax like {@link ...}
    let mut curly_brace_depth: i32 = 0;

    let mut brace_depth: i32 = 0;

    // Tracks nesting inside square brackets `[]`.
    // Used to avoid interpreting `@` inside optional param syntax like `[param=@default]`
    let mut square_brace_depth: i32 = 0;

    // Tracks whether we're currently inside backticks `...`
    // This includes inline code blocks or markdown-style code inside comments.
    // When 0, we're outside backticks. When > 0, stores the count of opening backticks,
    // so we can match the closing sequence (per CommonMark, backtick strings must match).
    let mut backtick_count: u32 = 0;

    // Track whether we're currently inside quotes '...' or "..."
    // This includes package names when doing a @import
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;

    // Tracks whether we're at the logical start of a line.
    // Only `@` at the start of a line (after optional whitespace and `*` markers)
    // should be treated as a new tag. This prevents false positives from `@` in
    // email addresses (e.g. `user@example.com`) or npm scoped packages
    // (e.g. `@vue/shared`) appearing mid-line in tag descriptions.
    let mut at_line_start = true;

    // This flag tells us if we have already found the main comment block.
    // The first part before any @tags is considered the comment. Everything after is a tag.
    let mut comment_found = false;

    // These mark the current span of the "draft" being read (a comment or tag block)
    let (mut start, mut end) = (0, 0);

    // Turn the source into a character iterator we can peek at
    let mut chars = source_text.chars().peekable();

    // Iterate through every character in the input string
    while let Some(ch) = chars.next() {
        // A `@` is only considered the start of a tag if we are not nested inside
        // braces, square brackets, or backtick-quoted sections
        let can_parse = curly_brace_depth == 0
            && square_brace_depth == 0
            && brace_depth == 0
            && backtick_count == 0
            && !in_double_quotes
            && !in_single_quotes;

        match ch {
            // Handle backtick sequences of any length (per CommonMark):
            // - 1 backtick: inline code
            // - 2 backticks: inline code (used to escape backticks inside)
            // - 3+ backticks: code fence (for nested code blocks)
            // Opening and closing sequences must have the same number of backticks.
            '`' => {
                // Count consecutive backticks
                let mut count: u32 = 1;
                while chars.peek() == Some(&'`') {
                    chars.next();
                    end += 1;
                    count += 1;
                }
                if backtick_count == 0 {
                    // Opening a new backtick section
                    backtick_count = count;
                } else if backtick_count == count {
                    // Closing backtick section with matching count
                    backtick_count = 0;
                }
                // Mismatched count inside a backtick section: ignore
            }
            '"' => in_double_quotes = !in_double_quotes,
            '\'' => in_single_quotes = !in_single_quotes,
            '\n' => {
                in_double_quotes = false;
                in_single_quotes = false;
            }
            '{' => curly_brace_depth += 1,
            '}' => curly_brace_depth = curly_brace_depth.saturating_sub(1),
            '(' => brace_depth += 1,
            ')' => brace_depth = brace_depth.saturating_sub(1),
            '[' => square_brace_depth += 1,
            ']' => square_brace_depth = square_brace_depth.saturating_sub(1),

            '@' if can_parse && at_line_start => {
                let part = &source_text[start..end];
                let span = Span::new(
                    jsdoc_span_start + u32::try_from(start).unwrap_or_default(),
                    jsdoc_span_start + u32::try_from(end).unwrap_or_default(),
                );

                if comment_found {
                    // We've already seen the main comment — this is a tag
                    tags.push(parse_jsdoc_tag(part, span));
                } else {
                    // This is the first `@` we've encountered — treat what came before as the comment
                    comment = Some(JSDocCommentPart::new(part, span));
                    comment_found = true;
                }

                start = end;
            }
            _ => {}
        }

        // Update line-start tracking:
        // - `\n` resets to true (new line)
        // - Whitespace and `*` preserve true (JSDoc line leaders like ` * `)
        // - Everything else sets to false
        if ch == '\n' {
            at_line_start = true;
        } else if at_line_start && !matches!(ch, ' ' | '\t' | '\r' | '*') {
            at_line_start = false;
        }

        // Move the `end` pointer forward by the character's length
        end += ch.len_utf8();
    }

    // After the loop ends, we may have one final segment left to capture
    if start != end {
        let part = &source_text[start..end];
        let span = Span::new(
            jsdoc_span_start + u32::try_from(start).unwrap_or_default(),
            jsdoc_span_start + u32::try_from(end).unwrap_or_default(),
        );

        if comment_found {
            tags.push(parse_jsdoc_tag(part, span));
        } else {
            comment = Some(JSDocCommentPart::new(part, span));
        }
    }

    (comment.unwrap_or(JSDocCommentPart::new("", Span::empty(jsdoc_span_start))), tags)
}

/// tag_content: Starts with `@`, may be multiline
fn parse_jsdoc_tag(tag_content: &str, jsdoc_tag_span: Span) -> JSDocTag<'_> {
    debug_assert!(tag_content.starts_with('@'));
    // This surely exists, at least `@` itself
    let (k_start, k_end) = utils::find_token_range(tag_content).unwrap();

    // Kind: @xxx
    let kind = JSDocTagKindPart::new(
        &tag_content[k_start..k_end],
        Span::new(
            jsdoc_tag_span.start + u32::try_from(k_start).unwrap_or_default(),
            jsdoc_tag_span.start + u32::try_from(k_end).unwrap_or_default(),
        ),
    );

    // Body part: the rest of the tag content.
    // Splitter whitespace should be included to distinguish these cases for comment parser.
    // ```
    // /**
    //  * @k * <- should not omit
    //  */
    // /**
    //  * @k
    //  * <- should omit
    //  */
    // ```
    // If not included, both body content will start with `* <- ...` and fails to parse(trim).
    // This is only for comment parser, it will be ignored for type and type name parser.
    let body_content = &tag_content[k_end..];
    let body_span = Span::new(
        jsdoc_tag_span.start + u32::try_from(k_end).unwrap_or_default(),
        jsdoc_tag_span.end,
    );

    JSDocTag::new(kind, body_content, body_span)
}
