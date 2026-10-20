use oxc_htmlish_parser::Dialect;

use crate::HtmlishVariant;

/// What the variant changes in the shared rules, resolved once per document.
#[derive(Clone, Copy)]
pub struct DialectRules {
    /// Known attribute names match case-insensitively (Prettier's HTML parser lowercases them;
    /// a Vue file read as HTML too, Angular never).
    pub names_ignore_case: bool,
    /// Vue's compiler drops whitespace with a line break between two elements and keeps a same-line one as a space.
    pub keeps_element_gap_kind: bool,
    /// Angular: control flow blocks can occur (checked before looking at a node's kind).
    pub has_blocks: bool,
    /// `<!DOCTYPE html>` prints `<!doctype html>`: everywhere but XHTML, where an XML parser needs `<!DOCTYPE`
    /// (DIVERGENCES.md#doctype-case).
    pub lowercase_doctype: bool,
    /// A known HTML element's name and its known attributes print lowercase, as Prettier's HTML parser reads them:
    /// in a `.html` file, whose reader is an HTML parser; as written where embedded, in XHTML, Vue and Angular
    /// (DIVERGENCES.md#tag-name-case).
    pub lowercase_known_names: bool,
}

impl DialectRules {
    /// `sfc`: a Vue file read as a single-file component, not as HTML.
    /// `standalone`: a file of its own, not a part embedded in another language.
    pub fn new(variant: HtmlishVariant, sfc: bool, standalone: bool) -> Self {
        let dialect = variant.to_dialect();
        Self {
            names_ignore_case: dialect == Dialect::Html || (dialect == Dialect::Vue && !sfc),
            keeps_element_gap_kind: dialect == Dialect::Vue,
            has_blocks: dialect == Dialect::Angular,
            lowercase_doctype: variant != HtmlishVariant::Xhtml,
            lowercase_known_names: standalone && variant == HtmlishVariant::Html,
        }
    }
}
