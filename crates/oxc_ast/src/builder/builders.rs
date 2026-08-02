//! Builders which construct AST nodes in place in the memory arena.
//!
//! A node is built by calling `build` on the AST type, setting each of its fields,
//! then calling `finish`:
//!
//! ```ignore
//! let binary_expr = BinaryExpression::build(builder)
//!     .span_start(span_start)
//!     .left(lhs)
//!     .operator(operator)
//!     .right(self.parse_binary_expression_or_higher(precedence))
//!     .span_end(self.prev_token_end)
//!     .finish();
//! ```
//!
//! `build` methods allocate the node's memory up front, and each setter writes its field
//! into the arena immediately.
//!
//! Writing a field can also be delegated to another function via a [`Slot`]:
//!
//! ```ignore
//! let stmt = LabeledStatement::build(builder)
//!     .span_start(span_start)
//!     .label_with(|slot| self.parse_label_identifier(slot))
//!     .body_with(|slot| self.parse_statement(slot))
//!     .span_end(self.prev_token_end)
//!     .finish();
//!
//! // Function returns a zero-size `SlotFilled` token which guarantees
//! // the node has had all its fields initialized
//! fn parse_label_identifier<'s>(&mut self, slot: Slot<'s, LabelIdentifier<'a>>) -> SlotFilled<'s> {
//!     let span = self.next_token().span();
//!     slot.build(self)
//!         .span(span)
//!         .name(span.source_text(self.source_text))
//!         .finish()
//! }
//!
//! fn parse_statement<'s>(&mut self, slot: Slot<'s, Statement<'a>>) -> SlotFilled<'s> {
//!     let token = self.next_token();
//!     if token.kind() == Kind::Semicolon {
//!         return slot.build_empty_statement(self).span(token.span()).finish();
//!     }
//!     // ...
//! }
//! ```
//!
//! ## Why?
//!
//! The original AST builder methods (e.g. `BinaryExpression::boxed`) construct all of a node's
//! fields first, then allocates space for the `BinaryExpression` in arena, and only *then*
//! writes fields into the arena.
//!
//! This means that usually the node has to be constructed on the stack first, and then *copied*
//! into the arena.
//!
//! The in-place builders avoid this copying. Allocating space for the node happens *first*,
//! and each field is written immediately into its place in the arena as soon as it's ready,
//! with no intermediate step of constructing on the stack.
//!
//! To gain the greatest advantage, avoid returning AST types from functions, and instead
//! pass functions a [`Slot`] to fill (see example above).
//!
//! The builder types and methods are a lot of code, but ultimately a builder is just a pointer
//! (1 register). Every method is `#[inline(always)]`, so all this code compiles away entirely.
//! Primarily, the builder code exists only at the type level - ensuring that every field
//! of a node is initialized before the node is considered complete.
//!
//! ## Fields
//!
//! `node_id` is automatically filled in by the `build` method itself, and has no setter.
//!
//! `span` gets 3 setters:
//!
//! * `span_start` and `span_end` methods write the two halves separately, for a parser which knows
//!   where a node starts before parsing its children, and where it ends only afterwards.
//! * `span` method writes both `start` and `end` at once.
//!
//! If a node is abandoned part-built, its memory is simply wasted. AST types are never [`Drop`].
//!
//! ## Tracking which fields have been set
//!
//! A builder's `State` param is a tuple with one entry per field, which is [`No`] until that
//! field's setter is called, and [`Set`] afterwards.
//!
//! `finish` requires all fields to be [`Set`], so a field cannot be forgotten.
//! Omitting one gives ``error: `.operator()` has not been called on this builder``.
//!
//! ```
//! # use oxc_allocator::Allocator;
//! # use oxc_ast::{ast::IdentifierReference, builder::AstBuilder};
//! # use oxc_span::Span;
//! # let allocator = Allocator::new();
//! # let builder = AstBuilder::new(&allocator);
//! let ident = IdentifierReference::build(&builder)
//!     .span(Span::new(3, 6))
//!     .name("foo")
//!     .defaults()
//!     .finish();
//! ```
//!
//! Leave a field out and `finish` is not callable. The error names the setter which was missed:
//!
//! ```compile_fail
//! # use oxc_allocator::Allocator;
//! # use oxc_ast::{ast::IdentifierReference, builder::AstBuilder};
//! # use oxc_span::Span;
//! # let allocator = Allocator::new();
//! # let builder = AstBuilder::new(&allocator);
//! // Error: `.name()` has not been called on this builder
//! let ident = IdentifierReference::build(&builder).span(Span::new(3, 6)).defaults().finish();
//! ```
//!
//! The two halves of `span` are tracked separately, so setting only one is not enough:
//!
//! ```compile_fail
//! # use oxc_allocator::Allocator;
//! # use oxc_ast::{ast::IdentifierReference, builder::AstBuilder};
//! # let allocator = Allocator::new();
//! # let builder = AstBuilder::new(&allocator);
//! // Error: `.span_end()` has not been called on this builder
//! let ident = IdentifierReference::build(&builder).span_start(3).name("foo").defaults().finish();
//! ```
//!
//! A field with a default value is no exception - it is set by `defaults`, and forgetting to
//! call that is caught in the same way:
//!
//! ```compile_fail
//! # use oxc_allocator::Allocator;
//! # use oxc_ast::{ast::IdentifierReference, builder::AstBuilder};
//! # use oxc_span::Span;
//! # let allocator = Allocator::new();
//! # let builder = AstBuilder::new(&allocator);
//! // Error: `.reference_id()` has not been called on this builder
//! let ident = IdentifierReference::build(&builder).span(Span::new(3, 6)).name("foo").finish();
//! ```
//!
//! A builder reads its `State` through the `FieldsState` trait, which every builder shares.
//! It has as many slots as the largest builder needs, so a builder with fewer fields simply
//! leaves the rest unset and never names them. Code generic over a builder's state bounds it on
//! that trait, and names each field's state as a projection off it e.g. `State::Field3`.
//!
//! ## Building a field in place
//!
//! A field whose type is too large to return in registers gets a second setter method,
//! `<field>_with`, which hands the callee a [`Slot`] for that field:
//!
//! ```ignore
//! let function = Function::build(builder)
//!     .id_with(|slot| self.parse_binding_identifier(slot))
//!     // ...
//!     .finish();
//! ```
//!
//! The callee either [`fill`]s the slot with a value, or - if the field holds an AST node -
//! calls [`build`] on it to get a builder writing straight into the field. Either way it ends
//! with the [`SlotFilled`] token which `<field>_with` demands.
//!
//! This saves the callee returning the value on the stack for the caller to copy into place.
//!
//! A field which wraps its node gets a slot for the wrapper, which is narrowed first:
//!
//! * [`into_some`] converts a `Slot<Option<T>>` to a `Slot<T>`.
//! * [`into_contents`] converts a `Slot<ArenaBox<T>>` to a `Slot<T>`,
//!   reserving the node's memory and writing the [`ArenaBox`] on the way.
//!
//! Each step writes everything at its own level and hands on a slot for what remains,
//! so one token at the end stands for all of them:
//!
//! ```ignore
//! // Filling a `Slot` for an `Option<IdentifierName>` with `Some`
//! builder.id_with(|slot| slot.into_some().build(builder).span(span).name(name).finish())
//! // Filling a `Slot` for an `Option<IdentifierName>` with `None`
//! builder.id_with(|slot| slot.fill(None))
//! ```
//!
//! ```ignore
//! // Filling a `Slot` for an `ArenaBox<IdentifierName>`
//! // using a function that takes a `Slot` for an `IdentifierName`
//! fn write_ident_name<'a, 's>(slot: Slot<'s, IdentifierName<'a>>) -> SlotFilled<'s> {
//!     slot.build().span(SPAN).name("hello").finish()
//! }
//!
//! builder.id_with(|slot: Slot<'_, ArenaBox<'s, IdentifierName<'a>>>| {
//!     // Narrow the `Slot`. This allocates to create the `ArenaBox`.
//!     let slot: Slot<'_, IdentifierName<'a>> = slot.into_contents(&allocator);
//!     // Write into the narrowed `Slot`
//!     write_ident_name(slot)
//! })
//! ```
//!
//! ## Creating an `ArenaBox` to fill as a `Slot`
//!
//! You can also create an uninitialized [`ArenaBox`] and fill it with a function
//! that takes a [`Slot`]:
//!
//! ```ignore
//! // Create an `ArenaBox<'a, MaybeUninit<IdentifierName<'a>>>`
//! let boxed = IdentifierName::uninit(&allocator);
//! // Fill the box
//! let boxed = boxed.fill_with(|slot: Slot<IdentifierName<'a>>| write_ident_name(slot));
//! // `boxed` is now an initialized `ArenaBox<'a, IdentifierName<'a>>`
//! ```
//!
//! ## SAFETY
//!
//! `build` reserves memory in the arena sized and aligned for the node, so every one of
//! the node's fields is valid for writing, which is all a setter does.
//!
//! [`Set`] can only enter a type param by calling the setter which writes that field,
//! and the `*IsSet` traits which `finish` is bound by are sealed, so `finish` can only
//! be reached once every field has been written.
//!
//! A `<field>_with` method ensures the callee fills the field, by passing it a [`Slot`]
//! for the field, and requiring the corresponding [`SlotFilled`] token to be returned.
//!
//! The [`SlotFilled`] token carries a lifetime brand tied to that one slot,
//! so it uniquely identifies *which* slot it guarantees has been filled.
//! A [`SlotFilled`] token for one [`Slot`] cannot be used for any other slot.
//!
//! ## Supporting types
//!
//! The builders themselves are generated, one per AST type, in `generated/builders.rs`,
//! and re-exported from here. The types they are built on are defined in this file.
//!
//! [`Slot`], [`OwnedSlot`], [`SlotFilled`] and [`FillSlot`] are general-purpose arena types,
//! defined in `oxc_allocator`. They're used here, and re-exported for convenience.
//!
//! What this module adds on top of them is specific to AST nodes:
//!
//! * [`SlotBuild`] trait, which provides [`build`] method for slots containing AST nodes.
//! * [`BuilderTarget`] trait, which extends [`FillSlot`].
//! * `EnumSlot` trait, which provides enum layout logic.
//!
//! A builder is parameterized by a [`BuilderTarget`], which holds the exclusive reference
//! to the memory the node is built in, and says what `finish` returns. There are 2:
//!
//! * [`OwnedSlot<T>`]
//!   * The node has an allocation of its own.
//!   * `finish` returns an [`ArenaBox<T>`].
//! * [`Slot<T>`]
//!   * The node is written into memory its parent owns.
//!   * `finish` returns a [`SlotFilled`] token proving that the slot was filled.
//!
//! [`fill`]: Slot::fill
//! [`build`]: SlotBuild::build
//! [`into_some`]: Slot::into_some
//! [`into_contents`]: Slot::into_contents
//! [`ArenaBox`]: oxc_allocator::ArenaBox
//! [`ArenaBox<T>`]: oxc_allocator::ArenaBox
//! [`FillSlot`]: oxc_allocator::FillSlot
//! [`OwnedSlot`]: oxc_allocator::OwnedSlot
//! [`SlotFilled`]: oxc_allocator::SlotFilled

