//! AST
//! NOTE: This is not compatible with estree.

#![feature(let_chains)]
#![feature(is_some_and)]

mod serialize;

pub mod ast;
pub mod ast_builder;
pub mod ast_kind;
pub mod context;
pub mod source_type;
pub mod span;
pub mod syntax_directed_operations;
pub mod trivia;
pub mod visit;

pub use ast_kind::AstKind;
pub use num_bigint::BigUint;

pub use crate::ast_builder::*;
pub use crate::source_type::*;
pub use crate::span::*;
pub use crate::trivia::*;

pub type Atom = compact_str::CompactString;

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
#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
#[test]
fn no_bloat_enum_sizes() {
    use std::mem::size_of;

    #[allow(clippy::wildcard_imports)]
    use crate::ast::*;
    assert_eq!(size_of::<Statement>(), 16);
    assert_eq!(size_of::<Expression>(), 16);
    assert_eq!(size_of::<Declaration>(), 16);
    assert_eq!(size_of::<BindingPatternKind>(), 16);
    assert_eq!(size_of::<ModuleDeclarationKind>(), 16);
    assert_eq!(size_of::<ClassElement>(), 16);
    assert_eq!(size_of::<ExportDefaultDeclarationKind>(), 16);
    assert_eq!(size_of::<TSLiteral>(), 16);
    assert_eq!(size_of::<TSType>(), 16);
}
