// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]
//! AST Definitions
//!
//! # Enum inheritance
//!
//! Some enum AST types inherit variants from other enums using the `inherit_variants!` macro.
//!
//! "Inherit" means: If `enum Y` inherits the variants of `enum X`,
//! then all `X`'s variants are duplicated as variants of `Y`.
//!
//! This is mainly an explanation of the consumer-facing API. For further details on implementation,
//! see comments in `src/ast/macros.rs`.
//!
//! ## Defining enum inheritance
//!
//! Instead of nested enums:
//!
//! ```
//! pub enum Expression<'a> {
//!     BooleanLiteral(Box<'a, BooleanLiteral>),
//!     NullLiteral(Box<'a, NullLiteral>),
//!     // ...more variants
//!     MemberExpression(MemberExpression<'a>),
//! }
//!
//! pub enum MemberExpression<'a> {
//!     ComputedMemberExpression(Box<'a, ComputedMemberExpression<'a>>),
//!     StaticMemberExpression(Box<'a, StaticMemberExpression<'a>>),
//!     PrivateFieldExpression(Box<'a, PrivateFieldExpression<'a>>),
//! }
//! ```
//!
//! We define the types using `inherit_variants!` macro:
//!
//! ```
//! inherit_variants! {
//! #[repr(C, u8)]
//! pub enum Expression<'a> {
//!     BooleanLiteral(Box<'a, BooleanLiteral>) = 0,
//!     NullLiteral(Box<'a, NullLiteral>) = 1,
//!     // ...more variants
//!     @inherit MemberExpression,
//! }
//! }
//!
//! #[repr(C, u8)]
//! pub enum MemberExpression<'a> {
//!     ComputedMemberExpression(Box<'a, ComputedMemberExpression<'a>>) = 48,
//!     StaticMemberExpression(Box<'a, StaticMemberExpression<'a>>) = 49,
//!     PrivateFieldExpression(Box<'a, PrivateFieldExpression<'a>>) = 50,
//! }
//! ```
//!
//! `inherit_variants!` macro expands `Expression` to:
//!
//! ```
//! #[repr(C, u8)]
//! pub enum Expression<'a> {
//!     BooleanLiteral(Box<'a, BooleanLiteral>) = 0,
//!     NullLiteral(Box<'a, NullLiteral>) = 1,
//!     // ...more variants
//!
//!     // Inherited from `MemberExpression`
//!     ComputedMemberExpression(Box<'a, ComputedMemberExpression<'a>>) = 48,
//!     StaticMemberExpression(Box<'a, StaticMemberExpression<'a>>) = 49,
//!     PrivateFieldExpression(Box<'a, PrivateFieldExpression<'a>>) = 50,
//! }
//!
//! shared_enum_variants!(
//!     Expression, MemberExpression,
//!     is_member_expression,
//!     into_member_expression,
//!     as_member_expression, as_member_expression_mut,
//!     to_member_expression, to_member_expression_mut,
//!     [ComputedMemberExpression, StaticMemberExpression, PrivateFieldExpression]
//! )
//! ```
//!
//! See `src/ast/macros.rs` for what `shared_enum_variants!` macro expands to.
//! It provides the APIs listed below.
//!
//! ## Using inherited variants
//!
//! #### Creation
//!
//! ```
//! // Old
//! let expr = Expression::MemberExpression(
//!   MemberExpression::ComputedMemberExpression(computed_member_expr)
//! );
//!
//! // New
//! let expr = Expression::ComputedMemberExpression(computed_member_expr);
//! ```
//!
//! #### Conversion
//!
//! ```
//! // Old
//! let expr = Expression::MemberExpression(member_expr);
//!
//! // New
//! let expr = Expression::from(member_expr);
//! ```
//!
//! ```
//! // Old
//! let maybe_member_expr = match expr {
//!     Expression::MemberExpression(member_expr) => Some(member_expr),
//!     _ => None,
//! };
//!
//! // New
//! let maybe_member_expr = MemberExpression::try_from(expr).ok();
//! ```
//!
//! #### Testing
//!
//! ```
//! // Old
//! if matches!(expr, Expression::MemberExpression(_)) { }
//!
//! // New
//! if expr.is_member_expression() { }
//! // or
//! if matches!(expr, match_member_expression!(Expression)) { }
//! ```
//!
//! #### Branching
//!
//! ```
//! // Old
//! if let Expression::MemberExpression(member_expr) = &expr { }
//!
//! // New
//! if let Some(member_expr) = expr.as_member_expression() { }
//! ```
//!
//! #### Matching
//!
//! ```
//! // Old
//! match get_expression() {
//!     Expression::MemberExpression(member_expr) => visitor.visit(member_expr),
//! }
//!
//! // New (exhaustive match)
//! match get_expression() {
//!     expr @ match_member_expression!(Expression) => visitor.visit(expr.to_member_expression()),
//! }
//!
//! // New (alternative)
//! match get_expression() {
//!     expr if expr.is_member_expression() => visitor.visit(expr.to_member_expression()),
//! }
//! ```
//!
//! ## Why `#[repr(C, u8)]` on enums?
//!
//! `#[repr(C, u8)]` allows us to define the discriminants for variants in both the "inherited"
//! and "inheritee" enums.
//!
//! The discriminants and "payloads" match between the 2 types for the inherited variants.
//! Therefore `MemberExpression::ComputedMemberExpression` and `Expression::ComputedMemberExpression`
//! have identical representations in memory, and a `MemberExpression` can be converted to an
//! `Expression` with a zero-cost transmute.
//!
//! The APIs listed above use this property.
//!
//! It is **essential** that the discriminants and "payload" types match between the "inherited"
//! and "inheritee" types, or using the APIs below would be instant UB.
//! The `shared_enum_variants!` macro generates const assertions to ensure
//! these invariants are upheld, and it will be caught at compile time if they don't.
//!
//! If you are seeing compile-time errors in `src/ast/macros.rs`, this will be the cause.

pub(crate) mod comment;
pub(crate) mod js;
pub(crate) mod jsx;
pub(crate) mod literal;
pub(crate) mod macros;
pub(crate) mod ts;

use macros::inherit_variants;
// Re-export AST types from other crates
pub use oxc_span::{Atom, Language, LanguageVariant, ModuleKind, SourceType, Span};
pub use oxc_syntax::{
    number::{BigintBase, NumberBase},
    operator::{
        AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
    },
};

pub use self::{comment::*, js::*, jsx::*, literal::*, ts::*};
