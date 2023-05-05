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
#[test]
fn no_bloat_enum_sizes() {
    use std::mem::size_of;

    #[allow(clippy::wildcard_imports)]
    use crate::ast::*;
    assert_eq!(size_of::<Statement>(), 16);
    assert_eq!(size_of::<Expression>(), 16);
    assert_eq!(size_of::<Declaration>(), 16);
    assert_eq!(size_of::<BindingPatternKind>(), 16);
    assert_eq!(size_of::<ModuleDeclaration>(), 16);
    assert_eq!(size_of::<ClassElement>(), 16);
    assert_eq!(size_of::<ExportDefaultDeclarationKind>(), 16);
    assert_eq!(size_of::<AssignmentTargetPattern>(), 16);
    assert_eq!(size_of::<AssignmentTargetMaybeDefault>(), 24);
    assert_eq!(size_of::<AssignmentTargetProperty>(), 16);
    assert_eq!(size_of::<TSLiteral>(), 16);
    assert_eq!(size_of::<TSType>(), 16);
}
