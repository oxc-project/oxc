//! Code blocks: fences are recomputed (the shortest backtick or tilde run the content permits, at least 3),
//! indented blocks stay indented.

use cow_utils::CowUtils;
use rustc_hash::FxHashSet;

use oxc_allocator::ArenaVec;
use oxc_formatter_core::{
    Buffer, BufferExtensions, DispatchRequest, Document, FormatElement, FormatOptions, InputKind,
    Interned, PrintWidth, arena_cow_str,
    builders::{
        FormatOnce, exact_line_breaks, hard_line_break, mark_as_root, space_align, text, token,
    },
    dispatch_ir, map_text_in_ir, push_text_with_literal_lines, write,
};
use oxc_markdown_parser::{
    ast::{CodeBlock, CodeBlockKind},
    decode,
};

use super::{
    MarkdownFormatter, fence_run, format_with, join_pieces, max_run, write_indented_lines,
};

pub fn write_code_block<'a>(code: &'a CodeBlock<'a>, f: &mut MarkdownFormatter<'_, 'a>) {
    let value: &'a str = join_pieces(&code.lines, f);

    match code.kind {
        CodeBlockKind::Indented => {
            write!(
                f,
                space_align(
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
            // Backtick fences; tildes inside a JS template literal
            // or when the info string has a backtick (only a `~~~` fence allows one)
            let info_has_backtick = [lang, meta].iter().flatten().any(|part| part.contains('`'));
            let fence_char =
                if info_has_backtick || f.context().in_js_template() { b'~' } else { b'`' };

            let embedded = lang.and_then(|lang| format_embedded(lang, meta, value, fence_char, f));
            let content_run = embedded.as_ref().map_or_else(|| max_run(value, fence_char), |e| e.1);
            let fence: &'a str = arena_cow_str(&fence_run(fence_char, (content_run + 1).max(3)), f);
            // Single-use, so the child IR moves into the buffer instead of being cloned
            let fenced = FormatOnce::new(|f: &mut MarkdownFormatter<'_, 'a>| {
                write!(f, text(fence));
                // `lang` + one space + `meta`, decoded, the whitespace between them normalized
                for (i, part) in [lang, meta].into_iter().flatten().enumerate() {
                    if i > 0 {
                        write!(f, token(" "));
                    }
                    write!(f, text(cooked_info(part, f)));
                }
                write!(f, hard_line_break());
                if let Some((ir, _)) = embedded {
                    f.write_elements(ir);
                    write!(f, [hard_line_break(), text(fence)]);
                } else {
                    write_indented_lines(value, f);
                    // A trailing blank line in the content (or an empty block's one empty line) must survive:
                    // `hard_line_break` would see an empty line and print nothing.
                    write!(f, [exact_line_breaks(1), text(fence)]);
                }
            });
            // The child's literal lines (a template literal, a block scalar) stay relative to the fence,
            // not column 0 (inside a blockquote, that would lose the `> `).
            write!(f, mark_as_root(&fenced));
        }
    }
}

/// Parent→child context for a fenced code block dispatched from Markdown:
/// a fact about the parent, not a pair's data, since the host resolves the block's language.
///
/// Sent to every child language as `DispatchRequest::parent_context`;
/// the host translates it into the pair's data once the child language is known
/// (e.g. JS's `JsEmbeddedIn`: a lone JSX element prints without its semicolon).
pub struct XxxInMarkdownCodeBlock {
    /// This host is embedded in a JS template literal,
    /// so a Markdown child prints `~` fences too.
    pub in_js_template: bool,
}

/// Formats the content through the session's dispatcher;
/// returns the IR and the longest `fence_char` run it prints (the fence must outnumber it).
/// `None` keeps the block verbatim: no dispatcher, an unknown language, a failed parse,
/// or a block whose line numbers carry meaning (`apps/oxfmt/DIVERGENCES.md#line-ranged-code-block`).
fn format_embedded<'a>(
    lang: &str,
    meta: Option<&str>,
    value: &str,
    fence_char: u8,
    f: &MarkdownFormatter<'_, 'a>,
) -> Option<(ArenaVec<'a, FormatElement<'a>>, usize)> {
    if !f.session().has_dispatcher() || value.trim().is_empty() || meta.is_some_and(has_line_ranges)
    {
        return None;
    }
    let language = decode::decode(lang, true);
    let ir = dispatch_ir(
        f,
        DispatchRequest {
            language: &language,
            text: value,
            input_kind: InputKind::VirtualDocument,
            parent_context: Some(&XxxInMarkdownCodeBlock {
                in_js_template: f.context().in_js_template(),
            }),
        },
    )?;

    let (has_newline, has_fence_char) = scan_texts(&ir, fence_char);
    // Newline inside a text (a template literal, a block comment) becomes a literal line,
    // which `mark_as_root` keeps at the fence's column
    let ir = if has_newline {
        let indent_width = f.options().indent_width();
        map_text_in_ir(&ir, f.allocator(), &mut |text, out| {
            text.contains('\n') && {
                push_text_with_literal_lines(out, text, indent_width);
                true
            }
        })
    } else {
        ir
    };
    if !has_fence_char {
        return Some((ir, 0));
    }

    // A run may span texts, so count it on the printed content.
    // The child's `TailwindClass` elements index the session's scope; unsorted is enough to count.
    let classes = f.session().unsorted_tailwind_classes();
    let probe = Document::new(ArenaVec::from_iter_in(ir.iter().cloned(), &f.allocator()), classes);
    // Unlimited width, as Prettier's: a run is counted on the content, not on how it breaks
    let options = f.options().as_print_options().with_print_width(PrintWidth::new(u32::MAX));
    let printed = probe.print(value.len(), options).ok()?;
    let run = max_run(printed.as_code(), fence_char);
    Some((ir, run))
}