#![expect(clippy::inline_always)]

use std::cell::Cell;

use oxc_syntax::node::NodeId;

// `FillSlot` is not re-exported. `BuilderTarget` below has it as a supertrait, so a bound on `BuilderTarget`
// gets everything a builder needs without putting `FillSlot`'s methods in scope at the call site.
use oxc_allocator::FillSlot;

// Re-exported for convenience - the builder API is written in terms of these, but they are
// general-purpose arena types, not AST ones, so they are defined in `oxc_allocator`
pub use oxc_allocator::{OwnedSlot, Slot, SlotFilled};

use super::{AstBuild, GetAstBuilder};

/// Export all the builder types, and the [`traits`] module holding the `Slot` extension traits.
///
/// The traits are behind a module rather than exported alongside the builders, so that they
/// can be brought into scope as a group with `use oxc_ast::builder::builders::traits::*`.
pub use crate::generated::builders::{FieldsState, NoFieldsSet, builders::*, markers, traits};

/// Where a builder writes the `T` it is building.
///
/// Nothing but a rename of [`FillSlot`], which supplies everything - `Output`, and the methods
/// a builder writes through. `Target: BuilderTarget<Program>` says what the bound is *for*,
/// where `Target: FillSlot<Program>` only says how it works.
///
/// It also keeps [`FillSlot`] out of this crate's namespace, so a caller holding a concrete [`Slot`]
/// does not have `as_mut_ptr` and friends within reach - those need a deliberate
/// `use oxc_allocator::FillSlot`. A bound on this trait does reach them, though - method resolution
/// on a type param goes through supertraits whether or not the supertrait is imported.
///
/// Sealed by inheritance - [`FillSlot`] is sealed, so this cannot be implemented for anything
/// which does not already implement that, which is [`Slot`] and [`OwnedSlot`], and nothing else.
/// That matters because the generated builders write through `Target::as_mut_ptr`'s pointer in
/// `unsafe` code, on the strength of a guarantee only those 2 types make.
///
/// [`FillSlot`]: oxc_allocator::FillSlot
pub trait BuilderTarget<T>: FillSlot<T> {}

