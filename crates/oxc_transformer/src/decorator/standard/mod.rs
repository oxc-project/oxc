//! Lowering of standard (TC39 2023-11) decorators.
//!
//! Based on esbuild's `lowerClass` (`internal/js_parser/js_parser_lower_class.go`).
//!
//! Decorated classes are lowered entirely: all fields, private names and static blocks, the same
//! as esbuild's `computeClassLoweringInfo` does when any element is decorated. Decorator calls are
//! emitted after the class using the `@oxc-project/runtime` helpers adapted from esbuild's runtime
//! (`npm/runtime/scripts/helpers/`):
//!
//! ```js
//! _init = decoratorStart(base);
//! decorateElement(_init, flags, key, _x_dec, Class);      // per element, spec order
//! Class = decorateElement(_init, 0, "Class", _Class_decorators, Class);
//! runInitializers(_init, 3, Class);                       // static method extra initializers
//! /* static fields and blocks */
//! runInitializers(_init, 1, Class);                       // class extra initializers
//! ```
//!
//! See [`run_initializers`] for the flags passed to `runInitializers`.
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
//! protocol differs from ours. Local output fixtures cover Oxc's lowering choices.
//!
//! ## Class replacement
//!
//! Oxc follows `ClassDefinitionEvaluation` in the draft specification of the decorators proposal
//! ([tc39/ecma262#2417](https://github.com/tc39/ecma262/pull/2417), revision of 2024-02-02).
//! The proposal is at Stage 2.7 and Test262 has no semantic tests for it yet, so this text may
//! still change. The draft's known spec bugs
//! (tc39/proposal-decorators#568, #569, #572) do not affect the steps below; Oxc already uses the
//! initializer order that #569 asks for.
//!
//! The steps below are numbered as in that revision. `F` is the original constructor and `newF`
//! is the value returned by the last class decorator.
//!
//! 1. All elements are evaluated against `F` (`ClassElementEvaluation`, the "for each ClassElement"
//!    loop). Every method, accessor, field initializer and static block therefore has `F`, or
//!    `F.prototype`, as its `[[HomeObject]]`. `super` inside a static element always resolves
//!    through the *original* class's prototype chain, even after replacement. Oxc saves `F` in
//!    `_originalClass` for lowered `super` accesses.
//! 2. Element decorators run, then `InitializePrivateMethods(F, staticElements)` installs static
//!    private methods, getters, setters and the private `get`/`set` pair of `static accessor #x`
//!    on `F`. This happens *before* class decorators run, so they are never present on `newF`.
//!    `newF.#m()`, `#m in newF` and `newF.#a` throw or are `false`. Oxc adds `F` to the static
//!    brand before applying class decorators.
//! 3. `ApplyDecoratorsToClassDefinition` returns `newF`; `F` is set to `newF` and the inner class
//!    binding is initialized with it ("Set F to newF.\[\[Value\]\]", then
//!    `classEnv.InitializeBinding`).
//! 4. Static method extra initializers are called with `this` = `newF`.
//! 5. Static fields and the storage of static auto-accessors are defined on `newF`
//!    (`InitializeFieldOrAccessor(F, ...)` after step 3), and their initializers and static blocks
//!    run with `this` = `newF`. Static private *fields* are therefore present on `newF`, unlike
//!    static private methods.
//! 6. Class extra initializers run with `this` = `newF`.
//!
//! ### Comparison with Babel and esbuild
//!
//! Without semantic Test262 tests, each implementation's behaviour follows from how it lowers the
//! class. Babel moves static elements into a wrapper class whose `super(_Class)` call returns the
//! replacement, so its static elements are installed on, and resolve `super` through, the
//! replacement. esbuild applies decorators in a flat sequence after the class and adds the static
//! brand after replacing the class binding. For static elements, observed with Babel's `2023-11`
//! version:
//!
//! | | field `this` | `super` | method extra initializer `this` | `newF.#m()` |
//! |---|---|---|---|---|
//! | Draft, Oxc | `newF` | `F`'s parent | `newF` | throws |
//! | Babel | `newF` | `newF`'s parent | `F` | works |
//! | esbuild | `F` or `newF` | varies by case | `newF` | works |
//!
//! - Field `this`: esbuild lowers static fields only when an element is decorated. With only a
//!   class decorator, `static x = this` stays in the class body and sees `F`, and the field is
//!   defined on `F`.
//! - `super`: the draft is the only reading that works for the common
//!   `(C) => class extends C {}` replacement. Resolving through `newF` finds `C`'s own static
//!   methods instead of its parent's (Babel's `static x = super.who()` returns `C.who()`).
//! - Static method extra initializers: the draft, esbuild and Oxc agree on `newF`.
//! - Static private methods are the open question. The draft installs them on `F` but static
//!   private fields on `newF`, so with a replacing class decorator `A.#m()` throws inside the
//!   class body while `A.#field` works. Babel and esbuild install them on the replacement only.
//!   No spec issue tracks this yet. Oxc keeps the draft: adding `newF` to the static brand as
//!   well would match neither (`#m in F` stays true), and moving the brand to `newF` alone
//!   would break static private calls from element decorators, which run before replacement.
//!
//! Babel exec fixtures that depend on Babel's choices (`2023-11-classes*/replacement-static-*`)
//! are expected to fail in `babel_exec.snap.md`. `oxc/replacement-super-home-object` and
//! `oxc/replacement-static-spec-model` pin the behaviour above.
//!
//! `StandardDecoratorState` is split by concern: `element` collects decorators, `accessor`
//! lowers `accessor` properties, `name` handles named evaluation, and `sequence` emits the
//! decorator calls. Class-properties drives them in evaluation order; see
//! `es2022/class_properties/decorators.rs`.

