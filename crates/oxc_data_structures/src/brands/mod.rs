//! Brands. A way to tie values to the one object which created them, checked at compile time.
//!
//! # The problem
//!
//! Some values are only valid when used with the object which created them.
//!
//! e.g. `Names` is a list of names. Names can be added to it, but never removed.
//! `Names::push` adds a name, and returns a `NameIndex`, which is the index of that name.
//! `Names::get` takes a `NameIndex`, and returns the name at that index, without a bounds check.
//! That's only sound if the `NameIndex` came from that `Names`. One from a different `Names`
//! could be out of bounds.
//!
//! `Names` could check at runtime. Each `Names` would have a unique ID number, each `NameIndex`
//! would store the ID of the `Names` which created it, and `get` would check the IDs match.
//! But that'd make `NameIndex` larger, and cost a check on every call.
//!
//! # The solution
//!
//! A brand works the same way as the ID number, except the ID is a lifetime `'id`.
//! So it's the compiler which checks that IDs match, and there's no runtime cost.
//!
//! - Each `Names` has its own brand, identified by a lifetime `'id` which no other `Names` has.
//! - Each `NameIndex` which that `Names` creates carries its brand, so has the same lifetime `'id`.
//! - `Names<'id>::get` only accepts a `NameIndex<'id>`.
//!   Passing it a `NameIndex` from any other `Names` is a type error.
//!
//! The code for `Names` and `NameIndex` is in the [example](#example) below.
//!
//! In these docs:
//!
//! - "Owner" is the object which a brand is unique to. e.g. a `Names<'id>`.
//! - "Branded values" are the values which carry the owner's brand. e.g. `NameIndex<'id>`s.
//!
//! # The types
//!
//! This module provides a type for each of them to hold.
//!
//! - [`BrandOrigin<'id>`] is held by the owner. It's the single origin of brand `'id`.
//!   There's only one for each `'id`, and it can't be copied, so there's only one owner
//!   for each `'id`.
//! - [`Brand<'id>`] is held by each branded value. It's the brand itself.
//!   It can be copied, but otherwise can only be obtained from the owner's [`BrandOrigin<'id>`]
//!   (with [`BrandOrigin::brand`]). So only the owner can create branded values from scratch.
//!
//! There can be many [`Brand<'id>`]s, but they're all copies of the same brand,
//! and all come from the same [`BrandOrigin<'id>`].
//!
//! Both types are zero-sized, so cost nothing at runtime.
//!
//! ## Why 2 types?
//!
//! A brand only works if there's one owner for each `'id`. [`Brand`] can't guarantee that,
//! because it can be copied.
//!
//! If owners were created from a `Brand`, e.g. `Names::new(brand)`, nothing would stop that
//! being called twice with the same brand. That'd create 2 `Names`s with the same `'id`,
//! and each would accept the other's `NameIndex`es. So `Names::new` would have to be
//! an unsafe function, whose callers promise not to create any other `Names` with the same `Brand`.
//! Where one owner contains another, every constructor on the way down would need to be unsafe,
//! with the same promise.
//!
//! [`BrandOrigin`] makes the compiler enforce unique ownership, and avoids this hazard.
//! It can't be copied, and the owner takes it by value, so it can only ever be used
//! to create one owner. Constructors which take a `BrandOrigin` can be safe functions.
//!
//! # What makes `'id` unique
//!
//! A [`BrandOrigin`] is never returned from a function. [`BrandOrigin::scope`] creates one,
//! and passes it to a closure instead. That closure is the brand's scope.
//!
//! ```ignore
//! pub fn scope<R>(f: impl for<'id> FnOnce(BrandOrigin<'id>) -> R) -> R
//! ```
//!
//! - `for<'id>` means `f` must accept a `BrandOrigin<'id>` with *any* lifetime `'id`.
//!   So code in `f` knows nothing about `'id`, and the compiler won't let it treat `'id`
//!   as the same as any other lifetime. That includes the `'id` of another `BrandOrigin`.
//! - `R` is declared outside `for<'id>`, so the type which `f` returns can't contain `'id`.
//!   Nor can the type of any variable outside `f`, so `f` can't store the brand there either.
//!   So the brand can't leave its scope. The owner and its branded values only exist inside `f`.
//!
//! The compiler enforces both, so creating and using a brand needs no `unsafe`.
//!
//! An owner type usually wraps this in a `scope` function of its own, with the same shape,
//! so its users don't have to handle the `BrandOrigin`. e.g. `Names::scope` in the example below.
//!
//! # Example
//!
//! ```
//! use oxc_data_structures::brands::{Brand, BrandOrigin};
//!
//! /// List of names. Names can be added, but never removed.
//! struct Names<'id> {
//!     names: Vec<String>,
//!     /// Origin of this `Names`'s brand `'id`
//!     brand_origin: BrandOrigin<'id>,
//! }
//!
//! /// Index of a name in the `Names<'id>` which created it.
//! #[derive(Clone, Copy)]
//! struct NameIndex<'id> {
//!     index: usize,
//!     /// Brand of the `Names<'id>` which created this index
//!     brand: Brand<'id>,
//! }
//!
//! impl Names<'_> {
//!     /// Create a `Names` with a new brand, and call `f` with it. Returns what `f` returns.
//!     fn scope<R>(f: impl for<'id> FnOnce(Names<'id>) -> R) -> R {
//!         BrandOrigin::scope(|brand_origin| {
//!             let names = Names { names: vec![], brand_origin };
//!             f(names)
//!         })
//!     }
//! }
//!
//! impl<'id> Names<'id> {
//!     /// Add a name. Returns its index.
//!     fn push(&mut self, name: &str) -> NameIndex<'id> {
//!         let index = self.names.len();
//!         self.names.push(name.to_string());
//!         NameIndex { index, brand: self.brand_origin.brand() }
//!     }
//!
//!     /// Get the name at `index`.
//!     fn get(&self, index: NameIndex<'id>) -> &str {
//!         // SAFETY: `index` has the same brand `'id` as this `Names`, so this `Names` created it.
//!         // `index.index` was in bounds then, and names are never removed, so it still is.
//!         unsafe { self.names.get_unchecked(index.index) }
//!     }
//! }
//!
//! Names::scope(|mut names1| {
//!     Names::scope(|mut names2| {
//!         let ann = names1.push("Ann");
//!
//!         names2.push("Bob");
//!         let cat = names2.push("Cat");
//!
//!         assert_eq!(names1.get(ann), "Ann");
//!         assert_eq!(names2.get(cat), "Cat");
//!
//!         // Compile error. `cat` doesn't have the same brand as `names1`.
//!         // Without brands, it would compile, and read out of bounds.
//!         // names1.get(cat);
//!     });
//! });
//! ```
//!
//! # Using one lifetime as both brand and borrow
//!
//! An owner which borrows data can use the same lifetime for the borrow and for its brand.
//! e.g. `Text<'a>` holds a `&'a str`, and `'a` is also its brand.
//! That saves adding a 2nd lifetime to the type, but takes more work at both ends of `f`.
//!
//! - Going in:
//!   The `&'data str` has to be shortened to a `&'id str`. The compiler only allows that
//!   if it knows `'data` outlives `'id`, and in `f` it knows nothing about `'id`.
//!   [`BrandOrigin::scope_within`] solves that. It creates a scope which is within `'data`,
//!   and passes `f` a [`BrandScope<'id, 'data>`] to say so.
//! - Coming out:
//!   Whatever `f` gets from the owner has lifetime `'id`, so can't be returned from `f`,
//!   even if it contains nothing branded (e.g. a `&'id str` slice of the text).
//!   [`BrandScope::extend_lifetime`] solves that. It changes the lifetime from `'id` back
//!   to `'data`. It only accepts types which implement [`Unbranded`], so the brand still
//!   can't leave `f`.
//!
//! See [`BrandOrigin::scope_within`] for an example.
//!
//! # Rules for an owner
//!
//! The compiler checks the lifetimes. It can't check these:
//!
//! - The owner must take its [`BrandOrigin`] by value, and keep it. A type which only holds
//!   a [`Brand`] isn't an owner. [`Brand`] is `Copy`, so there can be many of them.
//! - Keep the field which holds a [`Brand`] private. Code which can read that field can copy
//!   the brand into a branded value of its own making.
//! - If the owner uses [`BrandOrigin::scope_within`], its unsafe code must treat `'id` as if
//!   it was `'data`. Anything which unsafe code gives lifetime `'id` must be valid for `'data`,
//!   because [`BrandScope::extend_lifetime`] turns the one into the other.
//!   e.g. Don't return a `&'id T` which points to memory that's freed when the scope ends.
//!
//! # Details
//!
//! - [`BrandOrigin`] and [`Brand`] are invariant over `'id`. If they were covariant,
//!   compiler would shorten 2 different brands to a common lifetime wherever that made them match.
//!   e.g. a function which takes 2 branded values with the same `'id` would accept values
//!   from 2 different owners.
//! - [`BrandScope`] is invariant over both its lifetimes.
//!   If it was covariant over `'id`, [`BrandScope::extend_lifetime`] could extend the lifetime
//!   of a reference to a local variable. If it was contravariant over `'data`, it could extend
//!   a lifetime to longer than `'data`.
//! - Both brand types have a private field. So outside this module, the only ways to obtain one
//!   are [`BrandOrigin::scope`], [`BrandOrigin::scope_within`], [`BrandOrigin::brand`],
//!   or copying an existing [`Brand`].

