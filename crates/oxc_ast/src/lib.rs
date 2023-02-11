//! AST
//! NOTE: This is not compatible with estree.

#![feature(let_chains)]
#![feature(is_some_and)]

mod serialize;

pub mod ast;
pub mod ast_builder;
pub mod context;
pub mod node;
pub mod source_type;

pub use num_bigint::BigUint;

pub use self::ast::*;
pub use self::ast_builder::*;
pub use self::node::*;
pub use self::source_type::*;

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
