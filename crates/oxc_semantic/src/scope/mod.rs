//! ECMAScript Scope Tree
//! See [Scope Analysis](https://tc39.es/ecma262/#sec-syntax-directed-operations-scope-analysis)
//! Code Adapted from [acorn](https://github.com/acornjs/acorn/blob/master/acorn/src/scope.js)
#![allow(non_upper_case_globals)]

mod builder;
mod id;
mod tree;

use bitflags::bitflags;
use oxc_span::Atom;
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

bitflags! {
    #[derive(Default)]
    pub struct ScopeFlags: u16 {
        const Top              = 1 << 0;
        const Function         = 1 << 1;
        const Arrow            = 1 << 2;
        const ClassStaticBlock = 1 << 4;
        const TsModuleBlock    = 1 << 5; // `declare namespace`
        const Constructor      = 1 << 6;
        const GetAccessor      = 1 << 7;
        const SetAccessor      = 1 << 8;
        const VAR = Self::Top.bits | Self::Function.bits | Self::ClassStaticBlock.bits | Self::TsModuleBlock.bits;
        const MODIFIERS = Self::Constructor.bits | Self::GetAccessor.bits | Self::SetAccessor.bits;
    }
}

impl Scope {
    #[must_use]
    pub fn new(flags: ScopeFlags, strict_mode: bool) -> Self {
        Self {
            strict_mode,
            flags,
            variables: FxHashMap::default(),
            unresolved_references: FxHashMap::default(),
        }
    }

    #[must_use]
    pub fn get_variable_symbol_id(&self, name: &Atom) -> Option<SymbolId> {
        self.variables.get(name).copied()
    }

    #[must_use]
    pub fn strict_mode(&self) -> bool {
        self.strict_mode
    }

    #[must_use]
    pub fn is_top(&self) -> bool {
        self.flags.intersects(ScopeFlags::Top)
    }

    #[must_use]
    pub fn is_ts_module(&self) -> bool {
        self.flags.intersects(ScopeFlags::TsModuleBlock)
    }

    #[must_use]
    pub fn is_function(&self) -> bool {
        self.flags.intersects(ScopeFlags::Function)
    }

    #[must_use]
    pub fn is_static_block(&self) -> bool {
        self.flags.intersects(ScopeFlags::ClassStaticBlock)
    }

    #[must_use]
    pub fn is_constructor(&self) -> bool {
        self.flags.intersects(ScopeFlags::Constructor)
    }

    #[must_use]
    pub fn is_get_accessor(&self) -> bool {
        self.flags.intersects(ScopeFlags::GetAccessor)
    }

    #[must_use]
    pub fn is_set_accessor(&self) -> bool {
        self.flags.intersects(ScopeFlags::SetAccessor)
    }
}