use std::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    mem,
};

mod unbranded;

#[cfg(doctest)]
mod compile_fail_tests;
#[cfg(test)]
mod tests;

pub use unbranded::Unbranded;

/// The single origin of brand `'id` (see [module docs]).
///
/// Held by the owner of the brand.
/// Every [`Brand<'id>`] comes from it, via [`BrandOrigin::brand`].
///
/// Created by [`BrandOrigin::scope`] or [`BrandOrigin::scope_within`], which pass it to a closure.
/// That closure is the brand's scope, and the [`BrandOrigin`] can't leave it.
///
/// Not `Copy` or `Clone`, so there's only one [`BrandOrigin`] for each `'id`,
/// and therefore only one owner for each `'id`.
///
/// [module docs]: self
#[repr(transparent)]
pub struct BrandOrigin<'id>(InvariantLifetime<'id>);

impl BrandOrigin<'_> {
    /// Create a [`BrandOrigin`] for a new brand, and call `f` with it. Returns what `f` returns.
    ///
    /// `f` is the scope of the brand.
    ///
    /// - `f` must accept a `BrandOrigin<'id>` with any lifetime `'id`, so code in `f` can't treat
    ///   `'id` as the same as any other lifetime. That's what makes the brand unique.
    /// - The type which `f` returns can't contain `'id`, so the [`BrandOrigin`],
    ///   and anything else which has the brand, can't leave `f`.
    ///
    /// See [module docs] for an example.
    ///
    /// [module docs]: self
    #[expect(clippy::inline_always, reason = "only calls `f`")]
    #[inline(always)]
    pub fn scope<R>(f: impl for<'id> FnOnce(BrandOrigin<'id>) -> R) -> R {
        f(BrandOrigin(PhantomData))
    }

    /// Same as [`BrandOrigin::scope`], but the scope is within lifetime `'data`.
    /// i.e. code in `f` can also rely on `'data` outliving `'id`.
    ///
    /// For an owner which uses one lifetime as both its brand and the lifetime of data it borrows
    /// (see [module docs]). In `f`, a reference with lifetime `'data` can be shortened to `'id`.
    ///
    /// `f` is also passed a [`BrandScope<'id, 'data>`].
    ///
    /// - It's the presence of that type in the signature of `f` which tells the compiler
    ///   that `'data` outlives `'id`. `f` doesn't need to do anything with it for that.
    /// - Anything which `f` gets from the owner has lifetime `'id`, so can't be returned from `f`.
    ///   To return it, change its lifetime back to `'data` with [`BrandScope::extend_lifetime`].
    ///
    /// `f` doesn't have to give the type of its `BrandScope` parameter.
    /// The compiler can infer `'data`. The example below gives it (`BrandScope<'_, 'a>`)
    /// to make clear which lifetime the scope is within.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_data_structures::brands::{BrandOrigin, BrandScope};
    ///
    /// /// Holds text. `'a` is both the lifetime of the text, and this `Text`'s brand.
    /// struct Text<'a> {
    ///     text: &'a str,
    ///     brand_origin: BrandOrigin<'a>,
    /// }
    ///
    /// fn first_word<'a>(text: &'a str) -> &'a str {
    ///     BrandOrigin::scope_within(|brand_origin, brand_scope: BrandScope<'_, 'a>| {
    ///         // `text` is shortened from `&'a str` to `&'id str` here
    ///         let text = Text { text, brand_origin };
    ///
    ///         // `word` is a `&'id str`
    ///         let word = text.text.split(' ').next().unwrap();
    ///
    ///         // Change it back to a `&'a str`, so it can be returned
    ///         brand_scope.extend_lifetime::<&str>(word)
    ///     })
    /// }
    ///
    /// assert_eq!(first_word("hello world"), "hello");
    /// ```
    ///
    /// [module docs]: self#using-one-lifetime-as-both-brand-and-borrow
    #[expect(clippy::inline_always, reason = "only calls `f`")]
    #[inline(always)]
    pub fn scope_within<'data, R>(
        f: impl for<'id> FnOnce(BrandOrigin<'id>, BrandScope<'id, 'data>) -> R,
    ) -> R {
        f(BrandOrigin(PhantomData), BrandScope(PhantomData, PhantomData))
    }
}

