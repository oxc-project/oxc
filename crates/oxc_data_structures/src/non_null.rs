//! Non-null pointer types with explicit const/mut permissions.
//!
//! [`NonNullConst<T>`] and [`NonNullMut<T>`] are wrappers around [`NonNull<T>`]
//! that encode read-only vs read-write permission in the type system.
//!
//! They have no runtime overhead, and exist purely for type-level safety beyond what [`NonNull`] offers.
//!
//! # Why not just use `*const T` / `*mut T`?
//!
//! Raw pointers don't carry a non-null guarantee, so:
//!
//! * `Option<*const T>` is 16 bytes on 64-bit platforms (no niche optimization).
//! * LLVM doesn't get `nonnull` metadata, preventing certain optimizations.
//!
//! # Why not just use `NonNull<T>`?
//!
//! `NonNull<T>` erases the const/mut distinction entirely - `as_ptr()` returns `*mut T`,
//! and `as_mut()` is available regardless of how the pointer was created.
//! This makes it easy to accidentally write through a pointer that only has read provenance,
//! which is instant UB.
//!
//! # What these types provide
//!
//! | Pointer type                         | Non-null | Const/mut | Niche |
//! |--------------------------------------|----------|-----------|-------|
//! | `*const T` / `*mut T`                | No       | Yes       | No    |
//! | `NonNull<T>`                         | Yes      | **No**    | Yes   |
//! | `NonNullConst<T>` / `NonNullMut<T>`  | Yes      | Yes       | Yes   |
//!
//! # Variance
//!
//! Both `NonNullConst<T>` and `NonNullMut<T>` are **covariant** in `T`, matching `NonNull<T>`.
//! Types that need invariance (e.g. mutable iterators) should add their own `PhantomData<&'a mut T>` or similar.
//!
//! # Provenance
//!
//! These types enforce permissions at the API level, but cannot enforce pointer provenance.
//! It is still the caller's responsibility to ensure that:
//!
//! * A `NonNullConst<T>` is only created from a pointer with at least read provenance.
//! * A `NonNullMut<T>` is only created from a pointer with write provenance.
//!
//! # Note on `as_ref` and `as_mut` methods
//!
//! Standard library's `NonNull::as_ref` and `NonNull::as_mut` methods take `&self` and `&mut self` respectively.
//! Our versions here take `self` (`NonNullConst` and `NonNullMut` are both `Copy`).
//!
//! It appears that Rust's maintainers consider these methods taking `&self` and `&mut self` to have been a mistake,
//! and are only not changing it because of backwards compatibility.
//!
//! Ralf Jung: "My conclusion back then was that this was meant as a kind of safety net...
//! I am not sure if that safety net is really helping anyone, though."
//! <https://github.com/rust-lang/rust/pull/80771#issuecomment-756049892>
//!
//! Thom Chiovoloni: "I think it's better to take by value. If we could change the others compatibly, I think we would."
//! <https://github.com/rust-lang/rust/pull/96100#issuecomment-1100506407>
//!
//! The newer `NonNull::as_uninit_ref` / `as_uninit_mut` take `self` by value, explicitly to correct the pattern.
//! `as_ref` / `as_mut` on `*const T` and `*mut T` also take `self` by value.
//!
//! We are not bound by backwards compatibility, so we can correct the mistake.

// All methods just delegate to `NonNull`'s methods
#![expect(clippy::inline_always)]

use std::{
    cmp,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    num::NonZeroUsize,
    ptr::NonNull,
};

// =====================================================================================
// NonNullConst<T>
// =====================================================================================

/// A non-null pointer with read-only permission.
///
/// This is a `#[repr(transparent)]` wrapper around [`NonNull<T>`], so it has the same layout, niche optimization,
/// and LLVM metadata benefits.
///
/// Unlike `NonNull<T>`:
/// 1. It does not provide an `as_mut` method.
/// 2. [`as_ptr`] method returns a `*const T` (not `*mut T`).
///
/// # When to use
///
/// Use [`NonNullConst<T>`] when you have a pointer that should only be read through.
/// This replaces both `*const T` (gaining non-null guarantees) and `NonNull<T>` (gaining const/mut clarity).
///
/// Use [`NonNullMut<T>`] when you have a pointer that may be both read and written through.
///
/// # Validity
///
/// Like `NonNull<T>`, this type guarantees only that the pointer is non-null.
/// It does not guarantee that the pointer is valid for reads, properly aligned, or points to initialized memory.
/// Those are preconditions of individual methods (e.g. `read`, `as_ref`) and must be upheld by the caller.
///
/// [`as_ptr`]: Self::as_ptr
#[repr(transparent)]
#[must_use]
pub struct NonNullConst<T: ?Sized>(NonNull<T>);

impl<T: ?Sized> NonNullConst<T> {
    // ---------------------------------------------------------------------------------
    // Constructors
    // ---------------------------------------------------------------------------------

    /// Create a new [`NonNullConst<T>`] from a raw `*const T` pointer.
    ///
    /// Returns `None` if `ptr` is null.
    #[inline(always)]
    pub const fn new(ptr: *const T) -> Option<Self> {
        match NonNull::new(ptr.cast_mut()) {
            Some(p) => Some(Self(p)),
            None => None,
        }
    }

    /// Create a new [`NonNullConst<T>`] from a raw `*const T` pointer without checking for null.
    ///
    /// # SAFETY
    /// `ptr` must be non-null.
    #[inline(always)]
    pub const unsafe fn new_unchecked(ptr: *const T) -> Self {
        // SAFETY: Caller guarantees `ptr` is non-null
        Self(unsafe { NonNull::new_unchecked(ptr.cast_mut()) })
    }

    /// Create a dangling but well-aligned [`NonNullConst<T>`].
    ///
    /// The returned pointer is not valid for non-zero-sized reads. See [Validity](NonNullConst#validity).
    #[inline(always)]
    pub const fn dangling() -> Self
    where
        T: Sized,
    {
        Self(NonNull::dangling())
    }

    /// Create a [`NonNullConst<T>`] from a shared reference.
    ///
    /// Since `&T` is [`Copy`], the reference is copied rather than consumed and remains usable after the call.
    ///
    /// # Warning
    ///
    /// The returned pointer aliases the same memory as `r`.
    /// Reading through it while a `&mut T` to the same memory is live violates the `&mut T`'s
    /// exclusivity guarantee, which is undefined behavior.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_data_structures::non_null::NonNullConst;
    /// let mut n = 0u32;
    /// let const_ptr = NonNullConst::from_ref(&n);
    ///
    /// // `from_ref` returns a raw pointer with no lifetime.
    /// // Borrow checker sees the `&n` borrow as ending here, so it allows `&mut n` below.
    /// let mut_ref = &mut n;
    ///
    /// // Don't do this! UB! `mut_ref` is still live.
    /// // unsafe { const_ptr.read() };
    ///
    /// *mut_ref = 1;
    /// ```
    #[inline(always)]
    pub const fn from_ref(r: &T) -> Self {
        Self(NonNull::from_ref(r))
    }

    /// Create a [`NonNullConst<T>`] pointing to the first element of a slice.
    ///
    /// The pointer is valid for reads only if the slice is not empty.
    #[inline(always)]
    pub const fn from_slice_data(slice: &[T]) -> Self
    where
        T: Sized,
    {
        NonNullConst::<[T]>::from_ref(slice).as_data_ptr()
    }

    /// Create a [`NonNullConst<T>`] from a [`NonNull<T>`], discarding write permission.
    #[inline(always)]
    pub const fn from_non_null(ptr: NonNull<T>) -> Self {
        Self(ptr)
    }

