use std::{
    hash::{Hash, Hasher},
    ops::Range,
};

use serde::Serialize;

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

pub type Span = Range<usize>;

#[derive(Debug, Default, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct Node {
    pub start: usize,
    pub end: usize,
}

impl Node {
    #[must_use]
    #[inline]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub const fn range(&self) -> Span {
        self.start..self.end
    }
}

// #[allow(clippy::derive_hash_xor_eq)]
impl Hash for Node {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        // hash to nothing so all ast nodes can be comparible with hash
    }
}

pub trait GetNode {
    #[must_use]
    fn node(&self) -> Node;
}

impl<'a> GetNode for Statement<'a> {
    fn node(&self) -> Node {
        match self {
            Self::BlockStatement(stmt) => stmt.node,
            Self::BreakStatement(stmt) => stmt.node,
            Self::ContinueStatement(stmt) => stmt.node,
            Self::DebuggerStatement(stmt) => stmt.node,
            Self::DoWhileStatement(stmt) => stmt.node,
            Self::EmptyStatement(stmt) => stmt.node,
            Self::ExpressionStatement(stmt) => stmt.node,
            Self::ForInStatement(stmt) => stmt.node,
            Self::ForOfStatement(stmt) => stmt.node,
            Self::ForStatement(stmt) => stmt.node,
            Self::IfStatement(stmt) => stmt.node,
            Self::LabeledStatement(stmt) => stmt.node,
            Self::ReturnStatement(stmt) => stmt.node,
            Self::SwitchStatement(stmt) => stmt.node,
            Self::ThrowStatement(stmt) => stmt.node,
            Self::TryStatement(stmt) => stmt.node,
            Self::WhileStatement(stmt) => stmt.node,
            Self::WithStatement(stmt) => stmt.node,
            Self::ModuleDeclaration(decl) => decl.node,
            Self::Declaration(decl) => decl.node(),
        }
    }
}

impl<'a> GetNode for Expression<'a> {
    fn node(&self) -> Node {
        match self {
            Self::BooleanLiteral(e) => e.node,
            Self::NullLiteral(e) => e.node,
            Self::NumberLiteral(e) => e.node,
            Self::BigintLiteral(e) => e.node,
            Self::RegExpLiteral(e) => e.node,
            Self::StringLiteral(e) => e.node,
            Self::TemplateLiteral(e) => e.node,
            Self::Identifier(e) => e.node,
            Self::MetaProperty(e) => e.node,
            Self::Super(e) => e.node,
            Self::ArrayExpression(e) => e.node,
            Self::ArrowFunctionExpression(e) => e.node,
            Self::AssignmentExpression(e) => e.node,
            Self::AwaitExpression(e) => e.node,
            Self::BinaryExpression(e) => e.node,
            Self::PrivateInExpression(e) => e.node,
            Self::CallExpression(e) => e.node,
            Self::ChainExpression(e) => e.node,
            Self::ClassExpression(e) => e.node,
            Self::ConditionalExpression(e) => e.node,
            Self::FunctionExpression(e) => e.node,
            Self::ImportExpression(e) => e.node,
            Self::LogicalExpression(e) => e.node,
            Self::MemberExpression(e) => e.node(),
            Self::NewExpression(e) => e.node,
            Self::ObjectExpression(e) => e.node,
            Self::ParenthesizedExpression(e) => e.node,
            Self::SequenceExpression(e) => e.node,
            Self::TaggedTemplateExpression(e) => e.node,
            Self::ThisExpression(e) => e.node,
            Self::UnaryExpression(e) => e.node,
            Self::UpdateExpression(e) => e.node,
            Self::YieldExpression(e) => e.node,
            Self::JSXElement(e) => e.node,
            Self::JSXFragment(e) => e.node,
            Self::TSAsExpression(e) => e.node,
            Self::TSTypeAssertion(e) => e.node,
            Self::TSNonNullExpression(e) => e.node,
            Self::TSInstantiationExpression(e) => e.node,
        }
    }
}

impl<'a> GetNode for BindingPatternKind<'a> {
    fn node(&self) -> Node {
        match self {
            Self::BindingIdentifier(ident) => ident.node,
            Self::ObjectPattern(pat) => pat.node,
            Self::ArrayPattern(pat) => pat.node,
            Self::RestElement(elem) => elem.node,
            Self::AssignmentPattern(pat) => pat.node,
        }
    }
}

