//! Symbol and Symbol Table for tracking of semantics of variables
#![allow(non_upper_case_globals)]

mod builder;
mod id;
mod mangler;
mod reference;
mod table;

use bitflags::bitflags;
use oxc_span::{Atom, Span};

use self::reference::ResolvedReferenceId;
pub use self::{
    builder::SymbolTableBuilder,
    id::SymbolId,
    mangler::{Mangler, Slot},
    reference::{Reference, ReferenceFlag, ResolvedReference},
    table::SymbolTable,
};
use crate::node::AstNodeId;

#[derive(Debug)]
pub struct Symbol {
    id: SymbolId,
    /// Pointer to the AST Node where this symbol is declared
    declaration: AstNodeId,
    name: Atom,
    span: Span,
    flags: SymbolFlags,
    slot: Slot,
    /// Pointers to the AST Nodes that reference this symbol
    references: Vec<ResolvedReferenceId>,
}

#[cfg(target_pointer_width = "64")]
mod size_asserts {
    use oxc_index::static_assert_size;

    use super::Symbol;

    static_assert_size!(Symbol, 96);
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SymbolFlags: u16 {
        const None                    = 0;
        /// Variable (var) or parameter
        const FunctionScopedVariable  = 1 << 0;
        /// A block-scoped variable (let or const)
        const BlockScopedVariable     = 1 << 1;
        /// A const variable (const)
        const ConstVariable           = 1 << 2;
        /// Is this symbol inside an import declaration
        const Import                  = 1 << 3;
        /// Is this symbol inside an export declaration
        const Export                  = 1 << 4;
        const Class                   = 1 << 5;
        const CatchVariable           = 1 << 6; // try {} catch(catch_variable) {}

        const Variable = Self::FunctionScopedVariable.bits() | Self::BlockScopedVariable.bits();
        const Value = Self::Variable.bits() | Self::Class.bits();

        /// Variables can be redeclared, but can not redeclare a block-scoped declaration with the
        /// same name, or any other value that is not a variable, e.g. ValueModule or Class
        const FunctionScopedVariableExcludes = Self::Value.bits() - Self::FunctionScopedVariable.bits();

        /// Block-scoped declarations are not allowed to be re-declared
        /// they can not merge with anything in the value space
        const BlockScopedVariableExcludes = Self::Value.bits();

        const ClassExcludes = Self::Value.bits();
    }
}

impl SymbolFlags {
    pub fn is_variable(&self) -> bool {
        self.intersects(Self::Variable)
    }
}

impl Symbol {
    pub fn new(
        id: SymbolId,
        declaration: AstNodeId,
        name: Atom,
        span: Span,
        flags: SymbolFlags,
    ) -> Self {
        Self { id, declaration, name, span, flags, slot: Slot::default(), references: vec![] }
    }

    pub fn id(&self) -> SymbolId {
        self.id
    }

    pub fn name(&self) -> &Atom {
        &self.name
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn flags(&self) -> SymbolFlags {
        self.flags
    }

    pub fn slot(&self) -> Slot {
        self.slot
    }

    pub fn is_const(&self) -> bool {
        self.flags.contains(SymbolFlags::ConstVariable)
    }

    pub fn is_class(&self) -> bool {
        self.flags.contains(SymbolFlags::Class)
    }

    pub fn is_export(&self) -> bool {
        self.flags.contains(SymbolFlags::Export)
    }

    pub fn references(&self) -> &[ResolvedReferenceId] {
        &self.references
    }

    pub fn declaration(&self) -> AstNodeId {
        self.declaration
    }
}
