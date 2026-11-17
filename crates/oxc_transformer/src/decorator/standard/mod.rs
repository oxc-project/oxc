//! Lowering of standard (TC39 2023-11) decorators.
//!
//! Based on esbuild's `lowerClass` (`internal/js_parser/js_parser_lower_class.go`).
//!
//! Decorated classes are lowered entirely: all fields, private names and static blocks, the same
//! as esbuild's `computeClassLoweringInfo` does when any element is decorated. Decorator calls are
//! emitted after the class using esbuild's runtime helpers:
//!
//! ```js
//! _init = __decoratorStart(base);
//! __decorateElement(_init, flags, key, _x_dec, Class);   // per element, spec order
//! Class = __decorateElement(_init, 0, "Class", _Class_decorators, Class);
//! __runInitializers(_init, 3, Class);                     // static method extra initializers
//! /* static fields and blocks */
//! __runInitializers(_init, 1, Class);                     // class extra initializers
//! ```
//!
//! Undecorated auto-accessors are rewritten in place when class fields are native. Standard and
//! legacy decorators share the storage/getter/setter builder. Static accessors use a class reference
//! as their receiver, so inherited accessors continue to access the defining class's storage.
//!
//! Lowering is opt-in: `decorator.lower: true` lowers decorators and auto-accessors at any target.
//! Otherwise they are preserved for a later tool, while other syntax transforms and TypeScript
//! erasure still run.
//!
//! Babel's 2023-* decorator fixtures are execution oracles, not output snapshots: their helper
//! protocol differs from ours. Local output fixtures cover Oxc's lowering choices. Optional calls
//! on lowered private fields use `?.bind`, so an undefined private callee short-circuits. This also
//! affects undecorated classes at targets which lower private fields.
//!
//! `StandardDecoratorState` is split by concern: `element` collects decorators, `accessor`
//! lowers `accessor` properties, `name` handles named evaluation, and `sequence` emits the
//! decorator calls. Class-properties drives them in evaluation order; see
//! `es2022/class_properties/decorators.rs`.

use rustc_hash::{FxHashMap, FxHashSet};

use oxc_allocator::Address;
use oxc_ast::ast::*;
use oxc_str::JSStr;
use oxc_traverse::BoundIdentifier;

use crate::decorator::DecoratorMode;

mod accessor;
mod class_name;
mod element;
mod name;
mod read_only;
mod sequence;

pub use class_name::ClassNameRewriter;

/// Element kind encoded by the decorator runtime helper protocol.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DecoratorElementKind {
    Method = 1,
    Getter = 2,
    Setter = 3,
    Accessor = 4,
    Field = 5,
}

impl DecoratorElementKind {
    fn bucket(self, is_static: bool) -> DecoratorBucket {
        match (self == Self::Field, is_static) {
            (false, true) => DecoratorBucket::StaticNonField,
            (false, false) => DecoratorBucket::InstanceNonField,
            (true, true) => DecoratorBucket::StaticField,
            (true, false) => DecoratorBucket::InstanceField,
        }
    }
}
const FLAG_STATIC: u32 = 8;
const FLAG_PRIVATE: u32 = 16;

/// Standard decorator state shared across the class-properties traversal.
///
/// Node metadata survives visits to nested classes. Exit-phase scratch buffers are reused
/// for each class; per-class state lives in [`ClassDecoratorState`] on the class stack.
#[derive(Default)]
pub struct StandardDecoratorState<'a> {
    /// If `true`, lower standard (TC39) decorators.
    enabled: bool,
    /// Whether loop subtrees contain a class that needs decorator lowering, for nested unbraced
    /// loops. An entry is removed when it is used. Entries for loops whose body is never searched
    /// remain; arena addresses are not reused during one transform, so they cannot be misread.
    loop_classes: FxHashMap<Address, bool>,
    /// Original key names of `accessor` storage fields initialized with an anonymous class, for
    /// named evaluation. Recorded only for those fields, so this stays small.
    accessor_names: FxHashMap<Address, JSStr<'a>>,
    /// Names of anonymous class expressions in instance field initializers, for named evaluation.
    named_classes: FxHashMap<Address, Expression<'a>>,
    /// Decorated class declarations to convert to `let _Foo = class Foo {}`, keyed by class address.
    /// Values are `(inner binding, outer binding)`.
    declaration_rewrites: FxHashMap<Address, (BoundIdentifier<'a>, BoundIdentifier<'a>)>,
    /// Scratch state reset before emitting each decorated class.
    exit: DecoratorExitState<'a>,
}

