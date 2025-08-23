use oxc_ast::{AstKind, ast::Program};
use oxc_data_structures::stack::NonEmptyStack;
use oxc_span::Span;

use super::{FormatContext, GroupId, SyntaxNode, UniqueGroupIdBuilder};

/// This structure stores the state that is relevant for the formatting of the whole document.
///
/// This structure is different from [crate::Formatter] in that the formatting infrastructure
/// creates a new [crate::Formatter] for every [crate::write!] call, whereas this structure stays alive
/// for the whole process of formatting a root with [crate::format!].
pub struct FormatState<'ast> {
    context: FormatContext<'ast>,
    group_id_builder: UniqueGroupIdBuilder,
    // This is using a RefCell as it only exists in debug mode,
    // the Formatter is still completely immutable in release builds
    // #[cfg(debug_assertions)]
    // pub printed_tokens: PrintedTokens,
}

impl std::fmt::Debug for FormatState<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("FormatState").field("context", &self.context).finish()
    }
}

impl<'ast> FormatState<'ast> {
    /// Creates a new state with the given language specific context
    pub fn new(program: &'ast Program<'ast>, context: FormatContext<'ast>) -> Self {
        Self {
            context,
            group_id_builder: UniqueGroupIdBuilder::default(),
            // #[cfg(debug_assertions)]
            // printed_tokens: Default::default(),
        }
    }

    pub fn into_context(self) -> FormatContext<'ast> {
        self.context
    }

    /// Returns the context specifying how to format the current CST
    pub fn context(&self) -> &FormatContext<'ast> {
        &self.context
    }

    /// Returns a mutable reference to the context
    pub fn context_mut(&mut self) -> &mut FormatContext<'ast> {
        &mut self.context
    }

    /// Creates a new group id that is unique to this document. The passed debug name is used in the
    /// [std::fmt::Debug] of the document if this is a debug build.
    /// The name is unused for production builds and has no meaning on the equality of two group ids.
    pub fn group_id(&self, debug_name: &'static str) -> GroupId {
        self.group_id_builder.group_id(debug_name)
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub fn track_token(&mut self, span: Span) {}

    /// Tracks the given token as formatted
    #[cfg(debug_assertions)]
    #[inline]
    pub fn track_token(&mut self, span: Span) {
        // self.printed_tokens.track_token(token);
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub fn set_token_tracking_disabled(&mut self, _: bool) {}

    /// Disables or enables token tracking for a portion of the code.
    ///
    /// It can be useful to disable the token tracking when it is necessary to re-format a node with different parameters.
    #[cfg(debug_assertions)]
    pub fn set_token_tracking_disabled(&mut self, enabled: bool) {
        todo!()
        // self.printed_tokens.set_disabled(enabled)
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub fn is_token_tracking_disabled(&self) -> bool {
        false
    }

    /// Returns `true` if token tracking is currently disabled.
    #[cfg(debug_assertions)]
    pub fn is_token_tracking_disabled(&self) -> bool {
        todo!()
        // self.printed_tokens.is_disabled()
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub fn assert_formatted_all_tokens(&self, _root: &SyntaxNode) {}

    /// Asserts in debug builds that all tokens have been printed.
    #[cfg(debug_assertions)]
    #[inline]
    pub fn assert_formatted_all_tokens(&self, root: &SyntaxNode) {
        todo!()
        // self.printed_tokens.assert_all_tracked(root);
    }
}

impl FormatState<'_> {
    pub fn snapshot(&self) -> FormatStateSnapshot {
        todo!()
        // FormatStateSnapshot {
        // #[cfg(debug_assertions)]
        // printed_tokens: self.printed_tokens.snapshot(),
        // }
    }

    pub fn restore_snapshot(&mut self, _snapshot: FormatStateSnapshot) {
        todo!()
        // let FormatStateSnapshot {
        // #[cfg(debug_assertions)]
        // printed_tokens,
        // } = snapshot;

        // cfg_if::cfg_if! {
        // if #[cfg(debug_assertions)] {
        // self.printed_tokens.restore(printed_tokens);
        // }
        // }
    }
}

pub struct FormatStateSnapshot;
// #[cfg(debug_assertions)]
// printed_tokens: printed_tokens::PrintedTokensSnapshot,
