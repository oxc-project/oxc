//! # Oxc AST (Abstract Syntax Tree) Nodes
//!
//! Supports JavaScript, TypeScript and JSX.
//!
//! ## Types
//!
//! AST types are similar to [estree] and [typescript-eslint]'s definition, with a few notable exceptions:
//!
//! * `Identifier` is replaced with explicit [`BindingIdentifier`], [`IdentifierReference`],
//!   [`IdentifierName`], per ECMAScript Specification.
//! * `AssignmentExpression`.`left` `Pattern` is replaced with [`AssignmentTarget`].
//! * `Literal` is replaced with [`BooleanLiteral`], [`NumericLiteral`], [`StringLiteral`] etc.
//!
//! Field order of types follows "Evaluation order" defined by [ECMAScript spec].
//! For TypeScript types, we follow how field order is defined in [tsc].
//!
//! Oxc's visitors ([`Visit`], [`VisitMut`], [`Traverse`]) visit AST node fields in same order
//! as they are defined in the types here.
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
//! [`BooleanLiteral`]: ast::BooleanLiteral
//! [`NumericLiteral`]: ast::NumericLiteral
//! [`StringLiteral`]: ast::StringLiteral
//! [`oxc_parser`]: <https://docs.rs/oxc_parser>
//! [`Parser`]: <https://docs.rs/oxc_parser/latest/oxc_parser/struct.Parser.html>
//! [estree]: <https://github.com/estree/estree>
//! [typescript-eslint]: <https://github.com/typescript-eslint/typescript-eslint/tree/v8.9.0/packages/ast-spec>
//! [ECMAScript spec]: <https://tc39.es/ecma262/>
//! [tsc]: <https://github.com/microsoft/TypeScript>
//! [`Traverse`]: <https://github.com/oxc-project/oxc/tree/main/crates/oxc_traverse>

// TODO: I'm not sure if it is a but or intentional but clippy needs this allowed both on this
// module and the generated one.
#![allow(clippy::self_named_module_files)]
#![warn(missing_docs)]

#[cfg(feature = "serialize")]
mod serialize;

pub mod ast;
mod ast_builder_impl;
mod ast_impl;
mod ast_kind_impl;
pub mod precedence;
mod trivia;
#[cfg(feature = "serialize")]
pub mod utf8_to_utf16;

mod generated {
    #![allow(missing_docs)]
    #[cfg(debug_assertions)]
    pub mod assert_layouts;
    pub mod ast_builder;
    pub mod ast_kind;
    pub mod derive_clone_in;
    pub mod derive_content_eq;
    #[cfg(feature = "serialize")]
    pub mod derive_estree;
    pub mod derive_get_address;
    pub mod derive_get_span;
    pub mod derive_get_span_mut;
    pub mod get_id;
    pub mod visit;
    pub mod visit_mut;
}

pub mod visit {
    #![allow(missing_docs)]
    pub use crate::generated::{visit::*, visit_mut::*};
}

pub use generated::{ast_builder, ast_kind};

pub use crate::{
    ast::comment::{Comment, CommentKind, CommentPosition},
    ast_builder::AstBuilder,
    ast_builder_impl::NONE,
    ast_kind::{AstKind, AstType},
    trivia::{comments_range, has_comments_between, CommentsRange},
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
    use std::mem::size_of;

    use crate::ast;

    assert_eq!(size_of::<ast::Statement>(), 16);
    assert_eq!(size_of::<ast::Expression>(), 16);
    assert_eq!(size_of::<ast::Declaration>(), 16);
    assert_eq!(size_of::<ast::BindingPatternKind>(), 16);
    assert_eq!(size_of::<ast::ModuleDeclaration>(), 16);
    assert_eq!(size_of::<ast::ClassElement>(), 16);
    assert_eq!(size_of::<ast::ExportDefaultDeclarationKind>(), 16);
    assert_eq!(size_of::<ast::AssignmentTargetPattern>(), 16);
    assert_eq!(size_of::<ast::AssignmentTargetMaybeDefault>(), 16);
    assert_eq!(size_of::<ast::AssignmentTargetProperty>(), 16);
    assert_eq!(size_of::<ast::TSLiteral>(), 16);
    assert_eq!(size_of::<ast::TSType>(), 16);
}

#[test]
fn lifetime_variance() {
    use crate::ast;

    fn _assert_program_variant_lifetime<'a: 'b, 'b>(program: ast::Program<'a>) -> ast::Program<'b> {
        program
    }
}