impl<'a> GetNode for BindingPattern<'a> {
    fn node(&self) -> Node {
        match &self.kind {
            BindingPatternKind::BindingIdentifier(ident) => ident.node,
            BindingPatternKind::ObjectPattern(pat) => pat.node,
            BindingPatternKind::ArrayPattern(pat) => pat.node,
            BindingPatternKind::RestElement(pat) => pat.node,
            BindingPatternKind::AssignmentPattern(pat) => pat.node,
        }
    }
}

impl<'a> GetNode for ClassElement<'a> {
    fn node(&self) -> Node {
        match self {
            Self::StaticBlock(block) => block.node,
            Self::MethodDefinition(def) => def.node,
            Self::PropertyDefinition(def) => def.node,
            Self::AccessorProperty(def) => def.node,
            Self::TSAbstractMethodDefinition(def) => def.method_definition.node,
            Self::TSAbstractPropertyDefinition(def) => def.property_definition.node,
            Self::TSIndexSignature(sig) => sig.node,
        }
    }
}

impl<'a> GetNode for PropertyKey<'a> {
    fn node(&self) -> Node {
        match self {
            Self::Identifier(ident) => ident.node,
            Self::PrivateIdentifier(ident) => ident.node,
            Self::Expression(expr) => expr.node(),
        }
    }
}

impl<'a> GetNode for MemberExpression<'a> {
    fn node(&self) -> Node {
        match self {
            Self::ComputedMemberExpression(expr) => expr.node,
            Self::StaticMemberExpression(expr) => expr.node,
            Self::PrivateFieldExpression(expr) => expr.node,
        }
    }
}

impl GetNode for ImportAttributeKey {
    fn node(&self) -> Node {
        match self {
            Self::Identifier(identifier) => identifier.node,
            Self::StringLiteral(literal) => literal.node,
        }
    }
}

impl GetNode for ModuleExportName {
    fn node(&self) -> Node {
        match self {
            Self::Identifier(identifier) => identifier.node,
            Self::StringLiteral(literal) => literal.node,
        }
    }
}

impl<'a> GetNode for Declaration<'a> {
    fn node(&self) -> Node {
        match self {
            Self::VariableDeclaration(decl) => decl.node,
            Self::FunctionDeclaration(decl) => decl.node,
            Self::ClassDeclaration(decl) => decl.node,
            Self::TSTypeAliasDeclaration(decl) => decl.node,
            Self::TSInterfaceDeclaration(decl) => decl.node,
            Self::TSEnumDeclaration(decl) => decl.node,
            Self::TSModuleDeclaration(decl) => decl.node,
            Self::TSImportEqualsDeclaration(decl) => decl.node,
        }
    }
}

impl GetNode for TSModuleDeclarationName {
    fn node(&self) -> Node {
        match self {
            Self::Identifier(ident) => ident.node,
            Self::StringLiteral(lit) => lit.node,
        }
    }
}

impl<'a> GetNode for ObjectProperty<'a> {
    fn node(&self) -> Node {
        match self {
            Self::Property(p) => p.node,
            Self::SpreadProperty(p) => p.node,
        }
    }
}

impl<'a> GetNode for ObjectPatternProperty<'a> {
    fn node(&self) -> Node {
        match self {
            Self::Property(p) => p.node,
            Self::RestElement(e) => e.node,
        }
    }
}

impl<'a> GetNode for AssignmentTarget<'a> {
    fn node(&self) -> Node {
        match self {
            Self::SimpleAssignmentTarget(SimpleAssignmentTarget::AssignmentTargetIdentifier(
                ident,
            )) => ident.node,
            Self::SimpleAssignmentTarget(SimpleAssignmentTarget::MemberAssignmentTarget(expr)) => {
                expr.node()
            }
            Self::SimpleAssignmentTarget(SimpleAssignmentTarget::TSAsExpression(expr)) => expr.node,
            Self::SimpleAssignmentTarget(SimpleAssignmentTarget::TSNonNullExpression(expr)) => {
                expr.node
            }
            Self::SimpleAssignmentTarget(SimpleAssignmentTarget::TSTypeAssertion(expr)) => {
                expr.node
            }
            Self::AssignmentTargetPattern(AssignmentTargetPattern::ArrayAssignmentTarget(pat)) => {
                pat.node
            }
            Self::AssignmentTargetPattern(AssignmentTargetPattern::ObjectAssignmentTarget(pat)) => {
                pat.node
            }
        }
    }
}

