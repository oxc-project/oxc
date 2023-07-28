use oxc_span::{GetSpan, Span};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

impl<'a> GetSpan for Statement<'a> {
    fn span(&self) -> Span {
        match self {
            Self::BlockStatement(stmt) => stmt.span,
            Self::BreakStatement(stmt) => stmt.span,
            Self::ContinueStatement(stmt) => stmt.span,
            Self::DebuggerStatement(stmt) => stmt.span,
            Self::DoWhileStatement(stmt) => stmt.span,
            Self::EmptyStatement(stmt) => stmt.span,
            Self::ExpressionStatement(stmt) => stmt.span,
            Self::ForInStatement(stmt) => stmt.span,
            Self::ForOfStatement(stmt) => stmt.span,
            Self::ForStatement(stmt) => stmt.span,
            Self::IfStatement(stmt) => stmt.span,
            Self::LabeledStatement(stmt) => stmt.span,
            Self::ReturnStatement(stmt) => stmt.span,
            Self::SwitchStatement(stmt) => stmt.span,
            Self::ThrowStatement(stmt) => stmt.span,
            Self::TryStatement(stmt) => stmt.span,
            Self::WhileStatement(stmt) => stmt.span,
            Self::WithStatement(stmt) => stmt.span,
            Self::ModuleDeclaration(decl) => decl.span(),
            Self::Declaration(decl) => decl.span(),
        }
    }
}

impl<'a> GetSpan for Expression<'a> {
    fn span(&self) -> Span {
        match self {
            Self::BooleanLiteral(e) => e.span,
            Self::NullLiteral(e) => e.span,
            Self::NumberLiteral(e) => e.span,
            Self::BigintLiteral(e) => e.span,
            Self::RegExpLiteral(e) => e.span,
            Self::StringLiteral(e) => e.span,
            Self::TemplateLiteral(e) => e.span,
            Self::Identifier(e) => e.span,
            Self::MetaProperty(e) => e.span,
            Self::Super(e) => e.span,
            Self::ArrayExpression(e) => e.span,
            Self::ArrowExpression(e) => e.span,
            Self::AssignmentExpression(e) => e.span,
            Self::AwaitExpression(e) => e.span,
            Self::BinaryExpression(e) => e.span,
            Self::PrivateInExpression(e) => e.span,
            Self::CallExpression(e) => e.span,
            Self::ChainExpression(e) => e.span,
            Self::ClassExpression(e) => e.span,
            Self::ConditionalExpression(e) => e.span,
            Self::FunctionExpression(e) => e.span,
            Self::ImportExpression(e) => e.span,
            Self::LogicalExpression(e) => e.span,
            Self::MemberExpression(e) => e.span(),
            Self::NewExpression(e) => e.span,
            Self::ObjectExpression(e) => e.span,
            Self::ParenthesizedExpression(e) => e.span,
            Self::SequenceExpression(e) => e.span,
            Self::TaggedTemplateExpression(e) => e.span,
            Self::ThisExpression(e) => e.span,
            Self::UnaryExpression(e) => e.span,
            Self::UpdateExpression(e) => e.span,
            Self::YieldExpression(e) => e.span,
            Self::JSXElement(e) => e.span,
            Self::JSXFragment(e) => e.span,
            Self::TSAsExpression(e) => e.span,
            Self::TSSatisfiesExpression(e) => e.span,
            Self::TSTypeAssertion(e) => e.span,
            Self::TSNonNullExpression(e) => e.span,
            Self::TSInstantiationExpression(e) => e.span,
        }
    }
}

impl<'a> GetSpan for BindingPatternKind<'a> {
    fn span(&self) -> Span {
        match self {
            Self::BindingIdentifier(ident) => ident.span,
            Self::ObjectPattern(pat) => pat.span,
            Self::ArrayPattern(pat) => pat.span,
            Self::AssignmentPattern(pat) => pat.span,
        }
    }
}

impl<'a> GetSpan for BindingPattern<'a> {
    fn span(&self) -> Span {
        match &self.kind {
            BindingPatternKind::BindingIdentifier(ident) => ident.span,
            BindingPatternKind::ObjectPattern(pat) => pat.span,
            BindingPatternKind::ArrayPattern(pat) => pat.span,
            BindingPatternKind::AssignmentPattern(pat) => pat.span,
        }
    }
}

