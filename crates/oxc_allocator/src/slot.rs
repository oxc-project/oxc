//! Exclusive references to places in the arena which are yet to be written.
//!
//! Where a function produces a value that its caller is going to store somewhere, it can instead
//! be handed a [`Slot`] for that place, and write the value straight into it. This saves returning
//! the value on the stack for the caller to copy into place, which matters for values too large
//! to return in registers.
//!
//! Producing the value in pieces works the same way. Each piece is written as it is known,
//! and nothing has to be kept alive in between.
//!
//! # Slot types
//!
//! There are 2 different slot types:
//!
//! * [`Slot`] covers a place that something else owns - a field of a larger value, or an element of a [`Vec`].
//! * [`OwnedSlot`] covers a value with an allocation of its own, so filling one yields a [`Box`].
//!
//! # Proving a slot was filled
//!
//! Filling a [`Slot`] yields a [`SlotFilled`] token. The token is the only evidence its holder gets
//! that the place was written, so whoever handed out the [`Slot`] demands a [`SlotFilled`] token back.
//!
//! The token carries the slot's `'slot` lifetime as a brand, so it proves that *this* slot was filled,
//! rather than that *some* slot was. A [`Slot`] is usually handed to a closure bound by
//! `for<'slot> FnOnce(Slot<'slot, T>) -> SlotFilled<'slot>`, making `'slot` universally quantified,
//! so no token the closure obtains elsewhere can stand in for this one.
//!
//! Narrowing a [`Slot`] - [`into_some`], [`into_contents`], [`into_part`] - passes the token on to
//! the narrower slot, so one token at the end stands for every level.
//!
//! # Abandoning a slot
//!
//! Nothing happens if a [`Slot`] is dropped without being filled, or unwound past by a panic.
//! Whoever owns the place never gets a token, so never learns it was written, and does not read it.
//! A value already written into a wider place the slot was narrowed from is not dropped either -
//! for arena data, which is never dropped, that costs nothing.
//!
//! What is left behind is not simply uninitialized. Narrowing writes as it goes - a discriminant, a [`Box`] -
//! so an abandoned slot can leave a place holding neither the value it held before nor a valid one.
//! This is why [`Slot::new`] requires that nothing else can read the place - whoever could would be
//! reading an uninitiatized or partially-initialized value.
//!
//! [`Box`]: crate::Box
//! [`Vec`]: crate::Vec
//! [`into_some`]: Slot::into_some
//! [`into_contents`]: Slot::into_contents
//! [`into_part`]: Slot::into_part

#![expect(clippy::inline_always)]

use std::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use crate::{Box, GetAllocator};

/// A place a `T` is written into, and what taking the result yields.
///
/// Abstracts over the two kinds of slot, for code which writes a `T` a piece at a time,
/// and should not have to care which of them it is writing into:
///
/// * [`Slot`] - the `T` goes in a place someone else owns, so [`assume_filled`] returns a [`SlotFilled`] token.
/// * [`OwnedSlot`] - owns its allocation, so [`assume_filled`] returns a [`Box`].
///
/// These methods are not intended for use in user code, as the semantics are subtle.
/// They should be used as primitives for building other safe mechanisms for constructing types.
/// These methods are only accessible with [`FillSlot`] trait in scope, which helps to hide them
/// from general public API.
///
/// Sealed, so [`Slot`] and [`OwnedSlot`] are the only implementors there can be.
/// [`as_mut_ptr`] is a *safe* method whose pointer callers write through in `unsafe` code,
/// on the strength of the guarantee documented on it. A trait anyone could implement cannot
/// make that guarantee - an implementation returning a dangling pointer would turn those
/// `unsafe` writes into undefined behaviour, without containing any `unsafe` itself.
/// Sealing is what makes the guarantee a statement about 2 known implementations,
/// rather than a promise implementors are trusted to keep.
///
/// [`assume_filled`]: Self::assume_filled
/// [`as_mut_ptr`]: Self::as_mut_ptr
/// [`Box`]: crate::Box
pub trait FillSlot<T>: Sealed {
    /// Type which [`assume_filled`] returns.
    ///
    /// [`assume_filled`]: Self::assume_filled
    type Output;

