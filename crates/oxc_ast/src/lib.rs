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
pub mod module_record;
mod source_type;
mod span;
pub mod syntax_directed_operations;
mod trivia;
mod visit;
mod visit_mut;

pub use num_bigint::BigUint;

pub use crate::ast_builder::AstBuilder;
pub use crate::ast_kind::AstKind;
pub use crate::source_type::{Language, LanguageVariant, ModuleKind, SourceType, VALID_EXTENSIONS};
pub use crate::trivia::Trivias;
pub use crate::visit::Visit;
pub use crate::visit_mut::VisitMut;

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
    use oxc_index::static_assert_size;

    #[allow(clippy::wildcard_imports)]
    use crate::ast::*;

    static_assert_size!(Statement, 16);
    static_assert_size!(Expression, 16);
    static_assert_size!(Declaration, 16);
    static_assert_size!(BindingPatternKind, 16);
    static_assert_size!(ModuleDeclaration, 16);
    static_assert_size!(ClassElement, 16);
    static_assert_size!(ExportDefaultDeclarationKind, 16);
    static_assert_size!(AssignmentTargetPattern, 16);
    static_assert_size!(AssignmentTargetMaybeDefault, 24);
    static_assert_size!(AssignmentTargetProperty, 16);
    static_assert_size!(TSLiteral, 16);
    static_assert_size!(TSType, 16);
}
