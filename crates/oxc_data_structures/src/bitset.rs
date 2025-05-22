//! Fixed size bitset with typed indices.

use std::{
    iter::FusedIterator,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Mul, Not, Range, Shl},
    ptr,
};

use crate::assert_unchecked;

/// Macro to create a fixed-size bitset, and an index type for indexing into it.
///
/// Similar to `bitflags!` macro, but bitself can be any length, and indices don't need to be named.
///
/// Can be created with any size, as long as the size is known at compile time (a const).
///
/// It is impossible to obtain an instance of the index type which is greater than the length of
/// the bitset, so all operations can use unchecked indexing. This makes it very performant.
///
/// All methods for getting/setting bits are const.
///
/// See [`Bitset`] (which is the type which is the underlying storage of bitsets created with this macro)
/// for details of all methods.
///
/// # Example
///
// Note: This should compile, but there appears to be a bug in doctests runner.
/// ```no_compile
/// use oxc_data_structures::bitset::bitset;
///
/// struct Fruit {
///     name: &'static str,
///     delicousness: u32,
/// }
///
/// static FRUITS: &[Fruit] = &[
///     Fruit { name: "banana", delicousness: 10 },
///     Fruit { name: "apple", delicousness: 2 },
///     Fruit { name: "lemon", delicousness: 5 },
///     Fruit { name: "peach", delicousness: u32::MAX },
/// ];
///
/// // `pub` makes both the bitset type `FruitBowl` and the index type `FruitId` public.
/// // Where the bitset is small in size, it's recommended to derive `Copy`
/// bitset! {
///     #[derive(Copy)]
///     pub bitset FruitBowl<FruitId>( FRUITS.len() );
/// }
///
/// const BANANA: FruitId = FruitId::new(0);
/// const APPLE: FruitId = FruitId::new(1);
/// const LEMON: FruitId = FruitId::new(2);
/// const PEACH: FruitId = FruitId::new(3);
///
/// let mut fruit_bowl = FruitBowl::empty();
/// fruit_bowl.add(BANANA);
/// fruit_bowl.add(LEMON);
/// fruit_bowl.set(PEACH, true);
/// fruit_bowl.remove(LEMON);
///
/// assert!(fruit_bowl.has(BANANA));
/// assert!(fruit_bowl.has(PEACH));
/// assert!(!fruit_bowl.has(LEMON));
/// assert!(!fruit_bowl.has(APPLE));
///
/// for (fruit_id, is_in_bowl) in &fruit_bowl {
///     println!(
///         "{} {} in the bowl",
///         FRUITS[fruit_id.to_usize()].name,
///         if is_in_bowl { "is" } else { "isn't" }
///     );
/// }
///
/// for fruit_id in fruit_bowl.true_bits_iter() {
///     // Only banana and peach get squeezed
///     println!("squeezing {}", FRUITS[fruit_id.to_usize()].name);
/// }
/// ```
//
// Note: `[<__bitset_ $Ty>]` syntax in macro below is processed by `paste!` macro
// to concatenate idents. If `$Ty` is `Foo`, `[<__bitset_ $Ty>]` creates ident `__bitset_Foo`.
#[macro_export]
macro_rules! bitset {
    // With visiibility modifier.
    // e.g. `pub bitset Foo<FooId>(8);`
    // or `#[derive(Copy)] pub(crate) bitset Foo<FooId>(8);`
    ( $(#[$($attr:tt)+])* $vis:vis bitset $Ty:ident<$Index:ident>($bits:expr) $(;)? ) => {
        bitset!( @ <$(#[$($attr)+])*> [$vis] $Ty, $Index, $bits );
    };

    // Without visiibility modifier.
    // e.g. `bitset Foo<FooId>(8);`
    // or `#[derive(Copy)] bitset Foo<FooId>(8);`
    ( $(#[$($attr:tt)+])* bitset $Ty:ident<$Index:ident>($bits:expr) $(;)? ) => {
        bitset!( @ <$(#[$($attr)+])*> $Ty, $Index, $bits );
    };

    // Implementation.
    // e.g. `@ <#[derive(Copy)]> [pub(crate)] Foo, FooId, 8`
    ( @ <$(#[$($attr:tt)+])*> $([$vis:vis])? $Ty:ident, $Index:ident, $bits:expr ) => {
        $crate::bitset::__private::paste! {
            // Bring in `$bits` from context outside macro
            #[allow(non_upper_case_globals, clippy::allow_attributes)]
            const [<__bitset_ $Ty _BITS>]: usize = $bits;

            // Wrap all remaining code in a module to prevent code outside this macro from accessing
            // fields of bitset or index types. This is necessary for safety invariant that you cannot
            // obtain an index type for an index which is out of bounds.
            #[allow(non_snake_case, clippy::allow_attributes)]
            mod [<__bitset_ $Ty>] {
                use std::{fmt::{self, Debug}, iter::Map, ops::{Index, Range}};
                use $crate::{assert_unchecked, bitset::{Bitset, BitsIter, TrueBitsIter}};

                // Calculate number of units required, and check `BITS` fits in a `u32`
                const BITS: usize = super::[<__bitset_ $Ty _BITS>];
                const _: () = assert!(BITS <= u32::MAX as usize, "`BITS` must be `<= `u32::MAX`");
                #[allow(clippy::cast_possible_truncation)]
                const BITS_U32: u32 = BITS as u32;
                const UNITS: usize = BITS.div_ceil(usize::BITS as usize);

                // --------------------------------------------------
                // Index type
                // --------------------------------------------------

                #[doc = concat!(" Index type for indexing into [`", stringify!($Ty), "`] bitset.")]
                // Note: Must *not* implement `Default` for index type, because `BITS` could be 0.
                // In that case an index of 0 would be out of bounds. So we must not allow creating it.
                #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
                #[repr(transparent)]
                $($vis)? struct $Index(u32);

                impl $Index {
                    #[doc = concat!(" Create an [`", stringify!($Index), "`] from a `usize`.")]
                    ///
                    /// # Panics
                    #[doc = concat!(" Panics if `index` is greater than number of bits in [`", stringify!($Ty), "`].")]
                    #[inline(always)]
                    pub const fn new(index: usize) -> Self {
                        assert!(index < BITS, "`index` is out of range");
                        // No valid index can exceed `u32::MAX` due to const assertion above
                        #[allow(clippy::cast_possible_truncation)]
                        let index = index as u32;
                        Self(index)
                    }

                    #[doc = concat!(" Create an [`", stringify!($Index), "`] from a `usize` without checks.")]
                    ///
                    /// # SAFETY
                    #[doc = concat!(" `index` must be less than number of bits in [`", stringify!($Ty), "`].")]
                    #[inline(always)]
                    pub const fn new_unchecked(index: usize) -> Self {
                        // No valid index can exceed `u32::MAX` due to const assertion above
                        #[allow(clippy::cast_possible_truncation)]
                        let index = index as u32;
                        Self(index)
                    }

                    #[doc = concat!(" Convert [`", stringify!($Index), "`] to `u32`")]
                    #[inline(always)]
                    pub const fn to_u32(self) -> u32 {
                        let index = self.0;
                        // Communicate upper bound to compiler, which may help it optimize calling code.
                        // SAFETY: It's impossible to create an instance of index type which is > `BITS`.
                        unsafe { assert_unchecked!(index < BITS_U32) };
                        index
                    }

                    #[doc = concat!(" Convert [`", stringify!($Index), "`] to `usize`")]
                    #[inline(always)]
                    pub const fn to_usize(self) -> usize {
                        let index = self.0 as usize;
                        // Communicate upper bound to compiler, which may help it optimize calling code.
                        // SAFETY: It's impossible to create an instance of index type which is > `BITS`.
                        unsafe { assert_unchecked!(index < BITS) };
                        index
                    }
                }

                impl From<$Index> for u32 {
                    #[inline(always)]
                    fn from(index: $Index) -> u32 {
                        index.to_u32()
                    }
                }

                impl From<$Index> for usize {
                    #[inline(always)]
                    fn from(index: $Index) -> usize {
                        index.to_usize()
                    }
                }

                // --------------------------------------------------
                // Bitset type
                // --------------------------------------------------

                $(#[$($attr)+])*
                #[doc = concat!(" [`", stringify!($Ty), "`] bit set.")]
                #[derive(Clone, PartialEq, Eq, Default, Hash)]
                #[repr(transparent)]
                $($vis)? struct $Ty(Bitset<BITS, UNITS>);

                impl $Ty {
                    #[doc = concat!(" Create an empty [`", stringify!($Ty), "`] with no bits set.")]
                    #[inline]
                    pub const fn empty() -> Self {
                        Self(Bitset::empty())
                    }

                    #[doc = concat!(" Create a [`", stringify!($Ty), "`] with all bits set.")]
                    #[inline]
                    pub const fn all() -> Self {
                        Self(Bitset::all())
                    }

                    /// Returns `true` if all bits are `false`.
                    #[inline]
                    pub const fn is_empty(&self) -> bool {
                        self.0.is_empty()
                    }

                    /// Get value of the bit at `index`.
                    #[inline]
                    pub const fn get(&self, index: $Index) -> bool {
                        // SAFETY: It's impossible to create an instance of index type which is > `BITS`
                        unsafe { self.0.get(index.0 as usize) }
                    }

                    /// Get value of the bit at `index`.
                    ///
                    /// Alias for [`get`].
                    ///
                    /// [`get`]: Self::get
                    #[inline]
                    pub const fn has(&self, index: $Index) -> bool {
                        self.get(index)
                    }

                    /// Set bit at `index` to `value`.
                    #[inline]
                    pub const fn set(&mut self, index: $Index, value: bool) {
                        // SAFETY: It's impossible to create an instance of index type which is > `BITS`
                        unsafe { self.0.set(index.0 as usize, value) }
                    }

                    /// Set bit at `index` to `true`.
                    pub const fn add(&mut self, index: $Index) {
                        // SAFETY: It's impossible to create an instance of index type which is > `BITS`
                        unsafe { self.0.add(index.0 as usize) };
                    }

                    /// Set bit at `index` to `false`.
                    pub const fn remove(&mut self, index: $Index) {
                        // SAFETY: It's impossible to create an instance of index type which is > `BITS`
                        unsafe { self.0.remove(index.0 as usize) };
                    }

                    #[doc = concat!(" Return a new [`", stringify!($Ty), "`] with bit at `index` set to `true`.")]
                    #[doc = concat!(" Original `", stringify!($Ty), "` is not modified.")]
                    #[must_use]
                    pub const fn with(self, index: $Index) -> Self {
                        // SAFETY: It's impossible to create an instance of index type which is > `BITS`
                        Self(unsafe { self.0.with(index.0 as usize) })
                    }

                    #[doc = concat!(" Return a new [`", stringify!($Ty), "`] with bit at `index` set to `false`.")]
                    #[doc = concat!(" Original `", stringify!($Ty), "` is not modified.")]
                    #[must_use]
                    pub const fn without(self, index: $Index) -> Self {
                        // SAFETY: It's impossible to create an instance of index type which is > `BITS`
                        Self(unsafe { self.0.without(index.0 as usize) })
                    }

                    #[doc = concat!(" Returns an iterator over all valid [`", stringify!($Index), "`] indices")]
                    #[doc = concat!(" of bits in a [`", stringify!($Ty), "`].")]
                    pub fn indices() -> Map<Range<usize>, fn(usize) -> $Index> {
                        // SAFETY: All numbers in range `0..BITS` are less than `BITS`
                        (0..BITS).map(|index| $Index(index as u32))
                    }

                    #[doc = concat!(" Returns an iterator over the [`", stringify!($Index), "`] indices")]
                    #[doc = concat!(" and values of all bits in the [`", stringify!($Ty), "`].")]
                    pub fn iter(&self) -> Map<BitsIter<'_, BITS, UNITS>, fn((usize, bool)) -> ($Index, bool)> {
                        // SAFETY: `BitsIter` only returns indices which are in bounds.
                        // No valid index can exceed `u32::MAX` due to const assertion above.
                        self.0.iter().map(|(index, value)| ($Index(index as u32), value))
                    }

                    #[doc = concat!(" Returns an iterator over the [`", stringify!($Index), "`] indices")]
                    #[doc = concat!(" of bits in the [`", stringify!($Ty), "`] which are `true`.")]
                    pub fn true_bits_iter(&self) -> Map<TrueBitsIter<'_, BITS, UNITS>, fn(usize) -> $Index> {
                        // SAFETY: `TrueBitsIter` only returns indices which are in bounds.
                        // No valid index can exceed `u32::MAX` due to const assertion above.
                        self.0.true_bits_iter().map(|index| $Index(index as u32))
                    }
                }

                impl<'b> IntoIterator for &'b $Ty {
                    type Item = ($Index, bool);
                    type IntoIter = Map<BitsIter<'b, BITS, UNITS>, fn((usize, bool)) -> ($Index, bool)>;

                    #[inline]
                    fn into_iter(self) -> Self::IntoIter {
                        self.iter()
                    }
                }

                impl Index<$Index> for $Ty {
                    type Output = bool;

                    fn index(&self, index: $Index) -> &bool {
                        // SAFETY: It's impossible to create an instance of index type which is > `BITS`
                        let value = unsafe { self.0.get(index.0 as usize) };
                        if value { &true } else { &false }
                    }
                }
            }

            $($vis)? use [<__bitset_ $Ty>]::*;
        }
    };
}
pub use bitset;

// Used in `bitset!` macro for building concatenated idents
#[doc(hidden)]
pub mod __private {
    pub use paste::paste;
}

/// Fixed size bitset.
///
/// This is the underlying implementation for bitsets created with the [`bitset!` macro].
///
/// Using the macro to create a bitset is preferable, as it offers custom index types and safe methods,
/// wheras all methods of [`Bitset`] are unsafe.
///
/// Bitset will be whatever size is required to contain `BITS` bits, but rounded up to a multiple of
/// the size of `usize`, and with pointer alignment for fast access.
/// Minimum size of a bitset is the size of `usize` (8 bytes on 64-bit systems).
///
/// [`bitset!` macro]: self::bitset
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Bitset<const BITS: usize, const BYTES: usize, const UNITS: usize, Unit: UnsignedInt> {
    units: [Unit; UNITS],
}

impl<const BITS: usize, const BYTES: usize, const UNITS: usize, Unit: UnsignedInt>
    Bitset<BITS, BYTES, UNITS, Unit>
{
    /// Create an empty [`Bitset`] with no bits set.
    #[inline]
    pub const fn empty() -> Self {
        // Assert `BITS` and `UNITS` correspond to each other.
        // This invariant is required to ensure soundness of other methods.
        const {
            assert!(UNITS == BITS.div_ceil(Unit::BITS), "`BITS` does not match `UNITS` and `Unit`");
            assert!(
                BYTES == BITS.div_ceil(Unit::BYTES),
                "`BYTES` does not match `UNITS` and `Unit`"
            );
        }

        Self { units: [Unit::ZERO; UNITS] }
    }

    /// Create a [`Bitset`] with all bits set.
    #[inline]
    pub const fn all() -> Self {
        let mut out = Self::empty();
        let mut index = 0;
        while index < BITS {
            // SAFETY: `index < BITS`
            unsafe { out.set(index, true) };
            index += 1;
        }
        out
    }

    /// Returns `true` if all bits are `false`.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        let mut i = 0;
        while i < BYTES {
            // SAFETY: TODO
            let byte = unsafe {
                *ptr::from_ref(&self.units).cast::<u8>().add(i).as_ref().unwrap_unchecked()
            };
            if byte != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    /// Get value of the bit at `index`.
    ///
    /// # SAFETY
    /// `index` must be less than `BITS`.
    #[inline]
    pub const unsafe fn get(&self, index: usize) -> bool {
        // SAFETY: Caller guarantees `index` is less than `BITS`
        let (unit_index, mask) = unsafe { Self::unit_index_and_mask(index) };
        // Use pointer ops here because `slice.get_unchecked` is not supported in const contexts.
        // SAFETY: `unit_index_and_mask` always returns `unit_index` which is less than `UNITS`
        // so in bounds of `self.units`.
        let unit = unsafe {
            *ptr::from_ref(&self.units).cast::<Unit>().add(unit_index).as_ref().unwrap_unchecked()
        };
        (unit & mask) != Unit::ZERO
    }

    /// Get value of the bit at `index`.
    ///
    /// Alias for [`get`].
    ///
    /// # SAFETY
    /// `index` must be less than `BITS`.
    ///
    /// [`get`]: Self::get
    #[inline]
    pub const unsafe fn has(&self, index: usize) -> bool {
        // SAFETY: `get` has same safety requirements has this method
        unsafe { self.get(index) }
    }

    /// Set bit at `index` to `value`.
    ///
    /// # SAFETY
    /// `index` must be less than `BITS`.
    #[inline]
    pub const unsafe fn set(&mut self, index: usize, value: bool) {
        // SAFETY: Caller guarantees `index` is less than `BITS`
        let (unit_index, mask) = unsafe { Self::unit_index_and_mask(index) };
        // Use pointer ops here because `slice.get_unchecked` is not supported in const contexts.
        // SAFETY: `unit_index_and_mask` always returns `unit_index` which is less than `UNITS`
        // so in bounds of `self.units`.
        let unit_mut = unsafe {
            ptr::from_mut(&mut self.units)
                .cast::<Unit>()
                .add(unit_index)
                .as_mut()
                .unwrap_unchecked()
        };
        *unit_mut &= !mask;
        *unit_mut |= if value { mask } else { Unit::ZERO };
    }

    /// Get unit index and mask for a bit (internal method).
    ///
    /// Unit index returned will always be less than `UNITS`.
    ///
    /// # SAFETY
    /// `index` must be less than `BITS`.
    #[inline(always)]
    const unsafe fn unit_index_and_mask(
        index: usize,
    ) -> (/* unit index */ usize, /* bit mask */ Unit) {
        // Inform compiler of upper bound on `index`.
        // If `BITS <= Unit::BITS` (i.e. bitset is a single `Unit`), it can deduce that `unit_index`
        // is always 0. https://godbolt.org/z/3sesq9hW3
        // SAFETY: Caller guarantees `index` must be less than `BITS`.
        unsafe { assert_unchecked!(index < BITS) };

        let unit_index = index / Unit::BITS;
        let mask = Unit::ONE << (index & (Unit::BITS - 1));
        (unit_index, mask)
    }

    /// Set bit at `index` to `true`.
    ///
    /// # SAFETY
    /// `index` must be less than `BITS`.
    pub const unsafe fn add(&mut self, index: usize) {
        // SAFETY: Caller guarantees `index` must be less than `BITS`
        unsafe { self.set(index, true) };
    }

    /// Set bit at `index` to `false`.
    ///
    /// # SAFETY
    /// `index` must be less than `BITS`.
    pub const unsafe fn remove(&mut self, index: usize) {
        // SAFETY: Caller guarantees `index` must be less than `BITS`
        unsafe { self.set(index, false) };
    }

    /// Return a new [`Bitset`] with bit at `index` set to `true`.
    /// Original `Bitset` is not modified.
    ///
    /// # SAFETY
    /// `index` must be less than `BITS`.
    #[must_use]
    pub const unsafe fn with(mut self, index: usize) -> Self {
        // SAFETY: Caller guarantees `index` must be less than `BITS`
        unsafe { self.set(index, true) };
        self
    }

    /// Return a new [`Bitset`] with bit at `index` set to `false`.
    /// Original `Bitset` is not modified.
    ///
    /// # SAFETY
    /// `index` must be less than `BITS`.
    #[must_use]
    pub const unsafe fn without(mut self, index: usize) -> Self {
        // SAFETY: Caller guarantees `index` must be less than `BITS`
        unsafe { self.set(index, false) };
        self
    }

    /// Returns an iterator over all valid indices of bits in the [`Bitset`].
    pub const fn indices() -> Range<usize> {
        0..BITS
    }

    /// Returns an iterator over the indices of bits in the [`Bitset`], and their values.
    pub fn iter(&self) -> BitsIter<'_, BITS, BYTES, UNITS, Unit> {
        BitsIter::new(self)
    }

    /// Returns an iterator over the indices of bits in the [`Bitset`] which are `true`.
    pub fn true_bits_iter(&self) -> TrueBitsIter<'_, BITS, BYTES, UNITS, Unit> {
        TrueBitsIter::new(self)
    }
}

impl<const BITS: usize, const BYTES: usize, const UNITS: usize, Unit: UnsignedInt> Default
    for Bitset<BITS, BYTES, UNITS, Unit>
{
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl<const BITS: usize, const BYTES: usize, const UNITS: usize, Unit: UnsignedInt> Index<usize>
    for Bitset<BITS, BYTES, UNITS, Unit>
{
    type Output = bool;

    /// # Panics
    /// Panics if `index >= BITS`.
    fn index(&self, index: usize) -> &bool {
        assert!(index < BITS, "index out of bounds");
        // SAFETY: Just checked `index < BITS`
        let value = unsafe { self.get(index) };
        if value { &true } else { &false }
    }
}

/// Iterator over the bits in a [`Bitset`].
pub struct BitsIter<
    'b,
    const BITS: usize,
    const BYTES: usize,
    const UNITS: usize,
    Unit: UnsignedInt,
> {
    bitset: &'b Bitset<BITS, BYTES, UNITS, Unit>,
    next_index: usize,
}

impl<'b, const BITS: usize, const BYTES: usize, const UNITS: usize, Unit: UnsignedInt>
    BitsIter<'b, BITS, BYTES, UNITS, Unit>
{
    /// Create new [`BitsIter`] from a [`Bitset`].
    fn new(bitset: &'b Bitset<BITS, BYTES, UNITS, Unit>) -> Self {
        Self { bitset, next_index: 0 }
    }
}

impl<const BITS: usize, const BYTES: usize, const UNITS: usize, Unit: UnsignedInt> Iterator
    for BitsIter<'_, BITS, BYTES, UNITS, Unit>
{
    type Item = (usize, bool);

    fn next(&mut self) -> Option<(usize, bool)> {
        if self.next_index < BITS {
            let index = self.next_index;
            self.next_index += 1;
            // SAFETY: `index < BITS`
            let value = unsafe { self.bitset.get(index) };
            Some((index, value))
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = BITS - self.next_index;
        // Communicate upper bound to compiler, which may help it optimize calling code.
        // SAFETY: `next_index` cannot be more than `BITS`, therefore subtraction above cannot wrap,
        // and `remaining` cannot be more than `BITS` either.
        unsafe { assert_unchecked!(remaining <= BITS) };
        (remaining, Some(remaining))
    }
}

impl<const BITS: usize, const BYTES: usize, const UNITS: usize, Unit: UnsignedInt> ExactSizeIterator
    for BitsIter<'_, BITS, BYTES, UNITS, Unit>
{
}

impl<const BITS: usize, const BYTES: usize, const UNITS: usize, Unit: UnsignedInt> FusedIterator
    for BitsIter<'_, BITS, BYTES, UNITS, Unit>
{
}

impl<'b, const BITS: usize, const BYTES: usize, const UNITS: usize, Unit: UnsignedInt> IntoIterator
    for &'b Bitset<BITS, BYTES, UNITS, Unit>
{
    type Item = (usize, bool);
    type IntoIter = BitsIter<'b, BITS, BYTES, UNITS, Unit>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BitsIter::new(self)
    }
}

/// Iterator over the bits which are `true` in a [`Bitset`].
pub struct TrueBitsIter<
    'b,
    const BITS: usize,
    const BYTES: usize,
    const UNITS: usize,
    Unit: UnsignedInt,
> {
    bitset: &'b Bitset<BITS, BYTES, UNITS, Unit>,
    next_index: usize,
}

impl<'b, const BITS: usize, const BYTES: usize, const UNITS: usize, Unit: UnsignedInt>
    TrueBitsIter<'b, BITS, BYTES, UNITS, Unit>
{
    /// Create new [`TrueBitsIter`] from a [`Bitset`].
    fn new(bitset: &'b Bitset<BITS, BYTES, UNITS, Unit>) -> Self {
        Self { bitset, next_index: 0 }
    }
}

impl<const BITS: usize, const BYTES: usize, const UNITS: usize, Unit: UnsignedInt> Iterator
    for TrueBitsIter<'_, BITS, BYTES, UNITS, Unit>
{
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        while self.next_index < BITS {
            let index = self.next_index;
            self.next_index += 1;

            // SAFETY: `index < BITS`
            if unsafe { self.bitset.get(index) } {
                return Some(index);
            }
        }

        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let max_remaining = BITS - self.next_index;
        // Communicate upper bound to compiler, which may help it optimize calling code.
        // SAFETY: `next_index` cannot be more than `BITS`, therefore subtraction above cannot wrap,
        // and `max_remaining` cannot be more than `BITS` either.
        unsafe { assert_unchecked!(max_remaining <= BITS) };
        (0, Some(max_remaining))
    }
}

impl<const BITS: usize, const BYTES: usize, const UNITS: usize, Unit: UnsignedInt> FusedIterator
    for TrueBitsIter<'_, BITS, BYTES, UNITS, Unit>
{
}

struct Unit<const BYTES: usize>;

trait ValidUnit {
    type Uint: Copy
        + Eq
        + PartialEq
        + BitAnd
        + BitAndAssign
        + BitOr
        + BitOrAssign
        + Not<Output = Self::Uint>
        + Shl<usize>
        + Sealed;
}

macro_rules! impl_valid_unit {
    ($ty:ident, $bits:literal) => {
        impl ValidUnit for Unit<$bits> {
            type Uint = $ty;
        }
    };
}

impl_valid_unit!(u8, 1);
impl_valid_unit!(u16, 2);
impl_valid_unit!(u32, 3);
impl_valid_unit!(u64, 4);
impl_valid_unit!(u128, 5);

/// TODO: Docs
#[expect(private_bounds)]
pub trait UnsignedInt:
    Copy
    + Eq
    + PartialEq
    + BitAnd
    + BitAndAssign
    + BitOr
    + BitOrAssign
    + Not<Output = Self>
    + Shl<usize>
    + Sealed
{
    /// Number of bits in this integer type
    const BITS: usize;
    /// Number of bytes in this integer type
    const BYTES: usize;
    /// Zero value for this integer type
    const ZERO: Self;
    /// One value for this integer type
    const ONE: Self;
}

trait Sealed {}

macro_rules! impl_unsigned_int {
    ($ty:ident) => {
        impl UnsignedInt for $ty {
            const BITS: usize = size_of::<Self>() * 8;
            const BYTES: usize = size_of::<Self>();
            const ZERO: Self = 0;
            const ONE: Self = 1;
        }
        impl Sealed for $ty {}
    };
}

impl_unsigned_int!(u8);
impl_unsigned_int!(u16);
impl_unsigned_int!(u32);
impl_unsigned_int!(u64);
impl_unsigned_int!(u128);
