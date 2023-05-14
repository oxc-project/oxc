#![allow(non_upper_case_globals)]

use bitflags::bitflags;
use oxc_index::{Idx, IndexVec};
use oxc_span::{Atom, Span};

use crate::reference::Reference;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
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
    names: IndexVec<SymbolId, Atom>,
    spans: IndexVec<SymbolId, Span>,
    flags: IndexVec<SymbolId, SymbolFlags>,
    references: IndexVec<SymbolId, Vec<Reference>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            names: IndexVec::new(),
            spans: IndexVec::new(),
            flags: IndexVec::new(),
            references: IndexVec::new(),
        }
    }

    pub fn get_name(&self, symbol_id: SymbolId) -> &Atom {
        &self.names[symbol_id]
    }

    pub fn add_symbol(&mut self, name: Atom, span: Span, flag: SymbolFlags) -> SymbolId {
        let _ = self.names.push(name);
        let _ = self.spans.push(span);
        let _ = self.references.push(vec![]);
        self.flags.push(flag)
    }

    pub fn add_reference(&mut self, reference: Reference) {
        self.references[reference.symbol_id()].push(reference);
    }

    pub fn mangle(&mut self) {
        let mut offset = 0;
        for (symbol_id, flag) in self.flags.iter_enumerated() {
            if flag.is_variable() {
                let name = loop {
                    let name = Atom::base54(symbol_id.index() + offset);
                    if Self::is_keyword(&name) {
                        offset += 1;
                    } else {
                        break name;
                    }
                };
                self.names[symbol_id] = name;
            }
        }
    }

    #[rustfmt::skip]
    fn is_keyword(s: &str) -> bool {
        let len = s.len();
        if len == 1 {
            return false;
        }
        matches!(s, "as" | "do" | "if" | "in" | "is" | "of" | "any" | "for" | "get"
                | "let" | "new" | "out" | "set" | "try" | "var" | "case" | "else"
                | "enum" | "from" | "meta" | "null" | "this" | "true" | "type"
                | "void" | "with")
    }
}