use rustc_hash::FxHashMap;

use oxc_allocator::Address;
use oxc_ast::ast::*;
use oxc_span::Span;
use oxc_traverse::BoundIdentifier;

use crate::decorator::DecoratorMode;
pub use crate::decorator::accessor::AccessorKeys;

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
/// `runInitializers(array, flags, self, value)` protocol.
///
/// `decoratorStart` (`npm/runtime/scripts/helpers/decoratorStart.js`) creates the context array:
/// slot 0 holds class extra initializers, 1 static method extra initializers, 2 instance method
/// extra initializers and 3 the metadata object. From slot 4, each decorated field or accessor has
/// an `[initializers, extra initializers]` pair of slots.
///
/// `runInitializers` (`npm/runtime/scripts/helpers/runInitializers.js`) reads slot `flags >> 1`.
/// If `flags & 1`, it calls extra initializers for their side effects; otherwise it threads the
/// value through the initializers.
pub mod run_initializers {
    const CLASS_EXTRAS_SLOT: u32 = 0;
    const STATIC_METHOD_EXTRAS_SLOT: u32 = 1;
    const INSTANCE_METHOD_EXTRAS_SLOT: u32 = 2;
    const FIRST_ELEMENT_SLOT: u32 = 4;

    const fn extras(slot: u32) -> u32 {
        (slot << 1) | 1
    }

    /// Flags for class extra initializers.
    pub const CLASS_EXTRAS: u32 = extras(CLASS_EXTRAS_SLOT);
    /// Flags for static method, getter and setter extra initializers.
    pub const STATIC_METHOD_EXTRAS: u32 = extras(STATIC_METHOD_EXTRAS_SLOT);
    /// Flags for instance method, getter and setter extra initializers.
    pub const INSTANCE_METHOD_EXTRAS: u32 = extras(INSTANCE_METHOD_EXTRAS_SLOT);

    /// Flags for the initializers of the decorated field or accessor at `index`.
    pub const fn element_initializers(index: u32) -> u32 {
        (FIRST_ELEMENT_SLOT + 2 * index) << 1
    }

    /// Flags for the extra initializers of the decorated field or accessor at `index`.
    pub const fn element_extras(index: u32) -> u32 {
        extras(FIRST_ELEMENT_SLOT + 2 * index + 1)
    }
}

