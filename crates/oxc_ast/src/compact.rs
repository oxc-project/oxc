//! Experiment into implementing compact enums with pointer tagging.
//!
//! https://github.com/oxc-project/backlog/issues/91
//!
//! Use a `Compactable` trait to allow enums defined outside of the crate where `Compact`
//! is defined to still use `Compact`.
//!
//! Idea is to define all methods on `Compact`, so user doesn't have to have have `Compactable`
//! trait in scope to use them, but have `Compact`'s methods delegate to the type's `Compactable` impl
//! for actions which are type-specific.
//!
//! Problem is with `CompactRef` and `CompactMut` - lifetime errors trying to define these
//! as associated types on `Compactable` trait. I'm not sure where the `'r` lifetime should come in.
//! I assume this can be overcome, but I just don't know how.
//!
//! TODO: Do we also want an 8-byte type which is equivalent in memory to `Compact<T>`,
//! but represents a borrow of a `Compact<T>`? (like `&Compact<T>`, but without the indirection)
//! Call it `CompactBorrow<'b, T>`?
//!
//! TODO: Also need methods on `Compact<T>` for narrowing / widening where an enum inherits from other enums.
//! e.g. `Compact<Expression>` -> `Compact<MemberExpression>`.
//! Doing that would probably need codegen-ing enum inheritance methods, and getting rid of the
//! `inherit_variants!` macro (which would be no bad thing anyway).
//! But how is that possible? Won't orphan rules get in the way?
//!
//! Maybe `Compactable` trait defines a type which `Compact<T>` derefs / deref-muts to
//! e.g. `CompactExpression`.
//! Then implementer can own that type and define `as_*` methods on it.
//! Those methods would be callable on `Compact<T>` itself via auto-deref.
//!
//! But that still wouldn't solve owned `Compact<MemberExpression>` into `Compact<Expression>`,
//! or vice-versa.
//! Can't define `impl<'a> From<Compact<MemberExpression<'a>>> for Compact<Expression<'a>>`
//! or `impl<'a> TryFrom<Compact<Expression<'a>>> for Compact<MemberExpression<'a>>`
//! except in crate that `Compact` is defined.
//! You'd need to do `Expression::from(member_expr.unpack()).pack()`.
//! That's a bit crap. Also not sure if compiler would boil it down to the no-op that it should be.

#![expect(
    missing_docs,
    clippy::elidable_lifetime_names,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    clippy::inline_always
)]

// ----------------------------------------
// Types and traits
// ----------------------------------------

