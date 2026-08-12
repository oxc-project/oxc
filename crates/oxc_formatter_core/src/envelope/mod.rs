//! Front matter WRITING shared by document-envelope hosts (CSS today, Markdown later);
//! detection stays in [`crate::spec::parse_front_matter`] (pure text mechanics),
//! this module owns the IR side: dispatching the block's body through the session and composing the frame.
//!
//! Core encodes no language meaning: a host opts in by calling, and its embeddable set arrives as data.
//! That set is load-bearing: dispatching every resolved language would let e.g. `---json` format through a native branch,
//! where Prettier keeps it verbatim.

use oxc_allocator::ArenaVec;

use crate::{
    Buffer as _, BufferExtensions as _, FormatContext, FormatElement, Formatter, TailwindCollector,
    builders::{hard_line_break, text},
    dispatch_fragment_ir,
    spec::FrontMatter,
    write,
};

/// Writes a front matter block.
///
/// The body dispatches through the session when its resolved language is in `embeddable_languages`,
/// the whole block stays verbatim otherwise.
/// Membership means "gets the frame treatment", not "gets formatted":
/// a member language without a serving formatter keeps its body verbatim (`PreserveOriginal`),
/// but its EMPTY block still normalizes through the frame.
/// Never fails; any refusal (`PreserveOriginal`, operational error, non-embeddable language) keeps the block's bytes as-is
/// while the host document still formats.
///
/// The composed shape (Prettier `embed.js` + its css/markdown printers):
/// opening delimiter with the explicit language re-emitted (`---yaml`),
/// the body IR between hardlines, then the closing delimiter (a `...` closing stays `...`).
/// Spacing between the block and the host's body stays the caller's concern.
pub fn write_front_matter<'a, C>(
    fm: &FrontMatter<'a>,
    embeddable_languages: &[&str],
    f: &mut Formatter<'_, 'a, C>,
) where
    C: FormatContext + TailwindCollector,
{
    let language = fm.language();
    if embeddable_languages.contains(&language) {
        let body = fm.value.trim();
        if body.is_empty() {
            write_frame(fm, None, f);
            return;
        }
        // The body is a Fragment:
        // front matter inside front matter must never acquire envelope semantics
        // (generic FM recursion).
        if let Some(ir) = dispatch_fragment_ir(f, language, body, None) {
            write_frame(fm, Some(ir), f);
            return;
        }
        // `PreserveOriginal` or an operational error:
        // fall through to the verbatim block below.
    }
    write!(f, text(fm.raw));
}

fn write_frame<'a, C: FormatContext>(
    fm: &FrontMatter<'a>,
    body: Option<ArenaVec<'a, FormatElement<'a>>>,
    f: &mut Formatter<'_, 'a, C>,
) {
    write!(f, [text(fm.start_delimiter), fm.explicit_language.map(text)]);
    if let Some(ir) = body {
        write!(f, hard_line_break());
        f.write_elements(ir);
    }
    write!(f, [hard_line_break(), text(fm.end_delimiter)]);
}