    // ---------------------------------------------------------------------------------
    // Conversion
    // ---------------------------------------------------------------------------------

    /// Get the pointer as a `*const T`.
    #[inline(always)]
    pub const fn as_ptr(self) -> *const T {
        self.0.as_ptr().cast_const()
    }

    /// Get the pointer as a [`NonNull<T>`].
    ///
    /// This is an escape hatch for interop with APIs that take `NonNull<T>`.
    #[inline(always)]
    pub const fn as_non_null(self) -> NonNull<T> {
        self.0
    }

    /// Get a shared reference to the value.
    ///
    /// # SAFETY
    /// The pointer must point to a valid, initialized value of type `T`, must be properly aligned,
    /// and the returned reference must not be used to violate Rust's aliasing rules.
    /// The reference's lifetime is unbound - the caller must ensure it does not outlive the data it points to.
    ///
    /// See also docs for [`NonNull::as_ref`].
    #[inline(always)]
    pub const unsafe fn as_ref<'a>(self) -> &'a T {
        // SAFETY: Caller guarantees the pointer is valid for reads and properly aligned
        unsafe { self.0.as_ref() }
    }

    /// Cast to a [`NonNullConst`] of a different type.
    #[inline(always)]
    pub const fn cast<U>(self) -> NonNullConst<U> {
        NonNullConst(self.0.cast::<U>())
    }

    /// Cast to [`NonNullMut<T>`], upgrading to write permission.
    ///
    /// # SAFETY
    /// The pointer must have write provenance. See [`NonNullMut` docs](NonNullMut#provenance) for rare exceptions.
    #[inline(always)]
    pub const unsafe fn cast_mut(self) -> NonNullMut<T> {
        NonNullMut(self.0)
    }

    // ---------------------------------------------------------------------------------
    // Pointer arithmetic
    // ---------------------------------------------------------------------------------

    /// Add an offset to the pointer (in units of `T`).
    ///
    /// # SAFETY
    /// The resulting pointer must not exceed the bounds of the allocated object.
    ///
    /// See also docs for [`NonNull::add`].
    #[inline(always)]
    pub const unsafe fn add(self, count: usize) -> Self
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees the resulting pointer is in bounds
        Self(unsafe { self.0.add(count) })
    }

    /// Subtract an offset from the pointer (in units of `T`).
    ///
    /// # SAFETY
    /// The resulting pointer must not exceed the bounds of the allocated object.
    ///
    /// See also docs for [`NonNull::sub`].
    #[inline(always)]
    pub const unsafe fn sub(self, count: usize) -> Self
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees the resulting pointer is in bounds
        Self(unsafe { self.0.sub(count) })
    }

    /// Add a byte offset to the pointer.
    ///
    /// # SAFETY
    /// The resulting pointer must not exceed the bounds of the allocated object.
    ///
    /// See also docs for [`NonNull::byte_add`].
    #[inline(always)]
    pub const unsafe fn byte_add(self, count: usize) -> Self {
        // SAFETY: Caller guarantees the resulting pointer is in bounds
        Self(unsafe { self.0.byte_add(count) })
    }

    /// Subtract a byte offset from the pointer.
    ///
    /// # SAFETY
    /// The resulting pointer must not exceed the bounds of the allocated object.
    ///
    /// See also docs for [`NonNull::byte_sub`].
    #[inline(always)]
    pub const unsafe fn byte_sub(self, count: usize) -> Self {
        // SAFETY: Caller guarantees the resulting pointer is in bounds
        Self(unsafe { self.0.byte_sub(count) })
    }

    /// Calculate a signed offset from the pointer (in units of `T`).
    ///
    /// # SAFETY
    /// The resulting pointer must not exceed the bounds of the allocated object.
    ///
    /// See also docs for [`NonNull::offset`].
    #[inline(always)]
    pub const unsafe fn offset(self, count: isize) -> Self
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees the resulting pointer is in bounds
        Self(unsafe { self.0.offset(count) })
    }

    /// Calculate the signed offset from `origin` to `self` (in units of `T`).
    ///
    /// When you can guarantee that `self >= origin`, [`offset_from_unsigned`] can be more performant than this method.
    ///
    /// # SAFETY
    /// Both pointers must be in bounds of the same allocated object.
    ///
    /// See also docs for [`NonNull::offset_from`].
    ///
    /// [`offset_from_unsigned`]: Self::offset_from_unsigned
    #[inline(always)]
    pub unsafe fn offset_from(self, origin: impl IntoNonNull<T>) -> isize
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees both pointers are in bounds of the same allocation
        unsafe { self.0.offset_from(origin.into_non_null()) }
    }

    /// Calculate the unsigned offset from `origin` to `self` (in units of `T`).
    ///
    /// When you can guarantee that `self >= origin`, this method can be more performant than [`offset_from`],
    /// but requiring stronger guarantees to avoid UB.
    ///
    /// # SAFETY
    /// Both pointers must be in bounds of the same allocated object, and `self >= origin`.
    ///
    /// See also docs for [`NonNull::offset_from_unsigned`].
    ///
    /// [`offset_from`]: Self::offset_from
    #[inline(always)]
    pub unsafe fn offset_from_unsigned(self, origin: impl IntoNonNull<T>) -> usize
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees both pointers are in bounds and `self >= origin`
        unsafe { self.0.offset_from_unsigned(origin.into_non_null()) }
    }

    // ---------------------------------------------------------------------------------
    // Reading
    // ---------------------------------------------------------------------------------

    /// Read the value from the pointer without moving it.
    ///
    /// If `T` is `Copy` and the aliasing requirements of [`as_ref`] are met
    /// (no active `&mut T` to the same memory), `*self.as_ref()` may be more performant.
    /// Going through a `&T` gives LLVM `dereferenceable` metadata that a raw pointer read does not.
    ///
    /// # SAFETY
    /// The pointer must point to a valid, initialized value of type `T` and must be properly aligned.
    ///
    /// See also docs for [`NonNull::read`].
    ///
    /// [`as_ref`]: Self::as_ref
    #[inline(always)]
    pub const unsafe fn read(self) -> T
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees the pointer is valid for reads and properly aligned
        unsafe { self.0.read() }
    }

    /// Read the value from the pointer without moving it. The pointer does not need to be aligned.
    ///
    /// # SAFETY
    /// The pointer must point to a valid, initialized value of type `T`.
    ///
    /// See also docs for [`NonNull::read_unaligned`].
    #[inline(always)]
    pub const unsafe fn read_unaligned(self) -> T
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees the pointer is valid for reads
        unsafe { self.0.read_unaligned() }
    }

    // ---------------------------------------------------------------------------------
    // Address
    // ---------------------------------------------------------------------------------

    /// Get the memory address of the pointer as a [`NonZeroUsize`].
    #[inline(always)]
    pub fn addr(self) -> NonZeroUsize {
        self.0.addr()
    }

    /// Get the memory address of the pointer as a `usize`.
    #[inline(always)]
    pub fn addr_usize(self) -> usize {
        self.0.addr().get()
    }

    /// Create a new pointer with the given address, using `self`'s provenance.
    ///
    /// See also docs for [`NonNull::with_addr`].
    #[inline(always)]
    pub fn with_addr(self, addr: NonZeroUsize) -> Self {
        Self(self.0.with_addr(addr))
    }

    /// Adjust the pointer's address using the given function.
    ///
    /// See also docs for [`NonNull::map_addr`].
    #[inline(always)]
    pub fn map_addr(self, f: impl FnOnce(NonZeroUsize) -> NonZeroUsize) -> Self {
        Self(self.0.map_addr(f))
    }

    /// Adjust the pointer's address using a function that operates on `usize`.
    ///
    /// The function must not return 0.
    ///
    /// # SAFETY
    /// The function must return a non-zero value.
    #[inline(always)]
    pub unsafe fn map_addr_usize(self, f: impl FnOnce(usize) -> usize) -> Self {
        // SAFETY: Caller guarantees `f` returns a non-zero value
        Self(self.0.map_addr(|addr| unsafe { NonZeroUsize::new_unchecked(f(addr.get())) }))
    }
}