impl<'a> GetNode for PropertyValue<'a> {
    fn node(&self) -> Node {
        match self {
            Self::Pattern(pat) => pat.node(),
            Self::Expression(expr) => expr.node(),
        }
    }
}

impl<'a> GetNode for Argument<'a> {
    fn node(&self) -> Node {
        match self {
            Self::SpreadElement(e) => e.node,
            Self::Expression(expr) => expr.node(),
        }
    }
}

impl<'a> GetNode for ForStatementInit<'a> {
    fn node(&self) -> Node {
        match self {
            Self::VariableDeclaration(x) => x.node,
            Self::Expression(x) => x.node(),
        }
    }
}

impl<'a> GetNode for SimpleAssignmentTarget<'a> {
    fn node(&self) -> Node {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.node,
            Self::MemberAssignmentTarget(expr) => expr.node(),
            Self::TSAsExpression(expr) => expr.node,
            Self::TSNonNullExpression(expr) => expr.node,
            Self::TSTypeAssertion(expr) => expr.node,
        }
    }
}

impl<'a> GetNode for JSXElementName<'a> {
    fn node(&self) -> Node {
        match self {
            Self::Identifier(ident) => ident.node,
            Self::NamespacedName(name) => name.node,
            Self::MemberExpression(expr) => expr.node,
        }
    }
}

impl<'a> GetNode for TSSignature<'a> {
    fn node(&self) -> Node {
        match self {
            Self::TSIndexSignature(sig) => sig.node,
            Self::TSPropertySignature(sig) => sig.node,
            Self::TSCallSignatureDeclaration(decl) => decl.node,
            Self::TSConstructSignatureDeclaration(decl) => decl.node,
            Self::TSMethodSignature(sig) => sig.node,
        }
    }
}

impl<'a> GetNode for TSType<'a> {
    fn node(&self) -> Node {
        match self {
            Self::TSConditionalType(t) => t.node,
            Self::TSFunctionType(t) => t.node,
            Self::TSLiteralType(t) => t.node,
            Self::TSTypeReference(t) => t.node,
            Self::TSTypeQuery(t) => t.node,
            Self::TSUnionType(t) => t.node,
            Self::TSTupleType(t) => t.node,
            Self::TSArrayType(t) => t.node,
            Self::TSIntersectionType(t) => t.node,
            Self::TSMappedType(t) => t.node,
            Self::TSInferType(t) => t.node,
            Self::TSConstructorType(t) => t.node,
            Self::TSIndexedAccessType(t) => t.node,
            Self::TSTypeOperatorType(t) => t.node,
            Self::TSImportType(t) => t.node,
            Self::TSQualifiedName(t) => t.node,
            Self::TSTypePredicate(t) => t.node,
            Self::TSTypeLiteral(t) => t.node,
            Self::TSTemplateLiteralType(t) => t.node,
            Self::TSAnyKeyword(t) => t.node,
            Self::TSUnknownKeyword(t) => t.node,
            Self::TSUndefinedKeyword(t) => t.node,
            Self::TSNullKeyword(t) => t.node,
            Self::TSNumberKeyword(t) => t.node,
            Self::TSStringKeyword(t) => t.node,
            Self::TSNeverKeyword(t) => t.node,
            Self::TSBooleanKeyword(t) => t.node,
            Self::TSSymbolKeyword(t) => t.node,
            Self::TSBigIntKeyword(t) => t.node,
            Self::TSThisKeyword(t) => t.node,
            Self::TSVoidKeyword(t) => t.node,
            Self::TSObjectKeyword(t) => t.node,
            Self::JSDocNullableType(t) => t.node,
            Self::JSDocUnknownType(t) => t.node,
        }
    }
}

impl<'a> GetNode for ExportDefaultDeclarationKind<'a> {
    fn node(&self) -> Node {
        match self {
            Self::ClassDeclaration(x) => x.node,
            Self::Expression(x) => x.node(),
            Self::FunctionDeclaration(x) => x.node,
            Self::TSEnumDeclaration(x) => x.node,
            Self::TSInterfaceDeclaration(x) => x.node,
        }
    }
}
