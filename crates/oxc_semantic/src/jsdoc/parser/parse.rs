use oxc_span::Span;

use super::{
    jsdoc_parts::{JSDocCommentPart, JSDocTagKindPart},
    jsdoc_tag::JSDocTag,
    utils,
};

/// source_text: Inside of /**HERE*/, NOT includes `/**` and `*/`
/// span_start: Global positioned `Span` start for this JSDoc comment
pub fn parse_jsdoc(source_text: &str, jsdoc_span_start: u32) -> (JSDocCommentPart, Vec<JSDocTag>) {
    debug_assert!(!source_text.starts_with("/*"));
    debug_assert!(!source_text.ends_with("*/"));

    // JSDoc consists of comment and tags.
    // - Comment goes first, and tags(`@xxx`) follow
    // - Both can be optional
    // - Each tag is also separated by whitespace + `@`
    let mut comment = None;
    let mut tags = vec![];

    // So, find `@` to split comment and each tag.
    // But `@` can be found inside of `{}` (e.g. `{@see link}`), it should be distinguished.
    let mut in_braces = false;
    // Also, `@` is often found inside of backtick(` or ```), like markdown.
    let mut in_backticks = false;
    let mut comment_found = false;
    // Parser local offsets, not for global span
    let (mut start, mut end) = (0, 0);

    let mut chars = source_text.chars().peekable();
    while let Some(ch) = chars.next() {
        let can_parse = !(in_braces || in_backticks);
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
            '{' => in_braces = true,
            '}' => in_braces = false,
            '@' if can_parse => {
                let part = &source_text[start..end];
                let span = Span::new(
                    jsdoc_span_start + u32::try_from(start).unwrap_or_default(),
                    jsdoc_span_start + u32::try_from(end).unwrap_or_default(),
                );

                if comment_found {
                    tags.push(parse_jsdoc_tag(part, span));
                } else {
                    comment = Some(JSDocCommentPart::new(part, span));
                    comment_found = true;
                }

                // Prepare for the next draft
                start = end;
            }
            _ => {}
        }
        // Update the current draft
        end += ch.len_utf8();
    }

    // If `@` not found, flush the last draft
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

    (
        comment.unwrap_or(JSDocCommentPart::new("", Span::new(jsdoc_span_start, jsdoc_span_start))),
        tags,
    )
}

/// tag_content: Starts with `@`, may be mulitline
fn parse_jsdoc_tag(tag_content: &str, jsdoc_tag_span: Span) -> JSDocTag {
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
