#![allow(non_upper_case_globals)]

use bitflags::bitflags;
use oxc_ast::SourceType;
use oxc_index::{Idx, IndexVec};

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
        const Modifiers = Self::Constructor.bits() | Self::GetAccessor.bits() | Self::SetAccessor.bits();
    }
}

impl ScopeFlags {
    fn with_strict_mode(self, yes: bool) -> Self {
        if yes { self | Self::StrictMode } else { self }
    }
}

#[derive(Debug)]
pub struct Scope {
    parent_id: Option<ScopeId>,
    flags: ScopeFlags,
}

impl Scope {
    fn new(parent_id: Option<ScopeId>, flags: ScopeFlags) -> Self {
        Self { parent_id, flags }
    }

    pub fn parent_id(&self) -> Option<ScopeId> {
        self.parent_id
    }

    fn is_strict_mode(&self) -> bool {
        self.flags.contains(ScopeFlags::StrictMode)
    }

    pub fn is_function(&self) -> bool {
        self.flags.intersects(ScopeFlags::Function)
    }
}

#[derive(Debug)]
pub struct ScopeTree(IndexVec<ScopeId, Scope>);

impl ScopeTree {
    pub fn new() -> Self {
        Self(IndexVec::new())
    }

    pub fn get_scope(&self, scope_id: ScopeId) -> &Scope {
        &self.0[scope_id]
    }

    pub fn root_scope(&self) -> &Scope {
        self.get_scope(ScopeId::new(0))
    }

    pub fn add_scope(&mut self, scope: Scope) -> ScopeId {
        self.0.push(scope)
    }
}

pub struct ScopeTreeBuilder {
    scope_tree: ScopeTree,

    current_scope_id: ScopeId,
}

impl ScopeTreeBuilder {
    pub fn new(source_type: SourceType) -> Self {
        let mut scope_tree = ScopeTree::new();
        let scope_flags = ScopeFlags::Top
            .with_strict_mode(source_type.is_module() || source_type.always_strict());
        let root_scope = Scope::new(None, scope_flags);
        let current_scope_id = scope_tree.add_scope(root_scope);
        Self { scope_tree, current_scope_id }
    }

    pub fn build(self) -> ScopeTree {
        self.scope_tree
    }

    fn current_scope(&self) -> &Scope {
        self.scope_tree.get_scope(self.current_scope_id)
    }

    pub fn enter(&mut self, flags: ScopeFlags) {
        let mut flags = flags;
        // Inherit strict mode for functions
        // https://tc39.es/ecma262/#sec-strict-mode-code
        let mut strict_mode = self.scope_tree.root_scope().is_strict_mode();
        let parent_scope = self.current_scope();
        if !strict_mode && parent_scope.is_function() && parent_scope.is_strict_mode() {
            strict_mode = true;
        }

        // inherit flags for non-function scopes
        if !flags.contains(ScopeFlags::Function) {
            flags |= parent_scope.flags & ScopeFlags::Modifiers;
        };

        if strict_mode {
            flags |= ScopeFlags::StrictMode;
        }

        let scope = Scope::new(Some(self.current_scope_id), flags);
        self.current_scope_id = self.scope_tree.add_scope(scope);
    }

    pub fn leave(&mut self) {
        if let Some(parent_id) = self.current_scope().parent_id() {
            self.current_scope_id = parent_id;
        }
    }
}