impl<'a> GetSpan for ClassElement<'a> {
    fn span(&self) -> Span {
        match self {
            Self::StaticBlock(block) => block.span,
            Self::MethodDefinition(def) => def.span,
            Self::PropertyDefinition(def) => def.span,
            Self::AccessorProperty(def) => def.span,
            Self::TSAbstractMethodDefinition(def) => def.method_definition.span,
            Self::TSAbstractPropertyDefinition(def) => def.property_definition.span,
            Self::TSIndexSignature(sig) => sig.span,
        }
    }
}

impl<'a> GetSpan for PropertyKey<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(ident) => ident.span,
            Self::PrivateIdentifier(ident) => ident.span,
            Self::Expression(expr) => expr.span(),
        }
    }
}

impl<'a> GetSpan for MemberExpression<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ComputedMemberExpression(expr) => expr.span,
            Self::StaticMemberExpression(expr) => expr.span,
            Self::PrivateFieldExpression(expr) => expr.span,
        }
    }
}

impl GetSpan for ImportAttributeKey {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(identifier) => identifier.span,
            Self::StringLiteral(literal) => literal.span,
        }
    }
}

impl GetSpan for ModuleExportName {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(identifier) => identifier.span,
            Self::StringLiteral(literal) => literal.span,
        }
    }
}

impl<'a> GetSpan for ModuleDeclaration<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ImportDeclaration(decl) => decl.span,
            Self::ExportAllDeclaration(decl) => decl.span,
            Self::ExportDefaultDeclaration(decl) => decl.span,
            Self::ExportNamedDeclaration(decl) => decl.span,
            Self::TSExportAssignment(decl) => decl.span,
            Self::TSNamespaceExportDeclaration(decl) => decl.span,
        }
    }
}

impl<'a> GetSpan for Declaration<'a> {
    fn span(&self) -> Span {
        match self {
            Self::VariableDeclaration(decl) => decl.span,
            Self::FunctionDeclaration(decl) => decl.span,
            Self::ClassDeclaration(decl) => decl.span,
            Self::TSTypeAliasDeclaration(decl) => decl.span,
            Self::TSInterfaceDeclaration(decl) => decl.span,
            Self::TSEnumDeclaration(decl) => decl.span,
            Self::TSModuleDeclaration(decl) => decl.span,
            Self::TSImportEqualsDeclaration(decl) => decl.span,
        }
    }
}

impl GetSpan for TSModuleDeclarationName {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(ident) => ident.span,
            Self::StringLiteral(lit) => lit.span,
        }
    }
}

impl<'a> GetSpan for ObjectPropertyKind<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ObjectProperty(p) => p.span,
            Self::SpreadProperty(p) => p.span,
        }
    }
}

impl<'a> GetSpan for AssignmentTarget<'a> {
    fn span(&self) -> Span {
        match self {
            Self::SimpleAssignmentTarget(target) => target.span(),
            Self::AssignmentTargetPattern(AssignmentTargetPattern::ArrayAssignmentTarget(pat)) => {
                pat.span
            }
            Self::AssignmentTargetPattern(AssignmentTargetPattern::ObjectAssignmentTarget(pat)) => {
                pat.span
            }
        }
    }
}

impl<'a> GetSpan for Argument<'a> {
    fn span(&self) -> Span {
        match self {
            Self::SpreadElement(e) => e.span,
            Self::Expression(expr) => expr.span(),
        }
    }
}

impl<'a> GetSpan for ArrayExpressionElement<'a> {
    fn span(&self) -> Span {
        match self {
            Self::SpreadElement(e) => e.span,
            Self::Expression(expr) => expr.span(),
            Self::Elision(span) => *span,
        }
    }
}

impl<'a> GetSpan for ForStatementInit<'a> {
    fn span(&self) -> Span {
        match self {
            Self::VariableDeclaration(x) => x.span,
            Self::Expression(x) => x.span(),
        }
    }
}

impl<'a> GetSpan for ForStatementLeft<'a> {
    fn span(&self) -> Span {
        match self {
            Self::VariableDeclaration(x) => x.span,
            Self::AssignmentTarget(x) => x.span(),
        }
    }
}

impl<'a> GetSpan for SimpleAssignmentTarget<'a> {
    fn span(&self) -> Span {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.span,
            Self::MemberAssignmentTarget(expr) => expr.span(),
            Self::TSAsExpression(expr) => expr.span,
            Self::TSSatisfiesExpression(expr) => expr.span,
            Self::TSNonNullExpression(expr) => expr.span,
            Self::TSTypeAssertion(expr) => expr.span,
        }
    }
}