use std::{
    marker::PhantomData,
    mem::transmute,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use oxc_allocator::Box;

use crate::ast::*;

// TODO: Need alternative version for 32-bit systems.
//
// Cannot use high-bits tagging on 32-bit systems, but low-bits tagging can still work,
// where the enum has no more than 4 variants, and payloads are all pointer-aligned.
// If enum doesn't satisfy those constraints, need to store discriminant and pointer separately
// (i.e. `Compact` doesn't do anything).

/// A compact representation of an enum for which all variants are [`Box`]es.
///
/// Utilizes pointer tagging to reduce such enums from 16 to 8 bytes on 64-bit systems.
pub struct Compact<T: Compactable> {
    ptr: NonNull<u8>,
    _marker: PhantomData<T>,
}

impl<T: Compactable> Compact<T> {
    /// Convert T to `Compact<T>`.
    ///
    /// This method is only intended for internal use. Use `Compact<T>::pack` or `T::compact` instead.
    ///
    /// # SAFETY
    /// `payload` must correspond to `ty`, so the pair make a valid `T`.
    #[inline(always)] // This method is only 2 instructions
    pub unsafe fn new<'a, U>(ty: T::Ty, payload: Box<'a, U>) -> Self {
        // TODO: Skip shift if `MAX_DISCRIMINANT < (1 << PAYLOAD_FREE_LOWER_BITS)`.
        // There's enough space in lower bits to store the tag without shifting.
        // Also update `ty` and `payload` methods accordingly.

        const {
            assert!(T::Ty::MAX_DISCRIMINANT < 128);
        }

        let ptr = Box::into_non_null(payload).cast::<u8>().as_ptr();
        // Why does this produce an ADD instruction instead of OR?
        let ptr = ptr.map_addr(|addr| (addr << 7) | ty.to_usize());
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        Self { ptr, _marker: PhantomData }
    }

    /// Get type of the `Compact<T>`.
    #[inline(always)] // This method is only 1 instruction or no-op
    pub fn ty(&self) -> T::Ty {
        // If payload is aligned on 2 or more, bottom bit of payload pointer is always 0,
        // so no need to mask off that bit
        let lower_byte = self.ptr.as_ptr() as u8;
        let discriminant =
            if T::PAYLOAD_FREE_LOWER_BITS > 0 { lower_byte } else { lower_byte & 127 };
        unsafe { T::Ty::from_u8_unchecked(discriminant) }
    }

    /// Get pointer to payload of the `Compact<T>` as a `NonNull<u8>`.
    #[inline(always)] // This method is only 1 instruction
    fn payload(&self) -> NonNull<u8> {
        // Cast to `isize` before shifting, so shift is performed with sign extension
        #[expect(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
        let ptr = self.ptr.as_ptr().map_addr(|addr| ((addr as isize) >> 7) as usize);
        unsafe { NonNull::new_unchecked(ptr) }
    }

    /// Pack a `T` into a `Compact<T>`.
    ///
    /// It's usually simpler to bring [`Compactable`] trait into scope,
    /// and use `value.into_compact()` rather than `Compact::pack(value)`.
    #[inline(always)] // This method just delegates
    pub fn pack(value: T) -> Self {
        value.into_compact()
    }

    /// Unpack a `Compact<T>` into a `T`.
    #[inline(always)] // This method is only 2 or 3 instructions
    pub fn unpack(self) -> T {
        unsafe { T::from_ty_and_payload(self.ty(), self.payload()) }
    }

    // TODO
    /*
    #[inline(always)] // This method is only 2 or 3 instructions
    pub fn as_ref<'r>(&'r self) -> T::Ref<'r> {
        unsafe { T::Ref::from_ty_and_payload(self.ty(), self.payload()) }
    }

    #[inline(always)] // This method is only 2 or 3 instructions
    pub fn as_mut<'r>(&'r mut self) -> T::Mut<'r> {
        unsafe { T::Mut::from_ty_and_payload(self.ty(), self.payload()) }
    }
    */
}

impl<T: Compactable> From<T> for Compact<T> {
    #[inline(always)] // This method just delegates
    fn from(value: T) -> Self {
        value.into_compact()
    }
}

/// Trait for enums which can be compacted into a [`Compact<Self>`].
///
/// # SAFETY
///
/// * `Self` must be an enum.
/// * `Self` must be defined as `#[repr(C, u8)]`.
/// * All variants of `Self` must be `oxc_allocator::Box`es.
/// * All variants of `Self` must have discriminants less than 128.
///   This implies `Self` cannot have more than 128 variants.
pub unsafe trait Compactable: Sized {
    /// Number of lower bits in payload pointers which are always 0.
    ///
    /// e.g.:
    ///
    /// ```ignore
    /// #[repr(C, u8)]
    /// enum Foo<'a> {
    ///     Bar(Box<'a, u64>) = 0,
    ///     Qux(Box<'a, u32>) = 1,
    /// }
    /// ```
    ///
    /// Here `PAYLOAD_FREE_LOWER_BITS == 2` because `u32` is the lowest-aligned variant,
    /// having alignment 4, which means all pointers to a `u32` have 2 lowest bits 0.
    ///
    /// # SAFETY
    /// This value must be accurate!
    const PAYLOAD_FREE_LOWER_BITS: u32;

