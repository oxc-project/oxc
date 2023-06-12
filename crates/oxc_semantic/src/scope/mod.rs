//! ECMAScript Scope Tree
//! See [Scope Analysis](https://tc39.es/ecma262/#sec-syntax-directed-operations-scope-analysis)
//! Code Adapted from [acorn](https://github.com/acornjs/acorn/blob/master/acorn/src/scope.js)

mod builder;
mod id;
mod tree;

use oxc_span::Atom;
pub use oxc_syntax::scope::ScopeFlags;
use rustc_hash::FxHashMap;

pub use self::{builder::ScopeBuilder, id::ScopeId, tree::ScopeTree};
use crate::symbol::{Reference, SymbolId};

#[derive(Debug, Clone)]
pub struct Scope {
    /// [Strict Mode Code](https://tc39.es/ecma262/#sec-strict-mode-code)
    /// [Use Strict Directive Prologue](https://tc39.es/ecma262/#sec-directive-prologues-and-the-use-strict-directive)
    pub(crate) strict_mode: bool,

    pub flags: ScopeFlags,

    /// Variables declared in this scope
    pub variables: FxHashMap<Atom, SymbolId>,

    /// Unsolved references in this scope
    pub unresolved_references: FxHashMap<Atom, Vec<Reference>>,
}

impl Scope {
    pub fn new(flags: ScopeFlags, strict_mode: bool) -> Self {
        Self {
            strict_mode,
            flags,
            variables: FxHashMap::default(),
            unresolved_references: FxHashMap::default(),
        }
    }

    pub fn get_variable_symbol_id(&self, name: &Atom) -> Option<SymbolId> {
        self.variables.get(name).copied()
    }

    pub fn strict_mode(&self) -> bool {
        self.strict_mode
    }

    pub fn is_top(&self) -> bool {
        self.flags.intersects(ScopeFlags::Top)
    }

    pub fn is_ts_module(&self) -> bool {
        self.flags.intersects(ScopeFlags::TsModuleBlock)
    }

    pub fn is_function(&self) -> bool {
        self.flags.intersects(ScopeFlags::Function)
    }

    pub fn is_static_block(&self) -> bool {
        self.flags.intersects(ScopeFlags::ClassStaticBlock)
    }

    pub fn is_constructor(&self) -> bool {
        self.flags.intersects(ScopeFlags::Constructor)
    }

    pub fn is_get_accessor(&self) -> bool {
        self.flags.intersects(ScopeFlags::GetAccessor)
    }

    pub fn is_set_accessor(&self) -> bool {
        self.flags.intersects(ScopeFlags::SetAccessor)
    }
}
