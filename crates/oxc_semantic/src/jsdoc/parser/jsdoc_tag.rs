use super::jsdoc_parts::{
    JSDocCommentPart, JSDocTagKindPart, JSDocTagTypeNamePart, JSDocTagTypePart,
};
use crate::jsdoc::parser::utils;
use oxc_span::Span;

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
    /// comment can be multiline.
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
    /// comment can be multiline.
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
                // +1 for whitespace
                let c_start = self.body_raw.len().min(t_end + 1);
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
    /// comment can be multiline.
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
                    // +1 for whitespace
                    let c_start = self.body_raw.len().min(t_end + 1);
                    (
                        Some(JSDocTagTypePart::new(
                            &self.body_raw[t_start..t_end],
                            Span::new(
                                self.body_span.start + u32::try_from(t_start).unwrap_or_default(),
                                self.body_span.start + u32::try_from(t_end).unwrap_or_default(),
                            ),
                        )),
                        &self.body_raw[c_start..],
                        self.body_span.start + u32::try_from(t_end).unwrap_or_default(),
                    )
                }
                None => (None, self.body_raw, self.body_span.start),
            };

        let (name_part, comment_part) = match utils::find_token_range(name_comment_content) {
            Some((n_start, n_end)) => {
                // +1 for whitespace
                let c_start = name_comment_content.len().min(n_end + 1);
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
    use crate::{Semantic, SemanticBuilder};
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    fn build_semantic<'a>(allocator: &'a Allocator, source_text: &'a str) -> Semantic<'a> {
        let source_type = SourceType::default();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        let program = allocator.alloc(ret.program);
        let semantic = SemanticBuilder::new(source_text, source_type)
            .with_trivias(ret.trivias)
            .build(program)
            .semantic;
        semantic
    }

    #[test]
    fn jsdoc_tag_span() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            r"
                /** single line @k1 c1 */
                /**
                 * multi
                 * line @k1 c1
                 * @k2 c2a
                 * c2b
                 * @k3 c3
                 * @k4 c4a
                 * c4b
                 */
                ",
        );
        let mut jsdocs = semantic.jsdoc().iter_all();

        let jsdoc = jsdocs.next().unwrap();
        let mut tags = jsdoc.tags().iter();
        let tag = tags.next().unwrap();
        assert_eq!(tag.span.source_text(semantic.source_text), "@k1 c1 ");

        let jsdoc = jsdocs.next().unwrap();
        let mut tags = jsdoc.tags().iter();
        let tag = tags.next().unwrap();
        assert_eq!(tag.span.source_text(semantic.source_text), "@k1 c1\n                 * ");
        let tag = tags.next().unwrap();
        assert_eq!(
            tag.span.source_text(semantic.source_text),
            "@k2 c2a\n                 * c2b\n                 * "
        );
        let tag = tags.next().unwrap();
        assert_eq!(tag.span.source_text(semantic.source_text), "@k3 c3\n                 * ");
        let tag = tags.next().unwrap();
        assert_eq!(
            tag.span.source_text(semantic.source_text),
            "@k4 c4a\n                 * c4b\n                 "
        );
    }

    #[test]
    fn jsdoc_tag_kind() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            r"
                /**
                 * multi
                 * line @k1 c1
                 * @かいんど2 c2a
                 * c2b
                 *   @k3 c3 @k4 c4a
                 * c4b
                 */
                ",
        );
        let mut jsdocs = semantic.jsdoc().iter_all();
        let jsdoc = jsdocs.next().unwrap();
        let mut tags = jsdoc.tags().iter();

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.span.source_text(semantic.source_text), "@k1");
        assert_eq!(tag.kind.parsed(), "k1");
        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.span.source_text(semantic.source_text), "@かいんど2");
        assert_eq!(tag.kind.parsed(), "かいんど2");
        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.span.source_text(semantic.source_text), "@k3");
        assert_eq!(tag.kind.parsed(), "k3");
        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.span.source_text(semantic.source_text), "@k4");
        assert_eq!(tag.kind.parsed(), "k4");
    }

    #[test]
    fn jsdoc_tag_comment() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            r"
    /**
     * @k1 @k2 c2 @k3
     * c3a
     * c3b
     * @k4 c4 w/ {@inline}!
     */
                ",
        );
        let mut jsdocs = semantic.jsdoc().iter_all();
        let jsdoc = jsdocs.next().unwrap();
        let mut tags = jsdoc.tags().iter();

        let comment = tags.next().unwrap().comment();
        assert_eq!(comment.parsed(), "");
        assert_eq!(comment.span.source_text(semantic.source_text), "");
        assert_eq!(comment.span_first_line().source_text(semantic.source_text), "");
        let comment = tags.next().unwrap().comment();
        assert_eq!(comment.span.source_text(semantic.source_text), "c2 ");
        assert_eq!(comment.parsed(), "c2");
        assert_eq!(comment.span_first_line().source_text(semantic.source_text), "c2 ");
        let comment = tags.next().unwrap().comment();
        assert_eq!(comment.span.source_text(semantic.source_text), "     * c3a\n     * c3b\n     * ");
        assert_eq!(comment.parsed(), "c3a\nc3b");
        assert_eq!(comment.span_first_line().source_text(semantic.source_text), "");
        let comment = tags.next().unwrap().comment();
        assert_eq!(comment.span.source_text(semantic.source_text), "c4 w/ {@inline}!\n     ");
        assert_eq!(comment.parsed(), "c4 w/ {@inline}!");
        assert_eq!(comment.span_first_line().source_text(semantic.source_text), "c4 w/ {@inline}!");
    }

    //     #[test]
    //     fn parses_type() {
    //         assert_eq!(JSDocTag::new("t", " {t1}").r#type(), Some("t1"));
    //         assert_eq!(JSDocTag::new("t", "\n{t2} foo").r#type(), Some("t2"));
    //         assert_eq!(JSDocTag::new("t", " {t3 } ").r#type(), Some("t3 "));
    //         assert_eq!(JSDocTag::new("t", "  ").r#type(), None);
    //         assert_eq!(JSDocTag::new("t", " t4").r#type(), None);
    //         assert_eq!(JSDocTag::new("t", " {t5 ").r#type(), None);
    //         assert_eq!(JSDocTag::new("t", " {t6}\nx").r#type(), Some("t6"));
    //     }

    //     #[test]
    //     fn parses_type_comment() {
    //         assert_eq!(JSDocTag::new("r", " {t1} c1").type_comment(), (Some("t1"), "c1".to_string()));
    //         assert_eq!(JSDocTag::new("r", "\n{t2}").type_comment(), (Some("t2"), String::new()));
    //         assert_eq!(JSDocTag::new("r", " c3").type_comment(), (None, "c3".to_string()));
    //         assert_eq!(JSDocTag::new("r", " c4 foo").type_comment(), (None, "c4 foo".to_string()));
    //         assert_eq!(JSDocTag::new("r", " ").type_comment(), (None, String::new()));
    //         assert_eq!(
    //             JSDocTag::new("r", "\n{t5}\nc5\n...").type_comment(),
    //             (Some("t5"), "c5\n...".to_string())
    //         );
    //         assert_eq!(
    //             JSDocTag::new("r", " {t6} - c6").type_comment(),
    //             (Some("t6"), "- c6".to_string())
    //         );
    //         assert_eq!(
    //             JSDocTag::new("r", " {{ 型: t7 }} : c7").type_comment(),
    //             (Some("{ 型: t7 }"), ": c7".to_string())
    //         );
    //     }

    //     #[test]
    //     fn parses_type_name_comment() {
    //         assert_eq!(
    //             JSDocTag::new("p", " {t1} n1 c1").type_name_comment(),
    //             (Some("t1"), Some("n1"), "c1".to_string())
    //         );
    //         assert_eq!(
    //             JSDocTag::new("p", " {t2} n2").type_name_comment(),
    //             (Some("t2"), Some("n2"), String::new())
    //         );
    //         assert_eq!(
    //             JSDocTag::new("p", " n3 c3").type_name_comment(),
    //             (None, Some("n3"), "c3".to_string())
    //         );
    //         assert_eq!(JSDocTag::new("p", "").type_name_comment(), (None, None, String::new()));
    //         assert_eq!(JSDocTag::new("p", "\n\n").type_name_comment(), (None, None, String::new()));
    //         assert_eq!(
    //             JSDocTag::new("p", " {t4} n4 c4\n...").type_name_comment(),
    //             (Some("t4"), Some("n4"), "c4\n...".to_string())
    //         );
    //         assert_eq!(
    //             JSDocTag::new("p", " {t5} n5 - c5").type_name_comment(),
    //             (Some("t5"), Some("n5"), "- c5".to_string())
    //         );
    //         assert_eq!(
    //             JSDocTag::new("p", "\n{t6}\nn6\nc6").type_name_comment(),
    //             (Some("t6"), Some("n6"), "c6".to_string())
    //         );
    //         assert_eq!(
    //             JSDocTag::new("p", "\n\n{t7}\nn7\nc\n7").type_name_comment(),
    //             (Some("t7"), Some("n7"), "c\n7".to_string())
    //         );
    //         assert_eq!(
    //             JSDocTag::new("p", " {t8}").type_name_comment(),
    //             (Some("t8"), None, String::new())
    //         );
    //     }
}
