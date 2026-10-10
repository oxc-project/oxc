use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::source_text::SourceTextExt as _;
use crate::{
    ast_nodes::AstNode,
    formatter::{JsFormatter, JsFormatterExt as _},
    options::ArrayExpand,
};

/// The array-shaped nodes `arrayWrap` applies to,
/// the counterpart of [`super::object_like::ObjectLike`] for `objectWrap`.
#[derive(Clone, Copy)]
pub enum ArrayLike<'a, 'b> {
    ArrayExpression(&'b AstNode<'a, ArrayExpression<'a>>),
    ArrayPattern(&'b AstNode<'a, ArrayPattern<'a>>),
    ArrayAssignmentTarget(&'b AstNode<'a, ArrayAssignmentTarget<'a>>),
    TSTupleType(&'b AstNode<'a, TSTupleType<'a>>),
}

impl<'a> ArrayLike<'a, '_> {
    fn span(&self) -> Span {
        match self {
            Self::ArrayExpression(a) => a.span,
            Self::ArrayPattern(a) => a.span,
            Self::ArrayAssignmentTarget(a) => a.span,
            Self::TSTupleType(t) => t.span,
        }
    }

    /// Holes and the rest element count as elements.
    fn element_count(&self) -> usize {
        match self {
            Self::ArrayExpression(a) => a.elements.len(),
            Self::ArrayPattern(a) => a.elements.len() + usize::from(a.rest.is_some()),
            Self::ArrayAssignmentTarget(a) => a.elements.len() + usize::from(a.rest.is_some()),
            Self::TSTupleType(t) => t.element_types.len(),
        }
    }

    /// Leading holes have no node to measure against,
    /// so the first present element (or the rest element) anchors the check.
    fn first_element_start(&self) -> Option<u32> {
        match self {
            Self::ArrayExpression(a) => {
                a.elements.iter().find(|e| !e.is_elision()).map(|e| e.span().start)
            }
            Self::ArrayPattern(a) => a
                .elements
                .iter()
                .flatten()
                .map(|e| e.span().start)
                .next()
                .or_else(|| a.rest.as_deref().map(|rest| rest.span.start)),
            Self::ArrayAssignmentTarget(a) => a
                .elements
                .iter()
                .flatten()
                .map(|e| e.span().start)
                .next()
                .or_else(|| a.rest.as_ref().map(|rest| rest.span.start)),
            Self::TSTupleType(t) => t.element_types.first().map(|e| e.span().start),
        }
    }

    fn elements_have_leading_newline(&self, f: &JsFormatter<'_, 'a>) -> bool {
        self.first_element_start()
            .is_some_and(|start| f.source_text().contains_newline_between(self.span().start, start))
    }

    /// Whether `arrayWrap` forces this array to expand,
    /// on top of what Prettier's own rules decide.
    pub fn should_wrap(&self, f: &JsFormatter<'_, 'a>) -> bool {
        match f.options().array_expand {
            ArrayExpand::Auto | ArrayExpand::Never => false,
            ArrayExpand::Preserve => self.elements_have_leading_newline(f),
            ArrayExpand::ForceAboveThreshold(threshold) => {
                self.element_count() > threshold as usize || self.elements_have_leading_newline(f)
            }
        }
    }
}
