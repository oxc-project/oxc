use serde::Serialize;

use crate::react_compiler_ast::common::BaseNode;
use crate::react_compiler_ast::common::RawNode;
use crate::react_compiler_ast::expressions::{Expression, Identifier};

/// Covers assignment targets and patterns.
/// In Babel, LVal includes Identifier, MemberExpression, ObjectPattern, ArrayPattern,
/// RestElement, AssignmentPattern.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum PatternLike {
    Identifier(Identifier),
    ObjectPattern(ObjectPattern),
    ArrayPattern(ArrayPattern),
    AssignmentPattern(AssignmentPattern),
    RestElement(RestElement),
    // Expressions can appear in pattern positions (e.g., MemberExpression as LVal)
    MemberExpression(crate::react_compiler_ast::expressions::MemberExpression),
    TSAsExpression(crate::react_compiler_ast::expressions::TSAsExpression),
    TSSatisfiesExpression(crate::react_compiler_ast::expressions::TSSatisfiesExpression),
    TSNonNullExpression(crate::react_compiler_ast::expressions::TSNonNullExpression),
    TSTypeAssertion(crate::react_compiler_ast::expressions::TSTypeAssertion),
    // Flow's analogue of the TS cast wrappers: `(expr: SomeType)`.
    TypeCastExpression(crate::react_compiler_ast::expressions::TypeCastExpression),
}

impl PatternLike {
    /// Convert to the matching [`Expression`] variant when this pattern shares
    /// a node `type` with `Expression` (i.e. it can appear in expression
    /// position), otherwise `None`.
    ///
    /// Reproduces exactly the set that `serde_json::from_value::<Expression>`
    /// of the same node would accept: the eight variants below wrap the same
    /// inner types as their `Expression` counterparts (`AssignmentPattern`
    /// included — `Expression` carries it for error-recovery positions), while
    /// the pattern-only variants (`ObjectPattern`, `ArrayPattern`,
    /// `RestElement`) are not expressions and yield `None`.
    pub fn as_expression(&self) -> Option<Expression> {
        match self {
            PatternLike::Identifier(x) => Some(Expression::Identifier(x.clone())),
            PatternLike::MemberExpression(x) => Some(Expression::MemberExpression(x.clone())),
            PatternLike::AssignmentPattern(x) => Some(Expression::AssignmentPattern(x.clone())),
            PatternLike::TSAsExpression(x) => Some(Expression::TSAsExpression(x.clone())),
            PatternLike::TSSatisfiesExpression(x) => {
                Some(Expression::TSSatisfiesExpression(x.clone()))
            }
            PatternLike::TSNonNullExpression(x) => Some(Expression::TSNonNullExpression(x.clone())),
            PatternLike::TSTypeAssertion(x) => Some(Expression::TSTypeAssertion(x.clone())),
            PatternLike::TypeCastExpression(x) => Some(Expression::TypeCastExpression(x.clone())),
            PatternLike::ObjectPattern(_)
            | PatternLike::ArrayPattern(_)
            | PatternLike::RestElement(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectPattern {
    #[serde(flatten)]
    pub base: BaseNode,
    pub properties: Vec<ObjectPatternProperty>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeAnnotation")]
    pub type_annotation: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ObjectPatternProperty {
    ObjectProperty(ObjectPatternProp),
    RestElement(RestElement),
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectPatternProp {
    #[serde(flatten)]
    pub base: BaseNode,
    pub key: Box<Expression>,
    pub value: Box<PatternLike>,
    pub computed: bool,
    pub shorthand: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArrayPattern {
    #[serde(flatten)]
    pub base: BaseNode,
    pub elements: Vec<Option<PatternLike>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeAnnotation")]
    pub type_annotation: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssignmentPattern {
    #[serde(flatten)]
    pub base: BaseNode,
    pub left: Box<PatternLike>,
    pub right: Box<Expression>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeAnnotation")]
    pub type_annotation: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RestElement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub argument: Box<PatternLike>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeAnnotation")]
    pub type_annotation: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
}