/// `decorateElement` flags (`npm/runtime/scripts/helpers/decorateElement.js`): the low 3 bits
/// hold the [`DecoratorElementKind`], `0` for the class itself.
const CLASS_KIND: u32 = 0;
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
    /// loops. Only unbraced loop bodies are cached, and each entry is removed when traversal
    /// visits that body, so the map is empty once the program has been traversed.
    loop_classes: FxHashMap<Address, bool>,
    /// Names of anonymous class expressions in instance field initializers, for named evaluation.
    named_classes: FxHashMap<Address, Expression<'a>>,
    /// Decorated class declaration that just exited, to convert to `let _Foo = class Foo {}` when
    /// the statement containing it exits, which is the next statement to exit.
    pending_declaration_rewrite: Option<DeclarationRewrite<'a>>,
    /// Scratch state reset before emitting each decorated class.
    exit: DecoratorExitState<'a>,
}

/// Scratch state used after all nested classes have finished their traversal.
#[derive(Default)]
struct DecoratorExitState<'a> {
    /// `decorateElement` calls, in evaluation order.
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

/// What the entry phase recorded about the elements of one class, keyed by element address.
///
/// Filled before the class is pushed onto the class stack (auto-accessors are lowered first), and
/// moved into [`ClassDecoratorState`] when it is created.
#[derive(Default)]
pub struct ClassDecoratorTables<'a> {
    elements: FxHashMap<Address, ElementInfo<'a>>,
}

#[derive(Default)]
struct ElementInfo<'a> {
    /// Element generated for an `accessor`.
    accessor: Option<AccessorPart>,
    /// Initializer slot index of a decorated field or decorated `accessor` storage field.
    slot: Option<u32>,
    /// Key of a decorated instance field, captured before it is moved into the constructor.
    instance_key: Option<Expression<'a>>,
    /// A method which may collide, kept on its home object under a distinct symbol.
    public_key: Option<PublicMethodKey<'a>>,
}

/// Part of a lowered `accessor`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum AccessorPart {
    /// Storage field of a decorated `accessor`.
    DecoratedStorage,
    Getter,
    /// Setter, which shares its getter's decorators and descriptor.
    Setter {
        getter: Address,
    },
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
    /// Class name symbol referenced by the class heritage, which must see the decorated class.
    pub(crate) heritage_class_symbol: Option<oxc_syntax::symbol::SymbolId>,
    /// Element tables of this class.
    pub(crate) tables: ClassDecoratorTables<'a>,
    /// `(inner binding, outer binding)` of a declaration whose name is referenced in its body.
    pub(crate) declaration_rewrite: Option<(BoundIdentifier<'a>, BoundIdentifier<'a>)>,
}

/// A decorated class declaration to convert to a `let` declaration of a class expression.
struct DeclarationRewrite<'a> {
    class: Address,
    inner: BoundIdentifier<'a>,
    outer: BoundIdentifier<'a>,
}

/// Bucket indexes for `decorateElement` calls, in the order they are evaluated.
#[derive(Clone, Copy)]
#[expect(clippy::enum_variant_names)]
enum DecoratorBucket {
    StaticNonField = 0,
    InstanceNonField = 1,
    StaticField = 2,
    InstanceField = 3,
}

/// One `decorateElement` call, described by class-properties when it lowers the element.
pub struct DecoratedElement<'b, 'a> {
    pub kind: DecoratorElementKind,
    pub is_static: bool,
    /// Element name passed to the helper.
    pub key: Expression<'a>,
    /// `_x_dec` array of the element's decorators. `None` for an undecorated method captured
    /// only so its descriptor survives a colliding decorated element.
    pub decorators: Option<&'b BoundIdentifier<'a>>,
    /// Index of the captured descriptor of a public method that may collide.
    pub public_descriptor: Option<usize>,
    /// Source span of the element, so the call maps to its decorators.
    pub span: Span,
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
        !self.named_classes.is_empty()
            || self.pending_declaration_rewrite.is_some()
            || !self.loop_classes.is_empty()
    }
}
