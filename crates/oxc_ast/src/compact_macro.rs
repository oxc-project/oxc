#![expect(
    clippy::undocumented_unsafe_blocks,
    clippy::unnecessary_box_returns,
    clippy::inline_always
)]

use std::{marker::PhantomData, num::NonZeroUsize, ptr::NonNull};

// ------------------------------
// Types, traits, and macros.
// Defined once to cover all compacted enum types.
// ------------------------------

/// In-memory representation of a [`CompactEnum`].
///
/// Stored as a tagged pointer, with the discriminant of the enum variant in the low 7 bits,
/// and the address of the payload in the high bits 57 bits.
#[repr(transparent)]
pub struct CompactEnumRepr<E: CompactEnum> {
    ptr: NonNull<u8>,
    _marker: PhantomData<E>,
}

impl<E: CompactEnum> CompactEnumRepr<E> {
    /// Create a new [`CompactEnumRepr`] from type and a pointer.
    ///
    /// # SAFETY
    /// Provided `ptr` must be a valid pointer to the payload of a variant of `E`.
    /// The variant must be the one with the same discriminant as `ty`.
    #[inline(always)]
    pub unsafe fn new(ty: E::Ty, ptr: NonNull<u8>) -> Self {
        let ptr = ptr.map_addr(|addr| {
            let addr = (addr.get() << 7) | (ty.to_u8() as usize);
            unsafe { NonZeroUsize::new_unchecked(addr) }
        });
        Self { ptr, _marker: PhantomData }
    }

    /// Create a new [`CompactEnumRepr`] from type and a `Box`.
    ///
    /// # SAFETY
    /// Provided `boxed`'s type must be correspond to the variant of `E` with the same discriminant as `ty`.
    pub unsafe fn new_from_box<T>(ty: E::Ty, boxed: Box<T>) -> Self {
        let ptr: NonNull<u8> = NonNull::from(Box::leak(boxed)).cast();
        unsafe { Self::new(ty, ptr) }
    }

    /// Get type of a [`CompactEnumRepr`].
    #[inline(always)]
    pub fn ty(&self) -> E::Ty {
        #[expect(clippy::cast_possible_truncation)]
        let tag = self.ptr.addr().get() as u8;
        unsafe { E::Ty::from_u8_unchecked(tag) }
    }

    /// Get raw pointer to the payload of a [`CompactEnumRepr`].
    #[inline(always)]
    pub fn ptr(&self) -> NonNull<u8> {
        #[expect(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
        self.ptr.map_addr(|addr| unsafe {
            NonZeroUsize::new_unchecked(((addr.get() as isize) >> 7) as usize)
        })
    }

    /// Consume a [`CompactEnumRepr`] and return the payload as a boxed value.
    ///
    /// # SAFETY
    /// The provided type `T` must correspond to the type of the payload of this tagged pointer.
    #[inline(always)]
    pub unsafe fn into_box<T>(self) -> Box<T> {
        let ptr = self.ptr().cast::<T>().as_ptr();
        unsafe { Box::from_raw(ptr) }
    }

    /// Convert a reference to a [`CompactEnumRepr`] into a reference to the payload.
    ///
    /// # SAFETY
    /// The provided type `T` must correspond to the type of the payload of this tagged pointer.
    #[inline(always)]
    pub unsafe fn as_ref<T>(&self) -> &T {
        unsafe { self.ptr().cast::<T>().as_ref() }
    }

    /// Convert a mutable reference to a [`CompactEnumRepr`] into a mutablereference to the payload.
    ///
    /// # SAFETY
    /// The type `T` must correspond to the type of the payload of this tagged pointer.
    #[inline(always)]
    pub unsafe fn as_mut<T>(&mut self) -> &mut T {
        unsafe { self.ptr().cast::<T>().as_mut() }
    }
}

