use oxc_span::Span;

use crate::jsdoc::parser::utils;

use super::jsdoc_parts::{
    JSDocCommentPart, JSDocTagKindPart, JSDocTagTypeNamePart, JSDocTagTypePart,
};

// Initially, I attempted to parse into specific structures such as:
// - `@param {type} name comment`: `JSDocParameterTag { type, name, comment }`
// - `@returns {type} comment`: `JSDocReturnsTag { type, comment }`
// - `@whatever comment`: `JSDocUnknownTag { comment }`
// - etc...
//
// However, I discovered that some use cases, like `eslint-plugin-jsdoc`, provide an option to create an alias for the tag kind.
// .e.g. Preferring `@foo` instead of `@param`
//
// This means that:
// - We cannot parse a tag exactly as it was written
// - We cannot assume that `@param` will always map to `JSDocParameterTag`
//
// Therefore, I decided to provide a generic structure with helper methods to parse the tag according to the needs.
//
// I also considered providing an API with methods like `as_param() -> JSDocParameterTag` or `as_return() -> JSDocReturnTag`, etc.
//
// However:
// - There are many kinds of tags, but most of them have a similar structure
// - JSDoc is not a strict format; it's just a comment
// - Users can invent their own tags like `@whatever {type}` and may want to parse its type
//
// As a result, I ended up providing helper methods that are fit for purpose.

#[derive(Debug, Clone)]
pub struct JSDocTag<'a> {
    pub span: Span,
    pub kind: JSDocTagKindPart<'a>,
    body_raw: &'a str,
    body_span: Span,
}