    /// Pointer to the place the `T` goes, for writing it a part at a time.
    ///
    /// The pointer is derived from the place's own exclusive reference, so writing through it is sound -
    /// but only until the place is used again. Any later use, including another call to this method,
    /// invalidates it. Get a fresh pointer per write rather than holding one.
    fn as_mut_ptr(&mut self) -> *mut T;

    /// Reborrow as a [`Slot`], for a shorter lifetime.
    ///
    /// The reborrow ends when the returned [`Slot`] is dropped, after which this one is usable again -
    /// which is what lets a holder lend the place out and still take the result afterwards.
    ///
    /// The returned slot gets a `'slot` brand of its own, so filling it does not produce a token for this place.
    /// Only the holder of this one can say whether the `T` ended up initialized.
    ///
    /// That brand is borrowed from this slot, so a token for it cannot outlive the reborrow,
    /// let alone be handed back as proof about the place reborrowed from:
    ///
    /// ```compile_fail
    /// use oxc_allocator::{Allocator, FillSlot, OwnedSlot};
    ///
    /// let allocator = Allocator::new();
    /// let allocator = &allocator;
    ///
    /// let slot = OwnedSlot::<u32>::new_in(&allocator);
    /// // error[E0515]: cannot return value referencing function parameter `slot`
    /// let _boxed = slot.fill_with(|mut slot| slot.reborrow().fill(123));
    /// ```
    #[expect(clippy::elidable_lifetime_names)]
    fn reborrow<'borrow>(&'borrow mut self) -> Slot<'borrow, T>;

    /// Take what this place yields once the `T` has been written - a [`Box`] if it owns the allocation,
    /// a [`SlotFilled`] token if it does not.
    ///
    /// # SAFETY
    /// The `T` must be fully initialized.
    ///
    /// [`Box`]: crate::Box
    unsafe fn assume_filled(self) -> Self::Output;
}

/// Private module, so [`Sealed`] cannot be implemented outside of it,
/// and therefore neither can [`FillSlot`], which has it as a supertrait.
mod private {
    use super::{OwnedSlot, Slot};

    /// Sealing trait, implemented only for [`Slot`] and [`OwnedSlot`].
    pub trait Sealed {}
    impl<T> Sealed for Slot<'_, T> {}
    impl<T> Sealed for OwnedSlot<'_, T> {}
}
use private::Sealed;

/// Exclusive reference to a place where a `T` is to be written, but has not been yet.
///
/// Handed to whoever is to produce the `T`, so that they write it straight into
/// its final location in memory, instead of returning it for the caller to copy there.
///
/// Either [`fill`] it with a value, or build the `T` into it in pieces.
/// Either way, the result is a [`SlotFilled`] token, which is how the holder of the slot
/// learns that it was filled.
///
/// [`fill`]: Slot::fill
#[must_use]
#[repr(transparent)]
pub struct Slot<'slot, T> {
    /// The `T` (maybe uninitialized) that the [`Slot`] covers.
    ///
    /// A reference, not a pointer, so that it carries `noalias` when a [`Slot`] is passed to
    /// a function which does not get inlined. Without it, the compiler has to assume that writing
    /// through the slot could alias the arena's bump pointer - which lives in a `Cell`,
    /// so is not `noalias` itself - and reload it after every write.
    ///
    /// `&mut` is invariant in `T`, which is required. Whatever contains this place is still typed
    /// with the original lifetime, so shortening `T` and filling the slot with a shorter-lived value
    /// would let that value escape as longer-lived.
    content: &'slot mut MaybeUninit<T>,

    /// `&mut` is *covariant* in its lifetime, so it does not on its own keep `'slot` invariant.
    /// Holding the token does. `'slot` brands the slot, and a brand which could be narrowed
    /// would let a token for one slot stand for another.
    filled_token: SlotFilled<'slot>,
}

