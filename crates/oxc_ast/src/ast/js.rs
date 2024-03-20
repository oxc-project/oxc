// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use std::{cell::Cell, fmt, hash::Hash};

#[cfg(feature = "raw")]
use layout_inspect::Inspect;
use oxc_allocator::{Box, Vec};
use oxc_macros::ast_node;
use oxc_span::{Atom, CompactStr, SourceType, Span};
use oxc_syntax::{
    operator::{
        AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
    },
    reference::{ReferenceFlag, ReferenceId},
    symbol::SymbolId,
};
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

use super::{jsx::*, literal::*, ts::*};

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export interface BindingIdentifier extends Span { type: "Identifier", name: Atom }
export interface IdentifierReference extends Span { type: "Identifier", name: Atom }
export interface IdentifierName extends Span { type: "Identifier", name: Atom }
export interface LabelIdentifier extends Span { type: "Identifier", name: Atom }
export interface AssignmentTargetRest extends Span { type: "RestElement", argument: AssignmentTarget }
export interface BindingRestElement extends Span { type: "RestElement", argument: BindingPattern }
export interface FormalParameterRest extends Span {
    type: "RestElement",
    argument: BindingPatternKind,
    typeAnnotation?: TSTypeAnnotation,
    optional: boolean,
}
"#;

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct Program<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub source_type: SourceType,
    pub directives: Vec<'a, Directive<'a>>,
    pub hashbang: Option<Hashbang<'a>>,
    pub body: Vec<'a, Statement<'a>>,
}

impl<'a> Program<'a> {
    pub fn is_empty(&self) -> bool {
        self.body.is_empty() && self.directives.is_empty()
    }

    pub fn is_strict(&self) -> bool {
        self.source_type.is_module()
            || self.source_type.always_strict()
            || self.directives.iter().any(|d| d.directive == "use strict")
    }
}

/// Expression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum Expression<'a> {
    BooleanLiteral(Box<'a, BooleanLiteral>),
    NullLiteral(Box<'a, NullLiteral>),
    NumericLiteral(Box<'a, NumericLiteral<'a>>),
    BigintLiteral(Box<'a, BigIntLiteral<'a>>),
    RegExpLiteral(Box<'a, RegExpLiteral<'a>>),
    StringLiteral(Box<'a, StringLiteral<'a>>),
    TemplateLiteral(Box<'a, TemplateLiteral<'a>>),

    Identifier(Box<'a, IdentifierReference<'a>>),

    MetaProperty(Box<'a, MetaProperty<'a>>),
    Super(Box<'a, Super>),

    ArrayExpression(Box<'a, ArrayExpression<'a>>),
    ArrowFunctionExpression(Box<'a, ArrowFunctionExpression<'a>>),
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
    ParenthesizedExpression(Box<'a, ParenthesizedExpression<'a>>),
    SequenceExpression(Box<'a, SequenceExpression<'a>>),
    TaggedTemplateExpression(Box<'a, TaggedTemplateExpression<'a>>),
    ThisExpression(Box<'a, ThisExpression>),
    UnaryExpression(Box<'a, UnaryExpression<'a>>),
    UpdateExpression(Box<'a, UpdateExpression<'a>>),
    YieldExpression(Box<'a, YieldExpression<'a>>),
    PrivateInExpression(Box<'a, PrivateInExpression<'a>>),

    JSXElement(Box<'a, JSXElement<'a>>),
    JSXFragment(Box<'a, JSXFragment<'a>>),

    TSAsExpression(Box<'a, TSAsExpression<'a>>),
    TSSatisfiesExpression(Box<'a, TSSatisfiesExpression<'a>>),
    TSTypeAssertion(Box<'a, TSTypeAssertion<'a>>),
    TSNonNullExpression(Box<'a, TSNonNullExpression<'a>>),
    TSInstantiationExpression(Box<'a, TSInstantiationExpression<'a>>),
}

impl<'a> Expression<'a> {
    pub fn is_primary_expression(&self) -> bool {
        self.is_literal()
            || matches!(
                self,
                Self::Identifier(_)
                    | Self::ThisExpression(_)
                    | Self::FunctionExpression(_)
                    | Self::ClassExpression(_)
                    | Self::ParenthesizedExpression(_)
                    | Self::ArrayExpression(_)
                    | Self::ObjectExpression(_)
            )
    }

    pub fn is_literal(&self) -> bool {
        // Note: TemplateLiteral is not `Literal`
        matches!(
            self,
            Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::NumericLiteral(_)
                | Self::BigintLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::StringLiteral(_)
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

    /// Determines whether the given expr is a `void expr`
    pub fn is_void(&self) -> bool {
        matches!(self, Self::UnaryExpression(expr) if expr.operator == UnaryOperator::Void)
    }

    /// Determines whether the given expr is a `void 0`
    pub fn is_void_0(&self) -> bool {
        match self {
            Self::UnaryExpression(expr) if expr.operator == UnaryOperator::Void => {
                matches!(&expr.argument, Self::NumericLiteral(lit) if lit.value == 0.0)
            }
            _ => false,
        }
    }

    /// Determines whether the given expr is a `0`
    pub fn is_number_0(&self) -> bool {
        matches!(self, Self::NumericLiteral(lit) if lit.value == 0.0)
    }

    pub fn is_number(&self, val: f64) -> bool {
        matches!(self, Self::NumericLiteral(lit) if (lit.value - val).abs() < f64::EPSILON)
    }

    /// Determines whether the given numeral literal's raw value is exactly val
    pub fn is_specific_raw_number_literal(&self, val: &str) -> bool {
        matches!(self, Self::NumericLiteral(lit) if lit.raw == val)
    }

    /// Determines whether the given expr evaluate to `undefined`
    pub fn evaluate_to_undefined(&self) -> bool {
        self.is_undefined() || self.is_void()
    }

    /// Determines whether the given expr is a `null` or `undefined` or `void 0`
    pub fn is_null_or_undefined(&self) -> bool {
        self.is_null() || self.evaluate_to_undefined()
    }

    /// Determines whether the given expr is a `NaN` literal
    pub fn is_nan(&self) -> bool {
        matches!(self, Self::Identifier(ident) if ident.name == "NaN")
    }

    /// Remove nested parentheses from this expression.
    pub fn without_parenthesized(&self) -> &Self {
        match self {
            Expression::ParenthesizedExpression(Box(expr)) => {
                expr.expression.without_parenthesized()
            }
            _ => self,
        }
    }

    pub fn is_specific_id(&self, name: &str) -> bool {
        match self.get_inner_expression() {
            Expression::Identifier(ident) => ident.name == name,
            _ => false,
        }
    }

    pub fn is_specific_member_access(&self, object: &str, property: &str) -> bool {
        match self.get_inner_expression() {
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

    pub fn get_inner_expression(&self) -> &Expression<'a> {
        match self {
            Expression::ParenthesizedExpression(expr) => expr.expression.get_inner_expression(),
            Expression::TSAsExpression(expr) => expr.expression.get_inner_expression(),
            Expression::TSSatisfiesExpression(expr) => expr.expression.get_inner_expression(),
            Expression::TSInstantiationExpression(expr) => expr.expression.get_inner_expression(),
            Expression::TSNonNullExpression(expr) => expr.expression.get_inner_expression(),
            Expression::TSTypeAssertion(expr) => expr.expression.get_inner_expression(),
            _ => self,
        }
    }

    pub fn is_identifier_reference(&self) -> bool {
        matches!(self, Expression::Identifier(_))
    }

    pub fn get_identifier_reference(&self) -> Option<&IdentifierReference> {
        match self.get_inner_expression() {
            Expression::Identifier(ident) => Some(ident),
            _ => None,
        }
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_))
    }

    pub fn is_call_expression(&self) -> bool {
        matches!(self, Expression::CallExpression(_))
    }

    pub fn is_call_like_expression(&self) -> bool {
        self.is_call_expression()
            && matches!(self, Expression::NewExpression(_) | Expression::ImportExpression(_))
    }

    pub fn is_binaryish(&self) -> bool {
        matches!(self, Expression::BinaryExpression(_) | Expression::LogicalExpression(_))
    }

    /// Returns literal's value converted to the Boolean type
    /// returns `true` when node is truthy, `false` when node is falsy, `None` when it cannot be determined.
    pub fn get_boolean_value(&self) -> Option<bool> {
        match self {
            Self::BooleanLiteral(lit) => Some(lit.value),
            Self::NullLiteral(_) => Some(false),
            Self::NumericLiteral(lit) => Some(lit.value != 0.0),
            Self::BigintLiteral(lit) => Some(!lit.is_zero()),
            Self::RegExpLiteral(_) => Some(true),
            Self::StringLiteral(lit) => Some(!lit.value.is_empty()),
            _ => None,
        }
    }

    pub fn get_member_expr(&self) -> Option<&MemberExpression<'a>> {
        match self.get_inner_expression() {
            Expression::ChainExpression(chain_expr) => match &chain_expr.expression {
                ChainElement::CallExpression(_) => None,
                ChainElement::MemberExpression(member_expr) => Some(member_expr),
            },
            Expression::MemberExpression(member_expr) => Some(member_expr),
            _ => None,
        }
    }

    pub fn is_immutable_value(&self) -> bool {
        match self {
            Self::BooleanLiteral(_)
            | Self::NullLiteral(_)
            | Self::NumericLiteral(_)
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

/// Identifier Name
#[ast_node]
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename = "Identifier"))]
pub struct IdentifierName<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: Atom<'a>,
}

impl<'a> IdentifierName<'a> {
    pub fn new(span: Span, name: Atom<'a>) -> Self {
        Self { span, name }
    }
}

/// Identifier Reference
#[ast_node]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename = "Identifier"))]
pub struct IdentifierReference<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: Atom<'a>,
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub reference_id: Cell<Option<ReferenceId>>,
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub reference_flag: ReferenceFlag,
}

impl<'a> Hash for IdentifierReference<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.span.hash(state);
        self.name.hash(state);
    }
}

