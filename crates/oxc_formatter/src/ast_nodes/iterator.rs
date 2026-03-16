//! Iterator implementations for `Vec<T>` in AstNode.
//!
//! This module provides two macros for generating iterator implementations:
//! - `impl_ast_node_vec!` - For non-Option types (uses `.map()`)
//! - `impl_ast_node_vec_for_option!` - For Option types (uses `.and_then()`)

use std::cmp::min;

use oxc_allocator::{Allocator, Vec};
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use super::{AstNode, AstNodes};

/// Iterator for `AstNode<Vec<T>>`.
pub struct AstNodeIterator<'a, T> {
    inner: std::iter::Peekable<std::slice::Iter<'a, T>>,
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
    /// Function to compute `following_span_start` from the next element.
    get_following_span_start: fn(&T) -> u32,
}

/// Custom span getter for Statement that handles decorated exports.
/// <https://github.com/oxc-project/oxc/issues/10409>
fn get_statement_span(stmt: &Statement<'_>) -> u32 {
    match stmt {
        Statement::ExportDefaultDeclaration(export) => {
            if let ExportDefaultDeclarationKind::ClassDeclaration(class) = &export.declaration
                && let Some(decorator) = class.decorators.first()
            {
                min(decorator.span.start, export.span.start)
            } else {
                export.span.start
            }
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(Declaration::ClassDeclaration(class)) = &export.declaration
                && let Some(decorator) = class.decorators.first()
            {
                min(decorator.span.start, export.span.start)
            } else {
                export.span.start
            }
        }
        _ => stmt.span().start,
    }
}

macro_rules! impl_ast_node_vec {
    ($type:ty) => {
        impl_ast_node_vec!($type, false, |n: &$type| n.span().start);
    };
    ($type:ty, has_following_span_in_the_last_item) => {
        impl_ast_node_vec!($type, true, |n: &$type| n.span().start);
    };
    ($type:ty, $has_following_span_in_the_last_item:tt, $get_span:expr) => {
        impl<'a> AstNode<'a, Vec<'a, $type>> {
            pub fn iter(&self) -> AstNodeIterator<'a, $type> {
                AstNodeIterator {
                    inner: self.inner.iter().peekable(),
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
                let get_span: fn(&$type) -> u32 = $get_span;
                let mut inner_iter = self.inner.iter();
                self.allocator
                    .alloc(inner_iter.next().map(|inner| AstNode {
                        inner,
                        parent: self.parent,
                        allocator: self.allocator,
                        following_span_start: inner_iter.next().map_or(following, get_span),
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
                        following_span_start: self.inner.peek().map_or(following, |n| get_span(*n)),
                    }))
                    .as_ref()
            }
        }

        impl<'a> IntoIterator for &AstNode<'a, Vec<'a, $type>> {
            type Item = &'a AstNode<'a, $type>;
            type IntoIter = AstNodeIterator<'a, $type>;
            fn into_iter(self) -> Self::IntoIter {
                AstNodeIterator {
                    inner: self.inner.iter().peekable(),
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

macro_rules! impl_ast_node_vec_for_option {
    ($type:ty) => {
        impl_ast_node_vec_for_option!($type, false);
    };
    ($type:ty, has_following_span_in_the_last_item) => {
        impl_ast_node_vec_for_option!($type, true);
    };
    ($type:ty, $has_following_span_in_the_last_item:tt) => {
        impl<'a> AstNode<'a, Vec<'a, $type>> {
            pub fn iter(&self) -> AstNodeIterator<'a, $type> {
                AstNodeIterator {
                    inner: self.inner.iter().peekable(),
                    parent: self.parent,
                    allocator: self.allocator,
                    following_span_start: if $has_following_span_in_the_last_item {
                        self.following_span_start
                    } else {
                        0
                    },
                    get_following_span_start: |opt| opt.as_ref().map_or(0, |n| n.span().start),
                }
            }

            pub fn first(&self) -> Option<&'a AstNode<'a, $type>> {
                let following = if $has_following_span_in_the_last_item {
                    self.following_span_start
                } else {
                    0
                };
                let mut inner_iter = self.inner.iter();
                self.allocator
                    .alloc(inner_iter.next().map(|inner| {
                        AstNode {
                            inner,
                            parent: self.parent,
                            allocator: self.allocator,
                            following_span_start: inner_iter
                                .next()
                                .and_then(|opt| opt.as_ref().map(|n| n.span().start))
                                .unwrap_or(following),
                        }
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
                        following_span_start: self.inner.peek().map_or(following, |n| get_span(*n)),
                    }))
                    .as_ref()
            }
        }

        impl<'a> IntoIterator for &AstNode<'a, Vec<'a, $type>> {
            type Item = &'a AstNode<'a, $type>;
            type IntoIter = AstNodeIterator<'a, $type>;
            fn into_iter(self) -> Self::IntoIter {
                AstNodeIterator {
                    inner: self.inner.iter().peekable(),
                    parent: self.parent,
                    allocator: self.allocator,
                    following_span_start: if $has_following_span_in_the_last_item {
                        self.following_span_start
                    } else {
                        0
                    },
                    get_following_span_start: |opt| opt.as_ref().map_or(0, |n| n.span().start),
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
impl_ast_node_vec_for_option!(Option<BindingPattern<'a>>, has_following_span_in_the_last_item);
impl_ast_node_vec_for_option!(
    Option<AssignmentTargetMaybeDefault<'a>>,
    has_following_span_in_the_last_item
);

// Custom get_span for Statement to handle decorated exports.
// <https://github.com/oxc-project/oxc/issues/10409>
impl_ast_node_vec!(Statement<'a>, false, get_statement_span);