impl<'slot, T> Slot<'slot, T> {
    /// Create a new [`Slot`] for the `T` that `ptr` points to.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be non-null, and point to memory sized and aligned for a `T`.
    /// * That memory must stay valid for `'slot`.
    /// * That memory must only be accessed through this [`Slot`], or slots derived from it, for `'slot`.
    /// * Nothing may read that memory unless this [`Slot`] is filled - during `'slot`, or after it ends.
    ///   A slot can be abandoned without being filled, by being dropped or unwound past by a panic,
    ///   and nothing puts the memory back as it was. Narrowing it first ([`into_part`], [`into_contents`])
    ///   can have written part of a value already, so what an abandoned slot leaves behind is neither
    ///   what was there before, nor a valid `T`.
    /// * `'slot` must not brand any other [`Slot`]. Filling this one yields a [`SlotFilled`]
    ///   token for `'slot`, which stands as proof that the slot branded `'slot` was filled,
    ///   so a second slot sharing the brand would be covered by that same proof.
    ///
    /// [`into_part`]: Slot::into_part
    /// [`into_contents`]: Slot::into_contents
    //
    // `#[inline(always)]` because this is a no-op at runtime
    #[inline(always)]
    pub unsafe fn new(ptr: *mut T) -> Self {
        // SAFETY: Caller guarantees `ptr` is non-null
        let ptr = unsafe { NonNull::new_unchecked(ptr) };

        // SAFETY: Caller guarantees `ptr` points to memory laid out for a `T`, which stays valid for `'slot`
        // and is accessed only through this `Slot` for that time, so the reference neither dangles nor aliases.
        // `MaybeUninit<T>` has no validity requirements, so the memory does not have to be initialized.
        let content = unsafe { ptr.cast::<MaybeUninit<T>>().as_mut() };

        Self { content, filled_token: SlotFilled::new() }
    }

    /// Fill the [`Slot`] with `value`.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_allocator::{Allocator, OwnedSlot, Slot, SlotFilled};
    ///
    /// // Writes the value straight into its final place, instead of returning it
    /// // for the caller to copy there
    /// fn make_value<'slot>(slot: Slot<'slot, [u64; 8]>) -> SlotFilled<'slot> {
    ///     slot.fill([123; 8])
    /// }
    ///
    /// let allocator = Allocator::new();
    /// let allocator = &allocator;
    ///
    /// let boxed = OwnedSlot::new_in(&allocator).fill_with(make_value);
    /// assert_eq!(*boxed, [123; 8]);
    /// ```
    //
    // `#[inline(always)]` because this is a single store
    #[inline(always)]
    pub fn fill(self, value: T) -> SlotFilled<'slot> {
        self.content.write(value);
        self.filled_token
    }

    /// Narrow the [`Slot`] to one for a part of the `T`, `byte_offset` bytes from its start.
    ///
    /// The narrowed slot carries this slot's [`SlotFilled`] token, so filling the narrowed slot
    /// discharges the obligation to fill this [`Slot`].
    ///
    /// The caller must first write whatever else the `T` needs to be valid e.g. an enum discriminant, other fields.
    ///
    /// # SAFETY
    ///
    /// * `byte_offset + size_of::<P>()` must not exceed `size_of::<T>()`.
    /// * `byte_offset` must be a multiple of `align_of::<P>()`, and `align_of::<P>()` must not
    ///   exceed `align_of::<T>()`, so that the place is aligned for a `P`.
    /// * Writing a valid `P` there, together with whatever the caller has already written,
    ///   must leave a valid, fully initialized `T`.
    //
    // `#[inline(always)]` because this is only a single addition op at runtime
    #[inline(always)]
    pub unsafe fn into_part<P>(self, byte_offset: usize) -> Slot<'slot, P> {
        let ptr = NonNull::from(self.content);

        // SAFETY: Caller guarantees the `P` lies within the `T` at `byte_offset`, which the slot covers,
        // and that the place there is aligned for a `P`.
        // `MaybeUninit<P>` has no validity requirements, so it need not be initialized.
        // `self.content` was moved into `NonNull::from`, so the `&mut` this pointer is derived
        // from is gone, and this new one is the only reference to the memory.
        let part = unsafe { ptr.byte_add(byte_offset).cast::<MaybeUninit<P>>().as_mut() };

        Slot { content: part, filled_token: self.filled_token }
    }
}

