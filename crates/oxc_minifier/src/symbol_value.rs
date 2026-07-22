use oxc_ecmascript::constant_evaluation::ConstantValue;
use oxc_syntax::reference::ReferenceFlags;

/// The kind of fresh value a binding was initialized with, or `None` when the
/// value may alias another binding (or is untracked).
///
/// A fresh value cannot alias another binding, so a property write to a
/// provably-unused fresh local is normally unobservable and droppable. But the
/// *kind* matters: writing certain keys on a function/class/array throws a
/// strict-mode `TypeError` (non-writable own properties, the `caller` /
/// `arguments` poison) or has an observable value-domain effect (`Array`
/// `length`). The kind drives the key denylist in
/// `remove_unused_member_assignment` that keeps those writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreshValueKind {
    /// Not a fresh value (may alias another binding), or not tracked.
    None,
    /// Function expression, arrow function, or function declaration
    /// (including generator and async forms).
    Function,
    /// Class expression or class declaration.
    Class,
    /// Object literal.
    Object,
    /// Array literal.
    Array,
}

/// Cached counts for the resolved references to a symbol.
///
/// Keep queries here so consumers do not have to coordinate related counters
/// or repeat the invariant that member-write target reads are a subset of all
/// reads.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReferenceCounts {
    reads: u32,
    writes: u32,
    member_write_target_reads: u32,
}

impl ReferenceCounts {
    #[inline]
    pub fn record(&mut self, flags: ReferenceFlags) {
        debug_assert!(!flags.is_member_write_target() || flags.is_read());
        if flags.is_read() {
            self.reads += 1;
        }
        if flags.is_write() {
            self.writes += 1;
        }
        if flags.is_member_write_target() {
            self.member_write_target_reads += 1;
        }
    }

    #[inline]
    pub fn has_reads(self) -> bool {
        self.reads > 0
    }

    #[inline]
    pub fn has_writes(self) -> bool {
        self.writes > 0
    }

    #[inline]
    pub fn has_single_read(self) -> bool {
        self.reads == 1
    }

    #[inline]
    pub fn has_multiple_reads(self) -> bool {
        self.reads > 1
    }

    #[inline]
    pub fn has_only_member_write_target_reads(self) -> bool {
        self.writes == 0 && self.reads == self.member_write_target_reads
    }
}

#[derive(Debug)]
pub struct SymbolValue<'a> {
    /// Initialized constant value evaluated from expressions.
    /// `None` when the value is not a constant evaluated value.
    pub initialized_constant: Option<ConstantValue<'a>>,

    /// The `initialized_constant` is the implicit `undefined` of a declaration
    /// with no initializer (`let x;`), not an evaluated initializer. Textually
    /// inlining such a read prints `void 0` — longer than a mangled identifier
    /// read — and there is no initializer whose elimination pays for it, so
    /// `inline_identifier_reference` skips it (rolldown#10174). Constant-driven
    /// folds (`if (x)`, `x === void 0`, `return x`) are unaffected: they
    /// resolve the value through `initialized_constant`.
    pub implicit_undefined: bool,

    pub references: ReferenceCounts,

    /// The kind of fresh value the symbol holds (cannot alias another binding),
    /// or `FreshValueKind::None` when the value may alias. Set for function/class
    /// declarations and variable declarations initialized with
    /// object/array/function/class literals. See `FreshValueKind`.
    pub kind: FreshValueKind,

    /// The symbol is provably falsy in **boolean context** but not necessarily
    /// foldable in value context. Set for a write-once binding with a falsy
    /// constant initializer whose `initialized_constant` was withheld (a hoisted
    /// `var` whose declarative prelude isn't clean): a read before the
    /// initializer sees `undefined`, but `undefined` and the falsy init are
    /// indistinguishable inside `if (x)` / `x ? …` / `!x`, so such reads fold to
    /// `false` there. See `minimize_expression_in_boolean_context` / #14001.
    pub boolean_falsy: bool,
}
