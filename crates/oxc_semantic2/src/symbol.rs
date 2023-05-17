#![allow(non_upper_case_globals)]

use std::collections::BTreeMap;

use bitflags::bitflags;
use oxc_index::{Idx, IndexVec};
use oxc_span::{Atom, Span};

use crate::reference::Reference;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SymbolId(usize);

impl Idx for SymbolId {
    fn new(idx: usize) -> Self {
        Self(idx)
    }

    fn index(self) -> usize {
        self.0
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SymbolFlags: u16 {
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

/// Symbol Table
///
/// `SoA` (Struct of Arrays) for memory efficiency.
#[derive(Debug)]
pub struct SymbolTable {
    pub(crate) spans: IndexVec<SymbolId, Span>,
    pub(crate) names: IndexVec<SymbolId, Atom>,
    pub(crate) flags: IndexVec<SymbolId, SymbolFlags>,
    pub(crate) references: IndexVec<SymbolId, Vec<Reference>>,
    pub(crate) unresolved_references: BTreeMap<Span, Atom>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            spans: IndexVec::new(),
            names: IndexVec::new(),
            flags: IndexVec::new(),
            references: IndexVec::new(),
            unresolved_references: BTreeMap::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.spans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_name(&self, symbol_id: SymbolId) -> &Atom {
        &self.names[symbol_id]
    }

    pub fn get_flag(&self, symbol_id: SymbolId) -> SymbolFlags {
        self.flags[symbol_id]
    }

    pub fn add_symbol(&mut self, span: Span, name: Atom, flag: SymbolFlags) -> SymbolId {
        let _ = self.spans.push(span);
        let _ = self.names.push(name);
        let _ = self.references.push(vec![]);
        self.flags.push(flag)
    }

    pub fn add_reference(&mut self, reference: Reference) {
        self.references[reference.symbol_id()].push(reference);
    }

    pub fn add_unresolved_reference(&mut self, span: Span, name: Atom) {
        self.unresolved_references.insert(span, name);
    }
}