/// Whether any text of the IR holds a newline / the fence character;
/// a shared `Interned` subtree is walked once.
#[expect(clippy::mutable_key_type)] // `Interned` hashes by pointer identity
fn scan_texts(ir: &[FormatElement<'_>], fence_char: u8) -> (bool, bool) {
    fn walk<'a>(
        elements: &[FormatElement<'a>],
        fence_char: u8,
        seen: &mut FxHashSet<Interned<'a>>,
        found: &mut (bool, bool),
    ) {
        for element in elements {
            match element {
                FormatElement::Text { text, .. } | FormatElement::Token { text } => {
                    found.0 |= text.contains('\n');
                    found.1 |= text.as_bytes().contains(&fence_char);
                }
                FormatElement::BestFitting(best_fitting) => {
                    for variant in best_fitting.variants() {
                        walk(variant, fence_char, seen, found);
                    }
                }
                FormatElement::Interned(interned) if seen.insert(interned.clone()) => {
                    walk(interned, fence_char, seen, found);
                }
                _ => {}
            }
        }
    }
    let mut found = (false, false);
    walk(ir, fence_char, &mut FxHashSet::default(), &mut found);
    found
}

/// Whether the meta addresses lines by number (`{1,3-5}`, VitePress / Docusaurus line highlighting),
/// which formatting the content would shift.
fn has_line_ranges(meta: &str) -> bool {
    meta.split('{').skip(1).filter_map(|rest| rest.split_once('}')).any(|(ranges, _)| {
        ranges.bytes().any(|b| b.is_ascii_digit())
            && ranges.bytes().all(|b| b.is_ascii_digit() || matches!(b, b',' | b'-' | b' '))
    })
}

/// An info string part decoded; a backslash left by decoding is escaped again,
/// or the next parse would apply it to what it protected (`\\[` printed as `\[` decodes to `[`).
fn cooked_info<'a>(part: &'a str, f: &MarkdownFormatter<'_, 'a>) -> &'a str {
    f.allocator().alloc_str(&decode::decode(part, true).cow_replace('\\', "\\\\"))
}