impl<'slot, T> FillSlot<T> for Slot<'slot, T> {
    type Output = SlotFilled<'slot>;

    // `#[inline(always)]` because this is a no-op at runtime
    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.content.as_mut_ptr()
    }

    // `#[inline(always)]` because this is a no-op at runtime
    #[inline(always)]
    #[expect(clippy::elidable_lifetime_names)]
    fn reborrow<'borrow>(&'borrow mut self) -> Slot<'borrow, T> {
        Slot { content: &mut *self.content, filled_token: SlotFilled::new() }
    }

    // `#[inline(always)]` because this is a no-op at runtime
    #[inline(always)]
    unsafe fn assume_filled(self) -> SlotFilled<'slot> {
        // Caller guarantees the `T` is initialized, which is what this token stands for
        self.filled_token
    }
}

impl<'slot, T> Slot<'slot, Option<T>> {
    /// Narrow the [`Slot`] to one for the `Some` payload.
    ///
    /// Writes nothing, just converts the `Slot`'s type.
    ///
    /// To write `None`, [`fill`] this slot with `None` instead.
    ///
    /// `T` and `Option<T>` must have the same size (`T` has a niche).
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_allocator::{Allocator, OwnedSlot};
    ///
    /// let allocator = Allocator::new();
    /// let allocator = &allocator;
    ///
    /// // `&str` has a niche, so `Option<&str>` is the same size as `&str`.
    /// // Filling the narrowed slot leaves the place holding `Some`.
    /// let slot = OwnedSlot::<Option<&str>>::new_in(&allocator);
    /// let boxed = slot.fill_with(|slot| slot.into_some().fill("hello"));
    /// assert_eq!(*boxed, Some("hello"));
    ///
    /// // Writing `None` is just filling the original slot
    /// let slot = OwnedSlot::<Option<&str>>::new_in(&allocator);
    /// let boxed = slot.fill_with(|slot| slot.fill(None));
    /// assert_eq!(*boxed, None);
    /// ```
    ///
    /// [`fill`]: Slot::fill
    //
    // `#[inline(always)]` because this is a no-op at runtime
    #[inline(always)]
    pub fn into_some(self) -> Slot<'slot, T> {
        const {
            assert!(
                size_of::<Option<T>>() == size_of::<T>(),
                "`Slot<Option<T>>::into_some` can only be used on types where `T` and `Option<T>` are the same size"
            );
        }

        // SAFETY: `Option<T>` is the same size as `T`, so `T` fills the whole `Option`, at offset 0.
        // There are no spare bytes for a discriminant - `T` and `Some(T)` are bit identical.
        // So filling the returned slot leaves the place containing `Some`.
        // The place is aligned for `Option<T>`, whose alignment is at least `T`'s.
        unsafe { self.into_part(0) }
    }
}

