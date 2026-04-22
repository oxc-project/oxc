use oxc_span::Span;

use crate::parser::utils;

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
        Self { span: kind.span.merge(body_span), kind, body_raw: body_content, body_span }
    }

    /// Use for various simple tags like `@access`, `@deprecated`, ...etc.
    /// Comment can be multiline.
    ///
    /// Variants:
    /// ```text
    /// @kind comment
    /// @kind
    /// ```
    pub fn comment(&self) -> JSDocCommentPart<'a> {
        JSDocCommentPart::new(self.body_raw, self.body_span)
    }

    /// Use for `@type`, `@satisfies`, ...etc.
    ///
    /// Variants:
    /// ```text
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
    /// ```text
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
    /// ```text
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