    /// Type representing discriminants of the enum (`Self`).
    ///
    /// # SAFETY
    /// Discriminants of `Ty` must match discriminants of `Self`.
    type Ty: CompactType;

    // TODO
    // type Ref: CompactRef;
    // type Mut: CompactMut;

    /// Type of this enum instance (discriminant).
    fn ty(self) -> Self::Ty;

    /// Compact a `Self` to a `Compact<Self>`.
    fn into_compact(self) -> Compact<Self>;

    /// Create a `Self` from `Ty` and payload pointer.
    ///
    /// # SAFETY
    /// `ty` and `payload` pair must together represent a valid `Self`.
    unsafe fn from_ty_and_payload(ty: Self::Ty, payload: NonNull<u8>) -> Self;
}

/// Trait for types representing discriminants of a compactable enum.
///
/// # SAFETY
/// * Must be an enum.
/// * Must be declared with `#[repr(u8)]`.
/// * All variants of enum must correspond to the discriminants of its related type.
pub unsafe trait CompactType: Copy + 'static {
    /// All variants of this type.
    ///
    /// # SAFETY
    /// Must include all variants.
    const VARIANTS: &[Self];

    /// Largest value of any variant.
    ///
    /// # SAFETY
    /// * Must be accurate.
    /// * Must be less than 128.
    const MAX_DISCRIMINANT: u8;

    /// Convert [`u8`] to this [`CompactType`], without checks.
    ///
    /// Should be implemented as `std::mem::transmute::<Self, u8>()`.
    unsafe fn from_u8_unchecked(n: u8) -> Self;

    /// Convert this [`CompactType`] to a [`u8`].
    ///
    /// Should be implemented as `self as u8`.
    fn to_u8(self) -> u8;

    /// Convert this [`CompactType`] to a [`usize`].
    ///
    /// This default impl should not be overridden.
    #[inline(always)] // This method is only 1 instruction, or maybe a no-op
    fn to_usize(self) -> usize {
        self.to_u8() as usize
    }
}

/// Trait for enums representing a `&Compact<T>` in a form which can be `match`-ed.
///
/// `CompactRef`s should not be passed around.
/// Instead pass around `&Compact<T>`s, and only convert to `CompactRef` just in time to `match` on it.
pub unsafe trait CompactRef {
    type Ty: CompactType;

    unsafe fn from_ty_and_payload(ty: Self::Ty, payload: NonNull<u8>) -> Self;
}

/// Trait for enums representing a `&mut Compact<T>` in a form which can be `match`-ed.
///
/// `CompactMut`s should not be passed around.
/// Instead pass around `&mut Compact<T>`s, and only convert to `CompactMut` just in time to `match` on it.
pub unsafe trait CompactMut {
    type Ty: CompactType;

    unsafe fn from_ty_and_payload(ty: Self::Ty, payload: NonNull<u8>) -> Self;
}

/// Calculate the smallest value in a slice of `usize`s.
const fn min(values: &[usize]) -> usize {
    let mut min = 0;
    let mut i = 0;
    while i < values.len() {
        if values[i] < min {
            min = values[i];
        }
        i += 1;
    }
    min
}

// ----------------------------------------
// Codegen-ed for all AST enums which have all variants boxed
// ----------------------------------------

/*
// Original type def
#[repr(C, u8)]
pub enum ObjectPropertyKind<'a> {
    ObjectProperty(Box<'a, ObjectProperty<'a>>) = 0,
    SpreadProperty(Box<'a, SpreadElement<'a>>) = 1,
}
*/

/// The type of an [`ObjectPropertyKind`].
//
// Discriminants here must match discriminants of `ObjectPropertyKind`.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ObjectPropertyKindType {
    ObjectProperty = 0,
    SpreadProperty = 1,
}

unsafe impl CompactType for ObjectPropertyKindType {
    const VARIANTS: &[Self] =
        &[ObjectPropertyKindType::ObjectProperty, ObjectPropertyKindType::SpreadProperty];