/// Scratch state used after all nested classes have finished their traversal.
#[derive(Default)]
struct DecoratorExitState<'a> {
    /// `__decorateElement` calls, in evaluation order.
    elements: [Vec<Expression<'a>>; 4],
    /// `_Class_decorators` binding.
    class_decorators: Option<BoundIdentifier<'a>>,
    /// Length of `insert_before` after class decorators.
    class_prefix_len: usize,
    /// Inferred name of an anonymous decorated class.
    class_name: Option<Expression<'a>>,
    /// Super class temp var binding.
    super_class: Option<BoundIdentifier<'a>>,
    /// `[flags, original key, temporary symbol]` records captured before decoration.
    public_descriptors: Vec<Expression<'a>>,
}

/// Element tables of one class, filled during the entry phase before the class is pushed onto
/// the class stack.
///
/// The entry phase lowers auto-accessors before the class is entered, so these tables are
/// collected first and moved into [`ClassDecoratorState`] when it is created.
#[derive(Default)]
pub struct ClassDecoratorTables<'a> {
    /// Initializer slot index of decorated fields and decorated `accessor` storage fields.
    fields: FxHashMap<Address, u32>,
    /// Storage fields created for decorated `accessor`s.
    accessor_storage: FxHashSet<Address>,
    /// Getters created for `accessor`s, followed immediately by their generated setters.
    accessor_getters: FxHashSet<Address>,
    /// Keys of decorated instance fields, captured before they are moved into constructor.
    instance_keys: FxHashMap<Address, Expression<'a>>,
    /// Methods which may collide, kept on their home object under distinct symbols.
    public_keys: FxHashMap<Address, PublicMethodKey<'a>>,
}

pub struct PublicMethodKey<'a> {
    pub binding: BoundIdentifier<'a>,
    /// The generated setter shares its auto-accessor getter's descriptor.
    pub accessor_setter: bool,
}

/// Decorator state for a decorated class. Stored in `ClassDetails`.
pub struct ClassDecoratorState<'a> {
    /// `_init` var holding the decorator context array.
    pub(crate) init: BoundIdentifier<'a>,
    /// Class contains a decorated instance method / getter / setter.
    pub(crate) instance_method_extras: bool,
    /// Class contains a decorated static method / getter / setter.
    pub(crate) static_method_extras: bool,
    /// Original constructor used by `super` in moved class elements.
    original_class: Option<BoundIdentifier<'a>>,
    /// Element tables of this class.
    pub(crate) tables: ClassDecoratorTables<'a>,
}

/// Bucket indexes for `__decorateElement` calls, in the order they are evaluated.
#[derive(Clone, Copy)]
#[expect(clippy::enum_variant_names)]
enum DecoratorBucket {
    StaticNonField = 0,
    InstanceNonField = 1,
    StaticField = 2,
    InstanceField = 3,
}

/// Bindings selected by class lowering for decorator application and replacement.
pub struct ClassDecoratorBindings<'b, 'a> {
    pub target: &'b BoundIdentifier<'a>,
    pub declaration: Option<&'b BoundIdentifier<'a>>,
    pub temporary: Option<&'b BoundIdentifier<'a>>,
    pub static_brand: Option<&'b BoundIdentifier<'a>>,
}

/// Private-element access selected by class lowering, independent of its storage representation.
pub struct PrivateDecoratorAccess<'a> {
    pub brand: BoundIdentifier<'a>,
    pub value: BoundIdentifier<'a>,
    pub getter: Option<BoundIdentifier<'a>>,
    pub setter: Option<BoundIdentifier<'a>>,
}

impl<'a> StandardDecoratorState<'a> {
    pub(crate) fn new(mode: DecoratorMode) -> Self {
        Self { enabled: mode == DecoratorMode::Standard, ..Self::default() }
    }

    pub(crate) fn enabled(&self) -> bool {
        self.enabled
    }

    /// Begin emission for one decorated class, retaining scratch buffer allocations.
    pub(crate) fn begin_class_exit(&mut self, name: Option<Expression<'a>>) {
        for bucket in &mut self.exit.elements {
            bucket.clear();
        }
        self.exit.class_decorators = None;
        self.exit.class_prefix_len = 0;
        self.exit.class_name = name;
        self.exit.super_class = None;
        self.exit.public_descriptors.clear();
    }

    pub(crate) fn has_class_decorators(&self) -> bool {
        self.exit.class_decorators.is_some()
    }

    /// Whether a class exit left unconsumed entries in the class-keyed maps.
    ///
    /// Per-class element tables live in [`ClassDecoratorState`] and die with their class. The
    /// remaining class-keyed maps must be drained by that class's exit.
    pub(crate) fn has_leftover_class_entries(&self) -> bool {
        !self.named_classes.is_empty() || !self.declaration_rewrites.is_empty()
    }
}
