//! Common code for JavaScript Syntax

pub mod operator;

#[derive(Debug, Clone, Copy, Hash)]
pub enum NumberBase {
    Float,
    Decimal,
    Binary,
    Octal,
    Hex,
}