impl<'slot, 'alloc, T> Slot<'slot, Box<'alloc, T>> {
    /// Reserve memory in the arena for the `T`, write the [`Box`] into the [`Slot`],
    /// and return a [`Slot`] for the `T` itself.
    ///
    /// The `T` is not initialized - only the [`Box`] pointing to it is. The obligation to fill it
    /// passes to the returned [`Slot`], whose [`SlotFilled`] token stands for both.
    ///
    /// The [`Box`] is written before the `T` is built, so this slot's reference is dead
    /// by the time building the `T` happens.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_allocator::{Allocator, ArenaBox, OwnedSlot};
    ///
    /// let allocator = Allocator::new();
    /// let allocator = &allocator;
    ///
    /// // The place holds a `Box`. `into_contents` reserves arena memory for the `u64`,
    /// // writes the `Box` pointing at it, and hands back a `Slot` for the `u64` itself.
    /// // One token at the end stands for both.
    /// let slot = OwnedSlot::<ArenaBox<u64>>::new_in(&allocator);
    /// let boxed = slot.fill_with(|slot| slot.into_contents(&allocator).fill(123));
    /// assert_eq!(**boxed, 123);
    /// ```
    //
    // `#[inline(always)]` because the allocation is a bump-pointer increment, and the rest is a single store
    #[inline(always)]
    pub fn into_contents(self, allocator: &impl GetAllocator<'alloc>) -> Slot<'slot, T> {
        let mut ptr = NonNull::from(allocator.allocator().alloc_uninit::<T>());

        // SAFETY:
        // `ptr` points into the arena, to memory sized and aligned for a `T`, which lives for `'alloc`.
        // The `Box` written here is a `Box<'alloc, T>`, so it cannot outlive that.
        //
        // The `T` is not initialized yet, which `Box::from_non_null` allows.
        // It is initialized before anything can dereference the `Box`, because the only route to that memory
        // is the `Slot` returned below, which by `Slot`'s contract nothing may read unless the `Slot` is filled.
        self.content.write(unsafe { Box::from_non_null(ptr.cast::<T>()) });

        // SAFETY: `alloc_uninit`'s reference was moved into `NonNull::from`, so `ptr` is the only
        // pointer to this memory, and the reference derived from it here is the only one.
        // The `Box` written above holds `ptr` itself, which stays valid while this reference,
        // derived from it, is used to fill the `T`.
        // The memory lives for `'alloc`, and `'alloc: 'slot` is implied by this slot holding
        // a `Box<'alloc, T>`, so the reference cannot outlive it.
        let content = unsafe { ptr.as_mut() };

        Slot { content, filled_token: self.filled_token }
    }
}

/// Exclusive reference to a value which owns its allocation in the arena, and has not been written yet.
///
/// Like a [`Slot`], but where a [`Slot`] covers a place someone else owns, this owns what it covers,
/// so filling it yields a [`Box`] rather than a [`SlotFilled`] token.
///
/// `'alloc` is not branded, unlike a [`Slot`]'s `'slot`. It is the arena's own lifetime.
/// There is no token tied to it which could be passed off as proof about a different allocation.
#[must_use]
#[repr(transparent)]
pub struct OwnedSlot<'alloc, T> {
    /// The `T` (maybe uninitialized) that the [`OwnedSlot`] covers.
    ///
    /// A reference rather than a pointer, for the same reason as [`Slot::content`] - see there.
    content: &'alloc mut MaybeUninit<T>,
}

impl<'alloc, T> OwnedSlot<'alloc, T> {
    /// Reserve memory in the arena for a `T`, without initializing it.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_allocator::{Allocator, OwnedSlot};
    ///
    /// let allocator = Allocator::new();
    /// let allocator = &allocator;
    ///
    /// // Reserve the memory now, write the `T` later
    /// let slot = OwnedSlot::<u64>::new_in(&allocator);
    /// let boxed = slot.fill(123);
    /// assert_eq!(*boxed, 123);
    /// ```
    //
    // `#[inline(always)]` because the allocation is a bump-pointer increment
    #[inline(always)]
    pub fn new_in(allocator: &impl GetAllocator<'alloc>) -> Self {
        let content = allocator.allocator().alloc_uninit::<T>();
        Self { content }
    }

    /// Create an [`OwnedSlot`] for the `T` that `ptr` points to.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must point to within an [`Allocator`].
    ///   Filling this slot yields a [`Box`], which requires it.
    /// * `ptr` must point to memory sized and aligned for a `T`.
    /// * That memory must stay valid for `'alloc`.
    /// * The returned [`OwnedSlot`] must own it - nothing else may access it for `'alloc`,
    ///   and nothing may read it unless the slot is filled. An [`OwnedSlot`] can be abandoned
    ///   without being filled, by being dropped or unwound past by a panic, and nothing puts
    ///   the memory back as it was - see [`Slot::new`], whose contract this shares.
    ///
    /// # Example
    ///
    /// ```
    /// use std::ptr::NonNull;
    /// use oxc_allocator::{Allocator, OwnedSlot};
    ///
    /// let allocator = Allocator::new();
    ///
    /// // Reserve memory for a `u64`, and take the only pointer to it
    /// let ptr = NonNull::from(allocator.alloc_uninit::<u64>()).cast::<u64>();
    ///
    /// // SAFETY: `ptr` points to memory in the arena laid out for a `u64`, which lives
    /// // as long as `allocator`. Nothing else holds a pointer to it, and nothing reads it.
    /// let slot = unsafe { OwnedSlot::from_non_null(ptr) };
    ///
    /// let boxed = slot.fill(123);
    /// assert_eq!(*boxed, 123);
    /// ```
    ///
    /// [`Allocator`]: crate::Allocator
    //
    // `#[inline(always)]` because this is a no-op at runtime
    #[inline(always)]
    pub unsafe fn from_non_null(ptr: NonNull<T>) -> Self {
        // SAFETY: Caller guarantees `ptr` points to memory laid out for a `T`, which stays valid
        // for `'alloc`, and which this `OwnedSlot` owns - so the reference neither dangles nor aliases.
        // `MaybeUninit<T>` has no validity requirements, so it need not be initialized.
        let content = unsafe { ptr.cast::<MaybeUninit<T>>().as_mut() };
        Self { content }
    }