// =====================================================================================
// NonNullMut<T>
// =====================================================================================

/// A non-null pointer with read-write permission.
///
/// This is a `#[repr(transparent)]` wrapper around [`NonNull<T>`], so it has the same layout, niche optimization,
/// and LLVM metadata benefits.
///
/// Unlike [`NonNullConst<T>`], it provides [`as_mut`] and [`as_mut_ptr`] methods.
///
/// # When to use
///
/// Use [`NonNullMut<T>`] when you have a pointer that may be both read and written through.
/// This replaces both `*mut T` (gaining non-null guarantees) and `NonNull<T>` (gaining const/mut clarity).
///
/// Use [`NonNullConst<T>`] when you have a pointer that should only be read through.
///
/// # Validity
///
/// Like `NonNull<T>`, this type guarantees only that the pointer is non-null.
/// It does not guarantee that the pointer is valid for reads or writes, properly aligned,
/// or points to initialized memory.
/// Those are preconditions of individual methods (e.g. `read`, `write`, `as_ref`, `as_mut`)
/// and must be upheld by the caller.
///
/// # Provenance
///
/// Unlike `&mut T`, merely *creating* a `NonNullMut<T>` from a pointer without write provenance is not UB.
/// UB only occurs if write provenance is actually exercised - by calling `write`, `as_mut`, etc.
/// Note that `as_mut` creates a `&mut T`, which is UB in itself even if the reference is never accessed.
///
/// The type's API *assumes* write provenance - constructors document this as a safety requirement,
/// and write methods will produce UB if the provenance doesn't actually permit writes.
/// If you need to create a `NonNullMut` without write provenance (e.g. for a sentinel value),
/// you may do so, but you must ensure by other means that it is never written through.
///
/// [`as_mut`]: Self::as_mut
/// [`as_mut_ptr`]: Self::as_mut_ptr
#[repr(transparent)]
#[must_use]
pub struct NonNullMut<T: ?Sized>(NonNull<T>);

impl<T: ?Sized> NonNullMut<T> {
    // ---------------------------------------------------------------------------------
    // Constructors
    // ---------------------------------------------------------------------------------

    /// Create a new [`NonNullMut<T>`] from a raw `*mut T` pointer.
    ///
    /// Returns `None` if `ptr` is null.
    ///
    /// # SAFETY
    /// `ptr` must have write provenance, if non-null.
    /// See [type-level docs](NonNullMut#provenance) for the rare exceptions to this rule.
    #[inline(always)]
    pub const unsafe fn new(ptr: *mut T) -> Option<Self> {
        match NonNull::new(ptr) {
            Some(p) => Some(Self(p)),
            None => None,
        }
    }

