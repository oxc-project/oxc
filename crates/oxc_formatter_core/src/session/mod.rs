//! Per-run execution unit shared by a root formatter and its embedded children.

pub mod embedded;

use std::sync::Arc;

use oxc_allocator::Allocator;

use crate::{
    DispatchPayload, DispatchRequest, DispatchResponse, Document, FormatDispatcher, PrinterOptions,
    UniqueGroupIdBuilder,
};

/// Upper bound on nested embedded dispatches;
/// exceeding it is an operational error, catching accidental A-in-B-in-A infinite recursion.
/// Real nesting is shallow (css-in-md-in-js would be 3); 8 is a generous ceiling.
const MAX_DISPATCH_DEPTH: u8 = 8;

/// String-in/string-out embedded formatter: `(language, code, print_width)` to formatted code.
///
/// The string-out channel's session-carried contract, for consumers whose result must come back as TEXT
/// (currently a JSDoc fence re-embedded line-by-line into a comment).
/// `print_width` is the caller's effective width at the embedding position
/// (e.g. a JSDoc fence's comment-content width minus the fence indent).
/// The returned text carries no trailing whitespace (implementations trim before returning).
/// `Err` means "keep the input verbatim".
pub type StringEmbedder = Arc<dyn Fn(&str, &str, usize) -> Result<String, String> + Send + Sync>;

/// Print-time batch sorter for the classes referenced by `FormatElement::TailwindClass(index)`;
/// runs once when a root formatter finalizes its `Document`.
pub type TailwindSorter = Arc<dyn Fn(Vec<String>) -> Vec<String> + Send + Sync>;

/// The three per-run services a host installs on a [`FormatSession`], one field per duty.
///
/// Core only transports them; consumers invoke via [`FormatSession::dispatch`] /
/// [`FormatSession::string_embedder`] / [`FormatSession::sort_tailwind_classes`].
/// `None` anywhere means "this run does not provide that service" and each consumer degrades gracefully.
#[derive(Clone, Default)]
pub struct SessionServices {
    /// IR channel: embedded-language dispatch merging child IR into the parent's document.
    pub dispatcher: Option<FormatDispatcher>,
    /// String channel: string-in/string-out embedded formatting (see [`StringEmbedder`]).
    pub string_embedder: Option<StringEmbedder>,
    /// Print-time service: batch Tailwind class sorting (see [`TailwindSorter`]).
    pub tailwind_sorter: Option<TailwindSorter>,
}

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

impl InputKind {
    /// Whether this input kind formats a leading front matter block as an envelope:
    /// `true` for roots and complete embedded documents,
    /// `false` for a [`Self::Fragment`] (its head is content, not an envelope).
    pub fn owns_front_matter(self) -> bool {
        matches!(self, Self::PhysicalFile | Self::VirtualDocument)
    }
}

/// Execution unit threaded through a formatting run: one arena, one `GroupId` space,
/// the run's [`SessionServices`], plus the input's envelope semantics.
///
/// The same type serves standalone roots and dispatched children,
/// so any formatter (not just JS) can dispatch embedded languages.
/// Cloning hands out another handle to the SAME session (shared `GroupId` space, same depth);
/// the session for one embedded child is derived internally by [`Self::dispatch`].
#[derive(Clone)]
pub struct FormatSession<'a> {
    allocator: &'a Allocator,
    group_id_builder: Arc<UniqueGroupIdBuilder>,
    services: SessionServices,
    input_kind: InputKind,
    dispatch_depth: u8,
}

impl<'a> FormatSession<'a> {
    /// Creates a service-less root session
    /// (standalone runs: embeds stay as-is, Tailwind classes print unsorted).
    /// Hosts that install services use [`Self::with_services`].
    /// Children are derived internally by [`Self::dispatch`], never re-rooted,
    /// so the `GroupId` space stays shared.
    pub fn new(allocator: &'a Allocator, input_kind: InputKind) -> Self {
        Self::with_services(allocator, input_kind, SessionServices::default())
    }