/// Trait for compact enums.
///
/// Compacted enums are compact (8-byte) representation of enums which have the following characteristics:
///
/// - All variants have discriminants in the range 0..=127.
/// - All variants contain a `Box<T>`.
/// - `T` in all variants is aligned on at least 2.
///
/// e.g.:
/// ```
/// enum CanBeCompacted {
///     X(Box<u128>),
///     Y(Box<u64>),
///     Z(Box<u32>),
/// }
/// ```
///
/// # SAFETY
/// It is only safe to implement this trait on a type which is a new-type wrapper around a `CompactEnumRepr<Self>`,
/// and where `Self` satisfies the above requirements.
pub unsafe trait CompactEnum {
    /// Type of the type of this compact enum.
    type Ty: CompactEnumType;

    /// Type of the converter for this compact enum.
    type Converter<T>;

    /// Wrap an owned `T`, `&T` reference, or `&mut T` reference in a `Self::Converter<T>`.
    fn converter<T>(val: T) -> Self::Converter<T>;
}

/// Trait for type of a [`CompactEnum`].
///
/// # SAFETY
/// The type this is implemented on must be a `#[repr(u8)]` fieldless enum with
/// all variants having discriminants in the range `0..=127`.
pub unsafe trait CompactEnumType: Copy {
    /// Create a new [`CompactEnumType`] from a `u8`.
    ///
    /// # SAFETY
    /// Provided `ty` must be a valid discriminant for the enum.
    unsafe fn from_u8_unchecked(ty: u8) -> Self;

    /// Convert a [`CompactEnumType`] into a `u8`.
    fn to_u8(self) -> u8;
}