    const MAX_DISCRIMINANT: u8 = {
        let mut max = 0;
        let mut i = 0;
        while i < Self::VARIANTS.len() {
            let v = Self::VARIANTS[i] as u8;
            if v > max {
                max = v;
            }
            i += 1;
        }
        max
    };

    #[inline(always)] // This method is a no-op
    unsafe fn from_u8_unchecked(n: u8) -> Self {
        unsafe { transmute::<u8, Self>(n) }
    }

    #[inline(always)] // This method is a no-op
    fn to_u8(self) -> u8 {
        self as u8
    }
}

unsafe impl<'a> Compactable for ObjectPropertyKind<'a> {
    const PAYLOAD_FREE_LOWER_BITS: u32 =
        min(&[align_of::<ObjectProperty>(), align_of::<SpreadElement>()]).trailing_zeros();

    type Ty = ObjectPropertyKindType;
    // type Ref = ObjectPropertyKindRef<'a, 'r>;
    // type Mut = ObjectPropertyKindMut<'a, 'r>;

    #[inline(always)] // Should boil down to a no-op
    fn ty(self) -> ObjectPropertyKindType {
        match self {
            ObjectPropertyKind::ObjectProperty(_) => ObjectPropertyKindType::ObjectProperty,
            ObjectPropertyKind::SpreadProperty(_) => ObjectPropertyKindType::SpreadProperty,
        }
    }

    #[inline(always)] // Match should boil down to a no-op, only `Compact::new` will remain
    fn into_compact(self) -> Compact<Self> {
        unsafe {
            match self {
                ObjectPropertyKind::ObjectProperty(it) => {
                    Compact::new(ObjectPropertyKindType::ObjectProperty, it)
                }
                ObjectPropertyKind::SpreadProperty(it) => {
                    Compact::new(ObjectPropertyKindType::SpreadProperty, it)
                }
            }
        }
    }

    #[inline(always)] // Should boil down to a no-op
    unsafe fn from_ty_and_payload(ty: ObjectPropertyKindType, payload: NonNull<u8>) -> Self {
        unsafe {
            match ty {
                ObjectPropertyKindType::ObjectProperty => {
                    Self::ObjectProperty(Box::from_non_null(payload.cast()))
                }
                ObjectPropertyKindType::SpreadProperty => {
                    Self::SpreadProperty(Box::from_non_null(payload.cast()))
                }
            }
        }
    }
}

// Can't do this as a blanket impl due to orphan rules
impl<'a> From<Compact<ObjectPropertyKind<'a>>> for ObjectPropertyKind<'a> {
    #[inline(always)] // This method just delegates
    fn from(compact: Compact<ObjectPropertyKind<'a>>) -> Self {
        compact.unpack()
    }
}

/// Enum representing a `&Compact<ObjectPropertyKind>` in a form which can be `match`-ed.
///
/// `'r` is lifetime of the borrow of the `Compact<ObjectPropertyKind>`.
#[repr(C, u8)]
pub enum ObjectPropertyKindRef<'a, 'r> {
    ObjectProperty(&'r ObjectProperty<'a>) = ObjectPropertyKindType::ObjectProperty as u8,
    SpreadProperty(&'r SpreadElement<'a>) = ObjectPropertyKindType::SpreadProperty as u8,
}

/// Enum representing a `&mut Compact<ObjectPropertyKind>` in a form which can be `match`-ed.
///
/// `'r` is lifetime of the borrow of the `Compact<ObjectPropertyKind>`.
#[repr(C, u8)]
pub enum ObjectPropertyKindMut<'a, 'r> {
    ObjectProperty(&'r mut ObjectProperty<'a>) = ObjectPropertyKindType::ObjectProperty as u8,
    SpreadProperty(&'r mut SpreadElement<'a>) = ObjectPropertyKindType::SpreadProperty as u8,
}

