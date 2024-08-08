#![allow(clippy::wildcard_imports)]
// TODO: I'm not sure if it is a but or intentional but clippy needs this allowed both on this
// module and the generated one.
#![allow(clippy::self_named_module_files)]

//! # Oxc AST
//!
//! Abstract Syntax Tree nodes for Oxc. Supports both TypeScript and JavaScript.
//!
//! This is almost similar to [estree](https://github.com/estree/estree) except a few places:
//! * `Identifier` is replaced with explicit [`BindingIdentifier`], [`IdentifierReference`], [`IdentifierName`] per spec
//! * `AssignmentExpression`.`left` `Pattern` is replaced with [`AssignmentTarget`]
//!
//! ## Parsing
//!
//! You can obtain an AST by parsing source code with a [`Parser`] from [`oxc_parser`].
//!
//! ## Cargo Features
//! * `"serde"` enables support for serde serialization
//!
//! [`BindingIdentifier`]: ast::BindingIdentifier
//! [`IdentifierReference`]: ast::IdentifierReference
//! [`IdentifierName`]: ast::IdentifierName
//! [`AssignmentTarget`]: ast::AssignmentTarget
//! [`oxc_parser`]: <https://docs.rs/oxc_parser>
//! [`Parser`]: <https://docs.rs/oxc_parser/latest/oxc_parser/struct.Parser.html>

#[cfg(feature = "serialize")]
mod serialize;

pub mod ast;
mod ast_builder_impl;
mod ast_impl;
mod ast_kind_impl;
pub mod precedence;
pub mod syntax_directed_operations;
mod trivia;

mod generated {
    #[cfg(debug_assertions)]
    pub mod assert_layouts;
    pub mod ast_builder;
    pub mod ast_kind;
    pub mod derive_clone_in;
    pub mod derive_get_span;
    pub mod derive_get_span_mut;
    pub mod visit;
    pub mod visit_mut;
}

pub mod visit {
    pub use crate::generated::visit::*;
    pub use crate::generated::visit_mut::*;
}

pub use generated::ast_builder;
pub use generated::ast_kind;

pub use num_bigint::BigUint;

pub use crate::{
    ast_builder::AstBuilder,
    ast_kind::{AstKind, AstType},
    trivia::{Comment, CommentKind, SortedComments, Trivias},
    visit::{Visit, VisitMut},
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
#[test]
fn size_asserts() {
    use static_assertions::assert_eq_size;

    use crate::ast;

    assert_eq_size!(ast::Statement, [u8; 16]);
    assert_eq_size!(ast::Expression, [u8; 16]);
    assert_eq_size!(ast::Declaration, [u8; 16]);
    assert_eq_size!(ast::BindingPatternKind, [u8; 16]);
    assert_eq_size!(ast::ModuleDeclaration, [u8; 16]);
    assert_eq_size!(ast::ClassElement, [u8; 16]);
    assert_eq_size!(ast::ExportDefaultDeclarationKind, [u8; 16]);
    assert_eq_size!(ast::AssignmentTargetPattern, [u8; 16]);
    assert_eq_size!(ast::AssignmentTargetMaybeDefault, [u8; 16]);
    assert_eq_size!(ast::AssignmentTargetProperty, [u8; 16]);
    assert_eq_size!(ast::TSLiteral, [u8; 16]);
    assert_eq_size!(ast::TSType, [u8; 16]);
}

#[test]
fn lifetime_variance() {
    use crate::ast;

    fn _assert_program_variant_lifetime<'a: 'b, 'b>(program: ast::Program<'a>) -> ast::Program<'b> {
        program
    }
}
