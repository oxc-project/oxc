use crate::HtmlishVariant;

/// Which language `{{ }}` and code in attribute values are in (Vue's prints by `print/vue.rs`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TemplateExpressions {
    None,
    Vue,
    Angular,
}

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
    /// Angular: the content of an element with `i18n`, and an attribute's value marked by `i18n-x`, print as written (`Tree::keeps_angular_content`, `Tree::keeps_angular_value`).
    pub keeps_angular_content: bool,
    /// Vue: the root's elements but `html` are single-file component blocks (`<template>`, `<script>`, `<i18n>`),
    /// in a file read as HTML too, as Prettier reads them.
    pub vue_sfc: bool,
    /// The language of `{{ }}` and the attribute values that are code.
    pub template_expressions: TemplateExpressions,
    /// `<!DOCTYPE html>` prints `<!doctype html>` in a standalone HTML document, whose reader is an HTML parser
    /// (a Vue file read as HTML too), and an Angular template (a `.html` file to Prettier);
    /// as written where embedded, in XHTML and a Vue single-file component
    /// (DIVERGENCES.md#doctype-case).
    pub lowercase_doctype: bool,
    /// A known HTML element's name and its known attributes print lowercase, as Prettier's HTML parser reads them:
    /// in a `.html` file and a Vue file read as HTML, whose reader is an HTML parser;
    /// as written where embedded, in XHTML, a Vue SFC and Angular (DIVERGENCES.md#tag-name-case).
    pub lowercase_known_names: bool,
}

impl DialectRules {
    /// `sfc`: a Vue file read as a single-file component, not as HTML.
    /// `standalone`: a file of its own, not a part embedded in another language.
    pub fn new(variant: HtmlishVariant, sfc: bool, standalone: bool) -> Self {
        Self {
            names_ignore_case: matches!(variant, HtmlishVariant::Html | HtmlishVariant::Xhtml)
                || (variant == HtmlishVariant::Vue && !sfc),
            keeps_element_gap_kind: variant == HtmlishVariant::Vue,
            has_blocks: variant == HtmlishVariant::Angular,
            keeps_angular_content: variant == HtmlishVariant::Angular,
            vue_sfc: variant == HtmlishVariant::Vue,
            template_expressions: match variant {
                HtmlishVariant::Html | HtmlishVariant::Xhtml => TemplateExpressions::None,
                HtmlishVariant::Vue => TemplateExpressions::Vue,
                HtmlishVariant::Angular => TemplateExpressions::Angular,
            },
            lowercase_doctype: standalone
                && (matches!(variant, HtmlishVariant::Html | HtmlishVariant::Angular)
                    || (variant == HtmlishVariant::Vue && !sfc)),
            lowercase_known_names: standalone
                && (variant == HtmlishVariant::Html || (variant == HtmlishVariant::Vue && !sfc)),
        }
    }
}
