//! Per-run execution unit shared by a root formatter and its embedded children.

use std::sync::Arc;

use oxc_allocator::Allocator;

use crate::{DispatchOutcome, DispatchRequest, FormatDispatcher, UniqueGroupIdBuilder};

/// Upper bound on nested embedded dispatches;
/// exceeding it is an operational error, catching accidental A-in-B-in-A infinite recursion.
pub const MAX_DISPATCH_DEPTH: u16 = 32;

/// Document-envelope semantics of the input being formatted.
///
/// This describes ONLY who owns file-level envelope concerns (front matter, BOM).
/// It never selects parser dialects or tolerances: css-in-js and a JSDoc CSS fence are both [`InputKind::Fragment`],
/// yet parse in different modes, that difference travels as pair-specific dispatch context, not here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputKind {
    /// A root selected by filepath and config resolution.
    /// The only input that may own a BOM and (for supporting hosts) front matter.
    PhysicalFile,
    /// A complete document a host formatter passes as embedded input
    /// (e.g. a whole fenced stylesheet in Markdown).
    VirtualDocument,
    /// A grammar fragment such as css-in-js or a JSDoc fence.
    /// Never acquires file semantics, even when its head looks like front matter.
    Fragment,
}

/// Execution unit threaded through a formatting run: one arena, one `GroupId` space,
/// one dispatcher, plus the input's envelope semantics.
///
/// The same type serves standalone roots and dispatched children,
/// so any formatter (not just JS) can dispatch embedded languages.
/// Cloning hands out another handle to the SAME session (shared `GroupId` space, same depth);
/// the session for one embedded child comes from [`Self::derive_child`].
#[derive(Clone)]
pub struct FormatSession<'a> {
    allocator: &'a Allocator,
    group_id_builder: Arc<UniqueGroupIdBuilder>,
    dispatcher: Option<FormatDispatcher>,
    input_kind: InputKind,
    dispatch_depth: u16,
}

impl<'a> FormatSession<'a> {
    /// Creates a root session. Children must come from [`Self::derive_child`],
    /// never be re-rooted, so the `GroupId` space stays shared.
    pub fn new(
        allocator: &'a Allocator,
        input_kind: InputKind,
        dispatcher: Option<FormatDispatcher>,
    ) -> Self {
        Self {
            allocator,
            group_id_builder: Arc::new(UniqueGroupIdBuilder::default()),
            dispatcher,
            input_kind,
            dispatch_depth: 0,
        }
    }

    /// Derives the session for one embedded child: same arena, `GroupId` space,
    /// and dispatcher; the child's envelope semantics and one more dispatch level.
    /// The recursion limit over `dispatch_depth` is enforced by the dispatch path, not here.
    #[must_use]
    pub fn derive_child(&self, input_kind: InputKind) -> Self {
        Self {
            allocator: self.allocator,
            group_id_builder: Arc::clone(&self.group_id_builder),
            dispatcher: self.dispatcher.clone(),
            input_kind,
            dispatch_depth: self.dispatch_depth + 1,
        }
    }

    /// Formats one embedded-language request on a child session derived from this one.
    ///
    /// See [`DispatchOutcome`] for the outcome-vs-`Err` contract; this method adds two rules:
    /// no dispatcher installed (plain standalone run) counts as the deliberate
    /// `Ok(DispatchOutcome::PreserveOriginal)`, and reaching `MAX_DISPATCH_DEPTH` is an `Err`.
    ///
    /// The callback receives the derived child session;
    /// it must not create another `GroupId` space or bump the depth again.
    ///
    /// # Errors
    /// Operational failures only (recursion limit, transport/internal errors).
    pub fn dispatch(&self, request: DispatchRequest<'_>) -> Result<DispatchOutcome<'a>, String> {
        let Some(dispatcher) = &self.dispatcher else {
            return Ok(DispatchOutcome::PreserveOriginal);
        };
        if self.dispatch_depth >= MAX_DISPATCH_DEPTH {
            return Err(format!(
                "embedded dispatch exceeded the recursion limit ({MAX_DISPATCH_DEPTH})"
            ));
        }
        let child = self.derive_child(request.input_kind);
        dispatcher(&child, request)
    }

    /// The arena shared between the root and every embedded child.
    #[inline]
    pub fn allocator(&self) -> &'a Allocator {
        self.allocator
    }