impl<T> BuilderTarget<T> for Slot<'_, T> {}

impl<T> BuilderTarget<T> for OwnedSlot<'_, T> {}

/// Start building an AST node in the [`Slot`] for it.
///
/// The counterpart of the `build` method on the node type, for a node written into memory
/// its parent owns:
///
/// ```ignore
/// .argument_with(|slot| slot.build(builder).span(span).value(value).finish())
/// ```
///
/// [`Slot`] belongs to `oxc_allocator`, so this cannot be an inherent method on it.
/// The impls are generated, one per AST type.
pub trait SlotBuild<'a>: Sized {
    /// Builder for the node this [`Slot`] covers.
    type Builder;

    /// Start building the node, in place in the memory this [`Slot`] covers.
    ///
    /// Set every field on the returned builder, then call `finish`.
    fn build(self, builder: &impl GetAstBuilder<'a>) -> Self::Builder;
}

/// A [`Slot`] for an enum, which can be narrowed to one for a variant's payload.
///
/// Supertrait of the generated `<Enum>Slot` traits, which is where their `into_<variant>` methods
/// get [`into_variant`] from. Each of those traits names its own enum as `T`, which is what
/// makes their method bodies sound - see [`into_variant`].
///
/// Sealed, and `pub(crate)`, so only this crate can implement it, and only for [`Slot`].
/// That stops a downstream `impl ExpressionSlot for MyType {}` from picking up those method bodies
/// and running [`Expression`]'s layout reasoning against something which is not one.
///
/// # SAFETY
///
/// [`into_variant`] must do what it says - write the discriminant at offset 0 of the place
/// `Self` covers, and return a [`Slot`] for the payload within that same place.
///
/// The generated `<Enum>Slot` traits give their `into_<variant>` methods *safe* bodies which call it,
/// discharging its contract from the `T` in their own bound. An implementation which returned a slot
/// for anywhere else would make those safe methods hand out a slot over the wrong memory.
///
/// Sealing is not enough on its own to prevent that. `T` is a type param rather than an associated type,
/// so a second impl for [`Slot`] with a different `T` does not overlap the blanket one,
/// and would be accepted - e.g. `EnumSlot<'slot, Expression>` for a `Slot<'slot, Statement>`.
///
/// [`into_variant`]: EnumSlot::into_variant
/// [`Expression`]: crate::ast::Expression
pub(crate) unsafe trait EnumSlot<'slot, T>: Sealed + Sized {
    /// Write the discriminant for one variant of the enum, and narrow the [`Slot`] to one for
    /// that variant's payload.
    ///
    /// All of the layout reasoning for enum slots is in the one implementation of this,
    /// and nowhere else.
    ///
    /// A `#[repr(C, u8)]` enum is laid out as a `#[repr(C)]` struct of a `u8` tag followed by
    /// a union of the variants. So the tag is at offset 0, and the union - and therefore every
    /// variant's payload - is at `align_of::<T>()`, that being the alignment of the union,
    /// and hence of the enum.
    ///
    /// # SAFETY
    /// `T` must be a `#[repr(C, u8)]` enum, `discriminant` must be the discriminant of one
    /// of its variants, and `P` must be that variant's payload type.
    unsafe fn into_variant<P>(self, discriminant: u8) -> Slot<'slot, P>;
}

