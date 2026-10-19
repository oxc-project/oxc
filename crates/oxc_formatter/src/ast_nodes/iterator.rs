//! Iteration over `AstNode<ArenaVec<T>>`, and the `following_span_start` of the items.
//!
//! `impl_list_item!` gives each element type its rules for the following node.
//! The span getter returns `None` for elements the formatter never prints (dropped empty statements, elision holes),
//! which are skipped when computing `following_span_start`.

use std::slice;

use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    formatter::JsFormatter,
    utils::{is_dropped_statement, statement_span},
};

use super::{AstNode, AstNodes, Following};

/// Iterator for `AstNode<ArenaVec<T>>`.
#[derive(Clone)]
pub struct AstNodeIterator<'a, T> {
    inner: slice::Iter<'a, T>,
}

impl<'a, T> Iterator for AstNodeIterator<'a, T> {
    type Item = AstNode<'a, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(AstNode::new)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<T> DoubleEndedIterator for AstNodeIterator<'_, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(AstNode::new)
    }
}

impl<T> ExactSizeIterator for AstNodeIterator<'_, T> {}

impl<'a, T> AstNode<'a, ArenaVec<'a, T>> {
    #[inline]
    pub fn iter(self) -> AstNodeIterator<'a, T> {
        AstNodeIterator { inner: self.inner.iter() }
    }

    #[inline]
    pub fn first(self) -> Option<AstNode<'a, T>> {
        self.inner.first().map(AstNode::new)
    }

    #[inline]
    pub fn last(self) -> Option<AstNode<'a, T>> {
        self.inner.last().map(AstNode::new)
    }
}

impl<'a, T: ListItem> AstNode<'a, ArenaVec<'a, T>> {
    /// The node holding the list field: a list is not on the ancestor stack,
    /// it is formatted in the frame of its parent, which is the current node.
    pub fn parent(self, f: &JsFormatter<'_, 'a>) -> AstNodes<'a> {
        let parent = f.context().ancestors().current();
        debug_assert!(
            self.inner.iter().find(|item| item.node_address().is_some()).is_none_or(|item| {
                parent.following_of_child(item.node_address().unwrap(), item.span_start()).is_some()
            }),
            "a list is not a field of the current node {}",
            parent.debug_name()
        );
        parent
    }

    /// Returns the grandparent node (parent's parent).
    pub fn grand_parent(self, f: &JsFormatter<'_, 'a>) -> AstNodes<'a> {
        self.parent(f).parent(f)
    }
}