    /// Like [`Self::new`], with the host's [`SessionServices`] installed.
    pub fn with_services(
        allocator: &'a Allocator,
        input_kind: InputKind,
        services: SessionServices,
    ) -> Self {
        Self {
            allocator,
            group_id_builder: Arc::new(UniqueGroupIdBuilder::default()),
            services,
            input_kind,
            dispatch_depth: 0,
        }
    }

    /// Derives the session for one embedded child: same arena, `GroupId` space,
    /// and services; the child's envelope semantics and one more dispatch level.
    /// Private on purpose: [`Self::dispatch`] is the only caller,
    /// which makes "children never re-root the `GroupId` space or reset the depth" a compiler-checked fact.
    #[must_use]
    fn derive_child(&self, input_kind: InputKind) -> Self {
        Self {
            allocator: self.allocator,
            group_id_builder: Arc::clone(&self.group_id_builder),
            services: self.services.clone(),
            input_kind,
            dispatch_depth: self.dispatch_depth + 1,
        }
    }

    /// Formats one embedded-language request on a child session derived from this one.
    ///
    /// See [`DispatchResponse`] for the `PreserveOriginal`-vs-`Err` contract; this method adds three rules:
    /// - no dispatcher installed (plain standalone run) is the deliberate `Ok(DispatchResponse::PreserveOriginal)`
    /// - a request text starting with U+FEFF is deliberately preserved:
    ///   in an embedded position it is content, not a BOM (formatting must not swallow it), and no child entry handles it
    /// - reaching `MAX_DISPATCH_DEPTH` is an `Err`
    ///
    /// The callback receives the derived child session.
    ///
    /// # Errors
    /// Operational failures only (recursion limit, transport/internal errors).
    pub fn dispatch(&self, request: DispatchRequest<'_>) -> Result<DispatchResponse<'a>, String> {
        let Some(dispatcher) = &self.services.dispatcher else {
            return Ok(DispatchResponse::PreserveOriginal);
        };
        if request.text.starts_with('\u{feff}') {
            return Ok(DispatchResponse::PreserveOriginal);
        }
        if self.dispatch_depth >= MAX_DISPATCH_DEPTH {
            return Err(format!(
                "embedded dispatch exceeded the recursion limit ({MAX_DISPATCH_DEPTH})"
            ));
        }

        let child = self.derive_child(request.input_kind);
        dispatcher(&child, request)
    }

    /// Formats one embedded request and prints the result to text:
    /// dispatch → local Tailwind sort → [`Document`] print.
    ///
    /// The string-out counterpart of [`Self::dispatch`], for consumers whose result
    /// must come back as TEXT. The caller supplies `printer_options`, typically its own
    /// print options with the embedding position's effective width, and owns any
    /// re-embedding conventions (e.g. trailing-newline handling) on the returned text.
    /// A text consumer has no parent index space, so Tailwind classes sort locally
    /// through this session's sorter instead of merging upward.
    ///
    /// Returns `Ok(None)` when the dispatch deliberately preserved the input
    /// (see [`DispatchResponse::PreserveOriginal`]); the caller keeps its original text.
    ///
    /// # Errors
    /// Operational failures only (dispatch transport, recursion limit, print).
    pub fn dispatch_to_string(
        &self,
        request: DispatchRequest<'_>,
        printer_options: PrinterOptions,
    ) -> Result<Option<String>, String> {
        let text_len = request.text.len();
        let DispatchResponse::Formatted(DispatchPayload { doc, tailwind_classes, .. }) =
            self.dispatch(request)?
        else {
            return Ok(None);
        };
        let tailwind_classes = self.sort_tailwind_classes(tailwind_classes);

        let code = Document::new(doc, tailwind_classes)
            .print(text_len, printer_options)
            .map_err(|err| err.to_string())?
            .into_code();
        Ok(Some(code))
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

    /// The string channel service ([`StringEmbedder`]), when this run installs one.
    /// (A handle rather than an invoking method; see the alias for the `Err`-means-verbatim contract.)
    pub fn string_embedder(&self) -> Option<&StringEmbedder> {
        self.services.string_embedder.as_ref()
    }

    /// Sorts collected Tailwind classes through the print-time service ([`TailwindSorter`]),
    /// or returns them unsorted when this run installs no sorter.
    pub fn sort_tailwind_classes(&self, classes: Vec<String>) -> Vec<String> {
        match &self.services.tailwind_sorter {
            Some(sort) if !classes.is_empty() => sort(classes),
            _ => classes,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use oxc_allocator::Allocator;

    use super::{FormatSession, InputKind, MAX_DISPATCH_DEPTH, SessionServices};
    use crate::{
        DispatchPayload, DispatchRequest, DispatchResponse, FormatDispatcher, FormatState,
    };

    fn request<'r>() -> DispatchRequest<'r> {
        DispatchRequest {
            language: "yaml",
            text: "",
            input_kind: InputKind::Fragment,
            parent_context: None,
        }
    }

    #[test]
    fn dispatch_without_dispatcher_preserves_original() {
        let allocator = Allocator::default();
        let session = FormatSession::new(&allocator, InputKind::PhysicalFile);

        assert!(matches!(session.dispatch(request()), Ok(DispatchResponse::PreserveOriginal)));
    }

    #[test]
    fn dispatch_preserves_a_bom_headed_text() {
        let allocator = Allocator::default();
        let dispatcher: FormatDispatcher = Arc::new(|ctx, _request| {
            Ok(DispatchResponse::Formatted(DispatchPayload {
                doc: oxc_allocator::ArenaVec::new_in(&ctx.allocator()),
                tailwind_classes: Vec::new(),
                child_context: None,
            }))
        });
        let session = FormatSession::with_services(
            &allocator,
            InputKind::PhysicalFile,
            SessionServices { dispatcher: Some(dispatcher), ..SessionServices::default() },
        );

        let bom_request = DispatchRequest { text: "\u{feff}a: 1", ..request() };
        assert!(matches!(session.dispatch(bom_request), Ok(DispatchResponse::PreserveOriginal)));
        // A non-leading U+FEFF is ordinary content and formats normally
        let inner_request = DispatchRequest { text: "a: \u{feff}1", ..request() };
        assert!(matches!(session.dispatch(inner_request), Ok(DispatchResponse::Formatted(_))));
    }

    #[test]
    fn dispatch_stops_at_the_depth_limit() {
        let allocator = Allocator::default();
        let dispatcher: FormatDispatcher = Arc::new(|ctx, _request| {
            Ok(DispatchResponse::Formatted(DispatchPayload {
                doc: oxc_allocator::ArenaVec::new_in(&ctx.allocator()),
                tailwind_classes: Vec::new(),
                child_context: None,
            }))
        });
        let mut session = FormatSession::with_services(
            &allocator,
            InputKind::PhysicalFile,
            SessionServices { dispatcher: Some(dispatcher), ..SessionServices::default() },
        );

        assert!(matches!(session.dispatch(request()), Ok(DispatchResponse::Formatted(_))));
        for _ in 0..MAX_DISPATCH_DEPTH {
            session = session.derive_child(InputKind::Fragment);
        }
        assert!(session.dispatch(request()).is_err());
    }

    #[test]
    fn dispatch_propagates_operational_errors() {
        let allocator = Allocator::default();
        let dispatcher: FormatDispatcher = Arc::new(|_ctx, _request| Err("boom".to_string()));
        let session = FormatSession::with_services(
            &allocator,
            InputKind::PhysicalFile,
            SessionServices { dispatcher: Some(dispatcher), ..SessionServices::default() },
        );

        assert_eq!(session.dispatch(request()).err().as_deref(), Some("boom"));
    }

    #[test]
    fn derived_child_shares_the_group_id_space() {
        let allocator = Allocator::default();
        let parent = FormatSession::new(&allocator, InputKind::PhysicalFile);
        let child = parent.derive_child(InputKind::Fragment);

        assert_eq!(child.input_kind(), InputKind::Fragment);
        assert_eq!(child.dispatch_depth, 1);
        // Ids stay unique across the boundary; independent builders would
        // both hand out the same first id.
        assert_ne!(parent.group_id_builder().group_id("a"), child.group_id_builder().group_id("b"));
    }

    #[test]
    fn states_built_from_one_session_share_the_group_id_space() {
        let allocator = Allocator::default();
        let session = FormatSession::new(&allocator, InputKind::PhysicalFile);

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
