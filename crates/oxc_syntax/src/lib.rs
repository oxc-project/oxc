//! Common code for JavaScript Syntax

pub mod assumptions;
pub mod class;
pub mod identifier;
pub mod keyword;
pub mod module_record;
pub mod node;
pub mod operator;
pub mod precedence;
pub mod reference;
pub mod scope;
pub mod symbol;
pub mod xml_entities;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum NumberBase {
    Float,
    Decimal,
    Binary,
    Octal,
    Hex,
}

impl NumberBase {
    pub fn is_base_10(&self) -> bool {
        matches!(self, Self::Float | Self::Decimal)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BigintBase {
    Decimal,
    Binary,
    Octal,
    Hex,
}

impl BigintBase {
    pub fn is_base_10(&self) -> bool {
        self == &Self::Decimal
    }
}
