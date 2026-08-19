//! Iterator implementations for `ArenaVec<T>` in AstNode.
//!
//! `impl_ast_node_vec!` generates the iterator for each element type.
//! The span getter returns `None` for elements the formatter never prints (dropped empty statements, elision holes),
//! which are skipped when computing `following_span_start`.

use std::cmp::min;

use oxc_allocator::{Allocator, ArenaVec};
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::utils::is_dropped_statement;

use super::{AstNode, AstNodes};

/// Iterator for `AstNode<ArenaVec<T>>`.
pub struct AstNodeIterator<'a, T> {
    inner: std::slice::Iter<'a, T>,
    parent: AstNodes<'a>,
    allocator: &'a Allocator,
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
    Some(match stmt {
        Statement::ExportDefaultDeclaration(export) => {
            if let ExportDefaultDeclarationKind::ClassDeclaration(class) = &export.declaration
                && let Some(decorator) = class.decorators.first()
            {
                min(decorator.span.start, export.span.start)
            } else {
                export.span.start
            }
        }
        Statement::ExportDeclaration(export) => {
            if let Declaration::ClassDeclaration(class) = &export.declaration
                && let Some(decorator) = class.decorators.first()
            {
                min(decorator.span.start, export.span.start)
            } else {
                export.span.start
            }
        }
        _ => stmt.span().start,
    })
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
        impl<'a> AstNode<'a, ArenaVec<'a, $type>> {
            pub fn iter(&self) -> AstNodeIterator<'a, $type> {
                AstNodeIterator {
                    inner: self.inner.iter(),
                    parent: self.parent,
                    allocator: self.allocator,
                    following_span_start: if $has_following_span_in_the_last_item {
                        self.following_span_start
                    } else {
                        0
                    },
                    get_following_span_start: $get_span,
                }
            }

            pub fn first(&self) -> Option<&'a AstNode<'a, $type>> {
                let following = if $has_following_span_in_the_last_item {
                    self.following_span_start
                } else {
                    0
                };
                let get_span: fn(&$type) -> Option<u32> = $get_span;
                let mut inner_iter = self.inner.iter();
                self.allocator
                    .alloc(inner_iter.next().map(|inner| AstNode {
                        inner,
                        parent: self.parent,
                        allocator: self.allocator,
                        following_span_start: inner_iter.find_map(get_span).unwrap_or(following),
                    }))
                    .as_ref()
            }

            pub fn last(&self) -> Option<&'a AstNode<'a, $type>> {
                let following = if $has_following_span_in_the_last_item {
                    self.following_span_start
                } else {
                    0
                };
                self.allocator
                    .alloc(self.inner.last().map(|inner| AstNode {
                        inner,
                        parent: self.parent,
                        allocator: self.allocator,
                        following_span_start: following,
                    }))
                    .as_ref()
            }
        }

        impl<'a> Iterator for AstNodeIterator<'a, $type> {
            type Item = &'a AstNode<'a, $type>;
            fn next(&mut self) -> Option<Self::Item> {
                let allocator = self.allocator;
                let following = self.following_span_start;
                let get_span = self.get_following_span_start;
                allocator
                    .alloc(self.inner.next().map(|inner| AstNode {
                        parent: self.parent,
                        inner,
                        allocator,
                        following_span_start:
                            self.inner.clone().find_map(get_span).unwrap_or(following),
                    }))
                    .as_ref()
            }
        }

        impl<'a> IntoIterator for &AstNode<'a, ArenaVec<'a, $type>> {
            type Item = &'a AstNode<'a, $type>;
            type IntoIter = AstNodeIterator<'a, $type>;
            fn into_iter(self) -> Self::IntoIter {
                AstNodeIterator {
                    inner: self.inner.iter(),
                    parent: self.parent,
                    allocator: self.allocator,
                    following_span_start: if $has_following_span_in_the_last_item {
                        self.following_span_start
                    } else {
                        0
                    },
                    get_following_span_start: $get_span,
                }
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
// A `None` element is an elision hole with no span of its own
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
