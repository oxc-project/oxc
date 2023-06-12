//! Common code for JavaScript Syntax

pub mod identifier;
pub mod operator;
pub mod precedence;
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
