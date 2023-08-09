use std::{
    cell::Cell,
    fmt,
    hash::{Hash, Hasher},
};

use bitflags::bitflags;
use num_bigint::BigInt;
use oxc_allocator::{Box, Vec};
use oxc_semantic::{ReferenceFlag, ReferenceId, SymbolId};
use oxc_span::{Atom, Span};
use oxc_syntax::{
    operator::{
        AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
    },
    NumberBase,
};
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::HirId;

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct Program<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub directives: Vec<'a, Directive>,
    pub hashbang: Option<Hashbang>,
    pub body: Vec<'a, Statement<'a>>,
}

impl<'a> Program<'a> {
    pub fn is_empty(&self) -> bool {
        self.body.is_empty() && self.directives.is_empty()
    }
}

/// Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum Expression<'a> {
    BooleanLiteral(Box<'a, BooleanLiteral>),
    NullLiteral(Box<'a, NullLiteral>),
    NumberLiteral(Box<'a, NumberLiteral<'a>>),
    BigintLiteral(Box<'a, BigintLiteral>),
    RegExpLiteral(Box<'a, RegExpLiteral>),
    StringLiteral(Box<'a, StringLiteral>),
    TemplateLiteral(Box<'a, TemplateLiteral<'a>>),

    Identifier(Box<'a, IdentifierReference>),

    MetaProperty(Box<'a, MetaProperty>),
    Super(Box<'a, Super>),

    ArrayExpression(Box<'a, ArrayExpression<'a>>),
    ArrowExpression(Box<'a, ArrowExpression<'a>>),
    AssignmentExpression(Box<'a, AssignmentExpression<'a>>),
    AwaitExpression(Box<'a, AwaitExpression<'a>>),
    BinaryExpression(Box<'a, BinaryExpression<'a>>),
    CallExpression(Box<'a, CallExpression<'a>>),
    ChainExpression(Box<'a, ChainExpression<'a>>),
    ClassExpression(Box<'a, Class<'a>>),
    ConditionalExpression(Box<'a, ConditionalExpression<'a>>),
    FunctionExpression(Box<'a, Function<'a>>),
    ImportExpression(Box<'a, ImportExpression<'a>>),
    LogicalExpression(Box<'a, LogicalExpression<'a>>),
    MemberExpression(Box<'a, MemberExpression<'a>>),
    NewExpression(Box<'a, NewExpression<'a>>),
    ObjectExpression(Box<'a, ObjectExpression<'a>>),
    SequenceExpression(Box<'a, SequenceExpression<'a>>),
    TaggedTemplateExpression(Box<'a, TaggedTemplateExpression<'a>>),
    ThisExpression(Box<'a, ThisExpression>),
    UnaryExpression(Box<'a, UnaryExpression<'a>>),
    UpdateExpression(Box<'a, UpdateExpression<'a>>),
    YieldExpression(Box<'a, YieldExpression<'a>>),
    PrivateInExpression(Box<'a, PrivateInExpression<'a>>),

    JSXElement(Box<'a, JSXElement<'a>>),
    JSXFragment(Box<'a, JSXFragment<'a>>),
}

impl<'a> Expression<'a> {
    /// `PrimaryExpression`
    /// [tc39/ecma262#prod-PrimaryExpression](https://tc39.es/ecma262/#prod-PrimaryExpression)
    pub fn is_primary_expression(&self) -> bool {
        self.is_literal_expression()
            || matches!(
                self,
                Self::Identifier(_)
                    | Self::ThisExpression(_)
                    | Self::FunctionExpression(_)
                    | Self::ClassExpression(_)
                    | Self::ArrayExpression(_)
                    | Self::ObjectExpression(_)
            )
    }

    pub fn is_literal_expression(&self) -> bool {
        matches!(
            self,
            Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::NumberLiteral(_)
                | Self::BigintLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::StringLiteral(_) // TemplateLiteral is not `Literal` type per oxc_ast
        )
    }

    pub fn is_string_literal(&self) -> bool {
        matches!(self, Self::StringLiteral(_) | Self::TemplateLiteral(_))
    }

    pub fn is_specific_string_literal(&self, string: &str) -> bool {
        match self {
            Self::StringLiteral(s) => s.value == string,
            _ => false,
        }
    }

    /// Determines whether the given expr is a `null` literal
    pub fn is_null(&self) -> bool {
        matches!(self, Expression::NullLiteral(_))
    }

    /// Determines whether the given expr is a `undefined` literal
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Identifier(ident) if ident.name == "undefined")
    }

    /// Determines whether the given expr is a `NaN` literal
    pub fn is_nan(&self) -> bool {
        matches!(self, Self::Identifier(ident) if ident.name == "NaN")
    }

    /// Determines whether the given expr is a `void 0`
    pub fn is_void_0(&self) -> bool {
        match self {
            Self::UnaryExpression(expr) if expr.operator == UnaryOperator::Void => {
                matches!(&expr.argument, Self::NumberLiteral(lit) if lit.value == 0.0)
            }
            _ => false,
        }
    }

    /// Determines whether the given expr is a `0`
    pub fn is_number_0(&self) -> bool {
        matches!(self, Self::NumberLiteral(lit) if lit.value == 0.0)
    }

    /// Determines whether the given expr evaluate to `undefined`
    pub fn evaluate_to_undefined(&self) -> bool {
        self.is_undefined() || self.is_void_0()
    }

    /// Determines whether the given expr is a `null` or `undefined` or `void 0`
    pub fn is_null_or_undefined(&self) -> bool {
        self.is_null() || self.evaluate_to_undefined()
    }

    pub fn is_specific_id(&self, name: &str) -> bool {
        match self {
            Expression::Identifier(ident) => ident.name == name,
            _ => false,
        }
    }

    pub fn is_specific_member_access(&'a self, object: &str, property: &str) -> bool {
        match self {
            Expression::MemberExpression(expr) => expr.is_specific_member_access(object, property),
            Expression::ChainExpression(chain) => {
                let ChainElement::MemberExpression(expr) = &chain.expression else {
                    return false;
                };
                expr.is_specific_member_access(object, property)
            }
            _ => false,
        }
    }

    pub fn get_identifier_reference(&self) -> Option<&IdentifierReference> {
        match self {
            Expression::Identifier(ident) => Some(ident),
            _ => None,
        }
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Expression::FunctionExpression(_) | Expression::ArrowExpression(_))
    }

    /// Returns literal's value converted to the Boolean type
    /// returns `true` when node is truthy, `false` when node is falsy, `None` when it cannot be determined.
    pub fn get_boolean_value(&self) -> Option<bool> {
        use num_traits::Zero;

        match self {
            Self::BooleanLiteral(lit) => Some(lit.value),
            Self::NullLiteral(_) => Some(false),
            Self::NumberLiteral(lit) => Some(lit.value != 0.0),
            Self::BigintLiteral(lit) => Some(!lit.value.is_zero()),
            Self::RegExpLiteral(_) => Some(true),
            Self::StringLiteral(lit) => Some(!lit.value.is_empty()),
            _ => None,
        }
    }

    pub fn is_immutable_value(&self) -> bool {
        match self {
            Self::BooleanLiteral(_)
            | Self::NullLiteral(_)
            | Self::NumberLiteral(_)
            | Self::BigintLiteral(_)
            | Self::RegExpLiteral(_)
            | Self::StringLiteral(_) => true,
            Self::TemplateLiteral(lit) if lit.is_no_substitution_template() => true,
            Self::UnaryExpression(unary_expr) => unary_expr.argument.is_immutable_value(),
            Self::Identifier(ident) => {
                matches!(ident.name.as_str(), "undefined" | "Infinity" | "NaN")
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct BooleanLiteral {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub value: bool,
}

impl BooleanLiteral {
    pub fn as_str(&self) -> &'static str {
        if self.value {
            "true"
        } else {
            "false"
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct NullLiteral {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

impl Hash for NullLiteral {
    fn hash<H: Hasher>(&self, state: &mut H) {
        None::<bool>.hash(state);
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct NumberLiteral<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub value: f64,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub raw: &'a str,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub base: NumberBase,
}

impl<'a> NumberLiteral<'a> {
    /// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/base/JSCompDoubles.java#L113)
    /// <https://262.ecma-international.org/5.1/#sec-9.5>
    #[allow(clippy::cast_possible_truncation)] // for `as i32`
    pub fn ecmascript_to_int32(num: f64) -> i32 {
        // Fast path for most common case. Also covers -0.0
        let int32_value = num as i32;
        if (f64::from(int32_value) - num).abs() < f64::EPSILON {
            return int32_value;
        }

        // NaN, Infinity if not included in our NumberLiteral, so we just skip step 2.

        // step 3
        let pos_int = num.signum() * num.abs().floor();

        // step 4
        let int32bit = pos_int % 2f64.powi(32);

        // step5
        if int32bit >= 2f64.powi(31) {
            (int32bit - 2f64.powi(32)) as i32
        } else {
            int32bit as i32
        }
    }
}

impl<'a> Hash for NumberLiteral<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.base.hash(state);
        self.raw.hash(state);
    }
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct BigintLiteral {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::serialize::serialize_bigint"))]
    pub value: BigInt,
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct RegExpLiteral {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    // valid regex is printed as {}
    // invalid regex is printed as null, which we can't implement yet
    pub value: EmptyObject,
    pub regex: RegExp,
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RegExp {
    pub pattern: Atom,
    pub flags: RegExpFlags,
}

impl fmt::Display for RegExp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}/{}", self.pattern, self.flags)
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RegExpFlags: u8 {
        const G = 1 << 0;
        const I = 1 << 1;
        const M = 1 << 2;
        const S = 1 << 3;
        const U = 1 << 4;
        const Y = 1 << 5;
        const D = 1 << 6;
        /// v flag from `https://github.com/tc39/proposal-regexp-set-notation`
        const V = 1 << 7;
    }
}

impl fmt::Display for RegExpFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.contains(Self::G) {
            write!(f, "g")?;
        }
        if self.contains(Self::I) {
            write!(f, "i")?;
        }
        if self.contains(Self::M) {
            write!(f, "m")?;
        }
        if self.contains(Self::S) {
            write!(f, "s")?;
        }
        if self.contains(Self::U) {
            write!(f, "u")?;
        }
        if self.contains(Self::Y) {
            write!(f, "y")?;
        }
        if self.contains(Self::D) {
            write!(f, "d")?;
        }
        if self.contains(Self::V) {
            write!(f, "v")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct EmptyObject;

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct StringLiteral {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub value: Atom,
}

impl StringLiteral {
    /// Static Semantics: `IsStringWellFormedUnicode`
    /// test for \uD800-\uDFFF
    pub fn is_string_well_formed_unicode(&self) -> bool {
        let mut chars = self.value.chars();
        while let Some(c) = chars.next() {
            if c == '\\' && chars.next() == Some('u') {
                let hex = &chars.as_str()[..4];
                if let Ok(hex) = u32::from_str_radix(hex, 16) {
                    if (0xd800..=0xdfff).contains(&hex) {
                        return false;
                    }
                };
            }
        }
        true
    }
}

/// Identifier Name
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct IdentifierName {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: Atom,
}

/// Identifier Reference
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct IdentifierReference {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: Atom,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub reference_id: Cell<ReferenceId>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub reference_flag: ReferenceFlag,
}

impl Hash for IdentifierReference {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

/// Binding Identifier
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct BindingIdentifier {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: Atom,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub symbol_id: Cell<SymbolId>,
}

impl Hash for BindingIdentifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

/// Label Identifier
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct LabelIdentifier {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: Atom,
}

/// This Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ThisExpression {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

/// Array Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ArrayExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub elements: Vec<'a, ArrayExpressionElement<'a>>,
    pub trailing_comma: Option<Span>,
}

/// Array Expression Element
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum ArrayExpressionElement<'a> {
    SpreadElement(Box<'a, SpreadElement<'a>>),
    Expression(Expression<'a>),
    Elision(Span),
}

/// Object Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ObjectExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub properties: Vec<'a, ObjectPropertyKind<'a>>,
    pub trailing_comma: Option<Span>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum ObjectPropertyKind<'a> {
    ObjectProperty(Box<'a, ObjectProperty<'a>>),
    SpreadProperty(Box<'a, SpreadElement<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ObjectProperty<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub kind: PropertyKind,
    pub key: PropertyKey<'a>,
    pub value: Expression<'a>,
    pub method: bool,
    pub shorthand: bool,
    pub computed: bool,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum PropertyKey<'a> {
    Identifier(Box<'a, IdentifierName>),
    PrivateIdentifier(Box<'a, PrivateIdentifier>),
    Expression(Expression<'a>),
}

impl<'a> PropertyKey<'a> {
    pub fn static_name(&self) -> Option<Atom> {
        match self {
            Self::Identifier(ident) => Some(ident.name.clone()),
            Self::PrivateIdentifier(_) => None,
            Self::Expression(expr) => match expr {
                Expression::StringLiteral(lit) => Some(lit.value.clone()),
                Expression::RegExpLiteral(lit) => Some(Atom::from(lit.regex.to_string())),
                Expression::NumberLiteral(lit) => Some(Atom::from(lit.value.to_string())),
                Expression::BigintLiteral(lit) => Some(Atom::from(lit.value.to_string())),
                Expression::NullLiteral(_) => Some("null".into()),
                Expression::TemplateLiteral(lit) => {
                    lit.expressions.is_empty().then(|| lit.quasi()).flatten().cloned()
                }
                _ => None,
            },
        }
    }

    pub fn is_private_identifier(&self) -> bool {
        matches!(self, Self::PrivateIdentifier(_))
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum PropertyValue<'a> {
    // For AssignmentProperty in ObjectPattern <https://github.com/estree/estree/blob/master/es2015.md#objectpattern>
    Pattern(BindingPattern<'a>),
    Expression(Expression<'a>),
}

#[derive(Debug, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "lowercase"))]
pub enum PropertyKind {
    Init,
    Get,
    Set,
}

/// Template Literal
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TemplateLiteral<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub quasis: Vec<'a, TemplateElement>,
    pub expressions: Vec<'a, Expression<'a>>,
}

impl<'a> TemplateLiteral<'a> {
    pub fn is_no_substitution_template(&self) -> bool {
        self.expressions.is_empty() && self.quasis.len() == 1
    }

    /// Get single quasi from `template`
    pub fn quasi(&self) -> Option<&Atom> {
        self.quasis.first().and_then(|quasi| quasi.value.cooked.as_ref())
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TaggedTemplateExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub tag: Expression<'a>,
    pub quasi: TemplateLiteral<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TemplateElement {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub tail: bool,
    pub value: TemplateElementValue,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct TemplateElementValue {
    pub raw: Atom,
    pub cooked: Option<Atom>,
}

/// Member Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum MemberExpression<'a> {
    ComputedMemberExpression(ComputedMemberExpression<'a>),
    StaticMemberExpression(StaticMemberExpression<'a>),
    PrivateFieldExpression(PrivateFieldExpression<'a>),
}

impl<'a> MemberExpression<'a> {
    pub fn optional(&self) -> bool {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => expr.optional,
            MemberExpression::StaticMemberExpression(expr) => expr.optional,
            MemberExpression::PrivateFieldExpression(expr) => expr.optional,
        }
    }

    pub fn object(&self) -> &Expression<'a> {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => &expr.object,
            MemberExpression::StaticMemberExpression(expr) => &expr.object,
            MemberExpression::PrivateFieldExpression(expr) => &expr.object,
        }
    }

    pub fn static_property_name(&'a self) -> Option<&'a str> {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => match &expr.expression {
                Expression::StringLiteral(lit) => Some(&lit.value),
                Expression::TemplateLiteral(lit) => {
                    if lit.expressions.is_empty() && lit.quasis.len() == 1 {
                        Some(&lit.quasis[0].value.raw)
                    } else {
                        None
                    }
                }
                _ => None,
            },
            MemberExpression::StaticMemberExpression(expr) => Some(&expr.property.name),
            MemberExpression::PrivateFieldExpression(_) => None,
        }
    }

    /// Whether it is a static member access `object.property`
    pub fn is_specific_member_access(&'a self, object: &str, property: &str) -> bool {
        self.object().is_specific_id(object)
            && self.static_property_name().is_some_and(|p| p == property)
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ComputedMemberExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub object: Expression<'a>,
    pub expression: Expression<'a>,
    pub optional: bool, // for optional chaining
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct StaticMemberExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub object: Expression<'a>,
    pub property: IdentifierName,
    pub optional: bool, // for optional chaining
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct PrivateFieldExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub object: Expression<'a>,
    pub field: PrivateIdentifier,
    pub optional: bool, // for optional chaining
}

/// Call Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct CallExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub callee: Expression<'a>,
    pub arguments: Vec<'a, Argument<'a>>,
    pub optional: bool, // for optional chaining
}

impl<'a> CallExpression<'a> {
    pub fn is_require_call(&self) -> bool {
        if self.arguments.len() != 1 {
            return false;
        }
        if let Expression::Identifier(id) = &self.callee {
            id.name == "require"
                && matches!(
                    self.arguments.first(),
                    Some(Argument::Expression(
                        Expression::StringLiteral(_) | Expression::TemplateLiteral(_),
                    )),
                )
        } else {
            false
        }
    }

    pub fn is_symbol_or_symbol_for_call(&'a self) -> bool {
        // TODO: is 'Symbol' reference to global object
        match &self.callee {
            Expression::Identifier(id) => id.name == "Symbol",
            Expression::MemberExpression(member) => {
                matches!(member.object(), Expression::Identifier(id) if id.name == "Symbol")
                    && member.static_property_name() == Some("for")
            }
            _ => false,
        }
    }

    pub fn common_js_require(&self) -> Option<&StringLiteral> {
        if !(self.callee.is_specific_id("require") && self.arguments.len() == 1) {
            return None;
        }
        match &self.arguments[0] {
            Argument::Expression(Expression::StringLiteral(str_literal)) => Some(str_literal),
            _ => None,
        }
    }
}

/// New Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct NewExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub callee: Expression<'a>,
    pub arguments: Vec<'a, Argument<'a>>,
}

/// Meta Property `new.target` | `import.meta`
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct MetaProperty {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub meta: IdentifierName,
    pub property: IdentifierName,
}

/// Spread Element
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct SpreadElement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub argument: Expression<'a>,
}

/// Argument
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum Argument<'a> {
    SpreadElement(Box<'a, SpreadElement<'a>>),
    Expression(Expression<'a>),
}

/// Update Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct UpdateExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub operator: UpdateOperator,
    pub prefix: bool,
    pub argument: SimpleAssignmentTarget<'a>,
}

/// Unary Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct UnaryExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub operator: UnaryOperator,
    pub argument: Expression<'a>,
}

/// Binary Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct BinaryExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub left: Expression<'a>,
    pub operator: BinaryOperator,
    pub right: Expression<'a>,
}

/// Private Identifier in Shift Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct PrivateInExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub left: PrivateIdentifier,
    pub right: Expression<'a>,
}

/// Binary Logical Operators
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct LogicalExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub left: Expression<'a>,
    pub operator: LogicalOperator,
    pub right: Expression<'a>,
}

/// Conditional Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ConditionalExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub test: Expression<'a>,
    pub consequent: Expression<'a>,
    pub alternate: Expression<'a>,
}

/// Assignment Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct AssignmentExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub operator: AssignmentOperator,
    pub left: AssignmentTarget<'a>,
    pub right: Expression<'a>,
}

/// Destructuring Assignment
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum AssignmentTarget<'a> {
    SimpleAssignmentTarget(SimpleAssignmentTarget<'a>),
    AssignmentTargetPattern(AssignmentTargetPattern<'a>),
}

impl<'a> AssignmentTarget<'a> {
    pub fn is_destructuring_pattern(&self) -> bool {
        matches!(self, Self::AssignmentTargetPattern(_))
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum SimpleAssignmentTarget<'a> {
    AssignmentTargetIdentifier(Box<'a, IdentifierReference>),
    MemberAssignmentTarget(Box<'a, MemberExpression<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum AssignmentTargetPattern<'a> {
    ArrayAssignmentTarget(Box<'a, ArrayAssignmentTarget<'a>>),
    ObjectAssignmentTarget(Box<'a, ObjectAssignmentTarget<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ArrayAssignmentTarget<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
    pub rest: Option<AssignmentTarget<'a>>,
    pub trailing_comma: Option<Span>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ObjectAssignmentTarget<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub properties: Vec<'a, AssignmentTargetProperty<'a>>,
    pub rest: Option<AssignmentTarget<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum AssignmentTargetMaybeDefault<'a> {
    AssignmentTarget(AssignmentTarget<'a>),
    AssignmentTargetWithDefault(Box<'a, AssignmentTargetWithDefault<'a>>),
}

impl<'a> AssignmentTargetMaybeDefault<'a> {
    pub fn name(&self) -> Option<Atom> {
        let target = match self {
            Self::AssignmentTarget(target) => target,
            Self::AssignmentTargetWithDefault(target) => &target.binding,
        };

        if let AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::AssignmentTargetIdentifier(id),
        ) = target
        {
            Some(id.name.clone())
        } else {
            None
        }
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct AssignmentTargetWithDefault<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub binding: AssignmentTarget<'a>,
    pub init: Expression<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum AssignmentTargetProperty<'a> {
    AssignmentTargetPropertyIdentifier(Box<'a, AssignmentTargetPropertyIdentifier<'a>>),
    AssignmentTargetPropertyProperty(Box<'a, AssignmentTargetPropertyProperty<'a>>),
}

/// Assignment Property - Identifier Reference
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct AssignmentTargetPropertyIdentifier<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub binding: IdentifierReference,
    pub init: Option<Expression<'a>>,
}

/// Assignment Property - Property Name
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct AssignmentTargetPropertyProperty<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: PropertyKey<'a>,
    pub binding: AssignmentTargetMaybeDefault<'a>,
}

/// Sequence Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct SequenceExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expressions: Vec<'a, Expression<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct Super {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

/// Await Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct AwaitExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub argument: Expression<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ChainExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: ChainElement<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum ChainElement<'a> {
    CallExpression(Box<'a, CallExpression<'a>>),
    MemberExpression(Box<'a, MemberExpression<'a>>),
}

/// Statements
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum Statement<'a> {
    // Statements
    BlockStatement(Box<'a, BlockStatement<'a>>),
    BreakStatement(Box<'a, BreakStatement>),
    ContinueStatement(Box<'a, ContinueStatement>),
    DebuggerStatement(Box<'a, DebuggerStatement>),
    DoWhileStatement(Box<'a, DoWhileStatement<'a>>),
    ExpressionStatement(Box<'a, ExpressionStatement<'a>>),
    ForInStatement(Box<'a, ForInStatement<'a>>),
    ForOfStatement(Box<'a, ForOfStatement<'a>>),
    ForStatement(Box<'a, ForStatement<'a>>),
    IfStatement(Box<'a, IfStatement<'a>>),
    LabeledStatement(Box<'a, LabeledStatement<'a>>),
    ReturnStatement(Box<'a, ReturnStatement<'a>>),
    SwitchStatement(Box<'a, SwitchStatement<'a>>),
    ThrowStatement(Box<'a, ThrowStatement<'a>>),
    TryStatement(Box<'a, TryStatement<'a>>),
    WhileStatement(Box<'a, WhileStatement<'a>>),
    WithStatement(Box<'a, WithStatement<'a>>),

    ModuleDeclaration(Box<'a, ModuleDeclaration<'a>>),
    Declaration(Declaration<'a>),
}

/// Directive Prologue
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct Directive {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub hir_id: HirId,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: StringLiteral,
    // directives should always use the unescaped raw string
    pub directive: Atom,
}

/// Hashbang
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct Hashbang {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub hir_id: HirId,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub value: Atom,
}

/// Block Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct BlockStatement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub body: Vec<'a, Statement<'a>>,
}

/// Declarations and the Variable Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum Declaration<'a> {
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>),
    FunctionDeclaration(Box<'a, Function<'a>>),
    ClassDeclaration(Box<'a, Class<'a>>),

    TSEnumDeclaration(Box<'a, TSEnumDeclaration<'a>>),
}

/// Variable Declaration
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct VariableDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub kind: VariableDeclarationKind,
    pub declarations: Vec<'a, VariableDeclarator<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "lowercase"))]
pub enum VariableDeclarationKind {
    Var,
    Const,
    Let,
}

impl VariableDeclarationKind {
    pub fn is_const(&self) -> bool {
        matches!(self, Self::Const)
    }

    pub fn is_lexical(&self) -> bool {
        matches!(self, Self::Const | Self::Let)
    }
}

impl fmt::Display for VariableDeclarationKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Var => "var",
            Self::Const => "const",
            Self::Let => "let",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct VariableDeclarator<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub kind: VariableDeclarationKind,
    pub id: BindingPattern<'a>,
    pub init: Option<Expression<'a>>,
    pub definite: bool,
}

/// Expression Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ExpressionStatement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

/// If Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct IfStatement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub test: Expression<'a>,
    pub consequent: Option<Statement<'a>>,
    pub alternate: Option<Statement<'a>>,
}

/// Do-While Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct DoWhileStatement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub body: Option<Statement<'a>>,
    pub test: Expression<'a>,
}

/// While Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct WhileStatement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub test: Expression<'a>,
    pub body: Option<Statement<'a>>,
}

/// For Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ForStatement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub init: Option<ForStatementInit<'a>>,
    pub test: Option<Expression<'a>>,
    pub update: Option<Expression<'a>>,
    pub body: Option<Statement<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum ForStatementInit<'a> {
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>),
    Expression(Expression<'a>),
}

impl<'a> ForStatementInit<'a> {
    /// LexicalDeclaration[In, Yield, Await] :
    ///   LetOrConst BindingList[?In, ?Yield, ?Await] ;
    pub fn is_lexical_declaration(&self) -> bool {
        matches!(self, Self::VariableDeclaration(decl) if decl.kind.is_lexical())
    }
}

/// For-In Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ForInStatement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub left: ForStatementLeft<'a>,
    pub right: Expression<'a>,
    pub body: Option<Statement<'a>>,
}

/// For-Of Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ForOfStatement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub r#await: bool,
    pub left: ForStatementLeft<'a>,
    pub right: Expression<'a>,
    pub body: Option<Statement<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum ForStatementLeft<'a> {
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>),
    AssignmentTarget(AssignmentTarget<'a>),
}

impl<'a> ForStatementLeft<'a> {
    /// LexicalDeclaration[In, Yield, Await] :
    ///   LetOrConst BindingList[?In, ?Yield, ?Await] ;
    pub fn is_lexical_declaration(&self) -> bool {
        matches!(self, Self::VariableDeclaration(decl) if decl.kind.is_lexical())
    }
}

/// Continue Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ContinueStatement {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub label: Option<LabelIdentifier>,
}

/// Break Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct BreakStatement {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub label: Option<LabelIdentifier>,
}

/// Return Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ReturnStatement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub argument: Option<Expression<'a>>,
}

/// With Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct WithStatement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub object: Expression<'a>,
    pub body: Option<Statement<'a>>,
}

/// Switch Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct SwitchStatement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub discriminant: Expression<'a>,
    pub cases: Vec<'a, SwitchCase<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct SwitchCase<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub test: Option<Expression<'a>>,
    pub consequent: Vec<'a, Statement<'a>>,
}

impl<'a> SwitchCase<'a> {
    pub fn is_default_case(&self) -> bool {
        self.test.is_none()
    }
}

/// Labelled Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct LabeledStatement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub label: LabelIdentifier,
    pub body: Option<Statement<'a>>,
}

/// Throw Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ThrowStatement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub argument: Expression<'a>,
}

/// Try Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TryStatement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub block: Box<'a, BlockStatement<'a>>,
    pub handler: Option<Box<'a, CatchClause<'a>>>,
    pub finalizer: Option<Box<'a, BlockStatement<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct CatchClause<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub param: Option<BindingPattern<'a>>,
    pub body: Box<'a, BlockStatement<'a>>,
}

