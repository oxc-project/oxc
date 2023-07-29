//! Common code for JavaScript Syntax

pub mod identifier;
pub mod module_record;
pub mod operator;
pub mod precedence;
pub mod scope;
pub mod symbol;

pub use unicode_id_start;

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
