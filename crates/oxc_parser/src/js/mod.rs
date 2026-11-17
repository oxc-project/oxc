//! JavaScript Parsing Functions

mod grammar;

mod arrow;
mod binding;
mod class;
mod cover;
mod declaration;
mod expression;
mod function;
mod module;
mod object;
mod operator;
mod statement;

pub use cover::CoverElement;
pub use operator::kind_to_precedence;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Tristate {
    True,
    False,
    Maybe,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FunctionKind {
    Constructor,
    ClassMethod,
    ObjectMethod,
    Declaration,
    Expression,
    DefaultExport,
    TSDeclaration,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VariableDeclarationParent {
    For,
    Statement,
}
