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

    // Track indentation after the `*` prefix to detect 4-space indented code blocks.
    // When a line has 4+ spaces of content indent (after `* `), `@` on that line
    // should not be treated as a tag marker — it's inside an indented code block.
    let mut line_seen_star = false;
    let mut spaces_after_star: u32 = 0;

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
            '`' if !in_single_quotes && !in_double_quotes => {
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
            // Inside backtick-quoted sections (inline code / code fences),
            // all bracket/quote tracking is suspended. Characters like `{`, `}`,
            // `"`, etc. inside code literals are not syntactic and must not
            // affect `can_parse` — otherwise a stray `{` in inline code
            // (e.g. `` `{` ``) would prevent subsequent `@` tags from being
            // recognized, causing them to merge into the description.
            '"' if backtick_count == 0 && !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
            }
            '\'' if backtick_count == 0 && !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
            }
            '\n' => {
                in_double_quotes = false;
                in_single_quotes = false;
            }
            '{' if backtick_count == 0 && !in_double_quotes && !in_single_quotes => {
                curly_brace_depth += 1;
            }
            '}' if backtick_count == 0 && !in_double_quotes && !in_single_quotes => {
                curly_brace_depth = (curly_brace_depth - 1).max(0);
            }
            '(' if backtick_count == 0 && !in_double_quotes && !in_single_quotes => {
                brace_depth += 1;
            }
            ')' if backtick_count == 0 && !in_double_quotes && !in_single_quotes => {
                brace_depth = (brace_depth - 1).max(0);
            }
            '[' if backtick_count == 0 && !in_double_quotes && !in_single_quotes => {
                square_brace_depth += 1;
            }
            ']' if backtick_count == 0 && !in_double_quotes && !in_single_quotes => {
                square_brace_depth = (square_brace_depth - 1).max(0);
            }

            '@' if can_parse
                && at_line_start
                && !is_indented_code_block(line_seen_star, spaces_after_star) =>
            {
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
            line_seen_star = false;
            spaces_after_star = 0;
        } else if at_line_start {
            if ch == '*' {
                line_seen_star = true;
                spaces_after_star = 0;
            } else if matches!(ch, ' ' | '\t' | '\r') {
                if line_seen_star {
                    spaces_after_star += 1;
                }
            } else {
                at_line_start = false;
            }
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

/// Check if the current line position represents a 4-space indented code block.
/// In CommonMark, 4+ spaces of indent from the content margin indicate an indented
/// code block. In JSDoc, the content margin is after `* ` (star + one conventional space),
/// so we need 5+ spaces after `*` (1 conventional + 4 for code block).
/// If no `*` was seen on this line, this returns false (conservative).
fn is_indented_code_block(line_seen_star: bool, spaces_after_star: u32) -> bool {
    // 5 = 1 conventional space after `*` + 4 for indented code block
    line_seen_star && spaces_after_star >= 5
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: parse JSDoc and return tag kind strings.
    fn tag_kinds(source: &str) -> Vec<String> {
        let (_, tags) = parse_jsdoc(source, 0);
        tags.iter().map(|t| t.kind.parsed().to_string()).collect()
    }

    #[test]
    fn backtick_inside_quotes_does_not_prevent_tag_split() {
        // Bug A: backtick inside single quotes toggled backtick_count,
        // preventing @returns from being recognized as a separate tag.
        let src = " \n * @param {\"'\" | '\"' | '`'} string_start_char desc\n * @returns {number} The index";
        let kinds = tag_kinds(src);
        assert_eq!(kinds, vec!["param", "returns"]);
    }

    #[test]
    fn extra_closing_brace_does_not_prevent_tag_split() {
        // Bug B: extra `}` after param name decremented curly_brace_depth
        // below 0 (i32 saturating_sub), preventing next @param recognition.
        let src = " \n * @param {AST.SvelteElement | AST.RegularElement} node}\n * @param {{ stop: () => void }} context";
        let kinds = tag_kinds(src);
        assert_eq!(kinds, vec!["param", "param"]);
    }

    #[test]
    fn normal_tags_still_split_correctly() {
        let src = " \n * @param {string} name The name\n * @returns {boolean} True if valid";
        let kinds = tag_kinds(src);
        assert_eq!(kinds, vec!["param", "returns"]);
    }

    #[test]
    fn inline_link_does_not_split_tag() {
        // {@link ...} should NOT start a new tag
        let src = " \n * @param {string} name See {@link Foo} for details\n * @returns {void}";
        let kinds = tag_kinds(src);
        assert_eq!(kinds, vec!["param", "returns"]);
    }

    #[test]
    fn at_sign_mid_line_does_not_split_tag() {
        // @ in email or scoped package mid-line should not split
        let src = " \n * @param {string} email user@example.com address\n * @returns {void}";
        let kinds = tag_kinds(src);
        assert_eq!(kinds, vec!["param", "returns"]);
    }

    #[test]
    fn code_fence_does_not_prevent_tag_split() {
        // Backtick code fence should not affect tag splitting after the fence
        let src = " \n * @example\n * ```\n * const x = 1;\n * ```\n * @returns {void}";
        let kinds = tag_kinds(src);
        assert_eq!(kinds, vec!["example", "returns"]);
    }

    #[test]
    fn braces_inside_quotes_do_not_prevent_tag_split() {
        // Braces inside quoted strings in description text should not affect
        // curly_brace_depth tracking. This caused @param to be merged into the
        // description when the description contained unbalanced braces in quotes.
        let src = " \n * \"props\" of the form \"{ [key: string]: { type?: \"String\" | \"Object\" }\"\n * @param {null} node\n * @returns {never}";
        let kinds = tag_kinds(src);
        assert_eq!(kinds, vec!["param", "returns"]);
    }

    #[test]
    fn indented_code_block_at_sign_does_not_split_tag() {
        // `@` inside a 4-space indented code block (after `* `) should NOT be
        // treated as a tag marker. 5 spaces after `*` = 1 conventional + 4 for code block.
        let src = " \n * @deprecated\n *     @myDecorator\n *     class Foo {}\n * @type {string}";
        let kinds = tag_kinds(src);
        // @myDecorator is inside a code block, so only @deprecated and @type are real tags
        assert_eq!(kinds, vec!["deprecated", "type"]);
    }

    #[test]
    fn normal_indent_at_sign_still_splits() {
        // `@` with less than 4-space indent (3 spaces after conventional) should still split
        let src = " \n * @deprecated\n *    @type {string}";
        let kinds = tag_kinds(src);
        // 4 spaces after `*` = 1 conventional + 3 indent → NOT an indented code block
        assert_eq!(kinds, vec!["deprecated", "type"]);
    }
}