    /// Create a new [`NonNullMut<T>`] from a raw `*mut T` pointer without checking for null.
    ///
    /// # SAFETY
    /// * `ptr` must be non-null.
    /// * `ptr` must have write provenance. See [type-level docs](NonNullMut#provenance) for rare exceptions.
    #[inline(always)]
    pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        // SAFETY: Caller guarantees `ptr` is non-null
        Self(unsafe { NonNull::new_unchecked(ptr) })
    }

    /// Create a dangling but well-aligned [`NonNullMut<T>`].
    ///
    /// The returned pointer is not valid for non-zero-sized reads or writes. See [Validity](NonNullMut#validity).
    #[inline(always)]
    pub const fn dangling() -> Self
    where
        T: Sized,
    {
        Self(NonNull::dangling())
    }

    /// Create a [`NonNullMut<T>`] from a mutable reference.
    ///
    /// Unlike [`NonNullMut::from`] (the [`From`] trait impl), this method reborrows the reference,
    /// rather than consuming it, so the original reference remains usable after the call.
    ///
    /// # Warning
    ///
    /// The returned pointer and original reference alias the same memory.
    /// Reading or writing through the pointer while the reference is still live is undefined behavior.
    ///
    /// It may be preferable to use [`NonNullMut::from`] instead, which consumes the reference, to avoid this hazard.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_data_structures::non_null::NonNullMut;
    /// let mut n = 0u32;
    /// let mut_ref = &mut n;
    ///
    /// let ptr = NonNullMut::from_mut(mut_ref);
    ///
    /// // Don't do this! UB! `mut_ref` is still live.
    /// // unsafe { ptr.write(1) };
    ///
    /// // `mut_ref` is still usable after `from_mut`, unlike with `NonNullMut::from`.
    /// assert_eq!(*mut_ref, 0);
    /// ```
    #[inline(always)]
    pub const fn from_mut(r: &mut T) -> Self {
        Self(NonNull::from_mut(r))
    }

    /// Create a [`NonNullMut<T>`] pointing to the first element of a mutable slice.
    ///
    /// The pointer is valid for reads and writes only if the slice is not empty.
    #[inline(always)]
    pub const fn from_slice_data_mut(slice: &mut [T]) -> Self
    where
        T: Sized,
    {
        NonNullMut::<[T]>::from_mut(slice).as_data_ptr()
    }

    /// Create a [`NonNullMut<T>`] from a [`NonNull<T>`].
    ///
    /// # SAFETY
    /// `ptr` must have write provenance. See [type-level docs](NonNullMut#provenance) for rare exceptions.
    #[inline(always)]
    pub const unsafe fn from_non_null(ptr: NonNull<T>) -> Self {
        Self(ptr)
    }

    // ---------------------------------------------------------------------------------
    // Conversion
    // ---------------------------------------------------------------------------------

    /// Get the pointer as a `*const T`.
    ///
    /// Use [`as_mut_ptr`] if you need a `*mut T`.
    ///
    /// ```compile_fail
    /// # use oxc_data_structures::non_null::NonNullMut;
    /// let mut n = 0u32;
    /// let ptr = NonNullMut::from_mut(&mut n);
    /// let _: *mut u32 = ptr.as_ptr(); // error: `as_ptr` returns `*const T`, not `*mut T`
    /// ```
    ///
    /// [`as_mut_ptr`]: Self::as_mut_ptr
    #[inline(always)]
    pub const fn as_ptr(self) -> *const T {
        self.0.as_ptr().cast_const()
    }

    /// Get the pointer as a `*mut T`.
    #[inline(always)]
    pub const fn as_mut_ptr(self) -> *mut T {
        self.0.as_ptr()
    }

    /// Get the pointer as a [`NonNull<T>`].
    ///
    /// This is an escape hatch for interop with APIs that take `NonNull<T>`.
    #[inline(always)]
    pub const fn as_non_null(self) -> NonNull<T> {
        self.0
    }

    /// Get a shared reference to the value.
    ///
    /// # SAFETY
    /// The pointer must point to a valid, initialized value of type `T`, must be properly aligned,
    /// and the returned reference must not be used to violate Rust's aliasing rules.
    /// The reference's lifetime is unbound - the caller must ensure it does not outlive the data it points to.
    ///
    /// See also docs for [`NonNull::as_ref`].
    #[inline(always)]
    pub const unsafe fn as_ref<'a>(self) -> &'a T {
        // SAFETY: Caller guarantees the pointer is valid for reads and properly aligned
        unsafe { self.0.as_ref() }
    }

    /// Get a mutable reference to the value.
    ///
    /// # SAFETY
    /// The pointer must point to a valid, initialized value of type `T`, must be properly aligned,
    /// and the returned reference must be the only active reference to the value.
    /// The reference's lifetime is unbound - the caller must ensure it does not outlive the data it points to.
    ///
    /// See also docs for [`NonNull::as_mut`].
    #[inline(always)]
    pub const unsafe fn as_mut<'a>(mut self) -> &'a mut T {
        // SAFETY: Caller guarantees the pointer is valid for writes, properly aligned,
        // and no other references to the value exist.
        unsafe { self.0.as_mut() }
    }

    /// Cast to a [`NonNullMut`] of a different type.
    #[inline(always)]
    pub const fn cast<U>(self) -> NonNullMut<U> {
        NonNullMut(self.0.cast::<U>())
    }

    /// Downgrade to [`NonNullConst<T>`], discarding write permission.
    #[inline(always)]
    pub const fn cast_const(self) -> NonNullConst<T> {
        NonNullConst(self.0)
    }

    // ---------------------------------------------------------------------------------
    // Pointer arithmetic
    // ---------------------------------------------------------------------------------

    /// Add an offset to the pointer (in units of `T`).
    ///
    /// # SAFETY
    /// The resulting pointer must not exceed the bounds of the allocated object.
    ///
    /// See also docs for [`NonNull::add`].
    #[inline(always)]
    pub const unsafe fn add(self, count: usize) -> Self
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees the resulting pointer is in bounds
        Self(unsafe { self.0.add(count) })
    }

    /// Subtract an offset from the pointer (in units of `T`).
    ///
    /// # SAFETY
    /// The resulting pointer must not exceed the bounds of the allocated object.
    ///
    /// See also docs for [`NonNull::sub`].
    #[inline(always)]
    pub const unsafe fn sub(self, count: usize) -> Self
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees the resulting pointer is in bounds
        Self(unsafe { self.0.sub(count) })
    }

    /// Add a byte offset to the pointer.
    ///
    /// # SAFETY
    /// The resulting pointer must not exceed the bounds of the allocated object.
    ///
    /// See also docs for [`NonNull::byte_add`].
    #[inline(always)]
    pub const unsafe fn byte_add(self, count: usize) -> Self {
        // SAFETY: Caller guarantees the resulting pointer is in bounds
        Self(unsafe { self.0.byte_add(count) })
    }

    /// Subtract a byte offset from the pointer.
    ///
    /// # SAFETY
    /// The resulting pointer must not exceed the bounds of the allocated object.
    ///
    /// See also docs for [`NonNull::byte_sub`].
    #[inline(always)]
    pub const unsafe fn byte_sub(self, count: usize) -> Self {
        // SAFETY: Caller guarantees the resulting pointer is in bounds
        Self(unsafe { self.0.byte_sub(count) })
    }

    /// Calculate a signed offset from the pointer (in units of `T`).
    ///
    /// # SAFETY
    /// The resulting pointer must not exceed the bounds of the allocated object.
    ///
    /// See also docs for [`NonNull::offset`].
    #[inline(always)]
    pub const unsafe fn offset(self, count: isize) -> Self
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees the resulting pointer is in bounds
        Self(unsafe { self.0.offset(count) })
    }

    /// Calculate the signed offset from `origin` to `self` (in units of `T`).
    ///
    /// When you can guarantee that `self >= origin`, [`offset_from_unsigned`] can be more performant than this method.
    ///
    /// # SAFETY
    /// Both pointers must be in bounds of the same allocated object.
    ///
    /// See also docs for [`NonNull::offset_from`].
    ///
    /// [`offset_from_unsigned`]: Self::offset_from_unsigned
    #[inline(always)]
    pub unsafe fn offset_from(self, origin: impl IntoNonNull<T>) -> isize
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees both pointers are in bounds of the same allocation
        unsafe { self.0.offset_from(origin.into_non_null()) }
    }

    /// Calculate the unsigned offset from `origin` to `self` (in units of `T`).
    ///
    /// When you can guarantee that `self >= origin`, this method can be more performant than [`offset_from`],
    /// but requiring stronger guarantees to avoid UB.
    ///
    /// # SAFETY
    /// Both pointers must be in bounds of the same allocated object, and `self >= origin`.
    ///
    /// See also docs for [`NonNull::offset_from_unsigned`].
    ///
    /// [`offset_from`]: Self::offset_from
    #[inline(always)]
    pub unsafe fn offset_from_unsigned(self, origin: impl IntoNonNull<T>) -> usize
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees both pointers are in bounds and `self >= origin`
        unsafe { self.0.offset_from_unsigned(origin.into_non_null()) }
    }

    // ---------------------------------------------------------------------------------
    // Reading
    // ---------------------------------------------------------------------------------

    /// Read the value from the pointer without moving it.
    ///
    /// If `T` is `Copy` and the aliasing requirements of [`as_ref`] are met
    /// (no active `&mut T` to the same memory), `*self.as_ref()` may be more performant.
    /// Going through a `&T` gives LLVM `dereferenceable` metadata that a raw pointer read does not.
    ///
    /// # SAFETY
    /// The pointer must point to a valid, initialized value of type `T` and must be properly aligned.
    ///
    /// See also docs for [`NonNull::read`].
    ///
    /// [`as_ref`]: Self::as_ref
    #[inline(always)]
    pub const unsafe fn read(self) -> T
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees the pointer is valid for reads and properly aligned
        unsafe { self.0.read() }
    }

    /// Read the value from the pointer without moving it. The pointer does not need to be aligned.
    ///
    /// # SAFETY
    /// The pointer must point to a valid, initialized value of type `T`.
    ///
    /// See also docs for [`NonNull::read_unaligned`].
    #[inline(always)]
    pub const unsafe fn read_unaligned(self) -> T
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees the pointer is valid for reads
        unsafe { self.0.read_unaligned() }
    }

    // ---------------------------------------------------------------------------------
    // Writing
    // ---------------------------------------------------------------------------------

    /// Write a value to the pointer without reading or dropping the old value.
    ///
    /// # SAFETY
    /// The pointer must be valid for writes and properly aligned.
    /// The old value is not dropped - if `T` implements `Drop`, this can leak resources.
    ///
    /// See also docs for [`NonNull::write`].
    #[inline(always)]
    pub const unsafe fn write(self, val: T)
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees the pointer is valid for writes and properly aligned
        unsafe { self.0.write(val) }
    }

    /// Write a value to the pointer without reading or dropping the old value.
    /// The pointer does not need to be aligned.
    ///
    /// # SAFETY
    /// The pointer must be valid for writes.
    /// The old value is not dropped - if `T` implements `Drop`, this can leak resources.
    ///
    /// See also docs for [`NonNull::write_unaligned`].
    #[inline(always)]
    pub const unsafe fn write_unaligned(self, val: T)
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees the pointer is valid for writes
        unsafe { self.0.write_unaligned(val) }
    }

    /// Overwrite `count` bytes of memory starting at `self` with `val`.
    ///
    /// # SAFETY
    /// The pointer must be valid for writes of `count * size_of::<T>()` bytes and properly aligned.
    ///
    /// See also docs for [`NonNull::write_bytes`].
    #[inline(always)]
    pub const unsafe fn write_bytes(self, val: u8, count: usize)
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees the pointer is valid for the write
        unsafe { self.0.write_bytes(val, count) }
    }

    /// Replace the value at the pointer, returning the old value.
    ///
    /// # SAFETY
    /// The pointer must be valid for both reads and writes, and properly aligned.
    ///
    /// See also docs for [`NonNull::replace`].
    #[inline(always)]
    pub const unsafe fn replace(self, val: T) -> T
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees the pointer is valid for reads and writes
        unsafe { self.0.replace(val) }
    }

    // ---------------------------------------------------------------------------------
    // Copying
    // ---------------------------------------------------------------------------------

    /// Copy `count` elements from `src` to `self`. Source and destination may overlap.
    ///
    /// # SAFETY
    /// * `src` must be valid for reads of `count * size_of::<T>()` bytes, and properly aligned.
    /// * `self` must be valid for writes of `count * size_of::<T>()` bytes, and properly aligned.
    ///
    /// See also docs for [`NonNull::copy_from`].
    #[inline(always)]
    pub unsafe fn copy_from(self, src: impl IntoNonNull<T>, count: usize)
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees both pointers are valid for the copy
        unsafe { self.0.copy_from(src.into_non_null(), count) }
    }

    /// Copy `count` elements from `src` to `self`. Source and destination must NOT overlap.
    ///
    /// # SAFETY
    /// * `src` must be valid for reads of `count * size_of::<T>()` bytes, and properly aligned.
    /// * `self` must be valid for writes of `count * size_of::<T>()` bytes, and properly aligned.
    /// * The two memory regions must not overlap.
    ///
    /// See also docs for [`NonNull::copy_from_nonoverlapping`].
    #[inline(always)]
    pub unsafe fn copy_nonoverlapping_from(self, src: impl IntoNonNull<T>, count: usize)
    where
        T: Sized,
    {
        // SAFETY: Caller guarantees both pointers are valid and the regions don't overlap
        unsafe { self.0.copy_from_nonoverlapping(src.into_non_null(), count) }
    }

    // ---------------------------------------------------------------------------------
    // Dropping
    // ---------------------------------------------------------------------------------

    /// Drop the value at the pointer in place.
    ///
    /// # SAFETY
    /// The pointer must point to a valid, initialized value of type `T` and must be properly aligned.
    /// After calling this, the pointed-to value is uninitialized.
    ///
    /// See also docs for [`NonNull::drop_in_place`].
    #[inline(always)]
    pub unsafe fn drop_in_place(self) {
        // SAFETY: Caller guarantees the pointer is valid and the value is initialized
        unsafe { self.0.drop_in_place() }
    }

    // ---------------------------------------------------------------------------------
    // Address
    // ---------------------------------------------------------------------------------

    /// Get the memory address of the pointer as a [`NonZeroUsize`].
    #[inline(always)]
    pub fn addr(self) -> NonZeroUsize {
        self.0.addr()
    }

    /// Get the memory address of the pointer as a `usize`.
    #[inline(always)]
    pub fn addr_usize(self) -> usize {
        self.0.addr().get()
    }

    /// Create a new pointer with the given address, using `self`'s provenance.
    ///
    /// See also docs for [`NonNull::with_addr`].
    #[inline(always)]
    pub fn with_addr(self, addr: NonZeroUsize) -> Self {
        Self(self.0.with_addr(addr))
    }

    /// Adjust the pointer's address using the given function.
    ///
    /// See also docs for [`NonNull::map_addr`].
    #[inline(always)]
    pub fn map_addr(self, f: impl FnOnce(NonZeroUsize) -> NonZeroUsize) -> Self {
        Self(self.0.map_addr(f))
    }

    /// Adjust the pointer's address using a function that operates on `usize`.
    ///
    /// The function must not return 0.
    ///
    /// # SAFETY
    /// The function must return a non-zero value.
    #[inline(always)]
    pub unsafe fn map_addr_usize(self, f: impl FnOnce(usize) -> usize) -> Self {
        // SAFETY: Caller guarantees `f` returns a non-zero value
        Self(self.0.map_addr(|addr| unsafe { NonZeroUsize::new_unchecked(f(addr.get())) }))
    }
}