unsafe impl<'a, 'r> CompactRef for ObjectPropertyKindRef<'a, 'r> {
    type Ty = ObjectPropertyKindType;

    #[inline(always)] // Should boil down to a no-op
    unsafe fn from_ty_and_payload(ty: ObjectPropertyKindType, payload: NonNull<u8>) -> Self {
        unsafe {
            match ty {
                ObjectPropertyKindType::ObjectProperty => {
                    Self::ObjectProperty(payload.cast().as_ref())
                }
                ObjectPropertyKindType::SpreadProperty => {
                    Self::SpreadProperty(payload.cast().as_ref())
                }
            }
        }
    }
}

unsafe impl<'a, 'r> CompactMut for ObjectPropertyKindMut<'a, 'r> {
    type Ty = ObjectPropertyKindType;

    #[inline(always)] // Should boil down to a no-op
    unsafe fn from_ty_and_payload(ty: ObjectPropertyKindType, payload: NonNull<u8>) -> Self {
        unsafe {
            match ty {
                ObjectPropertyKindType::ObjectProperty => {
                    Self::ObjectProperty(payload.cast().as_mut())
                }
                ObjectPropertyKindType::SpreadProperty => {
                    Self::SpreadProperty(payload.cast().as_mut())
                }
            }
        }
    }
}

// `Deref` impls mean you can use `T`'s methods on a `T::Ref` or `T::Mut`.
// e.g. `compact_expr.as_ref().span()`, `compact_expr.as_ref().is_typescript_syntax()`.
//
// Blanket impl on all `CompactRef` + `CompactMut` is not possible, due to orphan rules.
// It's also not possible to implement `Deref` on `Compact<T>` directly.
impl<'a, 'r> Deref for ObjectPropertyKindRef<'a, 'r> {
    type Target = ObjectPropertyKind<'a>;
    fn deref(&self) -> &Self::Target {
        unsafe { NonNull::from(self).cast::<Self::Target>().as_ref() }
    }
}

impl<'a, 'r> Deref for ObjectPropertyKindMut<'a, 'r> {
    type Target = ObjectPropertyKind<'a>;
    fn deref(&self) -> &Self::Target {
        unsafe { NonNull::from(self).cast::<Self::Target>().as_ref() }
    }
}

// `*compact_expr.as_mut() = Expression::NullLiteral(...)` wouldn't do anything, because it'd only
// mutate the `ExpressionMut`, not the underlying `Compact<Expression>`.
// However, mutating properties of the `Expression` would work, as would calling methods of `Expression`
// which take a `&mut self` e.g. `compact_expr.as_mut().set_span(SPAN)`.
//
// TODO: Is this really safe? Can bad things happen if user assigns to `*compact_expr.as_mut()`?
// If `T` was `Drop` it'd definitely be wrong, as it'd cause a memory leak.
// But are there further problems with non-`Drop` types?
//
// Even if it's not unsafe, is it unwise? Assigning to a `CompactMut` and it silently failing
// to actually change the AST sounds like it could be a footgun.
impl<'a, 'r> DerefMut for ObjectPropertyKindMut<'a, 'r> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { NonNull::from(self).cast::<Self::Target>().as_mut() }
    }
}

// TODO: These methods need to be on `Compact` itself.
// The code below wouldn't compile if it was in a different crate from where `Compact` is defined.
impl<'a> Compact<ObjectPropertyKind<'a>> {
    #[inline(always)] // This method is only 2 or 3 instructions
    pub fn as_ref<'r>(&'r self) -> ObjectPropertyKindRef<'a, 'r> {
        unsafe { ObjectPropertyKindRef::from_ty_and_payload(self.ty(), self.payload()) }
    }

    #[inline(always)] // This method is only 2 or 3 instructions
    pub fn as_mut<'r>(&'r mut self) -> ObjectPropertyKindMut<'a, 'r> {
        unsafe { ObjectPropertyKindMut::from_ty_and_payload(self.ty(), self.payload()) }
    }
}