impl<'a> IdentifierReference<'a> {
    pub fn new(span: Span, name: Atom<'a>) -> Self {
        Self { span, name, reference_id: Cell::default(), reference_flag: ReferenceFlag::default() }
    }
}

/// Binding Identifier
#[ast_node]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename = "Identifier"))]
pub struct BindingIdentifier<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: Atom<'a>,
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub symbol_id: Cell<Option<SymbolId>>,
}

impl<'a> Hash for BindingIdentifier<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.span.hash(state);
        self.name.hash(state);
    }
}

impl<'a> BindingIdentifier<'a> {
    pub fn new(span: Span, name: Atom<'a>) -> Self {
        Self { span, name, symbol_id: Cell::default() }
    }
}

/// Label Identifier
#[ast_node]
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename = "Identifier"))]
pub struct LabelIdentifier<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: Atom<'a>,
}

/// This Expression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ThisExpression {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

/// <https://tc39.es/ecma262/#prod-ArrayLiteral>
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct ArrayExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    #[cfg_attr(feature = "serialize", tsify(type = "Array<SpreadElement | Expression | null>"))]
    pub elements: Vec<'a, ArrayExpressionElement<'a>>,
    /// Array trailing comma
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Trailing_commas#arrays>
    pub trailing_comma: Option<Span>,
}

/// Array Expression Element
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum ArrayExpressionElement<'a> {
    SpreadElement(Box<'a, SpreadElement<'a>>),
    Expression(Expression<'a>),
    /// Array hole for sparse arrays
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Trailing_commas#arrays>
    // Serialize as `null`. `serialize_elision` in serialize.rs.
    #[cfg_attr(
        feature = "serialize",
        serde(serialize_with = "ArrayExpressionElement::serialize_elision")
    )]
    Elision(Span),
}

impl<'a> ArrayExpressionElement<'a> {
    pub fn is_elision(&self) -> bool {
        matches!(self, Self::Elision(_))
    }
}

/// Object Expression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct ObjectExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub properties: Vec<'a, ObjectPropertyKind<'a>>,
    pub trailing_comma: Option<Span>,
}

impl<'a> ObjectExpression<'a> {
    pub fn has_proto(&self) -> bool {
        use crate::syntax_directed_operations::PropName;
        self.properties.iter().any(|p| p.prop_name().is_some_and(|name| name.0 == "__proto__"))
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum ObjectPropertyKind<'a> {
    ObjectProperty(Box<'a, ObjectProperty<'a>>),
    SpreadProperty(Box<'a, SpreadElement<'a>>),
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ObjectProperty<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub kind: PropertyKind,
    pub key: PropertyKey<'a>,
    pub value: Expression<'a>,
    pub init: Option<Expression<'a>>, // for `CoverInitializedName`
    pub method: bool,
    pub shorthand: bool,
    pub computed: bool,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum PropertyKey<'a> {
    Identifier(Box<'a, IdentifierName<'a>>),
    PrivateIdentifier(Box<'a, PrivateIdentifier<'a>>),
    Expression(Expression<'a>),
}

impl<'a> PropertyKey<'a> {
    pub fn static_name(&self) -> Option<CompactStr> {
        match self {
            Self::Identifier(ident) => Some(ident.name.to_compact_str()),
            Self::PrivateIdentifier(_) => None,
            Self::Expression(expr) => match expr {
                Expression::StringLiteral(lit) => Some(lit.value.to_compact_str()),
                Expression::RegExpLiteral(lit) => Some(lit.regex.to_string().into()),
                Expression::NumericLiteral(lit) => Some(lit.value.to_string().into()),
                Expression::BigintLiteral(lit) => Some(lit.raw.to_compact_str()),
                Expression::NullLiteral(_) => Some("null".into()),
                Expression::TemplateLiteral(lit) => lit
                    .expressions
                    .is_empty()
                    .then(|| lit.quasi())
                    .flatten()
                    .map(Atom::to_compact_str),
                _ => None,
            },
        }
    }

    pub fn is_specific_static_name(&self, name: &str) -> bool {
        self.static_name().is_some_and(|n| n == name)
    }

    pub fn is_identifier(&self) -> bool {
        matches!(self, Self::PrivateIdentifier(_) | Self::Identifier(_))
    }

    pub fn is_private_identifier(&self) -> bool {
        matches!(self, Self::PrivateIdentifier(_))
    }

    pub fn private_name(&self) -> Option<&Atom<'a>> {
        match self {
            Self::PrivateIdentifier(ident) => Some(&ident.name),
            _ => None,
        }
    }

    pub fn name(&self) -> Option<CompactStr> {
        if self.is_private_identifier() {
            self.private_name().map(Atom::to_compact_str)
        } else {
            self.static_name()
        }
    }

    pub fn is_specific_id(&self, name: &str) -> bool {
        match self {
            PropertyKey::Identifier(ident) => ident.name == name,
            _ => false,
        }
    }

    pub fn is_specific_string_literal(&self, string: &str) -> bool {
        match self {
            PropertyKey::Expression(expr) => expr.is_specific_string_literal(string),
            _ => false,
        }
    }
}

#[ast_node]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum PropertyKind {
    Init,
    Get,
    Set,
}

