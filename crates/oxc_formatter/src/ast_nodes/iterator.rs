//! Iterator implementations for `ArenaVec<T>` in AstNode.
//!
//! `impl_ast_node_vec!` generates the iterator for each element type.
//! The span getter returns `None` for elements the formatter never prints (dropped empty statements, elision holes),
//! which are skipped when computing `following_span_start`.

use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::utils::{is_dropped_statement, statement_span};

use super::{AstNode, AstNodes};

/// Iterator for `AstNode<ArenaVec<T>>`.
pub struct AstNodeIterator<'parent, 'a, T> {
    inner: std::slice::Iter<'a, T>,
    parent: AstNodes<'parent, 'a>,
    /// The `following_span_start` for the last element when there's no next element in this iterator.
    ///
    /// This is essential for [`Comments::get_trailing_comments`] to correctly distinguish trailing
    /// comments from leading comments of the following sibling. When `following_span_start` is 0,
    /// comments after the last element are treated as its trailing comments. But when set to
    /// the next sibling's span start, `get_trailing_comments` can properly determine which
    /// comments belong to the current node vs the following sibling outside this iterator.
    ///
    /// Example: For directives, without this field, comments between the last directive and
    /// first statement would be incorrectly treated as trailing comments of the directive,
    /// when they should be leading comments of the statement.
    ///
    /// See [`Comments::get_trailing_comments`] in `crates/oxc_formatter/src/formatter/comments.rs`
    /// for the detailed handling logic.
    ///
    /// [`Comments::get_trailing_comments`]: crate::formatter::Comments::get_trailing_comments
    following_span_start: u32,
    /// Computes `following_span_start` from a following element.
    /// `None` marks elements the formatter never prints;
    /// they are skipped so comments around them attach to the surviving neighbors.
    get_following_span_start: fn(&T) -> Option<u32>,
}

/// Custom span getter for Statement that handles decorated exports.
/// <https://github.com/oxc-project/oxc/issues/10409>
///
/// Returns `None` for statements dropped from statement lists.
fn get_statement_span(stmt: &Statement<'_>) -> Option<u32> {
    if is_dropped_statement(stmt) {
        return None;
    }
    Some(statement_span(stmt).start)
}

macro_rules! impl_ast_node_vec {
    ($type:ty) => {
        impl_ast_node_vec!($type, false, |n: &$type| Some(n.span().start));
    };
    ($type:ty, has_following_span_in_the_last_item) => {
        impl_ast_node_vec!($type, true, |n: &$type| Some(n.span().start));
    };
    ($type:ty, has_following_span_in_the_last_item, $get_span:expr) => {
        impl_ast_node_vec!($type, true, $get_span);
    };
    ($type:ty, $has_following_span_in_the_last_item:tt, $get_span:expr) => {
        impl<'parent, 'a> AstNode<'parent, 'a, ArenaVec<'a, $type>> {
            pub fn iter(&self) -> AstNodeIterator<'parent, 'a, $type> {
                AstNodeIterator {
                    inner: self.inner.iter(),
                    parent: self.parent,
                    following_span_start: if $has_following_span_in_the_last_item {
                        self.following_span_start
                    } else {
                        0
                    },
                    get_following_span_start: $get_span,
                }
            }

            pub fn first(&self) -> Option<AstNode<'parent, 'a, $type>> {
                self.iter().next()
            }

            pub fn last(&self) -> Option<AstNode<'parent, 'a, $type>> {
                let following = if $has_following_span_in_the_last_item {
                    self.following_span_start
                } else {
                    0
                };
                self.inner.last().map(|inner| AstNode {
                    inner,
                    parent: self.parent,
                    following_span_start: following,
                })
            }
        }

        impl<'parent, 'a> Iterator for AstNodeIterator<'parent, 'a, $type> {
            type Item = AstNode<'parent, 'a, $type>;
            fn next(&mut self) -> Option<Self::Item> {
                let inner = self.inner.next()?;
                let following_span_start = self
                    .inner
                    .clone()
                    .find_map(self.get_following_span_start)
                    .unwrap_or(self.following_span_start);
                Some(AstNode { inner, parent: self.parent, following_span_start })
            }
        }

        impl<'parent, 'a> IntoIterator for &AstNode<'parent, 'a, ArenaVec<'a, $type>> {
            type Item = AstNode<'parent, 'a, $type>;
            type IntoIter = AstNodeIterator<'parent, 'a, $type>;
            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }
    };
}

impl_ast_node_vec!(Expression<'a>);
impl_ast_node_vec!(ArrayExpressionElement<'a>);
impl_ast_node_vec!(ObjectPropertyKind<'a>);
impl_ast_node_vec!(TemplateElement<'a>);
impl_ast_node_vec!(Argument<'a>);
impl_ast_node_vec!(VariableDeclarator<'a>);
impl_ast_node_vec!(SwitchCase<'a>);
impl_ast_node_vec!(ClassElement<'a>);
impl_ast_node_vec!(ImportDeclarationSpecifier<'a>);
impl_ast_node_vec!(ImportAttribute<'a>);
impl_ast_node_vec!(ExportSpecifier<'a>);
impl_ast_node_vec!(JSXAttributeItem<'a>);
impl_ast_node_vec!(JSXChild<'a>);
impl_ast_node_vec!(TSEnumMember<'a>);
impl_ast_node_vec!(TSType<'a>);
impl_ast_node_vec!(TSTupleElement<'a>);
impl_ast_node_vec!(TSTypeParameter<'a>);
impl_ast_node_vec!(TSClassImplements<'a>);
impl_ast_node_vec!(TSSignature<'a>);
impl_ast_node_vec!(TSIndexSignatureName<'a>);
impl_ast_node_vec!(TSInterfaceHeritage<'a>);
impl_ast_node_vec!(Decorator<'a>);
// Directive needs `following_span_start` to distinguish trailing comments from leading comments
// of the first statement. See the struct field comment for `following_span_start` for details.
impl_ast_node_vec!(Directive<'a>, has_following_span_in_the_last_item);
// These types need `following_span_start` to correctly attribute comments between
// the last item and the rest element (e.g., `[a, /** @type {string[]} */ ...rest]`).
impl_ast_node_vec!(FormalParameter<'a>, has_following_span_in_the_last_item);
impl_ast_node_vec!(BindingProperty<'a>, has_following_span_in_the_last_item);
impl_ast_node_vec!(AssignmentTargetProperty<'a>, has_following_span_in_the_last_item);
// In these two, a `None` element is an elision hole with no span of its own;
// `write_array_node` bounds the trailing comments of the element before it at the hole's `,` found in the source
impl_ast_node_vec!(
    Option<BindingPattern<'a>>,
    has_following_span_in_the_last_item,
    |opt: &Option<BindingPattern<'a>>| opt.as_ref().map(|n| n.span().start)
);
impl_ast_node_vec!(
    Option<AssignmentTargetMaybeDefault<'a>>,
    has_following_span_in_the_last_item,
    |opt: &Option<AssignmentTargetMaybeDefault<'a>>| opt.as_ref().map(|n| n.span().start)
);

// Custom get_span for Statement to handle decorated exports.
// <https://github.com/oxc-project/oxc/issues/10409>
impl_ast_node_vec!(Statement<'a>, false, get_statement_span);
