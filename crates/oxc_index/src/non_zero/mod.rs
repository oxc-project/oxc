use core::fmt::Debug;
use core::hash::Hash;

pub mod indexing;
pub mod slice;
pub mod vec;

/// Represents a wrapped value convertible to and from a `usize`.
///
/// Generally you implement this via the [`super::define_index_type!`] macro, rather
/// than manually implementing it.
///
/// # Overflow
///
/// `NonZeroIdx` impls are allowed to be smaller than `usize`, which means converting
/// `usize` to an `NonZeroIdx` implementation might have to handle overflow.
///
/// The way overflow is handled is up to the implementation of `NonZeroIdx`, but it's
/// generally panicking, unless it was turned off via the
/// `DISABLE_MAX_INDEX_CHECK` option in [`super::define_index_type!`]. If you need more
/// subtle handling than this, then you're on your own (or, well, either handle
/// it earlier, or pick a bigger index type).
///
/// # Zero checks
///
/// `NonZeroIdx` impls are all, well "non-zero"; Which means it is always possible
/// for a `usize` value to be 0 and we can not produce a valid `NonZeroIdx` from it.
///
///
/// The way this is handled similar to `overflow` is up to the implementation of `NonZeroIdx`,
/// but it's generally panicking, unless it was turned off via the
/// `DISABLE_NON_ZERO_CHECK` option in [`super::define_index_type!`]. If you need more
/// subtle handling than this, then you're on your own (or, well, either handle
/// it earlier, or pick a bigger index type).
///
pub trait NonZeroIdx: Copy + 'static + Ord + Debug + Hash {
    /// Construct an Index from a `usize`. This is equivalent to `From<usize>`.
    ///
    /// # Panics
    ///
    /// Note that this will panic if `idx` does not fit (unless checking has
    /// been disabled, as mentioned above). Also note that `NonZeroIdx` implementations
    /// are free to define what "fit" means as they desire.
    fn from_usize(idx: usize) -> Self;

    /// Construct an Index from a `usize`. This is equivalent to `From<usize>`.
    ///
    /// SAFETY: `idx` shouldn't be `zero`. It should also fit.
    #[allow(unsafe_code)]
    unsafe fn from_usize_unchecked(idx: usize) -> Self;

    /// Get the underlying index. This is equivalent to `Into<usize>`
    fn index(self) -> usize;
}