    /// Fill an [`OwnedSlot`] with `value`, and return a [`Box<T>`] containing it.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_allocator::{Allocator, OwnedSlot};
    ///
    /// let allocator = Allocator::new();
    /// let allocator = &allocator;
    ///
    /// let boxed = OwnedSlot::new_in(&allocator).fill("hello");
    /// assert_eq!(*boxed, "hello");
    /// ```
    //
    // `#[inline(always)]` because this is a single store
    #[inline(always)]
    pub fn fill(self, value: T) -> Box<'alloc, T> {
        let content = self.content.write(value);

        // SAFETY: `content` is the `T` just written, so it is valid and initialized.
        // Every route to an `OwnedSlot` covers memory in the arena which lives for `'alloc` -
        // `new_in` allocates it there, and `from_non_null` requires it of its caller.
        // That is the lifetime of the `Box` created here.
        unsafe { Box::from_non_null(NonNull::from(content)) }
    }

    /// Fill an [`OwnedSlot`] with `fill` function, and return a [`Box<T>`].
    ///
    /// `fill` is passed a [`Slot`] for it to write into, and it must return the corresponding
    /// [`SlotFilled`] token.
    ///
    /// This is how a caller which wants the value itself calls a function which fills a [`Slot`],
    /// so that function does not have to be generic over where it writes.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_allocator::{Allocator, OwnedSlot, Slot, SlotFilled};
    ///
    /// // Writes into whatever place it is given - a field of a larger value, or as here,
    /// // one with an allocation of its own
    /// fn make_value<'slot>(slot: Slot<'slot, [u64; 8]>) -> SlotFilled<'slot> {
    ///     slot.fill([123; 8])
    /// }
    ///
    /// let allocator = Allocator::new();
    /// let allocator = &allocator;
    ///
    /// let boxed = OwnedSlot::new_in(&allocator).fill_with(make_value);
    /// assert_eq!(*boxed, [123; 8]);
    /// ```
    //
    // `#[inline(always)]` because this just delegates to `fill`
    #[inline(always)]
    pub fn fill_with(
        self,
        fill: impl for<'slot> FnOnce(Slot<'slot, T>) -> SlotFilled<'slot>,
    ) -> Box<'alloc, T> {
        // Reborrow rather than move `self.content` into the `Slot`, so that it is live again
        // once `fill` returns and can become the `Box`. The borrow checker enforces that nothing
        // else touches the memory for as long as the `Slot` exists.
        let slot = Slot { content: &mut *self.content, filled_token: SlotFilled::new() };
        let _filled_token = fill(slot);

        // SAFETY: The `SlotFilled` token `fill` returned is branded with that `Slot`'s `'slot`.
        // Only filling that slot can produce the matching `SlotFilled`, so the `T` is initialized now.
        unsafe { self.assume_filled() }
    }
}

impl<'alloc, T> FillSlot<T> for OwnedSlot<'alloc, T> {
    type Output = Box<'alloc, T>;