/// Debugger Statement
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct DebuggerStatement {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

/// Destructuring Binding Patterns
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum BindingPattern<'a> {
    BindingIdentifier(Box<'a, BindingIdentifier>),
    ObjectPattern(Box<'a, ObjectPattern<'a>>),
    ArrayPattern(Box<'a, ArrayPattern<'a>>),
    RestElement(Box<'a, RestElement<'a>>),
    AssignmentPattern(Box<'a, AssignmentPattern<'a>>),
}

impl<'a> BindingPattern<'a> {
    pub fn is_binding_identifier(&self) -> bool {
        matches!(self, Self::BindingIdentifier(_))
    }

    pub fn is_destructuring_pattern(&self) -> bool {
        matches!(self, Self::ObjectPattern(_) | Self::ArrayPattern(_))
    }

    pub fn is_rest_element(&self) -> bool {
        matches!(self, Self::RestElement(_))
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct AssignmentPattern<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub left: BindingPattern<'a>,
    pub right: Expression<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ObjectPattern<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub properties: Vec<'a, BindingProperty<'a>>,
    pub rest: Option<Box<'a, RestElement<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct BindingProperty<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub key: PropertyKey<'a>,
    pub value: BindingPattern<'a>,
    pub shorthand: bool,
    pub computed: bool,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ArrayPattern<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub elements: Vec<'a, Option<BindingPattern<'a>>>,
    pub rest: Option<Box<'a, RestElement<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct RestElement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub argument: BindingPattern<'a>,
}

/// Function Definitions
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[allow(clippy::struct_excessive_bools)]
pub struct Function<'a> {
    pub r#type: FunctionType,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: Option<BindingIdentifier>,
    pub expression: bool,
    pub generator: bool,
    pub r#async: bool,
    pub params: Box<'a, FormalParameters<'a>>,
    pub body: Option<Box<'a, FunctionBody<'a>>>,
}

impl<'a> Function<'a> {
    pub fn is_expression(&self) -> bool {
        self.r#type == FunctionType::FunctionExpression
    }

    pub fn is_function_declaration(&self) -> bool {
        matches!(self.r#type, FunctionType::FunctionDeclaration)
    }

    pub fn is_ts_declare_function(&self) -> bool {
        matches!(self.r#type, FunctionType::TSDeclareFunction)
    }

    pub fn is_declaration(&self) -> bool {
        matches!(self.r#type, FunctionType::FunctionDeclaration | FunctionType::TSDeclareFunction)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum FunctionType {
    FunctionDeclaration,
    FunctionExpression,
    TSDeclareFunction,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct FormalParameters<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub kind: FormalParameterKind,
    pub items: Vec<'a, FormalParameter<'a>>,
    pub rest: Option<Box<'a, RestElement<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct FormalParameter<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub pattern: BindingPattern<'a>,
    pub decorators: Vec<'a, Decorator<'a>>,
}

#[derive(Debug, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum FormalParameterKind {
    /// <https://tc39.es/ecma262/#prod-FormalParameters>
    FormalParameter,
    /// <https://tc39.es/ecma262/#prod-UniqueFormalParameters>
    UniqueFormalParameters,
    /// <https://tc39.es/ecma262/#prod-ArrowFormalParameters>
    ArrowFormalParameters,
    /// Part of TypeScript type signatures
    Signature,
}

impl<'a> FormalParameters<'a> {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct FunctionBody<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub directives: Vec<'a, Directive>,
    pub statements: Vec<'a, Statement<'a>>,
}

impl<'a> FunctionBody<'a> {
    pub fn is_empty(&self) -> bool {
        self.directives.is_empty() && self.statements.is_empty()
    }
}

/// Arrow Function Definitions
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct ArrowExpression<'a> {
    pub span: Span,
    /// Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    pub expression: bool,
    pub generator: bool,
    pub r#async: bool,
    pub params: Box<'a, FormalParameters<'a>>, // UniqueFormalParameters in spec
    /// See `expression` for whether this arrow expression returns an expression.
    pub body: Box<'a, FunctionBody<'a>>,
}

/// Generator Function Definitions
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct YieldExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub delegate: bool,
    pub argument: Option<Expression<'a>>,
}

/// Class Definitions
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
pub struct Class<'a> {
    pub r#type: ClassType,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: Option<BindingIdentifier>,
    pub super_class: Option<Expression<'a>>,
    pub body: Box<'a, ClassBody<'a>>,
    pub decorators: Vec<'a, Decorator<'a>>,
}

impl<'a> Class<'a> {
    pub fn is_expression(&self) -> bool {
        self.r#type == ClassType::ClassExpression
    }

    pub fn is_declaration(&self) -> bool {
        self.r#type == ClassType::ClassDeclaration
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum ClassType {
    ClassDeclaration,
    ClassExpression,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ClassBody<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub body: Vec<'a, ClassElement<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum ClassElement<'a> {
    StaticBlock(Box<'a, StaticBlock<'a>>),
    MethodDefinition(Box<'a, MethodDefinition<'a>>),
    PropertyDefinition(Box<'a, PropertyDefinition<'a>>),
    AccessorProperty(Box<'a, AccessorProperty<'a>>),
}

impl<'a> ClassElement<'a> {
    pub fn r#static(&self) -> bool {
        match self {
            Self::StaticBlock(_) => false,
            Self::MethodDefinition(def) => def.r#static,
            Self::PropertyDefinition(def) => def.r#static,
            Self::AccessorProperty(def) => def.r#static,
        }
    }

    pub fn computed(&self) -> bool {
        match self {
            Self::StaticBlock(_) => false,
            Self::MethodDefinition(def) => def.computed,
            Self::PropertyDefinition(def) => def.computed,
            Self::AccessorProperty(def) => def.computed,
        }
    }

    pub fn method_definition_kind(&self) -> Option<MethodDefinitionKind> {
        match self {
            Self::StaticBlock(_) | Self::PropertyDefinition(_) | Self::AccessorProperty(_) => None,
            Self::MethodDefinition(def) => Some(def.kind),
        }
    }

    pub fn property_key(&self) -> Option<&PropertyKey<'a>> {
        match self {
            Self::StaticBlock(_) => None,
            Self::MethodDefinition(def) => Some(&def.key),
            Self::PropertyDefinition(def) => Some(&def.key),
            Self::AccessorProperty(def) => Some(&def.key),
        }
    }

    pub fn static_name(&self) -> Option<Atom> {
        match self {
            Self::StaticBlock(_) => None,
            Self::MethodDefinition(def) => def.key.static_name(),
            Self::PropertyDefinition(def) => def.key.static_name(),
            Self::AccessorProperty(def) => def.key.static_name(),
        }
    }

    pub fn is_ts_empty_body_function(&self) -> bool {
        match self {
            Self::PropertyDefinition(_) | Self::StaticBlock(_) | Self::AccessorProperty(_) => false,
            Self::MethodDefinition(method) => method.value.body.is_none(),
        }
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[allow(clippy::struct_excessive_bools)]
pub struct MethodDefinition<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub key: PropertyKey<'a>,
    pub value: Box<'a, Function<'a>>, // FunctionExpression
    pub kind: MethodDefinitionKind,
    pub computed: bool,
    pub r#static: bool,
    pub r#override: bool,
    pub optional: bool,
    pub decorators: Vec<'a, Decorator<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[allow(clippy::struct_excessive_bools)]
pub struct PropertyDefinition<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub key: PropertyKey<'a>,
    pub value: Option<Expression<'a>>,
    pub computed: bool,
    pub r#static: bool,
    pub declare: bool,
    pub r#override: bool,
    pub optional: bool,
    pub definite: bool,
    pub readonly: bool,
    pub decorators: Vec<'a, Decorator<'a>>,
}

#[derive(Debug, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "lowercase"))]
pub enum MethodDefinitionKind {
    Constructor,
    Method,
    Get,
    Set,
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct PrivateIdentifier {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: Atom,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct StaticBlock<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub body: Vec<'a, Statement<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum ModuleDeclaration<'a> {
    ImportDeclaration(Box<'a, ImportDeclaration<'a>>),
    ExportAllDeclaration(Box<'a, ExportAllDeclaration<'a>>),
    ExportDefaultDeclaration(Box<'a, ExportDefaultDeclaration<'a>>),
    ExportNamedDeclaration(Box<'a, ExportNamedDeclaration<'a>>),
}

impl<'a> ModuleDeclaration<'a> {
    pub fn is_export(&self) -> bool {
        matches!(
            self,
            Self::ExportAllDeclaration(_)
                | Self::ExportDefaultDeclaration(_)
                | Self::ExportNamedDeclaration(_)
        )
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct AccessorProperty<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub key: PropertyKey<'a>,
    pub value: Option<Expression<'a>>,
    pub computed: bool,
    pub r#static: bool,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ImportExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub source: Expression<'a>,
    pub arguments: Vec<'a, Expression<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct ImportDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub specifiers: Vec<'a, ImportDeclarationSpecifier>,
    pub source: StringLiteral,
    pub assertions: Option<Vec<'a, ImportAttribute>>, // Some(vec![]) for empty assertion
    pub import_kind: ImportOrExportKind,              // `import type { foo } from 'bar'`
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum ImportDeclarationSpecifier {
    ImportSpecifier(ImportSpecifier),
    ImportDefaultSpecifier(ImportDefaultSpecifier),
    ImportNamespaceSpecifier(ImportNamespaceSpecifier),
}

// import {imported} from "source"
// import {imported as local} from "source"
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ImportSpecifier {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub imported: ModuleExportName,
    pub local: BindingIdentifier,
}

// import local from "source"
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ImportDefaultSpecifier {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub local: BindingIdentifier,
}

// import * as local from "source"
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ImportNamespaceSpecifier {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub local: BindingIdentifier,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ImportAttribute {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub key: ImportAttributeKey,
    pub value: StringLiteral,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum ImportAttributeKey {
    Identifier(IdentifierName),
    StringLiteral(StringLiteral),
}

impl ImportAttributeKey {
    pub fn as_atom(&self) -> Atom {
        match self {
            Self::Identifier(identifier) => identifier.name.clone(),
            Self::StringLiteral(literal) => literal.value.clone(),
        }
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ExportNamedDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub declaration: Option<Declaration<'a>>,
    pub specifiers: Vec<'a, ExportSpecifier>,
    pub source: Option<StringLiteral>,
    pub export_kind: ImportOrExportKind, // `export type { foo }`
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ExportDefaultDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub declaration: ExportDefaultDeclarationKind<'a>,
    pub exported: ModuleExportName, // `default`
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ExportAllDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub exported: Option<ModuleExportName>,
    pub source: StringLiteral,
    pub assertions: Option<Vec<'a, ImportAttribute>>, // Some(vec![]) for empty assertion
    pub export_kind: ImportOrExportKind,              // `export type *`
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct ExportSpecifier {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub local: ModuleExportName,
    pub exported: ModuleExportName,
    pub export_kind: ImportOrExportKind, // `export type *`
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum ExportDefaultDeclarationKind<'a> {
    Expression(Expression<'a>),
    FunctionDeclaration(Box<'a, Function<'a>>),
    ClassDeclaration(Box<'a, Class<'a>>),

    TSEnumDeclaration(Box<'a, TSEnumDeclaration<'a>>),
}

// es2022: <https://github.com/estree/estree/blob/master/es2022.md#modules>
// <https://github.com/tc39/ecma262/pull/2154>
// support:
//   import {"\0 any unicode" as foo} from "";
//   export {foo as "\0 any unicode"};
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum ModuleExportName {
    Identifier(IdentifierName),
    StringLiteral(StringLiteral),
}

impl fmt::Display for ModuleExportName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Identifier(identifier) => identifier.name.to_string(),
            Self::StringLiteral(literal) => literal.value.to_string(),
        };
        write!(f, "{s}")
    }
}

impl ModuleExportName {
    pub fn name(&self) -> &Atom {
        match self {
            Self::Identifier(identifier) => &identifier.name,
            Self::StringLiteral(literal) => &literal.value,
        }
    }
}

/* TypeScript */

/// Enum Declaration
///
/// `const_opt` enum `BindingIdentifier` { `EnumBody_opt` }
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSEnumDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: BindingIdentifier,
    pub members: Vec<'a, TSEnumMember<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSEnumMember<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: TSEnumMemberName<'a>,
    pub initializer: Option<Expression<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum TSEnumMemberName<'a> {
    Identifier(IdentifierName),
    StringLiteral(StringLiteral),
    // Invalid Grammar `enum E { [computed] }`
    ComputedPropertyName(Expression<'a>),
    // Invalid Grammar `enum E { 1 }`
    NumberLiteral(NumberLiteral<'a>),
}

#[derive(Debug, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub enum ImportOrExportKind {
    Value,
    Type,
}

impl ImportOrExportKind {
    pub fn is_value(&self) -> bool {
        matches!(self, Self::Value)
    }

    pub fn is_type(&self) -> bool {
        matches!(self, Self::Type)
    }
}

/// JSX Element
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct JSXElement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub opening_element: Box<'a, JSXOpeningElement<'a>>,
    pub closing_element: Option<Box<'a, JSXClosingElement<'a>>>,
    pub children: Vec<'a, JSXChild<'a>>,
}

/// JSX Opening Element
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct JSXOpeningElement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub self_closing: bool,
    pub name: JSXElementName<'a>,
    pub attributes: Vec<'a, JSXAttributeItem<'a>>,
}

/// JSX Closing Element
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXClosingElement<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: JSXElementName<'a>,
}

/// JSX Fragment
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct JSXFragment<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub opening_fragment: JSXOpeningFragment,
    pub closing_fragment: JSXClosingFragment,
    pub children: Vec<'a, JSXChild<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXOpeningFragment {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXClosingFragment {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

/// JSX Element Name
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum JSXElementName<'a> {
    Identifier(JSXIdentifier),
    NamespacedName(Box<'a, JSXNamespacedName>),
    MemberExpression(Box<'a, JSXMemberExpression<'a>>),
}

/// JSX Namespaced Name
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXNamespacedName {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub namespace: JSXIdentifier,
    pub property: JSXIdentifier,
}

/// JSX Member Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXMemberExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub object: JSXMemberExpressionObject<'a>,
    pub property: JSXIdentifier,
}

impl<'a> JSXMemberExpression<'a> {
    pub fn get_object_identifier(&self) -> &JSXIdentifier {
        match &self.object {
            JSXMemberExpressionObject::Identifier(ident) => ident,
            JSXMemberExpressionObject::MemberExpression(expr) => expr.get_object_identifier(),
        }
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum JSXMemberExpressionObject<'a> {
    Identifier(JSXIdentifier),
    MemberExpression(Box<'a, JSXMemberExpression<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXExpressionContainer<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: JSXExpression<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum JSXExpression<'a> {
    Expression(Expression<'a>),
    EmptyExpression(JSXEmptyExpression),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXEmptyExpression {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

// 1.3 JSX Attributes

/// JSX Attributes
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum JSXAttributeItem<'a> {
    Attribute(Box<'a, JSXAttribute<'a>>),
    SpreadAttribute(Box<'a, JSXSpreadAttribute<'a>>),
}

/// JSX Attribute
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXAttribute<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: JSXAttributeName<'a>,
    pub value: Option<JSXAttributeValue<'a>>,
}

/// JSX Spread Attribute
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXSpreadAttribute<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub argument: Expression<'a>,
}

/// JSX Attribute Name
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum JSXAttributeName<'a> {
    Identifier(JSXIdentifier),
    NamespacedName(Box<'a, JSXNamespacedName>),
}

/// JSX Attribute Value
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum JSXAttributeValue<'a> {
    StringLiteral(StringLiteral),
    ExpressionContainer(JSXExpressionContainer<'a>),
    Element(Box<'a, JSXElement<'a>>),
    Fragment(Box<'a, JSXFragment<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXIdentifier {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: Atom,
}

// 1.4 JSX Children

/// JSX Child
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum JSXChild<'a> {
    Text(JSXText),
    Element(Box<'a, JSXElement<'a>>),
    Fragment(Box<'a, JSXFragment<'a>>),
    ExpressionContainer(JSXExpressionContainer<'a>),
    Spread(JSXSpreadChild<'a>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXSpreadChild<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

/// JSX Text
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct JSXText {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub value: Atom,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct Decorator<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}