    /// The `GroupId` space shared between the root and every embedded child.
    #[inline]
    pub fn group_id_builder(&self) -> &UniqueGroupIdBuilder {
        &self.group_id_builder
    }

    /// Envelope semantics of the input this session formats.
    pub fn input_kind(&self) -> InputKind {
        self.input_kind
    }

    /// How many dispatch boundaries deep this session is (0 for a root).
    pub fn dispatch_depth(&self) -> u16 {
        self.dispatch_depth
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use oxc_allocator::Allocator;

    use super::{FormatSession, InputKind, MAX_DISPATCH_DEPTH};
    use crate::{DispatchOutcome, DispatchRequest, DispatchResult, FormatDispatcher, FormatState};

    fn request<'r>() -> DispatchRequest<'r> {
        DispatchRequest {
            language: "yaml",
            texts: &[],
            input_kind: InputKind::Fragment,
            parent_context: None,
        }
    }

    #[test]
    fn dispatch_without_dispatcher_preserves_original() {
        let allocator = Allocator::default();
        let session = FormatSession::new(&allocator, InputKind::PhysicalFile, None);

        assert!(matches!(session.dispatch(request()), Ok(DispatchOutcome::PreserveOriginal)));
    }

    #[test]
    fn dispatch_stops_at_the_depth_limit() {
        let allocator = Allocator::default();
        let dispatcher: FormatDispatcher = Arc::new(|_ctx, _request| {
            Ok(DispatchOutcome::Formatted(DispatchResult {
                docs: Vec::new(),
                tailwind_classes: Vec::new(),
                meta: None,
            }))
        });
        let mut session = FormatSession::new(&allocator, InputKind::PhysicalFile, Some(dispatcher));

        assert!(matches!(session.dispatch(request()), Ok(DispatchOutcome::Formatted(_))));
        for _ in 0..MAX_DISPATCH_DEPTH {
            session = session.derive_child(InputKind::Fragment);
        }
        assert!(session.dispatch(request()).is_err());
    }

    #[test]
    fn dispatch_propagates_operational_errors() {
        let allocator = Allocator::default();
        let dispatcher: FormatDispatcher = Arc::new(|_ctx, _request| Err("boom".to_string()));
        let session = FormatSession::new(&allocator, InputKind::PhysicalFile, Some(dispatcher));

        assert_eq!(session.dispatch(request()).err().as_deref(), Some("boom"));
    }

    #[test]
    fn derived_child_shares_the_group_id_space() {
        let allocator = Allocator::default();
        let parent = FormatSession::new(&allocator, InputKind::PhysicalFile, None);
        let child = parent.derive_child(InputKind::Fragment);

        assert_eq!(child.input_kind(), InputKind::Fragment);
        assert_eq!(child.dispatch_depth(), 1);
        // Ids stay unique across the boundary; independent builders would
        // both hand out the same first id.
        assert_ne!(parent.group_id_builder().group_id("a"), child.group_id_builder().group_id("b"));
    }

    #[test]
    fn states_built_from_one_session_share_the_group_id_space() {
        let allocator = Allocator::default();
        let session = FormatSession::new(&allocator, InputKind::PhysicalFile, None);

        let parent_state = FormatState::new_with_session((), session.clone());
        let child_state = FormatState::new_with_session((), session);

        // Ids stay unique across the two states;
        // independent builders would both hand out the same first id.
        assert_ne!(parent_state.group_id("parent"), child_state.group_id("child"));
    }

    #[test]
    fn compatibility_wrapper_keeps_independent_spaces() {
        let allocator = Allocator::default();
        let a = FormatState::new((), &allocator);
        let b = FormatState::new((), &allocator);

        // Pre-session behavior, pinned: two plain states are unrelated runs.
        assert_eq!(a.group_id("a"), b.group_id("b"));
    }
}
