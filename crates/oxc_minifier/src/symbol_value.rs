use oxc_ecmascript::constant_evaluation::ConstantValue;
use oxc_index::IndexVec;
use oxc_syntax::symbol::SymbolId;

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

    /// Symbol is exported.
    pub exported: bool,

    pub read_references_count: u32,
    pub write_references_count: u32,

    /// Number of read references that are member write targets (e.g. `a` in `a.foo = 1`).
    /// These reads exist only to access the object for a property write, not to use the value.
    /// Always <= `read_references_count`.
    pub member_write_target_read_count: u32,

    /// The kind of fresh value the symbol holds (cannot alias another binding),
    /// or `FreshValueKind::None` when the value may alias. Set for function/class
    /// declarations and variable declarations initialized with
    /// object/array/function/class literals. See `FreshValueKind`.
    pub kind: FreshValueKind,

    /// A hoisted function called before this `let`/`const`/`class`
    /// declarator executes could observe the binding's temporal dead zone.
    /// Module sources: set from `BodyFrame::hoisted_fn_referenced` (see its
    /// doc for the reachability model, including declarators inside blocks).
    /// Script top-level declarators: any executable statement precedes the
    /// declarator, or the declarator sits inside a block — a script's
    /// top-level hoisted functions are global-object-reachable without a
    /// local reference. Script nested bodies are closed namespaces and use
    /// the same bit as modules. Consumers must not
    /// resolve `initialized_constant` for a read inside a hoisted function
    /// declaration below the binding's scope; the position test (and why
    /// arrows / function expressions are exempt) lives on
    /// `inside_hoisted_function_below`.
    pub lexical_unsafe_prelude: bool,

    /// The symbol is provably falsy in **boolean context** but not necessarily
    /// foldable in value context. Set for a write-once binding with a falsy
    /// constant initializer whose `initialized_constant` was withheld (a hoisted
    /// `var` whose declarative prelude isn't clean): a read before the
    /// initializer sees `undefined`, but `undefined` and the falsy init are
    /// indistinguishable inside `if (x)` / `x ? …` / `!x`, so such reads fold to
    /// `false` there. See `minimize_expression_in_boolean_context` / #14001.
    ///
    /// Never set for TDZ-lexicals: their value-context constant is never
    /// withheld (`falsy_init` requires `value_withheld`), which is why the
    /// boolean-context fold consuming this flag carries no TDZ gate. If
    /// withholding is ever extended to lexicals, that fold must adopt the
    /// `lexical_unsafe_prelude` gate.
    pub boolean_falsy: bool,
}

/// Per-symbol scratch store indexed by `SymbolId`.
///
/// Symbol IDs are dense `u32`s, so an indexed `IndexVec` lookup beats a
/// `FxHashMap` (hash + probe) on every hot path in the peephole pass.
///
/// Sized once from `Scoping::symbols_len()`; no minifier pass mints new
/// symbols, so `init_value` panics on out-of-range — that's the signal to
/// add a grow path.
#[derive(Debug)]
pub struct SymbolValues<'a> {
    values: IndexVec<SymbolId, Option<SymbolValue<'a>>>,
}

impl<'a> SymbolValues<'a> {
    pub(crate) fn new(len: usize) -> Self {
        let mut values = IndexVec::with_capacity(len);
        values.resize_with(len, || None);
        Self { values }
    }

    /// Reset slots to `None` without releasing the buffer, so the next peephole
    /// iteration's `init_value` stays on the indexed-write fast path.
    pub fn reset(&mut self) {
        for slot in &mut self.values {
            *slot = None;
        }
    }

    #[inline]
    pub fn init_value(&mut self, symbol_id: SymbolId, symbol_value: SymbolValue<'a>) {
        self.values[symbol_id] = Some(symbol_value);
    }

    #[inline]
    pub fn get_symbol_value(&self, symbol_id: SymbolId) -> Option<&SymbolValue<'a>> {
        self.values.get(symbol_id)?.as_ref()
    }
}