// =====================================================================================
// Array and slice support
// =====================================================================================

impl<T, const N: usize> NonNullConst<[T; N]> {
    /// Get the number of elements in the array.
    #[expect(clippy::unused_self)]
    #[inline(always)]
    pub const fn len(self) -> usize {
        N
    }

    /// Returns `true` if the array has zero length.
    #[expect(clippy::unused_self)]
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        N == 0
    }

    /// Return a [`NonNullConst<T>`] pointing to the first element of the array.
    #[inline(always)]
    pub const fn as_data_ptr(self) -> NonNullConst<T> {
        NonNullConst(self.0.cast::<T>())
    }
}

impl<T, const N: usize> NonNullMut<[T; N]> {
    /// Get the number of elements in the array.
    #[expect(clippy::unused_self)]
    #[inline(always)]
    pub const fn len(self) -> usize {
        N
    }

    /// Returns `true` if the array has zero length.
    #[expect(clippy::unused_self)]
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        N == 0
    }

    /// Return a [`NonNullMut<T>`] pointing to the first element of the array.
    #[inline(always)]
    pub const fn as_data_ptr(self) -> NonNullMut<T> {
        NonNullMut(self.0.cast::<T>())
    }
}

impl<T> NonNullConst<[T]> {
    /// Create a `NonNullConst<[T]>` from a pointer and a length.
    #[inline(always)]
    pub const fn slice_from_raw_parts(data: NonNullConst<T>, len: usize) -> Self {
        Self(NonNull::slice_from_raw_parts(data.0, len))
    }

    /// Get the number of elements in the slice.
    #[inline(always)]
    pub const fn len(self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the slice has no elements.
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0.is_empty()
    }

    /// Return a [`NonNullConst<T>`] pointing to the first element of the slice, discarding the length.
    ///
    /// Equivalent to the unstable `NonNull::as_non_null_ptr`.
    #[inline(always)]
    pub const fn as_data_ptr(self) -> NonNullConst<T> {
        NonNullConst(self.0.cast::<T>())
    }
}

impl<T> NonNullMut<[T]> {
    /// Create a `NonNullMut<[T]>` from a pointer and a length.
    #[inline(always)]
    pub const fn slice_from_raw_parts(data: NonNullMut<T>, len: usize) -> Self {
        Self(NonNull::slice_from_raw_parts(data.0, len))
    }

    /// Get the number of elements in the slice.
    #[inline(always)]
    pub const fn len(self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the slice has no elements.
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0.is_empty()
    }

    /// Return a [`NonNullMut<T>`] pointing to the first element of the slice, discarding the length.
    ///
    /// Equivalent to the unstable `NonNull::as_non_null_ptr`.
    #[inline(always)]
    pub const fn as_data_ptr(self) -> NonNullMut<T> {
        NonNullMut(self.0.cast::<T>())
    }
}

// =====================================================================================
// IntoNonNull trait
// =====================================================================================

/// Conversion into a [`NonNull`] pointer, for use in methods that accept any non-null pointer type.
///
/// Implemented by [`NonNull`], [`NonNullConst`], and [`NonNullMut`].
///
/// This trait is sealed and cannot be implemented outside this crate.
#[expect(private_bounds)]
pub trait IntoNonNull<T: ?Sized>: Sealed {
    /// Convert into a `NonNull<T>`.
    fn into_non_null(self) -> NonNull<T>;
}

/// Supertrait that seals [`IntoNonNull`] - not exported, so external types cannot implement it.
trait Sealed {}

impl<T: ?Sized> IntoNonNull<T> for NonNull<T> {
    #[inline(always)]
    fn into_non_null(self) -> NonNull<T> {
        self
    }
}

impl<T: ?Sized> Sealed for NonNull<T> {}

impl<T: ?Sized> IntoNonNull<T> for NonNullConst<T> {
    #[inline(always)]
    fn into_non_null(self) -> NonNull<T> {
        self.0
    }
}

