#![allow(non_upper_case_globals)]

use std::collections::hash_map::Entry;

use bitflags::bitflags;
use oxc_index::{Idx, IndexVec};
use oxc_span::Atom;
use rustc_hash::FxHashMap;

use crate::symbol::SymbolId;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ScopeId(usize);

impl Idx for ScopeId {
    #[inline]
    fn new(idx: usize) -> Self {
        Self(idx)
    }

    fn index(self) -> usize {
        self.0
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ScopeFlags: u8 {
        const StrictMode       = 1 << 0;
        const Top              = 1 << 1;
        const Function         = 1 << 2;
        const Arrow            = 1 << 3;
        const ClassStaticBlock = 1 << 4;
        const Constructor      = 1 << 5;
        const GetAccessor      = 1 << 6;
        const SetAccessor      = 1 << 7;
        const Var = Self::Top.bits() | Self::Function.bits() | Self::ClassStaticBlock.bits();
        const Modifiers = Self::Constructor.bits() | Self::GetAccessor.bits() | Self::SetAccessor.bits();
    }
}

impl ScopeFlags {
    pub(crate) fn with_strict_mode(self, yes: bool) -> Self {
        if yes { self | Self::StrictMode } else { self }
    }
}

#[derive(Debug)]
pub struct Scope {
    parent_id: Option<ScopeId>,
    flags: ScopeFlags,
    bindings: FxHashMap<Atom, SymbolId>,
}

impl Scope {
    pub fn new(parent_id: Option<ScopeId>, flags: ScopeFlags) -> Self {
        Self { parent_id, flags, bindings: FxHashMap::default() }
    }

    pub fn flags(&self) -> ScopeFlags {
        self.flags
    }

    pub fn parent_id(&self) -> Option<ScopeId> {
        self.parent_id
    }

    pub fn is_strict_mode(&self) -> bool {
        self.flags.contains(ScopeFlags::StrictMode)
    }

    pub fn is_var_scope(&self) -> bool {
        self.flags.intersects(ScopeFlags::Var)
    }

    pub fn is_function(&self) -> bool {
        self.flags.intersects(ScopeFlags::Function)
    }

    pub fn get_binding(&self, name: &Atom) -> Option<SymbolId> {
        self.bindings.get(name).copied()
    }

    pub fn bindings_entry(&mut self, name: Atom) -> Entry<Atom, SymbolId> {
        self.bindings.entry(name)
    }

    pub fn add_symbol(&mut self, name: Atom, symbol_id: SymbolId) {
        self.bindings.insert(name, symbol_id);
    }
}

#[derive(Debug)]
pub struct ScopeTree(IndexVec<ScopeId, Scope>);

impl ScopeTree {
    pub fn new() -> Self {
        Self(IndexVec::new())
    }

    pub fn root_scope(&self) -> &Scope {
        self.get_scope(ScopeId::new(0))
    }

    pub fn get_scope(&self, scope_id: ScopeId) -> &Scope {
        &self.0[scope_id]
    }

    pub fn get_scope_mut(&mut self, scope_id: ScopeId) -> &mut Scope {
        &mut self.0[scope_id]
    }

    pub fn add_scope(&mut self, scope: Scope) -> ScopeId {
        self.0.push(scope)
    }

    pub fn ancestors(&self, scope_id: ScopeId) -> impl Iterator<Item = ScopeId> + '_ {
        std::iter::successors(Some(scope_id), |scope_id| self.get_scope(*scope_id).parent_id())
    }
}