/// Template Literal
///
/// This is interpreted by interleaving the expression elements in between the quasi elements.
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TemplateLiteral<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub quasis: Vec<'a, TemplateElement<'a>>,
    pub expressions: Vec<'a, Expression<'a>>,
}

impl<'a> TemplateLiteral<'a> {
    pub fn is_no_substitution_template(&self) -> bool {
        self.expressions.is_empty() && self.quasis.len() == 1
    }

    /// Get single quasi from `template`
    pub fn quasi(&self) -> Option<&Atom<'a>> {
        self.quasis.first().and_then(|quasi| quasi.value.cooked.as_ref())
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TaggedTemplateExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub tag: Expression<'a>,
    pub quasi: TemplateLiteral<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TemplateElement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub tail: bool,
    pub value: TemplateElementValue<'a>,
}

/// See [template-strings-cooked-vs-raw](https://exploringjs.com/impatient-js/ch_template-literals.html#template-strings-cooked-vs-raw)
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct TemplateElementValue<'a> {
    /// A raw interpretation where backslashes do not have special meaning.
    /// For example, \t produces two characters – a backslash and a t.
    /// This interpretation of the template strings is stored in property .raw of the first argument (an Array).
    pub raw: Atom<'a>,
    /// A cooked interpretation where backslashes have special meaning.
    /// For example, \t produces a tab character.
    /// This interpretation of the template strings is stored as an Array in the first argument.
    /// cooked = None when template literal has invalid escape sequence
    pub cooked: Option<Atom<'a>>,
}

/// <https://tc39.es/ecma262/#prod-MemberExpression>
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum MemberExpression<'a> {
    /// `MemberExpression[?Yield, ?Await] [ Expression[+In, ?Yield, ?Await] ]`
    ComputedMemberExpression(ComputedMemberExpression<'a>),
    /// `MemberExpression[?Yield, ?Await] . IdentifierName`
    StaticMemberExpression(StaticMemberExpression<'a>),
    /// `MemberExpression[?Yield, ?Await] . PrivateIdentifier`
    PrivateFieldExpression(PrivateFieldExpression<'a>),
}

impl<'a> MemberExpression<'a> {
    pub fn is_computed(&self) -> bool {
        matches!(self, MemberExpression::ComputedMemberExpression(_))
    }

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

    pub fn static_property_name(&self) -> Option<&str> {
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
            MemberExpression::StaticMemberExpression(expr) => Some(expr.property.name.as_str()),
            MemberExpression::PrivateFieldExpression(_) => None,
        }
    }

    pub fn static_property_info(&self) -> Option<(Span, &str)> {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => match &expr.expression {
                Expression::StringLiteral(lit) => Some((lit.span, &lit.value)),
                Expression::TemplateLiteral(lit) => {
                    if lit.expressions.is_empty() && lit.quasis.len() == 1 {
                        Some((lit.span, &lit.quasis[0].value.raw))
                    } else {
                        None
                    }
                }
                _ => None,
            },
            MemberExpression::StaticMemberExpression(expr) => {
                Some((expr.property.span, &expr.property.name))
            }
            MemberExpression::PrivateFieldExpression(_) => None,
        }
    }

    pub fn through_optional_is_specific_member_access(&self, object: &str, property: &str) -> bool {
        let object_matches = match self.object().without_parenthesized() {
            Expression::ChainExpression(x) => match &x.expression {
                ChainElement::CallExpression(_) => false,
                ChainElement::MemberExpression(me) => {
                    me.object().without_parenthesized().is_specific_id(object)
                }
            },
            x => x.is_specific_id(object),
        };

        let property_matches = self.static_property_name().is_some_and(|p| p == property);

        object_matches && property_matches
    }

    /// Whether it is a static member access `object.property`
    pub fn is_specific_member_access(&self, object: &str, property: &str) -> bool {
        self.object().is_specific_id(object)
            && self.static_property_name().is_some_and(|p| p == property)
    }
}

/// `MemberExpression[?Yield, ?Await] [ Expression[+In, ?Yield, ?Await] ]`
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ComputedMemberExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub object: Expression<'a>,
    pub expression: Expression<'a>,
    pub optional: bool, // for optional chaining
}

/// `MemberExpression[?Yield, ?Await] . IdentifierName`
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct StaticMemberExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub object: Expression<'a>,
    pub property: IdentifierName<'a>,
    pub optional: bool, // for optional chaining
}

/// `MemberExpression[?Yield, ?Await] . PrivateIdentifier`
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct PrivateFieldExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub object: Expression<'a>,
    pub field: PrivateIdentifier<'a>,
    pub optional: bool, // for optional chaining
}

/// Call Expression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct CallExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub callee: Expression<'a>,
    pub arguments: Vec<'a, Argument<'a>>,
    pub optional: bool, // for optional chaining
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
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

    pub fn is_symbol_or_symbol_for_call(&self) -> bool {
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
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct NewExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub callee: Expression<'a>,
    pub arguments: Vec<'a, Argument<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// Meta Property `new.target` | `import.meta`
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct MetaProperty<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub meta: IdentifierName<'a>,
    pub property: IdentifierName<'a>,
}

/// Spread Element
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct SpreadElement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub argument: Expression<'a>,
}

/// Argument
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum Argument<'a> {
    SpreadElement(Box<'a, SpreadElement<'a>>),
    Expression(Expression<'a>),
}

impl Argument<'_> {
    pub fn is_spread(&self) -> bool {
        matches!(self, Self::SpreadElement(_))
    }
}

/// Update Expression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct UpdateExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub operator: UpdateOperator,
    pub prefix: bool,
    pub argument: SimpleAssignmentTarget<'a>,
}

/// Unary Expression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct UnaryExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub operator: UnaryOperator,
    pub argument: Expression<'a>,
}

/// Binary Expression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct BinaryExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub left: Expression<'a>,
    pub operator: BinaryOperator,
    pub right: Expression<'a>,
}

/// Private Identifier in Shift Expression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct PrivateInExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub left: PrivateIdentifier<'a>,
    pub operator: BinaryOperator, // BinaryOperator::In
    pub right: Expression<'a>,
}

/// Binary Logical Operators
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct LogicalExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub left: Expression<'a>,
    pub operator: LogicalOperator,
    pub right: Expression<'a>,
}

