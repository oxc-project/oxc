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
    let mut in_backticks = false;

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
        let can_parse =
            curly_brace_depth == 0 && square_brace_depth == 0 && brace_depth == 0 && !in_backticks;

        match ch {
            // NOTE: For now, only odd backtick(s) are handled.
            // - 1 backtick: inline code
            // - 3, 5, ... backticks: code fence
            // Not so common but technically, major markdown parser can handle 3 or more backticks as code fence.
            // (for nested code blocks)
            // But for now, 4, 6, ... backticks are not handled here to keep things simple...
            '`' => {
                if chars.peek().is_some_and(|&c| c != '`') {
                    in_backticks = !in_backticks;
                }
            }
            '{' => curly_brace_depth += 1,
            '}' => curly_brace_depth = curly_brace_depth.saturating_sub(1),
            '(' => brace_depth += 1,
            ')' => brace_depth = brace_depth.saturating_sub(1),
            '[' => square_brace_depth += 1,
            ']' => square_brace_depth = square_brace_depth.saturating_sub(1),

            '@' if can_parse => {
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
