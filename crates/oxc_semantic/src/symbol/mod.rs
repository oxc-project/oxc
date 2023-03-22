//! Symbol and Symbol Table for tracking of semantics of variables
#![allow(non_upper_case_globals)]

mod id;
mod reference;
mod table;

use bitflags::bitflags;
use oxc_ast::{Atom, Span};

pub use self::{
    id::SymbolId,
    reference::{Reference, ReferenceFlag},
    table::SymbolTable,
};

#[derive(Debug)]
pub struct Symbol {
    id: SymbolId,
    name: Atom,
    span: Span,
    flags: SymbolFlags,
}

#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
#[test]
fn symbol_size() {
    use std::mem::size_of;
    assert_eq!(size_of::<Symbol>(), 48);
}

bitflags! {
    #[derive(Default)]
    pub struct SymbolFlags: u16 {
        const None                    = 0;
        /// Variable (var) or parameter
        const FunctionScopedVariable  = 1 << 0;
        /// A block-scoped variable (let or const)
        const BlockScopedVariable     = 1 << 1;
        /// A const variable (const)
        const ConstVariable           = 1 << 2;
        const Class                   = 1 << 5;
        const CatchVariable           = 1 << 6; // try {} catch(catch_variable) {}

        const Variable = Self::FunctionScopedVariable.bits | Self::BlockScopedVariable.bits;
        const Value = Self::Variable.bits | Self::Class.bits;

        /// Variables can be redeclared, but can not redeclare a block-scoped declaration with the
        /// same name, or any other value that is not a variable, e.g. ValueModule or Class
        const FunctionScopedVariableExcludes = Self::Value.bits - Self::FunctionScopedVariable.bits;

        /// Block-scoped declarations are not allowed to be re-declared
        /// they can not merge with anything in the value space
        const BlockScopedVariableExcludes = Self::Value.bits;

        const ClassExcludes = Self::Value.bits;
    }
}

impl Symbol {
    #[must_use]
    pub fn new(id: SymbolId, name: Atom, span: Span, flags: SymbolFlags) -> Self {
        Self { id, name, span, flags }
    }

    #[must_use]
    pub fn id(&self) -> SymbolId {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &Atom {
        &self.name
    }

    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }

    #[must_use]
    pub fn flags(&self) -> SymbolFlags {
        self.flags
    }
}
