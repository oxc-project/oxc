//! # Oxc AST
//!
//! This is almost similar to [estree](https://github.com/estree/estree) except a few places:
//! * `Identifier` is replaced with explicit `BindingIdentifier`, `IdentifierReference`, `IdentifierName` per spec
//! * `AssignmentExpression`.`left` `Pattern` is replaced with `AssignmentTarget`
//!
//! ## Cargo Features
//! * `"serde"` enables support for serde serialization

#![feature(let_chains)]

#[cfg(feature = "serde")]
mod serialize;

pub mod ast;
mod ast_builder;
mod ast_kind;
mod span;
pub mod syntax_directed_operations;
mod trivia;
mod visit;
mod visit_mut;

pub use num_bigint::BigUint;

pub use crate::{
    ast_builder::AstBuilder, ast_kind::AstKind, trivia::Trivias, visit::Visit, visit_mut::VisitMut,
};

// After experimenting with two types of boxed enum variants:
//   1.
//   ```
//      enum Expression {
//          Variant(Box<Struct>)
//      }
//      struct Struct {
//          expression: Expression
//      }
//   ```
//   2.
//   ```
//      enum Expression {
//          Variant(Struct)
//      }
//      struct Struct {
//          expression: Box<Expression>
//      }
//   ```
//  I have concluded that the first options is more performant and more ergonomic to use.
//  The following test make sure all enum variants are boxed, resulting 16 bytes for each enum.
//  Read `https://nnethercote.github.io/perf-book/type-sizes.html` for more details.
#[cfg(target_pointer_width = "64")]
mod size_asserts {
    use oxc_index::assert_eq_size;

    assert_eq_size!(crate::ast::Statement, [u8; 16]);
    assert_eq_size!(crate::ast::Expression, [u8; 16]);
    assert_eq_size!(crate::ast::Declaration, [u8; 16]);
    assert_eq_size!(crate::ast::BindingPatternKind, [u8; 16]);
    assert_eq_size!(crate::ast::ModuleDeclaration, [u8; 16]);
    assert_eq_size!(crate::ast::ClassElement, [u8; 16]);
    assert_eq_size!(crate::ast::ExportDefaultDeclarationKind, [u8; 16]);
    assert_eq_size!(crate::ast::AssignmentTargetPattern, [u8; 16]);
    assert_eq_size!(crate::ast::AssignmentTargetMaybeDefault, [u8; 24]);
    assert_eq_size!(crate::ast::AssignmentTargetProperty, [u8; 16]);
    assert_eq_size!(crate::ast::TSLiteral, [u8; 16]);
    assert_eq_size!(crate::ast::TSType, [u8; 16]);
}