/// Conditional Expression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ConditionalExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub test: Expression<'a>,
    pub consequent: Expression<'a>,
    pub alternate: Expression<'a>,
}

/// Assignment Expression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct AssignmentExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub operator: AssignmentOperator,
    pub left: AssignmentTarget<'a>,
    pub right: Expression<'a>,
}

/// Destructuring Assignment
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum AssignmentTarget<'a> {
    // First discriminant 1, to allow discriminant for `AssignmentTargetMaybeDefault`'s other variant
    // to fit in the niche before this, so `AssignmentTargetMaybeDefault` can be 24 bytes, not 32
    #[cfg(feature = "raw")]
    SimpleAssignmentTarget(SimpleAssignmentTarget<'a>) = 1,
    #[cfg(not(feature = "raw"))]
    SimpleAssignmentTarget(SimpleAssignmentTarget<'a>),
    AssignmentTargetPattern(AssignmentTargetPattern<'a>),
}

impl<'a> AssignmentTarget<'a> {
    pub fn is_simple(&self) -> bool {
        matches!(self, Self::SimpleAssignmentTarget(_))
    }

    pub fn is_destructuring_pattern(&self) -> bool {
        matches!(self, Self::AssignmentTargetPattern(_))
    }

    pub fn is_identifier(&self) -> bool {
        matches!(
            self,
            Self::SimpleAssignmentTarget(SimpleAssignmentTarget::AssignmentTargetIdentifier(_))
        )
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum SimpleAssignmentTarget<'a> {
    AssignmentTargetIdentifier(Box<'a, IdentifierReference<'a>>),
    MemberAssignmentTarget(Box<'a, MemberExpression<'a>>),
    TSAsExpression(Box<'a, TSAsExpression<'a>>),
    TSSatisfiesExpression(Box<'a, TSSatisfiesExpression<'a>>),
    TSNonNullExpression(Box<'a, TSNonNullExpression<'a>>),
    TSTypeAssertion(Box<'a, TSTypeAssertion<'a>>),
}

impl<'a> SimpleAssignmentTarget<'a> {
    pub fn get_expression(&self) -> Option<&Expression<'a>> {
        match self {
            Self::TSAsExpression(expr) => Some(&expr.expression),
            Self::TSSatisfiesExpression(expr) => Some(&expr.expression),
            Self::TSNonNullExpression(expr) => Some(&expr.expression),
            Self::TSTypeAssertion(expr) => Some(&expr.expression),
            _ => None,
        }
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum AssignmentTargetPattern<'a> {
    ArrayAssignmentTarget(Box<'a, ArrayAssignmentTarget<'a>>),
    ObjectAssignmentTarget(Box<'a, ObjectAssignmentTarget<'a>>),
}

// See serializer in serialize.rs
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct ArrayAssignmentTarget<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    #[cfg_attr(
        feature = "serialize",
        tsify(type = "Array<AssignmentTargetMaybeDefault | AssignmentTargetRest | null>")
    )]
    pub elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub rest: Option<AssignmentTargetRest<'a>>,
    pub trailing_comma: Option<Span>,
}

impl<'a> ArrayAssignmentTarget<'a> {
    pub fn new_with_elements(
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
    ) -> Self {
        Self { span, elements, rest: None, trailing_comma: None }
    }
}

// See serializer in serialize.rs
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ObjectAssignmentTarget<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    #[cfg_attr(
        feature = "serialize",
        tsify(type = "Array<AssignmentTargetProperty | AssignmentTargetRest>")
    )]
    pub properties: Vec<'a, AssignmentTargetProperty<'a>>,
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub rest: Option<AssignmentTargetRest<'a>>,
}

impl<'a> ObjectAssignmentTarget<'a> {
    pub fn new_with_properties(
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
    ) -> Self {
        Self { span, properties, rest: None }
    }

    pub fn is_empty(&self) -> bool {
        self.properties.is_empty() && self.rest.is_none()
    }

    pub fn len(&self) -> usize {
        self.properties.len() + usize::from(self.rest.is_some())
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename = "RestElement"))]
pub struct AssignmentTargetRest<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(rename = "argument"))]
    pub target: AssignmentTarget<'a>,
}

// No `#[ast_node]` because don't want this type to be `repr(u8)` so that discriminants of `AssignmentTarget`
// can be folded into this, so this type is 24 bytes, not 32
#[derive(Debug, Hash)]
#[cfg_attr(feature = "raw", derive(Inspect))]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum AssignmentTargetMaybeDefault<'a> {
    AssignmentTarget(AssignmentTarget<'a>),
    // Compiler assigns this discrimant 0
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

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct AssignmentTargetWithDefault<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub binding: AssignmentTarget<'a>,
    pub init: Expression<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum AssignmentTargetProperty<'a> {
    AssignmentTargetPropertyIdentifier(Box<'a, AssignmentTargetPropertyIdentifier<'a>>),
    AssignmentTargetPropertyProperty(Box<'a, AssignmentTargetPropertyProperty<'a>>),
}

/// Assignment Property - Identifier Reference
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct AssignmentTargetPropertyIdentifier<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub binding: IdentifierReference<'a>,
    pub init: Option<Expression<'a>>,
}

/// Assignment Property - Property Name
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct AssignmentTargetPropertyProperty<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: PropertyKey<'a>,
    pub binding: AssignmentTargetMaybeDefault<'a>,
}

/// Sequence Expression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct SequenceExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expressions: Vec<'a, Expression<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct Super {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

/// Await Expression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct AwaitExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub argument: Expression<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ChainExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: ChainElement<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum ChainElement<'a> {
    CallExpression(Box<'a, CallExpression<'a>>),
    MemberExpression(Box<'a, MemberExpression<'a>>),
}

/// Parenthesized Expression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ParenthesizedExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

/// Statements
// No `#[ast_node]` because don't want this type to be `repr(u8)` so that discriminants of `Declaration`
// can be folded into this, so this type is 16 bytes, not 24
#[derive(Debug, Hash)]
#[cfg_attr(feature = "raw", derive(Inspect))]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum Statement<'a> {
    // Statements
    BlockStatement(Box<'a, BlockStatement<'a>>),
    BreakStatement(Box<'a, BreakStatement<'a>>),
    ContinueStatement(Box<'a, ContinueStatement<'a>>),
    DebuggerStatement(Box<'a, DebuggerStatement>),
    DoWhileStatement(Box<'a, DoWhileStatement<'a>>),
    EmptyStatement(Box<'a, EmptyStatement>),
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

impl<'a> Statement<'a> {
    pub fn is_iteration_statement(&self) -> bool {
        matches!(
            self,
            Statement::DoWhileStatement(_)
                | Statement::ForInStatement(_)
                | Statement::ForOfStatement(_)
                | Statement::ForStatement(_)
                | Statement::WhileStatement(_)
        )
    }
}

/// Directive Prologue
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct Directive<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: StringLiteral<'a>,
    /// A Use Strict Directive is an ExpressionStatement in a Directive Prologue whose StringLiteral is either of the exact code point sequences "use strict" or 'use strict'.
    /// A Use Strict Directive may not contain an EscapeSequence or LineContinuation.
    /// <https://tc39.es/ecma262/#sec-directive-prologues-and-the-use-strict-directive>
    pub directive: Atom<'a>,
}

/// Hashbang
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct Hashbang<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub value: Atom<'a>,
}

