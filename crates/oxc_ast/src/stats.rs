use std::{
    cell::Cell, marker::PhantomData, mem::ManuallyDrop, ops::Deref, panic::Location, ptr::NonNull,
};

/// Records the number of relevant features in the AST.
#[derive(Debug, Default, Clone)]
// should this be repr(align(8))?
pub struct Statistics {
    /// Number of symbols created in a program
    symbols: Cell<u32>,
    /// Number of AST nodes in a program
    nodes: Cell<u32>,
    /// Number of nodes creating a scope in a program
    scopes: Cell<u32>,
    references: Cell<u32>,
    /// Set to [`Some`] when the statistics are taken out of a [`StatisticsCell`].
    ///
    /// The location of where statistics were taken from is stored to help with
    /// debugging. If we wanted to, we could store an [`Option<bool>`] for
    /// release builds.
    taken: Cell<Option<&'static Location<'static>>>,
}

impl Statistics {
    pub fn observe_symbol(&self) {
        self.assert_not_taken();
        self.symbols.set(self.symbols.get() + 1);
    }

    /// Increment the number of AST nodes seen.
    pub fn observe_node(&self) {
        self.assert_not_taken();
        self.nodes.set(self.nodes.get() + 1);
    }

    /// Increment the number of scopes that have been created.
    pub fn observe_scope(&self) {
        self.assert_not_taken();
        self.scopes.set(self.scopes.get() + 1);
    }

    /// Increment the number of symbol references that have been created.
    pub fn observe_reference(&self) {
        self.assert_not_taken();
        self.references.set(self.references.get() + 1);
    }

    /// Get the number of symbols that have been created.
    #[inline]
    pub fn symbols(&self) -> u32 {
        self.symbols.get()
    }

    /// Get the number of AST nodes in the traversed program.
    #[inline]
    pub fn nodes(&self) -> u32 {
        self.nodes.get()
    }

    /// Get the number of created symbols.
    #[inline]
    pub fn scopes(&self) -> u32 {
        self.scopes.get()
    }

    /// Get the number of symbol references created in a program.
    #[inline]
    pub fn references(&self) -> u32 {
        self.references.get()
    }

    /// Have the contents of this [`Statistics`] been taken yet?
    ///
    /// See [`Statistics::take`]
    #[inline]
    fn is_taken(&self) -> bool {
        self.taken.get().is_some()
    }

    /// Mark these [`Statistics`] as taken from the given [`Location`].
    #[inline]
    fn take(&mut self, location: &'static Location<'static>) {
        self.taken.set(Some(location));
    }

    /// If Statistics have been taken, returns the [`Location`] where they were
    /// taken from.
    #[inline]
    fn taken_from(&self) -> Option<&'static Location<'static>> {
        self.taken.get()
    }

    /// Assert that the statistics have not been taken yet.
    ///
    /// Note that "take" semantics only apply to statistics stored in a
    /// [`StatisticsCell`]. Statistics used directly can be checked by the
    /// compiler.
    ///
    /// # Panics
    /// If this [`Statistics`] has been taken already
    #[inline]
    fn assert_not_taken(&self) {
        if let Some(location) = self.taken_from() {
            panic!("Statistics became read-only after being taken at {location:?}");
        }
    }
}

/// A [`Cell`]-like type that holds a reference to a [`Statistics`] instance.
///
/// This structure acts as a safe way to share a reference to a
/// mutable set of statistics while preserving [`Copy`] semantics. Once created,
/// the underlying [`Statistics`] may be updated uding one of the various
/// `observe_*` methods, such as [`Statistics::observe_symbol`]. When AST
/// traversal is complete, you can call [`StatisticsCell::take`] to obtain the
/// finalized [`Statistics`] instance.
///
/// ## Safety
/// While this type is safe to use, it **will leak memory** if
/// [`StatisticsCell::take`] is never called.
/// In order to preserve [`Copy`], cells will not [`Drop`] their [`Statistics`]
/// instances. The only way to prevent [`Statistics`] from leaking is to take it
/// out of the cell.
///
/// The only valid way to use this type is in a situation like tree traversal,
/// where you are 100% confident that all copies of a cell will be dropped after
/// this cell has been passed to some method that uses it.
///
/// ```ignore
/// struct CountVisitor {
///   pub cell: StatisticsCell,
/// }
/// let ret = Parser::new(/* ... */).parse(/* ... */);
/// let cell = StatisticsCell::default();
/// l
/// let visit = CountVisitor { cell };
/// visit.visit_program(&ret.program);
/// drop(visit);
/// // At this point, `stats` must be no other copies of `stats` in existence.
/// let statistics: Statistics = cell.take();
/// ```
///
/// In order to prevent dereferences on freed memory, [`StatisticsCell::take`]
/// may only be called a single time, and statistics may not be mutated after
/// being taken.
///
/// ```ignore
/// let cell = StatisticsCell::default();
/// // fine, not taken yet
/// cell.observe_node();
/// let stats = cell.take();
/// // these will panic
/// cell.observe_node();
/// let stats2 = cell.take();
/// ```
#[derive(Debug, Clone, Copy)]
pub(crate) struct StatisticsCell<'s>(NonNull<ManuallyDrop<Statistics>>, PhantomData<&'s ()>);

impl<'s> Deref for StatisticsCell<'s> {
    type Target = Statistics;

    fn deref(&self) -> &Statistics {
        // SAFETY:
        // 1. Pointer is created by a Box, so it is well-aligneda and initialized,
        // 2. Meets the requirements for valid pointer reads
        //   a. dereferenceable and non-null: created by Box
        //   c. non-atomic: Statistics is not Send or Sync.
        unsafe { self.0.as_ref() }
    }
}

impl Default for StatisticsCell<'_> {
    #[must_use = "Dropping a newly-created StatisticsCell without taking it is a memory leak."]
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'s> StatisticsCell<'s> {
    #[must_use = "Dropping a newly-created StatisticsCell without taking it is a memory leak."]
    pub fn new() -> Self {
        let stats = Box::default();
        let stats = Box::leak(stats);
        Self(NonNull::new(stats).unwrap(), PhantomData)
    }

    /// Take the [`Statistics`] out of this cell, consuming it.
    ///
    /// You _must_ call this method on the last copy of the cell in order to avoid
    /// leaking memory.
    ///
    /// # Panics
    /// - If the statistics have already been taken.
    #[track_caller]
    pub fn take(mut self) -> Statistics {
        assert!(!self.is_taken(), "Statistics already taken at {:?}.", self.taken_from());
        let location = Location::caller();
        // SAFETY:
        // - pointer is created from a Box, so it is well-aligned and
        //   initialized.
        // - The only other time a mutable reference exists to this pointer is
        //   if this method is called on another StatisticsCell. If this
        //   happened, the above assertion would have already failed.
        unsafe { self.0.as_mut().take(location) };

        // SAFETY:
        // - Pointer is created from Box::leak, so it is valid.
        // - It is impossible for Box::from_raw to have been called already on
        //   this pointer, because if it had the statitiscs would already be
        //   taken and the above assertion would have failed.
        let boxed = unsafe { Box::from_raw(self.0.as_ptr()) };
        ManuallyDrop::into_inner(*boxed)
    }
}
