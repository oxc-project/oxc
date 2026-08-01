//! [`SemanticId`] trait for branchless cloning of semantic ID types.

use std::cell::Cell;

use nonmax::NonMaxU32;
use oxc_allocator::CloneInSemanticIds;
use oxc_index::Idx;

// The branchless bit-tricks in [`SemanticId`]'s methods rely on the exact discriminant values of
// [`CloneInSemanticIds`]. This crate is separate from the one that defines [`CloneInSemanticIds`],
// so assert the discriminants here rather than relying on them by proximity.
const _: () = {
    assert!(CloneInSemanticIds::With as u32 == 0);
    assert!(CloneInSemanticIds::Without as u32 == u32::MAX);
};

/// A semantic ID type - a [`NonMaxU32`]-backed index (`NodeId`, `ScopeId`, `SymbolId`, `ReferenceId`),
/// created by [`define_nonmax_u32_index_type!`].
///
/// Provides methods to clone a semantic ID, keeping it when cloning with semantic IDs
/// and resetting it when cloning without - branchless in both cases:
///
/// * [`clone_id`] - `CloneIn::clone_in_impl` for ID types delegates to it.
/// * [`clone_cell_option_id`] - the `CloneIn` derive calls it to clone `Cell<Option<Id>>` fields
///   (`ScopeId`, `SymbolId`, `ReferenceId`).
///
/// The [`Idx`] bound provides [`index`] and [`from_usize`], the two operations these methods need.
///
/// [`define_nonmax_u32_index_type!`]: oxc_index::define_nonmax_u32_index_type
/// [`clone_id`]: SemanticId::clone_id
/// [`clone_cell_option_id`]: SemanticId::clone_cell_option_id
/// [`index`]: Idx::index
/// [`from_usize`]: Idx::from_usize
pub trait SemanticId: Idx {
    /// Clone a semantic ID.
    ///
    /// Behavior depends on value of `with_semantic_ids`:
    ///
    /// * [`CloneInSemanticIds::With`]: Copy the current value.
    /// * [`CloneInSemanticIds::Without`]: Reset to the dummy ID `Id(0)`.
    ///
    /// Branchless - a single OR instruction.
    ///
    /// # How this works
    ///
    /// `index & mask`, where `mask` is all-ones for [`With`] (discriminant `0`)
    /// and all-zeros for [`Without`] (discriminant `u32::MAX`), then rebuilt.
    /// `from_usize`'s bounds check is elided: `index & mask <= index <= MAX`.
    ///
    /// <https://godbolt.org/z/1KWdc5hnb>
    ///
    /// [`With`]: CloneInSemanticIds::With
    /// [`Without`]: CloneInSemanticIds::Without
    #[must_use]
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this method is only a single instruction
    fn clone_id(&self, with_semantic_ids: CloneInSemanticIds) -> Self {
        let mask = !(with_semantic_ids as u32);
        // `index()` of a `NonMaxU32`-backed type always fits in `u32`, so `as u32` can't truncate
        #[expect(clippy::cast_possible_truncation)]
        let index = self.index() as u32;
        let masked = index & mask;
        Self::from_usize(masked as usize)
    }

    /// Clone a `Cell<Option<Id>>` semantic ID field.
    ///
    /// Behavior depends on value of `with_semantic_ids`:
    ///
    /// * [`CloneInSemanticIds::With`]: Copy the current value.
    /// * [`CloneInSemanticIds::Without`]: Reset to `None`.
    ///
    /// Branchless - a load plus an AND-NOT operation
    /// (a single `bic` instruction on aarch64, `not` + `and` on x86_64).
    ///
    /// <https://godbolt.org/z/1KWdc5hnb>
    ///
    /// # How this works
    ///
    /// The in-memory representation of `Option<Id>` (equivalently `Option<NonMaxU32>`) is:
    ///
    /// * `None` = `0` (niche optimization - `NonMaxU32` wraps a `NonZeroU32` whose niche is 0).
    /// * `Some(v)` = `!v` (`NonMaxU32` stores its value bitwise-inverted, and is never 0).
    ///
    /// On that representation, "keep, or replace with `None`" is `bits & mask`,
    /// where `mask` is all-ones for [`With`] and all-zeros for [`Without`].
    ///
    /// The code below computes exactly that, but via safe APIs.
    /// `bits` rebuilds the stored form as a plain logical value (`!id.index()` for `Some`, `0` for `None`),
    /// and `NonMaxU32::new(!x)` inverts the encoding (`new` returns `None` exactly when `x == 0`).
    /// Correctness is plain logic and doesn't depend on the representation - only the codegen does.
    /// The compiler folds it all to the same instructions as an unsafe `transmute`-based version.
    ///
    /// [`With`]: CloneInSemanticIds::With
    /// [`Without`]: CloneInSemanticIds::Without
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this method is only a few instructions
    fn clone_cell_option_id(
        cell: &Cell<Option<Self>>,
        with_semantic_ids: CloneInSemanticIds,
    ) -> Cell<Option<Self>> {
        let mask = !(with_semantic_ids as u32);
        // `index()` of a `NonMaxU32`-backed type always fits in `u32`, so `as u32` can't truncate
        #[expect(clippy::cast_possible_truncation)]
        let bits = match cell.get() {
            Some(id) => !(id.index() as u32),
            None => 0,
        };
        let masked = bits & mask;
        let raw = NonMaxU32::new(!masked);
        Cell::new(raw.map(|raw| Self::from_usize(raw.get() as usize)))
    }
}

#[cfg(test)]
mod test {
    use std::cell::Cell;

    use oxc_allocator::CloneInSemanticIds;

    use crate::node::NodeId;

    use super::SemanticId;

    // `NonMaxU32` cannot represent `u32::MAX`, so `u32::MAX - 1` is the largest ID we can test
    const INPUTS: [u32; 11] = [
        0,
        1,
        2,
        42,
        1000,
        u32::MAX / 2 - 1,
        u32::MAX / 2,
        u32::MAX / 2 + 1,
        u32::MAX - 1000,
        u32::MAX - 2,
        u32::MAX - 1,
    ];

    #[test]
    fn clone_id() {
        for n in INPUTS {
            let id = NodeId::from_usize(n as usize);
            // `With` copies the current value
            assert_eq!(id.clone_id(CloneInSemanticIds::With), id);
            // `Without` resets to the dummy ID `NodeId(0)`
            assert_eq!(id.clone_id(CloneInSemanticIds::Without), NodeId::from_usize(0));
        }
    }

    #[test]
    fn clone_cell_option_id() {
        // `None` is returned unchanged by both variants
        let cell = Cell::new(None);
        assert_eq!(NodeId::clone_cell_option_id(&cell, CloneInSemanticIds::With).get(), None);
        assert_eq!(NodeId::clone_cell_option_id(&cell, CloneInSemanticIds::Without).get(), None);

        for n in INPUTS {
            let id = NodeId::from_usize(n as usize);
            let cell = Cell::new(Some(id));
            // `With` copies the current value
            assert_eq!(
                NodeId::clone_cell_option_id(&cell, CloneInSemanticIds::With).get(),
                Some(id)
            );
            // `Without` resets to `None`
            assert_eq!(
                NodeId::clone_cell_option_id(&cell, CloneInSemanticIds::Without).get(),
                None
            );
        }
    }
}