/// Block Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct BlockStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub body: Vec<'a, Statement<'a>>,
}

/// Declarations and the Variable Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum Declaration<'a> {
    // First discriminant 19, to allow discriminant for `Statement`'s 19 variants to all fit
    // in the niche before this, so `Statement` can be 16 bytes, not 24 bytes
    #[cfg(feature = "raw")]
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>) = 19,
    #[cfg(not(feature = "raw"))]
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>),
    FunctionDeclaration(Box<'a, Function<'a>>),
    ClassDeclaration(Box<'a, Class<'a>>),
    UsingDeclaration(Box<'a, UsingDeclaration<'a>>),

    TSTypeAliasDeclaration(Box<'a, TSTypeAliasDeclaration<'a>>),
    TSInterfaceDeclaration(Box<'a, TSInterfaceDeclaration<'a>>),
    TSEnumDeclaration(Box<'a, TSEnumDeclaration<'a>>),
    TSModuleDeclaration(Box<'a, TSModuleDeclaration<'a>>),
    TSImportEqualsDeclaration(Box<'a, TSImportEqualsDeclaration<'a>>),
}

impl<'a> Declaration<'a> {
    pub fn is_typescript_syntax(&self) -> bool {
        match self {
            Self::VariableDeclaration(decl) => decl.is_typescript_syntax(),
            Self::FunctionDeclaration(func) => func.is_typescript_syntax(),
            Self::ClassDeclaration(class) => class.is_typescript_syntax(),
            _ => true,
        }
    }
    pub fn modifiers(&self) -> Option<&Modifiers<'a>> {
        match self {
            Declaration::VariableDeclaration(decl) => Some(&decl.modifiers),
            Declaration::FunctionDeclaration(decl) => Some(&decl.modifiers),
            Declaration::ClassDeclaration(decl) => Some(&decl.modifiers),
            Declaration::TSEnumDeclaration(decl) => Some(&decl.modifiers),
            Declaration::TSTypeAliasDeclaration(decl) => Some(&decl.modifiers),
            Declaration::TSModuleDeclaration(decl) => Some(&decl.modifiers),
            Declaration::TSInterfaceDeclaration(decl) => Some(&decl.modifiers),
            _ => None,
        }
    }
}

/// Variable Declaration
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct VariableDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub kind: VariableDeclarationKind,
    pub declarations: Vec<'a, VariableDeclarator<'a>>,
    /// Valid Modifiers: `export`, `declare`
    pub modifiers: Modifiers<'a>,
}

impl<'a> VariableDeclaration<'a> {
    pub fn is_typescript_syntax(&self) -> bool {
        self.modifiers.contains(ModifierKind::Declare)
    }
}

#[ast_node]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum VariableDeclarationKind {
    Var,
    Const,
    Let,
}

impl VariableDeclarationKind {
    pub fn is_var(&self) -> bool {
        matches!(self, Self::Var)
    }

    pub fn is_const(&self) -> bool {
        matches!(self, Self::Const)
    }

    pub fn is_lexical(&self) -> bool {
        matches!(self, Self::Const | Self::Let)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Var => "var",
            Self::Const => "const",
            Self::Let => "let",
        }
    }
}

impl fmt::Display for VariableDeclarationKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self.as_str();
        write!(f, "{s}")
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct VariableDeclarator<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub kind: VariableDeclarationKind,
    pub id: BindingPattern<'a>,
    pub init: Option<Expression<'a>>,
    pub definite: bool,
}

/// Using Declaration
/// * <https://github.com/tc39/proposal-explicit-resource-management>
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct UsingDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub is_await: bool,
    #[cfg_attr(feature = "serde-impl", serde(default))]
    pub declarations: Vec<'a, VariableDeclarator<'a>>,
}

/// Empty Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct EmptyStatement {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

/// Expression Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ExpressionStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

/// If Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct IfStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub test: Expression<'a>,
    pub consequent: Statement<'a>,
    pub alternate: Option<Statement<'a>>,
}

/// Do-While Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct DoWhileStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub body: Statement<'a>,
    pub test: Expression<'a>,
}

/// While Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct WhileStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub test: Expression<'a>,
    pub body: Statement<'a>,
}

/// For Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ForStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub init: Option<ForStatementInit<'a>>,
    pub test: Option<Expression<'a>>,
    pub update: Option<Expression<'a>>,
    pub body: Statement<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum ForStatementInit<'a> {
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>),
    Expression(Expression<'a>),
    UsingDeclaration(Box<'a, UsingDeclaration<'a>>),
}

impl<'a> ForStatementInit<'a> {
    /// LexicalDeclaration[In, Yield, Await] :
    ///   LetOrConst BindingList[?In, ?Yield, ?Await] ;
    pub fn is_lexical_declaration(&self) -> bool {
        matches!(self, Self::VariableDeclaration(decl) if decl.kind.is_lexical())
    }

    pub fn expression(&self) -> Option<&Expression<'a>> {
        match self {
            Self::Expression(e) => Some(e),
            _ => None,
        }
    }
}

/// For-In Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ForInStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub left: ForStatementLeft<'a>,
    pub right: Expression<'a>,
    pub body: Statement<'a>,
}

/// For-Of Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ForOfStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub r#await: bool,
    pub left: ForStatementLeft<'a>,
    pub right: Expression<'a>,
    pub body: Statement<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum ForStatementLeft<'a> {
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>),
    AssignmentTarget(AssignmentTarget<'a>),
    UsingDeclaration(Box<'a, UsingDeclaration<'a>>),
}

impl<'a> ForStatementLeft<'a> {
    /// LexicalDeclaration[In, Yield, Await] :
    ///   LetOrConst BindingList[?In, ?Yield, ?Await] ;
    pub fn is_lexical_declaration(&self) -> bool {
        matches!(self, Self::VariableDeclaration(decl) if decl.kind.is_lexical())
    }
}

/// Continue Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ContinueStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub label: Option<LabelIdentifier<'a>>,
}

/// Break Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct BreakStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub label: Option<LabelIdentifier<'a>>,
}

/// Return Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ReturnStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub argument: Option<Expression<'a>>,
}

/// With Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct WithStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub object: Expression<'a>,
    pub body: Statement<'a>,
}

/// Switch Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct SwitchStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub discriminant: Expression<'a>,
    pub cases: Vec<'a, SwitchCase<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct SwitchCase<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
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
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct LabeledStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub label: LabelIdentifier<'a>,
    pub body: Statement<'a>,
}

/// Throw Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ThrowStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub argument: Expression<'a>,
}

/// Try Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TryStatement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub block: Box<'a, BlockStatement<'a>>,
    pub handler: Option<Box<'a, CatchClause<'a>>>,
    pub finalizer: Option<Box<'a, BlockStatement<'a>>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct CatchClause<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub param: Option<BindingPattern<'a>>,
    pub body: Box<'a, BlockStatement<'a>>,
}