/// Match on a compact enum value with syntax similar to a standard `match`.
///
/// Supports variant arms (`Type::Variant(binding) => body`), wildcard arms (`_ => body`,
/// `_ if guard => body`), and binding arms (`name if guard => body`, `name => body`).
/// Arms are emitted in the order they are written.
///
/// The macro handles `T`, `&T`, and `&mut T` targets with identical syntax.
///
/// # Examples
///
/// ```ignore
/// // Exhaustive match - all variants handled
/// match_compact!(foo {
///     Foo::X(x) => do_x(x),
///     Foo::Y(y) => do_y(y),
///     Foo::Z(z) => do_z(z),
/// })
///
/// // Variant with wildcard
/// match_compact!(foo {
///     Foo::X(_) => 1,
///     _ => 0,
/// })
///
/// // Multi-pattern wildcard
/// match_compact!(foo {
///     Foo::X(_) | Foo::Y(_) => "x or y",
///     _ => "other",
/// })
///
/// // Variant with guard
/// match_compact!(foo {
///     Foo::X(x) if *x > 10 => "big x",
///     Foo::X(x) => "small x",
///     _ => "not x",
/// })
///
/// // Binding catch-all
/// match_compact!(bar.foo(), {
///     Foo::X(x) => do_x(x),
///     thing if thing.is_x_or_y() => 1,
///     _ => 0,
/// })
///
/// // Works with `&T`, `&mut T`, `Self`, expressions
/// match_compact!(self { Self::X(x) => ..., _ => ... })
/// match_compact!(&foo { Foo::X(x) => ..., _ => ... })
/// match_compact!(&mut foo { Foo::X(x) => ..., _ => ... })
/// ```
///
/// # How it works
///
/// The macro has 4 internal phases:
///
/// 1. Entry
/// 2. Process
/// 3. Fast / Munch
/// 4. Emit
///
/// ### 1. Entry
///
/// Handles different target forms (`ident`, `&ident`, `&mut ident`, `expr`).
///
/// * `ident` forms are tried first so `foo { ... }` is not misinterpreted as a struct literal.
///   The target is used directly (no `let` binding) so the original variable remains accessible
///   in arm bodies and catch-all guards. Comma between target and `{` is optional.
/// * Expression targets (e.g. `foo.bar()`) require a comma before `{`.
///   Expression is moved to a local `let target` to avoid evaluating the expression more than once.
///
/// All entries delegate to `@process`.
///
/// ### 2. Process (`@process`)
///
/// Routes to the fast path or the TT muncher:
///
/// * If all arms match the exhaustive variant pattern (`Type::Variant(binding) => body`),
///   delegates to `@fast` (simple repetition, faster compile time).
/// * Otherwise, delegates to `@munch` (TT muncher, handles all arm types).
///
/// ### 3a. Fast path (`@fast`)
///
/// Expands exhaustive variant arms directly using a simple repetition into
/// `match target.ty() { ... }`. No TT muncher overhead.
///
/// ### 3b. TT muncher (`@munch`)
///
/// Arms are processed one at a time and transformed into a single accumulator, preserving their original order.
///
/// * Variant arms with binding (`Type::Variant(binding) => body`) become discriminant pattern arms.
/// * Variant arms with binding and guard (`Type::Variant(binding) if guard => body`) use `matches!` in the guard
///   to evaluate with the binding without moving the target.
/// * Wildcard variant arms (`Type::Variant(_)`, with optional `|` and `if guard`) become discriminant pattern arms
///   without a converter call.
/// * `_` arms (with optional `if guard`) are passed through as-is.
/// * Binding catch-all arms (`name if guard => body`) use `matches!` with `ref` in the guard
///   (to borrow rather than move the target), and `let name = target` in the body.
///
/// Rules are tried in order:
///
/// * Variant with binding (`Ty::Variant(binding) => ...`)
/// * Wildcard variant (`Ty::Variant(_) => ...`)
/// * Wildcard (`_ => ...`)
/// * Binding catch-all (`ident => ...`)
///
/// A catch-all rule appends a trailing comma to the last arm if missing, so the main rules only need one form
/// (comma-terminated).
///
/// ### 4. Emit
///
/// The terminal `@munch` rule outputs `match target.ty() { accumulated_arms }`.
#[macro_export]
macro_rules! match_compact {
    // ==========
    // Entry: Identifier targets.
    // These are tried first so `foo { ... }` (without comma) is not misinterpreted as a struct literal.
    // Uses the target directly (no `let` binding), keeping it accessible in arm bodies and catch-all guards.
    // Delegates to `@process` which routes to `@fast` or `@munch`.
    // ==========

    // `match_compact!(foo { ... })` or `match_compact!(foo, { ... })`
    ($target:ident $(,)? { $($arms:tt)+ }) => {
        match_compact!(@process [$target] $($arms)+)
    };

    // `match_compact!(&foo { ... })` or `match_compact!(&foo, { ... })`
    (& $target:ident $(,)? { $($arms:tt)+ }) => {
        match_compact!(@process [& $target] $($arms)+)
    };

    // `match_compact!(&mut foo { ... })` or `match_compact!(&mut foo, { ... })`
    (&mut $target:ident $(,)? { $($arms:tt)+ }) => {
        match_compact!(@process [&mut $target] $($arms)+)
    };

    // ==========
    // Entry: Expression targets e.g. `match_compact!(foo.bar(), { ... })`.
    // Comma is required (cannot be omitted because `{` is not in the follow set of `expr`).
    // Move expression into `let target` to avoid evaluating it more than once.
    // Delegates to `@process` which routes to `@fast` or `@munch`.
    // ==========
    ($target:expr, { $($arms:tt)+ }) => {{
        let target = $target;
        match_compact!(@process [target] $($arms)+)
    }};

    // ==========
    // Process: Routes to `@fast` (exhaustive) or `@munch` (non-exhaustive).
    // ==========

    // Exhaustive: All arms are `Type::Variant(binding) => body` - use fast path
    (@process [$($target:tt)+]
        $($ty:ident :: $variant:ident ($binding:ident) => $body:expr),+ $(,)?
    ) => {
        match_compact!(@fast ($($target)+) { $($ty::$variant($binding) => $body),+ })
    };

    // Non-exhaustive: Has catch-all arms - use TT muncher
    (@process [$($target:tt)+] $($arms:tt)+) => {
        match_compact!(@munch [$($target)+] [] $($arms)+)
    };

    // ==========
    // Exhaustive fast path: Simple repetition, no TT muncher
    // ==========
    (@fast $target:tt {
        $($ty:ident :: $variant:ident ($binding:ident) => $body:expr),+
    }) => {{
        use $crate::compact_macro::CompactEnum;
        match $target.ty() {
            $(<$ty as CompactEnum>::Ty::$variant => {
                let $binding = unsafe { <$ty as CompactEnum>::converter($target).$variant() };
                $body
            }),+
        }
    }};

    // ==========
    // TT muncher rules.
    // Each rule matches one arm, transforms it, appends to the accumulator, and recurses.
    // `[$($target)+]` carries the target tokens through the recursion.
    // `[$($acc)*]` accumulates the transformed arms in order.
    // Variant rule is first so `ident ::` is tried before `ident if` / `ident =>`.
    // ==========

    // Variant with binding and guard.
    // `Type::Variant(binding) if guard => body`
    //
    // Guard uses `matches!` to evaluate with the binding without moving the target.
    // Converter is called in both the guard and body (cheap for `&T` - just a pointer cast).
    (@munch [$($target:tt)+] [$($acc:tt)*]
        $ty:ident :: $variant:ident ($binding:ident) if $guard:expr => $body:expr,
        $($rest:tt)*
    ) => {
        match_compact!(@munch [$($target)+]
            [$($acc)*
                <$ty as CompactEnum>::Ty::$variant
                    if matches!(
                        unsafe { <$ty as CompactEnum>::converter($($target)+).$variant() },
                        $binding if $guard
                    )
                => {
                    #[allow(unused_variables, clippy::allow_attributes)]
                    let $binding = unsafe { <$ty as CompactEnum>::converter($($target)+).$variant() };
                    $body
                },
            ]
            $($rest)*
        )
    };

    // Variant with binding.
    // `Type::Variant(binding) => body`
    (@munch [$($target:tt)+] [$($acc:tt)*]
        $ty:ident :: $variant:ident ($binding:ident) => $body:expr,
        $($rest:tt)*
    ) => {
        match_compact!(@munch [$($target)+]
            [$($acc)*
                <$ty as CompactEnum>::Ty::$variant => {
                    let $binding = unsafe { <$ty as CompactEnum>::converter($($target)+).$variant() };
                    $body
                },
            ]
            $($rest)*
        )
    };

    // Wildcard variant(s), with optional guard.
    // `Type::Variant(_) => body`
    // `Type::Variant(_) if guard => body`
    // `Type::Variant(_) | Type::Variant2(_) => body`
    // `Type::Variant(_) | Type::Variant2(_) if guard => body`
    (@munch [$($target:tt)+] [$($acc:tt)*]
        $($ty:ident :: $variant:ident (_))|+ $(if $guard:expr)? => $body:expr,
        $($rest:tt)*
    ) => {
        match_compact!(@munch [$($target)+]
            [$($acc)* $(<$ty as CompactEnum>::Ty::$variant)|+ $(if $guard)? => { $body },]
            $($rest)*
        )
    };

    // Wildcard, with optional guard.
    // `_ => body`
    // `_ if guard => body`
    //
    // Passed through as-is.
    (@munch [$($target:tt)+] [$($acc:tt)*]
        _ $(if $guard:expr)? => $body:expr,
        $($rest:tt)*
    ) => {
        match_compact!(@munch [$($target)+]
            [$($acc)* _ $(if $guard)? => $body,]
            $($rest)*
        )
    };

    // Binding catch-all arm with guard.
    // `name if guard => body`
    //
    // Guard uses `matches!` with `ref` to borrow (not move) the target.
    // Body binds `$name` to the target value.
    (@munch [$($target:tt)+] [$($acc:tt)*]
        $name:ident if $guard:expr => $body:expr,
        $($rest:tt)*
    ) => {
        match_compact!(@munch [$($target)+]
            [$($acc)*
                _ if matches!($($target)+, ref $name if $guard) => {
                    #[allow(unused_variables, clippy::allow_attributes)]
                    let $name = $($target)+;
                    $body
                },
            ]
            $($rest)*
        )
    };

    // Binding catch-all arm without guard.
    // `name => body`
    (@munch [$($target:tt)+] [$($acc:tt)*]
        $name:ident => $body:expr,
        $($rest:tt)*
    ) => {
        match_compact!(@munch [$($target)+]
            [$($acc)*
                _ => {
                    let $name = $($target)+;
                    $body
                },
            ]
            $($rest)*
        )
    };

    // Last arm without trailing comma.
    // Append a comma and re-invoke, so the normal rules match.
    (@munch [$($target:tt)+] [$($acc:tt)*] $($last:tt)+) => {
        match_compact!(@munch [$($target)+] [$($acc)*] $($last)+,)
    };

    // Terminal.
    // All arms consumed, emit the match.
    (@munch [$($target:tt)+] [$($acc:tt)+]) => {{
        use $crate::compact_macro::CompactEnum;
        match ($($target)+).ty() {
            $($acc)+
        }
    }};
}
pub use match_compact;

