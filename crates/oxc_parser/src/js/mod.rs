//! JavaScript Parsing Functions

#![allow(clippy::missing_errors_doc)]

mod grammar;
pub mod list;

mod arrow;
mod binding;
mod class;
pub mod declaration;
mod expression;
pub mod function;
mod module;
mod object;
mod operator;
mod statement;

#[derive(Debug, Clone, Copy)]
pub enum Tristate {
    True,
    False,
    Maybe,
}