/// Debugger Statement
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct DebuggerStatement {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

/// Destructuring Binding Patterns
/// * <https://tc39.es/ecma262/#prod-BindingPattern>
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
pub struct BindingPattern<'a> {
    // serde(flatten) the attributes because estree has no `BindingPattern`
    #[cfg_attr(feature = "serialize", serde(flatten))]
    #[cfg_attr(
        feature = "serialize",
        tsify(type = "(BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern)")
    )]
    pub kind: BindingPatternKind<'a>,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub optional: bool,
}

impl<'a> BindingPattern<'a> {
    pub fn new_with_kind(kind: BindingPatternKind<'a>) -> Self {
        Self { kind, type_annotation: None, optional: false }
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum BindingPatternKind<'a> {
    /// `const a = 1`
    BindingIdentifier(Box<'a, BindingIdentifier<'a>>),
    /// `const {a} = 1`
    ObjectPattern(Box<'a, ObjectPattern<'a>>),
    /// `const [a] = 1`
    ArrayPattern(Box<'a, ArrayPattern<'a>>),
    /// A defaulted binding pattern, i.e.:
    /// `const {a = 1} = 1`
    /// the assignment pattern is `a = 1`
    /// it has an inner left that has a BindingIdentifier
    AssignmentPattern(Box<'a, AssignmentPattern<'a>>),
}

impl<'a> BindingPatternKind<'a> {
    pub fn is_destructuring_pattern(&self) -> bool {
        matches!(self, Self::ObjectPattern(_) | Self::ArrayPattern(_))
    }

    pub fn is_binding_identifier(&self) -> bool {
        matches!(self, Self::BindingIdentifier(_))
    }

    pub fn is_assignment_pattern(&self) -> bool {
        matches!(self, Self::AssignmentPattern(_))
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct AssignmentPattern<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub left: BindingPattern<'a>,
    pub right: Expression<'a>,
}

// See serializer in serialize.rs
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ObjectPattern<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    #[cfg_attr(feature = "serialize", tsify(type = "Array<BindingProperty | BindingRestElement>"))]
    pub properties: Vec<'a, BindingProperty<'a>>,
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub rest: Option<Box<'a, BindingRestElement<'a>>>,
}

impl<'a> ObjectPattern<'a> {
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty() && self.rest.is_none()
    }

    pub fn len(&self) -> usize {
        self.properties.len() + usize::from(self.rest.is_some())
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct BindingProperty<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub key: PropertyKey<'a>,
    pub value: BindingPattern<'a>,
    pub shorthand: bool,
    pub computed: bool,
}

// See serializer in serialize.rs
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ArrayPattern<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    #[cfg_attr(
        feature = "serialize",
        tsify(type = "Array<BindingPattern | BindingRestElement | null>")
    )]
    pub elements: Vec<'a, Option<BindingPattern<'a>>>,
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub rest: Option<Box<'a, BindingRestElement<'a>>>,
}

impl<'a> ArrayPattern<'a> {
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty() && self.rest.is_none()
    }

    pub fn len(&self) -> usize {
        self.elements.len() + usize::from(self.rest.is_some())
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename = "RestElement"))]
pub struct BindingRestElement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub argument: BindingPattern<'a>,
}

/// Function Definitions
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
pub struct Function<'a> {
    pub r#type: FunctionType,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: Option<BindingIdentifier<'a>>,
    pub generator: bool,
    pub r#async: bool,
    /// Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    ///
    /// The JavaScript specification states that you cannot have a parameter called `this`,
    /// and so TypeScript uses that syntax space to let you declare the type for `this` in the function body.
    ///
    /// ```TypeScript
    /// interface DB {
    ///   filterUsers(filter: (this: User) => boolean): User[];
    /// }
    ///
    /// const db = getDB();
    /// const admins = db.filterUsers(function (this: User) {
    ///   return this.admin;
    /// });
    /// ```
    pub this_param: Option<TSThisParameter<'a>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub body: Option<Box<'a, FunctionBody<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    /// Valid modifiers: `export`, `default`, `async`
    pub modifiers: Modifiers<'a>,
}

impl<'a> Function<'a> {
    pub fn is_typescript_syntax(&self) -> bool {
        self.r#type == FunctionType::TSDeclareFunction
            || self.body.is_none()
            || self.modifiers.contains(ModifierKind::Declare)
    }

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

    pub fn is_strict(&self) -> bool {
        self.body.as_ref().is_some_and(|body| {
            body.directives.iter().any(|directive| directive.directive == "use strict")
        })
    }
}

#[ast_node]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum FunctionType {
    FunctionDeclaration,
    FunctionExpression,
    TSDeclareFunction,
    /// <https://github.com/typescript-eslint/typescript-eslint/pull/1289>
    TSEmptyBodyFunctionExpression,
}

/// <https://tc39.es/ecma262/#prod-FormalParameters>
// See serializer in serialize.rs
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct FormalParameters<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub kind: FormalParameterKind,
    #[cfg_attr(
        feature = "serialize",
        tsify(type = "Array<FormalParameter | FormalParameterRest>")
    )]
    pub items: Vec<'a, FormalParameter<'a>>,
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub rest: Option<Box<'a, BindingRestElement<'a>>>,
}

impl<'a> FormalParameters<'a> {
    pub fn parameters_count(&self) -> usize {
        self.items.len() + self.rest.as_ref().map_or(0, |_| 1)
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct FormalParameter<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub pattern: BindingPattern<'a>,
    pub accessibility: Option<TSAccessibility>,
    pub readonly: bool,
    pub r#override: bool,
    pub decorators: Vec<'a, Decorator<'a>>,
}

#[ast_node]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
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

impl FormalParameterKind {
    pub fn is_signature(&self) -> bool {
        matches!(self, Self::Signature)
    }
}

impl<'a> FormalParameters<'a> {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// <https://tc39.es/ecma262/#prod-FunctionBody>
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct FunctionBody<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub directives: Vec<'a, Directive<'a>>,
    pub statements: Vec<'a, Statement<'a>>,
}

impl<'a> FunctionBody<'a> {
    pub fn is_empty(&self) -> bool {
        self.directives.is_empty() && self.statements.is_empty()
    }
}

/// Arrow Function Definitions
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct ArrowFunctionExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    /// Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    pub expression: bool,
    pub r#async: bool,
    pub params: Box<'a, FormalParameters<'a>>,
    /// See `expression` for whether this arrow expression returns an expression.
    pub body: Box<'a, FunctionBody<'a>>,

    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

impl<'a> ArrowFunctionExpression<'a> {
    /// Get expression part of `ArrowFunctionExpression`: `() => expression_part`.
    pub fn get_expression(&self) -> Option<&Expression<'a>> {
        if self.expression {
            if let Statement::ExpressionStatement(expr_stmt) = &self.body.statements[0] {
                return Some(&expr_stmt.expression);
            }
        }
        None
    }
}

/// Generator Function Definitions
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct YieldExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub delegate: bool,
    pub argument: Option<Expression<'a>>,
}

/// Class Definitions
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
pub struct Class<'a> {
    pub r#type: ClassType,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: Option<BindingIdentifier<'a>>,
    pub super_class: Option<Expression<'a>>,
    pub body: Box<'a, ClassBody<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub super_type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    pub implements: Option<Vec<'a, Box<'a, TSClassImplements<'a>>>>,
    pub decorators: Vec<'a, Decorator<'a>>,
    /// Valid Modifiers: `export`, `abstract`
    pub modifiers: Modifiers<'a>,
}

