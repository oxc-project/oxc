use rustc_hash::FxHashMap;

use super::{FormatContext, GroupId, UniqueGroupIdBuilder, prelude::Interned};

/// This structure stores the state that is relevant for the formatting of the whole document.
///
/// This structure is different from [crate::Formatter] in that the formatting infrastructure
/// creates a new [crate::Formatter] for every [crate::write!] call, whereas this structure stays alive
/// for the whole process of formatting a root with [crate::format!].
pub struct FormatState<'ast> {
    context: FormatContext<'ast>,
    group_id_builder: UniqueGroupIdBuilder,
    // For the document IR printing process
    /// The interned elements that have been printed to this point
    printed_interned_elements: FxHashMap<Interned<'ast>, usize>,
}

impl std::fmt::Debug for FormatState<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("FormatState").field("context", &self.context).finish()
    }
}

impl<'ast> FormatState<'ast> {
    /// Creates a new state with the given language specific context
    pub fn new(context: FormatContext<'ast>) -> Self {
        Self {
            context,
            group_id_builder: UniqueGroupIdBuilder::default(),
            printed_interned_elements: FxHashMap::default(),
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

    #[expect(clippy::mutable_key_type)]
    pub fn printed_interned_elements(&mut self) -> &mut FxHashMap<Interned<'ast>, usize> {
        &mut self.printed_interned_elements
    }
}