// SAFETY: `Self` is `Slot<'slot, T>`, which covers a place laid out for a `T`
unsafe impl<'slot, T> EnumSlot<'slot, T> for Slot<'slot, T> {
    // `#[inline(always)]` because this is a single store
    #[inline(always)]
    unsafe fn into_variant<P>(mut self, discriminant: u8) -> Slot<'slot, P> {
        // SAFETY: The slot covers memory laid out for a `T`, valid for writes.
        // Caller guarantees `T` is a `#[repr(C, u8)]` enum, so its first byte is the discriminant,
        // and `discriminant` is one of its variants'.
        unsafe { self.as_mut_ptr().cast::<u8>().write(discriminant) };

        // SAFETY: Caller guarantees `P` is the payload type of the variant `discriminant` selects,
        // which is therefore within the memory the slot covers, at offset `align_of::<T>()`.
        // That offset is aligned for `P` - `T`'s alignment is the maximum of all its variants'
        // alignments, so it is a multiple of `P`'s, and `P`'s does not exceed it.
        // Together with the discriminant written above, filling the payload leaves a valid `T`.
        unsafe { self.into_part(align_of::<T>()) }
    }
}

/// Private module, so [`Sealed`] cannot be implemented outside of it,
/// and therefore neither can [`EnumSlot`], which has it as a supertrait.
///
/// [`Sealed`]: private::Sealed
mod private {
    use oxc_allocator::Slot;