impl<'a> Class<'a> {
    pub fn is_expression(&self) -> bool {
        self.r#type == ClassType::ClassExpression
    }

    pub fn is_declaration(&self) -> bool {
        self.r#type == ClassType::ClassDeclaration
    }

    pub fn is_declare(&self) -> bool {
        self.modifiers.contains(ModifierKind::Declare)
    }

    pub fn is_typescript_syntax(&self) -> bool {
        self.is_declare()
    }
}

#[ast_node]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum ClassType {
    ClassDeclaration,
    ClassExpression,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ClassBody<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub body: Vec<'a, ClassElement<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum ClassElement<'a> {
    StaticBlock(Box<'a, StaticBlock<'a>>),
    MethodDefinition(Box<'a, MethodDefinition<'a>>),
    PropertyDefinition(Box<'a, PropertyDefinition<'a>>),
    AccessorProperty(Box<'a, AccessorProperty<'a>>),
    TSIndexSignature(Box<'a, TSIndexSignature<'a>>),
}

impl<'a> ClassElement<'a> {
    pub fn r#static(&self) -> bool {
        match self {
            Self::TSIndexSignature(_) | Self::StaticBlock(_) => false,
            Self::MethodDefinition(def) => def.r#static,
            Self::PropertyDefinition(def) => def.r#static,
            Self::AccessorProperty(def) => def.r#static,
        }
    }

    pub fn computed(&self) -> bool {
        match self {
            Self::TSIndexSignature(_) | Self::StaticBlock(_) => false,
            Self::MethodDefinition(def) => def.computed,
            Self::PropertyDefinition(def) => def.computed,
            Self::AccessorProperty(def) => def.computed,
        }
    }

    pub fn accessibility(&self) -> Option<TSAccessibility> {
        match self {
            Self::StaticBlock(_) | Self::TSIndexSignature(_) | Self::AccessorProperty(_) => None,
            Self::MethodDefinition(def) => def.accessibility,
            Self::PropertyDefinition(def) => def.accessibility,
        }
    }

    pub fn method_definition_kind(&self) -> Option<MethodDefinitionKind> {
        match self {
            Self::TSIndexSignature(_)
            | Self::StaticBlock(_)
            | Self::PropertyDefinition(_)
            | Self::AccessorProperty(_) => None,
            Self::MethodDefinition(def) => Some(def.kind),
        }
    }

    pub fn property_key(&self) -> Option<&PropertyKey<'a>> {
        match self {
            Self::TSIndexSignature(_) | Self::StaticBlock(_) => None,
            Self::MethodDefinition(def) => Some(&def.key),
            Self::PropertyDefinition(def) => Some(&def.key),
            Self::AccessorProperty(def) => Some(&def.key),
        }
    }

    pub fn static_name(&self) -> Option<CompactStr> {
        match self {
            Self::TSIndexSignature(_) | Self::StaticBlock(_) => None,
            Self::MethodDefinition(def) => def.key.static_name(),
            Self::PropertyDefinition(def) => def.key.static_name(),
            Self::AccessorProperty(def) => def.key.static_name(),
        }
    }

    pub fn is_property(&self) -> bool {
        matches!(self, Self::PropertyDefinition(_) | Self::AccessorProperty(_))
    }

    pub fn is_ts_empty_body_function(&self) -> bool {
        match self {
            Self::PropertyDefinition(_)
            | Self::StaticBlock(_)
            | Self::AccessorProperty(_)
            | Self::TSIndexSignature(_) => false,
            Self::MethodDefinition(method) => method.value.body.is_none(),
        }
    }

    pub fn is_typescript_syntax(&self) -> bool {
        match self {
            Self::TSIndexSignature(_) => true,
            Self::MethodDefinition(method) => method.value.is_typescript_syntax(),
            Self::PropertyDefinition(property) => {
                property.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition
            }
            _ => false,
        }
    }

    pub fn has_decorator(&self) -> bool {
        match self {
            Self::MethodDefinition(method) => !method.decorators.is_empty(),
            Self::PropertyDefinition(property) => !property.decorators.is_empty(),
            Self::AccessorProperty(property) => !property.decorators.is_empty(),
            Self::StaticBlock(_) | Self::TSIndexSignature(_) => false,
        }
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
pub struct MethodDefinition<'a> {
    pub r#type: MethodDefinitionType,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub key: PropertyKey<'a>,
    pub value: Box<'a, Function<'a>>, // FunctionExpression
    pub kind: MethodDefinitionKind,
    pub computed: bool,
    pub r#static: bool,
    pub r#override: bool,
    pub optional: bool,
    pub accessibility: Option<TSAccessibility>,
    pub decorators: Vec<'a, Decorator<'a>>,
}

#[ast_node]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum MethodDefinitionType {
    MethodDefinition,
    TSAbstractMethodDefinition,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
pub struct PropertyDefinition<'a> {
    pub r#type: PropertyDefinitionType,
    #[cfg_attr(feature = "serialize", serde(flatten))]
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
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub accessibility: Option<TSAccessibility>,
    pub decorators: Vec<'a, Decorator<'a>>,
}

#[ast_node]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum PropertyDefinitionType {
    PropertyDefinition,
    TSAbstractPropertyDefinition,
}

#[ast_node]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum MethodDefinitionKind {
    Constructor,
    Method,
    Get,
    Set,
}

impl MethodDefinitionKind {
    pub fn is_constructor(&self) -> bool {
        matches!(self, Self::Constructor)
    }
    pub fn is_method(&self) -> bool {
        matches!(self, Self::Method)
    }
    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set)
    }
}

#[ast_node]
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct PrivateIdentifier<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: Atom<'a>,
}