impl<T: ?Sized> Sealed for NonNullConst<T> {}

impl<T: ?Sized> IntoNonNull<T> for NonNullMut<T> {
    #[inline(always)]
    fn into_non_null(self) -> NonNull<T> {
        self.0
    }
}

impl<T: ?Sized> Sealed for NonNullMut<T> {}

// =====================================================================================
// Standard trait impls
//
// Note: These traits can't be derived, as they'd gain a bound on e.g. `T: Copy`,
// which is inappropriate for pointer types.
// =====================================================================================

// -------------------------------------------------------------------------------------
// Copy / Clone
// -------------------------------------------------------------------------------------

impl<T: ?Sized> Copy for NonNullConst<T> {}

impl<T: ?Sized> Clone for NonNullConst<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for NonNullMut<T> {}

impl<T: ?Sized> Clone for NonNullMut<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

// -------------------------------------------------------------------------------------
// Send / Sync
// -------------------------------------------------------------------------------------

// `NonNullConst` and `NonNullMut` wrap `NonNull`, which is `!Send` and `!Sync`. They inherit that automatically.
// Types containing `NonNullConst` / `NonNullMut` must opt in to `Send` / `Sync` explicitly, same as they would
// with `NonNull` or `*mut T`.

// -------------------------------------------------------------------------------------
// From conversions
// -------------------------------------------------------------------------------------

impl<T: ?Sized> From<&T> for NonNullConst<T> {
    /// Convert a shared reference into a [`NonNullConst<T>`].
    ///
    /// Since `&T` is [`Copy`], `r` is copied rather than consumed and can be used again after this call.
    /// See docs for [`NonNullConst::from_ref`] for explanation of how this can be a hazard.
    #[inline(always)]
    fn from(r: &T) -> Self {
        Self::from_ref(r)
    }
}

impl<T: ?Sized> From<&mut T> for NonNullConst<T> {
    /// Convert a mutable reference into a [`NonNullConst<T>`], consuming the reference.
    ///
    /// Unlike [`NonNullConst::from_ref`], this consumes the reference — it cannot be used
    /// after this call. This is often what you want to avoid aliasing rules violations.
    ///
    /// Prefer `from_ref` when you need the reference to remain accessible.
    ///
    /// ```compile_fail
    /// # use oxc_data_structures::non_null::NonNullConst;
    /// let mut n = 0u32;
    /// let mut_ref = &mut n;
    ///
    /// let ptr1 = NonNullConst::from(mut_ref); // `mut_ref` is consumed here
    /// let ptr2 = NonNullConst::from(mut_ref); // error[E0382]: use of moved value: `mut_ref`
    /// ```
    #[inline(always)]
    fn from(r: &mut T) -> Self {
        Self(NonNull::from_ref(r))
    }
}

impl<T: ?Sized> From<&mut T> for NonNullMut<T> {
    /// Convert a mutable reference into a [`NonNullMut<T>`], consuming the reference.
    ///
    /// Unlike [`NonNullMut::from_mut`], this consumes the reference — it cannot be used
    /// after this call. This is often what you want to avoid aliasing rules violations.
    ///
    /// Prefer `from_mut` when you need the reference to remain accessible.
    ///
    /// ```compile_fail
    /// # use oxc_data_structures::non_null::NonNullMut;
    /// let mut n = 0u32;
    /// let mut_ref = &mut n;
    ///
    /// let ptr1 = NonNullMut::from(mut_ref); // `mut_ref` is consumed here
    /// let ptr2 = NonNullMut::from(mut_ref); // error[E0382]: use of moved value: `mut_ref`
    /// ```
    #[inline(always)]
    fn from(r: &mut T) -> Self {
        Self(NonNull::from_mut(r))
    }
}

impl<T: ?Sized> From<NonNull<T>> for NonNullConst<T> {
    #[inline(always)]
    fn from(ptr: NonNull<T>) -> Self {
        Self::from_non_null(ptr)
    }
}

impl<T: ?Sized> From<NonNullMut<T>> for NonNullConst<T> {
    /// Downgrade a [`NonNullMut<T>`] to [`NonNullConst<T>`].
    #[inline(always)]
    fn from(ptr: NonNullMut<T>) -> Self {
        ptr.cast_const()
    }
}

// -------------------------------------------------------------------------------------
// PartialEq / Eq
// -------------------------------------------------------------------------------------

// stdlib has `#[allow(ambiguous_wide_pointer_comparisons)]` on `PartialEq` impl for `NonNull`.
// https://github.com/rust-lang/rust/blob/a72e2a71d8fbcbc46cdd18784e2ab2c32cbd9c93/library/core/src/ptr/non_null.rs#L1712-L1719

#[expect(ambiguous_wide_pointer_comparisons)]
impl<T: ?Sized> PartialEq for NonNullConst<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T: ?Sized> Eq for NonNullConst<T> {}

#[expect(ambiguous_wide_pointer_comparisons)]
impl<T: ?Sized> PartialEq for NonNullMut<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T: ?Sized> Eq for NonNullMut<T> {}

// -------------------------------------------------------------------------------------
// PartialOrd / Ord
// -------------------------------------------------------------------------------------

// stdlib has `#[allow(ambiguous_wide_pointer_comparisons)]` on these impls for `NonNull`.
// https://github.com/rust-lang/rust/blob/a72e2a71d8fbcbc46cdd18784e2ab2c32cbd9c93/library/core/src/ptr/non_null.rs#L1721-L1737
//
// We also use `PartialOrd` impls which just delegate to `NonNull`'s implementations.
// Clippy flags these, but it seems safest to delegate direct to stdlib.

#[expect(clippy::non_canonical_partial_ord_impl, ambiguous_wide_pointer_comparisons)]
impl<T: ?Sized> PartialOrd for NonNullConst<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

#[expect(ambiguous_wide_pointer_comparisons)]
impl<T: ?Sized> Ord for NonNullConst<T> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

#[expect(clippy::non_canonical_partial_ord_impl, ambiguous_wide_pointer_comparisons)]
impl<T: ?Sized> PartialOrd for NonNullMut<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

#[expect(ambiguous_wide_pointer_comparisons)]
impl<T: ?Sized> Ord for NonNullMut<T> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

// -------------------------------------------------------------------------------------
// Hash
// -------------------------------------------------------------------------------------

impl<T: ?Sized> Hash for NonNullConst<T> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T: ?Sized> Hash for NonNullMut<T> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

// -------------------------------------------------------------------------------------
// Debug / Pointer
// -------------------------------------------------------------------------------------

impl<T: ?Sized> Debug for NonNullConst<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T: ?Sized> Debug for NonNullMut<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T: ?Sized> fmt::Pointer for NonNullConst<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.0, f)
    }
}

impl<T: ?Sized> fmt::Pointer for NonNullMut<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.0, f)
    }
}

// =====================================================================================
// Tests
// =====================================================================================

/// These tests are not exhaustive, as most methods just delegate to `NonNull`'s methods, and are trivial.
///
/// Tests here aim to cover:
/// * Basic tests for standard APIs.
/// * Additional methods that `NonNullConst` and `NonNullMut` implement beyond `NonNull`'s API.
#[cfg(test)]
mod tests {
    use super::{NonNullConst, NonNullMut};

    // ---------------------------------------------------------------------------------
    // Basic construction and conversion
    // ---------------------------------------------------------------------------------

    #[test]
    fn new_returns_none_for_null() {
        use std::ptr;

        assert!(NonNullConst::<u32>::new(ptr::null()).is_none());

        // SAFETY: No provenance requirements for null pointers
        assert!(unsafe { NonNullMut::<u32>::new(ptr::null_mut()) }.is_none());
    }