    // `#[inline(always)]` because this is a no-op at runtime
    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.content.as_mut_ptr()
    }

    // `#[inline(always)]` because this is a no-op at runtime
    #[inline(always)]
    #[expect(clippy::elidable_lifetime_names)]
    fn reborrow<'borrow>(&'borrow mut self) -> Slot<'borrow, T> {
        Slot { content: &mut *self.content, filled_token: SlotFilled::new() }
    }

    // `#[inline(always)]` because this is a no-op at runtime
    #[inline(always)]
    unsafe fn assume_filled(self) -> Box<'alloc, T> {
        // SAFETY: Caller guarantees the `T` is initialized
        let content = unsafe { self.content.assume_init_mut() };

        // SAFETY: `content` is the `T`, which the caller guarantees is initialized.
        // Every route to an `OwnedSlot` covers memory in the arena which lives for `'alloc` -
        // `new_in` allocates it there, and `from_non_null` requires it of its caller.
        // That is the lifetime of the `Box` created here.
        unsafe { Box::from_non_null(NonNull::from(content)) }
    }
}

/// Proof that a [`Slot`] has been filled.
///
/// Branded lifetime `'slot` ties it to one specific [`Slot`],
/// so it proves "*this* slot is filled" not "*some* slot is filled".
///
/// Filling the slot you were given produces the token you owe:
///
/// ```
/// use oxc_allocator::{Allocator, OwnedSlot};
///
/// let allocator = Allocator::new();
/// let allocator = &allocator;
///
/// let boxed = OwnedSlot::<u32>::new_in(&allocator).fill_with(|slot| slot.fill(123));
/// assert_eq!(*boxed, 123);
/// ```
///
/// A token obtained anywhere else does not compile in its place. `fill_with` binds `'slot`
/// with `for<'slot>`, so it is a fresh lifetime that nothing outside the closure can name,
/// and [`SlotFilled`] is invariant over it, so no other lifetime can be coerced to it:
///
/// ```compile_fail
/// use oxc_allocator::{Allocator, OwnedSlot, SlotFilled};
///
/// fn fill(token: SlotFilled<'_>, allocator: &Allocator) {
///     let slot = OwnedSlot::<u32>::new_in(&allocator);
///     // Error: the struct `SlotFilled<'slot>` is invariant over the parameter `'slot`
///     let _boxed = slot.fill_with(|_slot| token);
/// }
/// ```
#[must_use]
pub struct SlotFilled<'slot> {
    /// `PhantomData<fn(&'slot ()) -> &'slot ()>` is both co- and contra-variant in `'slot`,
    /// so `'slot` is invariant.
    brand: PhantomData<fn(&'slot ()) -> &'slot ()>,
}

impl SlotFilled<'_> {
    /// Create a [`SlotFilled`] token.
    ///
    /// Only ever call this where the slot branded `'slot` has just been filled, or where the token
    /// is inert - stored in a [`Slot`] to keep `'slot` invariant, or handed to a caller which has
    /// already proved the value is initialized.
    //
    // `#[inline(always)]` because this is a no-op at runtime
    #[inline(always)]
    fn new() -> Self {
        Self { brand: PhantomData }
    }
}

#[cfg(test)]
mod test {
    use std::{cell::Cell, mem::offset_of};

    use oxc_data_structures::types::implements;

    use crate::{Allocator, Box, FillSlot, OwnedSlot, Slot, SlotFilled, Vec};

