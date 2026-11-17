//! Lowering of standard (TC39 2023-11) auto-accessors.
//!
//! `decorator.lower: true` lowers undecorated `accessor` properties to private storage plus a
//! getter/setter pair. Decorated classes are still preserved for a downstream decorator transform.
//!
//! Undecorated auto-accessors are rewritten in place when class fields are native. Standard and
//! legacy decorators share the storage/getter/setter builder. Static accessors use a class reference
//! as their receiver, so inherited accessors continue to access the defining class's storage.
//!
//! Optional calls on lowered private fields use `?.bind`, so an undefined private callee
//! short-circuits. This also affects undecorated classes at targets which lower private fields.
//!
//! `StandardDecoratorState` is split by concern: `accessor` lowers `accessor` properties, `name`
//! handles named evaluation, and `element` scopes unbraced loop bodies. Class-properties drives
//! them; see `es2022/class_properties/decorators.rs`.

use rustc_hash::FxHashMap;

use oxc_allocator::Address;
use oxc_ast::ast::*;
use oxc_str::JSStr;

use crate::decorator::DecoratorMode;

mod accessor;
mod element;
mod name;

/// Standard decorator state shared across the class-properties traversal.
///
/// Node metadata survives visits to nested classes.
#[derive(Default)]
pub struct StandardDecoratorState<'a> {
    /// If `true`, lower standard (TC39) decorators.
    enabled: bool,
    /// Whether loop subtrees contain a class that needs decorator lowering.
    loop_classes: FxHashMap<Address, bool>,
    /// Original key names of `accessor` storage fields, for named evaluation.
    accessor_names: FxHashMap<Address, JSStr<'a>>,
    /// Names of anonymous class expressions in instance field initializers, for named evaluation.
    named_classes: FxHashMap<Address, Expression<'a>>,
}

impl StandardDecoratorState<'_> {
    pub(crate) fn new(mode: DecoratorMode) -> Self {
        Self { enabled: mode == DecoratorMode::Standard, ..Self::default() }
    }

    pub(crate) fn enabled(&self) -> bool {
        self.enabled
    }

    /// Whether a class exit left unconsumed entries in the class-keyed maps.
    ///
    /// Class-keyed maps must be drained by that class's exit.
    pub(crate) fn has_leftover_class_entries(&self) -> bool {
        !self.named_classes.is_empty()
    }
}