    #[test]
    fn new_returns_some_for_non_null() {
        let n = 0u32;
        let p = &raw const n;
        assert!(NonNullConst::new(p).is_some());

        let mut n = 0u32;
        let p = &raw mut n;
        // SAFETY: `p` has write provenance
        assert!(unsafe { NonNullMut::new(p) }.is_some());
    }

    #[test]
    fn cast_const_and_cast_mut_round_trip() {
        let mut n = 0u32;
        let mut_ptr = NonNullMut::from_mut(&mut n);
        let const_ptr = mut_ptr.cast_const();
        assert_eq!(mut_ptr.as_ptr(), const_ptr.as_ptr());

        // SAFETY: `const_ptr` was derived from `mut_ptr`, so it retains write provenance
        // and can be upgraded back to `NonNullMut`
        let round_tripped_mut_ptr = unsafe { const_ptr.cast_mut() };
        assert_eq!(round_tripped_mut_ptr, mut_ptr);
    }

    #[test]
    fn addr_usize_matches_addr() {
        let n = 0u32;
        let ptr = NonNullConst::from_ref(&n);
        assert_eq!(ptr.addr_usize(), ptr.addr().get());
    }

    #[test]
    fn map_addr_usize() {
        let arr = [0u8; 4];
        let first = NonNullConst::from_ref(&arr[0]);
        let second = NonNullConst::from_ref(&arr[1]);
        // SAFETY: Adding 1 byte stays within the `arr` allocation
        let mapped = unsafe { first.map_addr_usize(|a| a + 1) };
        assert_eq!(mapped.as_ptr(), second.as_ptr());

        let mut arr = [0u8; 4];
        let first = NonNullMut::from_slice_data_mut(&mut arr);
        // SAFETY: Adding 1 byte stays within the `arr` allocation
        let mapped = unsafe { first.map_addr_usize(|a| a + 1) };
        assert_eq!(mapped.addr_usize(), first.addr_usize() + 1);
    }

    // ---------------------------------------------------------------------------------
    // Slice/array methods
    // ---------------------------------------------------------------------------------

    #[test]
    fn from_slice_data_points_to_first_element() {
        let arr: [u32; 3] = [1, 2, 3];
        let ptr = NonNullConst::from_slice_data(&arr);
        assert_eq!(ptr.as_ptr(), arr.as_ptr());
        // SAFETY: `ptr` points to a valid, initialized `u32`
        assert_eq!(unsafe { ptr.read() }, 1);
    }

    #[test]
    fn from_slice_data_mut_points_to_first_element() {
        let mut arr: [u32; 3] = [1, 2, 3];
        let expected = arr.as_mut_ptr();
        let ptr = NonNullMut::from_slice_data_mut(&mut arr);
        assert_eq!(ptr.as_mut_ptr(), expected);
        // SAFETY: `ptr` points to a valid, initialized `u32`
        assert_eq!(unsafe { ptr.read() }, 1);
    }

    #[test]
    fn as_data_ptr_on_slice_points_to_first_element() {
        let arr: [u32; 3] = [1, 2, 3];
        let slice_ptr = NonNullConst::from_ref(&arr[..]);
        let data_ptr = slice_ptr.as_data_ptr();
        assert_eq!(data_ptr.as_ptr(), arr.as_ptr());

        let mut arr: [u32; 3] = [1, 2, 3];
        let slice_ptr = NonNullMut::from_mut(&mut arr[..]);
        let data_ptr = slice_ptr.as_data_ptr();
        assert_eq!(data_ptr.as_mut_ptr(), arr.as_mut_ptr());
    }

    #[test]
    fn as_data_ptr_on_array_points_to_first_element() {
        let arr: [u32; 3] = [1, 2, 3];
        let arr_ptr = NonNullConst::from_ref(&arr);
        let data_ptr = arr_ptr.as_data_ptr();
        assert_eq!(data_ptr.as_ptr(), arr.as_ptr());

        let mut arr: [u32; 3] = [1, 2, 3];
        let arr_ptr = NonNullMut::from_mut(&mut arr);
        let data_ptr = arr_ptr.as_data_ptr();
        assert_eq!(data_ptr.as_mut_ptr(), arr.as_mut_ptr());
    }

    #[test]
    fn slice_len_and_is_empty() {
        let arr: [u32; 3] = [1, 2, 3];
        let ptr = NonNullConst::from_ref(&arr[..]);
        assert_eq!(ptr.len(), 3);
        assert!(!ptr.is_empty());

        let empty: [u32; 0] = [];
        let empty_ptr = NonNullConst::from_ref(&empty[..]);
        assert_eq!(empty_ptr.len(), 0);
        assert!(empty_ptr.is_empty());

        let mut arr: [u32; 3] = [1, 2, 3];
        let ptr = NonNullMut::from_mut(&mut arr[..]);
        assert_eq!(ptr.len(), 3);
        assert!(!ptr.is_empty());

        let mut empty: [u32; 0] = [];
        let empty_ptr = NonNullMut::from_mut(&mut empty[..]);
        assert_eq!(empty_ptr.len(), 0);
        assert!(empty_ptr.is_empty());
    }

    #[test]
    fn array_len_and_is_empty() {
        let arr: [u32; 3] = [1, 2, 3];
        let ptr = NonNullConst::from_ref(&arr);
        assert_eq!(ptr.len(), 3);
        assert!(!ptr.is_empty());

        let empty: [u32; 0] = [];
        let empty_ptr = NonNullConst::from_ref(&empty);
        assert_eq!(empty_ptr.len(), 0);
        assert!(empty_ptr.is_empty());

        let mut arr: [u32; 3] = [1, 2, 3];
        let ptr = NonNullMut::from_mut(&mut arr);
        assert_eq!(ptr.len(), 3);
        assert!(!ptr.is_empty());

        let mut empty: [u32; 0] = [];
        let empty_ptr = NonNullMut::from_mut(&mut empty);
        assert_eq!(empty_ptr.len(), 0);
        assert!(empty_ptr.is_empty());
    }

    // ---------------------------------------------------------------------------------
    // IntoNonNull: Methods that accept any pointer type
    // ---------------------------------------------------------------------------------

    #[test]
    fn offset_from_accepts_all_pointer_types() {
        let mut arr = [0u32; 4];

        let base_const = NonNullConst::from_ref(&arr[0]);
        let base_mut = NonNullMut::from_mut(&mut arr[0]);
        let base_non_null = base_const.as_non_null();

        let tip_const = NonNullConst::from_ref(&arr[3]);
        let tip_mut = NonNullMut::from_mut(&mut arr[3]);

        // SAFETY: All pointers are in the same `arr` allocation
        unsafe {
            // `NonNullConst::offset_from` accepts all pointer types as origin
            assert_eq!(tip_const.offset_from(base_const), 3);
            assert_eq!(tip_const.offset_from(base_mut), 3);
            assert_eq!(tip_const.offset_from(base_non_null), 3);

            // `NonNullMut::offset_from` accepts all pointer types as origin
            assert_eq!(tip_mut.offset_from(base_const), 3);
            assert_eq!(tip_mut.offset_from(base_mut), 3);
            assert_eq!(tip_mut.offset_from(base_non_null), 3);
        }
    }

