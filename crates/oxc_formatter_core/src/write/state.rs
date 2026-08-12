use rustc_hash::FxHashMap;

use oxc_allocator::{Allocator, GetAllocator};

use crate::{FormatElement, FormatSession, GroupId, InputKind, format_element::Interned};

/// This structure stores the state that is relevant for the formatting of the whole document.
///
/// This structure is different from [crate::Formatter] in that the formatting infrastructure
/// creates a new [crate::Formatter] for every [`crate::write!`] call, whereas this structure stays alive
/// for the whole format run.
pub struct FormatState<'ast, C> {
    context: C,
    /// The shared execution unit of this format run; see [`FormatSession`].
    session: FormatSession<'ast>,
    // For the document IR printing process
    /// The interned elements that have been printed to this point
    printed_interned_elements: FxHashMap<Interned<'ast>, usize>,
    /// Heap staging vector shared by all [`crate::HeapVecBuffer`]s of this format run;
    /// see [`crate::HeapVecBuffer`] for the watermark scheme keeping their views disjoint.
    scratch: Vec<FormatElement<'ast>>,
}

impl<C: std::fmt::Debug> std::fmt::Debug for FormatState<'_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("FormatState").field("context", &self.context).finish()
    }
}

impl<'ast, C> FormatState<'ast, C> {
    /// Creates a new state with the given language specific context.
    ///
    /// Compatibility wrapper: builds a dispatcher-less [`InputKind::PhysicalFile`]
    /// session with its own `GroupId` space.
    /// Entry points that share a run with other formatters use [`Self::new_with_session`] instead.
    ///
    /// NOTE: standalone `format()` compatibility wrappers reach this
    /// (directly or via their own service-less session),
    /// including the string channel's JSDoc-fence formatting, which is semantically a fragment;
    /// nothing may consult `input_kind` for envelope decisions on those paths
    /// until that channel routes through the dispatcher.
    pub fn new(context: C, allocator: &'ast Allocator) -> Self {
        Self::new_with_session(context, FormatSession::new(allocator, InputKind::PhysicalFile))
    }

    /// Creates a new state on an existing session, sharing its arena and `GroupId` space.
    pub fn new_with_session(context: C, session: FormatSession<'ast>) -> Self {
        Self {
            context,
            session,
            printed_interned_elements: FxHashMap::default(),
            scratch: Vec::new(),
        }
    }

    /// The session this state formats under.
    pub fn session(&self) -> &FormatSession<'ast> {
        &self.session
    }

    /// The heap staging vector shared by all [`crate::HeapVecBuffer`]s of this format run.
    pub(crate) fn scratch(&self) -> &[FormatElement<'ast>] {
        &self.scratch
    }

    /// Mutable access to the heap staging vector; see [`FormatState::scratch`].
    pub(crate) fn scratch_mut(&mut self) -> &mut Vec<FormatElement<'ast>> {
        &mut self.scratch
    }

    /// Returns the allocator used for arena-allocating format elements.
    pub fn allocator(&self) -> &'ast Allocator {
        self.session.allocator()
    }

    pub fn into_context(self) -> C {
        self.context
    }

    /// Returns the context specifying how to format the current CST
    pub fn context(&self) -> &C {
        &self.context
    }

    /// Returns a mutable reference to the context
    pub fn context_mut(&mut self) -> &mut C {
        &mut self.context
    }

    /// Creates a new group id that is unique to this document. The passed debug name is used in the
    /// [std::fmt::Debug] of the document if this is a debug build.
    /// The name is unused for production builds and has no meaning on the equality of two group ids.
    pub fn group_id(&self, debug_name: &'static str) -> GroupId {
        self.session.group_id_builder().group_id(debug_name)
    }

    #[expect(clippy::mutable_key_type)]
    pub fn printed_interned_elements(&mut self) -> &mut FxHashMap<Interned<'ast>, usize> {
        &mut self.printed_interned_elements
    }
}

impl<'ast, C> GetAllocator<'ast> for FormatState<'ast, C> {
    #[inline]
    fn allocator(&self) -> &'ast Allocator {
        self.session.allocator()
    }
}
