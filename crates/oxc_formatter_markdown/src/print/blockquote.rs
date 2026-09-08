//! Blockquotes: `> ` before the first line, then before every line the body breaks into
//! (`prefix_align`; a blank line inside prints as `>`).

use oxc_formatter_core::{Buffer, builders::prefix_align, write};
use oxc_markdown_parser::ast::Blockquote;

use super::{MarkdownFormatter, block, format_with};

pub fn write_blockquote<'a>(quote: &'a Blockquote<'a>, f: &mut MarkdownFormatter<'_, 'a>) {
    // An empty one is `>`: the printer never trims a trailing space
    if quote.children.is_empty() {
        write!(f, ">");
        return;
    }
    let body = format_with(|f| block::write_blocks(&quote.children, block::Parent::Container, f));
    write!(f, ["> ", prefix_align(&"> ", &body)]);
}