impl<'id> BrandOrigin<'id> {
    /// Get the [`Brand`], to put in a value which belongs to the owner of this [`BrandOrigin`].
    #[expect(
        clippy::unused_self,
        reason = "taking `&self` proves the caller holds the `BrandOrigin`"
    )]
    #[expect(clippy::inline_always, reason = "no-op at runtime")]
    #[inline(always)]
    pub fn brand(&self) -> Brand<'id> {
        Brand(PhantomData)
    }
}

impl Debug for BrandOrigin<'_> {
    // A derived impl would also print the `PhantomData` field, which is only noise
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("BrandOrigin")
    }
}

/// Brand `'id` (see [module docs]).
///
/// Held by each branded value. It marks that value as belonging to the owner of brand `'id`.
///
/// It's `Copy`, but otherwise can only be obtained from the owner's [`BrandOrigin`],
/// with [`BrandOrigin::brand`]. So there can be many `Brand<'id>`s, but they're all copies
/// of the same brand.
///
/// [module docs]: self
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Brand<'id>(InvariantLifetime<'id>);

impl Debug for Brand<'_> {
    // A derived impl would also print the `PhantomData` field, which is only noise
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Brand")
    }
}

/// The scope of brand `'id`, which is within lifetime `'data`. i.e. `'data` outlives `'id`.
///
/// [`BrandOrigin::scope_within`] passes a `BrandScope<'id, 'data>` to its closure.
/// It does 2 jobs:
///
/// - Shortening lifetimes from `'data` to `'id`. For this, it's the type which matters,
///   not the value. Code in a function which has a parameter of type `BrandScope<'id, 'data>`
///   can rely on `'data` outliving `'id`.
/// - Extending lifetimes from `'id` back to `'data`, with [`BrandScope::extend_lifetime`].
///
/// Zero-sized.
//
// The `'data: 'id` bound is what tells the compiler that `'data` outlives `'id`,
// in any function which has a `BrandScope<'id, 'data>` parameter
pub struct BrandScope<'id, 'data: 'id>(InvariantLifetime<'id>, InvariantLifetime<'data>);