    /// Sealing trait, implemented only for [`Slot`].
    pub trait Sealed {}
    impl<T> Sealed for Slot<'_, T> {}
}
use private::Sealed;

/// Marker type for a field which has been set.
pub struct Set;

/// Marker type for a field which has not been set.
pub struct No;

/// Whether a field has been set, as a constant, so that `defaults` methods can skip
/// setting fields which have been set already.
///
/// [`Set`] and [`No`] are the only types which can reach a builder's type params,
/// and both are covered here, so there is nothing another impl could add - no need for sealing.
pub trait FieldState {
    /// `true` if the field has been set.
    const IS_SET: bool;
}

impl FieldState for Set {
    const IS_SET: bool = true;
}

impl FieldState for No {
    const IS_SET: bool = false;
}

/// Write the `node_id` field of a node.
///
/// # SAFETY
/// `node_id_ptr` must point to the `node_id` field of a node in the arena.
//
// `#[inline(always)]` because this is a single store
#[inline(always)]
pub(crate) unsafe fn init_node_id<'a, B: AstBuild<'a>>(
    node_id_ptr: *mut Cell<NodeId>,
    builder: &B,
) {
    // SAFETY: Caller guarantees `node_id_ptr` points to the `node_id` field of a node in the arena,
    // so it is valid for writing
    unsafe { node_id_ptr.write(Cell::new(builder.node_id())) };
}
