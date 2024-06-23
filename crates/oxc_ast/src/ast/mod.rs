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

mod js;
mod jsx;
mod literal;
mod macros;
mod ts;

use macros::inherit_variants;

pub use self::{js::*, jsx::*, literal::*, ts::*};

#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

use oxc_allocator::Vec;
use oxc_span::Span;

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct Modifier {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub kind: ModifierKind,
}

#[derive(Debug, Default, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(transparent))]
pub struct Modifiers<'a>(Option<Vec<'a, Modifier>>);

impl<'a> Modifiers<'a> {
    pub fn new(modifiers: Vec<'a, Modifier>) -> Self {
        Self(Some(modifiers))
    }

    pub fn empty() -> Self {
        Self(None)
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    pub fn contains(&self, target: ModifierKind) -> bool {
        self.0
            .as_ref()
            .map_or(false, |modifiers| modifiers.iter().any(|modifier| modifier.kind == target))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Modifier> + '_ {
        self.0.as_ref().into_iter().flat_map(|modifiers| modifiers.iter())
    }

    /// Find a modifier by kind
    pub fn find(&self, kind: ModifierKind) -> Option<&Modifier> {
        self.find_where(|modifier| modifier.kind == kind)
    }

    pub fn find_where<F>(&self, f: F) -> Option<&Modifier>
    where
        F: Fn(&Modifier) -> bool,
    {
        self.0.as_ref().and_then(|modifiers| modifiers.iter().find(|modifier| f(modifier)))
    }

    pub fn is_contains_declare(&self) -> bool {
        self.contains(ModifierKind::Declare)
    }

    pub fn is_contains_abstract(&self) -> bool {
        self.contains(ModifierKind::Abstract)
    }

    pub fn remove_type_modifiers(&mut self) {
        if let Some(list) = &mut self.0 {
            list.retain(|m| !m.kind.is_typescript_syntax());
        }
    }

    pub fn add_modifier(&mut self, modifier: Modifier) {
        if let Some(list) = self.0.as_mut() {
            list.push(modifier);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
pub enum ModifierKind {
    Abstract,
    Accessor,
    Async,
    Const,
    Declare,
    Default,
    Export,
    In,
    Public,
    Private,
    Protected,
    Readonly,
    Static,
    Out,
    Override,
}

impl ModifierKind {
    pub fn is_typescript_syntax(&self) -> bool {
        !matches!(self, Self::Async | Self::Default | Self::Export | Self::Static)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Abstract => "abstract",
            Self::Accessor => "accessor",
            Self::Async => "async",
            Self::Const => "const",
            Self::Declare => "declare",
            Self::Default => "default",
            Self::Export => "export",
            Self::In => "in",
            Self::Public => "public",
            Self::Private => "private",
            Self::Protected => "protected",
            Self::Readonly => "readonly",
            Self::Static => "static",
            Self::Out => "out",
            Self::Override => "override",
        }
    }
}