impl<'a> JSDocTag<'a> {
    pub fn new(kind: JSDocTagKindPart<'a>, body_content: &'a str, body_span: Span) -> JSDocTag<'a> {
        Self { span: kind.span.merge(&body_span), kind, body_raw: body_content, body_span }
    }

    /// Use for various simple tags like `@access`, `@deprecated`, ...etc.
    /// Comment can be multiline.
    ///
    /// Variants:
    /// ```
    /// @kind comment
    /// @kind
    /// ```
    pub fn comment(&self) -> JSDocCommentPart<'a> {
        JSDocCommentPart::new(self.body_raw, self.body_span)
    }

    /// Use for `@type`, `@satisfies`, ...etc.
    ///
    /// Variants:
    /// ```
    /// @kind {type}
    /// @kind
    /// ```
    pub fn r#type(&self) -> Option<JSDocTagTypePart<'a>> {
        utils::find_type_range(self.body_raw).map(|(t_start, t_end)| {
            JSDocTagTypePart::new(
                &self.body_raw[t_start..t_end],
                Span::new(
                    self.body_span.start + u32::try_from(t_start).unwrap_or_default(),
                    self.body_span.start + u32::try_from(t_end).unwrap_or_default(),
                ),
            )
        })
    }

    /// Use for `@yields`, `@returns`, ...etc.
    /// Comment can be multiline.
    ///
    /// Variants:
    /// ```
    /// @kind {type} comment
    /// @kind {type}
    /// @kind comment
    /// @kind
    /// ```
    pub fn type_comment(&self) -> (Option<JSDocTagTypePart<'a>>, JSDocCommentPart<'a>) {
        let (type_part, comment_part) = match utils::find_type_range(self.body_raw) {
            Some((t_start, t_end)) => {
                // Include whitespace for comment trimming
                let c_start = t_end;
                (
                    Some(JSDocTagTypePart::new(
                        &self.body_raw[t_start..t_end],
                        Span::new(
                            self.body_span.start + u32::try_from(t_start).unwrap_or_default(),
                            self.body_span.start + u32::try_from(t_end).unwrap_or_default(),
                        ),
                    )),
                    JSDocCommentPart::new(
                        &self.body_raw[c_start..],
                        Span::new(
                            self.body_span.start + u32::try_from(c_start).unwrap_or_default(),
                            self.body_span.end,
                        ),
                    ),
                )
            }
            None => (None, JSDocCommentPart::new(self.body_raw, self.body_span)),
        };

        (type_part, comment_part)
    }

    /// Use for `@param`, `@property`, `@typedef`, ...etc.
    /// Comment can be multiline.
    ///
    /// Variants:
    /// ```
    /// @kind {type} name comment
    /// @kind {type} name
    /// @kind {type}
    /// @kind name comment
    /// @kind name
    /// @kind
    /// ```
    pub fn type_name_comment(
        &self,
    ) -> (Option<JSDocTagTypePart<'a>>, Option<JSDocTagTypeNamePart<'a>>, JSDocCommentPart<'a>)
    {
        let (type_part, name_comment_content, span_start) =
            match utils::find_type_range(self.body_raw) {
                Some((t_start, t_end)) => {
                    // There may be, or may not be whitespace
                    // e.g. `@kind {type}name_comment`
                    let c_start = t_end;
                    (
                        Some(JSDocTagTypePart::new(
                            &self.body_raw[t_start..t_end],
                            Span::new(
                                self.body_span.start + u32::try_from(t_start).unwrap_or_default(),
                                self.body_span.start + u32::try_from(t_end).unwrap_or_default(),
                            ),
                        )),
                        &self.body_raw[c_start..],
                        self.body_span.start + u32::try_from(c_start).unwrap_or_default(),
                    )
                }
                None => (None, self.body_raw, self.body_span.start),
            };

        let (name_part, comment_part) = match utils::find_type_name_range(name_comment_content) {
            Some((n_start, n_end)) => {
                // Include whitespace for comment trimming
                let c_start = n_end;
                (
                    Some(JSDocTagTypeNamePart::new(
                        &name_comment_content[n_start..n_end],
                        Span::new(
                            span_start + u32::try_from(n_start).unwrap_or_default(),
                            span_start + u32::try_from(n_end).unwrap_or_default(),
                        ),
                    )),
                    JSDocCommentPart::new(
                        &name_comment_content[c_start..],
                        Span::new(
                            span_start + u32::try_from(c_start).unwrap_or_default(),
                            self.body_span.end,
                        ),
                    ),
                )
            }
            None => (
                None,
                JSDocCommentPart::new(
                    name_comment_content,
                    Span::new(span_start, self.body_span.end),
                ),
            ),
        };

        (type_part, name_part, comment_part)
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use crate::{Semantic, SemanticBuilder};

    fn build_semantic<'a>(allocator: &'a Allocator, source_text: &'a str) -> Semantic<'a> {
        let source_type = SourceType::default();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        let program = allocator.alloc(ret.program);
        let semantic = SemanticBuilder::new().with_build_jsdoc(true).build(program).semantic;
        semantic
    }

    #[test]
    fn jsdoc_tag_span() {
        for (source_text, tag_span_text) in [
            (
                "
                /**
                 * multi
                 * line @k1 c1
                 */
                ",
                "@k1 c1\n                 ",
            ),
            (
                "
                /**
                 * @k2 c2a
                 * c2b
                 *
                 */
                ",
                "@k2 c2a\n                 * c2b\n                 *\n                 ",
            ),
            (
                "
                /**
                 * multi
                 * @k3 c3
                 */
                ",
                "@k3 c3\n                 ",
            ),
            ("/** single line @k4 c4 */", "@k4 c4 "),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let tag = jsdocs.next().unwrap().tags().first().unwrap();
            assert_eq!(tag.span.source_text(source_text), tag_span_text);
        }
    }

    #[test]
    fn jsdoc_tag_kind() {
        for (source_text, tag_kind, tag_kind_span_text) in [
            ("/** single line @k1 c1 */", "k1", "@k1"),
            ("/** single line @k2*/", "k2", "@k2"),
            (
                "/**
             * multi
             * line
             * @k3 c3a
             * c3b
             */",
                "k3",
                "@k3",
            ),
            (
                "/**
             * multi
             * line @k4
             */",
                "k4",
                "@k4",
            ),
            (" /**@*/ ", "", "@"),
            (" /**@@*/ ", "", "@"),
            (" /** @あいう え */ ", "あいう", "@あいう"),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let tag = jsdocs.next().unwrap().tags().first().unwrap();
            assert_eq!(tag.kind.parsed(), tag_kind);
            assert_eq!(tag.kind.span.source_text(source_text), tag_kind_span_text);
        }
    }

    #[test]
    fn jsdoc_tag_comment() {
        for (source_text, parsed_comment_part) in [
            ("/** single line @k1 c1 */", ("c1", " c1 ")),
            ("/** single line @k2*/", ("", "")),
            (
                "/**
             * multi
             * line
             * @k3 c3a
             * c3b
             */",
                ("c3a\nc3b", " c3a\n             * c3b\n             "),
            ),
            (
                "/**
             * multi
             * line @k4
             */",
                ("", "\n             "),
            ),
            ("/**@k5 c5 w/ {@inline}!*/", ("c5 w/ {@inline}!", " c5 w/ {@inline}!")),
            (" /**@k6 */ ", ("", " ")),
            (" /**@*/ ", ("", "")),
            (" /**@@*/ ", ("", "")),
            (" /** @あいう え */ ", ("え", " え ")),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let comment = jsdocs.next().unwrap().tags().first().unwrap().comment();
            assert_eq!(
                (comment.parsed().as_str(), comment.span.source_text(source_text)),
                parsed_comment_part
            );
        }
    }

    #[test]
    fn jsdoc_tag_type() {
        for (source_text, parsed_type_part) in [
            ("/** @k0 */", None),
            ("/** @k1 {t1} */", Some(("t1", "{t1}"))),
            ("/** @k1 {} */", Some(("", "{}"))),
            (
                "/** @k2
            {t2} */",
                Some(("t2", "{t2}")),
            ),
            ("/** @k3 { t3  } */", Some(("t3", "{ t3  }"))),
            ("/** @k4 x{t4}y */", Some(("t4", "{t4}"))),
            ("/** @k5 {t5}} */", Some(("t5", "{t5}"))),
            ("/** @k6  */", None),
            ("/** @k7 x */", None),
            ("/** @k8 { */", None),
            ("/** @k9 {t9 */", None),
            ("/** @k10 {{t10} */", None),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let type_part = jsdocs.next().unwrap().tags().first().unwrap().r#type();
            assert_eq!(
                type_part.map(|t| (t.parsed(), t.span.source_text(source_text))),
                parsed_type_part
            );
        }
    }

    #[test]
    fn jsdoc_tag_type_comment() {
        for (source_text, parsed_type_part, parsed_comment_part) in [
            ("/** @k */", None, ("", " ")),
            ("/** @k1 {t1} c1 */", Some(("t1", "{t1}")), ("c1", " c1 ")),
            (
                "/** @k2
{t2} */",
                Some(("t2", "{t2}")),
                ("", " "),
            ),
            ("/** @k3  c3 */", None, ("c3", "  c3 ")),
            ("/** @k4\nc4 foo */", None, ("c4 foo", "\nc4 foo ")),
            (
                "/** @k5
{t5}
c5 */",
                Some(("t5", "{t5}")),
                ("c5", "\nc5 "),
            ),
            ("/** @k6 {t6} - c6 */", Some(("t6", "{t6}")), ("- c6", " - c6 ")),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let (type_part, comment_part) =
                jsdocs.next().unwrap().tags().first().unwrap().type_comment();
            assert_eq!(
                type_part.map(|t| (t.parsed(), t.span.source_text(source_text))),
                parsed_type_part
            );
            assert_eq!(
                (comment_part.parsed().as_str(), comment_part.span.source_text(source_text)),
                parsed_comment_part
            );
        }
    }

    #[test]
    fn jsdoc_tag_type_name_comment() {
        for (source_text, parsed_type_part, parsed_type_name_part, parsed_comment_part) in [
            ("/** @k */", None, None, ("", " ")),
            ("/** @k\n\n*/", None, None, ("", "\n\n")),
            ("/** @k1 {t1} n1 c1 */", Some(("t1", "{t1}")), Some(("n1", "n1")), ("c1", " c1 ")),
            ("/** @k2 {t2} n2*/", Some(("t2", "{t2}")), Some(("n2", "n2")), ("", "")),
            ("/** @k3 n3 c3 */", None, Some(("n3", "n3")), ("c3", " c3 ")),
            (
                "/** @k4 n4 c4
...*/",
                None,
                Some(("n4", "n4")),
                ("c4\n...", " c4\n..."),
            ),
            (
                "/** @k5  {t5}  n5  - c5 */",
                Some(("t5", "{t5}")),
                Some(("n5", "n5")),
                ("- c5", "  - c5 "),
            ),
            (
                "/** @k6
{t6}
n6
c6 */",
                Some(("t6", "{t6}")),
                Some(("n6", "n6")),
                ("c6", "\nc6 "),
            ),
            (
                "/** @k7

{t7}

n7

c7 */",
                Some(("t7", "{t7}")),
                Some(("n7", "n7")),
                ("c7", "\n\nc7 "),
            ),
            ("/** @k8 {t8} */", Some(("t8", "{t8}")), None, ("", " ")),
            ("/** @k9 n9 */", None, Some(("n9", "n9")), ("", " ")),
            ("/** @property n[].n10 */", None, Some(("n[].n10", "n[].n10")), ("", " ")),
            ("/** @property n.n11 */", None, Some(("n.n11", "n.n11")), ("", " ")),
            (
                r#"/** @property [cfg.n12="default value"] */"#,
                None,
                Some(("cfg.n12", r#"[cfg.n12="default value"]"#)),
                ("", " "),
            ),
            (
                "/** @property {t13} [n = 13] c13 */",
                Some(("t13", "{t13}")),
                Some(("n", "[n = 13]")),
                ("c13", " c13 "),
            ),
            (
                "/** @param {t14} [n14] - opt */",
                Some(("t14", "{t14}")),
                Some(("n14", "[n14]")),
                ("- opt", " - opt "),
            ),
            ("/** @param {t15}a */", Some(("t15", "{t15}")), Some(("a", "a")), ("", " ")),
            ("/** @type{t16}n16*/", Some(("t16", "{t16}")), Some(("n16", "n16")), ("", "")),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let (type_part, type_name_part, comment_part) =
                jsdocs.next().unwrap().tags().first().unwrap().type_name_comment();
            assert_eq!(
                type_part.map(|t| (t.parsed(), t.span.source_text(source_text))),
                parsed_type_part,
                "type_part failed to assert in {source_text}"
            );
            assert_eq!(
                type_name_part.map(|n| (n.parsed(), n.span.source_text(source_text))),
                parsed_type_name_part,
                "type_name_part failed to assert in {source_text}"
            );
            assert_eq!(
                (comment_part.parsed().as_str(), comment_part.span.source_text(source_text)),
                parsed_comment_part,
                "comment_part failed to assert in {source_text}"
            );
        }
    }
}
