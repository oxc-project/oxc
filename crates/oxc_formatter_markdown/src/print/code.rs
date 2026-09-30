//! Code blocks: fences are recomputed (the shortest backtick run the content permits, at least 3),
//! indented blocks stay indented.

use cow_utils::CowUtils;

use oxc_formatter_core::{
    Buffer, arena_cow_str,
    builders::{align, exact_line_breaks, hard_line_break, text, token},
    write,
};
use oxc_markdown_parser::{
    ast::{CodeBlock, CodeBlockKind},
    decode,
};

use super::{
    MarkdownFormatter, backticks, format_with, join_pieces, max_run, write_indented_lines,
};

pub fn write_code_block<'a>(code: &'a CodeBlock<'a>, f: &mut MarkdownFormatter<'_, 'a>) {
    let value: &'a str = join_pieces(&code.lines, f);

    match code.kind {
        CodeBlockKind::Indented => {
            write!(
                f,
                align(
                    4,
                    &format_with(|f| {
                        write!(f, token("    "));
                        write_indented_lines(value, f);
                    })
                )
            );
        }
        CodeBlockKind::Fenced { lang, meta, .. } => {
            let (lang, meta) =
                (lang.map(|s| f.context().slice(s)), meta.map(|s| f.context().slice(s)));
            // `~~~` fences normalize to backticks, unless the info string has one (only `~~~` allows it)
            let info_has_backtick = [lang, meta].iter().flatten().any(|part| part.contains('`'));
            let fence: &'a str = if info_has_backtick {
                "~~~"
            } else {
                arena_cow_str(&backticks((max_run(value, b'`') + 1).max(3)), f)
            };
            write!(f, text(fence));
            // `lang` + one space + `meta`, decoded, the whitespace between them normalized
            for (i, part) in [lang, meta].into_iter().flatten().enumerate() {
                if i > 0 {
                    write!(f, token(" "));
                }
                write!(f, text(cooked_info(part, f)));
            }
            write!(f, hard_line_break());
            write_indented_lines(value, f);
            // A trailing blank line in the content (or an empty block's one empty line) must survive:
            // `hard_line_break` would see an empty line and print nothing.
            write!(f, [exact_line_breaks(1), text(fence)]);
        }
    }
}

/// An info string part decoded; a backslash left by decoding is escaped again,
/// or the next parse would apply it to what it protected (`\\[` printed as `\[` decodes to `[`).
fn cooked_info<'a>(part: &'a str, f: &MarkdownFormatter<'_, 'a>) -> &'a str {
    f.allocator().alloc_str(&decode::decode(part, true).cow_replace('\\', "\\\\"))
}