impl<'a, T> IntoIterator for AstNode<'a, ArenaVec<'a, T>> {
    type Item = AstNode<'a, T>;
    type IntoIter = AstNodeIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &AstNode<'a, ArenaVec<'a, T>> {
    type Item = AstNode<'a, T>;
    type IntoIter = AstNodeIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An element of a list field, with its rules for the following node.
pub trait ListItem {
    /// Whether the last item's following node is the list field's own (the next field, or the parent's),
    /// instead of none (0).
    ///
    /// This is essential for [`Comments::get_trailing_comments`] to correctly distinguish trailing
    /// comments from leading comments of the following sibling. When `following_span_start` is 0,
    /// comments after the last element are treated as its trailing comments. But when set to
    /// the next sibling's span start, `get_trailing_comments` can properly determine which
    /// comments belong to the current node vs the following sibling outside this list.
    ///
    /// Example: For directives, without this, comments between the last directive and
    /// first statement would be incorrectly treated as trailing comments of the directive,
    /// when they should be leading comments of the statement.
    ///
    /// [`Comments::get_trailing_comments`]: crate::formatter::Comments::get_trailing_comments
    const LAST_HAS_FOLLOWING: bool;

    /// Whether the list can have elision holes, which have no span to search the items by.
    const HAS_HOLES: bool = false;

    /// The start of this item as the following node of the item before it.
    /// `None` marks items the formatter never prints;
    /// they are skipped so comments around them attach to the surviving neighbors.
    fn following_start(&self) -> Option<u32>;

    /// The address of the node this item holds, `None` for a hole.
    fn node_address(&self) -> Option<usize>;

    /// The start of the item's span, 0 for a hole.
    fn span_start(&self) -> u32;
}

/// The [`Following`] of the item of `list` at the address `child` (starting at `child_start`), `None` if it is not in `list`.
/// `field_following` is the list field's own.
pub fn list_following<T: ListItem>(
    list: &[T],
    child: usize,
    child_start: u32,
    field_following: impl FnOnce() -> Following,
) -> Option<Following> {
    let index = index_of(list, child, child_start)?;
    let following = list[index + 1..].iter().find_map(T::following_start);
    Some(match following {
        Some(start) => Following::Start(start),
        None if T::LAST_HAS_FOLLOWING => field_following(),
        None => Following::Start(0),
    })
}

fn index_of<T: ListItem>(list: &[T], child: usize, child_start: u32) -> Option<usize> {
    let is_child = |item: &T| item.node_address() == Some(child);
    if T::HAS_HOLES {
        return list.iter().position(is_child);
    }
    // The items are in source order: search from the first one starting at the child's start
    let start = list.partition_point(|item| item.span_start() < child_start);
    list[start..]
        .iter()
        .take_while(|item| item.span_start() == child_start)
        .position(is_child)
        .map(|index| start + index)
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

macro_rules! impl_list_item {
    ($type:ty) => {
        impl_list_item!($type, false, |n: &$type| Some(n.span().start));
    };
    ($type:ty, has_following_span_in_the_last_item) => {
        impl_list_item!($type, true, |n: &$type| Some(n.span().start));
    };
    ($type:ty, $has_following_span_in_the_last_item:tt, $get_span:expr) => {
        impl<'a> ListItem for $type {
            const LAST_HAS_FOLLOWING: bool = $has_following_span_in_the_last_item;

            #[inline]
            fn following_start(&self) -> Option<u32> {
                let get_span: fn(&$type) -> Option<u32> = $get_span;
                get_span(self)
            }

            #[inline]
            fn node_address(&self) -> Option<usize> {
                Some(AstNode::new(self).as_ast_nodes().address())
            }

            #[inline]
            fn span_start(&self) -> u32 {
                self.span().start
            }
        }
    };
}

/// An elision hole, with no span of its own.
macro_rules! impl_list_item_with_holes {
    ($type:ty) => {
        impl<'a> ListItem for Option<$type> {
            const LAST_HAS_FOLLOWING: bool = true;
            const HAS_HOLES: bool = true;

            #[inline]
            fn following_start(&self) -> Option<u32> {
                self.as_ref().map(|n| n.span().start)
            }

            #[inline]
            fn node_address(&self) -> Option<usize> {
                self.as_ref().map(|n| AstNode::new(n).as_ast_nodes().address())
            }

            #[inline]
            fn span_start(&self) -> u32 {
                self.as_ref().map_or(0, |n| n.span().start)
            }
        }
    };
}

impl_list_item!(Expression<'a>);
impl_list_item!(ArrayExpressionElement<'a>);
impl_list_item!(ObjectPropertyKind<'a>);
impl_list_item!(TemplateElement<'a>);
impl_list_item!(Argument<'a>);
impl_list_item!(VariableDeclarator<'a>);
impl_list_item!(SwitchCase<'a>);
impl_list_item!(ClassElement<'a>);
impl_list_item!(ImportDeclarationSpecifier<'a>);
impl_list_item!(ImportAttribute<'a>);
impl_list_item!(ExportSpecifier<'a>);
impl_list_item!(JSXAttributeItem<'a>);
impl_list_item!(JSXChild<'a>);
impl_list_item!(TSEnumMember<'a>);
impl_list_item!(TSType<'a>);
impl_list_item!(TSTupleElement<'a>);
impl_list_item!(TSTypeParameter<'a>);
impl_list_item!(TSClassImplements<'a>);
impl_list_item!(TSSignature<'a>);
impl_list_item!(TSIndexSignatureName<'a>);
impl_list_item!(TSInterfaceHeritage<'a>);
impl_list_item!(Decorator<'a>);
// Directive needs `following_span_start` to distinguish trailing comments from leading comments
// of the first statement. See `ListItem::LAST_HAS_FOLLOWING` for details.
impl_list_item!(Directive<'a>, has_following_span_in_the_last_item);
// These types need `following_span_start` to correctly attribute comments between
// the last item and the rest element (e.g., `[a, /** @type {string[]} */ ...rest]`).
impl_list_item!(FormalParameter<'a>, has_following_span_in_the_last_item);
impl_list_item!(BindingProperty<'a>, has_following_span_in_the_last_item);
impl_list_item!(AssignmentTargetProperty<'a>, has_following_span_in_the_last_item);
// In these two, a `None` element is an elision hole with no span of its own;
// `write_array_node` bounds the trailing comments of the element before it at the hole's `,` found in the source
impl_list_item_with_holes!(BindingPattern<'a>);
impl_list_item_with_holes!(AssignmentTargetMaybeDefault<'a>);

// Custom get_span for Statement to handle decorated exports.
// <https://github.com/oxc-project/oxc/issues/10409>
impl_list_item!(Statement<'a>, false, get_statement_span);