/// Returns `true` if a compact enum value matches a pattern.
///
/// Equivalent to `matches!` for compact enums. Uses `match_compact!` internally, so it supports the same target forms
/// (`ident`, `&ident`, `&mut ident`, `expr`) and the same pattern syntax.
///
/// # Examples
///
/// ```ignore
/// let is_x_or_y = matches_compact!(self, Self::X(_) | Self::Y(_));
/// let is_x_over_ten =matches_compact!(&foo, Foo::X(x) if *x > 10);
/// ```
#[macro_export]
macro_rules! matches_compact {
    ($target:ident, $($arm:tt)+) => {
        match_compact!($target, { $($arm)+ => true, _ => false, })
    };
    (& $target:ident, $($arm:tt)+) => {
        match_compact!(& $target, { $($arm)+ => true, _ => false, })
    };
    (&mut $target:ident, $($arm:tt)+) => {
        match_compact!(&mut $target, { $($arm)+ => true, _ => false, })
    };
    ($target:expr, $($arm:tt)+) => {
        match_compact!($target, { $($arm)+ => true, _ => false, })
    };
}
pub use matches_compact;

// ------------------------------
// Generated code for a type.
// Generated for each compacted enum type.
// ------------------------------

/// Compacted version of the following enum:
///
/// ```
/// pub enum Foo {
///     X(Box<u128>),
///     Y(Box<u64>),
///     Z(Box<u32>),
/// }
/// ```
#[repr(transparent)]
pub struct Foo(CompactEnumRepr<Foo>);

