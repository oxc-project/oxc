// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter/ast_nodes.rs`.

use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_formatter_core::Format;
use oxc_span::GetSpan;
use oxc_str::{Ident, JSStr};
use oxc_syntax::node::NodeId;

use crate::ast_nodes::{AsAstNodes, AstNode, Following, list_following, node_address};
use crate::formatter::{
    JsFormatter,
    trivia::{format_leading_comments, format_trailing_comments},
};

/// A typed view of an AST node with a kind, the entries of the formatter's ancestor stack.
#[derive(Clone, Copy)]
pub enum AstNodes<'a> {
    Dummy(),
    Program(AstNode<'a, Program<'a>>),
    IdentifierName(AstNode<'a, IdentifierName<'a>>),
    IdentifierReference(AstNode<'a, IdentifierReference<'a>>),
    BindingIdentifier(AstNode<'a, BindingIdentifier<'a>>),
    LabelIdentifier(AstNode<'a, LabelIdentifier<'a>>),
    ThisExpression(AstNode<'a, ThisExpression>),
    ArrayExpression(AstNode<'a, ArrayExpression<'a>>),
    Elision(AstNode<'a, Elision>),
    ObjectExpression(AstNode<'a, ObjectExpression<'a>>),
    ObjectProperty(AstNode<'a, ObjectProperty<'a>>),
    TemplateLiteral(AstNode<'a, TemplateLiteral<'a>>),
    TaggedTemplateExpression(AstNode<'a, TaggedTemplateExpression<'a>>),
    TemplateElement(AstNode<'a, TemplateElement<'a>>),
    ComputedMemberExpression(AstNode<'a, ComputedMemberExpression<'a>>),
    StaticMemberExpression(AstNode<'a, StaticMemberExpression<'a>>),
    PrivateFieldExpression(AstNode<'a, PrivateFieldExpression<'a>>),
    CallExpression(AstNode<'a, CallExpression<'a>>),
    NewExpression(AstNode<'a, NewExpression<'a>>),
    ImportMeta(AstNode<'a, ImportMeta>),
    NewTarget(AstNode<'a, NewTarget>),
    SpreadElement(AstNode<'a, SpreadElement<'a>>),
    UpdateExpression(AstNode<'a, UpdateExpression<'a>>),
    UnaryExpression(AstNode<'a, UnaryExpression<'a>>),
    BinaryExpression(AstNode<'a, BinaryExpression<'a>>),
    PrivateInExpression(AstNode<'a, PrivateInExpression<'a>>),
    LogicalExpression(AstNode<'a, LogicalExpression<'a>>),
    ConditionalExpression(AstNode<'a, ConditionalExpression<'a>>),
    AssignmentExpression(AstNode<'a, AssignmentExpression<'a>>),
    ArrayAssignmentTarget(AstNode<'a, ArrayAssignmentTarget<'a>>),
    ObjectAssignmentTarget(AstNode<'a, ObjectAssignmentTarget<'a>>),
    AssignmentTargetRest(AstNode<'a, AssignmentTargetRest<'a>>),
    AssignmentTargetWithDefault(AstNode<'a, AssignmentTargetWithDefault<'a>>),
    AssignmentTargetPropertyIdentifier(AstNode<'a, AssignmentTargetPropertyIdentifier<'a>>),
    AssignmentTargetPropertyProperty(AstNode<'a, AssignmentTargetPropertyProperty<'a>>),
    SequenceExpression(AstNode<'a, SequenceExpression<'a>>),
    Super(AstNode<'a, Super>),
    AwaitExpression(AstNode<'a, AwaitExpression<'a>>),
    ChainExpression(AstNode<'a, ChainExpression<'a>>),
    ParenthesizedExpression(AstNode<'a, ParenthesizedExpression<'a>>),
    Directive(AstNode<'a, Directive<'a>>),
    Hashbang(AstNode<'a, Hashbang<'a>>),
    BlockStatement(AstNode<'a, BlockStatement<'a>>),
    VariableDeclaration(AstNode<'a, VariableDeclaration<'a>>),
    VariableDeclarator(AstNode<'a, VariableDeclarator<'a>>),
    EmptyStatement(AstNode<'a, EmptyStatement>),
    ExpressionStatement(AstNode<'a, ExpressionStatement<'a>>),
    IfStatement(AstNode<'a, IfStatement<'a>>),
    DoWhileStatement(AstNode<'a, DoWhileStatement<'a>>),
    WhileStatement(AstNode<'a, WhileStatement<'a>>),
    ForStatement(AstNode<'a, ForStatement<'a>>),
    ForInStatement(AstNode<'a, ForInStatement<'a>>),
    ForOfStatement(AstNode<'a, ForOfStatement<'a>>),
    ContinueStatement(AstNode<'a, ContinueStatement<'a>>),
    BreakStatement(AstNode<'a, BreakStatement<'a>>),
    ReturnStatement(AstNode<'a, ReturnStatement<'a>>),
    WithStatement(AstNode<'a, WithStatement<'a>>),
    SwitchStatement(AstNode<'a, SwitchStatement<'a>>),
    SwitchCase(AstNode<'a, SwitchCase<'a>>),
    LabeledStatement(AstNode<'a, LabeledStatement<'a>>),
    ThrowStatement(AstNode<'a, ThrowStatement<'a>>),
    TryStatement(AstNode<'a, TryStatement<'a>>),
    CatchClause(AstNode<'a, CatchClause<'a>>),
    CatchParameter(AstNode<'a, CatchParameter<'a>>),
    DebuggerStatement(AstNode<'a, DebuggerStatement>),
    AssignmentPattern(AstNode<'a, AssignmentPattern<'a>>),
    ObjectPattern(AstNode<'a, ObjectPattern<'a>>),
    BindingProperty(AstNode<'a, BindingProperty<'a>>),
    ArrayPattern(AstNode<'a, ArrayPattern<'a>>),
    BindingRestElement(AstNode<'a, BindingRestElement<'a>>),
    Function(AstNode<'a, Function<'a>>),
    FormalParameters(AstNode<'a, FormalParameters<'a>>),
    FormalParameter(AstNode<'a, FormalParameter<'a>>),
    FormalParameterRest(AstNode<'a, FormalParameterRest<'a>>),
    FunctionBody(AstNode<'a, FunctionBody<'a>>),
    ArrowFunctionExpression(AstNode<'a, ArrowFunctionExpression<'a>>),
    YieldExpression(AstNode<'a, YieldExpression<'a>>),
    Class(AstNode<'a, Class<'a>>),
    ClassBody(AstNode<'a, ClassBody<'a>>),
    MethodDefinition(AstNode<'a, MethodDefinition<'a>>),
    PropertyDefinition(AstNode<'a, PropertyDefinition<'a>>),
    PrivateIdentifier(AstNode<'a, PrivateIdentifier<'a>>),
    StaticBlock(AstNode<'a, StaticBlock<'a>>),
    AccessorProperty(AstNode<'a, AccessorProperty<'a>>),
    ImportExpression(AstNode<'a, ImportExpression<'a>>),
    ImportDeclaration(AstNode<'a, ImportDeclaration<'a>>),
    ImportSpecifier(AstNode<'a, ImportSpecifier<'a>>),
    ImportDefaultSpecifier(AstNode<'a, ImportDefaultSpecifier<'a>>),
    ImportNamespaceSpecifier(AstNode<'a, ImportNamespaceSpecifier<'a>>),
    WithClause(AstNode<'a, WithClause<'a>>),
    ImportAttribute(AstNode<'a, ImportAttribute<'a>>),
    ExportDeclaration(AstNode<'a, ExportDeclaration<'a>>),
    ExportNamedDeclaration(AstNode<'a, ExportNamedDeclaration<'a>>),
    ExportFromDeclaration(AstNode<'a, ExportFromDeclaration<'a>>),
    ExportDefaultDeclaration(AstNode<'a, ExportDefaultDeclaration<'a>>),
    ExportAllDeclaration(AstNode<'a, ExportAllDeclaration<'a>>),
    ExportSpecifier(AstNode<'a, ExportSpecifier<'a>>),
    V8IntrinsicExpression(AstNode<'a, V8IntrinsicExpression<'a>>),
    BooleanLiteral(AstNode<'a, BooleanLiteral>),
    NullLiteral(AstNode<'a, NullLiteral>),
    NumericLiteral(AstNode<'a, NumericLiteral<'a>>),
    StringLiteral(AstNode<'a, StringLiteral<'a>>),
    BigIntLiteral(AstNode<'a, BigIntLiteral<'a>>),
    RegExpLiteral(AstNode<'a, RegExpLiteral<'a>>),
    JSXElement(AstNode<'a, JSXElement<'a>>),
    JSXOpeningElement(AstNode<'a, JSXOpeningElement<'a>>),
    JSXClosingElement(AstNode<'a, JSXClosingElement<'a>>),
    JSXFragment(AstNode<'a, JSXFragment<'a>>),
    JSXOpeningFragment(AstNode<'a, JSXOpeningFragment>),
    JSXClosingFragment(AstNode<'a, JSXClosingFragment>),
    JSXNamespacedName(AstNode<'a, JSXNamespacedName<'a>>),
    JSXMemberExpression(AstNode<'a, JSXMemberExpression<'a>>),
    JSXExpressionContainer(AstNode<'a, JSXExpressionContainer<'a>>),
    JSXEmptyExpression(AstNode<'a, JSXEmptyExpression>),
    JSXAttribute(AstNode<'a, JSXAttribute<'a>>),
    JSXSpreadAttribute(AstNode<'a, JSXSpreadAttribute<'a>>),
    JSXIdentifier(AstNode<'a, JSXIdentifier<'a>>),
    JSXSpreadChild(AstNode<'a, JSXSpreadChild<'a>>),
    JSXText(AstNode<'a, JSXText<'a>>),
    TSThisParameter(AstNode<'a, TSThisParameter<'a>>),
    TSEnumDeclaration(AstNode<'a, TSEnumDeclaration<'a>>),
    TSEnumBody(AstNode<'a, TSEnumBody<'a>>),
    TSEnumMember(AstNode<'a, TSEnumMember<'a>>),
    TSTypeAnnotation(AstNode<'a, TSTypeAnnotation<'a>>),
    TSLiteralType(AstNode<'a, TSLiteralType<'a>>),
    TSConditionalType(AstNode<'a, TSConditionalType<'a>>),
    TSUnionType(AstNode<'a, TSUnionType<'a>>),
    TSIntersectionType(AstNode<'a, TSIntersectionType<'a>>),
    TSParenthesizedType(AstNode<'a, TSParenthesizedType<'a>>),
    TSTypeOperator(AstNode<'a, TSTypeOperator<'a>>),
    TSArrayType(AstNode<'a, TSArrayType<'a>>),
    TSIndexedAccessType(AstNode<'a, TSIndexedAccessType<'a>>),
    TSTupleType(AstNode<'a, TSTupleType<'a>>),
    TSNamedTupleMember(AstNode<'a, TSNamedTupleMember<'a>>),
    TSOptionalType(AstNode<'a, TSOptionalType<'a>>),
    TSRestType(AstNode<'a, TSRestType<'a>>),
    TSAnyKeyword(AstNode<'a, TSAnyKeyword>),
    TSStringKeyword(AstNode<'a, TSStringKeyword>),
    TSBooleanKeyword(AstNode<'a, TSBooleanKeyword>),
    TSNumberKeyword(AstNode<'a, TSNumberKeyword>),
    TSNeverKeyword(AstNode<'a, TSNeverKeyword>),
    TSIntrinsicKeyword(AstNode<'a, TSIntrinsicKeyword>),
    TSUnknownKeyword(AstNode<'a, TSUnknownKeyword>),
    TSNullKeyword(AstNode<'a, TSNullKeyword>),
    TSUndefinedKeyword(AstNode<'a, TSUndefinedKeyword>),
    TSVoidKeyword(AstNode<'a, TSVoidKeyword>),
    TSSymbolKeyword(AstNode<'a, TSSymbolKeyword>),
    TSThisType(AstNode<'a, TSThisType>),
    TSObjectKeyword(AstNode<'a, TSObjectKeyword>),
    TSBigIntKeyword(AstNode<'a, TSBigIntKeyword>),
    TSTypeReference(AstNode<'a, TSTypeReference<'a>>),
    TSQualifiedName(AstNode<'a, TSQualifiedName<'a>>),
    TSTypeParameterInstantiation(AstNode<'a, TSTypeParameterInstantiation<'a>>),
    TSTypeParameter(AstNode<'a, TSTypeParameter<'a>>),
    TSTypeParameterDeclaration(AstNode<'a, TSTypeParameterDeclaration<'a>>),
    TSTypeAliasDeclaration(AstNode<'a, TSTypeAliasDeclaration<'a>>),
    TSClassImplements(AstNode<'a, TSClassImplements<'a>>),
    TSInterfaceDeclaration(AstNode<'a, TSInterfaceDeclaration<'a>>),
    TSInterfaceBody(AstNode<'a, TSInterfaceBody<'a>>),
    TSPropertySignature(AstNode<'a, TSPropertySignature<'a>>),
    TSIndexSignature(AstNode<'a, TSIndexSignature<'a>>),
    TSCallSignatureDeclaration(AstNode<'a, TSCallSignatureDeclaration<'a>>),
    TSMethodSignature(AstNode<'a, TSMethodSignature<'a>>),
    TSConstructSignatureDeclaration(AstNode<'a, TSConstructSignatureDeclaration<'a>>),
    TSIndexSignatureName(AstNode<'a, TSIndexSignatureName<'a>>),
    TSInterfaceHeritage(AstNode<'a, TSInterfaceHeritage<'a>>),
    TSTypePredicate(AstNode<'a, TSTypePredicate<'a>>),
    TSExternalModuleDeclaration(AstNode<'a, TSExternalModuleDeclaration<'a>>),
    TSNamespaceDeclaration(AstNode<'a, TSNamespaceDeclaration<'a>>),
    TSGlobalDeclaration(AstNode<'a, TSGlobalDeclaration<'a>>),
    TSModuleBlock(AstNode<'a, TSModuleBlock<'a>>),
    TSTypeLiteral(AstNode<'a, TSTypeLiteral<'a>>),
    TSInferType(AstNode<'a, TSInferType<'a>>),
    TSTypeQuery(AstNode<'a, TSTypeQuery<'a>>),
    TSImportType(AstNode<'a, TSImportType<'a>>),
    TSImportTypeQualifiedName(AstNode<'a, TSImportTypeQualifiedName<'a>>),
    TSFunctionType(AstNode<'a, TSFunctionType<'a>>),
    TSConstructorType(AstNode<'a, TSConstructorType<'a>>),
    TSMappedType(AstNode<'a, TSMappedType<'a>>),
    TSTemplateLiteralType(AstNode<'a, TSTemplateLiteralType<'a>>),
    TSAsExpression(AstNode<'a, TSAsExpression<'a>>),
    TSSatisfiesExpression(AstNode<'a, TSSatisfiesExpression<'a>>),
    TSTypeAssertion(AstNode<'a, TSTypeAssertion<'a>>),
    TSImportEqualsDeclaration(AstNode<'a, TSImportEqualsDeclaration<'a>>),
    TSExternalModuleReference(AstNode<'a, TSExternalModuleReference<'a>>),
    TSNonNullExpression(AstNode<'a, TSNonNullExpression<'a>>),
    Decorator(AstNode<'a, Decorator<'a>>),
    TSExportAssignment(AstNode<'a, TSExportAssignment<'a>>),
    TSNamespaceExportDeclaration(AstNode<'a, TSNamespaceExportDeclaration<'a>>),
    TSInstantiationExpression(AstNode<'a, TSInstantiationExpression<'a>>),
    JSDocNullableType(AstNode<'a, JSDocNullableType<'a>>),
    JSDocNonNullableType(AstNode<'a, JSDocNonNullableType<'a>>),
    JSDocUnknownType(AstNode<'a, JSDocUnknownType>),
}
impl AstNodes<'_> {
    /// Returns the span of this AST node.
    ///
    /// # Panics
    ///
    /// Panics when called on a `Dummy` node, which should never appear in a real AST.
    #[inline]
    pub fn span(&self) -> Span {
        match self {
            Self::Dummy() => panic!("Should never be called on a dummy node"),
            Self::Program(n) => n.span(),
            Self::IdentifierName(n) => n.span(),
            Self::IdentifierReference(n) => n.span(),
            Self::BindingIdentifier(n) => n.span(),
            Self::LabelIdentifier(n) => n.span(),
            Self::ThisExpression(n) => n.span(),
            Self::ArrayExpression(n) => n.span(),
            Self::Elision(n) => n.span(),
            Self::ObjectExpression(n) => n.span(),
            Self::ObjectProperty(n) => n.span(),
            Self::TemplateLiteral(n) => n.span(),
            Self::TaggedTemplateExpression(n) => n.span(),
            Self::TemplateElement(n) => n.span(),
            Self::ComputedMemberExpression(n) => n.span(),
            Self::StaticMemberExpression(n) => n.span(),
            Self::PrivateFieldExpression(n) => n.span(),
            Self::CallExpression(n) => n.span(),
            Self::NewExpression(n) => n.span(),
            Self::ImportMeta(n) => n.span(),
            Self::NewTarget(n) => n.span(),
            Self::SpreadElement(n) => n.span(),
            Self::UpdateExpression(n) => n.span(),
            Self::UnaryExpression(n) => n.span(),
            Self::BinaryExpression(n) => n.span(),
            Self::PrivateInExpression(n) => n.span(),
            Self::LogicalExpression(n) => n.span(),
            Self::ConditionalExpression(n) => n.span(),
            Self::AssignmentExpression(n) => n.span(),
            Self::ArrayAssignmentTarget(n) => n.span(),
            Self::ObjectAssignmentTarget(n) => n.span(),
            Self::AssignmentTargetRest(n) => n.span(),
            Self::AssignmentTargetWithDefault(n) => n.span(),
            Self::AssignmentTargetPropertyIdentifier(n) => n.span(),
            Self::AssignmentTargetPropertyProperty(n) => n.span(),
            Self::SequenceExpression(n) => n.span(),
            Self::Super(n) => n.span(),
            Self::AwaitExpression(n) => n.span(),
            Self::ChainExpression(n) => n.span(),
            Self::ParenthesizedExpression(n) => n.span(),
            Self::Directive(n) => n.span(),
            Self::Hashbang(n) => n.span(),
            Self::BlockStatement(n) => n.span(),
            Self::VariableDeclaration(n) => n.span(),
            Self::VariableDeclarator(n) => n.span(),
            Self::EmptyStatement(n) => n.span(),
            Self::ExpressionStatement(n) => n.span(),
            Self::IfStatement(n) => n.span(),
            Self::DoWhileStatement(n) => n.span(),
            Self::WhileStatement(n) => n.span(),
            Self::ForStatement(n) => n.span(),
            Self::ForInStatement(n) => n.span(),
            Self::ForOfStatement(n) => n.span(),
            Self::ContinueStatement(n) => n.span(),
            Self::BreakStatement(n) => n.span(),
            Self::ReturnStatement(n) => n.span(),
            Self::WithStatement(n) => n.span(),
            Self::SwitchStatement(n) => n.span(),
            Self::SwitchCase(n) => n.span(),
            Self::LabeledStatement(n) => n.span(),
            Self::ThrowStatement(n) => n.span(),
            Self::TryStatement(n) => n.span(),
            Self::CatchClause(n) => n.span(),
            Self::CatchParameter(n) => n.span(),
            Self::DebuggerStatement(n) => n.span(),
            Self::AssignmentPattern(n) => n.span(),
            Self::ObjectPattern(n) => n.span(),
            Self::BindingProperty(n) => n.span(),
            Self::ArrayPattern(n) => n.span(),
            Self::BindingRestElement(n) => n.span(),
            Self::Function(n) => n.span(),
            Self::FormalParameters(n) => n.span(),
            Self::FormalParameter(n) => n.span(),
            Self::FormalParameterRest(n) => n.span(),
            Self::FunctionBody(n) => n.span(),
            Self::ArrowFunctionExpression(n) => n.span(),
            Self::YieldExpression(n) => n.span(),
            Self::Class(n) => n.span(),
            Self::ClassBody(n) => n.span(),
            Self::MethodDefinition(n) => n.span(),
            Self::PropertyDefinition(n) => n.span(),
            Self::PrivateIdentifier(n) => n.span(),
            Self::StaticBlock(n) => n.span(),
            Self::AccessorProperty(n) => n.span(),
            Self::ImportExpression(n) => n.span(),
            Self::ImportDeclaration(n) => n.span(),
            Self::ImportSpecifier(n) => n.span(),
            Self::ImportDefaultSpecifier(n) => n.span(),
            Self::ImportNamespaceSpecifier(n) => n.span(),
            Self::WithClause(n) => n.span(),
            Self::ImportAttribute(n) => n.span(),
            Self::ExportDeclaration(n) => n.span(),
            Self::ExportNamedDeclaration(n) => n.span(),
            Self::ExportFromDeclaration(n) => n.span(),
            Self::ExportDefaultDeclaration(n) => n.span(),
            Self::ExportAllDeclaration(n) => n.span(),
            Self::ExportSpecifier(n) => n.span(),
            Self::V8IntrinsicExpression(n) => n.span(),
            Self::BooleanLiteral(n) => n.span(),
            Self::NullLiteral(n) => n.span(),
            Self::NumericLiteral(n) => n.span(),
            Self::StringLiteral(n) => n.span(),
            Self::BigIntLiteral(n) => n.span(),
            Self::RegExpLiteral(n) => n.span(),
            Self::JSXElement(n) => n.span(),
            Self::JSXOpeningElement(n) => n.span(),
            Self::JSXClosingElement(n) => n.span(),
            Self::JSXFragment(n) => n.span(),
            Self::JSXOpeningFragment(n) => n.span(),
            Self::JSXClosingFragment(n) => n.span(),
            Self::JSXNamespacedName(n) => n.span(),
            Self::JSXMemberExpression(n) => n.span(),
            Self::JSXExpressionContainer(n) => n.span(),
            Self::JSXEmptyExpression(n) => n.span(),
            Self::JSXAttribute(n) => n.span(),
            Self::JSXSpreadAttribute(n) => n.span(),
            Self::JSXIdentifier(n) => n.span(),
            Self::JSXSpreadChild(n) => n.span(),
            Self::JSXText(n) => n.span(),
            Self::TSThisParameter(n) => n.span(),
            Self::TSEnumDeclaration(n) => n.span(),
            Self::TSEnumBody(n) => n.span(),
            Self::TSEnumMember(n) => n.span(),
            Self::TSTypeAnnotation(n) => n.span(),
            Self::TSLiteralType(n) => n.span(),
            Self::TSConditionalType(n) => n.span(),
            Self::TSUnionType(n) => n.span(),
            Self::TSIntersectionType(n) => n.span(),
            Self::TSParenthesizedType(n) => n.span(),
            Self::TSTypeOperator(n) => n.span(),
            Self::TSArrayType(n) => n.span(),
            Self::TSIndexedAccessType(n) => n.span(),
            Self::TSTupleType(n) => n.span(),
            Self::TSNamedTupleMember(n) => n.span(),
            Self::TSOptionalType(n) => n.span(),
            Self::TSRestType(n) => n.span(),
            Self::TSAnyKeyword(n) => n.span(),
            Self::TSStringKeyword(n) => n.span(),
            Self::TSBooleanKeyword(n) => n.span(),
            Self::TSNumberKeyword(n) => n.span(),
            Self::TSNeverKeyword(n) => n.span(),
            Self::TSIntrinsicKeyword(n) => n.span(),
            Self::TSUnknownKeyword(n) => n.span(),
            Self::TSNullKeyword(n) => n.span(),
            Self::TSUndefinedKeyword(n) => n.span(),
            Self::TSVoidKeyword(n) => n.span(),
            Self::TSSymbolKeyword(n) => n.span(),
            Self::TSThisType(n) => n.span(),
            Self::TSObjectKeyword(n) => n.span(),
            Self::TSBigIntKeyword(n) => n.span(),
            Self::TSTypeReference(n) => n.span(),
            Self::TSQualifiedName(n) => n.span(),
            Self::TSTypeParameterInstantiation(n) => n.span(),
            Self::TSTypeParameter(n) => n.span(),
            Self::TSTypeParameterDeclaration(n) => n.span(),
            Self::TSTypeAliasDeclaration(n) => n.span(),
            Self::TSClassImplements(n) => n.span(),
            Self::TSInterfaceDeclaration(n) => n.span(),
            Self::TSInterfaceBody(n) => n.span(),
            Self::TSPropertySignature(n) => n.span(),
            Self::TSIndexSignature(n) => n.span(),
            Self::TSCallSignatureDeclaration(n) => n.span(),
            Self::TSMethodSignature(n) => n.span(),
            Self::TSConstructSignatureDeclaration(n) => n.span(),
            Self::TSIndexSignatureName(n) => n.span(),
            Self::TSInterfaceHeritage(n) => n.span(),
            Self::TSTypePredicate(n) => n.span(),
            Self::TSExternalModuleDeclaration(n) => n.span(),
            Self::TSNamespaceDeclaration(n) => n.span(),
            Self::TSGlobalDeclaration(n) => n.span(),
            Self::TSModuleBlock(n) => n.span(),
            Self::TSTypeLiteral(n) => n.span(),
            Self::TSInferType(n) => n.span(),
            Self::TSTypeQuery(n) => n.span(),
            Self::TSImportType(n) => n.span(),
            Self::TSImportTypeQualifiedName(n) => n.span(),
            Self::TSFunctionType(n) => n.span(),
            Self::TSConstructorType(n) => n.span(),
            Self::TSMappedType(n) => n.span(),
            Self::TSTemplateLiteralType(n) => n.span(),
            Self::TSAsExpression(n) => n.span(),
            Self::TSSatisfiesExpression(n) => n.span(),
            Self::TSTypeAssertion(n) => n.span(),
            Self::TSImportEqualsDeclaration(n) => n.span(),
            Self::TSExternalModuleReference(n) => n.span(),
            Self::TSNonNullExpression(n) => n.span(),
            Self::Decorator(n) => n.span(),
            Self::TSExportAssignment(n) => n.span(),
            Self::TSNamespaceExportDeclaration(n) => n.span(),
            Self::TSInstantiationExpression(n) => n.span(),
            Self::JSDocNullableType(n) => n.span(),
            Self::JSDocNonNullableType(n) => n.span(),
            Self::JSDocUnknownType(n) => n.span(),
        }
    }

    /// The address of the AST node, its identity together with the variant (0 for `Dummy`).
    #[inline]
    pub fn address(&self) -> usize {
        match self {
            Self::Dummy() => 0,
            Self::Program(n) => n.address(),
            Self::IdentifierName(n) => n.address(),
            Self::IdentifierReference(n) => n.address(),
            Self::BindingIdentifier(n) => n.address(),
            Self::LabelIdentifier(n) => n.address(),
            Self::ThisExpression(n) => n.address(),
            Self::ArrayExpression(n) => n.address(),
            Self::Elision(n) => n.address(),
            Self::ObjectExpression(n) => n.address(),
            Self::ObjectProperty(n) => n.address(),
            Self::TemplateLiteral(n) => n.address(),
            Self::TaggedTemplateExpression(n) => n.address(),
            Self::TemplateElement(n) => n.address(),
            Self::ComputedMemberExpression(n) => n.address(),
            Self::StaticMemberExpression(n) => n.address(),
            Self::PrivateFieldExpression(n) => n.address(),
            Self::CallExpression(n) => n.address(),
            Self::NewExpression(n) => n.address(),
            Self::ImportMeta(n) => n.address(),
            Self::NewTarget(n) => n.address(),
            Self::SpreadElement(n) => n.address(),
            Self::UpdateExpression(n) => n.address(),
            Self::UnaryExpression(n) => n.address(),
            Self::BinaryExpression(n) => n.address(),
            Self::PrivateInExpression(n) => n.address(),
            Self::LogicalExpression(n) => n.address(),
            Self::ConditionalExpression(n) => n.address(),
            Self::AssignmentExpression(n) => n.address(),
            Self::ArrayAssignmentTarget(n) => n.address(),
            Self::ObjectAssignmentTarget(n) => n.address(),
            Self::AssignmentTargetRest(n) => n.address(),
            Self::AssignmentTargetWithDefault(n) => n.address(),
            Self::AssignmentTargetPropertyIdentifier(n) => n.address(),
            Self::AssignmentTargetPropertyProperty(n) => n.address(),
            Self::SequenceExpression(n) => n.address(),
            Self::Super(n) => n.address(),
            Self::AwaitExpression(n) => n.address(),
            Self::ChainExpression(n) => n.address(),
            Self::ParenthesizedExpression(n) => n.address(),
            Self::Directive(n) => n.address(),
            Self::Hashbang(n) => n.address(),
            Self::BlockStatement(n) => n.address(),
            Self::VariableDeclaration(n) => n.address(),
            Self::VariableDeclarator(n) => n.address(),
            Self::EmptyStatement(n) => n.address(),
            Self::ExpressionStatement(n) => n.address(),
            Self::IfStatement(n) => n.address(),
            Self::DoWhileStatement(n) => n.address(),
            Self::WhileStatement(n) => n.address(),
            Self::ForStatement(n) => n.address(),
            Self::ForInStatement(n) => n.address(),
            Self::ForOfStatement(n) => n.address(),
            Self::ContinueStatement(n) => n.address(),
            Self::BreakStatement(n) => n.address(),
            Self::ReturnStatement(n) => n.address(),
            Self::WithStatement(n) => n.address(),
            Self::SwitchStatement(n) => n.address(),
            Self::SwitchCase(n) => n.address(),
            Self::LabeledStatement(n) => n.address(),
            Self::ThrowStatement(n) => n.address(),
            Self::TryStatement(n) => n.address(),
            Self::CatchClause(n) => n.address(),
            Self::CatchParameter(n) => n.address(),
            Self::DebuggerStatement(n) => n.address(),
            Self::AssignmentPattern(n) => n.address(),
            Self::ObjectPattern(n) => n.address(),
            Self::BindingProperty(n) => n.address(),
            Self::ArrayPattern(n) => n.address(),
            Self::BindingRestElement(n) => n.address(),
            Self::Function(n) => n.address(),
            Self::FormalParameters(n) => n.address(),
            Self::FormalParameter(n) => n.address(),
            Self::FormalParameterRest(n) => n.address(),
            Self::FunctionBody(n) => n.address(),
            Self::ArrowFunctionExpression(n) => n.address(),
            Self::YieldExpression(n) => n.address(),
            Self::Class(n) => n.address(),
            Self::ClassBody(n) => n.address(),
            Self::MethodDefinition(n) => n.address(),
            Self::PropertyDefinition(n) => n.address(),
            Self::PrivateIdentifier(n) => n.address(),
            Self::StaticBlock(n) => n.address(),
            Self::AccessorProperty(n) => n.address(),
            Self::ImportExpression(n) => n.address(),
            Self::ImportDeclaration(n) => n.address(),
            Self::ImportSpecifier(n) => n.address(),
            Self::ImportDefaultSpecifier(n) => n.address(),
            Self::ImportNamespaceSpecifier(n) => n.address(),
            Self::WithClause(n) => n.address(),
            Self::ImportAttribute(n) => n.address(),
            Self::ExportDeclaration(n) => n.address(),
            Self::ExportNamedDeclaration(n) => n.address(),
            Self::ExportFromDeclaration(n) => n.address(),
            Self::ExportDefaultDeclaration(n) => n.address(),
            Self::ExportAllDeclaration(n) => n.address(),
            Self::ExportSpecifier(n) => n.address(),
            Self::V8IntrinsicExpression(n) => n.address(),
            Self::BooleanLiteral(n) => n.address(),
            Self::NullLiteral(n) => n.address(),
            Self::NumericLiteral(n) => n.address(),
            Self::StringLiteral(n) => n.address(),
            Self::BigIntLiteral(n) => n.address(),
            Self::RegExpLiteral(n) => n.address(),
            Self::JSXElement(n) => n.address(),
            Self::JSXOpeningElement(n) => n.address(),
            Self::JSXClosingElement(n) => n.address(),
            Self::JSXFragment(n) => n.address(),
            Self::JSXOpeningFragment(n) => n.address(),
            Self::JSXClosingFragment(n) => n.address(),
            Self::JSXNamespacedName(n) => n.address(),
            Self::JSXMemberExpression(n) => n.address(),
            Self::JSXExpressionContainer(n) => n.address(),
            Self::JSXEmptyExpression(n) => n.address(),
            Self::JSXAttribute(n) => n.address(),
            Self::JSXSpreadAttribute(n) => n.address(),
            Self::JSXIdentifier(n) => n.address(),
            Self::JSXSpreadChild(n) => n.address(),
            Self::JSXText(n) => n.address(),
            Self::TSThisParameter(n) => n.address(),
            Self::TSEnumDeclaration(n) => n.address(),
            Self::TSEnumBody(n) => n.address(),
            Self::TSEnumMember(n) => n.address(),
            Self::TSTypeAnnotation(n) => n.address(),
            Self::TSLiteralType(n) => n.address(),
            Self::TSConditionalType(n) => n.address(),
            Self::TSUnionType(n) => n.address(),
            Self::TSIntersectionType(n) => n.address(),
            Self::TSParenthesizedType(n) => n.address(),
            Self::TSTypeOperator(n) => n.address(),
            Self::TSArrayType(n) => n.address(),
            Self::TSIndexedAccessType(n) => n.address(),
            Self::TSTupleType(n) => n.address(),
            Self::TSNamedTupleMember(n) => n.address(),
            Self::TSOptionalType(n) => n.address(),
            Self::TSRestType(n) => n.address(),
            Self::TSAnyKeyword(n) => n.address(),
            Self::TSStringKeyword(n) => n.address(),
            Self::TSBooleanKeyword(n) => n.address(),
            Self::TSNumberKeyword(n) => n.address(),
            Self::TSNeverKeyword(n) => n.address(),
            Self::TSIntrinsicKeyword(n) => n.address(),
            Self::TSUnknownKeyword(n) => n.address(),
            Self::TSNullKeyword(n) => n.address(),
            Self::TSUndefinedKeyword(n) => n.address(),
            Self::TSVoidKeyword(n) => n.address(),
            Self::TSSymbolKeyword(n) => n.address(),
            Self::TSThisType(n) => n.address(),
            Self::TSObjectKeyword(n) => n.address(),
            Self::TSBigIntKeyword(n) => n.address(),
            Self::TSTypeReference(n) => n.address(),
            Self::TSQualifiedName(n) => n.address(),
            Self::TSTypeParameterInstantiation(n) => n.address(),
            Self::TSTypeParameter(n) => n.address(),
            Self::TSTypeParameterDeclaration(n) => n.address(),
            Self::TSTypeAliasDeclaration(n) => n.address(),
            Self::TSClassImplements(n) => n.address(),
            Self::TSInterfaceDeclaration(n) => n.address(),
            Self::TSInterfaceBody(n) => n.address(),
            Self::TSPropertySignature(n) => n.address(),
            Self::TSIndexSignature(n) => n.address(),
            Self::TSCallSignatureDeclaration(n) => n.address(),
            Self::TSMethodSignature(n) => n.address(),
            Self::TSConstructSignatureDeclaration(n) => n.address(),
            Self::TSIndexSignatureName(n) => n.address(),
            Self::TSInterfaceHeritage(n) => n.address(),
            Self::TSTypePredicate(n) => n.address(),
            Self::TSExternalModuleDeclaration(n) => n.address(),
            Self::TSNamespaceDeclaration(n) => n.address(),
            Self::TSGlobalDeclaration(n) => n.address(),
            Self::TSModuleBlock(n) => n.address(),
            Self::TSTypeLiteral(n) => n.address(),
            Self::TSInferType(n) => n.address(),
            Self::TSTypeQuery(n) => n.address(),
            Self::TSImportType(n) => n.address(),
            Self::TSImportTypeQualifiedName(n) => n.address(),
            Self::TSFunctionType(n) => n.address(),
            Self::TSConstructorType(n) => n.address(),
            Self::TSMappedType(n) => n.address(),
            Self::TSTemplateLiteralType(n) => n.address(),
            Self::TSAsExpression(n) => n.address(),
            Self::TSSatisfiesExpression(n) => n.address(),
            Self::TSTypeAssertion(n) => n.address(),
            Self::TSImportEqualsDeclaration(n) => n.address(),
            Self::TSExternalModuleReference(n) => n.address(),
            Self::TSNonNullExpression(n) => n.address(),
            Self::Decorator(n) => n.address(),
            Self::TSExportAssignment(n) => n.address(),
            Self::TSNamespaceExportDeclaration(n) => n.address(),
            Self::TSInstantiationExpression(n) => n.address(),
            Self::JSDocNullableType(n) => n.address(),
            Self::JSDocNonNullableType(n) => n.address(),
            Self::JSDocUnknownType(n) => n.address(),
        }
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`) if this node is its parent.
    /// `Dummy` is the parent of the root, which has no following node.
    pub fn following_of_child(&self, child: usize, child_start: u32) -> Option<Following> {
        match self {
            Self::Dummy() => Some(Following::Start(0)),
            Self::Program(n) => n.following_of_child(child, child_start),
            Self::ArrayExpression(n) => n.following_of_child(child, child_start),
            Self::ObjectExpression(n) => n.following_of_child(child, child_start),
            Self::ObjectProperty(n) => n.following_of_child(child, child_start),
            Self::TemplateLiteral(n) => n.following_of_child(child, child_start),
            Self::TaggedTemplateExpression(n) => n.following_of_child(child, child_start),
            Self::ComputedMemberExpression(n) => n.following_of_child(child, child_start),
            Self::StaticMemberExpression(n) => n.following_of_child(child, child_start),
            Self::PrivateFieldExpression(n) => n.following_of_child(child, child_start),
            Self::CallExpression(n) => n.following_of_child(child, child_start),
            Self::NewExpression(n) => n.following_of_child(child, child_start),
            Self::SpreadElement(n) => n.following_of_child(child, child_start),
            Self::UpdateExpression(n) => n.following_of_child(child, child_start),
            Self::UnaryExpression(n) => n.following_of_child(child, child_start),
            Self::BinaryExpression(n) => n.following_of_child(child, child_start),
            Self::PrivateInExpression(n) => n.following_of_child(child, child_start),
            Self::LogicalExpression(n) => n.following_of_child(child, child_start),
            Self::ConditionalExpression(n) => n.following_of_child(child, child_start),
            Self::AssignmentExpression(n) => n.following_of_child(child, child_start),
            Self::ArrayAssignmentTarget(n) => n.following_of_child(child, child_start),
            Self::ObjectAssignmentTarget(n) => n.following_of_child(child, child_start),
            Self::AssignmentTargetRest(n) => n.following_of_child(child, child_start),
            Self::AssignmentTargetWithDefault(n) => n.following_of_child(child, child_start),
            Self::AssignmentTargetPropertyIdentifier(n) => n.following_of_child(child, child_start),
            Self::AssignmentTargetPropertyProperty(n) => n.following_of_child(child, child_start),
            Self::SequenceExpression(n) => n.following_of_child(child, child_start),
            Self::AwaitExpression(n) => n.following_of_child(child, child_start),
            Self::ChainExpression(n) => n.following_of_child(child, child_start),
            Self::ParenthesizedExpression(n) => n.following_of_child(child, child_start),
            Self::Directive(n) => n.following_of_child(child, child_start),
            Self::BlockStatement(n) => n.following_of_child(child, child_start),
            Self::VariableDeclaration(n) => n.following_of_child(child, child_start),
            Self::VariableDeclarator(n) => n.following_of_child(child, child_start),
            Self::ExpressionStatement(n) => n.following_of_child(child, child_start),
            Self::IfStatement(n) => n.following_of_child(child, child_start),
            Self::DoWhileStatement(n) => n.following_of_child(child, child_start),
            Self::WhileStatement(n) => n.following_of_child(child, child_start),
            Self::ForStatement(n) => n.following_of_child(child, child_start),
            Self::ForInStatement(n) => n.following_of_child(child, child_start),
            Self::ForOfStatement(n) => n.following_of_child(child, child_start),
            Self::ContinueStatement(n) => n.following_of_child(child, child_start),
            Self::BreakStatement(n) => n.following_of_child(child, child_start),
            Self::ReturnStatement(n) => n.following_of_child(child, child_start),
            Self::WithStatement(n) => n.following_of_child(child, child_start),
            Self::SwitchStatement(n) => n.following_of_child(child, child_start),
            Self::SwitchCase(n) => n.following_of_child(child, child_start),
            Self::LabeledStatement(n) => n.following_of_child(child, child_start),
            Self::ThrowStatement(n) => n.following_of_child(child, child_start),
            Self::TryStatement(n) => n.following_of_child(child, child_start),
            Self::CatchClause(n) => n.following_of_child(child, child_start),
            Self::CatchParameter(n) => n.following_of_child(child, child_start),
            Self::AssignmentPattern(n) => n.following_of_child(child, child_start),
            Self::ObjectPattern(n) => n.following_of_child(child, child_start),
            Self::BindingProperty(n) => n.following_of_child(child, child_start),
            Self::ArrayPattern(n) => n.following_of_child(child, child_start),
            Self::BindingRestElement(n) => n.following_of_child(child, child_start),
            Self::Function(n) => n.following_of_child(child, child_start),
            Self::FormalParameters(n) => n.following_of_child(child, child_start),
            Self::FormalParameter(n) => n.following_of_child(child, child_start),
            Self::FormalParameterRest(n) => n.following_of_child(child, child_start),
            Self::FunctionBody(n) => n.following_of_child(child, child_start),
            Self::ArrowFunctionExpression(n) => n.following_of_child(child, child_start),
            Self::YieldExpression(n) => n.following_of_child(child, child_start),
            Self::Class(n) => n.following_of_child(child, child_start),
            Self::ClassBody(n) => n.following_of_child(child, child_start),
            Self::MethodDefinition(n) => n.following_of_child(child, child_start),
            Self::PropertyDefinition(n) => n.following_of_child(child, child_start),
            Self::StaticBlock(n) => n.following_of_child(child, child_start),
            Self::AccessorProperty(n) => n.following_of_child(child, child_start),
            Self::ImportExpression(n) => n.following_of_child(child, child_start),
            Self::ImportDeclaration(n) => n.following_of_child(child, child_start),
            Self::ImportSpecifier(n) => n.following_of_child(child, child_start),
            Self::ImportDefaultSpecifier(n) => n.following_of_child(child, child_start),
            Self::ImportNamespaceSpecifier(n) => n.following_of_child(child, child_start),
            Self::WithClause(n) => n.following_of_child(child, child_start),
            Self::ImportAttribute(n) => n.following_of_child(child, child_start),
            Self::ExportDeclaration(n) => n.following_of_child(child, child_start),
            Self::ExportNamedDeclaration(n) => n.following_of_child(child, child_start),
            Self::ExportFromDeclaration(n) => n.following_of_child(child, child_start),
            Self::ExportDefaultDeclaration(n) => n.following_of_child(child, child_start),
            Self::ExportAllDeclaration(n) => n.following_of_child(child, child_start),
            Self::ExportSpecifier(n) => n.following_of_child(child, child_start),
            Self::V8IntrinsicExpression(n) => n.following_of_child(child, child_start),
            Self::JSXElement(n) => n.following_of_child(child, child_start),
            Self::JSXOpeningElement(n) => n.following_of_child(child, child_start),
            Self::JSXClosingElement(n) => n.following_of_child(child, child_start),
            Self::JSXFragment(n) => n.following_of_child(child, child_start),
            Self::JSXNamespacedName(n) => n.following_of_child(child, child_start),
            Self::JSXMemberExpression(n) => n.following_of_child(child, child_start),
            Self::JSXExpressionContainer(n) => n.following_of_child(child, child_start),
            Self::JSXAttribute(n) => n.following_of_child(child, child_start),
            Self::JSXSpreadAttribute(n) => n.following_of_child(child, child_start),
            Self::JSXSpreadChild(n) => n.following_of_child(child, child_start),
            Self::TSThisParameter(n) => n.following_of_child(child, child_start),
            Self::TSEnumDeclaration(n) => n.following_of_child(child, child_start),
            Self::TSEnumBody(n) => n.following_of_child(child, child_start),
            Self::TSEnumMember(n) => n.following_of_child(child, child_start),
            Self::TSTypeAnnotation(n) => n.following_of_child(child, child_start),
            Self::TSLiteralType(n) => n.following_of_child(child, child_start),
            Self::TSConditionalType(n) => n.following_of_child(child, child_start),
            Self::TSUnionType(n) => n.following_of_child(child, child_start),
            Self::TSIntersectionType(n) => n.following_of_child(child, child_start),
            Self::TSParenthesizedType(n) => n.following_of_child(child, child_start),
            Self::TSTypeOperator(n) => n.following_of_child(child, child_start),
            Self::TSArrayType(n) => n.following_of_child(child, child_start),
            Self::TSIndexedAccessType(n) => n.following_of_child(child, child_start),
            Self::TSTupleType(n) => n.following_of_child(child, child_start),
            Self::TSNamedTupleMember(n) => n.following_of_child(child, child_start),
            Self::TSOptionalType(n) => n.following_of_child(child, child_start),
            Self::TSRestType(n) => n.following_of_child(child, child_start),
            Self::TSTypeReference(n) => n.following_of_child(child, child_start),
            Self::TSQualifiedName(n) => n.following_of_child(child, child_start),
            Self::TSTypeParameterInstantiation(n) => n.following_of_child(child, child_start),
            Self::TSTypeParameter(n) => n.following_of_child(child, child_start),
            Self::TSTypeParameterDeclaration(n) => n.following_of_child(child, child_start),
            Self::TSTypeAliasDeclaration(n) => n.following_of_child(child, child_start),
            Self::TSClassImplements(n) => n.following_of_child(child, child_start),
            Self::TSInterfaceDeclaration(n) => n.following_of_child(child, child_start),
            Self::TSInterfaceBody(n) => n.following_of_child(child, child_start),
            Self::TSPropertySignature(n) => n.following_of_child(child, child_start),
            Self::TSIndexSignature(n) => n.following_of_child(child, child_start),
            Self::TSCallSignatureDeclaration(n) => n.following_of_child(child, child_start),
            Self::TSMethodSignature(n) => n.following_of_child(child, child_start),
            Self::TSConstructSignatureDeclaration(n) => n.following_of_child(child, child_start),
            Self::TSIndexSignatureName(n) => n.following_of_child(child, child_start),
            Self::TSInterfaceHeritage(n) => n.following_of_child(child, child_start),
            Self::TSTypePredicate(n) => n.following_of_child(child, child_start),
            Self::TSExternalModuleDeclaration(n) => n.following_of_child(child, child_start),
            Self::TSNamespaceDeclaration(n) => n.following_of_child(child, child_start),
            Self::TSGlobalDeclaration(n) => n.following_of_child(child, child_start),
            Self::TSModuleBlock(n) => n.following_of_child(child, child_start),
            Self::TSTypeLiteral(n) => n.following_of_child(child, child_start),
            Self::TSInferType(n) => n.following_of_child(child, child_start),
            Self::TSTypeQuery(n) => n.following_of_child(child, child_start),
            Self::TSImportType(n) => n.following_of_child(child, child_start),
            Self::TSImportTypeQualifiedName(n) => n.following_of_child(child, child_start),
            Self::TSFunctionType(n) => n.following_of_child(child, child_start),
            Self::TSConstructorType(n) => n.following_of_child(child, child_start),
            Self::TSMappedType(n) => n.following_of_child(child, child_start),
            Self::TSTemplateLiteralType(n) => n.following_of_child(child, child_start),
            Self::TSAsExpression(n) => n.following_of_child(child, child_start),
            Self::TSSatisfiesExpression(n) => n.following_of_child(child, child_start),
            Self::TSTypeAssertion(n) => n.following_of_child(child, child_start),
            Self::TSImportEqualsDeclaration(n) => n.following_of_child(child, child_start),
            Self::TSExternalModuleReference(n) => n.following_of_child(child, child_start),
            Self::TSNonNullExpression(n) => n.following_of_child(child, child_start),
            Self::Decorator(n) => n.following_of_child(child, child_start),
            Self::TSExportAssignment(n) => n.following_of_child(child, child_start),
            Self::TSNamespaceExportDeclaration(n) => n.following_of_child(child, child_start),
            Self::TSInstantiationExpression(n) => n.following_of_child(child, child_start),
            Self::JSDocNullableType(n) => n.following_of_child(child, child_start),
            Self::JSDocNonNullableType(n) => n.following_of_child(child, child_start),
            Self::IdentifierName(_)
            | Self::IdentifierReference(_)
            | Self::BindingIdentifier(_)
            | Self::LabelIdentifier(_)
            | Self::ThisExpression(_)
            | Self::Elision(_)
            | Self::TemplateElement(_)
            | Self::ImportMeta(_)
            | Self::NewTarget(_)
            | Self::Super(_)
            | Self::Hashbang(_)
            | Self::EmptyStatement(_)
            | Self::DebuggerStatement(_)
            | Self::PrivateIdentifier(_)
            | Self::BooleanLiteral(_)
            | Self::NullLiteral(_)
            | Self::NumericLiteral(_)
            | Self::StringLiteral(_)
            | Self::BigIntLiteral(_)
            | Self::RegExpLiteral(_)
            | Self::JSXOpeningFragment(_)
            | Self::JSXClosingFragment(_)
            | Self::JSXEmptyExpression(_)
            | Self::JSXIdentifier(_)
            | Self::JSXText(_)
            | Self::TSAnyKeyword(_)
            | Self::TSStringKeyword(_)
            | Self::TSBooleanKeyword(_)
            | Self::TSNumberKeyword(_)
            | Self::TSNeverKeyword(_)
            | Self::TSIntrinsicKeyword(_)
            | Self::TSUnknownKeyword(_)
            | Self::TSNullKeyword(_)
            | Self::TSUndefinedKeyword(_)
            | Self::TSVoidKeyword(_)
            | Self::TSSymbolKeyword(_)
            | Self::TSThisType(_)
            | Self::TSObjectKeyword(_)
            | Self::TSBigIntKeyword(_)
            | Self::JSDocUnknownType(_) => None,
        }
    }

    #[inline]
    pub fn debug_name(&self) -> &'static str {
        match self {
            Self::Dummy() => "Dummy",
            Self::Program(_) => "Program",
            Self::IdentifierName(_) => "IdentifierName",
            Self::IdentifierReference(_) => "IdentifierReference",
            Self::BindingIdentifier(_) => "BindingIdentifier",
            Self::LabelIdentifier(_) => "LabelIdentifier",
            Self::ThisExpression(_) => "ThisExpression",
            Self::ArrayExpression(_) => "ArrayExpression",
            Self::Elision(_) => "Elision",
            Self::ObjectExpression(_) => "ObjectExpression",
            Self::ObjectProperty(_) => "ObjectProperty",
            Self::TemplateLiteral(_) => "TemplateLiteral",
            Self::TaggedTemplateExpression(_) => "TaggedTemplateExpression",
            Self::TemplateElement(_) => "TemplateElement",
            Self::ComputedMemberExpression(_) => "ComputedMemberExpression",
            Self::StaticMemberExpression(_) => "StaticMemberExpression",
            Self::PrivateFieldExpression(_) => "PrivateFieldExpression",
            Self::CallExpression(_) => "CallExpression",
            Self::NewExpression(_) => "NewExpression",
            Self::ImportMeta(_) => "ImportMeta",
            Self::NewTarget(_) => "NewTarget",
            Self::SpreadElement(_) => "SpreadElement",
            Self::UpdateExpression(_) => "UpdateExpression",
            Self::UnaryExpression(_) => "UnaryExpression",
            Self::BinaryExpression(_) => "BinaryExpression",
            Self::PrivateInExpression(_) => "PrivateInExpression",
            Self::LogicalExpression(_) => "LogicalExpression",
            Self::ConditionalExpression(_) => "ConditionalExpression",
            Self::AssignmentExpression(_) => "AssignmentExpression",
            Self::ArrayAssignmentTarget(_) => "ArrayAssignmentTarget",
            Self::ObjectAssignmentTarget(_) => "ObjectAssignmentTarget",
            Self::AssignmentTargetRest(_) => "AssignmentTargetRest",
            Self::AssignmentTargetWithDefault(_) => "AssignmentTargetWithDefault",
            Self::AssignmentTargetPropertyIdentifier(_) => "AssignmentTargetPropertyIdentifier",
            Self::AssignmentTargetPropertyProperty(_) => "AssignmentTargetPropertyProperty",
            Self::SequenceExpression(_) => "SequenceExpression",
            Self::Super(_) => "Super",
            Self::AwaitExpression(_) => "AwaitExpression",
            Self::ChainExpression(_) => "ChainExpression",
            Self::ParenthesizedExpression(_) => "ParenthesizedExpression",
            Self::Directive(_) => "Directive",
            Self::Hashbang(_) => "Hashbang",
            Self::BlockStatement(_) => "BlockStatement",
            Self::VariableDeclaration(_) => "VariableDeclaration",
            Self::VariableDeclarator(_) => "VariableDeclarator",
            Self::EmptyStatement(_) => "EmptyStatement",
            Self::ExpressionStatement(_) => "ExpressionStatement",
            Self::IfStatement(_) => "IfStatement",
            Self::DoWhileStatement(_) => "DoWhileStatement",
            Self::WhileStatement(_) => "WhileStatement",
            Self::ForStatement(_) => "ForStatement",
            Self::ForInStatement(_) => "ForInStatement",
            Self::ForOfStatement(_) => "ForOfStatement",
            Self::ContinueStatement(_) => "ContinueStatement",
            Self::BreakStatement(_) => "BreakStatement",
            Self::ReturnStatement(_) => "ReturnStatement",
            Self::WithStatement(_) => "WithStatement",
            Self::SwitchStatement(_) => "SwitchStatement",
            Self::SwitchCase(_) => "SwitchCase",
            Self::LabeledStatement(_) => "LabeledStatement",
            Self::ThrowStatement(_) => "ThrowStatement",
            Self::TryStatement(_) => "TryStatement",
            Self::CatchClause(_) => "CatchClause",
            Self::CatchParameter(_) => "CatchParameter",
            Self::DebuggerStatement(_) => "DebuggerStatement",
            Self::AssignmentPattern(_) => "AssignmentPattern",
            Self::ObjectPattern(_) => "ObjectPattern",
            Self::BindingProperty(_) => "BindingProperty",
            Self::ArrayPattern(_) => "ArrayPattern",
            Self::BindingRestElement(_) => "BindingRestElement",
            Self::Function(_) => "Function",
            Self::FormalParameters(_) => "FormalParameters",
            Self::FormalParameter(_) => "FormalParameter",
            Self::FormalParameterRest(_) => "FormalParameterRest",
            Self::FunctionBody(_) => "FunctionBody",
            Self::ArrowFunctionExpression(_) => "ArrowFunctionExpression",
            Self::YieldExpression(_) => "YieldExpression",
            Self::Class(_) => "Class",
            Self::ClassBody(_) => "ClassBody",
            Self::MethodDefinition(_) => "MethodDefinition",
            Self::PropertyDefinition(_) => "PropertyDefinition",
            Self::PrivateIdentifier(_) => "PrivateIdentifier",
            Self::StaticBlock(_) => "StaticBlock",
            Self::AccessorProperty(_) => "AccessorProperty",
            Self::ImportExpression(_) => "ImportExpression",
            Self::ImportDeclaration(_) => "ImportDeclaration",
            Self::ImportSpecifier(_) => "ImportSpecifier",
            Self::ImportDefaultSpecifier(_) => "ImportDefaultSpecifier",
            Self::ImportNamespaceSpecifier(_) => "ImportNamespaceSpecifier",
            Self::WithClause(_) => "WithClause",
            Self::ImportAttribute(_) => "ImportAttribute",
            Self::ExportDeclaration(_) => "ExportDeclaration",
            Self::ExportNamedDeclaration(_) => "ExportNamedDeclaration",
            Self::ExportFromDeclaration(_) => "ExportFromDeclaration",
            Self::ExportDefaultDeclaration(_) => "ExportDefaultDeclaration",
            Self::ExportAllDeclaration(_) => "ExportAllDeclaration",
            Self::ExportSpecifier(_) => "ExportSpecifier",
            Self::V8IntrinsicExpression(_) => "V8IntrinsicExpression",
            Self::BooleanLiteral(_) => "BooleanLiteral",
            Self::NullLiteral(_) => "NullLiteral",
            Self::NumericLiteral(_) => "NumericLiteral",
            Self::StringLiteral(_) => "StringLiteral",
            Self::BigIntLiteral(_) => "BigIntLiteral",
            Self::RegExpLiteral(_) => "RegExpLiteral",
            Self::JSXElement(_) => "JSXElement",
            Self::JSXOpeningElement(_) => "JSXOpeningElement",
            Self::JSXClosingElement(_) => "JSXClosingElement",
            Self::JSXFragment(_) => "JSXFragment",
            Self::JSXOpeningFragment(_) => "JSXOpeningFragment",
            Self::JSXClosingFragment(_) => "JSXClosingFragment",
            Self::JSXNamespacedName(_) => "JSXNamespacedName",
            Self::JSXMemberExpression(_) => "JSXMemberExpression",
            Self::JSXExpressionContainer(_) => "JSXExpressionContainer",
            Self::JSXEmptyExpression(_) => "JSXEmptyExpression",
            Self::JSXAttribute(_) => "JSXAttribute",
            Self::JSXSpreadAttribute(_) => "JSXSpreadAttribute",
            Self::JSXIdentifier(_) => "JSXIdentifier",
            Self::JSXSpreadChild(_) => "JSXSpreadChild",
            Self::JSXText(_) => "JSXText",
            Self::TSThisParameter(_) => "TSThisParameter",
            Self::TSEnumDeclaration(_) => "TSEnumDeclaration",
            Self::TSEnumBody(_) => "TSEnumBody",
            Self::TSEnumMember(_) => "TSEnumMember",
            Self::TSTypeAnnotation(_) => "TSTypeAnnotation",
            Self::TSLiteralType(_) => "TSLiteralType",
            Self::TSConditionalType(_) => "TSConditionalType",
            Self::TSUnionType(_) => "TSUnionType",
            Self::TSIntersectionType(_) => "TSIntersectionType",
            Self::TSParenthesizedType(_) => "TSParenthesizedType",
            Self::TSTypeOperator(_) => "TSTypeOperator",
            Self::TSArrayType(_) => "TSArrayType",
            Self::TSIndexedAccessType(_) => "TSIndexedAccessType",
            Self::TSTupleType(_) => "TSTupleType",
            Self::TSNamedTupleMember(_) => "TSNamedTupleMember",
            Self::TSOptionalType(_) => "TSOptionalType",
            Self::TSRestType(_) => "TSRestType",
            Self::TSAnyKeyword(_) => "TSAnyKeyword",
            Self::TSStringKeyword(_) => "TSStringKeyword",
            Self::TSBooleanKeyword(_) => "TSBooleanKeyword",
            Self::TSNumberKeyword(_) => "TSNumberKeyword",
            Self::TSNeverKeyword(_) => "TSNeverKeyword",
            Self::TSIntrinsicKeyword(_) => "TSIntrinsicKeyword",
            Self::TSUnknownKeyword(_) => "TSUnknownKeyword",
            Self::TSNullKeyword(_) => "TSNullKeyword",
            Self::TSUndefinedKeyword(_) => "TSUndefinedKeyword",
            Self::TSVoidKeyword(_) => "TSVoidKeyword",
            Self::TSSymbolKeyword(_) => "TSSymbolKeyword",
            Self::TSThisType(_) => "TSThisType",
            Self::TSObjectKeyword(_) => "TSObjectKeyword",
            Self::TSBigIntKeyword(_) => "TSBigIntKeyword",
            Self::TSTypeReference(_) => "TSTypeReference",
            Self::TSQualifiedName(_) => "TSQualifiedName",
            Self::TSTypeParameterInstantiation(_) => "TSTypeParameterInstantiation",
            Self::TSTypeParameter(_) => "TSTypeParameter",
            Self::TSTypeParameterDeclaration(_) => "TSTypeParameterDeclaration",
            Self::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration",
            Self::TSClassImplements(_) => "TSClassImplements",
            Self::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration",
            Self::TSInterfaceBody(_) => "TSInterfaceBody",
            Self::TSPropertySignature(_) => "TSPropertySignature",
            Self::TSIndexSignature(_) => "TSIndexSignature",
            Self::TSCallSignatureDeclaration(_) => "TSCallSignatureDeclaration",
            Self::TSMethodSignature(_) => "TSMethodSignature",
            Self::TSConstructSignatureDeclaration(_) => "TSConstructSignatureDeclaration",
            Self::TSIndexSignatureName(_) => "TSIndexSignatureName",
            Self::TSInterfaceHeritage(_) => "TSInterfaceHeritage",
            Self::TSTypePredicate(_) => "TSTypePredicate",
            Self::TSExternalModuleDeclaration(_) => "TSExternalModuleDeclaration",
            Self::TSNamespaceDeclaration(_) => "TSNamespaceDeclaration",
            Self::TSGlobalDeclaration(_) => "TSGlobalDeclaration",
            Self::TSModuleBlock(_) => "TSModuleBlock",
            Self::TSTypeLiteral(_) => "TSTypeLiteral",
            Self::TSInferType(_) => "TSInferType",
            Self::TSTypeQuery(_) => "TSTypeQuery",
            Self::TSImportType(_) => "TSImportType",
            Self::TSImportTypeQualifiedName(_) => "TSImportTypeQualifiedName",
            Self::TSFunctionType(_) => "TSFunctionType",
            Self::TSConstructorType(_) => "TSConstructorType",
            Self::TSMappedType(_) => "TSMappedType",
            Self::TSTemplateLiteralType(_) => "TSTemplateLiteralType",
            Self::TSAsExpression(_) => "TSAsExpression",
            Self::TSSatisfiesExpression(_) => "TSSatisfiesExpression",
            Self::TSTypeAssertion(_) => "TSTypeAssertion",
            Self::TSImportEqualsDeclaration(_) => "TSImportEqualsDeclaration",
            Self::TSExternalModuleReference(_) => "TSExternalModuleReference",
            Self::TSNonNullExpression(_) => "TSNonNullExpression",
            Self::Decorator(_) => "Decorator",
            Self::TSExportAssignment(_) => "TSExportAssignment",
            Self::TSNamespaceExportDeclaration(_) => "TSNamespaceExportDeclaration",
            Self::TSInstantiationExpression(_) => "TSInstantiationExpression",
            Self::JSDocNullableType(_) => "JSDocNullableType",
            Self::JSDocNonNullableType(_) => "JSDocNonNullableType",
            Self::JSDocUnknownType(_) => "JSDocUnknownType",
        }
    }
}

impl<'a> AstNode<'a, Program<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn source_type(self) -> SourceType {
        self.inner.source_type
    }

    #[inline]
    pub fn source_text(self) -> &'a str {
        self.inner.source_text
    }

    #[inline]
    pub fn comments(self) -> &'a ArenaVec<'a, Comment> {
        &self.inner.comments
    }

    #[inline]
    pub fn hashbang(self) -> Option<AstNode<'a, Hashbang<'a>>> {
        self.inner.hashbang.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn directives(self) -> AstNode<'a, ArenaVec<'a, Directive<'a>>> {
        AstNode::new(&self.inner.directives)
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, ArenaVec<'a, Statement<'a>>> {
        AstNode::new(&self.inner.body)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if self.inner.hashbang.as_ref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .directives
                    .first()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.body.first().map(|n| n.span().start))
                    .map_or(Following::Inherit, Following::Start),
            );
        }
        if let Some(following) = list_following(&self.inner.directives, child, child_start, || {
            self.inner.body.first().map_or(Following::Inherit, |n| Following::Start(n.span().start))
        }) {
            return Some(following);
        }
        if let Some(following) =
            list_following(&self.inner.body, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, Program<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::Program(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, Expression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            Expression::BooleanLiteral(s) => AstNodes::BooleanLiteral(AstNode::new(s.as_ref())),
            Expression::NullLiteral(s) => AstNodes::NullLiteral(AstNode::new(s.as_ref())),
            Expression::NumericLiteral(s) => AstNodes::NumericLiteral(AstNode::new(s.as_ref())),
            Expression::BigIntLiteral(s) => AstNodes::BigIntLiteral(AstNode::new(s.as_ref())),
            Expression::RegExpLiteral(s) => AstNodes::RegExpLiteral(AstNode::new(s.as_ref())),
            Expression::StringLiteral(s) => AstNodes::StringLiteral(AstNode::new(s.as_ref())),
            Expression::TemplateLiteral(s) => AstNodes::TemplateLiteral(AstNode::new(s.as_ref())),
            Expression::Identifier(s) => AstNodes::IdentifierReference(AstNode::new(s.as_ref())),
            Expression::Super(s) => AstNodes::Super(AstNode::new(s.as_ref())),
            Expression::ArrayExpression(s) => AstNodes::ArrayExpression(AstNode::new(s.as_ref())),
            Expression::ArrowFunctionExpression(s) => {
                AstNodes::ArrowFunctionExpression(AstNode::new(s.as_ref()))
            }
            Expression::AssignmentExpression(s) => {
                AstNodes::AssignmentExpression(AstNode::new(s.as_ref()))
            }
            Expression::AwaitExpression(s) => AstNodes::AwaitExpression(AstNode::new(s.as_ref())),
            Expression::BinaryExpression(s) => AstNodes::BinaryExpression(AstNode::new(s.as_ref())),
            Expression::CallExpression(s) => AstNodes::CallExpression(AstNode::new(s.as_ref())),
            Expression::ChainExpression(s) => AstNodes::ChainExpression(AstNode::new(s.as_ref())),
            Expression::ClassExpression(s) => AstNodes::Class(AstNode::new(s.as_ref())),
            Expression::ConditionalExpression(s) => {
                AstNodes::ConditionalExpression(AstNode::new(s.as_ref()))
            }
            Expression::FunctionExpression(s) => AstNodes::Function(AstNode::new(s.as_ref())),
            Expression::ImportExpression(s) => AstNodes::ImportExpression(AstNode::new(s.as_ref())),
            Expression::LogicalExpression(s) => {
                AstNodes::LogicalExpression(AstNode::new(s.as_ref()))
            }
            Expression::NewExpression(s) => AstNodes::NewExpression(AstNode::new(s.as_ref())),
            Expression::ObjectExpression(s) => AstNodes::ObjectExpression(AstNode::new(s.as_ref())),
            Expression::ParenthesizedExpression(s) => {
                AstNodes::ParenthesizedExpression(AstNode::new(s.as_ref()))
            }
            Expression::SequenceExpression(s) => {
                AstNodes::SequenceExpression(AstNode::new(s.as_ref()))
            }
            Expression::TaggedTemplateExpression(s) => {
                AstNodes::TaggedTemplateExpression(AstNode::new(s.as_ref()))
            }
            Expression::ThisExpression(s) => AstNodes::ThisExpression(AstNode::new(s.as_ref())),
            Expression::UnaryExpression(s) => AstNodes::UnaryExpression(AstNode::new(s.as_ref())),
            Expression::UpdateExpression(s) => AstNodes::UpdateExpression(AstNode::new(s.as_ref())),
            Expression::YieldExpression(s) => AstNodes::YieldExpression(AstNode::new(s.as_ref())),
            Expression::PrivateInExpression(s) => {
                AstNodes::PrivateInExpression(AstNode::new(s.as_ref()))
            }
            Expression::ImportMeta(s) => AstNodes::ImportMeta(AstNode::new(s.as_ref())),
            Expression::NewTarget(s) => AstNodes::NewTarget(AstNode::new(s.as_ref())),
            Expression::JSXElement(s) => AstNodes::JSXElement(AstNode::new(s.as_ref())),
            Expression::JSXFragment(s) => AstNodes::JSXFragment(AstNode::new(s.as_ref())),
            Expression::TSAsExpression(s) => AstNodes::TSAsExpression(AstNode::new(s.as_ref())),
            Expression::TSSatisfiesExpression(s) => {
                AstNodes::TSSatisfiesExpression(AstNode::new(s.as_ref()))
            }
            Expression::TSTypeAssertion(s) => AstNodes::TSTypeAssertion(AstNode::new(s.as_ref())),
            Expression::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(AstNode::new(s.as_ref()))
            }
            Expression::TSInstantiationExpression(s) => {
                AstNodes::TSInstantiationExpression(AstNode::new(s.as_ref()))
            }
            Expression::V8IntrinsicExpression(s) => {
                AstNodes::V8IntrinsicExpression(AstNode::new(s.as_ref()))
            }
            it @ match_member_expression!(Expression) => {
                AstNode::new(it.to_member_expression()).as_ast_nodes()
            }
        }
    }
}

impl<'a> AstNode<'a, IdentifierName<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, IdentifierName<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::IdentifierName(*self)
    }
}

impl<'a> AstNode<'a, IdentifierReference<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, IdentifierReference<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::IdentifierReference(*self)
    }
}

impl<'a> AstNode<'a, BindingIdentifier<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, BindingIdentifier<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::BindingIdentifier(*self)
    }
}

impl<'a> AstNode<'a, LabelIdentifier<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, LabelIdentifier<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::LabelIdentifier(*self)
    }
}

impl<'a> AstNode<'a, ThisExpression> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ThisExpression> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ThisExpression(*self)
    }
}

impl<'a> AstNode<'a, ArrayExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn elements(self) -> AstNode<'a, ArenaVec<'a, ArrayExpressionElement<'a>>> {
        AstNode::new(&self.inner.elements)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.elements, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ArrayExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ArrayExpression(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ArrayExpressionElement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            ArrayExpressionElement::SpreadElement(s) => {
                AstNodes::SpreadElement(AstNode::new(s.as_ref()))
            }
            ArrayExpressionElement::Elision(s) => AstNodes::Elision(AstNode::new(s.as_ref())),
            it @ match_expression!(ArrayExpressionElement) => {
                AstNode::new(it.to_expression()).as_ast_nodes()
            }
        }
    }
}

impl<'a> AstNode<'a, Elision> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, Elision> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::Elision(*self)
    }
}

impl<'a> AstNode<'a, ObjectExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn properties(self) -> AstNode<'a, ArenaVec<'a, ObjectPropertyKind<'a>>> {
        AstNode::new(&self.inner.properties)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.properties, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ObjectExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ObjectExpression(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ObjectPropertyKind<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            ObjectPropertyKind::ObjectProperty(s) => {
                AstNodes::ObjectProperty(AstNode::new(s.as_ref()))
            }
            ObjectPropertyKind::SpreadProperty(s) => {
                AstNodes::SpreadElement(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, ObjectProperty<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn kind(self) -> PropertyKind {
        self.inner.kind
    }

    #[inline]
    pub fn key(self) -> AstNode<'a, PropertyKey<'a>> {
        AstNode::new(&self.inner.key)
    }

    #[inline]
    pub fn value(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.value)
    }

    #[inline]
    pub fn method(self) -> bool {
        self.inner.method
    }

    #[inline]
    pub fn shorthand(self) -> bool {
        self.inner.shorthand
    }

    #[inline]
    pub fn computed(self) -> bool {
        self.inner.computed
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.key;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.value.span().start));
        }
        let node = &self.inner.value;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ObjectProperty<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ObjectProperty(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, PropertyKey<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            PropertyKey::StaticIdentifier(s) => AstNodes::IdentifierName(AstNode::new(s.as_ref())),
            PropertyKey::PrivateIdentifier(s) => {
                AstNodes::PrivateIdentifier(AstNode::new(s.as_ref()))
            }
            it @ match_expression!(PropertyKey) => AstNode::new(it.to_expression()).as_ast_nodes(),
        }
    }
}

impl<'a> AstNode<'a, TemplateLiteral<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn quasis(self) -> AstNode<'a, ArenaVec<'a, TemplateElement<'a>>> {
        AstNode::new(&self.inner.quasis)
    }

    #[inline]
    pub fn expressions(self) -> AstNode<'a, ArenaVec<'a, Expression<'a>>> {
        AstNode::new(&self.inner.expressions)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.quasis, child, child_start, || {
            self.inner
                .expressions
                .first()
                .map_or(Following::Inherit, |n| Following::Start(n.span().start))
        }) {
            return Some(following);
        }
        if let Some(following) =
            list_following(&self.inner.expressions, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TemplateLiteral<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TemplateLiteral(*self)
    }
}

impl<'a> AstNode<'a, TaggedTemplateExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn tag(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.tag)
    }

    #[inline]
    pub fn type_arguments(self) -> Option<AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        self.inner.type_arguments.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn quasi(self) -> AstNode<'a, TemplateLiteral<'a>> {
        AstNode::new(&self.inner.quasi)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.tag;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_arguments
                    .as_deref()
                    .map_or(Following::Start(self.inner.quasi.span().start), |n| {
                        Following::Start(n.span().start)
                    }),
            );
        }
        if self.inner.type_arguments.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(self.inner.quasi.span().start));
        }
        let node = &self.inner.quasi;
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TaggedTemplateExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TaggedTemplateExpression(*self)
    }
}

impl<'a> AstNode<'a, TemplateElement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(self) -> &'a TemplateElementValue<'a> {
        &self.inner.value
    }

    #[inline]
    pub fn tail(self) -> bool {
        self.inner.tail
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TemplateElement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TemplateElement(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, MemberExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            MemberExpression::ComputedMemberExpression(s) => {
                AstNodes::ComputedMemberExpression(AstNode::new(s.as_ref()))
            }
            MemberExpression::StaticMemberExpression(s) => {
                AstNodes::StaticMemberExpression(AstNode::new(s.as_ref()))
            }
            MemberExpression::PrivateFieldExpression(s) => {
                AstNodes::PrivateFieldExpression(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, ComputedMemberExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.object)
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.expression)
    }

    #[inline]
    pub fn optional(self) -> bool {
        self.inner.optional
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.object;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.expression.span().start));
        }
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ComputedMemberExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ComputedMemberExpression(*self)
    }
}

impl<'a> AstNode<'a, StaticMemberExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.object)
    }

    #[inline]
    pub fn property(self) -> AstNode<'a, IdentifierName<'a>> {
        AstNode::new(&self.inner.property)
    }

    #[inline]
    pub fn optional(self) -> bool {
        self.inner.optional
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.object;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.property.span().start));
        }
        let node = &self.inner.property;
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, StaticMemberExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::StaticMemberExpression(*self)
    }
}

impl<'a> AstNode<'a, PrivateFieldExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.object)
    }

    #[inline]
    pub fn field(self) -> AstNode<'a, PrivateIdentifier<'a>> {
        AstNode::new(&self.inner.field)
    }

    #[inline]
    pub fn optional(self) -> bool {
        self.inner.optional
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.object;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.field.span().start));
        }
        let node = &self.inner.field;
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, PrivateFieldExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::PrivateFieldExpression(*self)
    }
}

impl<'a> AstNode<'a, CallExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn callee(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.callee)
    }

    #[inline]
    pub fn type_arguments(self) -> Option<AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        self.inner.type_arguments.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn arguments(self) -> AstNode<'a, ArenaVec<'a, Argument<'a>>> {
        AstNode::new(&self.inner.arguments)
    }

    #[inline]
    pub fn optional(self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn pure(self) -> bool {
        self.inner.pure
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        let node = &self.inner.callee;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_arguments
                    .as_deref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.arguments.first().map(|n| n.span().start))
                    .map_or(Following::Inherit, Following::Start),
            );
        }
        if self.inner.type_arguments.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .arguments
                    .first()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if let Some(following) =
            list_following(&self.inner.arguments, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, CallExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::CallExpression(*self)
    }
}

impl<'a> AstNode<'a, NewExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn callee(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.callee)
    }

    #[inline]
    pub fn type_arguments(self) -> Option<AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        self.inner.type_arguments.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn arguments(self) -> AstNode<'a, ArenaVec<'a, Argument<'a>>> {
        AstNode::new(&self.inner.arguments)
    }

    #[inline]
    pub fn pure(self) -> bool {
        self.inner.pure
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        let node = &self.inner.callee;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_arguments
                    .as_deref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.arguments.first().map(|n| n.span().start))
                    .map_or(Following::Inherit, Following::Start),
            );
        }
        if self.inner.type_arguments.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .arguments
                    .first()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if let Some(following) =
            list_following(&self.inner.arguments, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, NewExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::NewExpression(*self)
    }
}

impl<'a> AstNode<'a, ImportMeta> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ImportMeta> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ImportMeta(*self)
    }
}

impl<'a> AstNode<'a, NewTarget> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, NewTarget> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::NewTarget(*self)
    }
}

impl<'a> AstNode<'a, SpreadElement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.argument)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.argument;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, SpreadElement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::SpreadElement(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, Argument<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            Argument::SpreadElement(s) => AstNodes::SpreadElement(AstNode::new(s.as_ref())),
            it @ match_expression!(Argument) => AstNode::new(it.to_expression()).as_ast_nodes(),
        }
    }
}

impl<'a> AstNode<'a, UpdateExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn operator(self) -> UpdateOperator {
        self.inner.operator
    }

    #[inline]
    pub fn prefix(self) -> bool {
        self.inner.prefix
    }

    #[inline]
    pub fn argument(self) -> AstNode<'a, SimpleAssignmentTarget<'a>> {
        AstNode::new(&self.inner.argument)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.argument;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, UpdateExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::UpdateExpression(*self)
    }
}

impl<'a> AstNode<'a, UnaryExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn operator(self) -> UnaryOperator {
        self.inner.operator
    }

    #[inline]
    pub fn argument(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.argument)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.argument;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, UnaryExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::UnaryExpression(*self)
    }
}

impl<'a> AstNode<'a, BinaryExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.left)
    }

    #[inline]
    pub fn operator(self) -> BinaryOperator {
        self.inner.operator
    }

    #[inline]
    pub fn right(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.right)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.left;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.right.span().start));
        }
        let node = &self.inner.right;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, BinaryExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::BinaryExpression(*self)
    }
}

impl<'a> AstNode<'a, PrivateInExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left(self) -> AstNode<'a, PrivateIdentifier<'a>> {
        AstNode::new(&self.inner.left)
    }

    #[inline]
    pub fn right(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.right)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.left;
        if node_address(node) == child {
            return Some(Following::Start(self.inner.right.span().start));
        }
        let node = &self.inner.right;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, PrivateInExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::PrivateInExpression(*self)
    }
}

impl<'a> AstNode<'a, LogicalExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.left)
    }

    #[inline]
    pub fn operator(self) -> LogicalOperator {
        self.inner.operator
    }

    #[inline]
    pub fn right(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.right)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.left;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.right.span().start));
        }
        let node = &self.inner.right;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, LogicalExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::LogicalExpression(*self)
    }
}

impl<'a> AstNode<'a, ConditionalExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn test(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.test)
    }

    #[inline]
    pub fn consequent(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.consequent)
    }

    #[inline]
    pub fn alternate(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.alternate)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.test;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.consequent.span().start));
        }
        let node = &self.inner.consequent;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.alternate.span().start));
        }
        let node = &self.inner.alternate;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ConditionalExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ConditionalExpression(*self)
    }
}

impl<'a> AstNode<'a, AssignmentExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn operator(self) -> AssignmentOperator {
        self.inner.operator
    }

    #[inline]
    pub fn left(self) -> AstNode<'a, AssignmentTarget<'a>> {
        AstNode::new(&self.inner.left)
    }

    #[inline]
    pub fn right(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.right)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.left;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.right.span().start));
        }
        let node = &self.inner.right;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, AssignmentExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::AssignmentExpression(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, AssignmentTarget<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                AstNode::new(it.to_simple_assignment_target()).as_ast_nodes()
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                AstNode::new(it.to_assignment_target_pattern()).as_ast_nodes()
            }
        }
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, SimpleAssignmentTarget<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(s) => {
                AstNodes::IdentifierReference(AstNode::new(s.as_ref()))
            }
            SimpleAssignmentTarget::TSAsExpression(s) => {
                AstNodes::TSAsExpression(AstNode::new(s.as_ref()))
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(s) => {
                AstNodes::TSSatisfiesExpression(AstNode::new(s.as_ref()))
            }
            SimpleAssignmentTarget::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(AstNode::new(s.as_ref()))
            }
            SimpleAssignmentTarget::TSTypeAssertion(s) => {
                AstNodes::TSTypeAssertion(AstNode::new(s.as_ref()))
            }
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                AstNode::new(it.to_member_expression()).as_ast_nodes()
            }
        }
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, AssignmentTargetPattern<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            AssignmentTargetPattern::ArrayAssignmentTarget(s) => {
                AstNodes::ArrayAssignmentTarget(AstNode::new(s.as_ref()))
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(s) => {
                AstNodes::ObjectAssignmentTarget(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, ArrayAssignmentTarget<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn elements(self) -> AstNode<'a, ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>>> {
        AstNode::new(&self.inner.elements)
    }

    #[inline]
    pub fn rest(self) -> Option<AstNode<'a, AssignmentTargetRest<'a>>> {
        self.inner.rest.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.elements, child, child_start, || {
            self.inner
                .rest
                .as_deref()
                .map_or(Following::Start(0), |n| Following::Start(n.span().start))
        }) {
            return Some(following);
        }
        if self.inner.rest.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ArrayAssignmentTarget<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ArrayAssignmentTarget(*self)
    }
}

impl<'a> AstNode<'a, ObjectAssignmentTarget<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn properties(self) -> AstNode<'a, ArenaVec<'a, AssignmentTargetProperty<'a>>> {
        AstNode::new(&self.inner.properties)
    }

    #[inline]
    pub fn rest(self) -> Option<AstNode<'a, AssignmentTargetRest<'a>>> {
        self.inner.rest.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.properties, child, child_start, || {
            self.inner
                .rest
                .as_deref()
                .map_or(Following::Start(0), |n| Following::Start(n.span().start))
        }) {
            return Some(following);
        }
        if self.inner.rest.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ObjectAssignmentTarget<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ObjectAssignmentTarget(*self)
    }
}

impl<'a> AstNode<'a, AssignmentTargetRest<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn target(self) -> AstNode<'a, AssignmentTarget<'a>> {
        AstNode::new(&self.inner.target)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.target;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, AssignmentTargetRest<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::AssignmentTargetRest(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, AssignmentTargetMaybeDefault<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(s) => {
                AstNodes::AssignmentTargetWithDefault(AstNode::new(s.as_ref()))
            }
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                AstNode::new(it.to_assignment_target()).as_ast_nodes()
            }
        }
    }
}

impl<'a> AstNode<'a, AssignmentTargetWithDefault<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn binding(self) -> AstNode<'a, AssignmentTarget<'a>> {
        AstNode::new(&self.inner.binding)
    }

    #[inline]
    pub fn init(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.init)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.binding;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.init.span().start));
        }
        let node = &self.inner.init;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, AssignmentTargetWithDefault<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::AssignmentTargetWithDefault(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, AssignmentTargetProperty<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(s) => {
                AstNodes::AssignmentTargetPropertyIdentifier(AstNode::new(s.as_ref()))
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(s) => {
                AstNodes::AssignmentTargetPropertyProperty(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, AssignmentTargetPropertyIdentifier<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn binding(self) -> AstNode<'a, IdentifierReference<'a>> {
        AstNode::new(&self.inner.binding)
    }

    #[inline]
    pub fn init(self) -> Option<AstNode<'a, Expression<'a>>> {
        self.inner.init.as_ref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.binding;
        if node_address(node) == child {
            return Some(
                self.inner
                    .init
                    .as_ref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self
            .inner
            .init
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, AssignmentTargetPropertyIdentifier<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::AssignmentTargetPropertyIdentifier(*self)
    }
}

impl<'a> AstNode<'a, AssignmentTargetPropertyProperty<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(self) -> AstNode<'a, PropertyKey<'a>> {
        AstNode::new(&self.inner.name)
    }

    #[inline]
    pub fn binding(self) -> AstNode<'a, AssignmentTargetMaybeDefault<'a>> {
        AstNode::new(&self.inner.binding)
    }

    #[inline]
    pub fn computed(self) -> bool {
        self.inner.computed
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.name;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.binding.span().start));
        }
        let node = &self.inner.binding;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, AssignmentTargetPropertyProperty<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::AssignmentTargetPropertyProperty(*self)
    }
}

impl<'a> AstNode<'a, SequenceExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expressions(self) -> AstNode<'a, ArenaVec<'a, Expression<'a>>> {
        AstNode::new(&self.inner.expressions)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.expressions, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, SequenceExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::SequenceExpression(*self)
    }
}

impl<'a> AstNode<'a, Super> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, Super> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::Super(*self)
    }
}

impl<'a> AstNode<'a, AwaitExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.argument)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.argument;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, AwaitExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::AwaitExpression(*self)
    }
}

impl<'a> AstNode<'a, ChainExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, ChainElement<'a>> {
        AstNode::new(&self.inner.expression)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ChainExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ChainExpression(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ChainElement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            ChainElement::CallExpression(s) => AstNodes::CallExpression(AstNode::new(s.as_ref())),
            ChainElement::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(AstNode::new(s.as_ref()))
            }
            it @ match_member_expression!(ChainElement) => {
                AstNode::new(it.to_member_expression()).as_ast_nodes()
            }
        }
    }
}

impl<'a> AstNode<'a, ParenthesizedExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.expression)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ParenthesizedExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ParenthesizedExpression(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, Statement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            Statement::BlockStatement(s) => AstNodes::BlockStatement(AstNode::new(s.as_ref())),
            Statement::BreakStatement(s) => AstNodes::BreakStatement(AstNode::new(s.as_ref())),
            Statement::ContinueStatement(s) => {
                AstNodes::ContinueStatement(AstNode::new(s.as_ref()))
            }
            Statement::DebuggerStatement(s) => {
                AstNodes::DebuggerStatement(AstNode::new(s.as_ref()))
            }
            Statement::DoWhileStatement(s) => AstNodes::DoWhileStatement(AstNode::new(s.as_ref())),
            Statement::EmptyStatement(s) => AstNodes::EmptyStatement(AstNode::new(s.as_ref())),
            Statement::ExpressionStatement(s) => {
                AstNodes::ExpressionStatement(AstNode::new(s.as_ref()))
            }
            Statement::ForInStatement(s) => AstNodes::ForInStatement(AstNode::new(s.as_ref())),
            Statement::ForOfStatement(s) => AstNodes::ForOfStatement(AstNode::new(s.as_ref())),
            Statement::ForStatement(s) => AstNodes::ForStatement(AstNode::new(s.as_ref())),
            Statement::IfStatement(s) => AstNodes::IfStatement(AstNode::new(s.as_ref())),
            Statement::LabeledStatement(s) => AstNodes::LabeledStatement(AstNode::new(s.as_ref())),
            Statement::ReturnStatement(s) => AstNodes::ReturnStatement(AstNode::new(s.as_ref())),
            Statement::SwitchStatement(s) => AstNodes::SwitchStatement(AstNode::new(s.as_ref())),
            Statement::ThrowStatement(s) => AstNodes::ThrowStatement(AstNode::new(s.as_ref())),
            Statement::TryStatement(s) => AstNodes::TryStatement(AstNode::new(s.as_ref())),
            Statement::WhileStatement(s) => AstNodes::WhileStatement(AstNode::new(s.as_ref())),
            Statement::WithStatement(s) => AstNodes::WithStatement(AstNode::new(s.as_ref())),
            it @ match_declaration!(Statement) => AstNode::new(it.to_declaration()).as_ast_nodes(),
            it @ match_module_declaration!(Statement) => {
                AstNode::new(it.to_module_declaration()).as_ast_nodes()
            }
        }
    }
}

impl<'a> AstNode<'a, Directive<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, StringLiteral<'a>> {
        AstNode::new(&self.inner.expression)
    }

    #[inline]
    pub fn directive(self) -> Str<'a> {
        self.inner.directive
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, Directive<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::Directive(*self)
    }
}

impl<'a> AstNode<'a, Hashbang<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(self) -> Str<'a> {
        self.inner.value
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, Hashbang<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::Hashbang(*self)
    }
}

impl<'a> AstNode<'a, BlockStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, ArenaVec<'a, Statement<'a>>> {
        AstNode::new(&self.inner.body)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.body, child, child_start, || Following::Start(0))
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, BlockStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::BlockStatement(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, Declaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            Declaration::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(AstNode::new(s.as_ref()))
            }
            Declaration::FunctionDeclaration(s) => AstNodes::Function(AstNode::new(s.as_ref())),
            Declaration::ClassDeclaration(s) => AstNodes::Class(AstNode::new(s.as_ref())),
            Declaration::TSTypeAliasDeclaration(s) => {
                AstNodes::TSTypeAliasDeclaration(AstNode::new(s.as_ref()))
            }
            Declaration::TSInterfaceDeclaration(s) => {
                AstNodes::TSInterfaceDeclaration(AstNode::new(s.as_ref()))
            }
            Declaration::TSEnumDeclaration(s) => {
                AstNodes::TSEnumDeclaration(AstNode::new(s.as_ref()))
            }
            Declaration::TSExternalModuleDeclaration(s) => {
                AstNodes::TSExternalModuleDeclaration(AstNode::new(s.as_ref()))
            }
            Declaration::TSNamespaceDeclaration(s) => {
                AstNodes::TSNamespaceDeclaration(AstNode::new(s.as_ref()))
            }
            Declaration::TSGlobalDeclaration(s) => {
                AstNodes::TSGlobalDeclaration(AstNode::new(s.as_ref()))
            }
            Declaration::TSImportEqualsDeclaration(s) => {
                AstNodes::TSImportEqualsDeclaration(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, VariableDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn kind(self) -> VariableDeclarationKind {
        self.inner.kind
    }

    #[inline]
    pub fn declarations(self) -> AstNode<'a, ArenaVec<'a, VariableDeclarator<'a>>> {
        AstNode::new(&self.inner.declarations)
    }

    #[inline]
    pub fn declare(self) -> bool {
        self.inner.declare
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.declarations, child, child_start, || Following::Start(0))
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, VariableDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::VariableDeclaration(*self)
    }
}

impl<'a> AstNode<'a, VariableDeclarator<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id(self) -> AstNode<'a, BindingPattern<'a>> {
        AstNode::new(&self.inner.id)
    }

    #[inline]
    pub fn type_annotation(self) -> Option<AstNode<'a, TSTypeAnnotation<'a>>> {
        self.inner.type_annotation.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn init(self) -> Option<AstNode<'a, Expression<'a>>> {
        self.inner.init.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn definite(self) -> bool {
        self.inner.definite
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.id;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_annotation
                    .as_deref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.init.as_ref().map(|n| n.span().start))
                    .map_or(Following::Inherit, Following::Start),
            );
        }
        if self.inner.type_annotation.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .init
                    .as_ref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self
            .inner
            .init
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, VariableDeclarator<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::VariableDeclarator(*self)
    }
}

impl<'a> AstNode<'a, EmptyStatement> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, EmptyStatement> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::EmptyStatement(*self)
    }
}

impl<'a> AstNode<'a, ExpressionStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.expression)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ExpressionStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ExpressionStatement(*self)
    }
}

impl<'a> AstNode<'a, IfStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn test(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.test)
    }

    #[inline]
    pub fn consequent(self) -> AstNode<'a, Statement<'a>> {
        AstNode::new(&self.inner.consequent)
    }

    #[inline]
    pub fn alternate(self) -> Option<AstNode<'a, Statement<'a>>> {
        self.inner.alternate.as_ref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.test;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.consequent.span().start));
        }
        let node = &self.inner.consequent;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .alternate
                    .as_ref()
                    .map_or(Following::Start(0), |n| Following::Start(n.span().start)),
            );
        }
        if self
            .inner
            .alternate
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, IfStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::IfStatement(*self)
    }
}

impl<'a> AstNode<'a, DoWhileStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, Statement<'a>> {
        AstNode::new(&self.inner.body)
    }

    #[inline]
    pub fn test(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.test)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.body;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.test.span().start));
        }
        let node = &self.inner.test;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, DoWhileStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::DoWhileStatement(*self)
    }
}

impl<'a> AstNode<'a, WhileStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn test(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.test)
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, Statement<'a>> {
        AstNode::new(&self.inner.body)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.test;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.body.span().start));
        }
        let node = &self.inner.body;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, WhileStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::WhileStatement(*self)
    }
}

impl<'a> AstNode<'a, ForStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn init(self) -> Option<AstNode<'a, ForStatementInit<'a>>> {
        self.inner.init.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn test(self) -> Option<AstNode<'a, Expression<'a>>> {
        self.inner.test.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn update(self) -> Option<AstNode<'a, Expression<'a>>> {
        self.inner.update.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, Statement<'a>> {
        AstNode::new(&self.inner.body)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        if self
            .inner
            .init
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(
                self.inner
                    .test
                    .as_ref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.update.as_ref().map(|n| n.span().start))
                    .map_or(Following::Start(self.inner.body.span().start), Following::Start),
            );
        }
        if self
            .inner
            .test
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(
                self.inner
                    .update
                    .as_ref()
                    .map_or(Following::Start(self.inner.body.span().start), |n| {
                        Following::Start(n.span().start)
                    }),
            );
        }
        if self
            .inner
            .update
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Start(self.inner.body.span().start));
        }
        let node = &self.inner.body;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ForStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ForStatement(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ForStatementInit<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            ForStatementInit::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(AstNode::new(s.as_ref()))
            }
            it @ match_expression!(ForStatementInit) => {
                AstNode::new(it.to_expression()).as_ast_nodes()
            }
        }
    }
}

impl<'a> AstNode<'a, ForInStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left(self) -> AstNode<'a, ForStatementLeft<'a>> {
        AstNode::new(&self.inner.left)
    }

    #[inline]
    pub fn right(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.right)
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, Statement<'a>> {
        AstNode::new(&self.inner.body)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.left;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.right.span().start));
        }
        let node = &self.inner.right;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.body.span().start));
        }
        let node = &self.inner.body;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ForInStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ForInStatement(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ForStatementLeft<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            ForStatementLeft::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(AstNode::new(s.as_ref()))
            }
            it @ match_assignment_target!(ForStatementLeft) => {
                AstNode::new(it.to_assignment_target()).as_ast_nodes()
            }
        }
    }
}

impl<'a> AstNode<'a, ForOfStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#await(self) -> bool {
        self.inner.r#await
    }

    #[inline]
    pub fn left(self) -> AstNode<'a, ForStatementLeft<'a>> {
        AstNode::new(&self.inner.left)
    }

    #[inline]
    pub fn right(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.right)
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, Statement<'a>> {
        AstNode::new(&self.inner.body)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.left;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.right.span().start));
        }
        let node = &self.inner.right;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.body.span().start));
        }
        let node = &self.inner.body;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ForOfStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ForOfStatement(*self)
    }
}

impl<'a> AstNode<'a, ContinueStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn label(self) -> Option<AstNode<'a, LabelIdentifier<'a>>> {
        self.inner.label.as_ref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        if self.inner.label.as_ref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ContinueStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ContinueStatement(*self)
    }
}

impl<'a> AstNode<'a, BreakStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn label(self) -> Option<AstNode<'a, LabelIdentifier<'a>>> {
        self.inner.label.as_ref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        if self.inner.label.as_ref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, BreakStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::BreakStatement(*self)
    }
}

impl<'a> AstNode<'a, ReturnStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument(self) -> Option<AstNode<'a, Expression<'a>>> {
        self.inner.argument.as_ref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        if self
            .inner
            .argument
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ReturnStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ReturnStatement(*self)
    }
}

impl<'a> AstNode<'a, WithStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.object)
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, Statement<'a>> {
        AstNode::new(&self.inner.body)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.object;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.body.span().start));
        }
        let node = &self.inner.body;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, WithStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::WithStatement(*self)
    }
}

impl<'a> AstNode<'a, SwitchStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn discriminant(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.discriminant)
    }

    #[inline]
    pub fn cases(self) -> AstNode<'a, ArenaVec<'a, SwitchCase<'a>>> {
        AstNode::new(&self.inner.cases)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        let node = &self.inner.discriminant;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .cases
                    .first()
                    .map_or(Following::Start(0), |n| Following::Start(n.span().start)),
            );
        }
        if let Some(following) =
            list_following(&self.inner.cases, child, child_start, || Following::Start(0))
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, SwitchStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::SwitchStatement(*self)
    }
}

impl<'a> AstNode<'a, SwitchCase<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn test(self) -> Option<AstNode<'a, Expression<'a>>> {
        self.inner.test.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn consequent(self) -> AstNode<'a, ArenaVec<'a, Statement<'a>>> {
        AstNode::new(&self.inner.consequent)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if self
            .inner
            .test
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(
                self.inner
                    .consequent
                    .first()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if let Some(following) =
            list_following(&self.inner.consequent, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, SwitchCase<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::SwitchCase(*self)
    }
}

impl<'a> AstNode<'a, LabeledStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn label(self) -> AstNode<'a, LabelIdentifier<'a>> {
        AstNode::new(&self.inner.label)
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, Statement<'a>> {
        AstNode::new(&self.inner.body)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.label;
        if node_address(node) == child {
            return Some(Following::Start(self.inner.body.span().start));
        }
        let node = &self.inner.body;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, LabeledStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::LabeledStatement(*self)
    }
}

impl<'a> AstNode<'a, ThrowStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.argument)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.argument;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ThrowStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ThrowStatement(*self)
    }
}

impl<'a> AstNode<'a, TryStatement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn block(self) -> AstNode<'a, BlockStatement<'a>> {
        AstNode::new(self.inner.block.as_ref())
    }

    #[inline]
    pub fn handler(self) -> Option<AstNode<'a, CatchClause<'a>>> {
        self.inner.handler.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn finalizer(self) -> Option<AstNode<'a, BlockStatement<'a>>> {
        self.inner.finalizer.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = self.inner.block.as_ref();
        if node_address(node) == child {
            return Some(
                self.inner
                    .handler
                    .as_deref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.finalizer.as_deref().map(|n| n.span().start))
                    .map_or(Following::Start(0), Following::Start),
            );
        }
        if self.inner.handler.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .finalizer
                    .as_deref()
                    .map_or(Following::Start(0), |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.finalizer.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TryStatement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TryStatement(*self)
    }
}

impl<'a> AstNode<'a, CatchClause<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn param(self) -> Option<AstNode<'a, CatchParameter<'a>>> {
        self.inner.param.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, BlockStatement<'a>> {
        AstNode::new(self.inner.body.as_ref())
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        if self.inner.param.as_ref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(self.inner.body.span().start));
        }
        let node = self.inner.body.as_ref();
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, CatchClause<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::CatchClause(*self)
    }
}

impl<'a> AstNode<'a, CatchParameter<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn pattern(self) -> AstNode<'a, BindingPattern<'a>> {
        AstNode::new(&self.inner.pattern)
    }

    #[inline]
    pub fn type_annotation(self) -> Option<AstNode<'a, TSTypeAnnotation<'a>>> {
        self.inner.type_annotation.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.pattern;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_annotation
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.type_annotation.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, CatchParameter<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::CatchParameter(*self)
    }
}

impl<'a> AstNode<'a, DebuggerStatement> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, DebuggerStatement> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::DebuggerStatement(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, BindingPattern<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            BindingPattern::BindingIdentifier(s) => {
                AstNodes::BindingIdentifier(AstNode::new(s.as_ref()))
            }
            BindingPattern::ObjectPattern(s) => AstNodes::ObjectPattern(AstNode::new(s.as_ref())),
            BindingPattern::ArrayPattern(s) => AstNodes::ArrayPattern(AstNode::new(s.as_ref())),
            BindingPattern::AssignmentPattern(s) => {
                AstNodes::AssignmentPattern(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, AssignmentPattern<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left(self) -> AstNode<'a, BindingPattern<'a>> {
        AstNode::new(&self.inner.left)
    }

    #[inline]
    pub fn right(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.right)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.left;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.right.span().start));
        }
        let node = &self.inner.right;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, AssignmentPattern<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::AssignmentPattern(*self)
    }
}

impl<'a> AstNode<'a, ObjectPattern<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn properties(self) -> AstNode<'a, ArenaVec<'a, BindingProperty<'a>>> {
        AstNode::new(&self.inner.properties)
    }

    #[inline]
    pub fn rest(self) -> Option<AstNode<'a, BindingRestElement<'a>>> {
        self.inner.rest.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.properties, child, child_start, || {
            self.inner
                .rest
                .as_deref()
                .map_or(Following::Start(0), |n| Following::Start(n.span().start))
        }) {
            return Some(following);
        }
        if self.inner.rest.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ObjectPattern<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ObjectPattern(*self)
    }
}

impl<'a> AstNode<'a, BindingProperty<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn key(self) -> AstNode<'a, PropertyKey<'a>> {
        AstNode::new(&self.inner.key)
    }

    #[inline]
    pub fn value(self) -> AstNode<'a, BindingPattern<'a>> {
        AstNode::new(&self.inner.value)
    }

    #[inline]
    pub fn shorthand(self) -> bool {
        self.inner.shorthand
    }

    #[inline]
    pub fn computed(self) -> bool {
        self.inner.computed
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.key;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.value.span().start));
        }
        let node = &self.inner.value;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, BindingProperty<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::BindingProperty(*self)
    }
}

impl<'a> AstNode<'a, ArrayPattern<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn elements(self) -> AstNode<'a, ArenaVec<'a, Option<BindingPattern<'a>>>> {
        AstNode::new(&self.inner.elements)
    }

    #[inline]
    pub fn rest(self) -> Option<AstNode<'a, BindingRestElement<'a>>> {
        self.inner.rest.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.elements, child, child_start, || {
            self.inner
                .rest
                .as_deref()
                .map_or(Following::Start(0), |n| Following::Start(n.span().start))
        }) {
            return Some(following);
        }
        if self.inner.rest.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ArrayPattern<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ArrayPattern(*self)
    }
}

impl<'a> AstNode<'a, BindingRestElement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument(self) -> AstNode<'a, BindingPattern<'a>> {
        AstNode::new(&self.inner.argument)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.argument;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, BindingRestElement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::BindingRestElement(*self)
    }
}

impl<'a> AstNode<'a, Function<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(self) -> FunctionType {
        self.inner.r#type
    }

    #[inline]
    pub fn id(self) -> Option<AstNode<'a, BindingIdentifier<'a>>> {
        self.inner.id.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn generator(self) -> bool {
        self.inner.generator
    }

    #[inline]
    pub fn r#async(self) -> bool {
        self.inner.r#async
    }

    #[inline]
    pub fn declare(self) -> bool {
        self.inner.declare
    }

    #[inline]
    pub fn type_parameters(self) -> Option<AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        self.inner.type_parameters.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn this_param(self) -> Option<AstNode<'a, TSThisParameter<'a>>> {
        self.inner.this_param.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn params(self) -> AstNode<'a, FormalParameters<'a>> {
        AstNode::new(self.inner.params.as_ref())
    }

    #[inline]
    pub fn return_type(self) -> Option<AstNode<'a, TSTypeAnnotation<'a>>> {
        self.inner.return_type.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn body(self) -> Option<AstNode<'a, FunctionBody<'a>>> {
        self.inner.body.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn pure(self) -> bool {
        self.inner.pure
    }

    #[inline]
    pub fn pife(self) -> bool {
        self.inner.pife
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        if self.inner.id.as_ref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .type_parameters
                    .as_deref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.this_param.as_deref().map(|n| n.span().start))
                    .map_or(Following::Start(self.inner.params.span().start), Following::Start),
            );
        }
        if self.inner.type_parameters.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .this_param
                    .as_deref()
                    .map_or(Following::Start(self.inner.params.span().start), |n| {
                        Following::Start(n.span().start)
                    }),
            );
        }
        if self.inner.this_param.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(self.inner.params.span().start));
        }
        let node = self.inner.params.as_ref();
        if node_address(node) == child {
            return Some(
                self.inner
                    .return_type
                    .as_deref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.body.as_deref().map(|n| n.span().start))
                    .map_or(Following::Inherit, Following::Start),
            );
        }
        if self.inner.return_type.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .body
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.body.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, Function<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::Function(*self)
    }
}

impl<'a> AstNode<'a, FormalParameters<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn kind(self) -> FormalParameterKind {
        self.inner.kind
    }

    #[inline]
    pub fn items(self) -> AstNode<'a, ArenaVec<'a, FormalParameter<'a>>> {
        AstNode::new(&self.inner.items)
    }

    #[inline]
    pub fn rest(self) -> Option<AstNode<'a, FormalParameterRest<'a>>> {
        self.inner.rest.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.items, child, child_start, || {
            self.inner
                .rest
                .as_deref()
                .map_or(Following::Start(0), |n| Following::Start(n.span().start))
        }) {
            return Some(following);
        }
        if self.inner.rest.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, FormalParameters<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::FormalParameters(*self)
    }
}

impl<'a> AstNode<'a, FormalParameter<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn decorators(self) -> AstNode<'a, ArenaVec<'a, Decorator<'a>>> {
        AstNode::new(&self.inner.decorators)
    }

    #[inline]
    pub fn pattern(self) -> AstNode<'a, BindingPattern<'a>> {
        AstNode::new(&self.inner.pattern)
    }

    #[inline]
    pub fn type_annotation(self) -> Option<AstNode<'a, TSTypeAnnotation<'a>>> {
        self.inner.type_annotation.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn initializer(self) -> Option<AstNode<'a, Expression<'a>>> {
        self.inner.initializer.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn optional(self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn accessibility(self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }

    #[inline]
    pub fn readonly(self) -> bool {
        self.inner.readonly
    }

    #[inline]
    pub fn r#override(self) -> bool {
        self.inner.r#override
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.decorators, child, child_start, || {
            Following::Start(self.inner.pattern.span().start)
        }) {
            return Some(following);
        }
        let node = &self.inner.pattern;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_annotation
                    .as_deref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.initializer.as_deref().map(|n| n.span().start))
                    .map_or(Following::Inherit, Following::Start),
            );
        }
        if self.inner.type_annotation.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .initializer
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self
            .inner
            .initializer
            .as_deref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, FormalParameter<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::FormalParameter(*self)
    }
}

impl<'a> AstNode<'a, FormalParameterRest<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn decorators(self) -> AstNode<'a, ArenaVec<'a, Decorator<'a>>> {
        AstNode::new(&self.inner.decorators)
    }

    #[inline]
    pub fn rest(self) -> AstNode<'a, BindingRestElement<'a>> {
        AstNode::new(&self.inner.rest)
    }

    #[inline]
    pub fn type_annotation(self) -> Option<AstNode<'a, TSTypeAnnotation<'a>>> {
        self.inner.type_annotation.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.decorators, child, child_start, || {
            Following::Start(self.inner.rest.span().start)
        }) {
            return Some(following);
        }
        let node = &self.inner.rest;
        if node_address(node) == child {
            return Some(
                self.inner
                    .type_annotation
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.type_annotation.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, FormalParameterRest<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::FormalParameterRest(*self)
    }
}

impl<'a> AstNode<'a, FunctionBody<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn directives(self) -> AstNode<'a, ArenaVec<'a, Directive<'a>>> {
        AstNode::new(&self.inner.directives)
    }

    #[inline]
    pub fn statements(self) -> AstNode<'a, ArenaVec<'a, Statement<'a>>> {
        AstNode::new(&self.inner.statements)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.directives, child, child_start, || {
            self.inner
                .statements
                .first()
                .map_or(Following::Inherit, |n| Following::Start(n.span().start))
        }) {
            return Some(following);
        }
        if let Some(following) =
            list_following(&self.inner.statements, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, FunctionBody<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::FunctionBody(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ArrowFunctionBody<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            ArrowFunctionBody::FunctionBody(s) => AstNodes::FunctionBody(AstNode::new(s.as_ref())),
            it @ match_expression!(ArrowFunctionBody) => {
                AstNode::new(it.to_expression()).as_ast_nodes()
            }
        }
    }
}

impl<'a> AstNode<'a, ArrowFunctionExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#async(self) -> bool {
        self.inner.r#async
    }

    #[inline]
    pub fn type_parameters(self) -> Option<AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        self.inner.type_parameters.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn params(self) -> AstNode<'a, FormalParameters<'a>> {
        AstNode::new(self.inner.params.as_ref())
    }

    #[inline]
    pub fn return_type(self) -> Option<AstNode<'a, TSTypeAnnotation<'a>>> {
        self.inner.return_type.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, ArrowFunctionBody<'a>> {
        AstNode::new(&self.inner.body)
    }

    #[inline]
    pub fn pure(self) -> bool {
        self.inner.pure
    }

    #[inline]
    pub fn pife(self) -> bool {
        self.inner.pife
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        if self.inner.type_parameters.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(self.inner.params.span().start));
        }
        let node = self.inner.params.as_ref();
        if node_address(node) == child {
            return Some(
                self.inner
                    .return_type
                    .as_deref()
                    .map_or(Following::Start(self.inner.body.span().start), |n| {
                        Following::Start(n.span().start)
                    }),
            );
        }
        if self.inner.return_type.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(self.inner.body.span().start));
        }
        let node = &self.inner.body;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ArrowFunctionExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ArrowFunctionExpression(*self)
    }
}

impl<'a> AstNode<'a, YieldExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn delegate(self) -> bool {
        self.inner.delegate
    }

    #[inline]
    pub fn argument(self) -> Option<AstNode<'a, Expression<'a>>> {
        self.inner.argument.as_ref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        if self
            .inner
            .argument
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, YieldExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::YieldExpression(*self)
    }
}

impl<'a> AstNode<'a, Class<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(self) -> ClassType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(self) -> AstNode<'a, ArenaVec<'a, Decorator<'a>>> {
        AstNode::new(&self.inner.decorators)
    }

    #[inline]
    pub fn id(self) -> Option<AstNode<'a, BindingIdentifier<'a>>> {
        self.inner.id.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn type_parameters(self) -> Option<AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        self.inner.type_parameters.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn heritage(self) -> Option<AstNode<'a, ClassHeritage<'a>>> {
        self.inner.heritage.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn implements(self) -> AstNode<'a, ArenaVec<'a, TSClassImplements<'a>>> {
        AstNode::new(&self.inner.implements)
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, ClassBody<'a>> {
        AstNode::new(self.inner.body.as_ref())
    }

    #[inline]
    pub fn r#abstract(self) -> bool {
        self.inner.r#abstract
    }

    #[inline]
    pub fn declare(self) -> bool {
        self.inner.declare
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.decorators, child, child_start, || {
            self.inner
                .id
                .as_ref()
                .map(|n| n.span().start)
                .or_else(|| self.inner.type_parameters.as_deref().map(|n| n.span().start))
                .or_else(|| self.inner.heritage.as_ref().map(|n| n.span().start))
                .or_else(|| self.inner.implements.first().map(|n| n.span().start))
                .map_or(Following::Start(self.inner.body.span().start), Following::Start)
        }) {
            return Some(following);
        }
        if self.inner.id.as_ref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .type_parameters
                    .as_deref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.heritage.as_ref().map(|n| n.span().start))
                    .or_else(|| self.inner.implements.first().map(|n| n.span().start))
                    .map_or(Following::Start(self.inner.body.span().start), Following::Start),
            );
        }
        if self.inner.type_parameters.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .heritage
                    .as_ref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.implements.first().map(|n| n.span().start))
                    .map_or(Following::Start(self.inner.body.span().start), Following::Start),
            );
        }
        if let Some(node) = self.inner.heritage.as_ref() {
            if node_address(node) == child {
                return Some(
                    self.inner
                        .implements
                        .first()
                        .map_or(Following::Start(self.inner.body.span().start), |n| {
                            Following::Start(n.span().start)
                        }),
                );
            }
            if let Some(following) = AstNode::new(node).following_of_child(child, child_start) {
                return Some(following.or_inherited(|| {
                    self.inner
                        .implements
                        .first()
                        .map_or(Following::Start(self.inner.body.span().start), |n| {
                            Following::Start(n.span().start)
                        })
                }));
            }
        }
        if let Some(following) = list_following(&self.inner.implements, child, child_start, || {
            Following::Start(self.inner.body.span().start)
        }) {
            return Some(following);
        }
        let node = self.inner.body.as_ref();
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, Class<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::Class(*self)
    }
}

impl<'a> AstNode<'a, ClassHeritage<'a>> {
    #[inline]
    pub fn expression(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.expression)
    }

    #[inline]
    pub fn type_arguments(self) -> Option<AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        self.inner.type_arguments.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_arguments
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.type_arguments.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    /// The node on top of the ancestor stack, as this node is not on it.
    pub fn parent(self, f: &JsFormatter<'_, 'a>) -> AstNodes<'a> {
        let parent = f.context().ancestors().current();
        debug_assert!(
            parent.following_of_child(self.address(), self.span().start).is_some(),
            "a node without a kind is not a child of the current node {}",
            parent.debug_name()
        );
        parent
    }

    pub fn following_span_start(self, f: &JsFormatter<'_, 'a>) -> u32 {
        f.context().ancestors().following_span_start_of_child(self.address(), self.span().start)
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AstNode<'a, ClassBody<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, ArenaVec<'a, ClassElement<'a>>> {
        AstNode::new(&self.inner.body)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.body, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ClassBody<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ClassBody(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ClassElement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            ClassElement::StaticBlock(s) => AstNodes::StaticBlock(AstNode::new(s.as_ref())),
            ClassElement::MethodDefinition(s) => {
                AstNodes::MethodDefinition(AstNode::new(s.as_ref()))
            }
            ClassElement::PropertyDefinition(s) => {
                AstNodes::PropertyDefinition(AstNode::new(s.as_ref()))
            }
            ClassElement::AccessorProperty(s) => {
                AstNodes::AccessorProperty(AstNode::new(s.as_ref()))
            }
            ClassElement::TSIndexSignature(s) => {
                AstNodes::TSIndexSignature(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, MethodDefinition<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(self) -> MethodDefinitionType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(self) -> AstNode<'a, ArenaVec<'a, Decorator<'a>>> {
        AstNode::new(&self.inner.decorators)
    }

    #[inline]
    pub fn key(self) -> AstNode<'a, PropertyKey<'a>> {
        AstNode::new(&self.inner.key)
    }

    #[inline]
    pub fn value(self) -> AstNode<'a, Function<'a>> {
        AstNode::new(self.inner.value.as_ref())
    }

    #[inline]
    pub fn kind(self) -> MethodDefinitionKind {
        self.inner.kind
    }

    #[inline]
    pub fn computed(self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn r#static(self) -> bool {
        self.inner.r#static
    }

    #[inline]
    pub fn r#override(self) -> bool {
        self.inner.r#override
    }

    #[inline]
    pub fn optional(self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn accessibility(self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.decorators, child, child_start, || {
            Following::Start(self.inner.key.span().start)
        }) {
            return Some(following);
        }
        let node = &self.inner.key;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.value.span().start));
        }
        let node = self.inner.value.as_ref();
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, MethodDefinition<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::MethodDefinition(*self)
    }
}

impl<'a> AstNode<'a, PropertyDefinition<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(self) -> PropertyDefinitionType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(self) -> AstNode<'a, ArenaVec<'a, Decorator<'a>>> {
        AstNode::new(&self.inner.decorators)
    }

    #[inline]
    pub fn key(self) -> AstNode<'a, PropertyKey<'a>> {
        AstNode::new(&self.inner.key)
    }

    #[inline]
    pub fn type_annotation(self) -> Option<AstNode<'a, TSTypeAnnotation<'a>>> {
        self.inner.type_annotation.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn value(self) -> Option<AstNode<'a, Expression<'a>>> {
        self.inner.value.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn computed(self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn r#static(self) -> bool {
        self.inner.r#static
    }

    #[inline]
    pub fn declare(self) -> bool {
        self.inner.declare
    }

    #[inline]
    pub fn r#override(self) -> bool {
        self.inner.r#override
    }

    #[inline]
    pub fn optional(self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn definite(self) -> bool {
        self.inner.definite
    }

    #[inline]
    pub fn readonly(self) -> bool {
        self.inner.readonly
    }

    #[inline]
    pub fn accessibility(self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.decorators, child, child_start, || {
            Following::Start(self.inner.key.span().start)
        }) {
            return Some(following);
        }
        let node = &self.inner.key;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_annotation
                    .as_deref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.value.as_ref().map(|n| n.span().start))
                    .map_or(Following::Inherit, Following::Start),
            );
        }
        if self.inner.type_annotation.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .value
                    .as_ref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self
            .inner
            .value
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, PropertyDefinition<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::PropertyDefinition(*self)
    }
}

impl<'a> AstNode<'a, PrivateIdentifier<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, PrivateIdentifier<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::PrivateIdentifier(*self)
    }
}

impl<'a> AstNode<'a, StaticBlock<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, ArenaVec<'a, Statement<'a>>> {
        AstNode::new(&self.inner.body)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.body, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, StaticBlock<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::StaticBlock(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ModuleDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            ModuleDeclaration::ImportDeclaration(s) => {
                AstNodes::ImportDeclaration(AstNode::new(s.as_ref()))
            }
            ModuleDeclaration::ExportAllDeclaration(s) => {
                AstNodes::ExportAllDeclaration(AstNode::new(s.as_ref()))
            }
            ModuleDeclaration::ExportDefaultDeclaration(s) => {
                AstNodes::ExportDefaultDeclaration(AstNode::new(s.as_ref()))
            }
            ModuleDeclaration::ExportDeclaration(s) => {
                AstNodes::ExportDeclaration(AstNode::new(s.as_ref()))
            }
            ModuleDeclaration::ExportNamedDeclaration(s) => {
                AstNodes::ExportNamedDeclaration(AstNode::new(s.as_ref()))
            }
            ModuleDeclaration::ExportFromDeclaration(s) => {
                AstNodes::ExportFromDeclaration(AstNode::new(s.as_ref()))
            }
            ModuleDeclaration::TSExportAssignment(s) => {
                AstNodes::TSExportAssignment(AstNode::new(s.as_ref()))
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(s) => {
                AstNodes::TSNamespaceExportDeclaration(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, AccessorProperty<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(self) -> AccessorPropertyType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(self) -> AstNode<'a, ArenaVec<'a, Decorator<'a>>> {
        AstNode::new(&self.inner.decorators)
    }

    #[inline]
    pub fn key(self) -> AstNode<'a, PropertyKey<'a>> {
        AstNode::new(&self.inner.key)
    }

    #[inline]
    pub fn type_annotation(self) -> Option<AstNode<'a, TSTypeAnnotation<'a>>> {
        self.inner.type_annotation.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn value(self) -> Option<AstNode<'a, Expression<'a>>> {
        self.inner.value.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn computed(self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn r#static(self) -> bool {
        self.inner.r#static
    }

    #[inline]
    pub fn r#override(self) -> bool {
        self.inner.r#override
    }

    #[inline]
    pub fn definite(self) -> bool {
        self.inner.definite
    }

    #[inline]
    pub fn accessibility(self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.decorators, child, child_start, || {
            Following::Start(self.inner.key.span().start)
        }) {
            return Some(following);
        }
        let node = &self.inner.key;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_annotation
                    .as_deref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.value.as_ref().map(|n| n.span().start))
                    .map_or(Following::Inherit, Following::Start),
            );
        }
        if self.inner.type_annotation.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .value
                    .as_ref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self
            .inner
            .value
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, AccessorProperty<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::AccessorProperty(*self)
    }
}

impl<'a> AstNode<'a, ImportExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn source(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.source)
    }

    #[inline]
    pub fn options(self) -> Option<AstNode<'a, Expression<'a>>> {
        self.inner.options.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn phase(self) -> Option<ImportPhase> {
        self.inner.phase
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.source;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .options
                    .as_ref()
                    .map_or(Following::Start(0), |n| Following::Start(n.span().start)),
            );
        }
        if self
            .inner
            .options
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ImportExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ImportExpression(*self)
    }
}

impl<'a> AstNode<'a, ImportDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn specifiers(self) -> Option<AstNode<'a, ArenaVec<'a, ImportDeclarationSpecifier<'a>>>> {
        self.inner.specifiers.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn source(self) -> AstNode<'a, StringLiteral<'a>> {
        AstNode::new(&self.inner.source)
    }

    #[inline]
    pub fn phase(self) -> Option<ImportPhase> {
        self.inner.phase
    }

    #[inline]
    pub fn with_clause(self) -> Option<AstNode<'a, WithClause<'a>>> {
        self.inner.with_clause.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn import_kind(self) -> ImportOrExportKind {
        self.inner.import_kind
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = self.inner.specifiers.as_ref().and_then(|node| {
            list_following(node, child, child_start, || {
                Following::Start(self.inner.source.span().start)
            })
        }) {
            return Some(following);
        }
        let node = &self.inner.source;
        if node_address(node) == child {
            return Some(
                self.inner
                    .with_clause
                    .as_deref()
                    .map_or(Following::Start(0), |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.with_clause.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ImportDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ImportDeclaration(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ImportDeclarationSpecifier<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                AstNodes::ImportSpecifier(AstNode::new(s.as_ref()))
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                AstNodes::ImportDefaultSpecifier(AstNode::new(s.as_ref()))
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                AstNodes::ImportNamespaceSpecifier(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, ImportSpecifier<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn imported(self) -> AstNode<'a, ModuleExportName<'a>> {
        AstNode::new(&self.inner.imported)
    }

    #[inline]
    pub fn local(self) -> AstNode<'a, BindingIdentifier<'a>> {
        AstNode::new(&self.inner.local)
    }

    #[inline]
    pub fn import_kind(self) -> ImportOrExportKind {
        self.inner.import_kind
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.imported;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.local.span().start));
        }
        let node = &self.inner.local;
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ImportSpecifier<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ImportSpecifier(*self)
    }
}

impl<'a> AstNode<'a, ImportDefaultSpecifier<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn local(self) -> AstNode<'a, BindingIdentifier<'a>> {
        AstNode::new(&self.inner.local)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.local;
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ImportDefaultSpecifier<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ImportDefaultSpecifier(*self)
    }
}

impl<'a> AstNode<'a, ImportNamespaceSpecifier<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn local(self) -> AstNode<'a, BindingIdentifier<'a>> {
        AstNode::new(&self.inner.local)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.local;
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ImportNamespaceSpecifier<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ImportNamespaceSpecifier(*self)
    }
}

impl<'a> AstNode<'a, WithClause<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn keyword(self) -> WithClauseKeyword {
        self.inner.keyword
    }

    #[inline]
    pub fn with_entries(self) -> AstNode<'a, ArenaVec<'a, ImportAttribute<'a>>> {
        AstNode::new(&self.inner.with_entries)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.with_entries, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, WithClause<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::WithClause(*self)
    }
}

impl<'a> AstNode<'a, ImportAttribute<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn key(self) -> AstNode<'a, ImportAttributeKey<'a>> {
        AstNode::new(&self.inner.key)
    }

    #[inline]
    pub fn value(self) -> AstNode<'a, StringLiteral<'a>> {
        AstNode::new(&self.inner.value)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.key;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.value.span().start));
        }
        let node = &self.inner.value;
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ImportAttribute<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ImportAttribute(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ImportAttributeKey<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            ImportAttributeKey::Identifier(s) => AstNodes::IdentifierName(AstNode::new(s)),
            ImportAttributeKey::StringLiteral(s) => AstNodes::StringLiteral(AstNode::new(s)),
        }
    }
}

impl<'a> AstNode<'a, ExportDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn declaration(self) -> AstNode<'a, Declaration<'a>> {
        AstNode::new(&self.inner.declaration)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.declaration;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ExportDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ExportDeclaration(*self)
    }
}

impl<'a> AstNode<'a, ExportNamedDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn specifiers(self) -> AstNode<'a, ArenaVec<'a, ExportSpecifier<'a>>> {
        AstNode::new(&self.inner.specifiers)
    }

    #[inline]
    pub fn export_kind(self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.specifiers, child, child_start, || Following::Start(0))
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ExportNamedDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ExportNamedDeclaration(*self)
    }
}

impl<'a> AstNode<'a, ExportFromDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn specifiers(self) -> AstNode<'a, ArenaVec<'a, ExportSpecifier<'a>>> {
        AstNode::new(&self.inner.specifiers)
    }

    #[inline]
    pub fn source(self) -> AstNode<'a, StringLiteral<'a>> {
        AstNode::new(&self.inner.source)
    }

    #[inline]
    pub fn export_kind(self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    #[inline]
    pub fn with_clause(self) -> Option<AstNode<'a, WithClause<'a>>> {
        self.inner.with_clause.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.specifiers, child, child_start, || {
            Following::Start(self.inner.source.span().start)
        }) {
            return Some(following);
        }
        let node = &self.inner.source;
        if node_address(node) == child {
            return Some(
                self.inner
                    .with_clause
                    .as_deref()
                    .map_or(Following::Start(0), |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.with_clause.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ExportFromDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ExportFromDeclaration(*self)
    }
}

impl<'a> AstNode<'a, ExportDefaultDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn declaration(self) -> AstNode<'a, ExportDefaultDeclarationKind<'a>> {
        AstNode::new(&self.inner.declaration)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.declaration;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ExportDefaultDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ExportDefaultDeclaration(*self)
    }
}

impl<'a> AstNode<'a, ExportAllDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn exported(self) -> Option<AstNode<'a, ModuleExportName<'a>>> {
        self.inner.exported.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn source(self) -> AstNode<'a, StringLiteral<'a>> {
        AstNode::new(&self.inner.source)
    }

    #[inline]
    pub fn with_clause(self) -> Option<AstNode<'a, WithClause<'a>>> {
        self.inner.with_clause.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn export_kind(self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        if self
            .inner
            .exported
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Start(self.inner.source.span().start));
        }
        let node = &self.inner.source;
        if node_address(node) == child {
            return Some(
                self.inner
                    .with_clause
                    .as_deref()
                    .map_or(Following::Start(0), |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.with_clause.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ExportAllDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ExportAllDeclaration(*self)
    }
}

impl<'a> AstNode<'a, ExportSpecifier<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn local(self) -> AstNode<'a, ModuleExportName<'a>> {
        AstNode::new(&self.inner.local)
    }

    #[inline]
    pub fn exported(self) -> AstNode<'a, ModuleExportName<'a>> {
        AstNode::new(&self.inner.exported)
    }

    #[inline]
    pub fn export_kind(self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.local;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.exported.span().start));
        }
        let node = &self.inner.exported;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ExportSpecifier<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::ExportSpecifier(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ExportDefaultDeclarationKind<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            ExportDefaultDeclarationKind::FunctionDeclaration(s) => {
                AstNodes::Function(AstNode::new(s.as_ref()))
            }
            ExportDefaultDeclarationKind::ClassDeclaration(s) => {
                AstNodes::Class(AstNode::new(s.as_ref()))
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(s) => {
                AstNodes::TSInterfaceDeclaration(AstNode::new(s.as_ref()))
            }
            it @ match_expression!(ExportDefaultDeclarationKind) => {
                AstNode::new(it.to_expression()).as_ast_nodes()
            }
        }
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, ModuleExportName<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            ModuleExportName::IdentifierName(s) => AstNodes::IdentifierName(AstNode::new(s)),
            ModuleExportName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(AstNode::new(s))
            }
            ModuleExportName::StringLiteral(s) => AstNodes::StringLiteral(AstNode::new(s)),
        }
    }
}

impl<'a> AstNode<'a, V8IntrinsicExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(self) -> AstNode<'a, IdentifierName<'a>> {
        AstNode::new(&self.inner.name)
    }

    #[inline]
    pub fn arguments(self) -> AstNode<'a, ArenaVec<'a, Argument<'a>>> {
        AstNode::new(&self.inner.arguments)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        let node = &self.inner.name;
        if node_address(node) == child {
            return Some(
                self.inner
                    .arguments
                    .first()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if let Some(following) =
            list_following(&self.inner.arguments, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, V8IntrinsicExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::V8IntrinsicExpression(*self)
    }
}

impl<'a> AstNode<'a, BooleanLiteral> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(self) -> bool {
        self.inner.value
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, BooleanLiteral> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::BooleanLiteral(*self)
    }
}

impl<'a> AstNode<'a, NullLiteral> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, NullLiteral> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::NullLiteral(*self)
    }
}

impl<'a> AstNode<'a, NumericLiteral<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(self) -> f64 {
        self.inner.value
    }

    #[inline]
    pub fn raw(self) -> Option<Str<'a>> {
        self.inner.raw
    }

    #[inline]
    pub fn base(self) -> NumberBase {
        self.inner.base
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, NumericLiteral<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::NumericLiteral(*self)
    }
}

impl<'a> AstNode<'a, StringLiteral<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(self) -> JSStr<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(self) -> Option<Str<'a>> {
        self.inner.raw
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, StringLiteral<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::StringLiteral(*self)
    }
}

impl<'a> AstNode<'a, BigIntLiteral<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(self) -> Str<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(self) -> Option<Str<'a>> {
        self.inner.raw
    }

    #[inline]
    pub fn base(self) -> BigintBase {
        self.inner.base
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, BigIntLiteral<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::BigIntLiteral(*self)
    }
}

impl<'a> AstNode<'a, RegExpLiteral<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn regex(self) -> &'a RegExp<'a> {
        &self.inner.regex
    }

    #[inline]
    pub fn raw(self) -> Option<Str<'a>> {
        self.inner.raw
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, RegExpLiteral<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::RegExpLiteral(*self)
    }
}

impl<'a> AstNode<'a, JSXElement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn opening_element(self) -> AstNode<'a, JSXOpeningElement<'a>> {
        AstNode::new(self.inner.opening_element.as_ref())
    }

    #[inline]
    pub fn children(self) -> AstNode<'a, ArenaVec<'a, JSXChild<'a>>> {
        AstNode::new(&self.inner.children)
    }

    #[inline]
    pub fn closing_element(self) -> Option<AstNode<'a, JSXClosingElement<'a>>> {
        self.inner.closing_element.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        let node = self.inner.opening_element.as_ref();
        if node_address(node) == child {
            return Some(
                self.inner
                    .children
                    .first()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.closing_element.as_deref().map(|n| n.span().start))
                    .map_or(Following::Inherit, Following::Start),
            );
        }
        if let Some(following) = list_following(&self.inner.children, child, child_start, || {
            self.inner
                .closing_element
                .as_deref()
                .map_or(Following::Inherit, |n| Following::Start(n.span().start))
        }) {
            return Some(following);
        }
        if self.inner.closing_element.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXElement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXElement(*self)
    }
}

impl<'a> AstNode<'a, JSXOpeningElement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(self) -> AstNode<'a, JSXElementName<'a>> {
        AstNode::new(&self.inner.name)
    }

    #[inline]
    pub fn type_arguments(self) -> Option<AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        self.inner.type_arguments.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn attributes(self) -> AstNode<'a, ArenaVec<'a, JSXAttributeItem<'a>>> {
        AstNode::new(&self.inner.attributes)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        let node = &self.inner.name;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_arguments
                    .as_deref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.attributes.first().map(|n| n.span().start))
                    .map_or(Following::Inherit, Following::Start),
            );
        }
        if self.inner.type_arguments.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .attributes
                    .first()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if let Some(following) =
            list_following(&self.inner.attributes, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXOpeningElement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXOpeningElement(*self)
    }
}

impl<'a> AstNode<'a, JSXClosingElement<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(self) -> AstNode<'a, JSXElementName<'a>> {
        AstNode::new(&self.inner.name)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.name;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXClosingElement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXClosingElement(*self)
    }
}

impl<'a> AstNode<'a, JSXFragment<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn opening_fragment(self) -> AstNode<'a, JSXOpeningFragment> {
        AstNode::new(&self.inner.opening_fragment)
    }

    #[inline]
    pub fn children(self) -> AstNode<'a, ArenaVec<'a, JSXChild<'a>>> {
        AstNode::new(&self.inner.children)
    }

    #[inline]
    pub fn closing_fragment(self) -> AstNode<'a, JSXClosingFragment> {
        AstNode::new(&self.inner.closing_fragment)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        let node = &self.inner.opening_fragment;
        if node_address(node) == child {
            return Some(
                self.inner
                    .children
                    .first()
                    .map_or(Following::Start(self.inner.closing_fragment.span().start), |n| {
                        Following::Start(n.span().start)
                    }),
            );
        }
        if let Some(following) = list_following(&self.inner.children, child, child_start, || {
            Following::Start(self.inner.closing_fragment.span().start)
        }) {
            return Some(following);
        }
        let node = &self.inner.closing_fragment;
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXFragment<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXFragment(*self)
    }
}

impl<'a> AstNode<'a, JSXOpeningFragment> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXOpeningFragment> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXOpeningFragment(*self)
    }
}

impl<'a> AstNode<'a, JSXClosingFragment> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXClosingFragment> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXClosingFragment(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXElementName<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            JSXElementName::Identifier(s) => AstNodes::JSXIdentifier(AstNode::new(s.as_ref())),
            JSXElementName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(AstNode::new(s.as_ref()))
            }
            JSXElementName::NamespacedName(s) => {
                AstNodes::JSXNamespacedName(AstNode::new(s.as_ref()))
            }
            JSXElementName::MemberExpression(s) => {
                AstNodes::JSXMemberExpression(AstNode::new(s.as_ref()))
            }
            JSXElementName::ThisExpression(s) => AstNodes::ThisExpression(AstNode::new(s.as_ref())),
        }
    }
}

impl<'a> AstNode<'a, JSXNamespacedName<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn namespace(self) -> AstNode<'a, JSXIdentifier<'a>> {
        AstNode::new(&self.inner.namespace)
    }

    #[inline]
    pub fn name(self) -> AstNode<'a, JSXIdentifier<'a>> {
        AstNode::new(&self.inner.name)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.namespace;
        if node_address(node) == child {
            return Some(Following::Start(self.inner.name.span().start));
        }
        let node = &self.inner.name;
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXNamespacedName<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXNamespacedName(*self)
    }
}

impl<'a> AstNode<'a, JSXMemberExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object(self) -> AstNode<'a, JSXMemberExpressionObject<'a>> {
        AstNode::new(&self.inner.object)
    }

    #[inline]
    pub fn property(self) -> AstNode<'a, JSXIdentifier<'a>> {
        AstNode::new(&self.inner.property)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.object;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.property.span().start));
        }
        let node = &self.inner.property;
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXMemberExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXMemberExpression(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXMemberExpressionObject<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            JSXMemberExpressionObject::IdentifierReference(s) => {
                AstNodes::IdentifierReference(AstNode::new(s.as_ref()))
            }
            JSXMemberExpressionObject::MemberExpression(s) => {
                AstNodes::JSXMemberExpression(AstNode::new(s.as_ref()))
            }
            JSXMemberExpressionObject::ThisExpression(s) => {
                AstNodes::ThisExpression(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, JSXExpressionContainer<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, JSXExpression<'a>> {
        AstNode::new(&self.inner.expression)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXExpressionContainer<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXExpressionContainer(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            JSXExpression::EmptyExpression(s) => {
                AstNodes::JSXEmptyExpression(AstNode::new(s.as_ref()))
            }
            it @ match_expression!(JSXExpression) => {
                AstNode::new(it.to_expression()).as_ast_nodes()
            }
        }
    }
}

impl<'a> AstNode<'a, JSXEmptyExpression> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXEmptyExpression> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXEmptyExpression(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXAttributeItem<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            JSXAttributeItem::Attribute(s) => AstNodes::JSXAttribute(AstNode::new(s.as_ref())),
            JSXAttributeItem::SpreadAttribute(s) => {
                AstNodes::JSXSpreadAttribute(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, JSXAttribute<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(self) -> AstNode<'a, JSXAttributeName<'a>> {
        AstNode::new(&self.inner.name)
    }

    #[inline]
    pub fn value(self) -> Option<AstNode<'a, JSXAttributeValue<'a>>> {
        self.inner.value.as_ref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.name;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .value
                    .as_ref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self
            .inner
            .value
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXAttribute<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXAttribute(*self)
    }
}

impl<'a> AstNode<'a, JSXSpreadAttribute<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.argument)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.argument;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXSpreadAttribute<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXSpreadAttribute(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXAttributeName<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            JSXAttributeName::Identifier(s) => AstNodes::JSXIdentifier(AstNode::new(s.as_ref())),
            JSXAttributeName::NamespacedName(s) => {
                AstNodes::JSXNamespacedName(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXAttributeValue<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            JSXAttributeValue::StringLiteral(s) => {
                AstNodes::StringLiteral(AstNode::new(s.as_ref()))
            }
            JSXAttributeValue::ExpressionContainer(s) => {
                AstNodes::JSXExpressionContainer(AstNode::new(s.as_ref()))
            }
            JSXAttributeValue::Element(s) => AstNodes::JSXElement(AstNode::new(s.as_ref())),
            JSXAttributeValue::Fragment(s) => AstNodes::JSXFragment(AstNode::new(s.as_ref())),
        }
    }
}

impl<'a> AstNode<'a, JSXIdentifier<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(self) -> Str<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXIdentifier<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXIdentifier(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXChild<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            JSXChild::Text(s) => AstNodes::JSXText(AstNode::new(s.as_ref())),
            JSXChild::Element(s) => AstNodes::JSXElement(AstNode::new(s.as_ref())),
            JSXChild::Fragment(s) => AstNodes::JSXFragment(AstNode::new(s.as_ref())),
            JSXChild::ExpressionContainer(s) => {
                AstNodes::JSXExpressionContainer(AstNode::new(s.as_ref()))
            }
            JSXChild::Spread(s) => AstNodes::JSXSpreadChild(AstNode::new(s.as_ref())),
        }
    }
}

impl<'a> AstNode<'a, JSXSpreadChild<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.expression)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXSpreadChild<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXSpreadChild(*self)
    }
}

impl<'a> AstNode<'a, JSXText<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(self) -> Str<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(self) -> Option<Str<'a>> {
        self.inner.raw
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSXText<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSXText(*self)
    }
}

impl<'a> AstNode<'a, TSThisParameter<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn this_span(self) -> Span {
        self.inner.this_span
    }

    #[inline]
    pub fn type_annotation(self) -> Option<AstNode<'a, TSTypeAnnotation<'a>>> {
        self.inner.type_annotation.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        if self.inner.type_annotation.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSThisParameter<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSThisParameter(*self)
    }
}

impl<'a> AstNode<'a, TSEnumDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id(self) -> AstNode<'a, BindingIdentifier<'a>> {
        AstNode::new(&self.inner.id)
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, TSEnumBody<'a>> {
        AstNode::new(&self.inner.body)
    }

    #[inline]
    pub fn r#const(self) -> bool {
        self.inner.r#const
    }

    #[inline]
    pub fn declare(self) -> bool {
        self.inner.declare
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.id;
        if node_address(node) == child {
            return Some(Following::Start(self.inner.body.span().start));
        }
        let node = &self.inner.body;
        if node_address(node) == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSEnumDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSEnumDeclaration(*self)
    }
}

impl<'a> AstNode<'a, TSEnumBody<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn members(self) -> AstNode<'a, ArenaVec<'a, TSEnumMember<'a>>> {
        AstNode::new(&self.inner.members)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.members, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSEnumBody<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSEnumBody(*self)
    }
}

impl<'a> AstNode<'a, TSEnumMember<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id(self) -> AstNode<'a, TSEnumMemberName<'a>> {
        AstNode::new(&self.inner.id)
    }

    #[inline]
    pub fn initializer(self) -> Option<AstNode<'a, Expression<'a>>> {
        self.inner.initializer.as_ref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.id;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .initializer
                    .as_ref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self
            .inner
            .initializer
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSEnumMember<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSEnumMember(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSEnumMemberName<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            TSEnumMemberName::Identifier(s) => AstNodes::IdentifierName(AstNode::new(s.as_ref())),
            TSEnumMemberName::String(s) => AstNodes::StringLiteral(AstNode::new(s.as_ref())),
            TSEnumMemberName::ComputedString(s) => {
                AstNodes::StringLiteral(AstNode::new(s.as_ref()))
            }
            TSEnumMemberName::ComputedTemplateString(s) => {
                AstNodes::TemplateLiteral(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, TSTypeAnnotation<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.type_annotation)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.type_annotation;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTypeAnnotation<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSTypeAnnotation(*self)
    }
}

impl<'a> AstNode<'a, TSLiteralType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn literal(self) -> AstNode<'a, TSLiteral<'a>> {
        AstNode::new(&self.inner.literal)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.literal;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSLiteralType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSLiteralType(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSLiteral<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            TSLiteral::BooleanLiteral(s) => AstNodes::BooleanLiteral(AstNode::new(s.as_ref())),
            TSLiteral::NumericLiteral(s) => AstNodes::NumericLiteral(AstNode::new(s.as_ref())),
            TSLiteral::BigIntLiteral(s) => AstNodes::BigIntLiteral(AstNode::new(s.as_ref())),
            TSLiteral::StringLiteral(s) => AstNodes::StringLiteral(AstNode::new(s.as_ref())),
            TSLiteral::TemplateLiteral(s) => AstNodes::TemplateLiteral(AstNode::new(s.as_ref())),
            TSLiteral::UnaryExpression(s) => AstNodes::UnaryExpression(AstNode::new(s.as_ref())),
        }
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            TSType::TSAnyKeyword(s) => AstNodes::TSAnyKeyword(AstNode::new(s.as_ref())),
            TSType::TSBigIntKeyword(s) => AstNodes::TSBigIntKeyword(AstNode::new(s.as_ref())),
            TSType::TSBooleanKeyword(s) => AstNodes::TSBooleanKeyword(AstNode::new(s.as_ref())),
            TSType::TSIntrinsicKeyword(s) => AstNodes::TSIntrinsicKeyword(AstNode::new(s.as_ref())),
            TSType::TSNeverKeyword(s) => AstNodes::TSNeverKeyword(AstNode::new(s.as_ref())),
            TSType::TSNullKeyword(s) => AstNodes::TSNullKeyword(AstNode::new(s.as_ref())),
            TSType::TSNumberKeyword(s) => AstNodes::TSNumberKeyword(AstNode::new(s.as_ref())),
            TSType::TSObjectKeyword(s) => AstNodes::TSObjectKeyword(AstNode::new(s.as_ref())),
            TSType::TSStringKeyword(s) => AstNodes::TSStringKeyword(AstNode::new(s.as_ref())),
            TSType::TSSymbolKeyword(s) => AstNodes::TSSymbolKeyword(AstNode::new(s.as_ref())),
            TSType::TSUndefinedKeyword(s) => AstNodes::TSUndefinedKeyword(AstNode::new(s.as_ref())),
            TSType::TSUnknownKeyword(s) => AstNodes::TSUnknownKeyword(AstNode::new(s.as_ref())),
            TSType::TSVoidKeyword(s) => AstNodes::TSVoidKeyword(AstNode::new(s.as_ref())),
            TSType::TSArrayType(s) => AstNodes::TSArrayType(AstNode::new(s.as_ref())),
            TSType::TSConditionalType(s) => AstNodes::TSConditionalType(AstNode::new(s.as_ref())),
            TSType::TSConstructorType(s) => AstNodes::TSConstructorType(AstNode::new(s.as_ref())),
            TSType::TSFunctionType(s) => AstNodes::TSFunctionType(AstNode::new(s.as_ref())),
            TSType::TSImportType(s) => AstNodes::TSImportType(AstNode::new(s.as_ref())),
            TSType::TSIndexedAccessType(s) => {
                AstNodes::TSIndexedAccessType(AstNode::new(s.as_ref()))
            }
            TSType::TSInferType(s) => AstNodes::TSInferType(AstNode::new(s.as_ref())),
            TSType::TSIntersectionType(s) => AstNodes::TSIntersectionType(AstNode::new(s.as_ref())),
            TSType::TSLiteralType(s) => AstNodes::TSLiteralType(AstNode::new(s.as_ref())),
            TSType::TSMappedType(s) => AstNodes::TSMappedType(AstNode::new(s.as_ref())),
            TSType::TSNamedTupleMember(s) => AstNodes::TSNamedTupleMember(AstNode::new(s.as_ref())),
            TSType::TSTemplateLiteralType(s) => {
                AstNodes::TSTemplateLiteralType(AstNode::new(s.as_ref()))
            }
            TSType::TSThisType(s) => AstNodes::TSThisType(AstNode::new(s.as_ref())),
            TSType::TSTupleType(s) => AstNodes::TSTupleType(AstNode::new(s.as_ref())),
            TSType::TSTypeLiteral(s) => AstNodes::TSTypeLiteral(AstNode::new(s.as_ref())),
            TSType::TSTypeOperatorType(s) => AstNodes::TSTypeOperator(AstNode::new(s.as_ref())),
            TSType::TSTypePredicate(s) => AstNodes::TSTypePredicate(AstNode::new(s.as_ref())),
            TSType::TSTypeQuery(s) => AstNodes::TSTypeQuery(AstNode::new(s.as_ref())),
            TSType::TSTypeReference(s) => AstNodes::TSTypeReference(AstNode::new(s.as_ref())),
            TSType::TSUnionType(s) => AstNodes::TSUnionType(AstNode::new(s.as_ref())),
            TSType::TSParenthesizedType(s) => {
                AstNodes::TSParenthesizedType(AstNode::new(s.as_ref()))
            }
            TSType::JSDocNullableType(s) => AstNodes::JSDocNullableType(AstNode::new(s.as_ref())),
            TSType::JSDocNonNullableType(s) => {
                AstNodes::JSDocNonNullableType(AstNode::new(s.as_ref()))
            }
            TSType::JSDocUnknownType(s) => AstNodes::JSDocUnknownType(AstNode::new(s.as_ref())),
        }
    }
}

impl<'a> AstNode<'a, TSConditionalType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn check_type(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.check_type)
    }

    #[inline]
    pub fn extends_type(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.extends_type)
    }

    #[inline]
    pub fn true_type(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.true_type)
    }

    #[inline]
    pub fn false_type(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.false_type)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.check_type;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.extends_type.span().start));
        }
        let node = &self.inner.extends_type;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.true_type.span().start));
        }
        let node = &self.inner.true_type;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.false_type.span().start));
        }
        let node = &self.inner.false_type;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSConditionalType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSConditionalType(*self)
    }
}

impl<'a> AstNode<'a, TSUnionType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn types(self) -> AstNode<'a, ArenaVec<'a, TSType<'a>>> {
        AstNode::new(&self.inner.types)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.types, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSUnionType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSUnionType(*self)
    }
}

impl<'a> AstNode<'a, TSIntersectionType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn types(self) -> AstNode<'a, ArenaVec<'a, TSType<'a>>> {
        AstNode::new(&self.inner.types)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.types, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSIntersectionType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSIntersectionType(*self)
    }
}

impl<'a> AstNode<'a, TSParenthesizedType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.type_annotation)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.type_annotation;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSParenthesizedType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSParenthesizedType(*self)
    }
}

impl<'a> AstNode<'a, TSTypeOperator<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn operator(self) -> TSTypeOperatorOperator {
        self.inner.operator
    }

    #[inline]
    pub fn type_annotation(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.type_annotation)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.type_annotation;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTypeOperator<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSTypeOperator(*self)
    }
}

impl<'a> AstNode<'a, TSArrayType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn element_type(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.element_type)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.element_type;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSArrayType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSArrayType(*self)
    }
}

impl<'a> AstNode<'a, TSIndexedAccessType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object_type(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.object_type)
    }

    #[inline]
    pub fn index_type(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.index_type)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.object_type;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.index_type.span().start));
        }
        let node = &self.inner.index_type;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSIndexedAccessType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSIndexedAccessType(*self)
    }
}

impl<'a> AstNode<'a, TSTupleType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn element_types(self) -> AstNode<'a, ArenaVec<'a, TSTupleElement<'a>>> {
        AstNode::new(&self.inner.element_types)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.element_types, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTupleType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSTupleType(*self)
    }
}

impl<'a> AstNode<'a, TSNamedTupleMember<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn label(self) -> AstNode<'a, IdentifierName<'a>> {
        AstNode::new(&self.inner.label)
    }

    #[inline]
    pub fn element_type(self) -> AstNode<'a, TSTupleElement<'a>> {
        AstNode::new(&self.inner.element_type)
    }

    #[inline]
    pub fn optional(self) -> bool {
        self.inner.optional
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.label;
        if node_address(node) == child {
            return Some(Following::Start(self.inner.element_type.span().start));
        }
        let node = &self.inner.element_type;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSNamedTupleMember<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSNamedTupleMember(*self)
    }
}

impl<'a> AstNode<'a, TSOptionalType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.type_annotation)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.type_annotation;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSOptionalType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSOptionalType(*self)
    }
}

impl<'a> AstNode<'a, TSRestType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.type_annotation)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.type_annotation;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSRestType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSRestType(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTupleElement<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            TSTupleElement::TSOptionalType(s) => AstNodes::TSOptionalType(AstNode::new(s.as_ref())),
            TSTupleElement::TSRestType(s) => AstNodes::TSRestType(AstNode::new(s.as_ref())),
            it @ match_ts_type!(TSTupleElement) => AstNode::new(it.to_ts_type()).as_ast_nodes(),
        }
    }
}

impl<'a> AstNode<'a, TSAnyKeyword> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSAnyKeyword> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSAnyKeyword(*self)
    }
}

impl<'a> AstNode<'a, TSStringKeyword> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSStringKeyword> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSStringKeyword(*self)
    }
}

impl<'a> AstNode<'a, TSBooleanKeyword> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSBooleanKeyword> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSBooleanKeyword(*self)
    }
}

impl<'a> AstNode<'a, TSNumberKeyword> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSNumberKeyword> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSNumberKeyword(*self)
    }
}

impl<'a> AstNode<'a, TSNeverKeyword> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSNeverKeyword> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSNeverKeyword(*self)
    }
}

impl<'a> AstNode<'a, TSIntrinsicKeyword> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSIntrinsicKeyword> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSIntrinsicKeyword(*self)
    }
}

impl<'a> AstNode<'a, TSUnknownKeyword> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSUnknownKeyword> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSUnknownKeyword(*self)
    }
}

impl<'a> AstNode<'a, TSNullKeyword> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSNullKeyword> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSNullKeyword(*self)
    }
}

impl<'a> AstNode<'a, TSUndefinedKeyword> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSUndefinedKeyword> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSUndefinedKeyword(*self)
    }
}

impl<'a> AstNode<'a, TSVoidKeyword> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSVoidKeyword> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSVoidKeyword(*self)
    }
}

impl<'a> AstNode<'a, TSSymbolKeyword> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSSymbolKeyword> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSSymbolKeyword(*self)
    }
}

impl<'a> AstNode<'a, TSThisType> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSThisType> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSThisType(*self)
    }
}

impl<'a> AstNode<'a, TSObjectKeyword> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSObjectKeyword> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSObjectKeyword(*self)
    }
}

impl<'a> AstNode<'a, TSBigIntKeyword> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSBigIntKeyword> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSBigIntKeyword(*self)
    }
}

impl<'a> AstNode<'a, TSTypeReference<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_name(self) -> AstNode<'a, TSTypeName<'a>> {
        AstNode::new(&self.inner.type_name)
    }

    #[inline]
    pub fn type_arguments(self) -> Option<AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        self.inner.type_arguments.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.type_name;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_arguments
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.type_arguments.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTypeReference<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSTypeReference(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTypeName<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            TSTypeName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(AstNode::new(s.as_ref()))
            }
            TSTypeName::QualifiedName(s) => AstNodes::TSQualifiedName(AstNode::new(s.as_ref())),
            TSTypeName::ThisExpression(s) => AstNodes::ThisExpression(AstNode::new(s.as_ref())),
        }
    }
}

impl<'a> AstNode<'a, TSQualifiedName<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left(self) -> AstNode<'a, TSTypeName<'a>> {
        AstNode::new(&self.inner.left)
    }

    #[inline]
    pub fn right(self) -> AstNode<'a, IdentifierName<'a>> {
        AstNode::new(&self.inner.right)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.left;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.right.span().start));
        }
        let node = &self.inner.right;
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSQualifiedName<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSQualifiedName(*self)
    }
}

impl<'a> AstNode<'a, TSTypeParameterInstantiation<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn params(self) -> AstNode<'a, ArenaVec<'a, TSType<'a>>> {
        AstNode::new(&self.inner.params)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.params, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTypeParameterInstantiation<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSTypeParameterInstantiation(*self)
    }
}

impl<'a> AstNode<'a, TSTypeParameter<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(self) -> AstNode<'a, BindingIdentifier<'a>> {
        AstNode::new(&self.inner.name)
    }

    #[inline]
    pub fn constraint(self) -> Option<AstNode<'a, TSType<'a>>> {
        self.inner.constraint.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn default(self) -> Option<AstNode<'a, TSType<'a>>> {
        self.inner.default.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn r#in(self) -> bool {
        self.inner.r#in
    }

    #[inline]
    pub fn out(self) -> bool {
        self.inner.out
    }

    #[inline]
    pub fn r#const(self) -> bool {
        self.inner.r#const
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.name;
        if node_address(node) == child {
            return Some(
                self.inner
                    .constraint
                    .as_ref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.default.as_ref().map(|n| n.span().start))
                    .map_or(Following::Inherit, Following::Start),
            );
        }
        if self
            .inner
            .constraint
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(
                self.inner
                    .default
                    .as_ref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self
            .inner
            .default
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTypeParameter<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSTypeParameter(*self)
    }
}

impl<'a> AstNode<'a, TSTypeParameterDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn params(self) -> AstNode<'a, ArenaVec<'a, TSTypeParameter<'a>>> {
        AstNode::new(&self.inner.params)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.params, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTypeParameterDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSTypeParameterDeclaration(*self)
    }
}

impl<'a> AstNode<'a, TSTypeAliasDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id(self) -> AstNode<'a, BindingIdentifier<'a>> {
        AstNode::new(&self.inner.id)
    }

    #[inline]
    pub fn type_parameters(self) -> Option<AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        self.inner.type_parameters.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn type_annotation(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.type_annotation)
    }

    #[inline]
    pub fn declare(self) -> bool {
        self.inner.declare
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.id;
        if node_address(node) == child {
            return Some(
                self.inner
                    .type_parameters
                    .as_deref()
                    .map_or(Following::Start(self.inner.type_annotation.span().start), |n| {
                        Following::Start(n.span().start)
                    }),
            );
        }
        if self.inner.type_parameters.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(self.inner.type_annotation.span().start));
        }
        let node = &self.inner.type_annotation;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTypeAliasDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSTypeAliasDeclaration(*self)
    }
}

impl<'a> AstNode<'a, TSClassImplements<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, TSTypeName<'a>> {
        AstNode::new(&self.inner.expression)
    }

    #[inline]
    pub fn type_arguments(self) -> Option<AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        self.inner.type_arguments.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_arguments
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.type_arguments.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSClassImplements<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSClassImplements(*self)
    }
}

impl<'a> AstNode<'a, TSInterfaceDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id(self) -> AstNode<'a, BindingIdentifier<'a>> {
        AstNode::new(&self.inner.id)
    }

    #[inline]
    pub fn type_parameters(self) -> Option<AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        self.inner.type_parameters.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn extends(self) -> AstNode<'a, ArenaVec<'a, TSInterfaceHeritage<'a>>> {
        AstNode::new(&self.inner.extends)
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, TSInterfaceBody<'a>> {
        AstNode::new(self.inner.body.as_ref())
    }

    #[inline]
    pub fn declare(self) -> bool {
        self.inner.declare
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        let node = &self.inner.id;
        if node_address(node) == child {
            return Some(
                self.inner
                    .type_parameters
                    .as_deref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.extends.first().map(|n| n.span().start))
                    .map_or(Following::Start(self.inner.body.span().start), Following::Start),
            );
        }
        if self.inner.type_parameters.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .extends
                    .first()
                    .map_or(Following::Start(self.inner.body.span().start), |n| {
                        Following::Start(n.span().start)
                    }),
            );
        }
        if let Some(following) = list_following(&self.inner.extends, child, child_start, || {
            Following::Start(self.inner.body.span().start)
        }) {
            return Some(following);
        }
        let node = self.inner.body.as_ref();
        if node_address(node) == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSInterfaceDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSInterfaceDeclaration(*self)
    }
}

impl<'a> AstNode<'a, TSInterfaceBody<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, ArenaVec<'a, TSSignature<'a>>> {
        AstNode::new(&self.inner.body)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.body, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSInterfaceBody<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSInterfaceBody(*self)
    }
}

impl<'a> AstNode<'a, TSPropertySignature<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn computed(self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn optional(self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn readonly(self) -> bool {
        self.inner.readonly
    }

    #[inline]
    pub fn key(self) -> AstNode<'a, PropertyKey<'a>> {
        AstNode::new(&self.inner.key)
    }

    #[inline]
    pub fn type_annotation(self) -> Option<AstNode<'a, TSTypeAnnotation<'a>>> {
        self.inner.type_annotation.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.key;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_annotation
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.type_annotation.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSPropertySignature<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSPropertySignature(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSSignature<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            TSSignature::TSIndexSignature(s) => {
                AstNodes::TSIndexSignature(AstNode::new(s.as_ref()))
            }
            TSSignature::TSPropertySignature(s) => {
                AstNodes::TSPropertySignature(AstNode::new(s.as_ref()))
            }
            TSSignature::TSCallSignatureDeclaration(s) => {
                AstNodes::TSCallSignatureDeclaration(AstNode::new(s.as_ref()))
            }
            TSSignature::TSConstructSignatureDeclaration(s) => {
                AstNodes::TSConstructSignatureDeclaration(AstNode::new(s.as_ref()))
            }
            TSSignature::TSMethodSignature(s) => {
                AstNodes::TSMethodSignature(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, TSIndexSignature<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn parameter(self) -> AstNode<'a, TSIndexSignatureName<'a>> {
        AstNode::new(&self.inner.parameter)
    }

    #[inline]
    pub fn type_annotation(self) -> AstNode<'a, TSTypeAnnotation<'a>> {
        AstNode::new(self.inner.type_annotation.as_ref())
    }

    #[inline]
    pub fn readonly(self) -> bool {
        self.inner.readonly
    }

    #[inline]
    pub fn r#static(self) -> bool {
        self.inner.r#static
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.parameter;
        if node_address(node) == child {
            return Some(Following::Start(self.inner.type_annotation.span().start));
        }
        let node = self.inner.type_annotation.as_ref();
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSIndexSignature<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSIndexSignature(*self)
    }
}

impl<'a> AstNode<'a, TSCallSignatureDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_parameters(self) -> Option<AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        self.inner.type_parameters.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn this_param(self) -> Option<AstNode<'a, TSThisParameter<'a>>> {
        self.inner.this_param.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn params(self) -> AstNode<'a, FormalParameters<'a>> {
        AstNode::new(self.inner.params.as_ref())
    }

    #[inline]
    pub fn return_type(self) -> Option<AstNode<'a, TSTypeAnnotation<'a>>> {
        self.inner.return_type.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        if self.inner.type_parameters.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .this_param
                    .as_deref()
                    .map_or(Following::Start(self.inner.params.span().start), |n| {
                        Following::Start(n.span().start)
                    }),
            );
        }
        if self.inner.this_param.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(self.inner.params.span().start));
        }
        let node = self.inner.params.as_ref();
        if node_address(node) == child {
            return Some(
                self.inner
                    .return_type
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.return_type.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSCallSignatureDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSCallSignatureDeclaration(*self)
    }
}

impl<'a> AstNode<'a, TSMethodSignature<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn key(self) -> AstNode<'a, PropertyKey<'a>> {
        AstNode::new(&self.inner.key)
    }

    #[inline]
    pub fn computed(self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn optional(self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn kind(self) -> TSMethodSignatureKind {
        self.inner.kind
    }

    #[inline]
    pub fn type_parameters(self) -> Option<AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        self.inner.type_parameters.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn this_param(self) -> Option<AstNode<'a, TSThisParameter<'a>>> {
        self.inner.this_param.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn params(self) -> AstNode<'a, FormalParameters<'a>> {
        AstNode::new(self.inner.params.as_ref())
    }

    #[inline]
    pub fn return_type(self) -> Option<AstNode<'a, TSTypeAnnotation<'a>>> {
        self.inner.return_type.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.key;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_parameters
                    .as_deref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.this_param.as_deref().map(|n| n.span().start))
                    .map_or(Following::Start(self.inner.params.span().start), Following::Start),
            );
        }
        if self.inner.type_parameters.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .this_param
                    .as_deref()
                    .map_or(Following::Start(self.inner.params.span().start), |n| {
                        Following::Start(n.span().start)
                    }),
            );
        }
        if self.inner.this_param.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(self.inner.params.span().start));
        }
        let node = self.inner.params.as_ref();
        if node_address(node) == child {
            return Some(
                self.inner
                    .return_type
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.return_type.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSMethodSignature<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSMethodSignature(*self)
    }
}

impl<'a> AstNode<'a, TSConstructSignatureDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_parameters(self) -> Option<AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        self.inner.type_parameters.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn params(self) -> AstNode<'a, FormalParameters<'a>> {
        AstNode::new(self.inner.params.as_ref())
    }

    #[inline]
    pub fn return_type(self) -> Option<AstNode<'a, TSTypeAnnotation<'a>>> {
        self.inner.return_type.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        if self.inner.type_parameters.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(self.inner.params.span().start));
        }
        let node = self.inner.params.as_ref();
        if node_address(node) == child {
            return Some(
                self.inner
                    .return_type
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.return_type.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSConstructSignatureDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSConstructSignatureDeclaration(*self)
    }
}

impl<'a> AstNode<'a, TSIndexSignatureName<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(self) -> Ident<'a> {
        self.inner.name
    }

    #[inline]
    pub fn type_annotation(self) -> AstNode<'a, TSTypeAnnotation<'a>> {
        AstNode::new(self.inner.type_annotation.as_ref())
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = self.inner.type_annotation.as_ref();
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSIndexSignatureName<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSIndexSignatureName(*self)
    }
}

impl<'a> AstNode<'a, TSInterfaceHeritage<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_name(self) -> AstNode<'a, TSTypeName<'a>> {
        AstNode::new(&self.inner.type_name)
    }

    #[inline]
    pub fn type_arguments(self) -> Option<AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        self.inner.type_arguments.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.type_name;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_arguments
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.type_arguments.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSInterfaceHeritage<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSInterfaceHeritage(*self)
    }
}

impl<'a> AstNode<'a, TSTypePredicate<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn parameter_name(self) -> AstNode<'a, TSTypePredicateName<'a>> {
        AstNode::new(&self.inner.parameter_name)
    }

    #[inline]
    pub fn asserts(self) -> bool {
        self.inner.asserts
    }

    #[inline]
    pub fn type_annotation(self) -> Option<AstNode<'a, TSTypeAnnotation<'a>>> {
        self.inner.type_annotation.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.parameter_name;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_annotation
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.type_annotation.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTypePredicate<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSTypePredicate(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTypePredicateName<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            TSTypePredicateName::Identifier(s) => {
                AstNodes::IdentifierName(AstNode::new(s.as_ref()))
            }
            TSTypePredicateName::This(s) => AstNodes::TSThisType(AstNode::new(s.as_ref())),
        }
    }
}

impl<'a> AstNode<'a, TSExternalModuleDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id(self) -> AstNode<'a, StringLiteral<'a>> {
        AstNode::new(&self.inner.id)
    }

    #[inline]
    pub fn body(self) -> Option<AstNode<'a, TSModuleBlock<'a>>> {
        self.inner.body.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn declare(self) -> bool {
        self.inner.declare
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.id;
        if node_address(node) == child {
            return Some(
                self.inner
                    .body
                    .as_deref()
                    .map_or(Following::Start(0), |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.body.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSExternalModuleDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSExternalModuleDeclaration(*self)
    }
}

impl<'a> AstNode<'a, TSNamespaceDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id(self) -> AstNode<'a, BindingIdentifier<'a>> {
        AstNode::new(&self.inner.id)
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, TSNamespaceDeclarationBody<'a>> {
        AstNode::new(&self.inner.body)
    }

    #[inline]
    pub fn kind(self) -> TSNamespaceDeclarationKind {
        self.inner.kind
    }

    #[inline]
    pub fn declare(self) -> bool {
        self.inner.declare
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.id;
        if node_address(node) == child {
            return Some(Following::Start(self.inner.body.span().start));
        }
        let node = &self.inner.body;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSNamespaceDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSNamespaceDeclaration(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSNamespaceDeclarationBody<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            TSNamespaceDeclarationBody::TSNamespaceDeclaration(s) => {
                AstNodes::TSNamespaceDeclaration(AstNode::new(s.as_ref()))
            }
            TSNamespaceDeclarationBody::TSModuleBlock(s) => {
                AstNodes::TSModuleBlock(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, TSGlobalDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn global_span(self) -> Span {
        self.inner.global_span
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, TSModuleBlock<'a>> {
        AstNode::new(&self.inner.body)
    }

    #[inline]
    pub fn declare(self) -> bool {
        self.inner.declare
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.body;
        if node_address(node) == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSGlobalDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSGlobalDeclaration(*self)
    }
}

impl<'a> AstNode<'a, TSModuleBlock<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn directives(self) -> AstNode<'a, ArenaVec<'a, Directive<'a>>> {
        AstNode::new(&self.inner.directives)
    }

    #[inline]
    pub fn body(self) -> AstNode<'a, ArenaVec<'a, Statement<'a>>> {
        AstNode::new(&self.inner.body)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.directives, child, child_start, || {
            self.inner.body.first().map_or(Following::Inherit, |n| Following::Start(n.span().start))
        }) {
            return Some(following);
        }
        if let Some(following) =
            list_following(&self.inner.body, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSModuleBlock<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSModuleBlock(*self)
    }
}

impl<'a> AstNode<'a, TSTypeLiteral<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn members(self) -> AstNode<'a, ArenaVec<'a, TSSignature<'a>>> {
        AstNode::new(&self.inner.members)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) =
            list_following(&self.inner.members, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTypeLiteral<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSTypeLiteral(*self)
    }
}

impl<'a> AstNode<'a, TSInferType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_parameter(self) -> AstNode<'a, TSTypeParameter<'a>> {
        AstNode::new(self.inner.type_parameter.as_ref())
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = self.inner.type_parameter.as_ref();
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSInferType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSInferType(*self)
    }
}

impl<'a> AstNode<'a, TSTypeQuery<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expr_name(self) -> AstNode<'a, TSTypeQueryExprName<'a>> {
        AstNode::new(&self.inner.expr_name)
    }

    #[inline]
    pub fn type_arguments(self) -> Option<AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        self.inner.type_arguments.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expr_name;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .type_arguments
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.type_arguments.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTypeQuery<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSTypeQuery(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTypeQueryExprName<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            TSTypeQueryExprName::TSImportType(s) => {
                AstNodes::TSImportType(AstNode::new(s.as_ref()))
            }
            it @ match_ts_type_name!(TSTypeQueryExprName) => {
                AstNode::new(it.to_ts_type_name()).as_ast_nodes()
            }
        }
    }
}

impl<'a> AstNode<'a, TSImportType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn source(self) -> AstNode<'a, StringLiteral<'a>> {
        AstNode::new(&self.inner.source)
    }

    #[inline]
    pub fn options(self) -> Option<AstNode<'a, ObjectExpression<'a>>> {
        self.inner.options.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn qualifier(self) -> Option<AstNode<'a, TSImportTypeQualifier<'a>>> {
        self.inner.qualifier.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn type_arguments(self) -> Option<AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        self.inner.type_arguments.as_deref().map(AstNode::new)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.source;
        if node_address(node) == child {
            return Some(
                self.inner
                    .options
                    .as_deref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.qualifier.as_ref().map(|n| n.span().start))
                    .or_else(|| self.inner.type_arguments.as_deref().map(|n| n.span().start))
                    .map_or(Following::Inherit, Following::Start),
            );
        }
        if self.inner.options.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .qualifier
                    .as_ref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.type_arguments.as_deref().map(|n| n.span().start))
                    .map_or(Following::Inherit, Following::Start),
            );
        }
        if self
            .inner
            .qualifier
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(
                self.inner
                    .type_arguments
                    .as_deref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self.inner.type_arguments.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSImportType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSImportType(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSImportTypeQualifier<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            TSImportTypeQualifier::Identifier(s) => {
                AstNodes::IdentifierName(AstNode::new(s.as_ref()))
            }
            TSImportTypeQualifier::QualifiedName(s) => {
                AstNodes::TSImportTypeQualifiedName(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, TSImportTypeQualifiedName<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left(self) -> AstNode<'a, TSImportTypeQualifier<'a>> {
        AstNode::new(&self.inner.left)
    }

    #[inline]
    pub fn right(self) -> AstNode<'a, IdentifierName<'a>> {
        AstNode::new(&self.inner.right)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.left;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.right.span().start));
        }
        let node = &self.inner.right;
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSImportTypeQualifiedName<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSImportTypeQualifiedName(*self)
    }
}

impl<'a> AstNode<'a, TSFunctionType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_parameters(self) -> Option<AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        self.inner.type_parameters.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn this_param(self) -> Option<AstNode<'a, TSThisParameter<'a>>> {
        self.inner.this_param.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn params(self) -> AstNode<'a, FormalParameters<'a>> {
        AstNode::new(self.inner.params.as_ref())
    }

    #[inline]
    pub fn return_type(self) -> AstNode<'a, TSTypeAnnotation<'a>> {
        AstNode::new(self.inner.return_type.as_ref())
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        if self.inner.type_parameters.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(
                self.inner
                    .this_param
                    .as_deref()
                    .map_or(Following::Start(self.inner.params.span().start), |n| {
                        Following::Start(n.span().start)
                    }),
            );
        }
        if self.inner.this_param.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(self.inner.params.span().start));
        }
        let node = self.inner.params.as_ref();
        if node_address(node) == child {
            return Some(Following::Start(self.inner.return_type.span().start));
        }
        let node = self.inner.return_type.as_ref();
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSFunctionType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSFunctionType(*self)
    }
}

impl<'a> AstNode<'a, TSConstructorType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#abstract(self) -> bool {
        self.inner.r#abstract
    }

    #[inline]
    pub fn type_parameters(self) -> Option<AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        self.inner.type_parameters.as_deref().map(AstNode::new)
    }

    #[inline]
    pub fn params(self) -> AstNode<'a, FormalParameters<'a>> {
        AstNode::new(self.inner.params.as_ref())
    }

    #[inline]
    pub fn return_type(self) -> AstNode<'a, TSTypeAnnotation<'a>> {
        AstNode::new(self.inner.return_type.as_ref())
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        if self.inner.type_parameters.as_deref().is_some_and(|node| node_address(node) == child) {
            return Some(Following::Start(self.inner.params.span().start));
        }
        let node = self.inner.params.as_ref();
        if node_address(node) == child {
            return Some(Following::Start(self.inner.return_type.span().start));
        }
        let node = self.inner.return_type.as_ref();
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSConstructorType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSConstructorType(*self)
    }
}

impl<'a> AstNode<'a, TSMappedType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn key(self) -> AstNode<'a, BindingIdentifier<'a>> {
        AstNode::new(&self.inner.key)
    }

    #[inline]
    pub fn constraint(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.constraint)
    }

    #[inline]
    pub fn name_type(self) -> Option<AstNode<'a, TSType<'a>>> {
        self.inner.name_type.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn type_annotation(self) -> Option<AstNode<'a, TSType<'a>>> {
        self.inner.type_annotation.as_ref().map(AstNode::new)
    }

    #[inline]
    pub fn optional(self) -> Option<TSMappedTypeModifierOperator> {
        self.inner.optional
    }

    #[inline]
    pub fn readonly(self) -> Option<TSMappedTypeModifierOperator> {
        self.inner.readonly
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.key;
        if node_address(node) == child {
            return Some(Following::Start(self.inner.constraint.span().start));
        }
        let node = &self.inner.constraint;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(
                self.inner
                    .name_type
                    .as_ref()
                    .map(|n| n.span().start)
                    .or_else(|| self.inner.type_annotation.as_ref().map(|n| n.span().start))
                    .map_or(Following::Inherit, Following::Start),
            );
        }
        if self
            .inner
            .name_type
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(
                self.inner
                    .type_annotation
                    .as_ref()
                    .map_or(Following::Inherit, |n| Following::Start(n.span().start)),
            );
        }
        if self
            .inner
            .type_annotation
            .as_ref()
            .is_some_and(|node| AstNode::new(node).as_ast_nodes().address() == child)
        {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSMappedType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSMappedType(*self)
    }
}

impl<'a> AstNode<'a, TSTemplateLiteralType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn quasis(self) -> AstNode<'a, ArenaVec<'a, TemplateElement<'a>>> {
        AstNode::new(&self.inner.quasis)
    }

    #[inline]
    pub fn types(self) -> AstNode<'a, ArenaVec<'a, TSType<'a>>> {
        AstNode::new(&self.inner.types)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, child_start: u32) -> Option<Following> {
        if let Some(following) = list_following(&self.inner.quasis, child, child_start, || {
            self.inner
                .types
                .first()
                .map_or(Following::Inherit, |n| Following::Start(n.span().start))
        }) {
            return Some(following);
        }
        if let Some(following) =
            list_following(&self.inner.types, child, child_start, || Following::Inherit)
        {
            return Some(following);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTemplateLiteralType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSTemplateLiteralType(*self)
    }
}

impl<'a> AstNode<'a, TSAsExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.expression)
    }

    #[inline]
    pub fn type_annotation(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.type_annotation)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.type_annotation.span().start));
        }
        let node = &self.inner.type_annotation;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSAsExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSAsExpression(*self)
    }
}

impl<'a> AstNode<'a, TSSatisfiesExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.expression)
    }

    #[inline]
    pub fn type_annotation(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.type_annotation)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.type_annotation.span().start));
        }
        let node = &self.inner.type_annotation;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSSatisfiesExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSSatisfiesExpression(*self)
    }
}

impl<'a> AstNode<'a, TSTypeAssertion<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.type_annotation)
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.expression)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.type_annotation;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.expression.span().start));
        }
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSTypeAssertion<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSTypeAssertion(*self)
    }
}

impl<'a> AstNode<'a, TSImportEqualsDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id(self) -> AstNode<'a, BindingIdentifier<'a>> {
        AstNode::new(&self.inner.id)
    }

    #[inline]
    pub fn module_reference(self) -> AstNode<'a, TSModuleReference<'a>> {
        AstNode::new(&self.inner.module_reference)
    }

    #[inline]
    pub fn import_kind(self) -> ImportOrExportKind {
        self.inner.import_kind
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.id;
        if node_address(node) == child {
            return Some(Following::Start(self.inner.module_reference.span().start));
        }
        let node = &self.inner.module_reference;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSImportEqualsDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSImportEqualsDeclaration(*self)
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSModuleReference<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        match self.inner {
            TSModuleReference::ExternalModuleReference(s) => {
                AstNodes::TSExternalModuleReference(AstNode::new(s.as_ref()))
            }
            TSModuleReference::IdentifierReference(s) => {
                AstNodes::IdentifierReference(AstNode::new(s.as_ref()))
            }
            TSModuleReference::QualifiedName(s) => {
                AstNodes::TSQualifiedName(AstNode::new(s.as_ref()))
            }
        }
    }
}

impl<'a> AstNode<'a, TSExternalModuleReference<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, StringLiteral<'a>> {
        AstNode::new(&self.inner.expression)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSExternalModuleReference<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSExternalModuleReference(*self)
    }
}

impl<'a> AstNode<'a, TSNonNullExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.expression)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSNonNullExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSNonNullExpression(*self)
    }
}

impl<'a> AstNode<'a, Decorator<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.expression)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, Decorator<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::Decorator(*self)
    }
}

impl<'a> AstNode<'a, TSExportAssignment<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.expression)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSExportAssignment<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSExportAssignment(*self)
    }
}

impl<'a> AstNode<'a, TSNamespaceExportDeclaration<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id(self) -> AstNode<'a, IdentifierName<'a>> {
        AstNode::new(&self.inner.id)
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.id;
        if node_address(node) == child {
            return Some(Following::Start(0));
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSNamespaceExportDeclaration<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSNamespaceExportDeclaration(*self)
    }
}

impl<'a> AstNode<'a, TSInstantiationExpression<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(self) -> AstNode<'a, Expression<'a>> {
        AstNode::new(&self.inner.expression)
    }

    #[inline]
    pub fn type_arguments(self) -> AstNode<'a, TSTypeParameterInstantiation<'a>> {
        AstNode::new(self.inner.type_arguments.as_ref())
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.expression;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Start(self.inner.type_arguments.span().start));
        }
        let node = self.inner.type_arguments.as_ref();
        if node_address(node) == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, TSInstantiationExpression<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::TSInstantiationExpression(*self)
    }
}

impl<'a> AstNode<'a, JSDocNullableType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.type_annotation)
    }

    #[inline]
    pub fn postfix(self) -> bool {
        self.inner.postfix
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.type_annotation;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSDocNullableType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSDocNullableType(*self)
    }
}

impl<'a> AstNode<'a, JSDocNonNullableType<'a>> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation(self) -> AstNode<'a, TSType<'a>> {
        AstNode::new(&self.inner.type_annotation)
    }

    #[inline]
    pub fn postfix(self) -> bool {
        self.inner.postfix
    }

    /// The [`Following`] of the child at the address `child` (starting at `child_start`), `None` if it is not a child of this node.
    pub fn following_of_child(self, child: usize, _child_start: u32) -> Option<Following> {
        let node = &self.inner.type_annotation;
        if AstNode::new(node).as_ast_nodes().address() == child {
            return Some(Following::Inherit);
        }
        None
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSDocNonNullableType<'a>> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSDocNonNullableType(*self)
    }
}

impl<'a> AstNode<'a, JSDocUnknownType> {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(self, f: &mut JsFormatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(self, f: &mut JsFormatter<'_, 'a>) {
        if f.context().comments().unprinted_comments().is_empty() {
            return;
        }
        format_trailing_comments(
            self.parent(f).span(),
            self.inner.span(),
            self.following_span_start(f),
        )
        .fmt(f);
    }
}

impl<'a> AsAstNodes<'a> for AstNode<'a, JSDocUnknownType> {
    #[inline]
    fn as_ast_nodes(&self) -> AstNodes<'a> {
        AstNodes::JSDocUnknownType(*self)
    }
}
