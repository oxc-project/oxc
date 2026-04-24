// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter/ast_nodes.rs`.

#![expect(clippy::match_same_arms)]
use oxc_allocator::{Allocator, Vec};
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_str::Ident;
use oxc_syntax::node::NodeId;

use crate::ast_nodes::AstNode;
use crate::formatter::{
    Format, Formatter,
    trivia::{format_leading_comments, format_trailing_comments},
};

#[derive(Clone, Copy)]
pub enum AstNodes<'a, 'b> {
    Dummy(),
    Program(&'b AstNode<'a, 'b, Program<'a>>),
    IdentifierName(&'b AstNode<'a, 'b, IdentifierName<'a>>),
    IdentifierReference(&'b AstNode<'a, 'b, IdentifierReference<'a>>),
    BindingIdentifier(&'b AstNode<'a, 'b, BindingIdentifier<'a>>),
    LabelIdentifier(&'b AstNode<'a, 'b, LabelIdentifier<'a>>),
    ThisExpression(&'b AstNode<'a, 'b, ThisExpression>),
    ArrayExpression(&'b AstNode<'a, 'b, ArrayExpression<'a>>),
    Elision(&'b AstNode<'a, 'b, Elision>),
    ObjectExpression(&'b AstNode<'a, 'b, ObjectExpression<'a>>),
    ObjectProperty(&'b AstNode<'a, 'b, ObjectProperty<'a>>),
    TemplateLiteral(&'b AstNode<'a, 'b, TemplateLiteral<'a>>),
    TaggedTemplateExpression(&'b AstNode<'a, 'b, TaggedTemplateExpression<'a>>),
    TemplateElement(&'b AstNode<'a, 'b, TemplateElement<'a>>),
    ComputedMemberExpression(&'b AstNode<'a, 'b, ComputedMemberExpression<'a>>),
    StaticMemberExpression(&'b AstNode<'a, 'b, StaticMemberExpression<'a>>),
    PrivateFieldExpression(&'b AstNode<'a, 'b, PrivateFieldExpression<'a>>),
    CallExpression(&'b AstNode<'a, 'b, CallExpression<'a>>),
    NewExpression(&'b AstNode<'a, 'b, NewExpression<'a>>),
    MetaProperty(&'b AstNode<'a, 'b, MetaProperty<'a>>),
    SpreadElement(&'b AstNode<'a, 'b, SpreadElement<'a>>),
    UpdateExpression(&'b AstNode<'a, 'b, UpdateExpression<'a>>),
    UnaryExpression(&'b AstNode<'a, 'b, UnaryExpression<'a>>),
    BinaryExpression(&'b AstNode<'a, 'b, BinaryExpression<'a>>),
    PrivateInExpression(&'b AstNode<'a, 'b, PrivateInExpression<'a>>),
    LogicalExpression(&'b AstNode<'a, 'b, LogicalExpression<'a>>),
    ConditionalExpression(&'b AstNode<'a, 'b, ConditionalExpression<'a>>),
    AssignmentExpression(&'b AstNode<'a, 'b, AssignmentExpression<'a>>),
    ArrayAssignmentTarget(&'b AstNode<'a, 'b, ArrayAssignmentTarget<'a>>),
    ObjectAssignmentTarget(&'b AstNode<'a, 'b, ObjectAssignmentTarget<'a>>),
    AssignmentTargetRest(&'b AstNode<'a, 'b, AssignmentTargetRest<'a>>),
    AssignmentTargetWithDefault(&'b AstNode<'a, 'b, AssignmentTargetWithDefault<'a>>),
    AssignmentTargetPropertyIdentifier(&'b AstNode<'a, 'b, AssignmentTargetPropertyIdentifier<'a>>),
    AssignmentTargetPropertyProperty(&'b AstNode<'a, 'b, AssignmentTargetPropertyProperty<'a>>),
    SequenceExpression(&'b AstNode<'a, 'b, SequenceExpression<'a>>),
    Super(&'b AstNode<'a, 'b, Super>),
    AwaitExpression(&'b AstNode<'a, 'b, AwaitExpression<'a>>),
    ChainExpression(&'b AstNode<'a, 'b, ChainExpression<'a>>),
    ParenthesizedExpression(&'b AstNode<'a, 'b, ParenthesizedExpression<'a>>),
    Directive(&'b AstNode<'a, 'b, Directive<'a>>),
    Hashbang(&'b AstNode<'a, 'b, Hashbang<'a>>),
    BlockStatement(&'b AstNode<'a, 'b, BlockStatement<'a>>),
    VariableDeclaration(&'b AstNode<'a, 'b, VariableDeclaration<'a>>),
    VariableDeclarator(&'b AstNode<'a, 'b, VariableDeclarator<'a>>),
    EmptyStatement(&'b AstNode<'a, 'b, EmptyStatement>),
    ExpressionStatement(&'b AstNode<'a, 'b, ExpressionStatement<'a>>),
    IfStatement(&'b AstNode<'a, 'b, IfStatement<'a>>),
    DoWhileStatement(&'b AstNode<'a, 'b, DoWhileStatement<'a>>),
    WhileStatement(&'b AstNode<'a, 'b, WhileStatement<'a>>),
    ForStatement(&'b AstNode<'a, 'b, ForStatement<'a>>),
    ForInStatement(&'b AstNode<'a, 'b, ForInStatement<'a>>),
    ForOfStatement(&'b AstNode<'a, 'b, ForOfStatement<'a>>),
    ContinueStatement(&'b AstNode<'a, 'b, ContinueStatement<'a>>),
    BreakStatement(&'b AstNode<'a, 'b, BreakStatement<'a>>),
    ReturnStatement(&'b AstNode<'a, 'b, ReturnStatement<'a>>),
    WithStatement(&'b AstNode<'a, 'b, WithStatement<'a>>),
    SwitchStatement(&'b AstNode<'a, 'b, SwitchStatement<'a>>),
    SwitchCase(&'b AstNode<'a, 'b, SwitchCase<'a>>),
    LabeledStatement(&'b AstNode<'a, 'b, LabeledStatement<'a>>),
    ThrowStatement(&'b AstNode<'a, 'b, ThrowStatement<'a>>),
    TryStatement(&'b AstNode<'a, 'b, TryStatement<'a>>),
    CatchClause(&'b AstNode<'a, 'b, CatchClause<'a>>),
    CatchParameter(&'b AstNode<'a, 'b, CatchParameter<'a>>),
    DebuggerStatement(&'b AstNode<'a, 'b, DebuggerStatement>),
    AssignmentPattern(&'b AstNode<'a, 'b, AssignmentPattern<'a>>),
    ObjectPattern(&'b AstNode<'a, 'b, ObjectPattern<'a>>),
    BindingProperty(&'b AstNode<'a, 'b, BindingProperty<'a>>),
    ArrayPattern(&'b AstNode<'a, 'b, ArrayPattern<'a>>),
    BindingRestElement(&'b AstNode<'a, 'b, BindingRestElement<'a>>),
    Function(&'b AstNode<'a, 'b, Function<'a>>),
    FormalParameters(&'b AstNode<'a, 'b, FormalParameters<'a>>),
    FormalParameter(&'b AstNode<'a, 'b, FormalParameter<'a>>),
    FormalParameterRest(&'b AstNode<'a, 'b, FormalParameterRest<'a>>),
    FunctionBody(&'b AstNode<'a, 'b, FunctionBody<'a>>),
    ArrowFunctionExpression(&'b AstNode<'a, 'b, ArrowFunctionExpression<'a>>),
    YieldExpression(&'b AstNode<'a, 'b, YieldExpression<'a>>),
    Class(&'b AstNode<'a, 'b, Class<'a>>),
    ClassBody(&'b AstNode<'a, 'b, ClassBody<'a>>),
    MethodDefinition(&'b AstNode<'a, 'b, MethodDefinition<'a>>),
    PropertyDefinition(&'b AstNode<'a, 'b, PropertyDefinition<'a>>),
    PrivateIdentifier(&'b AstNode<'a, 'b, PrivateIdentifier<'a>>),
    StaticBlock(&'b AstNode<'a, 'b, StaticBlock<'a>>),
    AccessorProperty(&'b AstNode<'a, 'b, AccessorProperty<'a>>),
    ImportExpression(&'b AstNode<'a, 'b, ImportExpression<'a>>),
    ImportDeclaration(&'b AstNode<'a, 'b, ImportDeclaration<'a>>),
    ImportSpecifier(&'b AstNode<'a, 'b, ImportSpecifier<'a>>),
    ImportDefaultSpecifier(&'b AstNode<'a, 'b, ImportDefaultSpecifier<'a>>),
    ImportNamespaceSpecifier(&'b AstNode<'a, 'b, ImportNamespaceSpecifier<'a>>),
    WithClause(&'b AstNode<'a, 'b, WithClause<'a>>),
    ImportAttribute(&'b AstNode<'a, 'b, ImportAttribute<'a>>),
    ExportNamedDeclaration(&'b AstNode<'a, 'b, ExportNamedDeclaration<'a>>),
    ExportDefaultDeclaration(&'b AstNode<'a, 'b, ExportDefaultDeclaration<'a>>),
    ExportAllDeclaration(&'b AstNode<'a, 'b, ExportAllDeclaration<'a>>),
    ExportSpecifier(&'b AstNode<'a, 'b, ExportSpecifier<'a>>),
    V8IntrinsicExpression(&'b AstNode<'a, 'b, V8IntrinsicExpression<'a>>),
    BooleanLiteral(&'b AstNode<'a, 'b, BooleanLiteral>),
    NullLiteral(&'b AstNode<'a, 'b, NullLiteral>),
    NumericLiteral(&'b AstNode<'a, 'b, NumericLiteral<'a>>),
    StringLiteral(&'b AstNode<'a, 'b, StringLiteral<'a>>),
    BigIntLiteral(&'b AstNode<'a, 'b, BigIntLiteral<'a>>),
    RegExpLiteral(&'b AstNode<'a, 'b, RegExpLiteral<'a>>),
    JSXElement(&'b AstNode<'a, 'b, JSXElement<'a>>),
    JSXOpeningElement(&'b AstNode<'a, 'b, JSXOpeningElement<'a>>),
    JSXClosingElement(&'b AstNode<'a, 'b, JSXClosingElement<'a>>),
    JSXFragment(&'b AstNode<'a, 'b, JSXFragment<'a>>),
    JSXOpeningFragment(&'b AstNode<'a, 'b, JSXOpeningFragment>),
    JSXClosingFragment(&'b AstNode<'a, 'b, JSXClosingFragment>),
    JSXNamespacedName(&'b AstNode<'a, 'b, JSXNamespacedName<'a>>),
    JSXMemberExpression(&'b AstNode<'a, 'b, JSXMemberExpression<'a>>),
    JSXExpressionContainer(&'b AstNode<'a, 'b, JSXExpressionContainer<'a>>),
    JSXEmptyExpression(&'b AstNode<'a, 'b, JSXEmptyExpression>),
    JSXAttribute(&'b AstNode<'a, 'b, JSXAttribute<'a>>),
    JSXSpreadAttribute(&'b AstNode<'a, 'b, JSXSpreadAttribute<'a>>),
    JSXIdentifier(&'b AstNode<'a, 'b, JSXIdentifier<'a>>),
    JSXSpreadChild(&'b AstNode<'a, 'b, JSXSpreadChild<'a>>),
    JSXText(&'b AstNode<'a, 'b, JSXText<'a>>),
    TSThisParameter(&'b AstNode<'a, 'b, TSThisParameter<'a>>),
    TSEnumDeclaration(&'b AstNode<'a, 'b, TSEnumDeclaration<'a>>),
    TSEnumBody(&'b AstNode<'a, 'b, TSEnumBody<'a>>),
    TSEnumMember(&'b AstNode<'a, 'b, TSEnumMember<'a>>),
    TSTypeAnnotation(&'b AstNode<'a, 'b, TSTypeAnnotation<'a>>),
    TSLiteralType(&'b AstNode<'a, 'b, TSLiteralType<'a>>),
    TSConditionalType(&'b AstNode<'a, 'b, TSConditionalType<'a>>),
    TSUnionType(&'b AstNode<'a, 'b, TSUnionType<'a>>),
    TSIntersectionType(&'b AstNode<'a, 'b, TSIntersectionType<'a>>),
    TSParenthesizedType(&'b AstNode<'a, 'b, TSParenthesizedType<'a>>),
    TSTypeOperator(&'b AstNode<'a, 'b, TSTypeOperator<'a>>),
    TSArrayType(&'b AstNode<'a, 'b, TSArrayType<'a>>),
    TSIndexedAccessType(&'b AstNode<'a, 'b, TSIndexedAccessType<'a>>),
    TSTupleType(&'b AstNode<'a, 'b, TSTupleType<'a>>),
    TSNamedTupleMember(&'b AstNode<'a, 'b, TSNamedTupleMember<'a>>),
    TSOptionalType(&'b AstNode<'a, 'b, TSOptionalType<'a>>),
    TSRestType(&'b AstNode<'a, 'b, TSRestType<'a>>),
    TSAnyKeyword(&'b AstNode<'a, 'b, TSAnyKeyword>),
    TSStringKeyword(&'b AstNode<'a, 'b, TSStringKeyword>),
    TSBooleanKeyword(&'b AstNode<'a, 'b, TSBooleanKeyword>),
    TSNumberKeyword(&'b AstNode<'a, 'b, TSNumberKeyword>),
    TSNeverKeyword(&'b AstNode<'a, 'b, TSNeverKeyword>),
    TSIntrinsicKeyword(&'b AstNode<'a, 'b, TSIntrinsicKeyword>),
    TSUnknownKeyword(&'b AstNode<'a, 'b, TSUnknownKeyword>),
    TSNullKeyword(&'b AstNode<'a, 'b, TSNullKeyword>),
    TSUndefinedKeyword(&'b AstNode<'a, 'b, TSUndefinedKeyword>),
    TSVoidKeyword(&'b AstNode<'a, 'b, TSVoidKeyword>),
    TSSymbolKeyword(&'b AstNode<'a, 'b, TSSymbolKeyword>),
    TSThisType(&'b AstNode<'a, 'b, TSThisType>),
    TSObjectKeyword(&'b AstNode<'a, 'b, TSObjectKeyword>),
    TSBigIntKeyword(&'b AstNode<'a, 'b, TSBigIntKeyword>),
    TSTypeReference(&'b AstNode<'a, 'b, TSTypeReference<'a>>),
    TSQualifiedName(&'b AstNode<'a, 'b, TSQualifiedName<'a>>),
    TSTypeParameterInstantiation(&'b AstNode<'a, 'b, TSTypeParameterInstantiation<'a>>),
    TSTypeParameter(&'b AstNode<'a, 'b, TSTypeParameter<'a>>),
    TSTypeParameterDeclaration(&'b AstNode<'a, 'b, TSTypeParameterDeclaration<'a>>),
    TSTypeAliasDeclaration(&'b AstNode<'a, 'b, TSTypeAliasDeclaration<'a>>),
    TSClassImplements(&'b AstNode<'a, 'b, TSClassImplements<'a>>),
    TSInterfaceDeclaration(&'b AstNode<'a, 'b, TSInterfaceDeclaration<'a>>),
    TSInterfaceBody(&'b AstNode<'a, 'b, TSInterfaceBody<'a>>),
    TSPropertySignature(&'b AstNode<'a, 'b, TSPropertySignature<'a>>),
    TSIndexSignature(&'b AstNode<'a, 'b, TSIndexSignature<'a>>),
    TSCallSignatureDeclaration(&'b AstNode<'a, 'b, TSCallSignatureDeclaration<'a>>),
    TSMethodSignature(&'b AstNode<'a, 'b, TSMethodSignature<'a>>),
    TSConstructSignatureDeclaration(&'b AstNode<'a, 'b, TSConstructSignatureDeclaration<'a>>),
    TSIndexSignatureName(&'b AstNode<'a, 'b, TSIndexSignatureName<'a>>),
    TSInterfaceHeritage(&'b AstNode<'a, 'b, TSInterfaceHeritage<'a>>),
    TSTypePredicate(&'b AstNode<'a, 'b, TSTypePredicate<'a>>),
    TSModuleDeclaration(&'b AstNode<'a, 'b, TSModuleDeclaration<'a>>),
    TSGlobalDeclaration(&'b AstNode<'a, 'b, TSGlobalDeclaration<'a>>),
    TSModuleBlock(&'b AstNode<'a, 'b, TSModuleBlock<'a>>),
    TSTypeLiteral(&'b AstNode<'a, 'b, TSTypeLiteral<'a>>),
    TSInferType(&'b AstNode<'a, 'b, TSInferType<'a>>),
    TSTypeQuery(&'b AstNode<'a, 'b, TSTypeQuery<'a>>),
    TSImportType(&'b AstNode<'a, 'b, TSImportType<'a>>),
    TSImportTypeQualifiedName(&'b AstNode<'a, 'b, TSImportTypeQualifiedName<'a>>),
    TSFunctionType(&'b AstNode<'a, 'b, TSFunctionType<'a>>),
    TSConstructorType(&'b AstNode<'a, 'b, TSConstructorType<'a>>),
    TSMappedType(&'b AstNode<'a, 'b, TSMappedType<'a>>),
    TSTemplateLiteralType(&'b AstNode<'a, 'b, TSTemplateLiteralType<'a>>),
    TSAsExpression(&'b AstNode<'a, 'b, TSAsExpression<'a>>),
    TSSatisfiesExpression(&'b AstNode<'a, 'b, TSSatisfiesExpression<'a>>),
    TSTypeAssertion(&'b AstNode<'a, 'b, TSTypeAssertion<'a>>),
    TSImportEqualsDeclaration(&'b AstNode<'a, 'b, TSImportEqualsDeclaration<'a>>),
    TSExternalModuleReference(&'b AstNode<'a, 'b, TSExternalModuleReference<'a>>),
    TSNonNullExpression(&'b AstNode<'a, 'b, TSNonNullExpression<'a>>),
    Decorator(&'b AstNode<'a, 'b, Decorator<'a>>),
    TSExportAssignment(&'b AstNode<'a, 'b, TSExportAssignment<'a>>),
    TSNamespaceExportDeclaration(&'b AstNode<'a, 'b, TSNamespaceExportDeclaration<'a>>),
    TSInstantiationExpression(&'b AstNode<'a, 'b, TSInstantiationExpression<'a>>),
    JSDocNullableType(&'b AstNode<'a, 'b, JSDocNullableType<'a>>),
    JSDocNonNullableType(&'b AstNode<'a, 'b, JSDocNonNullableType<'a>>),
    JSDocUnknownType(&'b AstNode<'a, 'b, JSDocUnknownType>),
}
impl AstNodes<'_, '_> {
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
            Self::MetaProperty(n) => n.span(),
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
            Self::ExportNamedDeclaration(n) => n.span(),
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
            Self::TSModuleDeclaration(n) => n.span(),
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
    #[inline]
    pub fn parent(&self) -> &Self {
        match self {
            Self::Dummy() => panic!("Should never be called on a dummy node"),
            Self::Program(n) => n.parent(),
            Self::IdentifierName(n) => n.parent(),
            Self::IdentifierReference(n) => n.parent(),
            Self::BindingIdentifier(n) => n.parent(),
            Self::LabelIdentifier(n) => n.parent(),
            Self::ThisExpression(n) => n.parent(),
            Self::ArrayExpression(n) => n.parent(),
            Self::Elision(n) => n.parent(),
            Self::ObjectExpression(n) => n.parent(),
            Self::ObjectProperty(n) => n.parent(),
            Self::TemplateLiteral(n) => n.parent(),
            Self::TaggedTemplateExpression(n) => n.parent(),
            Self::TemplateElement(n) => n.parent(),
            Self::ComputedMemberExpression(n) => n.parent(),
            Self::StaticMemberExpression(n) => n.parent(),
            Self::PrivateFieldExpression(n) => n.parent(),
            Self::CallExpression(n) => n.parent(),
            Self::NewExpression(n) => n.parent(),
            Self::MetaProperty(n) => n.parent(),
            Self::SpreadElement(n) => n.parent(),
            Self::UpdateExpression(n) => n.parent(),
            Self::UnaryExpression(n) => n.parent(),
            Self::BinaryExpression(n) => n.parent(),
            Self::PrivateInExpression(n) => n.parent(),
            Self::LogicalExpression(n) => n.parent(),
            Self::ConditionalExpression(n) => n.parent(),
            Self::AssignmentExpression(n) => n.parent(),
            Self::ArrayAssignmentTarget(n) => n.parent(),
            Self::ObjectAssignmentTarget(n) => n.parent(),
            Self::AssignmentTargetRest(n) => n.parent(),
            Self::AssignmentTargetWithDefault(n) => n.parent(),
            Self::AssignmentTargetPropertyIdentifier(n) => n.parent(),
            Self::AssignmentTargetPropertyProperty(n) => n.parent(),
            Self::SequenceExpression(n) => n.parent(),
            Self::Super(n) => n.parent(),
            Self::AwaitExpression(n) => n.parent(),
            Self::ChainExpression(n) => n.parent(),
            Self::ParenthesizedExpression(n) => n.parent(),
            Self::Directive(n) => n.parent(),
            Self::Hashbang(n) => n.parent(),
            Self::BlockStatement(n) => n.parent(),
            Self::VariableDeclaration(n) => n.parent(),
            Self::VariableDeclarator(n) => n.parent(),
            Self::EmptyStatement(n) => n.parent(),
            Self::ExpressionStatement(n) => n.parent(),
            Self::IfStatement(n) => n.parent(),
            Self::DoWhileStatement(n) => n.parent(),
            Self::WhileStatement(n) => n.parent(),
            Self::ForStatement(n) => n.parent(),
            Self::ForInStatement(n) => n.parent(),
            Self::ForOfStatement(n) => n.parent(),
            Self::ContinueStatement(n) => n.parent(),
            Self::BreakStatement(n) => n.parent(),
            Self::ReturnStatement(n) => n.parent(),
            Self::WithStatement(n) => n.parent(),
            Self::SwitchStatement(n) => n.parent(),
            Self::SwitchCase(n) => n.parent(),
            Self::LabeledStatement(n) => n.parent(),
            Self::ThrowStatement(n) => n.parent(),
            Self::TryStatement(n) => n.parent(),
            Self::CatchClause(n) => n.parent(),
            Self::CatchParameter(n) => n.parent(),
            Self::DebuggerStatement(n) => n.parent(),
            Self::AssignmentPattern(n) => n.parent(),
            Self::ObjectPattern(n) => n.parent(),
            Self::BindingProperty(n) => n.parent(),
            Self::ArrayPattern(n) => n.parent(),
            Self::BindingRestElement(n) => n.parent(),
            Self::Function(n) => n.parent(),
            Self::FormalParameters(n) => n.parent(),
            Self::FormalParameter(n) => n.parent(),
            Self::FormalParameterRest(n) => n.parent(),
            Self::FunctionBody(n) => n.parent(),
            Self::ArrowFunctionExpression(n) => n.parent(),
            Self::YieldExpression(n) => n.parent(),
            Self::Class(n) => n.parent(),
            Self::ClassBody(n) => n.parent(),
            Self::MethodDefinition(n) => n.parent(),
            Self::PropertyDefinition(n) => n.parent(),
            Self::PrivateIdentifier(n) => n.parent(),
            Self::StaticBlock(n) => n.parent(),
            Self::AccessorProperty(n) => n.parent(),
            Self::ImportExpression(n) => n.parent(),
            Self::ImportDeclaration(n) => n.parent(),
            Self::ImportSpecifier(n) => n.parent(),
            Self::ImportDefaultSpecifier(n) => n.parent(),
            Self::ImportNamespaceSpecifier(n) => n.parent(),
            Self::WithClause(n) => n.parent(),
            Self::ImportAttribute(n) => n.parent(),
            Self::ExportNamedDeclaration(n) => n.parent(),
            Self::ExportDefaultDeclaration(n) => n.parent(),
            Self::ExportAllDeclaration(n) => n.parent(),
            Self::ExportSpecifier(n) => n.parent(),
            Self::V8IntrinsicExpression(n) => n.parent(),
            Self::BooleanLiteral(n) => n.parent(),
            Self::NullLiteral(n) => n.parent(),
            Self::NumericLiteral(n) => n.parent(),
            Self::StringLiteral(n) => n.parent(),
            Self::BigIntLiteral(n) => n.parent(),
            Self::RegExpLiteral(n) => n.parent(),
            Self::JSXElement(n) => n.parent(),
            Self::JSXOpeningElement(n) => n.parent(),
            Self::JSXClosingElement(n) => n.parent(),
            Self::JSXFragment(n) => n.parent(),
            Self::JSXOpeningFragment(n) => n.parent(),
            Self::JSXClosingFragment(n) => n.parent(),
            Self::JSXNamespacedName(n) => n.parent(),
            Self::JSXMemberExpression(n) => n.parent(),
            Self::JSXExpressionContainer(n) => n.parent(),
            Self::JSXEmptyExpression(n) => n.parent(),
            Self::JSXAttribute(n) => n.parent(),
            Self::JSXSpreadAttribute(n) => n.parent(),
            Self::JSXIdentifier(n) => n.parent(),
            Self::JSXSpreadChild(n) => n.parent(),
            Self::JSXText(n) => n.parent(),
            Self::TSThisParameter(n) => n.parent(),
            Self::TSEnumDeclaration(n) => n.parent(),
            Self::TSEnumBody(n) => n.parent(),
            Self::TSEnumMember(n) => n.parent(),
            Self::TSTypeAnnotation(n) => n.parent(),
            Self::TSLiteralType(n) => n.parent(),
            Self::TSConditionalType(n) => n.parent(),
            Self::TSUnionType(n) => n.parent(),
            Self::TSIntersectionType(n) => n.parent(),
            Self::TSParenthesizedType(n) => n.parent(),
            Self::TSTypeOperator(n) => n.parent(),
            Self::TSArrayType(n) => n.parent(),
            Self::TSIndexedAccessType(n) => n.parent(),
            Self::TSTupleType(n) => n.parent(),
            Self::TSNamedTupleMember(n) => n.parent(),
            Self::TSOptionalType(n) => n.parent(),
            Self::TSRestType(n) => n.parent(),
            Self::TSAnyKeyword(n) => n.parent(),
            Self::TSStringKeyword(n) => n.parent(),
            Self::TSBooleanKeyword(n) => n.parent(),
            Self::TSNumberKeyword(n) => n.parent(),
            Self::TSNeverKeyword(n) => n.parent(),
            Self::TSIntrinsicKeyword(n) => n.parent(),
            Self::TSUnknownKeyword(n) => n.parent(),
            Self::TSNullKeyword(n) => n.parent(),
            Self::TSUndefinedKeyword(n) => n.parent(),
            Self::TSVoidKeyword(n) => n.parent(),
            Self::TSSymbolKeyword(n) => n.parent(),
            Self::TSThisType(n) => n.parent(),
            Self::TSObjectKeyword(n) => n.parent(),
            Self::TSBigIntKeyword(n) => n.parent(),
            Self::TSTypeReference(n) => n.parent(),
            Self::TSQualifiedName(n) => n.parent(),
            Self::TSTypeParameterInstantiation(n) => n.parent(),
            Self::TSTypeParameter(n) => n.parent(),
            Self::TSTypeParameterDeclaration(n) => n.parent(),
            Self::TSTypeAliasDeclaration(n) => n.parent(),
            Self::TSClassImplements(n) => n.parent(),
            Self::TSInterfaceDeclaration(n) => n.parent(),
            Self::TSInterfaceBody(n) => n.parent(),
            Self::TSPropertySignature(n) => n.parent(),
            Self::TSIndexSignature(n) => n.parent(),
            Self::TSCallSignatureDeclaration(n) => n.parent(),
            Self::TSMethodSignature(n) => n.parent(),
            Self::TSConstructSignatureDeclaration(n) => n.parent(),
            Self::TSIndexSignatureName(n) => n.parent(),
            Self::TSInterfaceHeritage(n) => n.parent(),
            Self::TSTypePredicate(n) => n.parent(),
            Self::TSModuleDeclaration(n) => n.parent(),
            Self::TSGlobalDeclaration(n) => n.parent(),
            Self::TSModuleBlock(n) => n.parent(),
            Self::TSTypeLiteral(n) => n.parent(),
            Self::TSInferType(n) => n.parent(),
            Self::TSTypeQuery(n) => n.parent(),
            Self::TSImportType(n) => n.parent(),
            Self::TSImportTypeQualifiedName(n) => n.parent(),
            Self::TSFunctionType(n) => n.parent(),
            Self::TSConstructorType(n) => n.parent(),
            Self::TSMappedType(n) => n.parent(),
            Self::TSTemplateLiteralType(n) => n.parent(),
            Self::TSAsExpression(n) => n.parent(),
            Self::TSSatisfiesExpression(n) => n.parent(),
            Self::TSTypeAssertion(n) => n.parent(),
            Self::TSImportEqualsDeclaration(n) => n.parent(),
            Self::TSExternalModuleReference(n) => n.parent(),
            Self::TSNonNullExpression(n) => n.parent(),
            Self::Decorator(n) => n.parent(),
            Self::TSExportAssignment(n) => n.parent(),
            Self::TSNamespaceExportDeclaration(n) => n.parent(),
            Self::TSInstantiationExpression(n) => n.parent(),
            Self::JSDocNullableType(n) => n.parent(),
            Self::JSDocNonNullableType(n) => n.parent(),
            Self::JSDocUnknownType(n) => n.parent(),
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
            Self::MetaProperty(_) => "MetaProperty",
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
            Self::ExportNamedDeclaration(_) => "ExportNamedDeclaration",
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
            Self::TSModuleDeclaration(_) => "TSModuleDeclaration",
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

impl<'a> AstNode<'a, '_, Program<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn source_type(&self) -> SourceType {
        self.inner.source_type
    }

    #[inline]
    pub fn source_text(&self) -> &'a str {
        self.inner.source_text
    }

    #[inline]
    pub fn comments(&self) -> &Vec<'a, Comment> {
        &self.inner.comments
    }

    #[inline]
    pub fn hashbang<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Hashbang<'a>>> {
        let following_span_start = self
            .inner
            .directives
            .first()
            .map(|n| n.span().start)
            .or_else(|| self.inner.body.first().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.hashbang.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::Program(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn directives<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Directive<'a>>> {
        let following_span_start = self
            .inner
            .body
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: AstNodes::Program(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::Program(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, Expression<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            Expression::BooleanLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::BooleanLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::NullLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::NullLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::NumericLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::NumericLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::BigIntLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::BigIntLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::RegExpLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::RegExpLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::StringLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::StringLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TemplateLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TemplateLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::Identifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::IdentifierReference(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::MetaProperty(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::MetaProperty(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::Super(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::Super(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ArrayExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ArrayExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ArrowFunctionExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ArrowFunctionExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::AssignmentExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::AssignmentExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::AwaitExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::AwaitExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::BinaryExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::BinaryExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::CallExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::CallExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ChainExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ChainExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ClassExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::Class(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ConditionalExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ConditionalExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::FunctionExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::Function(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ImportExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ImportExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::LogicalExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::LogicalExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::NewExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::NewExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ObjectExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ObjectExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ParenthesizedExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ParenthesizedExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::SequenceExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::SequenceExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TaggedTemplateExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TaggedTemplateExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::ThisExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ThisExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::UnaryExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::UnaryExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::UpdateExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::UpdateExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::YieldExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::YieldExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::PrivateInExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::PrivateInExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::JSXElement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXElement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::JSXFragment(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXFragment(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TSAsExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSAsExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TSSatisfiesExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSSatisfiesExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TSTypeAssertion(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSTypeAssertion(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TSNonNullExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSNonNullExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::TSInstantiationExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSInstantiationExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Expression::V8IntrinsicExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::V8IntrinsicExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_member_expression!(Expression) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_member_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, IdentifierName<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(&self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, IdentifierReference<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(&self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, BindingIdentifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(&self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, LabelIdentifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(&self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ThisExpression> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ArrayExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn elements<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, ArrayExpressionElement<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: AstNodes::ArrayExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ArrayExpressionElement<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            ArrayExpressionElement::SpreadElement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::SpreadElement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ArrayExpressionElement::Elision(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::Elision(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(ArrayExpressionElement) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, Elision> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ObjectExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn properties<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, ObjectPropertyKind<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: AstNodes::ObjectExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ObjectPropertyKind<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            ObjectPropertyKind::ObjectProperty(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ObjectProperty(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ObjectPropertyKind::SpreadProperty(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::SpreadElement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, ObjectProperty<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn kind(&self) -> PropertyKind {
        self.inner.kind
    }

    #[inline]
    pub fn key<'c>(&'c self) -> &'c AstNode<'a, 'c, PropertyKey<'a>> {
        let following_span_start = self.inner.value.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::ObjectProperty(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn value<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: AstNodes::ObjectProperty(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn method(&self) -> bool {
        self.inner.method
    }

    #[inline]
    pub fn shorthand(&self) -> bool {
        self.inner.shorthand
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, PropertyKey<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            PropertyKey::StaticIdentifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::IdentifierName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            PropertyKey::PrivateIdentifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::PrivateIdentifier(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(PropertyKey) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, TemplateLiteral<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn quasis<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, TemplateElement<'a>>> {
        let following_span_start = self
            .inner
            .expressions
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.quasis,
            allocator: self.allocator,
            parent: AstNodes::TemplateLiteral(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn expressions<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expressions,
            allocator: self.allocator,
            parent: AstNodes::TemplateLiteral(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TaggedTemplateExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn tag<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.quasi.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.tag,
            allocator: self.allocator,
            parent: AstNodes::TaggedTemplateExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.inner.quasi.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TaggedTemplateExpression(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn quasi<'c>(&'c self) -> &'c AstNode<'a, 'c, TemplateLiteral<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.quasi,
            allocator: self.allocator,
            parent: AstNodes::TaggedTemplateExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TemplateElement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(&self) -> &TemplateElementValue<'a> {
        &self.inner.value
    }

    #[inline]
    pub fn tail(&self) -> bool {
        self.inner.tail
    }

    #[inline]
    pub fn lone_surrogates(&self) -> bool {
        self.inner.lone_surrogates
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, MemberExpression<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            MemberExpression::ComputedMemberExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ComputedMemberExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            MemberExpression::StaticMemberExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::StaticMemberExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            MemberExpression::PrivateFieldExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::PrivateFieldExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, ComputedMemberExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.expression.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: AstNodes::ComputedMemberExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::ComputedMemberExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, StaticMemberExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.property.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: AstNodes::StaticMemberExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn property<'c>(&'c self) -> &'c AstNode<'a, 'c, IdentifierName<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: AstNodes::StaticMemberExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, PrivateFieldExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.field.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: AstNodes::PrivateFieldExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn field<'c>(&'c self) -> &'c AstNode<'a, 'c, PrivateIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.field,
            allocator: self.allocator,
            parent: AstNodes::PrivateFieldExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, CallExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn callee<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.arguments.first().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.callee,
            allocator: self.allocator,
            parent: AstNodes::CallExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self
            .inner
            .arguments
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::CallExpression(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn arguments<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Argument<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: AstNodes::CallExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn pure(&self) -> bool {
        self.inner.pure
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, NewExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn callee<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.arguments.first().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.callee,
            allocator: self.allocator,
            parent: AstNodes::NewExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self
            .inner
            .arguments
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::NewExpression(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn arguments<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Argument<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: AstNodes::NewExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn pure(&self) -> bool {
        self.inner.pure
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, MetaProperty<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn meta<'c>(&'c self) -> &'c AstNode<'a, 'c, IdentifierName<'a>> {
        let following_span_start = self.inner.property.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.meta,
            allocator: self.allocator,
            parent: AstNodes::MetaProperty(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn property<'c>(&'c self) -> &'c AstNode<'a, 'c, IdentifierName<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: AstNodes::MetaProperty(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, SpreadElement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: AstNodes::SpreadElement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, Argument<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            Argument::SpreadElement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::SpreadElement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(Argument) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, UpdateExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn operator(&self) -> UpdateOperator {
        self.inner.operator
    }

    #[inline]
    pub fn prefix(&self) -> bool {
        self.inner.prefix
    }

    #[inline]
    pub fn argument<'c>(&'c self) -> &'c AstNode<'a, 'c, SimpleAssignmentTarget<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: AstNodes::UpdateExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, UnaryExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn operator(&self) -> UnaryOperator {
        self.inner.operator
    }

    #[inline]
    pub fn argument<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: AstNodes::UnaryExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, BinaryExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.right.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::BinaryExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn operator(&self) -> BinaryOperator {
        self.inner.operator
    }

    #[inline]
    pub fn right<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::BinaryExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, PrivateInExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left<'c>(&'c self) -> &'c AstNode<'a, 'c, PrivateIdentifier<'a>> {
        let following_span_start = self.inner.right.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::PrivateInExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn right<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::PrivateInExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, LogicalExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.right.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::LogicalExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn operator(&self) -> LogicalOperator {
        self.inner.operator
    }

    #[inline]
    pub fn right<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::LogicalExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ConditionalExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn test<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.consequent.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: AstNodes::ConditionalExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn consequent<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.alternate.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: AstNodes::ConditionalExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn alternate<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.alternate,
            allocator: self.allocator,
            parent: AstNodes::ConditionalExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, AssignmentExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn operator(&self) -> AssignmentOperator {
        self.inner.operator
    }

    #[inline]
    pub fn left<'c>(&'c self) -> &'c AstNode<'a, 'c, AssignmentTarget<'a>> {
        let following_span_start = self.inner.right.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::AssignmentExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn right<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::AssignmentExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, AssignmentTarget<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        #[expect(clippy::needless_return)]
        match self.inner {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_simple_assignment_target(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_assignment_target_pattern(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        }
    }
}

impl<'a> AstNode<'a, '_, SimpleAssignmentTarget<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::IdentifierReference(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            SimpleAssignmentTarget::TSAsExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSAsExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSSatisfiesExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            SimpleAssignmentTarget::TSNonNullExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSNonNullExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            SimpleAssignmentTarget::TSTypeAssertion(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSTypeAssertion(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_member_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, AssignmentTargetPattern<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            AssignmentTargetPattern::ArrayAssignmentTarget(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ArrayAssignmentTarget(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ObjectAssignmentTarget(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, ArrayAssignmentTarget<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn elements<'c>(
        &'c self,
    ) -> &'c AstNode<'a, 'c, Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>> {
        let following_span_start = self.inner.rest.as_deref().map_or(0, |n| n.span().start);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: AstNodes::ArrayAssignmentTarget(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn rest<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, AssignmentTargetRest<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ArrayAssignmentTarget(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ObjectAssignmentTarget<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn properties<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, AssignmentTargetProperty<'a>>> {
        let following_span_start = self.inner.rest.as_deref().map_or(0, |n| n.span().start);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: AstNodes::ObjectAssignmentTarget(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn rest<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, AssignmentTargetRest<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ObjectAssignmentTarget(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, AssignmentTargetRest<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn target<'c>(&'c self) -> &'c AstNode<'a, 'c, AssignmentTarget<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.target,
            allocator: self.allocator,
            parent: AstNodes::AssignmentTargetRest(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, AssignmentTargetMaybeDefault<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::AssignmentTargetWithDefault(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_assignment_target(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, AssignmentTargetWithDefault<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn binding<'c>(&'c self) -> &'c AstNode<'a, 'c, AssignmentTarget<'a>> {
        let following_span_start = self.inner.init.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: AstNodes::AssignmentTargetWithDefault(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn init<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.init,
            allocator: self.allocator,
            parent: AstNodes::AssignmentTargetWithDefault(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, AssignmentTargetProperty<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::AssignmentTargetPropertyIdentifier(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::AssignmentTargetPropertyProperty(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, AssignmentTargetPropertyIdentifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn binding<'c>(&'c self) -> &'c AstNode<'a, 'c, IdentifierReference<'a>> {
        let following_span_start = self
            .inner
            .init
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: AstNodes::AssignmentTargetPropertyIdentifier(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn init<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::AssignmentTargetPropertyIdentifier(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, AssignmentTargetPropertyProperty<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name<'c>(&'c self) -> &'c AstNode<'a, 'c, PropertyKey<'a>> {
        let following_span_start = self.inner.binding.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: AstNodes::AssignmentTargetPropertyProperty(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn binding<'c>(&'c self) -> &'c AstNode<'a, 'c, AssignmentTargetMaybeDefault<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: AstNodes::AssignmentTargetPropertyProperty(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, SequenceExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expressions<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expressions,
            allocator: self.allocator,
            parent: AstNodes::SequenceExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, Super> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, AwaitExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: AstNodes::AwaitExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ChainExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, ChainElement<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::ChainExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ChainElement<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            ChainElement::CallExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::CallExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ChainElement::TSNonNullExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSNonNullExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_member_expression!(ChainElement) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_member_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, ParenthesizedExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::ParenthesizedExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, Statement<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            Statement::BlockStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::BlockStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::BreakStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::BreakStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ContinueStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ContinueStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::DebuggerStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::DebuggerStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::DoWhileStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::DoWhileStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::EmptyStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::EmptyStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ExpressionStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ExpressionStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ForInStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ForInStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ForOfStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ForOfStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ForStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ForStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::IfStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::IfStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::LabeledStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::LabeledStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ReturnStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ReturnStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::SwitchStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::SwitchStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::ThrowStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ThrowStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::TryStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TryStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::WhileStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::WhileStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Statement::WithStatement(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::WithStatement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_declaration!(Statement) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_declaration(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
            it @ match_module_declaration!(Statement) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_module_declaration(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, Directive<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, StringLiteral<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::Directive(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn directive(&self) -> Str<'a> {
        self.inner.directive
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, Hashbang<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(&self) -> Str<'a> {
        self.inner.value
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, BlockStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Statement<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::BlockStatement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, Declaration<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            Declaration::VariableDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::VariableDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::FunctionDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::Function(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::ClassDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::Class(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSTypeAliasDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSTypeAliasDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSInterfaceDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSInterfaceDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSEnumDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSEnumDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSModuleDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSModuleDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSGlobalDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSGlobalDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            Declaration::TSImportEqualsDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSImportEqualsDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, VariableDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn kind(&self) -> VariableDeclarationKind {
        self.inner.kind
    }

    #[inline]
    pub fn declarations<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, VariableDeclarator<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.declarations,
            allocator: self.allocator,
            parent: AstNodes::VariableDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, VariableDeclarator<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn kind(&self) -> VariableDeclarationKind {
        self.inner.kind
    }

    #[inline]
    pub fn id<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingPattern<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.init.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::VariableDeclarator(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .init
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::VariableDeclarator(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn init<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::VariableDeclarator(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn definite(&self) -> bool {
        self.inner.definite
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, EmptyStatement> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ExpressionStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::ExpressionStatement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, IfStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn test<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.consequent.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: AstNodes::IfStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn consequent<'c>(&'c self) -> &'c AstNode<'a, 'c, Statement<'a>> {
        let following_span_start = self.inner.alternate.as_ref().map_or(0, |n| n.span().start);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: AstNodes::IfStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn alternate<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Statement<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.alternate.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::IfStatement(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, DoWhileStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, Statement<'a>> {
        let following_span_start = self.inner.test.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::DoWhileStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn test<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: AstNodes::DoWhileStatement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, WhileStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn test<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.body.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: AstNodes::WhileStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, Statement<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::WhileStatement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ForStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn init<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, ForStatementInit<'a>>> {
        let following_span_start = self
            .inner
            .test
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.update.as_ref().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ForStatement(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn test<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Expression<'a>>> {
        let following_span_start = self
            .inner
            .update
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.test.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ForStatement(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn update<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Expression<'a>>> {
        let following_span_start = self.inner.body.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.update.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ForStatement(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, Statement<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::ForStatement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ForStatementInit<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            ForStatementInit::VariableDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::VariableDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(ForStatementInit) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, ForInStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left<'c>(&'c self) -> &'c AstNode<'a, 'c, ForStatementLeft<'a>> {
        let following_span_start = self.inner.right.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::ForInStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn right<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.body.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::ForInStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, Statement<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::ForInStatement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ForStatementLeft<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            ForStatementLeft::VariableDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::VariableDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_assignment_target!(ForStatementLeft) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_assignment_target(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, ForOfStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#await(&self) -> bool {
        self.inner.r#await
    }

    #[inline]
    pub fn left<'c>(&'c self) -> &'c AstNode<'a, 'c, ForStatementLeft<'a>> {
        let following_span_start = self.inner.right.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::ForOfStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn right<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.body.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::ForOfStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, Statement<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::ForOfStatement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ContinueStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn label<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, LabelIdentifier<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.label.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ContinueStatement(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, BreakStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn label<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, LabelIdentifier<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.label.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::BreakStatement(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ReturnStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Expression<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.argument.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ReturnStatement(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, WithStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.body.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: AstNodes::WithStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, Statement<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::WithStatement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, SwitchStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn discriminant<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.cases.first().map_or(0, |n| n.span().start);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.discriminant,
            allocator: self.allocator,
            parent: AstNodes::SwitchStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn cases<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, SwitchCase<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.cases,
            allocator: self.allocator,
            parent: AstNodes::SwitchStatement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, SwitchCase<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn test<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Expression<'a>>> {
        let following_span_start = self
            .inner
            .consequent
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.test.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::SwitchCase(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn consequent<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: AstNodes::SwitchCase(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, LabeledStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn label<'c>(&'c self) -> &'c AstNode<'a, 'c, LabelIdentifier<'a>> {
        let following_span_start = self.inner.body.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.label,
            allocator: self.allocator,
            parent: AstNodes::LabeledStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, Statement<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::LabeledStatement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ThrowStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: AstNodes::ThrowStatement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TryStatement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn block<'c>(&'c self) -> &'c AstNode<'a, 'c, BlockStatement<'a>> {
        let following_span_start = self
            .inner
            .handler
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.finalizer.as_deref().map(|n| n.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.block.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TryStatement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn handler<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, CatchClause<'a>>> {
        let following_span_start = self.inner.finalizer.as_deref().map_or(0, |n| n.span().start);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.handler.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TryStatement(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn finalizer<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, BlockStatement<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.finalizer.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TryStatement(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, CatchClause<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn param<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, CatchParameter<'a>>> {
        let following_span_start = self.inner.body.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.param.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::CatchClause(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, BlockStatement<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::CatchClause(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, CatchParameter<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn pattern<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingPattern<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.pattern,
            allocator: self.allocator,
            parent: AstNodes::CatchParameter(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::CatchParameter(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, DebuggerStatement> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, BindingPattern<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            BindingPattern::BindingIdentifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::BindingIdentifier(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            BindingPattern::ObjectPattern(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ObjectPattern(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            BindingPattern::ArrayPattern(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ArrayPattern(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            BindingPattern::AssignmentPattern(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::AssignmentPattern(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, AssignmentPattern<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingPattern<'a>> {
        let following_span_start = self.inner.right.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::AssignmentPattern(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn right<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::AssignmentPattern(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ObjectPattern<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn properties<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, BindingProperty<'a>>> {
        let following_span_start = self.inner.rest.as_deref().map_or(0, |n| n.span().start);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: AstNodes::ObjectPattern(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn rest<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, BindingRestElement<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ObjectPattern(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, BindingProperty<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn key<'c>(&'c self) -> &'c AstNode<'a, 'c, PropertyKey<'a>> {
        let following_span_start = self.inner.value.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::BindingProperty(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn value<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingPattern<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: AstNodes::BindingProperty(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn shorthand(&self) -> bool {
        self.inner.shorthand
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ArrayPattern<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn elements<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Option<BindingPattern<'a>>>> {
        let following_span_start = self.inner.rest.as_deref().map_or(0, |n| n.span().start);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: AstNodes::ArrayPattern(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn rest<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, BindingRestElement<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ArrayPattern(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, BindingRestElement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingPattern<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: AstNodes::BindingRestElement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, Function<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(&self) -> FunctionType {
        self.inner.r#type
    }

    #[inline]
    pub fn id<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, BindingIdentifier<'a>>> {
        let following_span_start = self
            .inner
            .type_parameters
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.this_param.as_deref().map(|n| n.span().start))
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.id.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::Function(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn generator(&self) -> bool {
        self.inner.generator
    }

    #[inline]
    pub fn r#async(&self) -> bool {
        self.inner.r#async
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    #[inline]
    pub fn type_parameters<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .this_param
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::Function(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn this_param<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSThisParameter<'a>>> {
        let following_span_start = self.inner.params.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::Function(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params<'c>(&'c self) -> &'c AstNode<'a, 'c, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.body.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::Function(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn return_type<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .body
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::Function(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn body<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, FunctionBody<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.body.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::Function(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn pure(&self) -> bool {
        self.inner.pure
    }

    #[inline]
    pub fn pife(&self) -> bool {
        self.inner.pife
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, FormalParameters<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn kind(&self) -> FormalParameterKind {
        self.inner.kind
    }

    #[inline]
    pub fn items<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, FormalParameter<'a>>> {
        let following_span_start = self.inner.rest.as_deref().map_or(0, |n| n.span().start);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.items,
            allocator: self.allocator,
            parent: AstNodes::FormalParameters(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn rest<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, FormalParameterRest<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::FormalParameters(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, FormalParameter<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn decorators<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Decorator<'a>>> {
        let following_span_start = self.inner.pattern.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: AstNodes::FormalParameter(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn pattern<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingPattern<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.initializer.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.pattern,
            allocator: self.allocator,
            parent: AstNodes::FormalParameter(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .initializer
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::FormalParameter(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn initializer<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.initializer.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::FormalParameter(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }

    #[inline]
    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    #[inline]
    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, FormalParameterRest<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn decorators<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Decorator<'a>>> {
        let following_span_start = self.inner.rest.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: AstNodes::FormalParameterRest(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn rest<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingRestElement<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.rest,
            allocator: self.allocator,
            parent: AstNodes::FormalParameterRest(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::FormalParameterRest(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, FunctionBody<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn directives<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Directive<'a>>> {
        let following_span_start = self
            .inner
            .statements
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: AstNodes::FunctionBody(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn statements<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.statements,
            allocator: self.allocator,
            parent: AstNodes::FunctionBody(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ArrowFunctionExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression(&self) -> bool {
        self.inner.expression
    }

    #[inline]
    pub fn r#async(&self) -> bool {
        self.inner.r#async
    }

    #[inline]
    pub fn type_parameters<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self.inner.params.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ArrowFunctionExpression(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params<'c>(&'c self) -> &'c AstNode<'a, 'c, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::ArrowFunctionExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn return_type<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSTypeAnnotation<'a>>> {
        let following_span_start = self.inner.body.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ArrowFunctionExpression(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, FunctionBody<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::ArrowFunctionExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn pure(&self) -> bool {
        self.inner.pure
    }

    #[inline]
    pub fn pife(&self) -> bool {
        self.inner.pife
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, YieldExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn delegate(&self) -> bool {
        self.inner.delegate
    }

    #[inline]
    pub fn argument<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.argument.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::YieldExpression(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, Class<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(&self) -> ClassType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Decorator<'a>>> {
        let following_span_start = self
            .inner
            .id
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.type_parameters.as_deref().map(|n| n.span().start))
            .or_else(|| self.inner.super_class.as_ref().map(|n| n.span().start))
            .or_else(|| self.inner.super_type_arguments.as_deref().map(|n| n.span().start))
            .or_else(|| self.inner.implements.first().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: AstNodes::Class(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn id<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, BindingIdentifier<'a>>> {
        let following_span_start = self
            .inner
            .type_parameters
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.super_class.as_ref().map(|n| n.span().start))
            .or_else(|| self.inner.super_type_arguments.as_deref().map(|n| n.span().start))
            .or_else(|| self.inner.implements.first().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.id.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::Class(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn type_parameters<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .super_class
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.super_type_arguments.as_deref().map(|n| n.span().start))
            .or_else(|| self.inner.implements.first().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::Class(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn super_class<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Expression<'a>>> {
        let following_span_start = self
            .inner
            .super_type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.implements.first().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.super_class.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::Class(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn super_type_arguments<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self
            .inner
            .implements
            .first()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.super_type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::Class(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn implements<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, TSClassImplements<'a>>> {
        let following_span_start = self.inner.body.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.implements,
            allocator: self.allocator,
            parent: AstNodes::Class(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, ClassBody<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::Class(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn r#abstract(&self) -> bool {
        self.inner.r#abstract
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ClassBody<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, ClassElement<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::ClassBody(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ClassElement<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            ClassElement::StaticBlock(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::StaticBlock(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ClassElement::MethodDefinition(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::MethodDefinition(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ClassElement::PropertyDefinition(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::PropertyDefinition(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ClassElement::AccessorProperty(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::AccessorProperty(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ClassElement::TSIndexSignature(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSIndexSignature(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, MethodDefinition<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(&self) -> MethodDefinitionType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Decorator<'a>>> {
        let following_span_start = self.inner.key.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: AstNodes::MethodDefinition(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn key<'c>(&'c self) -> &'c AstNode<'a, 'c, PropertyKey<'a>> {
        let following_span_start = self.inner.value.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::MethodDefinition(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn value<'c>(&'c self) -> &'c AstNode<'a, 'c, Function<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.value.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::MethodDefinition(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn kind(&self) -> MethodDefinitionKind {
        self.inner.kind
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }

    #[inline]
    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, PropertyDefinition<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(&self) -> PropertyDefinitionType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Decorator<'a>>> {
        let following_span_start = self.inner.key.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: AstNodes::PropertyDefinition(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn key<'c>(&'c self) -> &'c AstNode<'a, 'c, PropertyKey<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.value.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::PropertyDefinition(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .value
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::PropertyDefinition(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn value<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::PropertyDefinition(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    #[inline]
    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn definite(&self) -> bool {
        self.inner.definite
    }

    #[inline]
    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    #[inline]
    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, PrivateIdentifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(&self) -> Ident<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, StaticBlock<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::StaticBlock(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ModuleDeclaration<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            ModuleDeclaration::ImportDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ImportDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::ExportAllDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ExportAllDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::ExportDefaultDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ExportDefaultDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::ExportNamedDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ExportNamedDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::TSExportAssignment(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSExportAssignment(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSNamespaceExportDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, AccessorProperty<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#type(&self) -> AccessorPropertyType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Decorator<'a>>> {
        let following_span_start = self.inner.key.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: AstNodes::AccessorProperty(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn key<'c>(&'c self) -> &'c AstNode<'a, 'c, PropertyKey<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.value.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::AccessorProperty(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSTypeAnnotation<'a>>> {
        let following_span_start = self
            .inner
            .value
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::AccessorProperty(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn value<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::AccessorProperty(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }

    #[inline]
    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }

    #[inline]
    pub fn definite(&self) -> bool {
        self.inner.definite
    }

    #[inline]
    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ImportExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn source<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self
            .inner
            .options
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: AstNodes::ImportExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn options<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.options.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ImportExpression(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn phase(&self) -> Option<ImportPhase> {
        self.inner.phase
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ImportDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn specifiers<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, Vec<'a, ImportDeclarationSpecifier<'a>>>> {
        let following_span_start = self.inner.source.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.specifiers.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ImportDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn source<'c>(&'c self) -> &'c AstNode<'a, 'c, StringLiteral<'a>> {
        let following_span_start = self.inner.with_clause.as_deref().map_or(0, |n| n.span().start);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: AstNodes::ImportDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn phase(&self) -> Option<ImportPhase> {
        self.inner.phase
    }

    #[inline]
    pub fn with_clause<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, WithClause<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ImportDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ImportDeclarationSpecifier<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ImportSpecifier(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ImportDefaultSpecifier(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ImportNamespaceSpecifier(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, ImportSpecifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn imported<'c>(&'c self) -> &'c AstNode<'a, 'c, ModuleExportName<'a>> {
        let following_span_start = self.inner.local.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.imported,
            allocator: self.allocator,
            parent: AstNodes::ImportSpecifier(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn local<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: AstNodes::ImportSpecifier(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ImportDefaultSpecifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn local<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: AstNodes::ImportDefaultSpecifier(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ImportNamespaceSpecifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn local<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: AstNodes::ImportNamespaceSpecifier(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, WithClause<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn keyword(&self) -> WithClauseKeyword {
        self.inner.keyword
    }

    #[inline]
    pub fn with_entries<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, ImportAttribute<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.with_entries,
            allocator: self.allocator,
            parent: AstNodes::WithClause(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ImportAttribute<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn key<'c>(&'c self) -> &'c AstNode<'a, 'c, ImportAttributeKey<'a>> {
        let following_span_start = self.inner.value.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::ImportAttribute(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn value<'c>(&'c self) -> &'c AstNode<'a, 'c, StringLiteral<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: AstNodes::ImportAttribute(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ImportAttributeKey<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            ImportAttributeKey::Identifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::IdentifierName(allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ImportAttributeKey::StringLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::StringLiteral(allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, ExportNamedDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn declaration<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Declaration<'a>>> {
        let following_span_start = self
            .inner
            .specifiers
            .first()
            .map(|n| n.span().start)
            .or_else(|| self.inner.source.as_ref().map(|n| n.span().start))
            .or_else(|| self.inner.with_clause.as_deref().map(|n| n.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.declaration.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ExportNamedDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn specifiers<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, ExportSpecifier<'a>>> {
        let following_span_start = self
            .inner
            .source
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.with_clause.as_deref().map(|n| n.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.specifiers,
            allocator: self.allocator,
            parent: AstNodes::ExportNamedDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn source<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, StringLiteral<'a>>> {
        let following_span_start = self.inner.with_clause.as_deref().map_or(0, |n| n.span().start);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.source.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ExportNamedDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    #[inline]
    pub fn with_clause<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, WithClause<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ExportNamedDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ExportDefaultDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn declaration<'c>(&'c self) -> &'c AstNode<'a, 'c, ExportDefaultDeclarationKind<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.declaration,
            allocator: self.allocator,
            parent: AstNodes::ExportDefaultDeclaration(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ExportAllDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn exported<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, ModuleExportName<'a>>> {
        let following_span_start = self.inner.source.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.exported.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::ExportAllDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn source<'c>(&'c self) -> &'c AstNode<'a, 'c, StringLiteral<'a>> {
        let following_span_start = self.inner.with_clause.as_deref().map_or(0, |n| n.span().start);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: AstNodes::ExportAllDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn with_clause<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, WithClause<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::ExportAllDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ExportSpecifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn local<'c>(&'c self) -> &'c AstNode<'a, 'c, ModuleExportName<'a>> {
        let following_span_start = self.inner.exported.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: AstNodes::ExportSpecifier(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn exported<'c>(&'c self) -> &'c AstNode<'a, 'c, ModuleExportName<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.exported,
            allocator: self.allocator,
            parent: AstNodes::ExportSpecifier(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, ExportDefaultDeclarationKind<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            ExportDefaultDeclarationKind::FunctionDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::Function(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ExportDefaultDeclarationKind::ClassDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::Class(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSInterfaceDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(ExportDefaultDeclarationKind) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, ModuleExportName<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            ModuleExportName::IdentifierName(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::IdentifierName(allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleExportName::IdentifierReference(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::IdentifierReference(allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            ModuleExportName::StringLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::StringLiteral(allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, V8IntrinsicExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name<'c>(&'c self) -> &'c AstNode<'a, 'c, IdentifierName<'a>> {
        let following_span_start = self
            .inner
            .arguments
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: AstNodes::V8IntrinsicExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn arguments<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Argument<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: AstNodes::V8IntrinsicExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, BooleanLiteral> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(&self) -> bool {
        self.inner.value
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, NullLiteral> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, NumericLiteral<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(&self) -> f64 {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Str<'a>> {
        self.inner.raw
    }

    #[inline]
    pub fn base(&self) -> NumberBase {
        self.inner.base
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, StringLiteral<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(&self) -> Str<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Str<'a>> {
        self.inner.raw
    }

    #[inline]
    pub fn lone_surrogates(&self) -> bool {
        self.inner.lone_surrogates
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, BigIntLiteral<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(&self) -> Str<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Str<'a>> {
        self.inner.raw
    }

    #[inline]
    pub fn base(&self) -> BigintBase {
        self.inner.base
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, RegExpLiteral<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn regex(&self) -> &RegExp<'a> {
        &self.inner.regex
    }

    #[inline]
    pub fn raw(&self) -> Option<Str<'a>> {
        self.inner.raw
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXElement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn opening_element<'c>(&'c self) -> &'c AstNode<'a, 'c, JSXOpeningElement<'a>> {
        let following_span_start = self
            .inner
            .children
            .first()
            .map(|n| n.span().start)
            .or_else(|| self.inner.closing_element.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.opening_element.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::JSXElement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn children<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, JSXChild<'a>>> {
        let following_span_start = self
            .inner
            .closing_element
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.children,
            allocator: self.allocator,
            parent: AstNodes::JSXElement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn closing_element<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, JSXClosingElement<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.closing_element.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::JSXElement(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXOpeningElement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name<'c>(&'c self) -> &'c AstNode<'a, 'c, JSXElementName<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.attributes.first().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: AstNodes::JSXOpeningElement(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self
            .inner
            .attributes
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::JSXOpeningElement(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn attributes<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, JSXAttributeItem<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.attributes,
            allocator: self.allocator,
            parent: AstNodes::JSXOpeningElement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXClosingElement<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name<'c>(&'c self) -> &'c AstNode<'a, 'c, JSXElementName<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: AstNodes::JSXClosingElement(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXFragment<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn opening_fragment<'c>(&'c self) -> &'c AstNode<'a, 'c, JSXOpeningFragment> {
        let following_span_start = self
            .inner
            .children
            .first()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.closing_fragment.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.opening_fragment,
            allocator: self.allocator,
            parent: AstNodes::JSXFragment(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn children<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, JSXChild<'a>>> {
        let following_span_start = self.inner.closing_fragment.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.children,
            allocator: self.allocator,
            parent: AstNodes::JSXFragment(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn closing_fragment<'c>(&'c self) -> &'c AstNode<'a, 'c, JSXClosingFragment> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.closing_fragment,
            allocator: self.allocator,
            parent: AstNodes::JSXFragment(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXOpeningFragment> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXClosingFragment> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXElementName<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            JSXElementName::Identifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXIdentifier(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXElementName::IdentifierReference(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::IdentifierReference(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXElementName::NamespacedName(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXNamespacedName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXElementName::MemberExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXMemberExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXElementName::ThisExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ThisExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, JSXNamespacedName<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn namespace<'c>(&'c self) -> &'c AstNode<'a, 'c, JSXIdentifier<'a>> {
        let following_span_start = self.inner.name.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.namespace,
            allocator: self.allocator,
            parent: AstNodes::JSXNamespacedName(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn name<'c>(&'c self) -> &'c AstNode<'a, 'c, JSXIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: AstNodes::JSXNamespacedName(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXMemberExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object<'c>(&'c self) -> &'c AstNode<'a, 'c, JSXMemberExpressionObject<'a>> {
        let following_span_start = self.inner.property.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: AstNodes::JSXMemberExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn property<'c>(&'c self) -> &'c AstNode<'a, 'c, JSXIdentifier<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: AstNodes::JSXMemberExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXMemberExpressionObject<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            JSXMemberExpressionObject::IdentifierReference(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::IdentifierReference(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXMemberExpressionObject::MemberExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXMemberExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXMemberExpressionObject::ThisExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ThisExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, JSXExpressionContainer<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, JSXExpression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::JSXExpressionContainer(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXExpression<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            JSXExpression::EmptyExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXEmptyExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_expression!(JSXExpression) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, JSXEmptyExpression> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXAttributeItem<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            JSXAttributeItem::Attribute(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXAttribute(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXAttributeItem::SpreadAttribute(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXSpreadAttribute(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, JSXAttribute<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name<'c>(&'c self) -> &'c AstNode<'a, 'c, JSXAttributeName<'a>> {
        let following_span_start = self
            .inner
            .value
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: AstNodes::JSXAttribute(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn value<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, JSXAttributeValue<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::JSXAttribute(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXSpreadAttribute<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn argument<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: AstNodes::JSXSpreadAttribute(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXAttributeName<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            JSXAttributeName::Identifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXIdentifier(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXAttributeName::NamespacedName(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXNamespacedName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, JSXAttributeValue<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            JSXAttributeValue::StringLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::StringLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXAttributeValue::ExpressionContainer(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXExpressionContainer(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXAttributeValue::Element(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXElement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXAttributeValue::Fragment(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXFragment(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, JSXIdentifier<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(&self) -> Str<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXChild<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            JSXChild::Text(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXText(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXChild::Element(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXElement(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXChild::Fragment(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXFragment(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXChild::ExpressionContainer(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXExpressionContainer(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            JSXChild::Spread(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSXSpreadChild(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, JSXSpreadChild<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::JSXSpreadChild(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSXText<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn value(&self) -> Str<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Str<'a>> {
        self.inner.raw
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSThisParameter<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn this_span(&self) -> Span {
        self.inner.this_span
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSThisParameter(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSEnumDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingIdentifier<'a>> {
        let following_span_start = self.inner.body.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::TSEnumDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, TSEnumBody<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::TSEnumDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn r#const(&self) -> bool {
        self.inner.r#const
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSEnumBody<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn members<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, TSEnumMember<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.members,
            allocator: self.allocator,
            parent: AstNodes::TSEnumBody(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSEnumMember<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id<'c>(&'c self) -> &'c AstNode<'a, 'c, TSEnumMemberName<'a>> {
        let following_span_start = self
            .inner
            .initializer
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::TSEnumMember(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn initializer<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, Expression<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.initializer.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::TSEnumMember(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSEnumMemberName<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            TSEnumMemberName::Identifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::IdentifierName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSEnumMemberName::String(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::StringLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSEnumMemberName::ComputedString(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::StringLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSEnumMemberName::ComputedTemplateString(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TemplateLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, TSTypeAnnotation<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSTypeAnnotation(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSLiteralType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn literal<'c>(&'c self) -> &'c AstNode<'a, 'c, TSLiteral<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.literal,
            allocator: self.allocator,
            parent: AstNodes::TSLiteralType(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSLiteral<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            TSLiteral::BooleanLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::BooleanLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSLiteral::NumericLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::NumericLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSLiteral::BigIntLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::BigIntLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSLiteral::StringLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::StringLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSLiteral::TemplateLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TemplateLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSLiteral::UnaryExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::UnaryExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, TSType<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            TSType::TSAnyKeyword(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSAnyKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSBigIntKeyword(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSBigIntKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSBooleanKeyword(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSBooleanKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSIntrinsicKeyword(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSIntrinsicKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSNeverKeyword(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSNeverKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSNullKeyword(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSNullKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSNumberKeyword(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSNumberKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSObjectKeyword(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSObjectKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSStringKeyword(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSStringKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSSymbolKeyword(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSSymbolKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSUndefinedKeyword(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSUndefinedKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSUnknownKeyword(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSUnknownKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSVoidKeyword(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSVoidKeyword(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSArrayType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSArrayType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSConditionalType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSConditionalType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSConstructorType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSConstructorType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSFunctionType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSFunctionType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSImportType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSImportType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSIndexedAccessType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSIndexedAccessType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSInferType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSInferType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSIntersectionType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSIntersectionType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSLiteralType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSLiteralType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSMappedType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSMappedType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSNamedTupleMember(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSNamedTupleMember(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSTemplateLiteralType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSTemplateLiteralType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSThisType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSThisType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSTupleType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSTupleType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSTypeLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSTypeLiteral(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSTypeOperatorType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSTypeOperator(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSTypePredicate(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSTypePredicate(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSTypeQuery(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSTypeQuery(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSTypeReference(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSTypeReference(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSUnionType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSUnionType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::TSParenthesizedType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSParenthesizedType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::JSDocNullableType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSDocNullableType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::JSDocNonNullableType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSDocNonNullableType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSType::JSDocUnknownType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::JSDocUnknownType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, TSConditionalType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn check_type<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.inner.extends_type.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.check_type,
            allocator: self.allocator,
            parent: AstNodes::TSConditionalType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn extends_type<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.inner.true_type.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.extends_type,
            allocator: self.allocator,
            parent: AstNodes::TSConditionalType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn true_type<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.inner.false_type.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.true_type,
            allocator: self.allocator,
            parent: AstNodes::TSConditionalType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn false_type<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.false_type,
            allocator: self.allocator,
            parent: AstNodes::TSConditionalType(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSUnionType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn types<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: AstNodes::TSUnionType(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSIntersectionType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn types<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: AstNodes::TSIntersectionType(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSParenthesizedType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSParenthesizedType(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTypeOperator<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn operator(&self) -> TSTypeOperatorOperator {
        self.inner.operator
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSTypeOperator(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSArrayType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn element_type<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.element_type,
            allocator: self.allocator,
            parent: AstNodes::TSArrayType(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSIndexedAccessType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn object_type<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.inner.index_type.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.object_type,
            allocator: self.allocator,
            parent: AstNodes::TSIndexedAccessType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn index_type<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.index_type,
            allocator: self.allocator,
            parent: AstNodes::TSIndexedAccessType(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTupleType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn element_types<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, TSTupleElement<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.element_types,
            allocator: self.allocator,
            parent: AstNodes::TSTupleType(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSNamedTupleMember<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn label<'c>(&'c self) -> &'c AstNode<'a, 'c, IdentifierName<'a>> {
        let following_span_start = self.inner.element_type.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.label,
            allocator: self.allocator,
            parent: AstNodes::TSNamedTupleMember(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn element_type<'c>(&'c self) -> &'c AstNode<'a, 'c, TSTupleElement<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.element_type,
            allocator: self.allocator,
            parent: AstNodes::TSNamedTupleMember(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSOptionalType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSOptionalType(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSRestType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSRestType(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTupleElement<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            TSTupleElement::TSOptionalType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSOptionalType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSTupleElement::TSRestType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSRestType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_ts_type!(TSTupleElement) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_ts_type(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, TSAnyKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSStringKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSBooleanKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSNumberKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSNeverKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSIntrinsicKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSUnknownKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSNullKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSUndefinedKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSVoidKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSSymbolKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSThisType> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSObjectKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSBigIntKeyword> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTypeReference<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_name<'c>(&'c self) -> &'c AstNode<'a, 'c, TSTypeName<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.type_name,
            allocator: self.allocator,
            parent: AstNodes::TSTypeReference(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSTypeReference(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTypeName<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            TSTypeName::IdentifierReference(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::IdentifierReference(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSTypeName::QualifiedName(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSQualifiedName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSTypeName::ThisExpression(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::ThisExpression(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, TSQualifiedName<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left<'c>(&'c self) -> &'c AstNode<'a, 'c, TSTypeName<'a>> {
        let following_span_start = self.inner.right.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::TSQualifiedName(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn right<'c>(&'c self) -> &'c AstNode<'a, 'c, IdentifierName<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::TSQualifiedName(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTypeParameterInstantiation<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn params<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.params,
            allocator: self.allocator,
            parent: AstNodes::TSTypeParameterInstantiation(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTypeParameter<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingIdentifier<'a>> {
        let following_span_start = self
            .inner
            .constraint
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.default.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: AstNodes::TSTypeParameter(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn constraint<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSType<'a>>> {
        let following_span_start = self
            .inner
            .default
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.constraint.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::TSTypeParameter(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn default<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.default.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::TSTypeParameter(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn r#in(&self) -> bool {
        self.inner.r#in
    }

    #[inline]
    pub fn out(&self) -> bool {
        self.inner.out
    }

    #[inline]
    pub fn r#const(&self) -> bool {
        self.inner.r#const
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTypeParameterDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn params<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, TSTypeParameter<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.params,
            allocator: self.allocator,
            parent: AstNodes::TSTypeParameterDeclaration(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTypeAliasDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingIdentifier<'a>> {
        let following_span_start = self
            .inner
            .type_parameters
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.type_annotation.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::TSTypeAliasDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_parameters<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self.inner.type_annotation.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSTypeAliasDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSTypeAliasDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSClassImplements<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, TSTypeName<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSClassImplements(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSClassImplements(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSInterfaceDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingIdentifier<'a>> {
        let following_span_start = self
            .inner
            .type_parameters
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.extends.first().map(|n| n.span().start))
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::TSInterfaceDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_parameters<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .extends
            .first()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.body.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSInterfaceDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn extends<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, TSInterfaceHeritage<'a>>> {
        let following_span_start = self.inner.body.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.extends,
            allocator: self.allocator,
            parent: AstNodes::TSInterfaceDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, TSInterfaceBody<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSInterfaceDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSInterfaceBody<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, TSSignature<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::TSInterfaceBody(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSPropertySignature<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    #[inline]
    pub fn key<'c>(&'c self) -> &'c AstNode<'a, 'c, PropertyKey<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::TSPropertySignature(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSPropertySignature(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSSignature<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            TSSignature::TSIndexSignature(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSIndexSignature(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSSignature::TSPropertySignature(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSPropertySignature(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSSignature::TSCallSignatureDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSCallSignatureDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSSignature::TSConstructSignatureDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSConstructSignatureDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSSignature::TSMethodSignature(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSMethodSignature(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, TSIndexSignature<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn parameters<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, TSIndexSignatureName<'a>>> {
        let following_span_start = self.inner.type_annotation.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.parameters,
            allocator: self.allocator,
            parent: AstNodes::TSIndexSignature(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> &'c AstNode<'a, 'c, TSTypeAnnotation<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.type_annotation.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSIndexSignature(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    #[inline]
    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSCallSignatureDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_parameters<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .this_param
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSCallSignatureDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn this_param<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSThisParameter<'a>>> {
        let following_span_start = self.inner.params.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSCallSignatureDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params<'c>(&'c self) -> &'c AstNode<'a, 'c, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSCallSignatureDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn return_type<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSCallSignatureDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSMethodSignature<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn key<'c>(&'c self) -> &'c AstNode<'a, 'c, PropertyKey<'a>> {
        let following_span_start = self
            .inner
            .type_parameters
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.this_param.as_deref().map(|n| n.span().start))
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::TSMethodSignature(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    #[inline]
    pub fn kind(&self) -> TSMethodSignatureKind {
        self.inner.kind
    }

    #[inline]
    pub fn type_parameters<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .this_param
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSMethodSignature(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn this_param<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSThisParameter<'a>>> {
        let following_span_start = self.inner.params.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSMethodSignature(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params<'c>(&'c self) -> &'c AstNode<'a, 'c, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSMethodSignature(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn return_type<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSMethodSignature(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSConstructSignatureDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_parameters<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self.inner.params.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSConstructSignatureDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params<'c>(&'c self) -> &'c AstNode<'a, 'c, FormalParameters<'a>> {
        let following_span_start = self
            .inner
            .return_type
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSConstructSignatureDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn return_type<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSConstructSignatureDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSIndexSignatureName<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn name(&self) -> Str<'a> {
        self.inner.name
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> &'c AstNode<'a, 'c, TSTypeAnnotation<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.type_annotation.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSIndexSignatureName(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSInterfaceHeritage<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSInterfaceHeritage(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSInterfaceHeritage(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTypePredicate<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn parameter_name<'c>(&'c self) -> &'c AstNode<'a, 'c, TSTypePredicateName<'a>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.parameter_name,
            allocator: self.allocator,
            parent: AstNodes::TSTypePredicate(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn asserts(&self) -> bool {
        self.inner.asserts
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSTypeAnnotation<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSTypePredicate(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTypePredicateName<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            TSTypePredicateName::Identifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::IdentifierName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSTypePredicateName::This(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSThisType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, TSModuleDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id<'c>(&'c self) -> &'c AstNode<'a, 'c, TSModuleDeclarationName<'a>> {
        let following_span_start = self.inner.body.as_ref().map_or(0, |n| n.span().start);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::TSModuleDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSModuleDeclarationBody<'a>>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.body.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::TSModuleDeclaration(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn kind(&self) -> TSModuleDeclarationKind {
        self.inner.kind
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSModuleDeclarationName<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            TSModuleDeclarationName::Identifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::BindingIdentifier(allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSModuleDeclarationName::StringLiteral(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::StringLiteral(allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, TSModuleDeclarationBody<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            TSModuleDeclarationBody::TSModuleDeclaration(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSModuleDeclaration(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSModuleDeclarationBody::TSModuleBlock(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSModuleBlock(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, TSGlobalDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn global_span(&self) -> Span {
        self.inner.global_span
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, TSModuleBlock<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::TSGlobalDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSModuleBlock<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn directives<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Directive<'a>>> {
        let following_span_start = self
            .inner
            .body
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: AstNodes::TSModuleBlock(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn body<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, Statement<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: AstNodes::TSModuleBlock(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTypeLiteral<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn members<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, TSSignature<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.members,
            allocator: self.allocator,
            parent: AstNodes::TSTypeLiteral(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSInferType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_parameter<'c>(&'c self) -> &'c AstNode<'a, 'c, TSTypeParameter<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.type_parameter.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSInferType(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTypeQuery<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expr_name<'c>(&'c self) -> &'c AstNode<'a, 'c, TSTypeQueryExprName<'a>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expr_name,
            allocator: self.allocator,
            parent: AstNodes::TSTypeQuery(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSTypeQuery(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTypeQueryExprName<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            TSTypeQueryExprName::TSImportType(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSImportType(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            it @ match_ts_type_name!(TSTypeQueryExprName) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                return allocator
                    .alloc(AstNode {
                        inner: it.to_ts_type_name(),
                        parent,
                        allocator: self.allocator,
                        following_span_start: self.following_span_start,
                    })
                    .as_ast_nodes();
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, TSImportType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn source<'c>(&'c self) -> &'c AstNode<'a, 'c, StringLiteral<'a>> {
        let following_span_start = self
            .inner
            .options
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.qualifier.as_ref().map(|n| n.span().start))
            .or_else(|| self.inner.type_arguments.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: AstNodes::TSImportType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn options<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, ObjectExpression<'a>>> {
        let following_span_start = self
            .inner
            .qualifier
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.type_arguments.as_deref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.options.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSImportType(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn qualifier<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSImportTypeQualifier<'a>>> {
        let following_span_start = self
            .inner
            .type_arguments
            .as_deref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.qualifier.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::TSImportType(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn type_arguments<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterInstantiation<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSImportType(self),
                following_span_start,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSImportTypeQualifier<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            TSImportTypeQualifier::Identifier(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::IdentifierName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSImportTypeQualifier::QualifiedName(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSImportTypeQualifiedName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, TSImportTypeQualifiedName<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn left<'c>(&'c self) -> &'c AstNode<'a, 'c, TSImportTypeQualifier<'a>> {
        let following_span_start = self.inner.right.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: AstNodes::TSImportTypeQualifiedName(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn right<'c>(&'c self) -> &'c AstNode<'a, 'c, IdentifierName<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: AstNodes::TSImportTypeQualifiedName(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSFunctionType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_parameters<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self
            .inner
            .this_param
            .as_deref()
            .map(|n| n.span().start)
            .or_else(|| Some(self.inner.params.span().start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSFunctionType(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn this_param<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSThisParameter<'a>>> {
        let following_span_start = self.inner.params.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSFunctionType(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params<'c>(&'c self) -> &'c AstNode<'a, 'c, FormalParameters<'a>> {
        let following_span_start = self.inner.return_type.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSFunctionType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn return_type<'c>(&'c self) -> &'c AstNode<'a, 'c, TSTypeAnnotation<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.return_type.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSFunctionType(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSConstructorType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn r#abstract(&self) -> bool {
        self.inner.r#abstract
    }

    #[inline]
    pub fn type_parameters<'c>(
        &'c self,
    ) -> Option<&'c AstNode<'a, 'c, TSTypeParameterDeclaration<'a>>> {
        let following_span_start = self.inner.params.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: AstNodes::TSConstructorType(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params<'c>(&'c self) -> &'c AstNode<'a, 'c, FormalParameters<'a>> {
        let following_span_start = self.inner.return_type.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSConstructorType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn return_type<'c>(&'c self) -> &'c AstNode<'a, 'c, TSTypeAnnotation<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.return_type.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSConstructorType(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSMappedType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn key<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingIdentifier<'a>> {
        let following_span_start = self.inner.constraint.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: AstNodes::TSMappedType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn constraint<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self
            .inner
            .name_type
            .as_ref()
            .map(|n| n.span().start)
            .or_else(|| self.inner.type_annotation.as_ref().map(|n| n.span().start))
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.constraint,
            allocator: self.allocator,
            parent: AstNodes::TSMappedType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn name_type<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSType<'a>>> {
        let following_span_start = self
            .inner
            .type_annotation
            .as_ref()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.name_type.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::TSMappedType(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> Option<&'c AstNode<'a, 'c, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: AstNodes::TSMappedType(self),
                following_span_start,
            }))
            .as_ref()
    }

    #[inline]
    pub fn optional(&self) -> Option<TSMappedTypeModifierOperator> {
        self.inner.optional
    }

    #[inline]
    pub fn readonly(&self) -> Option<TSMappedTypeModifierOperator> {
        self.inner.readonly
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTemplateLiteralType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn quasis<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, TemplateElement<'a>>> {
        let following_span_start = self
            .inner
            .types
            .first()
            .map(|n| n.span().start)
            .or(Some(self.following_span_start))
            .unwrap_or(0);
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.quasis,
            allocator: self.allocator,
            parent: AstNodes::TSTemplateLiteralType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn types<'c>(&'c self) -> &'c AstNode<'a, 'c, Vec<'a, TSType<'a>>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: AstNodes::TSTemplateLiteralType(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSAsExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.type_annotation.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSAsExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSAsExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSSatisfiesExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.type_annotation.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSSatisfiesExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSSatisfiesExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSTypeAssertion<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.inner.expression.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::TSTypeAssertion(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSTypeAssertion(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSImportEqualsDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id<'c>(&'c self) -> &'c AstNode<'a, 'c, BindingIdentifier<'a>> {
        let following_span_start = self.inner.module_reference.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::TSImportEqualsDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn module_reference<'c>(&'c self) -> &'c AstNode<'a, 'c, TSModuleReference<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.module_reference,
            allocator: self.allocator,
            parent: AstNodes::TSImportEqualsDeclaration(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSModuleReference<'a>> {
    #[inline]
    pub fn as_ast_nodes<'c>(&'c self) -> &'c AstNodes<'a, 'c> {
        let parent = self.parent;
        let node = match self.inner {
            TSModuleReference::ExternalModuleReference(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSExternalModuleReference(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSModuleReference::IdentifierReference(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::IdentifierReference(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
            TSModuleReference::QualifiedName(s) => {
                let allocator: &'c oxc_allocator::Allocator = self.allocator;
                AstNodes::TSQualifiedName(allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        };
        let allocator: &'c oxc_allocator::Allocator = self.allocator;
        allocator.alloc(node)
    }
}

impl<'a> AstNode<'a, '_, TSExternalModuleReference<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, StringLiteral<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSExternalModuleReference(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSNonNullExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSNonNullExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, Decorator<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::Decorator(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSExportAssignment<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSExportAssignment(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSNamespaceExportDeclaration<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn id<'c>(&'c self) -> &'c AstNode<'a, 'c, IdentifierName<'a>> {
        let following_span_start = 0;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: AstNodes::TSNamespaceExportDeclaration(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, TSInstantiationExpression<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn expression<'c>(&'c self) -> &'c AstNode<'a, 'c, Expression<'a>> {
        let following_span_start = self.inner.type_arguments.span().start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: AstNodes::TSInstantiationExpression(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn type_arguments<'c>(&'c self) -> &'c AstNode<'a, 'c, TSTypeParameterInstantiation<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: self.inner.type_arguments.as_ref(),
            allocator: self.allocator,
            parent: AstNodes::TSInstantiationExpression(self),
            following_span_start,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSDocNullableType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::JSDocNullableType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn postfix(&self) -> bool {
        self.inner.postfix
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSDocNonNullableType<'a>> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    #[inline]
    pub fn type_annotation<'c>(&'c self) -> &'c AstNode<'a, 'c, TSType<'a>> {
        let following_span_start = self.following_span_start;
        let allocator: &'c Allocator = self.allocator;
        allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: AstNodes::JSDocNonNullableType(self),
            following_span_start,
        })
    }

    #[inline]
    pub fn postfix(&self) -> bool {
        self.inner.postfix
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}

impl<'a> AstNode<'a, '_, JSDocUnknownType> {
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_leading_comments(self.span()).fmt(f);
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
            .fmt(f);
    }
}