impl<'a> GetSpan for JSXElementName<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(ident) => ident.span,
            Self::NamespacedName(name) => name.span,
            Self::MemberExpression(expr) => expr.span,
        }
    }
}

impl<'a> GetSpan for TSSignature<'a> {
    fn span(&self) -> Span {
        match self {
            Self::TSIndexSignature(sig) => sig.span,
            Self::TSPropertySignature(sig) => sig.span,
            Self::TSCallSignatureDeclaration(decl) => decl.span,
            Self::TSConstructSignatureDeclaration(decl) => decl.span,
            Self::TSMethodSignature(sig) => sig.span,
        }
    }
}

impl<'a> GetSpan for TSType<'a> {
    fn span(&self) -> Span {
        match self {
            Self::TSConditionalType(t) => t.span,
            Self::TSFunctionType(t) => t.span,
            Self::TSLiteralType(t) => t.span,
            Self::TSTypeReference(t) => t.span,
            Self::TSTypeQuery(t) => t.span,
            Self::TSUnionType(t) => t.span,
            Self::TSTupleType(t) => t.span,
            Self::TSArrayType(t) => t.span,
            Self::TSIntersectionType(t) => t.span,
            Self::TSMappedType(t) => t.span,
            Self::TSInferType(t) => t.span,
            Self::TSConstructorType(t) => t.span,
            Self::TSIndexedAccessType(t) => t.span,
            Self::TSTypeOperatorType(t) => t.span,
            Self::TSImportType(t) => t.span,
            Self::TSQualifiedName(t) => t.span,
            Self::TSTypePredicate(t) => t.span,
            Self::TSTypeLiteral(t) => t.span,
            Self::TSTemplateLiteralType(t) => t.span,
            Self::TSAnyKeyword(t) => t.span,
            Self::TSUnknownKeyword(t) => t.span,
            Self::TSUndefinedKeyword(t) => t.span,
            Self::TSNullKeyword(t) => t.span,
            Self::TSNumberKeyword(t) => t.span,
            Self::TSStringKeyword(t) => t.span,
            Self::TSNeverKeyword(t) => t.span,
            Self::TSBooleanKeyword(t) => t.span,
            Self::TSSymbolKeyword(t) => t.span,
            Self::TSBigIntKeyword(t) => t.span,
            Self::TSThisKeyword(t) => t.span,
            Self::TSVoidKeyword(t) => t.span,
            Self::TSObjectKeyword(t) => t.span,
            Self::JSDocNullableType(t) => t.span,
            Self::JSDocUnknownType(t) => t.span,
        }
    }
}

impl<'a> GetSpan for ExportDefaultDeclarationKind<'a> {
    fn span(&self) -> Span {
        match self {
            Self::ClassDeclaration(x) => x.span,
            Self::Expression(x) => x.span(),
            Self::FunctionDeclaration(x) => x.span,
            Self::TSEnumDeclaration(x) => x.span,
            Self::TSInterfaceDeclaration(x) => x.span,
        }
    }
}

impl GetSpan for ImportDeclarationSpecifier {
    fn span(&self) -> Span {
        match self {
            Self::ImportSpecifier(specifier) => specifier.span,
            Self::ImportDefaultSpecifier(specifier) => specifier.span,
            Self::ImportNamespaceSpecifier(specifier) => specifier.span,
        }
    }
}

impl<'a> GetSpan for JSXChild<'a> {
    fn span(&self) -> Span {
        match self {
            Self::Element(element) => element.span,
            Self::ExpressionContainer(container) => container.span,
            Self::Text(text) => text.span,
            Self::Fragment(fragment) => fragment.span,
            Self::Spread(spread) => spread.span,
        }
    }
}

impl<'a> GetSpan for AssignmentTargetPattern<'a> {
    fn span(&self) -> Span {
        match &self {
            Self::ArrayAssignmentTarget(x) => x.span,
            Self::ObjectAssignmentTarget(x) => x.span,
        }
    }
}

impl<'a> GetSpan for JSXAttributeItem<'a> {
    fn span(&self) -> Span {
        match &self {
            JSXAttributeItem::Attribute(attr) => attr.span,
            JSXAttributeItem::SpreadAttribute(attr) => attr.span,
        }
    }
}

impl<'a> GetSpan for JSXExpression<'a> {
    fn span(&self) -> Span {
        match &self {
            JSXExpression::Expression(expr) => expr.span(),
            JSXExpression::EmptyExpression(exmpty_expr) => exmpty_expr.span,
        }
    }
}