    // A slot is `Send`/`Sync` exactly when `T` is, which it gets from holding a `&mut MaybeUninit<T>`.
    // `Cell` (`Send` but not `Sync`) and `Vec` (`Sync` but not `Send`) pin the 2 auto traits
    // independently of each other.
    //
    // Unlike `Vec`, a slot holds no `&Arena`, so one on another thread cannot allocate.
    // `into_contents` is the only method which allocates, and it takes the allocator as a param -
    // `&Allocator` is not `Send`, so it cannot follow a slot across.
    #[test]
    fn slot_send_sync() {
        assert!(implements!(Slot<u32>: Send));
        assert!(implements!(Slot<u32>: Sync));
        assert!(implements!(Slot<Box<u32>>: Send));
        assert!(implements!(Slot<Box<u32>>: Sync));
        assert!(implements!(Slot<Cell<u32>>: Send));
        assert!(implements!(Slot<Cell<u32>>: !Sync));
        assert!(implements!(Slot<Vec<u32>>: !Send));
        assert!(implements!(Slot<Vec<u32>>: Sync));

        assert!(implements!(OwnedSlot<u32>: Send));
        assert!(implements!(OwnedSlot<u32>: Sync));
        assert!(implements!(OwnedSlot<Box<u32>>: Send));
        assert!(implements!(OwnedSlot<Box<u32>>: Sync));
        assert!(implements!(OwnedSlot<Cell<u32>>: Send));
        assert!(implements!(OwnedSlot<Cell<u32>>: !Sync));
        assert!(implements!(OwnedSlot<Vec<u32>>: !Send));
        assert!(implements!(OwnedSlot<Vec<u32>>: Sync));

        // The premise the reasoning above rests on, and now the only thing enforcing it -
        // `Slot<Box<T>>`, the shape `into_contents` applies to, is itself `Send`.
        // `&T` is `Send` only if `T` is `Sync`, so what this pins is that the arena
        // is single-threaded.
        assert!(implements!(&Allocator: !Send));
    }

    // `SlotFilled` carries no data, only a brand, so it is inert on any thread
    #[test]
    fn slot_filled_send_sync() {
        assert!(implements!(SlotFilled: Send));
        assert!(implements!(SlotFilled: Sync));
    }

    #[test]
    fn owned_slot_fill() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let boxed: Box<'_, u32> = OwnedSlot::new_in(&allocator).fill(123);
        assert_eq!(*boxed, 123);
    }

    #[test]
    fn owned_slot_fill_with() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        // The closure writes straight into the `Slot`, and returns the token as proof
        let boxed: Box<'_, u32> = OwnedSlot::new_in(&allocator).fill_with(|slot| slot.fill(123));
        assert_eq!(*boxed, 123);
    }

    #[test]
    fn slot_into_some() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        // `&str` has a niche, so `Option<&str>` is the same size as `&str`
        let boxed: Box<'_, Option<&str>> =
            OwnedSlot::new_in(&allocator).fill_with(|slot| slot.into_some().fill("hello"));
        assert_eq!(*boxed, Some("hello"));
    }

    #[test]
    fn slot_into_contents() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        // The `Box` is written into the outer place, and the `u32` it points at into the arena memory it reserves
        let boxed: Box<'_, Box<'_, u32>> = OwnedSlot::new_in(&allocator)
            .fill_with(|slot| slot.into_contents(&allocator).fill(123));
        assert_eq!(**boxed, 123);
    }

    #[test]
    fn slot_into_part() {
        #[derive(PartialEq, Debug)]
        struct Pair {
            a: u32,
            b: u64,
        }

        let allocator = Allocator::default();
        let allocator = &allocator;

        let boxed: Box<'_, Pair> = OwnedSlot::<Pair>::new_in(&allocator).fill_with(|mut slot| {
            // Write one field through the pointer...
            // SAFETY: The slot covers memory laid out for a `Pair`, so its `a` field is valid for writing.
            unsafe { (&raw mut (*slot.as_mut_ptr()).a).write(1) };
            // ...and the other by narrowing the slot to it.
            // SAFETY: `b` lies within the `Pair` at its own offset, which is aligned for a `u64`.
            // `a` was written above, so filling `b` leaves a valid `Pair`.
            let b_slot: Slot<'_, u64> = unsafe { slot.into_part(offset_of!(Pair, b)) };
            b_slot.fill(2)
        });

        assert_eq!(*boxed, Pair { a: 1, b: 2 });
    }

    #[test]
    fn reborrow_leaves_the_place_usable() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let mut owned = OwnedSlot::<u32>::new_in(&allocator);
        // Lend the place out. The token this yields is for the reborrow, not for `owned`.
        let _reborrow_token = owned.reborrow().fill(123);
        // `owned` is usable again once the reborrow has ended
        // SAFETY: The `u32` was written through the reborrow above
        let boxed: Box<'_, u32> = unsafe { owned.assume_filled() };
        assert_eq!(*boxed, 123);
    }
}