impl<'id, 'data> BrandScope<'id, 'data> {
    /// Change lifetime of `value` from `'id` to `'data`.
    ///
    /// `T` is the type of `value` (with any lifetime), and has to be given explicitly.
    /// e.g. `brand_scope.extend_lifetime::<&str>(text)`.
    ///
    /// `T` must implement [`Unbranded`], so `value` can't be a brand, or contain one.
    ///
    /// See [`BrandOrigin::scope_within`] for an example.
    #[expect(
        clippy::unused_self,
        reason = "taking `&self` proves `'id` and `'data` are the lifetimes of a scope"
    )]
    #[expect(clippy::inline_always, reason = "no-op at runtime")]
    #[inline(always)]
    pub fn extend_lifetime<T: Unbranded>(
        &self,
        value: T::WithLifetime<'id>,
    ) -> T::WithLifetime<'data> {
        // SAFETY: `T::WithLifetime<'id>` and `T::WithLifetime<'data>` are the same type,
        // apart from the lifetime.
        //
        // `BrandScope` is only created by `BrandOrigin::scope_within`, which passes its closure
        // a `BrandScope<'id, 'data>`. `BrandScope` is invariant over both lifetimes,
        // so `'data` and `'id` here are exactly the `'data` and `'id` of that scope.
        //
        // - The brand doesn't leave the closure. `Unbranded`'s contract is that nothing relies on
        //   the lifetime of `T::WithLifetime<'id>` to identify an owner.
        // - `value` is valid for `'data`. Code in the closure knows nothing about `'id`,
        //   except that `'data` outlives it. So safe code can only give lifetime `'id` to:
        //   - Data which is valid for `'data` or longer.
        //   - Data which lives forever (e.g. `Box::leak`).
        //   - A local variable of a closure which never returns. But compiler rejects any call
        //     after such a borrow (the call could unwind, and free the variable),
        //     including this call.
        //
        //   Unsafe code must follow the same rule (see "Rules for an owner" in module docs).
        unsafe { mem::transmute::<T::WithLifetime<'id>, T::WithLifetime<'data>>(value) }
    }
}

/// Marker which makes a type invariant over lifetime `'id`.
///
/// It uses a `fn` pointer because that's `Send` and `Sync`.
/// Other invariant markers are not (e.g. `PhantomData<*mut &'id ()>`).
type InvariantLifetime<'id> = PhantomData<fn(&'id ()) -> &'id ()>;