impl<'a> PrivateIdentifier<'a> {
    pub fn new(span: Span, name: Atom<'a>) -> Self {
        Self { span, name }
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct StaticBlock<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub body: Vec<'a, Statement<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum ModuleDeclaration<'a> {
    /// import hello from './world.js';
    /// import * as t from './world.js';
    ImportDeclaration(Box<'a, ImportDeclaration<'a>>),
    /// export * as numbers from '../numbers.js'
    ExportAllDeclaration(Box<'a, ExportAllDeclaration<'a>>),
    /// export default 5;
    ExportDefaultDeclaration(Box<'a, ExportDefaultDeclaration<'a>>),
    /// export {five} from './numbers.js';
    /// export {six, seven};
    ExportNamedDeclaration(Box<'a, ExportNamedDeclaration<'a>>),

    /// export = 5;
    TSExportAssignment(Box<'a, TSExportAssignment<'a>>),
    /// export as namespace React;
    TSNamespaceExportDeclaration(Box<'a, TSNamespaceExportDeclaration<'a>>),
}

impl<'a> ModuleDeclaration<'a> {
    pub fn is_import(&self) -> bool {
        matches!(self, Self::ImportDeclaration(_))
    }

    pub fn is_export(&self) -> bool {
        matches!(
            self,
            Self::ExportAllDeclaration(_)
                | Self::ExportDefaultDeclaration(_)
                | Self::ExportNamedDeclaration(_)
                | Self::TSExportAssignment(_)
                | Self::TSNamespaceExportDeclaration(_)
        )
    }

    pub fn is_default_export(&self) -> bool {
        matches!(self, Self::ExportDefaultDeclaration(_))
    }

    pub fn source(&self) -> Option<&StringLiteral<'a>> {
        match self {
            Self::ImportDeclaration(decl) => Some(&decl.source),
            Self::ExportAllDeclaration(decl) => Some(&decl.source),
            Self::ExportNamedDeclaration(decl) => decl.source.as_ref(),
            Self::ExportDefaultDeclaration(_)
            | Self::TSExportAssignment(_)
            | Self::TSNamespaceExportDeclaration(_) => None,
        }
    }

    pub fn with_clause(&self) -> Option<&WithClause<'a>> {
        match self {
            Self::ImportDeclaration(decl) => decl.with_clause.as_ref(),
            Self::ExportAllDeclaration(decl) => decl.with_clause.as_ref(),
            Self::ExportNamedDeclaration(decl) => decl.with_clause.as_ref(),
            Self::ExportDefaultDeclaration(_)
            | Self::TSExportAssignment(_)
            | Self::TSNamespaceExportDeclaration(_) => None,
        }
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct AccessorProperty<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub key: PropertyKey<'a>,
    pub value: Option<Expression<'a>>,
    pub computed: bool,
    pub r#static: bool,
    pub decorators: Vec<'a, Decorator<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ImportExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub source: Expression<'a>,
    pub arguments: Vec<'a, Expression<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct ImportDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    /// `None` for `import 'foo'`, `Some([])` for `import {} from 'foo'`
    pub specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
    pub source: StringLiteral<'a>,
    /// Some(vec![]) for empty assertion
    pub with_clause: Option<WithClause<'a>>,
    /// `import type { foo } from 'bar'`
    pub import_kind: ImportOrExportKind,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum ImportDeclarationSpecifier<'a> {
    /// import {imported} from "source"
    /// import {imported as local} from "source"
    ImportSpecifier(ImportSpecifier<'a>),
    /// import local from "source"
    ImportDefaultSpecifier(ImportDefaultSpecifier<'a>),
    /// import * as local from "source"
    ImportNamespaceSpecifier(ImportNamespaceSpecifier<'a>),
}

// import {imported} from "source"
// import {imported as local} from "source"
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct ImportSpecifier<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub imported: ModuleExportName<'a>,
    pub local: BindingIdentifier<'a>,
    pub import_kind: ImportOrExportKind,
}

// import local from "source"
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ImportDefaultSpecifier<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub local: BindingIdentifier<'a>,
}

// import * as local from "source"
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ImportNamespaceSpecifier<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub local: BindingIdentifier<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct WithClause<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub attributes_keyword: IdentifierName<'a>, // `with` or `assert`
    pub with_entries: Vec<'a, ImportAttribute<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ImportAttribute<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub key: ImportAttributeKey<'a>,
    pub value: StringLiteral<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum ImportAttributeKey<'a> {
    Identifier(IdentifierName<'a>),
    StringLiteral(StringLiteral<'a>),
}

impl<'a> ImportAttributeKey<'a> {
    pub fn as_atom(&self) -> Atom<'a> {
        match self {
            Self::Identifier(identifier) => identifier.name.clone(),
            Self::StringLiteral(literal) => literal.value.clone(),
        }
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct ExportNamedDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub declaration: Option<Declaration<'a>>,
    pub specifiers: Vec<'a, ExportSpecifier<'a>>,
    pub source: Option<StringLiteral<'a>>,
    /// `export type { foo }`
    pub export_kind: ImportOrExportKind,
    /// Some(vec![]) for empty assertion
    pub with_clause: Option<WithClause<'a>>,
}

impl<'a> ExportNamedDeclaration<'a> {
    pub fn is_typescript_syntax(&self) -> bool {
        self.export_kind == ImportOrExportKind::Type
            || self.declaration.as_ref().map_or(false, Declaration::is_typescript_syntax)
    }
}

/// Export Default Declaration
/// export default HoistableDeclaration
/// export default ClassDeclaration
/// export default AssignmentExpression
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct ExportDefaultDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub declaration: ExportDefaultDeclarationKind<'a>,
    pub exported: ModuleExportName<'a>, // `default`
}

impl<'a> ExportDefaultDeclaration<'a> {
    pub fn is_typescript_syntax(&self) -> bool {
        self.declaration.is_typescript_syntax()
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct ExportAllDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub exported: Option<ModuleExportName<'a>>,
    pub source: StringLiteral<'a>,
    pub with_clause: Option<WithClause<'a>>, // Some(vec![]) for empty assertion
    pub export_kind: ImportOrExportKind,     // `export type *`
}

impl<'a> ExportAllDeclaration<'a> {
    pub fn is_typescript_syntax(&self) -> bool {
        self.export_kind.is_type()
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct ExportSpecifier<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub local: ModuleExportName<'a>,
    pub exported: ModuleExportName<'a>,
    pub export_kind: ImportOrExportKind, // `export type *`
}

impl<'a> ExportSpecifier<'a> {
    pub fn new(span: Span, local: ModuleExportName<'a>, exported: ModuleExportName<'a>) -> Self {
        Self { span, local, exported, export_kind: ImportOrExportKind::Value }
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum ExportDefaultDeclarationKind<'a> {
    Expression(Expression<'a>),
    FunctionDeclaration(Box<'a, Function<'a>>),
    ClassDeclaration(Box<'a, Class<'a>>),

    TSInterfaceDeclaration(Box<'a, TSInterfaceDeclaration<'a>>),
    TSEnumDeclaration(Box<'a, TSEnumDeclaration<'a>>),
}

impl<'a> ExportDefaultDeclarationKind<'a> {
    #[inline]
    pub fn is_typescript_syntax(&self) -> bool {
        match self {
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => func.is_typescript_syntax(),
            ExportDefaultDeclarationKind::ClassDeclaration(class) => class.is_typescript_syntax(),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(_)
            | ExportDefaultDeclarationKind::TSEnumDeclaration(_) => true,
            ExportDefaultDeclarationKind::Expression(_) => false,
        }
    }
}

/// Support:
///   * `import {"\0 any unicode" as foo} from ""`
///   * `export {foo as "\0 any unicode"}`
/// * es2022: <https://github.com/estree/estree/blob/master/es2022.md#modules>
/// * <https://github.com/tc39/ecma262/pull/2154>
#[ast_node]
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum ModuleExportName<'a> {
    Identifier(IdentifierName<'a>),
    StringLiteral(StringLiteral<'a>),
}

impl<'a> fmt::Display for ModuleExportName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Identifier(identifier) => identifier.name.to_string(),
            Self::StringLiteral(literal) => format!(r#""{}""#, literal.value),
        };
        write!(f, "{s}")
    }
}

impl<'a> ModuleExportName<'a> {
    pub fn name(&self) -> &Atom<'a> {
        match self {
            Self::Identifier(identifier) => &identifier.name,
            Self::StringLiteral(literal) => &literal.value,
        }
    }
}