unsafe impl CompactEnum for Foo {
    type Ty = FooType;
    type Converter<T> = FooConverter<T>;

    #[inline(always)]
    fn converter<T>(val: T) -> FooConverter<T> {
        FooConverter(val)
    }
}

/// Type of [`Foo`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum FooType {
    X = 0,
    Y = 1,
    Z = 2,
}

/// SAFETY: `FooType` satisfies the requirements of [`CompactEnumType`].
unsafe impl CompactEnumType for FooType {
    #[inline(always)]
    unsafe fn from_u8_unchecked(ty: u8) -> Self {
        match ty {
            0 => FooType::X,
            1 => FooType::Y,
            2 => FooType::Z,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    #[inline(always)]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

// These types and methods are not required, if we always use `match_compact!` macro.
// But they're useful for particularly complex cases which `match_compact!` cannot handle (few and far between).

/// Uncompacted [`Foo`].
#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum FooUnpacked {
    X(Box<u128>) = 0,
    Y(Box<u64>) = 1,
    Z(Box<u32>) = 2,
}

/// Reference to a [`Foo`].
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FooRef<'r> {
    X(&'r u128) = 0,
    Y(&'r u64) = 1,
    Z(&'r u32) = 2,
}

/// Mutable reference to a [`Foo`].
#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum FooMut<'r> {
    X(&'r mut u128) = 0,
    Y(&'r mut u64) = 1,
    Z(&'r mut u32) = 2,
}

impl Foo {
    /// Get the type of a [`Foo`].
    #[inline(always)]
    pub fn ty(&self) -> FooType {
        self.0.ty()
    }

    /// Unpack a compact [`Foo`] into a uncompacted [`FooUnpacked`].
    #[inline(always)]
    pub fn unpack(self) -> FooUnpacked {
        unsafe {
            match self.ty() {
                FooType::X => FooUnpacked::X(self.0.into_box()),
                FooType::Y => FooUnpacked::Y(self.0.into_box()),
                FooType::Z => FooUnpacked::Z(self.0.into_box()),
            }
        }
    }

    /// Get a reference to a [`Foo`] as a [`FooRef`].
    #[inline(always)]
    pub fn as_ref(&self) -> FooRef<'_> {
        unsafe {
            match self.ty() {
                FooType::X => FooRef::X(self.0.as_ref()),
                FooType::Y => FooRef::Y(self.0.as_ref()),
                FooType::Z => FooRef::Z(self.0.as_ref()),
            }
        }
    }

    /// Get a mutable reference to a [`Foo`] as a [`FooMut`].
    #[inline(always)]
    pub fn as_mut(&mut self) -> FooMut<'_> {
        unsafe {
            match self.ty() {
                FooType::X => FooMut::X(self.0.as_mut()),
                FooType::Y => FooMut::Y(self.0.as_mut()),
                FooType::Z => FooMut::Z(self.0.as_mut()),
            }
        }
    }
}

impl FooUnpacked {
    /// Pack an uncompacted [`FooUnpacked`] into a compact [`Foo`].
    #[inline(always)]
    pub fn pack(self) -> Foo {
        unsafe {
            match self {
                FooUnpacked::X(x) => Foo(CompactEnumRepr::new_from_box(FooType::X, x)),
                FooUnpacked::Y(y) => Foo(CompactEnumRepr::new_from_box(FooType::Y, y)),
                FooUnpacked::Z(z) => Foo(CompactEnumRepr::new_from_box(FooType::Z, z)),
            }
        }
    }
}

/// Converter for [`Foo`] for use in `match_compact!` macro.
///
/// All methods are unsafe, and this type should not be used directly.
///
/// Purpose is to allow `match_compact!` to operate on a `Foo`, `&Foo`, or `&mut Foo`
/// with identical syntax, like Rust's built in `match`.
pub struct FooConverter<T>(T);

#[rustfmt::skip]
#[expect(non_snake_case)]
impl FooConverter<Foo> {
    #[inline(always)]
    unsafe fn X(self) -> Box<u128> { unsafe { self.0.0.into_box() }}
    #[inline(always)]
    unsafe fn Y(self) -> Box<u64> { unsafe { self.0.0.into_box() }}
    #[inline(always)]
    unsafe fn Z(self) -> Box<u32> { unsafe { self.0.0.into_box() }}
}

#[rustfmt::skip]
#[expect(non_snake_case)]
impl<'t> FooConverter<&'t Foo> {
    #[inline(always)]
    unsafe fn X(self) -> &'t u128 { unsafe { self.0.0.as_ref() }}
    #[inline(always)]
    unsafe fn Y(self) -> &'t u64 { unsafe { self.0.0.as_ref() }}
    #[inline(always)]
    unsafe fn Z(self) -> &'t u32 { unsafe { self.0.0.as_ref() }}
}

#[rustfmt::skip]
#[expect(non_snake_case)]
impl<'t> FooConverter<&'t mut Foo> {
    #[inline(always)]
    unsafe fn X(self) -> &'t mut u128 { unsafe { self.0.0.as_mut() }}
    #[inline(always)]
    unsafe fn Y(self) -> &'t mut u64 { unsafe { self.0.0.as_mut() }}
    #[inline(always)]
    unsafe fn Z(self) -> &'t mut u32 { unsafe { self.0.0.as_mut() }}
}

// ------------------------------
// Macro usage demonstration
// ------------------------------

pub fn foo(f: Foo) -> u128 {
    match_compact!(f {
        Foo::X(x) => *x,
        Foo::Y(y) => u128::from(*y),
        Foo::Z(z) => u128::from(*z),
    })

    /*
    // Equivalent to:
    match f.unpack() {
        FooUnpacked::X(x) => *x,
        FooUnpacked::Y(y) => u128::from(*y),
        FooUnpacked::Z(z) => u128::from(*z),
    }
    */
}

pub fn foo_with_default(f: Foo) -> u128 {
    match_compact!(f {
        Foo::X(x) => *x,
        _ if f.is_x_or_y() => 1,
        _ => 0,
    })
}

pub fn foo_ref(f: &Foo) -> u128 {
    match_compact!(f {
        Foo::X(x) => *x,
        Foo::Y(y) => u128::from(*y),
        Foo::Z(z) => u128::from(*z),
    })

    /*
    // Equivalent to:
    match f.as_ref() {
        FooRef::X(x) => *x,
        FooRef::Y(y) => u128::from(*y),
        FooRef::Z(z) => u128::from(*z),
    }
    */
}

pub fn foo_mut(f: &mut Foo) -> u128 {
    match_compact!(f {
        Foo::X(x) => *x,
        Foo::Y(y) => u128::from(*y),
        Foo::Z(z) => u128::from(*z),
    })

    /*
    // Equivalent to:
    match f.as_mut() {
        FooMut::X(x) => *x,
        FooMut::Y(y) => u128::from(*y),
        FooMut::Z(z) => u128::from(*z),
    }
    */
}

pub fn foo_mut_with_default(mut f: Foo) -> u128 {
    match_compact!(&mut f {
        Foo::X(x) => *x,
        _ if f.is_x_or_y() => 1,
        _ => 0,
    })
}

pub struct Bar {
    foo: Foo,
}

pub fn bar_with_default(bar: Bar) -> u128 {
    match_compact!(bar.foo, {
        Foo::X(x) => *x,
        thing if thing.is_x_or_y() => 1,
        Foo::Y(y) => u128::from(*y),
        _ => 0,
    })
}

impl Foo {
    pub fn as_u128(&self) -> u128 {
        match_compact!(self {
            Self::X(x) => *x,
            Self::Y(y) => u128::from(*y),
            Self::Z(z) => u128::from(*z),
        })

        /*
        // Equivalent to:
        match self.unpack() {
            FooUnpacked::X(x) => *x,
            FooUnpacked::Y(y) => u128::from(*y),
            FooUnpacked::Z(z) => u128::from(*z),
        }
        */
    }

    pub fn is_x(&self) -> bool {
        matches_compact!(self, Self::X(_))
        /*
        // Equivalent to:
        matches!(self.as_ref(), FooRef::X(_))
        */
    }

    pub fn is_x_or_y(&self) -> bool {
        matches_compact!(self, Self::X(_) | Self::Y(_))
        /*
        // Equivalent to:
        matches!(self.as_ref(), FooRef::X(_) | FooRef::Y(_))
        */
    }

    pub fn is_x_over_10(&self) -> bool {
        matches_compact!(self, Self::X(x) if *x > 10)
        /*
        // Equivalent to:
        matches!(self.as_ref(), FooRef::X(x) if *x > 10)
        */
    }
}