    #[test]
    fn offset_from_unsigned_accepts_all_pointer_types() {
        let mut arr = [0u32; 4];

        let base_const = NonNullConst::from_ref(&arr[0]);
        let base_mut = NonNullMut::from_mut(&mut arr[0]);
        let base_non_null = base_const.as_non_null();

        let tip_const = NonNullConst::from_ref(&arr[3]);
        let tip_mut = NonNullMut::from_mut(&mut arr[3]);

        // SAFETY:
        // * All pointers are in the same `arr` allocation.
        // * `tip` is at a higher address than all `base_*` pointers.
        unsafe {
            // `NonNullConst::offset_from_unsigned` accepts all pointer types as origin
            assert_eq!(tip_const.offset_from_unsigned(base_const), 3);
            assert_eq!(tip_const.offset_from_unsigned(base_mut), 3);
            assert_eq!(tip_const.offset_from_unsigned(base_non_null), 3);

            // `NonNullMut::offset_from_unsigned` accepts all pointer types as origin
            assert_eq!(tip_mut.offset_from_unsigned(base_const), 3);
            assert_eq!(tip_mut.offset_from_unsigned(base_mut), 3);
            assert_eq!(tip_mut.offset_from_unsigned(base_non_null), 3);
        }
    }

    #[test]
    fn copy_nonoverlapping_from_accepts_all_pointer_types() {
        // `NonNullConst` as source
        let src: [u32; 3] = [1, 2, 3];
        let src_ptr = NonNullConst::from_slice_data(&src);
        let mut dst = [0u32; 3];
        let dst_ptr = NonNullMut::from_slice_data_mut(&mut dst);
        // SAFETY: `src_ptr` and `dst_ptr` are valid for 3 elements and don't overlap
        unsafe { dst_ptr.copy_nonoverlapping_from(src_ptr, 3) };
        assert_eq!(dst, [1, 2, 3]);

        // `NonNullMut` as source
        let mut src: [u32; 3] = [1, 2, 3];
        let src_ptr = NonNullMut::from_slice_data_mut(&mut src);
        let mut dst = [0u32; 3];
        let dst_ptr = NonNullMut::from_slice_data_mut(&mut dst);
        // SAFETY: `src_ptr` and `dst_ptr` are valid for 3 elements and don't overlap
        unsafe { dst_ptr.copy_nonoverlapping_from(src_ptr, 3) };
        assert_eq!(dst, [1, 2, 3]);

        // `NonNull` as source
        let src: [u32; 3] = [1, 2, 3];
        let src_ptr = NonNullConst::from_slice_data(&src).as_non_null();
        let mut dst = [0u32; 3];
        let dst_ptr = NonNullMut::from_slice_data_mut(&mut dst);
        // SAFETY: `src_ptr` and `dst_ptr` are valid for 3 elements and don't overlap
        unsafe { dst_ptr.copy_nonoverlapping_from(src_ptr, 3) };
        assert_eq!(dst, [1, 2, 3]);
    }

    #[test]
    fn copy_from_allows_overlapping() {
        // `NonNullConst` as source
        let mut arr: [u32; 7] = [1, 2, 3, 4, 5, 6, 7];
        // Derive both pointers from the same mutable raw pointer to avoid Stacked Borrows violation
        let base = NonNullMut::from_slice_data_mut(&mut arr);
        let src = base.cast_const();
        // SAFETY: `base + 2` is within the `arr` allocation
        let dst = unsafe { base.add(2) };
        // SAFETY: Overlapping copy of 3 elements within the same allocation
        unsafe { dst.copy_from(src, 3) };
        assert_eq!(arr, [1, 2, 1, 2, 3, 6, 7]);

        // `NonNullMut` as source
        let mut arr: [u32; 7] = [1, 2, 3, 4, 5, 6, 7];
        let base = NonNullMut::from_slice_data_mut(&mut arr);
        let src = base;
        // SAFETY: `base + 2` is within the `arr` allocation
        let dst = unsafe { base.add(2) };
        // SAFETY: Overlapping copy of 3 elements within the same allocation
        unsafe { dst.copy_from(src, 3) };
        assert_eq!(arr, [1, 2, 1, 2, 3, 6, 7]);

        // `NonNull` as source
        let mut arr: [u32; 7] = [1, 2, 3, 4, 5, 6, 7];
        let base = NonNullMut::from_slice_data_mut(&mut arr);
        let src = base.as_non_null();
        // SAFETY: `base + 2` is within the `arr` allocation
        let dst = unsafe { base.add(2) };
        // SAFETY: Overlapping copy of 3 elements within the same allocation
        unsafe { dst.copy_from(src, 3) };
        assert_eq!(arr, [1, 2, 1, 2, 3, 6, 7]);
    }

    // ---------------------------------------------------------------------------------
    // PartialEq / Eq / PartialOrd / Ord / Hash
    // ---------------------------------------------------------------------------------

    #[test]
    fn eq() {
        let arr = [0u32; 2];
        let a = NonNullConst::from_ref(&arr[0]);
        let b = NonNullConst::from_ref(&arr[0]);
        let c = NonNullConst::from_ref(&arr[1]);

        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut arr = [0u32; 2];
        let a = NonNullMut::from_slice_data_mut(&mut arr);
        let b = a;
        // SAFETY: `a + 1` is within the allocation
        let c = unsafe { a.add(1) };

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn ord() {
        use std::cmp::Ordering;

        let arr = [0u32; 3];
        let a = NonNullConst::from_ref(&arr[0]);
        let b = NonNullConst::from_ref(&arr[1]);
        let c = NonNullConst::from_ref(&arr[2]);

        assert!(a < b);
        assert!(b > a);
        assert!(a < c);
        assert!(c > a);
        assert!(b < c);
        assert!(c > b);
        assert!(a <= a);
        assert!(a >= a);
        assert_eq!(a.cmp(&a), Ordering::Equal);
        assert_eq!(a.partial_cmp(&a), Some(Ordering::Equal));

        let mut arr = [0u32; 3];
        let base = NonNullMut::from_slice_data_mut(&mut arr);
        let a = base;
        // SAFETY: Offsets are within the allocation
        let (b, c) = unsafe { (base.add(1), base.add(2)) };

        assert!(a < b);
        assert!(b > a);
        assert!(a < c);
        assert!(c > a);
        assert!(b < c);
        assert!(c > b);
        assert!(a <= a);
        assert!(a >= a);
        assert_eq!(a.cmp(&a), Ordering::Equal);
        assert_eq!(a.partial_cmp(&a), Some(Ordering::Equal));
    }

    #[test]
    fn hash() {
        use std::hash::{DefaultHasher, Hash, Hasher};

        fn hash_of<T: Hash>(val: &T) -> u64 {
            let mut hasher = DefaultHasher::new();
            val.hash(&mut hasher);
            hasher.finish()
        }

        let arr = [0u32; 2];
        let a = NonNullConst::from_ref(&arr[0]);
        let b = NonNullConst::from_ref(&arr[0]);
        let c = NonNullConst::from_ref(&arr[1]);

        // Equal pointers must have equal hashes
        assert_eq!(hash_of(&a), hash_of(&b));
        // Different pointers should (almost certainly) have different hashes
        assert_ne!(hash_of(&a), hash_of(&c));

        let mut arr = [0u32; 2];
        let a = NonNullMut::from_slice_data_mut(&mut arr);
        // SAFETY: Offsets are within the allocation
        let (b, c) = unsafe { (a.add(0), a.add(1)) };

        // Equal pointers must have equal hashes
        assert_eq!(hash_of(&a), hash_of(&b));
        // Different pointers should (almost certainly) have different hashes
        assert_ne!(hash_of(&a), hash_of(&c));

        // `NonNullConst` and `NonNullMut` pointing to same address have same hash
        let mut x = 0u32;
        let const_ptr = NonNullConst::from_ref(&x);
        let mut_ptr = NonNullMut::from_mut(&mut x);
        assert_eq!(hash_of(&const_ptr), hash_of(&mut_ptr));
    }
}
