// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter2.rs`.

#![expect(clippy::match_same_arms, clippy::elidable_lifetime_names)]
use std::{mem::transmute, ops::Deref};

use oxc_allocator::{Allocator, Vec};
use oxc_ast::{AstKind, ast::*};
use oxc_span::GetSpan;

use crate::{
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        trivia::{format_leading_comments, format_trailing_comments},
    },
    parentheses::NeedsParentheses,
    write::FormatWrite,
};

fn transmute_self<'a, 'b, T>(s: &AstNode<'a, 'b, T>) -> &'a AstNode<'a, 'b, T> {
    /// * SAFETY: `s` is already allocated in Arena, so transmute from `&` to `&'a` is safe.
    unsafe {
        transmute(s)
    }
}

pub enum AstNodes<'a, 'b> {
    Dummy(),
    Program(&'a AstNode<'a, 'b, Program<'a>>),
    IdentifierName(&'a AstNode<'a, 'b, IdentifierName<'a>>),
    IdentifierReference(&'a AstNode<'a, 'b, IdentifierReference<'a>>),
    BindingIdentifier(&'a AstNode<'a, 'b, BindingIdentifier<'a>>),
    LabelIdentifier(&'a AstNode<'a, 'b, LabelIdentifier<'a>>),
    ThisExpression(&'a AstNode<'a, 'b, ThisExpression>),
    ArrayExpression(&'a AstNode<'a, 'b, ArrayExpression<'a>>),
    ArrayExpressionElement(&'a AstNode<'a, 'b, ArrayExpressionElement<'a>>),
    Elision(&'a AstNode<'a, 'b, Elision>),
    ObjectExpression(&'a AstNode<'a, 'b, ObjectExpression<'a>>),
    ObjectProperty(&'a AstNode<'a, 'b, ObjectProperty<'a>>),
    PropertyKey(&'a AstNode<'a, 'b, PropertyKey<'a>>),
    TemplateLiteral(&'a AstNode<'a, 'b, TemplateLiteral<'a>>),
    TaggedTemplateExpression(&'a AstNode<'a, 'b, TaggedTemplateExpression<'a>>),
    MemberExpression(&'a AstNode<'a, 'b, MemberExpression<'a>>),
    CallExpression(&'a AstNode<'a, 'b, CallExpression<'a>>),
    NewExpression(&'a AstNode<'a, 'b, NewExpression<'a>>),
    MetaProperty(&'a AstNode<'a, 'b, MetaProperty<'a>>),
    SpreadElement(&'a AstNode<'a, 'b, SpreadElement<'a>>),
    Argument(&'a AstNode<'a, 'b, Argument<'a>>),
    UpdateExpression(&'a AstNode<'a, 'b, UpdateExpression<'a>>),
    UnaryExpression(&'a AstNode<'a, 'b, UnaryExpression<'a>>),
    BinaryExpression(&'a AstNode<'a, 'b, BinaryExpression<'a>>),
    PrivateInExpression(&'a AstNode<'a, 'b, PrivateInExpression<'a>>),
    LogicalExpression(&'a AstNode<'a, 'b, LogicalExpression<'a>>),
    ConditionalExpression(&'a AstNode<'a, 'b, ConditionalExpression<'a>>),
    AssignmentExpression(&'a AstNode<'a, 'b, AssignmentExpression<'a>>),
    AssignmentTarget(&'a AstNode<'a, 'b, AssignmentTarget<'a>>),
    SimpleAssignmentTarget(&'a AstNode<'a, 'b, SimpleAssignmentTarget<'a>>),
    AssignmentTargetPattern(&'a AstNode<'a, 'b, AssignmentTargetPattern<'a>>),
    ArrayAssignmentTarget(&'a AstNode<'a, 'b, ArrayAssignmentTarget<'a>>),
    ObjectAssignmentTarget(&'a AstNode<'a, 'b, ObjectAssignmentTarget<'a>>),
    AssignmentTargetWithDefault(&'a AstNode<'a, 'b, AssignmentTargetWithDefault<'a>>),
    SequenceExpression(&'a AstNode<'a, 'b, SequenceExpression<'a>>),
    Super(&'a AstNode<'a, 'b, Super>),
    AwaitExpression(&'a AstNode<'a, 'b, AwaitExpression<'a>>),
    ChainExpression(&'a AstNode<'a, 'b, ChainExpression<'a>>),
    ParenthesizedExpression(&'a AstNode<'a, 'b, ParenthesizedExpression<'a>>),
    Directive(&'a AstNode<'a, 'b, Directive<'a>>),
    Hashbang(&'a AstNode<'a, 'b, Hashbang<'a>>),
    BlockStatement(&'a AstNode<'a, 'b, BlockStatement<'a>>),
    VariableDeclaration(&'a AstNode<'a, 'b, VariableDeclaration<'a>>),
    VariableDeclarator(&'a AstNode<'a, 'b, VariableDeclarator<'a>>),
    EmptyStatement(&'a AstNode<'a, 'b, EmptyStatement>),
    ExpressionStatement(&'a AstNode<'a, 'b, ExpressionStatement<'a>>),
    IfStatement(&'a AstNode<'a, 'b, IfStatement<'a>>),
    DoWhileStatement(&'a AstNode<'a, 'b, DoWhileStatement<'a>>),
    WhileStatement(&'a AstNode<'a, 'b, WhileStatement<'a>>),
    ForStatement(&'a AstNode<'a, 'b, ForStatement<'a>>),
    ForStatementInit(&'a AstNode<'a, 'b, ForStatementInit<'a>>),
    ForInStatement(&'a AstNode<'a, 'b, ForInStatement<'a>>),
    ForOfStatement(&'a AstNode<'a, 'b, ForOfStatement<'a>>),
    ContinueStatement(&'a AstNode<'a, 'b, ContinueStatement<'a>>),
    BreakStatement(&'a AstNode<'a, 'b, BreakStatement<'a>>),
    ReturnStatement(&'a AstNode<'a, 'b, ReturnStatement<'a>>),
    WithStatement(&'a AstNode<'a, 'b, WithStatement<'a>>),
    SwitchStatement(&'a AstNode<'a, 'b, SwitchStatement<'a>>),
    SwitchCase(&'a AstNode<'a, 'b, SwitchCase<'a>>),
    LabeledStatement(&'a AstNode<'a, 'b, LabeledStatement<'a>>),
    ThrowStatement(&'a AstNode<'a, 'b, ThrowStatement<'a>>),
    TryStatement(&'a AstNode<'a, 'b, TryStatement<'a>>),
    CatchClause(&'a AstNode<'a, 'b, CatchClause<'a>>),
    CatchParameter(&'a AstNode<'a, 'b, CatchParameter<'a>>),
    DebuggerStatement(&'a AstNode<'a, 'b, DebuggerStatement>),
    AssignmentPattern(&'a AstNode<'a, 'b, AssignmentPattern<'a>>),
    ObjectPattern(&'a AstNode<'a, 'b, ObjectPattern<'a>>),
    ArrayPattern(&'a AstNode<'a, 'b, ArrayPattern<'a>>),
    BindingRestElement(&'a AstNode<'a, 'b, BindingRestElement<'a>>),
    Function(&'a AstNode<'a, 'b, Function<'a>>),
    FormalParameters(&'a AstNode<'a, 'b, FormalParameters<'a>>),
    FormalParameter(&'a AstNode<'a, 'b, FormalParameter<'a>>),
    FunctionBody(&'a AstNode<'a, 'b, FunctionBody<'a>>),
    ArrowFunctionExpression(&'a AstNode<'a, 'b, ArrowFunctionExpression<'a>>),
    YieldExpression(&'a AstNode<'a, 'b, YieldExpression<'a>>),
    Class(&'a AstNode<'a, 'b, Class<'a>>),
    ClassBody(&'a AstNode<'a, 'b, ClassBody<'a>>),
    MethodDefinition(&'a AstNode<'a, 'b, MethodDefinition<'a>>),
    PropertyDefinition(&'a AstNode<'a, 'b, PropertyDefinition<'a>>),
    PrivateIdentifier(&'a AstNode<'a, 'b, PrivateIdentifier<'a>>),
    StaticBlock(&'a AstNode<'a, 'b, StaticBlock<'a>>),
    ModuleDeclaration(&'a AstNode<'a, 'b, ModuleDeclaration<'a>>),
    ImportExpression(&'a AstNode<'a, 'b, ImportExpression<'a>>),
    ImportDeclaration(&'a AstNode<'a, 'b, ImportDeclaration<'a>>),
    ImportSpecifier(&'a AstNode<'a, 'b, ImportSpecifier<'a>>),
    ImportDefaultSpecifier(&'a AstNode<'a, 'b, ImportDefaultSpecifier<'a>>),
    ImportNamespaceSpecifier(&'a AstNode<'a, 'b, ImportNamespaceSpecifier<'a>>),
    ExportNamedDeclaration(&'a AstNode<'a, 'b, ExportNamedDeclaration<'a>>),
    ExportDefaultDeclaration(&'a AstNode<'a, 'b, ExportDefaultDeclaration<'a>>),
    ExportAllDeclaration(&'a AstNode<'a, 'b, ExportAllDeclaration<'a>>),
    ExportSpecifier(&'a AstNode<'a, 'b, ExportSpecifier<'a>>),
    V8IntrinsicExpression(&'a AstNode<'a, 'b, V8IntrinsicExpression<'a>>),
    BooleanLiteral(&'a AstNode<'a, 'b, BooleanLiteral>),
    NullLiteral(&'a AstNode<'a, 'b, NullLiteral>),
    NumericLiteral(&'a AstNode<'a, 'b, NumericLiteral<'a>>),
    StringLiteral(&'a AstNode<'a, 'b, StringLiteral<'a>>),
    BigIntLiteral(&'a AstNode<'a, 'b, BigIntLiteral<'a>>),
    RegExpLiteral(&'a AstNode<'a, 'b, RegExpLiteral<'a>>),
    JSXElement(&'a AstNode<'a, 'b, JSXElement<'a>>),
    JSXOpeningElement(&'a AstNode<'a, 'b, JSXOpeningElement<'a>>),
    JSXClosingElement(&'a AstNode<'a, 'b, JSXClosingElement<'a>>),
    JSXFragment(&'a AstNode<'a, 'b, JSXFragment<'a>>),
    JSXElementName(&'a AstNode<'a, 'b, JSXElementName<'a>>),
    JSXNamespacedName(&'a AstNode<'a, 'b, JSXNamespacedName<'a>>),
    JSXMemberExpression(&'a AstNode<'a, 'b, JSXMemberExpression<'a>>),
    JSXMemberExpressionObject(&'a AstNode<'a, 'b, JSXMemberExpressionObject<'a>>),
    JSXExpressionContainer(&'a AstNode<'a, 'b, JSXExpressionContainer<'a>>),
    JSXAttributeItem(&'a AstNode<'a, 'b, JSXAttributeItem<'a>>),
    JSXSpreadAttribute(&'a AstNode<'a, 'b, JSXSpreadAttribute<'a>>),
    JSXIdentifier(&'a AstNode<'a, 'b, JSXIdentifier<'a>>),
    JSXText(&'a AstNode<'a, 'b, JSXText<'a>>),
    TSThisParameter(&'a AstNode<'a, 'b, TSThisParameter<'a>>),
    TSEnumDeclaration(&'a AstNode<'a, 'b, TSEnumDeclaration<'a>>),
    TSEnumBody(&'a AstNode<'a, 'b, TSEnumBody<'a>>),
    TSEnumMember(&'a AstNode<'a, 'b, TSEnumMember<'a>>),
    TSTypeAnnotation(&'a AstNode<'a, 'b, TSTypeAnnotation<'a>>),
    TSLiteralType(&'a AstNode<'a, 'b, TSLiteralType<'a>>),
    TSConditionalType(&'a AstNode<'a, 'b, TSConditionalType<'a>>),
    TSUnionType(&'a AstNode<'a, 'b, TSUnionType<'a>>),
    TSIntersectionType(&'a AstNode<'a, 'b, TSIntersectionType<'a>>),
    TSParenthesizedType(&'a AstNode<'a, 'b, TSParenthesizedType<'a>>),
    TSIndexedAccessType(&'a AstNode<'a, 'b, TSIndexedAccessType<'a>>),
    TSNamedTupleMember(&'a AstNode<'a, 'b, TSNamedTupleMember<'a>>),
    TSAnyKeyword(&'a AstNode<'a, 'b, TSAnyKeyword>),
    TSStringKeyword(&'a AstNode<'a, 'b, TSStringKeyword>),
    TSBooleanKeyword(&'a AstNode<'a, 'b, TSBooleanKeyword>),
    TSNumberKeyword(&'a AstNode<'a, 'b, TSNumberKeyword>),
    TSNeverKeyword(&'a AstNode<'a, 'b, TSNeverKeyword>),
    TSIntrinsicKeyword(&'a AstNode<'a, 'b, TSIntrinsicKeyword>),
    TSUnknownKeyword(&'a AstNode<'a, 'b, TSUnknownKeyword>),
    TSNullKeyword(&'a AstNode<'a, 'b, TSNullKeyword>),
    TSUndefinedKeyword(&'a AstNode<'a, 'b, TSUndefinedKeyword>),
    TSVoidKeyword(&'a AstNode<'a, 'b, TSVoidKeyword>),
    TSSymbolKeyword(&'a AstNode<'a, 'b, TSSymbolKeyword>),
    TSThisType(&'a AstNode<'a, 'b, TSThisType>),
    TSObjectKeyword(&'a AstNode<'a, 'b, TSObjectKeyword>),
    TSBigIntKeyword(&'a AstNode<'a, 'b, TSBigIntKeyword>),
    TSTypeReference(&'a AstNode<'a, 'b, TSTypeReference<'a>>),
    TSTypeName(&'a AstNode<'a, 'b, TSTypeName<'a>>),
    TSQualifiedName(&'a AstNode<'a, 'b, TSQualifiedName<'a>>),
    TSTypeParameterInstantiation(&'a AstNode<'a, 'b, TSTypeParameterInstantiation<'a>>),
    TSTypeParameter(&'a AstNode<'a, 'b, TSTypeParameter<'a>>),
    TSTypeParameterDeclaration(&'a AstNode<'a, 'b, TSTypeParameterDeclaration<'a>>),
    TSTypeAliasDeclaration(&'a AstNode<'a, 'b, TSTypeAliasDeclaration<'a>>),
    TSClassImplements(&'a AstNode<'a, 'b, TSClassImplements<'a>>),
    TSInterfaceDeclaration(&'a AstNode<'a, 'b, TSInterfaceDeclaration<'a>>),
    TSPropertySignature(&'a AstNode<'a, 'b, TSPropertySignature<'a>>),
    TSMethodSignature(&'a AstNode<'a, 'b, TSMethodSignature<'a>>),
    TSConstructSignatureDeclaration(&'a AstNode<'a, 'b, TSConstructSignatureDeclaration<'a>>),
    TSInterfaceHeritage(&'a AstNode<'a, 'b, TSInterfaceHeritage<'a>>),
    TSModuleDeclaration(&'a AstNode<'a, 'b, TSModuleDeclaration<'a>>),
    TSModuleBlock(&'a AstNode<'a, 'b, TSModuleBlock<'a>>),
    TSTypeLiteral(&'a AstNode<'a, 'b, TSTypeLiteral<'a>>),
    TSInferType(&'a AstNode<'a, 'b, TSInferType<'a>>),
    TSTypeQuery(&'a AstNode<'a, 'b, TSTypeQuery<'a>>),
    TSImportType(&'a AstNode<'a, 'b, TSImportType<'a>>),
    TSMappedType(&'a AstNode<'a, 'b, TSMappedType<'a>>),
    TSTemplateLiteralType(&'a AstNode<'a, 'b, TSTemplateLiteralType<'a>>),
    TSAsExpression(&'a AstNode<'a, 'b, TSAsExpression<'a>>),
    TSSatisfiesExpression(&'a AstNode<'a, 'b, TSSatisfiesExpression<'a>>),
    TSTypeAssertion(&'a AstNode<'a, 'b, TSTypeAssertion<'a>>),
    TSImportEqualsDeclaration(&'a AstNode<'a, 'b, TSImportEqualsDeclaration<'a>>),
    TSModuleReference(&'a AstNode<'a, 'b, TSModuleReference<'a>>),
    TSExternalModuleReference(&'a AstNode<'a, 'b, TSExternalModuleReference<'a>>),
    TSNonNullExpression(&'a AstNode<'a, 'b, TSNonNullExpression<'a>>),
    Decorator(&'a AstNode<'a, 'b, Decorator<'a>>),
    TSExportAssignment(&'a AstNode<'a, 'b, TSExportAssignment<'a>>),
    TSInstantiationExpression(&'a AstNode<'a, 'b, TSInstantiationExpression<'a>>),
}
impl<'a, 'b> AstNodes<'a, 'b> {
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
            Self::ArrayExpressionElement(n) => n.span(),
            Self::Elision(n) => n.span(),
            Self::ObjectExpression(n) => n.span(),
            Self::ObjectProperty(n) => n.span(),
            Self::PropertyKey(n) => n.span(),
            Self::TemplateLiteral(n) => n.span(),
            Self::TaggedTemplateExpression(n) => n.span(),
            Self::MemberExpression(n) => n.span(),
            Self::CallExpression(n) => n.span(),
            Self::NewExpression(n) => n.span(),
            Self::MetaProperty(n) => n.span(),
            Self::SpreadElement(n) => n.span(),
            Self::Argument(n) => n.span(),
            Self::UpdateExpression(n) => n.span(),
            Self::UnaryExpression(n) => n.span(),
            Self::BinaryExpression(n) => n.span(),
            Self::PrivateInExpression(n) => n.span(),
            Self::LogicalExpression(n) => n.span(),
            Self::ConditionalExpression(n) => n.span(),
            Self::AssignmentExpression(n) => n.span(),
            Self::AssignmentTarget(n) => n.span(),
            Self::SimpleAssignmentTarget(n) => n.span(),
            Self::AssignmentTargetPattern(n) => n.span(),
            Self::ArrayAssignmentTarget(n) => n.span(),
            Self::ObjectAssignmentTarget(n) => n.span(),
            Self::AssignmentTargetWithDefault(n) => n.span(),
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
            Self::ForStatementInit(n) => n.span(),
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
            Self::ArrayPattern(n) => n.span(),
            Self::BindingRestElement(n) => n.span(),
            Self::Function(n) => n.span(),
            Self::FormalParameters(n) => n.span(),
            Self::FormalParameter(n) => n.span(),
            Self::FunctionBody(n) => n.span(),
            Self::ArrowFunctionExpression(n) => n.span(),
            Self::YieldExpression(n) => n.span(),
            Self::Class(n) => n.span(),
            Self::ClassBody(n) => n.span(),
            Self::MethodDefinition(n) => n.span(),
            Self::PropertyDefinition(n) => n.span(),
            Self::PrivateIdentifier(n) => n.span(),
            Self::StaticBlock(n) => n.span(),
            Self::ModuleDeclaration(n) => n.span(),
            Self::ImportExpression(n) => n.span(),
            Self::ImportDeclaration(n) => n.span(),
            Self::ImportSpecifier(n) => n.span(),
            Self::ImportDefaultSpecifier(n) => n.span(),
            Self::ImportNamespaceSpecifier(n) => n.span(),
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
            Self::JSXElementName(n) => n.span(),
            Self::JSXNamespacedName(n) => n.span(),
            Self::JSXMemberExpression(n) => n.span(),
            Self::JSXMemberExpressionObject(n) => n.span(),
            Self::JSXExpressionContainer(n) => n.span(),
            Self::JSXAttributeItem(n) => n.span(),
            Self::JSXSpreadAttribute(n) => n.span(),
            Self::JSXIdentifier(n) => n.span(),
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
            Self::TSIndexedAccessType(n) => n.span(),
            Self::TSNamedTupleMember(n) => n.span(),
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
            Self::TSTypeName(n) => n.span(),
            Self::TSQualifiedName(n) => n.span(),
            Self::TSTypeParameterInstantiation(n) => n.span(),
            Self::TSTypeParameter(n) => n.span(),
            Self::TSTypeParameterDeclaration(n) => n.span(),
            Self::TSTypeAliasDeclaration(n) => n.span(),
            Self::TSClassImplements(n) => n.span(),
            Self::TSInterfaceDeclaration(n) => n.span(),
            Self::TSPropertySignature(n) => n.span(),
            Self::TSMethodSignature(n) => n.span(),
            Self::TSConstructSignatureDeclaration(n) => n.span(),
            Self::TSInterfaceHeritage(n) => n.span(),
            Self::TSModuleDeclaration(n) => n.span(),
            Self::TSModuleBlock(n) => n.span(),
            Self::TSTypeLiteral(n) => n.span(),
            Self::TSInferType(n) => n.span(),
            Self::TSTypeQuery(n) => n.span(),
            Self::TSImportType(n) => n.span(),
            Self::TSMappedType(n) => n.span(),
            Self::TSTemplateLiteralType(n) => n.span(),
            Self::TSAsExpression(n) => n.span(),
            Self::TSSatisfiesExpression(n) => n.span(),
            Self::TSTypeAssertion(n) => n.span(),
            Self::TSImportEqualsDeclaration(n) => n.span(),
            Self::TSModuleReference(n) => n.span(),
            Self::TSExternalModuleReference(n) => n.span(),
            Self::TSNonNullExpression(n) => n.span(),
            Self::Decorator(n) => n.span(),
            Self::TSExportAssignment(n) => n.span(),
            Self::TSInstantiationExpression(n) => n.span(),
        }
    }
    pub fn parent(&self) -> &'a Self {
        match self {
            Self::Dummy() => panic!("Should never be called on a dummy node"),
            Self::Program(n) => n.parent,
            Self::IdentifierName(n) => n.parent,
            Self::IdentifierReference(n) => n.parent,
            Self::BindingIdentifier(n) => n.parent,
            Self::LabelIdentifier(n) => n.parent,
            Self::ThisExpression(n) => n.parent,
            Self::ArrayExpression(n) => n.parent,
            Self::ArrayExpressionElement(n) => n.parent,
            Self::Elision(n) => n.parent,
            Self::ObjectExpression(n) => n.parent,
            Self::ObjectProperty(n) => n.parent,
            Self::PropertyKey(n) => n.parent,
            Self::TemplateLiteral(n) => n.parent,
            Self::TaggedTemplateExpression(n) => n.parent,
            Self::MemberExpression(n) => n.parent,
            Self::CallExpression(n) => n.parent,
            Self::NewExpression(n) => n.parent,
            Self::MetaProperty(n) => n.parent,
            Self::SpreadElement(n) => n.parent,
            Self::Argument(n) => n.parent,
            Self::UpdateExpression(n) => n.parent,
            Self::UnaryExpression(n) => n.parent,
            Self::BinaryExpression(n) => n.parent,
            Self::PrivateInExpression(n) => n.parent,
            Self::LogicalExpression(n) => n.parent,
            Self::ConditionalExpression(n) => n.parent,
            Self::AssignmentExpression(n) => n.parent,
            Self::AssignmentTarget(n) => n.parent,
            Self::SimpleAssignmentTarget(n) => n.parent,
            Self::AssignmentTargetPattern(n) => n.parent,
            Self::ArrayAssignmentTarget(n) => n.parent,
            Self::ObjectAssignmentTarget(n) => n.parent,
            Self::AssignmentTargetWithDefault(n) => n.parent,
            Self::SequenceExpression(n) => n.parent,
            Self::Super(n) => n.parent,
            Self::AwaitExpression(n) => n.parent,
            Self::ChainExpression(n) => n.parent,
            Self::ParenthesizedExpression(n) => n.parent,
            Self::Directive(n) => n.parent,
            Self::Hashbang(n) => n.parent,
            Self::BlockStatement(n) => n.parent,
            Self::VariableDeclaration(n) => n.parent,
            Self::VariableDeclarator(n) => n.parent,
            Self::EmptyStatement(n) => n.parent,
            Self::ExpressionStatement(n) => n.parent,
            Self::IfStatement(n) => n.parent,
            Self::DoWhileStatement(n) => n.parent,
            Self::WhileStatement(n) => n.parent,
            Self::ForStatement(n) => n.parent,
            Self::ForStatementInit(n) => n.parent,
            Self::ForInStatement(n) => n.parent,
            Self::ForOfStatement(n) => n.parent,
            Self::ContinueStatement(n) => n.parent,
            Self::BreakStatement(n) => n.parent,
            Self::ReturnStatement(n) => n.parent,
            Self::WithStatement(n) => n.parent,
            Self::SwitchStatement(n) => n.parent,
            Self::SwitchCase(n) => n.parent,
            Self::LabeledStatement(n) => n.parent,
            Self::ThrowStatement(n) => n.parent,
            Self::TryStatement(n) => n.parent,
            Self::CatchClause(n) => n.parent,
            Self::CatchParameter(n) => n.parent,
            Self::DebuggerStatement(n) => n.parent,
            Self::AssignmentPattern(n) => n.parent,
            Self::ObjectPattern(n) => n.parent,
            Self::ArrayPattern(n) => n.parent,
            Self::BindingRestElement(n) => n.parent,
            Self::Function(n) => n.parent,
            Self::FormalParameters(n) => n.parent,
            Self::FormalParameter(n) => n.parent,
            Self::FunctionBody(n) => n.parent,
            Self::ArrowFunctionExpression(n) => n.parent,
            Self::YieldExpression(n) => n.parent,
            Self::Class(n) => n.parent,
            Self::ClassBody(n) => n.parent,
            Self::MethodDefinition(n) => n.parent,
            Self::PropertyDefinition(n) => n.parent,
            Self::PrivateIdentifier(n) => n.parent,
            Self::StaticBlock(n) => n.parent,
            Self::ModuleDeclaration(n) => n.parent,
            Self::ImportExpression(n) => n.parent,
            Self::ImportDeclaration(n) => n.parent,
            Self::ImportSpecifier(n) => n.parent,
            Self::ImportDefaultSpecifier(n) => n.parent,
            Self::ImportNamespaceSpecifier(n) => n.parent,
            Self::ExportNamedDeclaration(n) => n.parent,
            Self::ExportDefaultDeclaration(n) => n.parent,
            Self::ExportAllDeclaration(n) => n.parent,
            Self::ExportSpecifier(n) => n.parent,
            Self::V8IntrinsicExpression(n) => n.parent,
            Self::BooleanLiteral(n) => n.parent,
            Self::NullLiteral(n) => n.parent,
            Self::NumericLiteral(n) => n.parent,
            Self::StringLiteral(n) => n.parent,
            Self::BigIntLiteral(n) => n.parent,
            Self::RegExpLiteral(n) => n.parent,
            Self::JSXElement(n) => n.parent,
            Self::JSXOpeningElement(n) => n.parent,
            Self::JSXClosingElement(n) => n.parent,
            Self::JSXFragment(n) => n.parent,
            Self::JSXElementName(n) => n.parent,
            Self::JSXNamespacedName(n) => n.parent,
            Self::JSXMemberExpression(n) => n.parent,
            Self::JSXMemberExpressionObject(n) => n.parent,
            Self::JSXExpressionContainer(n) => n.parent,
            Self::JSXAttributeItem(n) => n.parent,
            Self::JSXSpreadAttribute(n) => n.parent,
            Self::JSXIdentifier(n) => n.parent,
            Self::JSXText(n) => n.parent,
            Self::TSThisParameter(n) => n.parent,
            Self::TSEnumDeclaration(n) => n.parent,
            Self::TSEnumBody(n) => n.parent,
            Self::TSEnumMember(n) => n.parent,
            Self::TSTypeAnnotation(n) => n.parent,
            Self::TSLiteralType(n) => n.parent,
            Self::TSConditionalType(n) => n.parent,
            Self::TSUnionType(n) => n.parent,
            Self::TSIntersectionType(n) => n.parent,
            Self::TSParenthesizedType(n) => n.parent,
            Self::TSIndexedAccessType(n) => n.parent,
            Self::TSNamedTupleMember(n) => n.parent,
            Self::TSAnyKeyword(n) => n.parent,
            Self::TSStringKeyword(n) => n.parent,
            Self::TSBooleanKeyword(n) => n.parent,
            Self::TSNumberKeyword(n) => n.parent,
            Self::TSNeverKeyword(n) => n.parent,
            Self::TSIntrinsicKeyword(n) => n.parent,
            Self::TSUnknownKeyword(n) => n.parent,
            Self::TSNullKeyword(n) => n.parent,
            Self::TSUndefinedKeyword(n) => n.parent,
            Self::TSVoidKeyword(n) => n.parent,
            Self::TSSymbolKeyword(n) => n.parent,
            Self::TSThisType(n) => n.parent,
            Self::TSObjectKeyword(n) => n.parent,
            Self::TSBigIntKeyword(n) => n.parent,
            Self::TSTypeReference(n) => n.parent,
            Self::TSTypeName(n) => n.parent,
            Self::TSQualifiedName(n) => n.parent,
            Self::TSTypeParameterInstantiation(n) => n.parent,
            Self::TSTypeParameter(n) => n.parent,
            Self::TSTypeParameterDeclaration(n) => n.parent,
            Self::TSTypeAliasDeclaration(n) => n.parent,
            Self::TSClassImplements(n) => n.parent,
            Self::TSInterfaceDeclaration(n) => n.parent,
            Self::TSPropertySignature(n) => n.parent,
            Self::TSMethodSignature(n) => n.parent,
            Self::TSConstructSignatureDeclaration(n) => n.parent,
            Self::TSInterfaceHeritage(n) => n.parent,
            Self::TSModuleDeclaration(n) => n.parent,
            Self::TSModuleBlock(n) => n.parent,
            Self::TSTypeLiteral(n) => n.parent,
            Self::TSInferType(n) => n.parent,
            Self::TSTypeQuery(n) => n.parent,
            Self::TSImportType(n) => n.parent,
            Self::TSMappedType(n) => n.parent,
            Self::TSTemplateLiteralType(n) => n.parent,
            Self::TSAsExpression(n) => n.parent,
            Self::TSSatisfiesExpression(n) => n.parent,
            Self::TSTypeAssertion(n) => n.parent,
            Self::TSImportEqualsDeclaration(n) => n.parent,
            Self::TSModuleReference(n) => n.parent,
            Self::TSExternalModuleReference(n) => n.parent,
            Self::TSNonNullExpression(n) => n.parent,
            Self::Decorator(n) => n.parent,
            Self::TSExportAssignment(n) => n.parent,
            Self::TSInstantiationExpression(n) => n.parent,
        }
    }
}

pub struct AstNode<'a, 'b, T> {
    inner: &'b T,
    pub parent: &'a AstNodes<'a, 'b>,
    allocator: &'a Allocator,
}

impl<'a, 'b, T> Deref for AstNode<'a, 'b, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}
impl<'a, 'b, T> AsRef<T> for AstNode<'a, 'b, T> {
    fn as_ref(&self) -> &T {
        self.inner
    }
}

impl<'a, 'b> AstNode<'a, 'b, Program<'a>> {
    pub fn new(
        inner: &'b Program<'a>,
        parent: &'a AstNodes<'a, 'b>,
        allocator: &'a Allocator,
    ) -> Self {
        AstNode { inner, parent, allocator }
    }
}

impl<'a, 'b, T> AstNode<'a, 'b, Option<T>> {
    pub fn as_ref(&self) -> Option<&'a AstNode<'a, 'b, T>> {
        self.allocator
            .alloc(self.inner.as_ref().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
            }))
            .as_ref()
    }
}

impl<'a, 'b, T> AstNode<'a, 'b, Vec<'a, T>> {
    pub fn iter(&self) -> AstNodeIterator<'a, 'b, T> {
        AstNodeIterator { inner: self.inner.iter(), parent: self.parent, allocator: self.allocator }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, 'b, T>> {
        self.allocator
            .alloc(self.inner.first().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, 'b, T>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
            }))
            .as_ref()
    }
}

pub struct AstNodeIterator<'a, 'b, T> {
    inner: std::slice::Iter<'b, T>,
    parent: &'a AstNodes<'a, 'b>,
    allocator: &'a Allocator,
}

impl<'a, 'b, T> Iterator for AstNodeIterator<'a, 'b, T> {
    type Item = &'a AstNode<'a, 'b, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        self.allocator
            .alloc(self.inner.next().map(|inner| AstNode { parent: self.parent, inner, allocator }))
            .as_ref()
    }
}

impl<'a, 'b, T> IntoIterator for &AstNode<'a, 'b, Vec<'a, T>> {
    type Item = &'a AstNode<'a, 'b, T>;
    type IntoIter = AstNodeIterator<'a, 'b, T>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<T> {
            inner: self.inner.iter(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a, 'b> AstNode<'a, 'b, Program<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn source_type(&self) -> SourceType {
        self.inner.source_type
    }

    pub fn source_text(&self) -> &'a str {
        self.inner.source_text
    }

    pub fn comments(&self) -> &AstNode<'a, 'b, Vec<'a, Comment>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.comments,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Program(transmute_self(self))),
        })
    }

    pub fn hashbang(&self) -> Option<&AstNode<'a, 'b, Hashbang<'a>>> {
        self.allocator
            .alloc(self.inner.hashbang.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Program(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn directives(&self) -> &AstNode<'a, 'b, Vec<'a, Directive<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Program(transmute_self(self))),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Vec<'a, Statement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Program(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, Expression<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            Expression::BooleanLiteral(s) => {
                AstNodes::BooleanLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::NullLiteral(s) => AstNodes::NullLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            Expression::NumericLiteral(s) => {
                AstNodes::NumericLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::BigIntLiteral(s) => {
                AstNodes::BigIntLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::RegExpLiteral(s) => {
                AstNodes::RegExpLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::TemplateLiteral(s) => {
                AstNodes::TemplateLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::Identifier(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::MetaProperty(s) => AstNodes::MetaProperty(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            Expression::Super(s) => AstNodes::Super(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            Expression::ArrayExpression(s) => {
                AstNodes::ArrayExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::ArrowFunctionExpression(s) => {
                AstNodes::ArrowFunctionExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::AssignmentExpression(s) => {
                AstNodes::AssignmentExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::AwaitExpression(s) => {
                AstNodes::AwaitExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::BinaryExpression(s) => {
                AstNodes::BinaryExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::CallExpression(s) => {
                AstNodes::CallExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::ChainExpression(s) => {
                AstNodes::ChainExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::ClassExpression(s) => AstNodes::Class(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            Expression::ConditionalExpression(s) => {
                AstNodes::ConditionalExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::FunctionExpression(s) => {
                AstNodes::Function(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::ImportExpression(s) => {
                AstNodes::ImportExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::LogicalExpression(s) => {
                AstNodes::LogicalExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::NewExpression(s) => {
                AstNodes::NewExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::ObjectExpression(s) => {
                AstNodes::ObjectExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::ParenthesizedExpression(s) => {
                AstNodes::ParenthesizedExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::SequenceExpression(s) => {
                AstNodes::SequenceExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::TaggedTemplateExpression(s) => {
                AstNodes::TaggedTemplateExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::ThisExpression(s) => {
                AstNodes::ThisExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::UnaryExpression(s) => {
                AstNodes::UnaryExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::UpdateExpression(s) => {
                AstNodes::UpdateExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::YieldExpression(s) => {
                AstNodes::YieldExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::PrivateInExpression(s) => {
                AstNodes::PrivateInExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::JSXElement(s) => AstNodes::JSXElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            Expression::JSXFragment(s) => AstNodes::JSXFragment(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            Expression::TSAsExpression(s) => {
                AstNodes::TSAsExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::TSSatisfiesExpression(s) => {
                AstNodes::TSSatisfiesExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::TSTypeAssertion(s) => {
                AstNodes::TSTypeAssertion(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::TSInstantiationExpression(s) => {
                AstNodes::TSInstantiationExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Expression::V8IntrinsicExpression(s) => {
                AstNodes::V8IntrinsicExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            it @ match_member_expression!(Expression) => {
                AstNodes::MemberExpression(self.allocator.alloc(AstNode {
                    inner: it.to_member_expression(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, Expression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            Expression::BooleanLiteral(s) => {
                AstNode::<'a, 'b, BooleanLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::NullLiteral(s) => {
                AstNode::<'a, 'b, NullLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::NumericLiteral(s) => {
                AstNode::<'a, 'b, NumericLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::BigIntLiteral(s) => {
                AstNode::<'a, 'b, BigIntLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::RegExpLiteral(s) => {
                AstNode::<'a, 'b, RegExpLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::StringLiteral(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::TemplateLiteral(s) => {
                AstNode::<'a, 'b, TemplateLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::Identifier(s) => AstNode::<'a, 'b, IdentifierReference> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::MetaProperty(s) => {
                AstNode::<'a, 'b, MetaProperty> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::Super(s) => {
                AstNode::<'a, 'b, Super> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            Expression::ArrayExpression(s) => {
                AstNode::<'a, 'b, ArrayExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::ArrowFunctionExpression(s) => AstNode::<'a, 'b, ArrowFunctionExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::AssignmentExpression(s) => AstNode::<'a, 'b, AssignmentExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::AwaitExpression(s) => {
                AstNode::<'a, 'b, AwaitExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::BinaryExpression(s) => {
                AstNode::<'a, 'b, BinaryExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::CallExpression(s) => {
                AstNode::<'a, 'b, CallExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::ChainExpression(s) => {
                AstNode::<'a, 'b, ChainExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::ClassExpression(s) => {
                AstNode::<'a, 'b, Class> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            Expression::ConditionalExpression(s) => AstNode::<'a, 'b, ConditionalExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::FunctionExpression(s) => {
                AstNode::<'a, 'b, Function> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            Expression::ImportExpression(s) => {
                AstNode::<'a, 'b, ImportExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::LogicalExpression(s) => {
                AstNode::<'a, 'b, LogicalExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::NewExpression(s) => {
                AstNode::<'a, 'b, NewExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::ObjectExpression(s) => {
                AstNode::<'a, 'b, ObjectExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::ParenthesizedExpression(s) => AstNode::<'a, 'b, ParenthesizedExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::SequenceExpression(s) => AstNode::<'a, 'b, SequenceExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::TaggedTemplateExpression(s) => {
                AstNode::<'a, 'b, TaggedTemplateExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            Expression::ThisExpression(s) => {
                AstNode::<'a, 'b, ThisExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::UnaryExpression(s) => {
                AstNode::<'a, 'b, UnaryExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::UpdateExpression(s) => {
                AstNode::<'a, 'b, UpdateExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::YieldExpression(s) => {
                AstNode::<'a, 'b, YieldExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::PrivateInExpression(s) => AstNode::<'a, 'b, PrivateInExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::JSXElement(s) => {
                AstNode::<'a, 'b, JSXElement> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            Expression::JSXFragment(s) => {
                AstNode::<'a, 'b, JSXFragment> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::TSAsExpression(s) => {
                AstNode::<'a, 'b, TSAsExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::TSSatisfiesExpression(s) => AstNode::<'a, 'b, TSSatisfiesExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::TSTypeAssertion(s) => {
                AstNode::<'a, 'b, TSTypeAssertion> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Expression::TSNonNullExpression(s) => AstNode::<'a, 'b, TSNonNullExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Expression::TSInstantiationExpression(s) => {
                AstNode::<'a, 'b, TSInstantiationExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            Expression::V8IntrinsicExpression(s) => AstNode::<'a, 'b, V8IntrinsicExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            it @ match_member_expression!(Expression) => AstNode::<'a, 'b, MemberExpression> {
                inner: it.to_member_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, Expression<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, IdentifierName<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}
impl<'a, 'b> AstNode<'a, 'b, IdentifierReference<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}
impl<'a, 'b> AstNode<'a, 'b, BindingIdentifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}
impl<'a, 'b> AstNode<'a, 'b, LabelIdentifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}
impl<'a, 'b> AstNode<'a, 'b, ThisExpression> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, ArrayExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn elements(&self) -> &AstNode<'a, 'b, Vec<'a, ArrayExpressionElement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrayExpression(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, ArrayExpressionElement<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.allocator.alloc(AstNodes::ArrayExpressionElement(transmute_self(self)));
        let node = match self.inner {
            ArrayExpressionElement::SpreadElement(s) => {
                AstNodes::SpreadElement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            ArrayExpressionElement::Elision(s) => {
                AstNodes::Elision(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }))
            }
            it @ match_expression!(ArrayExpressionElement) => {
                return self
                    .allocator
                    .alloc(AstNode { inner: it.to_expression(), parent, allocator: self.allocator })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ArrayExpressionElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::ArrayExpressionElement(transmute_self(self)));
        match self.inner {
            ArrayExpressionElement::SpreadElement(s) => {
                AstNode::<'a, 'b, SpreadElement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ArrayExpressionElement::Elision(s) => {
                AstNode::<'a, 'b, Elision> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            it @ match_expression!(ArrayExpressionElement) => AstNode::<'a, 'b, Expression> {
                inner: it.to_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ArrayExpressionElement<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, Elision> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, ObjectExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn properties(&self) -> &AstNode<'a, 'b, Vec<'a, ObjectPropertyKind<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectExpression(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, ObjectPropertyKind<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            ObjectPropertyKind::ObjectProperty(s) => {
                AstNodes::ObjectProperty(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            ObjectPropertyKind::SpreadProperty(s) => {
                AstNodes::SpreadElement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ObjectPropertyKind<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ObjectPropertyKind::ObjectProperty(s) => {
                AstNode::<'a, 'b, ObjectProperty> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ObjectPropertyKind::SpreadProperty(s) => {
                AstNode::<'a, 'b, SpreadElement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ObjectPropertyKind<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ObjectProperty<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn kind(&self) -> PropertyKind {
        self.inner.kind
    }

    pub fn key(&self) -> &AstNode<'a, 'b, PropertyKey<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectProperty(transmute_self(self))),
        })
    }

    pub fn value(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectProperty(transmute_self(self))),
        })
    }

    pub fn method(&self) -> bool {
        self.inner.method
    }

    pub fn shorthand(&self) -> bool {
        self.inner.shorthand
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }
}

impl<'a, 'b> AstNode<'a, 'b, PropertyKey<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.allocator.alloc(AstNodes::PropertyKey(transmute_self(self)));
        let node = match self.inner {
            PropertyKey::StaticIdentifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            PropertyKey::PrivateIdentifier(s) => {
                AstNodes::PrivateIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            it @ match_expression!(PropertyKey) => {
                return self
                    .allocator
                    .alloc(AstNode { inner: it.to_expression(), parent, allocator: self.allocator })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, PropertyKey<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::PropertyKey(transmute_self(self)));
        match self.inner {
            PropertyKey::StaticIdentifier(s) => {
                AstNode::<'a, 'b, IdentifierName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            PropertyKey::PrivateIdentifier(s) => {
                AstNode::<'a, 'b, PrivateIdentifier> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            it @ match_expression!(PropertyKey) => AstNode::<'a, 'b, Expression> {
                inner: it.to_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, PropertyKey<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TemplateLiteral<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn quasis(&self) -> &AstNode<'a, 'b, Vec<'a, TemplateElement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasis,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TemplateLiteral(transmute_self(self))),
        })
    }

    pub fn expressions(&self) -> &AstNode<'a, 'b, Vec<'a, Expression<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expressions,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TemplateLiteral(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TaggedTemplateExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn tag(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.tag,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TaggedTemplateExpression(transmute_self(self))),
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation<'a>>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::TaggedTemplateExpression(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn quasi(&self) -> &AstNode<'a, 'b, TemplateLiteral<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasi,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TaggedTemplateExpression(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TemplateElement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn value(&self) -> &TemplateElementValue<'a> {
        &self.inner.value
    }

    pub fn tail(&self) -> bool {
        self.inner.tail
    }

    pub fn lone_surrogates(&self) -> bool {
        self.inner.lone_surrogates
    }
}

impl<'a, 'b> AstNode<'a, 'b, MemberExpression<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.allocator.alloc(AstNodes::MemberExpression(transmute_self(self)));
        let node = match self.inner {
            MemberExpression::ComputedMemberExpression(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            MemberExpression::StaticMemberExpression(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            MemberExpression::PrivateFieldExpression(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, MemberExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::MemberExpression(transmute_self(self)));
        match self.inner {
            MemberExpression::ComputedMemberExpression(s) => {
                AstNode::<'a, 'b, ComputedMemberExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            MemberExpression::StaticMemberExpression(s) => {
                AstNode::<'a, 'b, StaticMemberExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            MemberExpression::PrivateFieldExpression(s) => {
                AstNode::<'a, 'b, PrivateFieldExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, MemberExpression<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ComputedMemberExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn object(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }
}
impl<'a, 'b> AstNode<'a, 'b, StaticMemberExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn object(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn property(&self) -> &AstNode<'a, 'b, IdentifierName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }
}
impl<'a, 'b> AstNode<'a, 'b, PrivateFieldExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn object(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn field(&self) -> &AstNode<'a, 'b, PrivateIdentifier<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.field,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }
}
impl<'a, 'b> AstNode<'a, 'b, CallExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn callee(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.callee,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::CallExpression(transmute_self(self))),
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation<'a>>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::CallExpression(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn arguments(&self) -> &AstNode<'a, 'b, Vec<'a, Argument<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::CallExpression(transmute_self(self))),
        })
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn pure(&self) -> bool {
        self.inner.pure
    }
}
impl<'a, 'b> AstNode<'a, 'b, NewExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn callee(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.callee,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::NewExpression(transmute_self(self))),
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation<'a>>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::NewExpression(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn arguments(&self) -> &AstNode<'a, 'b, Vec<'a, Argument<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::NewExpression(transmute_self(self))),
        })
    }

    pub fn pure(&self) -> bool {
        self.inner.pure
    }
}
impl<'a, 'b> AstNode<'a, 'b, MetaProperty<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn meta(&self) -> &AstNode<'a, 'b, IdentifierName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.meta,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MetaProperty(transmute_self(self))),
        })
    }

    pub fn property(&self) -> &AstNode<'a, 'b, IdentifierName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MetaProperty(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, SpreadElement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SpreadElement(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, Argument<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.allocator.alloc(AstNodes::Argument(transmute_self(self)));
        let node = match self.inner {
            Argument::SpreadElement(s) => AstNodes::SpreadElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            it @ match_expression!(Argument) => {
                return self
                    .allocator
                    .alloc(AstNode { inner: it.to_expression(), parent, allocator: self.allocator })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, Argument<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::Argument(transmute_self(self)));
        match self.inner {
            Argument::SpreadElement(s) => {
                AstNode::<'a, 'b, SpreadElement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            it @ match_expression!(Argument) => AstNode::<'a, 'b, Expression> {
                inner: it.to_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, Argument<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, UpdateExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn operator(&self) -> UpdateOperator {
        self.inner.operator
    }

    pub fn prefix(&self) -> bool {
        self.inner.prefix
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, SimpleAssignmentTarget<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::UpdateExpression(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, UnaryExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn operator(&self) -> UnaryOperator {
        self.inner.operator
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::UnaryExpression(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, BinaryExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn left(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BinaryExpression(transmute_self(self))),
        })
    }

    pub fn operator(&self) -> BinaryOperator {
        self.inner.operator
    }

    pub fn right(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BinaryExpression(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, PrivateInExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn left(&self) -> &AstNode<'a, 'b, PrivateIdentifier<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PrivateInExpression(transmute_self(self))),
        })
    }

    pub fn right(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PrivateInExpression(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, LogicalExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn left(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LogicalExpression(transmute_self(self))),
        })
    }

    pub fn operator(&self) -> LogicalOperator {
        self.inner.operator
    }

    pub fn right(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LogicalExpression(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ConditionalExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn test(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ConditionalExpression(transmute_self(self))),
        })
    }

    pub fn consequent(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ConditionalExpression(transmute_self(self))),
        })
    }

    pub fn alternate(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.alternate,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ConditionalExpression(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, AssignmentExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn operator(&self) -> AssignmentOperator {
        self.inner.operator
    }

    pub fn left(&self) -> &AstNode<'a, 'b, AssignmentTarget<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentExpression(transmute_self(self))),
        })
    }

    pub fn right(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentExpression(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, AssignmentTarget<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.allocator.alloc(AstNodes::AssignmentTarget(transmute_self(self)));
        let node = match self.inner {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                AstNodes::SimpleAssignmentTarget(self.allocator.alloc(AstNode {
                    inner: it.to_simple_assignment_target(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                AstNodes::AssignmentTargetPattern(self.allocator.alloc(AstNode {
                    inner: it.to_assignment_target_pattern(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, AssignmentTarget<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::AssignmentTarget(transmute_self(self)));
        match self.inner {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                AstNode::<'a, 'b, SimpleAssignmentTarget> {
                    inner: it.to_simple_assignment_target(),
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                AstNode::<'a, 'b, AssignmentTargetPattern> {
                    inner: it.to_assignment_target_pattern(),
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, AssignmentTarget<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a, 'b> AstNode<'a, 'b, SimpleAssignmentTarget<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.allocator.alloc(AstNodes::SimpleAssignmentTarget(transmute_self(self)));
        let node = match self.inner {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            SimpleAssignmentTarget::TSAsExpression(s) => {
                AstNodes::TSAsExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(s) => {
                AstNodes::TSSatisfiesExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            SimpleAssignmentTarget::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            SimpleAssignmentTarget::TSTypeAssertion(s) => {
                AstNodes::TSTypeAssertion(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                AstNodes::MemberExpression(self.allocator.alloc(AstNode {
                    inner: it.to_member_expression(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, SimpleAssignmentTarget<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::SimpleAssignmentTarget(transmute_self(self)));
        match self.inner {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(s) => {
                AstNode::<'a, 'b, IdentifierReference> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            SimpleAssignmentTarget::TSAsExpression(s) => {
                AstNode::<'a, 'b, TSAsExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(s) => {
                AstNode::<'a, 'b, TSSatisfiesExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            SimpleAssignmentTarget::TSNonNullExpression(s) => {
                AstNode::<'a, 'b, TSNonNullExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            SimpleAssignmentTarget::TSTypeAssertion(s) => {
                AstNode::<'a, 'b, TSTypeAssertion> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                AstNode::<'a, 'b, MemberExpression> {
                    inner: it.to_member_expression(),
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, SimpleAssignmentTarget<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a, 'b> AstNode<'a, 'b, AssignmentTargetPattern<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.allocator.alloc(AstNodes::AssignmentTargetPattern(transmute_self(self)));
        let node = match self.inner {
            AssignmentTargetPattern::ArrayAssignmentTarget(s) => {
                AstNodes::ArrayAssignmentTarget(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(s) => {
                AstNodes::ObjectAssignmentTarget(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, AssignmentTargetPattern<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::AssignmentTargetPattern(transmute_self(self)));
        match self.inner {
            AssignmentTargetPattern::ArrayAssignmentTarget(s) => {
                AstNode::<'a, 'b, ArrayAssignmentTarget> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(s) => {
                AstNode::<'a, 'b, ObjectAssignmentTarget> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, AssignmentTargetPattern<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ArrayAssignmentTarget<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn elements(&self) -> &AstNode<'a, 'b, Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrayAssignmentTarget(transmute_self(self))),
        })
    }

    pub fn rest(&self) -> Option<&AstNode<'a, 'b, AssignmentTargetRest<'a>>> {
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ArrayAssignmentTarget(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ObjectAssignmentTarget<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn properties(&self) -> &AstNode<'a, 'b, Vec<'a, AssignmentTargetProperty<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectAssignmentTarget(transmute_self(self))),
        })
    }

    pub fn rest(&self) -> Option<&AstNode<'a, 'b, AssignmentTargetRest<'a>>> {
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::ObjectAssignmentTarget(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, AssignmentTargetRest<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn target(&self) -> &AstNode<'a, 'b, AssignmentTarget<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.target,
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, AssignmentTargetMaybeDefault<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(s) => {
                AstNodes::AssignmentTargetWithDefault(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                AstNodes::AssignmentTarget(self.allocator.alloc(AstNode {
                    inner: it.to_assignment_target(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, AssignmentTargetMaybeDefault<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(s) => {
                AstNode::<'a, 'b, AssignmentTargetWithDefault> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                AstNode::<'a, 'b, AssignmentTarget> {
                    inner: it.to_assignment_target(),
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, AssignmentTargetMaybeDefault<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, AssignmentTargetWithDefault<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn binding(&self) -> &AstNode<'a, 'b, AssignmentTarget<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::AssignmentTargetWithDefault(transmute_self(self))),
        })
    }

    pub fn init(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.init,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::AssignmentTargetWithDefault(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, AssignmentTargetProperty<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, AssignmentTargetProperty<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(s) => {
                AstNode::<'a, 'b, AssignmentTargetPropertyIdentifier> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(s) => {
                AstNode::<'a, 'b, AssignmentTargetPropertyProperty> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, AssignmentTargetProperty<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, AssignmentTargetPropertyIdentifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn binding(&self) -> &AstNode<'a, 'b, IdentifierReference<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn init(&self) -> Option<&AstNode<'a, 'b, Expression<'a>>> {
        self.allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.parent,
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, AssignmentTargetPropertyProperty<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> &AstNode<'a, 'b, PropertyKey<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn binding(&self) -> &AstNode<'a, 'b, AssignmentTargetMaybeDefault<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }
}
impl<'a, 'b> AstNode<'a, 'b, SequenceExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expressions(&self) -> &AstNode<'a, 'b, Vec<'a, Expression<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expressions,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SequenceExpression(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, Super> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, AwaitExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AwaitExpression(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ChainExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, ChainElement<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ChainExpression(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, ChainElement<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            ChainElement::CallExpression(s) => {
                AstNodes::CallExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            ChainElement::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            it @ match_member_expression!(ChainElement) => {
                AstNodes::MemberExpression(self.allocator.alloc(AstNode {
                    inner: it.to_member_expression(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ChainElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ChainElement::CallExpression(s) => {
                AstNode::<'a, 'b, CallExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ChainElement::TSNonNullExpression(s) => AstNode::<'a, 'b, TSNonNullExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            it @ match_member_expression!(ChainElement) => AstNode::<'a, 'b, MemberExpression> {
                inner: it.to_member_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ChainElement<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ParenthesizedExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ParenthesizedExpression(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, Statement<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            Statement::BlockStatement(s) => {
                AstNodes::BlockStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Statement::BreakStatement(s) => {
                AstNodes::BreakStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Statement::ContinueStatement(s) => {
                AstNodes::ContinueStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Statement::DebuggerStatement(s) => {
                AstNodes::DebuggerStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Statement::DoWhileStatement(s) => {
                AstNodes::DoWhileStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Statement::EmptyStatement(s) => {
                AstNodes::EmptyStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Statement::ExpressionStatement(s) => {
                AstNodes::ExpressionStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Statement::ForInStatement(s) => {
                AstNodes::ForInStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Statement::ForOfStatement(s) => {
                AstNodes::ForOfStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Statement::ForStatement(s) => AstNodes::ForStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            Statement::IfStatement(s) => AstNodes::IfStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            Statement::LabeledStatement(s) => {
                AstNodes::LabeledStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Statement::ReturnStatement(s) => {
                AstNodes::ReturnStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Statement::SwitchStatement(s) => {
                AstNodes::SwitchStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Statement::ThrowStatement(s) => {
                AstNodes::ThrowStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Statement::TryStatement(s) => AstNodes::TryStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            Statement::WhileStatement(s) => {
                AstNodes::WhileStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Statement::WithStatement(s) => AstNodes::WithStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            it @ match_declaration!(Statement) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_declaration(),
                        parent,
                        allocator: self.allocator,
                    })
                    .as_ast_nodes();
            }
            it @ match_module_declaration!(Statement) => {
                AstNodes::ModuleDeclaration(self.allocator.alloc(AstNode {
                    inner: it.to_module_declaration(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, Statement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            Statement::BlockStatement(s) => {
                AstNode::<'a, 'b, BlockStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::BreakStatement(s) => {
                AstNode::<'a, 'b, BreakStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::ContinueStatement(s) => {
                AstNode::<'a, 'b, ContinueStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::DebuggerStatement(s) => {
                AstNode::<'a, 'b, DebuggerStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::DoWhileStatement(s) => {
                AstNode::<'a, 'b, DoWhileStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::EmptyStatement(s) => {
                AstNode::<'a, 'b, EmptyStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::ExpressionStatement(s) => AstNode::<'a, 'b, ExpressionStatement> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Statement::ForInStatement(s) => {
                AstNode::<'a, 'b, ForInStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::ForOfStatement(s) => {
                AstNode::<'a, 'b, ForOfStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::ForStatement(s) => {
                AstNode::<'a, 'b, ForStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::IfStatement(s) => {
                AstNode::<'a, 'b, IfStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::LabeledStatement(s) => {
                AstNode::<'a, 'b, LabeledStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::ReturnStatement(s) => {
                AstNode::<'a, 'b, ReturnStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::SwitchStatement(s) => {
                AstNode::<'a, 'b, SwitchStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::ThrowStatement(s) => {
                AstNode::<'a, 'b, ThrowStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::TryStatement(s) => {
                AstNode::<'a, 'b, TryStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::WhileStatement(s) => {
                AstNode::<'a, 'b, WhileStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Statement::WithStatement(s) => {
                AstNode::<'a, 'b, WithStatement> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            it @ match_declaration!(Statement) => AstNode::<'a, 'b, Declaration> {
                inner: it.to_declaration(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            it @ match_module_declaration!(Statement) => AstNode::<'a, 'b, ModuleDeclaration> {
                inner: it.to_module_declaration(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, Statement<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, Directive<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, StringLiteral<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Directive(transmute_self(self))),
        })
    }

    pub fn directive(&self) -> Atom<'a> {
        self.inner.directive
    }
}
impl<'a, 'b> AstNode<'a, 'b, Hashbang<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }
}
impl<'a, 'b> AstNode<'a, 'b, BlockStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Vec<'a, Statement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BlockStatement(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, Declaration<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            Declaration::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Declaration::FunctionDeclaration(s) => {
                AstNodes::Function(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Declaration::ClassDeclaration(s) => AstNodes::Class(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            Declaration::TSTypeAliasDeclaration(s) => {
                AstNodes::TSTypeAliasDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Declaration::TSInterfaceDeclaration(s) => {
                AstNodes::TSInterfaceDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Declaration::TSEnumDeclaration(s) => {
                AstNodes::TSEnumDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Declaration::TSModuleDeclaration(s) => {
                AstNodes::TSModuleDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            Declaration::TSImportEqualsDeclaration(s) => {
                AstNodes::TSImportEqualsDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, Declaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            Declaration::VariableDeclaration(s) => AstNode::<'a, 'b, VariableDeclaration> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Declaration::FunctionDeclaration(s) => {
                AstNode::<'a, 'b, Function> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            Declaration::ClassDeclaration(s) => {
                AstNode::<'a, 'b, Class> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            Declaration::TSTypeAliasDeclaration(s) => AstNode::<'a, 'b, TSTypeAliasDeclaration> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Declaration::TSInterfaceDeclaration(s) => AstNode::<'a, 'b, TSInterfaceDeclaration> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Declaration::TSEnumDeclaration(s) => {
                AstNode::<'a, 'b, TSEnumDeclaration> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            Declaration::TSModuleDeclaration(s) => AstNode::<'a, 'b, TSModuleDeclaration> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            Declaration::TSImportEqualsDeclaration(s) => {
                AstNode::<'a, 'b, TSImportEqualsDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, Declaration<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, VariableDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn kind(&self) -> VariableDeclarationKind {
        self.inner.kind
    }

    pub fn declarations(&self) -> &AstNode<'a, 'b, Vec<'a, VariableDeclarator<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.declarations,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::VariableDeclaration(transmute_self(self))),
        })
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }
}
impl<'a, 'b> AstNode<'a, 'b, VariableDeclarator<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn kind(&self) -> VariableDeclarationKind {
        self.inner.kind
    }

    pub fn id(&self) -> &AstNode<'a, 'b, BindingPattern<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::VariableDeclarator(transmute_self(self))),
        })
    }

    pub fn init(&self) -> Option<&AstNode<'a, 'b, Expression<'a>>> {
        self.allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::VariableDeclarator(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn definite(&self) -> bool {
        self.inner.definite
    }
}
impl<'a, 'b> AstNode<'a, 'b, EmptyStatement> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, ExpressionStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExpressionStatement(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, IfStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn test(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::IfStatement(transmute_self(self))),
        })
    }

    pub fn consequent(&self) -> &AstNode<'a, 'b, Statement<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::IfStatement(transmute_self(self))),
        })
    }

    pub fn alternate(&self) -> Option<&AstNode<'a, 'b, Statement<'a>>> {
        self.allocator
            .alloc(self.inner.alternate.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::IfStatement(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, DoWhileStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Statement<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::DoWhileStatement(transmute_self(self))),
        })
    }

    pub fn test(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::DoWhileStatement(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, WhileStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn test(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WhileStatement(transmute_self(self))),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Statement<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WhileStatement(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ForStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn init(&self) -> Option<&AstNode<'a, 'b, ForStatementInit<'a>>> {
        self.allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ForStatement(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn test(&self) -> Option<&AstNode<'a, 'b, Expression<'a>>> {
        self.allocator
            .alloc(self.inner.test.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ForStatement(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn update(&self) -> Option<&AstNode<'a, 'b, Expression<'a>>> {
        self.allocator
            .alloc(self.inner.update.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ForStatement(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Statement<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForStatement(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, ForStatementInit<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.allocator.alloc(AstNodes::ForStatementInit(transmute_self(self)));
        let node = match self.inner {
            ForStatementInit::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            it @ match_expression!(ForStatementInit) => {
                return self
                    .allocator
                    .alloc(AstNode { inner: it.to_expression(), parent, allocator: self.allocator })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ForStatementInit<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::ForStatementInit(transmute_self(self)));
        match self.inner {
            ForStatementInit::VariableDeclaration(s) => AstNode::<'a, 'b, VariableDeclaration> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            it @ match_expression!(ForStatementInit) => AstNode::<'a, 'b, Expression> {
                inner: it.to_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ForStatementInit<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ForInStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn left(&self) -> &AstNode<'a, 'b, ForStatementLeft<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForInStatement(transmute_self(self))),
        })
    }

    pub fn right(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForInStatement(transmute_self(self))),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Statement<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForInStatement(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, ForStatementLeft<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            ForStatementLeft::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            it @ match_assignment_target!(ForStatementLeft) => {
                AstNodes::AssignmentTarget(self.allocator.alloc(AstNode {
                    inner: it.to_assignment_target(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ForStatementLeft<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ForStatementLeft::VariableDeclaration(s) => AstNode::<'a, 'b, VariableDeclaration> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            it @ match_assignment_target!(ForStatementLeft) => {
                AstNode::<'a, 'b, AssignmentTarget> {
                    inner: it.to_assignment_target(),
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ForStatementLeft<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ForOfStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn r#await(&self) -> bool {
        self.inner.r#await
    }

    pub fn left(&self) -> &AstNode<'a, 'b, ForStatementLeft<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForOfStatement(transmute_self(self))),
        })
    }

    pub fn right(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForOfStatement(transmute_self(self))),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Statement<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForOfStatement(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ContinueStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn label(&self) -> Option<&AstNode<'a, 'b, LabelIdentifier<'a>>> {
        self.allocator
            .alloc(self.inner.label.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ContinueStatement(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, BreakStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn label(&self) -> Option<&AstNode<'a, 'b, LabelIdentifier<'a>>> {
        self.allocator
            .alloc(self.inner.label.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::BreakStatement(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ReturnStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn argument(&self) -> Option<&AstNode<'a, 'b, Expression<'a>>> {
        self.allocator
            .alloc(self.inner.argument.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ReturnStatement(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, WithStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn object(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WithStatement(transmute_self(self))),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Statement<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WithStatement(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, SwitchStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn discriminant(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.discriminant,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SwitchStatement(transmute_self(self))),
        })
    }

    pub fn cases(&self) -> &AstNode<'a, 'b, Vec<'a, SwitchCase<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.cases,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SwitchStatement(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, SwitchCase<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn test(&self) -> Option<&AstNode<'a, 'b, Expression<'a>>> {
        self.allocator
            .alloc(self.inner.test.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::SwitchCase(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn consequent(&self) -> &AstNode<'a, 'b, Vec<'a, Statement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SwitchCase(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, LabeledStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn label(&self) -> &AstNode<'a, 'b, LabelIdentifier<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.label,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LabeledStatement(transmute_self(self))),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Statement<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LabeledStatement(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ThrowStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ThrowStatement(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TryStatement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn block(&self) -> &AstNode<'a, 'b, BlockStatement<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.block.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TryStatement(transmute_self(self))),
        })
    }

    pub fn handler(&self) -> Option<&AstNode<'a, 'b, CatchClause<'a>>> {
        self.allocator
            .alloc(self.inner.handler.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TryStatement(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn finalizer(&self) -> Option<&AstNode<'a, 'b, BlockStatement<'a>>> {
        self.allocator
            .alloc(self.inner.finalizer.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TryStatement(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, CatchClause<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn param(&self) -> Option<&AstNode<'a, 'b, CatchParameter<'a>>> {
        self.allocator
            .alloc(self.inner.param.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::CatchClause(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn body(&self) -> &AstNode<'a, 'b, BlockStatement<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::CatchClause(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, CatchParameter<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn pattern(&self) -> &AstNode<'a, 'b, BindingPattern<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.pattern,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::CatchParameter(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, DebuggerStatement> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, BindingPattern<'a>> {
    pub fn kind(&self) -> &AstNode<'a, 'b, BindingPatternKind<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.kind,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn type_annotation(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation<'a>>> {
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
            }))
            .as_ref()
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }
}

impl<'a, 'b> AstNode<'a, 'b, BindingPatternKind<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            BindingPatternKind::BindingIdentifier(s) => {
                AstNodes::BindingIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            BindingPatternKind::ObjectPattern(s) => {
                AstNodes::ObjectPattern(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            BindingPatternKind::ArrayPattern(s) => {
                AstNodes::ArrayPattern(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            BindingPatternKind::AssignmentPattern(s) => {
                AstNodes::AssignmentPattern(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, BindingPatternKind<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            BindingPatternKind::BindingIdentifier(s) => {
                AstNode::<'a, 'b, BindingIdentifier> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            BindingPatternKind::ObjectPattern(s) => {
                AstNode::<'a, 'b, ObjectPattern> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            BindingPatternKind::ArrayPattern(s) => {
                AstNode::<'a, 'b, ArrayPattern> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            BindingPatternKind::AssignmentPattern(s) => {
                AstNode::<'a, 'b, AssignmentPattern> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, BindingPatternKind<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, AssignmentPattern<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn left(&self) -> &AstNode<'a, 'b, BindingPattern<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentPattern(transmute_self(self))),
        })
    }

    pub fn right(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentPattern(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ObjectPattern<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn properties(&self) -> &AstNode<'a, 'b, Vec<'a, BindingProperty<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectPattern(transmute_self(self))),
        })
    }

    pub fn rest(&self) -> Option<&AstNode<'a, 'b, BindingRestElement<'a>>> {
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ObjectPattern(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, BindingProperty<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn key(&self) -> &AstNode<'a, 'b, PropertyKey<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn value(&self) -> &AstNode<'a, 'b, BindingPattern<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn shorthand(&self) -> bool {
        self.inner.shorthand
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }
}
impl<'a, 'b> AstNode<'a, 'b, ArrayPattern<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn elements(&self) -> &AstNode<'a, 'b, Vec<'a, Option<BindingPattern<'a>>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrayPattern(transmute_self(self))),
        })
    }

    pub fn rest(&self) -> Option<&AstNode<'a, 'b, BindingRestElement<'a>>> {
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ArrayPattern(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, BindingRestElement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, BindingPattern<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BindingRestElement(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, Function<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn r#type(&self) -> FunctionType {
        self.inner.r#type
    }

    pub fn id(&self) -> Option<&AstNode<'a, 'b, BindingIdentifier<'a>>> {
        self.allocator
            .alloc(self.inner.id.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn generator(&self) -> bool {
        self.inner.generator
    }

    pub fn r#async(&self) -> bool {
        self.inner.r#async
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration<'a>>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn this_param(&self) -> Option<&AstNode<'a, 'b, TSThisParameter<'a>>> {
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn params(&self) -> &AstNode<'a, 'b, FormalParameters<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Function(transmute_self(self))),
        })
    }

    pub fn return_type(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation<'a>>> {
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn body(&self) -> Option<&AstNode<'a, 'b, FunctionBody<'a>>> {
        self.allocator
            .alloc(self.inner.body.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn pure(&self) -> bool {
        self.inner.pure
    }
}
impl<'a, 'b> AstNode<'a, 'b, FormalParameters<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn kind(&self) -> FormalParameterKind {
        self.inner.kind
    }

    pub fn items(&self) -> &AstNode<'a, 'b, Vec<'a, FormalParameter<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.items,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FormalParameters(transmute_self(self))),
        })
    }

    pub fn rest(&self) -> Option<&AstNode<'a, 'b, BindingRestElement<'a>>> {
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::FormalParameters(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, FormalParameter<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn decorators(&self) -> &AstNode<'a, 'b, Vec<'a, Decorator<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FormalParameter(transmute_self(self))),
        })
    }

    pub fn pattern(&self) -> &AstNode<'a, 'b, BindingPattern<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.pattern,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FormalParameter(transmute_self(self))),
        })
    }

    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }

    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }
}
impl<'a, 'b> AstNode<'a, 'b, FunctionBody<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn directives(&self) -> &AstNode<'a, 'b, Vec<'a, Directive<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FunctionBody(transmute_self(self))),
        })
    }

    pub fn statements(&self) -> &AstNode<'a, 'b, Vec<'a, Statement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.statements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FunctionBody(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ArrowFunctionExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> bool {
        self.inner.expression
    }

    pub fn r#async(&self) -> bool {
        self.inner.r#async
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration<'a>>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::ArrowFunctionExpression(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn params(&self) -> &AstNode<'a, 'b, FormalParameters<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrowFunctionExpression(transmute_self(self))),
        })
    }

    pub fn return_type(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation<'a>>> {
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::ArrowFunctionExpression(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn body(&self) -> &AstNode<'a, 'b, FunctionBody<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrowFunctionExpression(transmute_self(self))),
        })
    }

    pub fn pure(&self) -> bool {
        self.inner.pure
    }
}
impl<'a, 'b> AstNode<'a, 'b, YieldExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn delegate(&self) -> bool {
        self.inner.delegate
    }

    pub fn argument(&self) -> Option<&AstNode<'a, 'b, Expression<'a>>> {
        self.allocator
            .alloc(self.inner.argument.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::YieldExpression(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, Class<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn r#type(&self) -> ClassType {
        self.inner.r#type
    }

    pub fn decorators(&self) -> &AstNode<'a, 'b, Vec<'a, Decorator<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
        })
    }

    pub fn id(&self) -> Option<&AstNode<'a, 'b, BindingIdentifier<'a>>> {
        self.allocator
            .alloc(self.inner.id.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration<'a>>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn super_class(&self) -> Option<&AstNode<'a, 'b, Expression<'a>>> {
        self.allocator
            .alloc(self.inner.super_class.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn super_type_arguments(
        &self,
    ) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation<'a>>> {
        self.allocator
            .alloc(self.inner.super_type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn implements(&self) -> &AstNode<'a, 'b, Vec<'a, TSClassImplements<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.implements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, ClassBody<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
        })
    }

    pub fn r#abstract(&self) -> bool {
        self.inner.r#abstract
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }
}
impl<'a, 'b> AstNode<'a, 'b, ClassBody<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Vec<'a, ClassElement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ClassBody(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, ClassElement<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            ClassElement::StaticBlock(s) => AstNodes::StaticBlock(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            ClassElement::MethodDefinition(s) => {
                AstNodes::MethodDefinition(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            ClassElement::PropertyDefinition(s) => {
                AstNodes::PropertyDefinition(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            ClassElement::AccessorProperty(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            ClassElement::TSIndexSignature(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ClassElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ClassElement::StaticBlock(s) => {
                AstNode::<'a, 'b, StaticBlock> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ClassElement::MethodDefinition(s) => {
                AstNode::<'a, 'b, MethodDefinition> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ClassElement::PropertyDefinition(s) => AstNode::<'a, 'b, PropertyDefinition> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            ClassElement::AccessorProperty(s) => {
                AstNode::<'a, 'b, AccessorProperty> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ClassElement::TSIndexSignature(s) => {
                AstNode::<'a, 'b, TSIndexSignature> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ClassElement<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, MethodDefinition<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn r#type(&self) -> MethodDefinitionType {
        self.inner.r#type
    }

    pub fn decorators(&self) -> &AstNode<'a, 'b, Vec<'a, Decorator<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MethodDefinition(transmute_self(self))),
        })
    }

    pub fn key(&self) -> &AstNode<'a, 'b, PropertyKey<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MethodDefinition(transmute_self(self))),
        })
    }

    pub fn value(&self) -> &AstNode<'a, 'b, Function<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.value.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MethodDefinition(transmute_self(self))),
        })
    }

    pub fn kind(&self) -> MethodDefinitionKind {
        self.inner.kind
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }

    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }
}
impl<'a, 'b> AstNode<'a, 'b, PropertyDefinition<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn r#type(&self) -> PropertyDefinitionType {
        self.inner.r#type
    }

    pub fn decorators(&self) -> &AstNode<'a, 'b, Vec<'a, Decorator<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PropertyDefinition(transmute_self(self))),
        })
    }

    pub fn key(&self) -> &AstNode<'a, 'b, PropertyKey<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PropertyDefinition(transmute_self(self))),
        })
    }

    pub fn type_annotation(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation<'a>>> {
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::PropertyDefinition(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn value(&self) -> Option<&AstNode<'a, 'b, Expression<'a>>> {
        self.allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::PropertyDefinition(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn definite(&self) -> bool {
        self.inner.definite
    }

    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }
}
impl<'a, 'b> AstNode<'a, 'b, PrivateIdentifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}
impl<'a, 'b> AstNode<'a, 'b, StaticBlock<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Vec<'a, Statement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::StaticBlock(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, ModuleDeclaration<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.allocator.alloc(AstNodes::ModuleDeclaration(transmute_self(self)));
        let node = match self.inner {
            ModuleDeclaration::ImportDeclaration(s) => {
                AstNodes::ImportDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            ModuleDeclaration::ExportAllDeclaration(s) => {
                AstNodes::ExportAllDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            ModuleDeclaration::ExportDefaultDeclaration(s) => {
                AstNodes::ExportDefaultDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            ModuleDeclaration::ExportNamedDeclaration(s) => {
                AstNodes::ExportNamedDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            ModuleDeclaration::TSExportAssignment(s) => {
                AstNodes::TSExportAssignment(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ModuleDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::ModuleDeclaration(transmute_self(self)));
        match self.inner {
            ModuleDeclaration::ImportDeclaration(s) => {
                AstNode::<'a, 'b, ImportDeclaration> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ModuleDeclaration::ExportAllDeclaration(s) => AstNode::<'a, 'b, ExportAllDeclaration> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            ModuleDeclaration::ExportDefaultDeclaration(s) => {
                AstNode::<'a, 'b, ExportDefaultDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            ModuleDeclaration::ExportNamedDeclaration(s) => {
                AstNode::<'a, 'b, ExportNamedDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            ModuleDeclaration::TSExportAssignment(s) => AstNode::<'a, 'b, TSExportAssignment> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            ModuleDeclaration::TSNamespaceExportDeclaration(s) => {
                AstNode::<'a, 'b, TSNamespaceExportDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ModuleDeclaration<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, AccessorProperty<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn r#type(&self) -> AccessorPropertyType {
        self.inner.r#type
    }

    pub fn decorators(&self) -> &AstNode<'a, 'b, Vec<'a, Decorator<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn key(&self) -> &AstNode<'a, 'b, PropertyKey<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn type_annotation(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation<'a>>> {
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
            }))
            .as_ref()
    }

    pub fn value(&self) -> Option<&AstNode<'a, 'b, Expression<'a>>> {
        self.allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.parent,
            }))
            .as_ref()
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }

    pub fn r#override(&self) -> bool {
        self.inner.r#override
    }

    pub fn definite(&self) -> bool {
        self.inner.definite
    }

    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.inner.accessibility
    }
}
impl<'a, 'b> AstNode<'a, 'b, ImportExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn source(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportExpression(transmute_self(self))),
        })
    }

    pub fn options(&self) -> Option<&AstNode<'a, 'b, Expression<'a>>> {
        self.allocator
            .alloc(self.inner.options.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ImportExpression(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn phase(&self) -> Option<ImportPhase> {
        self.inner.phase
    }
}
impl<'a, 'b> AstNode<'a, 'b, ImportDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn specifiers(&self) -> Option<&AstNode<'a, 'b, Vec<'a, ImportDeclarationSpecifier<'a>>>> {
        self.allocator
            .alloc(self.inner.specifiers.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ImportDeclaration(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn source(&self) -> &AstNode<'a, 'b, StringLiteral<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportDeclaration(transmute_self(self))),
        })
    }

    pub fn phase(&self) -> Option<ImportPhase> {
        self.inner.phase
    }

    pub fn with_clause(&self) -> Option<&AstNode<'a, 'b, WithClause<'a>>> {
        self.allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ImportDeclaration(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }
}

impl<'a, 'b> AstNode<'a, 'b, ImportDeclarationSpecifier<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                AstNodes::ImportSpecifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                AstNodes::ImportDefaultSpecifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                AstNodes::ImportNamespaceSpecifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ImportDeclarationSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                AstNode::<'a, 'b, ImportSpecifier> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                AstNode::<'a, 'b, ImportDefaultSpecifier> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                AstNode::<'a, 'b, ImportNamespaceSpecifier> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ImportDeclarationSpecifier<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ImportSpecifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn imported(&self) -> &AstNode<'a, 'b, ModuleExportName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.imported,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportSpecifier(transmute_self(self))),
        })
    }

    pub fn local(&self) -> &AstNode<'a, 'b, BindingIdentifier<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportSpecifier(transmute_self(self))),
        })
    }

    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }
}
impl<'a, 'b> AstNode<'a, 'b, ImportDefaultSpecifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn local(&self) -> &AstNode<'a, 'b, BindingIdentifier<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportDefaultSpecifier(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ImportNamespaceSpecifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn local(&self) -> &AstNode<'a, 'b, BindingIdentifier<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportNamespaceSpecifier(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, WithClause<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn attributes_keyword(&self) -> &AstNode<'a, 'b, IdentifierName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.attributes_keyword,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn with_entries(&self) -> &AstNode<'a, 'b, Vec<'a, ImportAttribute<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.with_entries,
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ImportAttribute<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn key(&self) -> &AstNode<'a, 'b, ImportAttributeKey<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn value(&self) -> &AstNode<'a, 'b, StringLiteral<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, ImportAttributeKey<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            ImportAttributeKey::Identifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }))
            }
            ImportAttributeKey::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ImportAttributeKey<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ImportAttributeKey::Identifier(s) => {
                AstNode::<'a, 'b, IdentifierName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ImportAttributeKey::StringLiteral(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ImportAttributeKey<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ExportNamedDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn declaration(&self) -> Option<&AstNode<'a, 'b, Declaration<'a>>> {
        self.allocator
            .alloc(self.inner.declaration.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::ExportNamedDeclaration(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn specifiers(&self) -> &AstNode<'a, 'b, Vec<'a, ExportSpecifier<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.specifiers,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportNamedDeclaration(transmute_self(self))),
        })
    }

    pub fn source(&self) -> Option<&AstNode<'a, 'b, StringLiteral<'a>>> {
        self.allocator
            .alloc(self.inner.source.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::ExportNamedDeclaration(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    pub fn with_clause(&self) -> Option<&AstNode<'a, 'b, WithClause<'a>>> {
        self.allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::ExportNamedDeclaration(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, ExportDefaultDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn exported(&self) -> &AstNode<'a, 'b, ModuleExportName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.exported,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportDefaultDeclaration(transmute_self(self))),
        })
    }

    pub fn declaration(&self) -> &AstNode<'a, 'b, ExportDefaultDeclarationKind<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.declaration,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportDefaultDeclaration(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, ExportAllDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn exported(&self) -> Option<&AstNode<'a, 'b, ModuleExportName<'a>>> {
        self.allocator
            .alloc(self.inner.exported.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ExportAllDeclaration(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn source(&self) -> &AstNode<'a, 'b, StringLiteral<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportAllDeclaration(transmute_self(self))),
        })
    }

    pub fn with_clause(&self) -> Option<&AstNode<'a, 'b, WithClause<'a>>> {
        self.allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ExportAllDeclaration(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }
}
impl<'a, 'b> AstNode<'a, 'b, ExportSpecifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn local(&self) -> &AstNode<'a, 'b, ModuleExportName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportSpecifier(transmute_self(self))),
        })
    }

    pub fn exported(&self) -> &AstNode<'a, 'b, ModuleExportName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.exported,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportSpecifier(transmute_self(self))),
        })
    }

    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }
}

impl<'a, 'b> AstNode<'a, 'b, ExportDefaultDeclarationKind<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            ExportDefaultDeclarationKind::FunctionDeclaration(s) => {
                AstNodes::Function(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            ExportDefaultDeclarationKind::ClassDeclaration(s) => {
                AstNodes::Class(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(s) => {
                AstNodes::TSInterfaceDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            it @ match_expression!(ExportDefaultDeclarationKind) => {
                return self
                    .allocator
                    .alloc(AstNode { inner: it.to_expression(), parent, allocator: self.allocator })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ExportDefaultDeclarationKind<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ExportDefaultDeclarationKind::FunctionDeclaration(s) => {
                AstNode::<'a, 'b, Function> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            ExportDefaultDeclarationKind::ClassDeclaration(s) => {
                AstNode::<'a, 'b, Class> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(s) => {
                AstNode::<'a, 'b, TSInterfaceDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            it @ match_expression!(ExportDefaultDeclarationKind) => AstNode::<'a, 'b, Expression> {
                inner: it.to_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ExportDefaultDeclarationKind<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a, 'b> AstNode<'a, 'b, ModuleExportName<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            ModuleExportName::IdentifierName(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }))
            }
            ModuleExportName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }))
            }
            ModuleExportName::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, ModuleExportName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            ModuleExportName::IdentifierName(s) => {
                AstNode::<'a, 'b, IdentifierName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            ModuleExportName::IdentifierReference(s) => AstNode::<'a, 'b, IdentifierReference> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            ModuleExportName::StringLiteral(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, ModuleExportName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, V8IntrinsicExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> &AstNode<'a, 'b, IdentifierName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::V8IntrinsicExpression(transmute_self(self))),
        })
    }

    pub fn arguments(&self) -> &AstNode<'a, 'b, Vec<'a, Argument<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::V8IntrinsicExpression(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, BooleanLiteral> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn value(&self) -> bool {
        self.inner.value
    }
}
impl<'a, 'b> AstNode<'a, 'b, NullLiteral> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, NumericLiteral<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn value(&self) -> f64 {
        self.inner.value
    }

    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }

    pub fn base(&self) -> NumberBase {
        self.inner.base
    }
}
impl<'a, 'b> AstNode<'a, 'b, StringLiteral<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }

    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }

    pub fn lone_surrogates(&self) -> bool {
        self.inner.lone_surrogates
    }
}
impl<'a, 'b> AstNode<'a, 'b, BigIntLiteral<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn raw(&self) -> Atom<'a> {
        self.inner.raw
    }

    pub fn base(&self) -> BigintBase {
        self.inner.base
    }
}
impl<'a, 'b> AstNode<'a, 'b, RegExpLiteral<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn regex(&self) -> &RegExp<'a> {
        &self.inner.regex
    }

    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXElement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn opening_element(&self) -> &AstNode<'a, 'b, JSXOpeningElement<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.opening_element.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXElement(transmute_self(self))),
        })
    }

    pub fn children(&self) -> &AstNode<'a, 'b, Vec<'a, JSXChild<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.children,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXElement(transmute_self(self))),
        })
    }

    pub fn closing_element(&self) -> Option<&AstNode<'a, 'b, JSXClosingElement<'a>>> {
        self.allocator
            .alloc(self.inner.closing_element.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::JSXElement(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXOpeningElement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> &AstNode<'a, 'b, JSXElementName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXOpeningElement(transmute_self(self))),
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation<'a>>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::JSXOpeningElement(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn attributes(&self) -> &AstNode<'a, 'b, Vec<'a, JSXAttributeItem<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.attributes,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXOpeningElement(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXClosingElement<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> &AstNode<'a, 'b, JSXElementName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXClosingElement(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXFragment<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn opening_fragment(&self) -> &AstNode<'a, 'b, JSXOpeningFragment> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.opening_fragment,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXFragment(transmute_self(self))),
        })
    }

    pub fn children(&self) -> &AstNode<'a, 'b, Vec<'a, JSXChild<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.children,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXFragment(transmute_self(self))),
        })
    }

    pub fn closing_fragment(&self) -> &AstNode<'a, 'b, JSXClosingFragment> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.closing_fragment,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXFragment(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXOpeningFragment> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXClosingFragment> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a, 'b> AstNode<'a, 'b, JSXElementName<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.allocator.alloc(AstNodes::JSXElementName(transmute_self(self)));
        let node = match self.inner {
            JSXElementName::Identifier(s) => {
                AstNodes::JSXIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            JSXElementName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            JSXElementName::NamespacedName(s) => {
                AstNodes::JSXNamespacedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            JSXElementName::MemberExpression(s) => {
                AstNodes::JSXMemberExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            JSXElementName::ThisExpression(s) => {
                AstNodes::ThisExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, JSXElementName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::JSXElementName(transmute_self(self)));
        match self.inner {
            JSXElementName::Identifier(s) => {
                AstNode::<'a, 'b, JSXIdentifier> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            JSXElementName::IdentifierReference(s) => AstNode::<'a, 'b, IdentifierReference> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            JSXElementName::NamespacedName(s) => {
                AstNode::<'a, 'b, JSXNamespacedName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            JSXElementName::MemberExpression(s) => AstNode::<'a, 'b, JSXMemberExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            JSXElementName::ThisExpression(s) => {
                AstNode::<'a, 'b, ThisExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, JSXElementName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXNamespacedName<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn namespace(&self) -> &AstNode<'a, 'b, JSXIdentifier<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.namespace,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXNamespacedName(transmute_self(self))),
        })
    }

    pub fn name(&self) -> &AstNode<'a, 'b, JSXIdentifier<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXNamespacedName(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXMemberExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn object(&self) -> &AstNode<'a, 'b, JSXMemberExpressionObject<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXMemberExpression(transmute_self(self))),
        })
    }

    pub fn property(&self) -> &AstNode<'a, 'b, JSXIdentifier<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXMemberExpression(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, JSXMemberExpressionObject<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent =
            self.allocator.alloc(AstNodes::JSXMemberExpressionObject(transmute_self(self)));
        let node = match self.inner {
            JSXMemberExpressionObject::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            JSXMemberExpressionObject::MemberExpression(s) => {
                AstNodes::JSXMemberExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            JSXMemberExpressionObject::ThisExpression(s) => {
                AstNodes::ThisExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, JSXMemberExpressionObject<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent =
            self.allocator.alloc(AstNodes::JSXMemberExpressionObject(transmute_self(self)));
        match self.inner {
            JSXMemberExpressionObject::IdentifierReference(s) => {
                AstNode::<'a, 'b, IdentifierReference> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            JSXMemberExpressionObject::MemberExpression(s) => {
                AstNode::<'a, 'b, JSXMemberExpression> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            JSXMemberExpressionObject::ThisExpression(s) => {
                AstNode::<'a, 'b, ThisExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, JSXMemberExpressionObject<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXExpressionContainer<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, JSXExpression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXExpressionContainer(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, JSXExpression<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            JSXExpression::EmptyExpression(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            it @ match_expression!(JSXExpression) => {
                return self
                    .allocator
                    .alloc(AstNode { inner: it.to_expression(), parent, allocator: self.allocator })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, JSXExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            JSXExpression::EmptyExpression(s) => AstNode::<'a, 'b, JSXEmptyExpression> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            it @ match_expression!(JSXExpression) => AstNode::<'a, 'b, Expression> {
                inner: it.to_expression(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, JSXExpression<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXEmptyExpression> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a, 'b> AstNode<'a, 'b, JSXAttributeItem<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.allocator.alloc(AstNodes::JSXAttributeItem(transmute_self(self)));
        let node = match self.inner {
            JSXAttributeItem::Attribute(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            JSXAttributeItem::SpreadAttribute(s) => {
                AstNodes::JSXSpreadAttribute(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, JSXAttributeItem<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::JSXAttributeItem(transmute_self(self)));
        match self.inner {
            JSXAttributeItem::Attribute(s) => {
                AstNode::<'a, 'b, JSXAttribute> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            JSXAttributeItem::SpreadAttribute(s) => AstNode::<'a, 'b, JSXSpreadAttribute> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, JSXAttributeItem<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXAttribute<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> &AstNode<'a, 'b, JSXAttributeName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn value(&self) -> Option<&AstNode<'a, 'b, JSXAttributeValue<'a>>> {
        self.allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.parent,
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXSpreadAttribute<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXSpreadAttribute(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, JSXAttributeName<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            JSXAttributeName::Identifier(s) => {
                AstNodes::JSXIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            JSXAttributeName::NamespacedName(s) => {
                AstNodes::JSXNamespacedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, JSXAttributeName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            JSXAttributeName::Identifier(s) => {
                AstNode::<'a, 'b, JSXIdentifier> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            JSXAttributeName::NamespacedName(s) => {
                AstNode::<'a, 'b, JSXNamespacedName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, JSXAttributeName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a, 'b> AstNode<'a, 'b, JSXAttributeValue<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            JSXAttributeValue::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            JSXAttributeValue::ExpressionContainer(s) => {
                AstNodes::JSXExpressionContainer(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            JSXAttributeValue::Element(s) => AstNodes::JSXElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            JSXAttributeValue::Fragment(s) => {
                AstNodes::JSXFragment(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, JSXAttributeValue<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            JSXAttributeValue::StringLiteral(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            JSXAttributeValue::ExpressionContainer(s) => {
                AstNode::<'a, 'b, JSXExpressionContainer> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            JSXAttributeValue::Element(s) => {
                AstNode::<'a, 'b, JSXElement> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            JSXAttributeValue::Fragment(s) => {
                AstNode::<'a, 'b, JSXFragment> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, JSXAttributeValue<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXIdentifier<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}

impl<'a, 'b> AstNode<'a, 'b, JSXChild<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            JSXChild::Text(s) => AstNodes::JSXText(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            JSXChild::Element(s) => AstNodes::JSXElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            JSXChild::Fragment(s) => AstNodes::JSXFragment(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            JSXChild::ExpressionContainer(s) => {
                AstNodes::JSXExpressionContainer(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            JSXChild::Spread(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, JSXChild<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            JSXChild::Text(s) => {
                AstNode::<'a, 'b, JSXText> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            JSXChild::Element(s) => {
                AstNode::<'a, 'b, JSXElement> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            JSXChild::Fragment(s) => {
                AstNode::<'a, 'b, JSXFragment> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            JSXChild::ExpressionContainer(s) => AstNode::<'a, 'b, JSXExpressionContainer> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            JSXChild::Spread(s) => {
                AstNode::<'a, 'b, JSXSpreadChild> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, JSXChild<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXSpreadChild<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSXText<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }

    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSThisParameter<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn this_span(&self) -> Span {
        self.inner.this_span
    }

    pub fn type_annotation(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation<'a>>> {
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSThisParameter(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSEnumDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn id(&self) -> &AstNode<'a, 'b, BindingIdentifier<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumDeclaration(transmute_self(self))),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, TSEnumBody<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumDeclaration(transmute_self(self))),
        })
    }

    pub fn r#const(&self) -> bool {
        self.inner.r#const
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSEnumBody<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn members(&self) -> &AstNode<'a, 'b, Vec<'a, TSEnumMember<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.members,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumBody(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSEnumMember<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn id(&self) -> &AstNode<'a, 'b, TSEnumMemberName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumMember(transmute_self(self))),
        })
    }

    pub fn initializer(&self) -> Option<&AstNode<'a, 'b, Expression<'a>>> {
        self.allocator
            .alloc(self.inner.initializer.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSEnumMember(transmute_self(self))),
            }))
            .as_ref()
    }
}

impl<'a, 'b> AstNode<'a, 'b, TSEnumMemberName<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            TSEnumMemberName::Identifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSEnumMemberName::String(s) => AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSEnumMemberName::ComputedString(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSEnumMemberName::ComputedTemplateString(s) => {
                AstNodes::TemplateLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSEnumMemberName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSEnumMemberName::Identifier(s) => {
                AstNode::<'a, 'b, IdentifierName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSEnumMemberName::String(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSEnumMemberName::ComputedString(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSEnumMemberName::ComputedTemplateString(s) => {
                AstNode::<'a, 'b, TemplateLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSEnumMemberName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeAnnotation<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAnnotation(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSLiteralType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn literal(&self) -> &AstNode<'a, 'b, TSLiteral<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.literal,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSLiteralType(transmute_self(self))),
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, TSLiteral<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            TSLiteral::BooleanLiteral(s) => {
                AstNodes::BooleanLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSLiteral::NumericLiteral(s) => {
                AstNodes::NumericLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSLiteral::BigIntLiteral(s) => AstNodes::BigIntLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSLiteral::StringLiteral(s) => AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSLiteral::TemplateLiteral(s) => {
                AstNodes::TemplateLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSLiteral::UnaryExpression(s) => {
                AstNodes::UnaryExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSLiteral<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSLiteral::BooleanLiteral(s) => {
                AstNode::<'a, 'b, BooleanLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSLiteral::NumericLiteral(s) => {
                AstNode::<'a, 'b, NumericLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSLiteral::BigIntLiteral(s) => {
                AstNode::<'a, 'b, BigIntLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSLiteral::StringLiteral(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSLiteral::TemplateLiteral(s) => {
                AstNode::<'a, 'b, TemplateLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSLiteral::UnaryExpression(s) => {
                AstNode::<'a, 'b, UnaryExpression> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSLiteral<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a, 'b> AstNode<'a, 'b, TSType<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            TSType::TSAnyKeyword(s) => AstNodes::TSAnyKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSType::TSBigIntKeyword(s) => {
                AstNodes::TSBigIntKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSBooleanKeyword(s) => {
                AstNodes::TSBooleanKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSIntrinsicKeyword(s) => {
                AstNodes::TSIntrinsicKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSNeverKeyword(s) => AstNodes::TSNeverKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSType::TSNullKeyword(s) => AstNodes::TSNullKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSType::TSNumberKeyword(s) => {
                AstNodes::TSNumberKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSObjectKeyword(s) => {
                AstNodes::TSObjectKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSStringKeyword(s) => {
                AstNodes::TSStringKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSSymbolKeyword(s) => {
                AstNodes::TSSymbolKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSUndefinedKeyword(s) => {
                AstNodes::TSUndefinedKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSUnknownKeyword(s) => {
                AstNodes::TSUnknownKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSVoidKeyword(s) => AstNodes::TSVoidKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSType::TSArrayType(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSType::TSConditionalType(s) => {
                AstNodes::TSConditionalType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSConstructorType(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSType::TSFunctionType(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSType::TSImportType(s) => AstNodes::TSImportType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSType::TSIndexedAccessType(s) => {
                AstNodes::TSIndexedAccessType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSInferType(s) => AstNodes::TSInferType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSType::TSIntersectionType(s) => {
                AstNodes::TSIntersectionType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSLiteralType(s) => AstNodes::TSLiteralType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSType::TSMappedType(s) => AstNodes::TSMappedType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSType::TSNamedTupleMember(s) => {
                AstNodes::TSNamedTupleMember(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSTemplateLiteralType(s) => {
                AstNodes::TSTemplateLiteralType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSThisType(s) => AstNodes::TSThisType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSType::TSTupleType(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSType::TSTypeLiteral(s) => AstNodes::TSTypeLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSType::TSTypeOperatorType(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSType::TSTypePredicate(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSType::TSTypeQuery(s) => AstNodes::TSTypeQuery(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSType::TSTypeReference(s) => {
                AstNodes::TSTypeReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::TSUnionType(s) => AstNodes::TSUnionType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
            })),
            TSType::TSParenthesizedType(s) => {
                AstNodes::TSParenthesizedType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSType::JSDocNullableType(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSType::JSDocNonNullableType(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSType::JSDocUnknownType(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSType::TSAnyKeyword(s) => {
                AstNode::<'a, 'b, TSAnyKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSBigIntKeyword(s) => {
                AstNode::<'a, 'b, TSBigIntKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSBooleanKeyword(s) => {
                AstNode::<'a, 'b, TSBooleanKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSIntrinsicKeyword(s) => AstNode::<'a, 'b, TSIntrinsicKeyword> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::TSNeverKeyword(s) => {
                AstNode::<'a, 'b, TSNeverKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSNullKeyword(s) => {
                AstNode::<'a, 'b, TSNullKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSNumberKeyword(s) => {
                AstNode::<'a, 'b, TSNumberKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSObjectKeyword(s) => {
                AstNode::<'a, 'b, TSObjectKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSStringKeyword(s) => {
                AstNode::<'a, 'b, TSStringKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSSymbolKeyword(s) => {
                AstNode::<'a, 'b, TSSymbolKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSUndefinedKeyword(s) => AstNode::<'a, 'b, TSUndefinedKeyword> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::TSUnknownKeyword(s) => {
                AstNode::<'a, 'b, TSUnknownKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSVoidKeyword(s) => {
                AstNode::<'a, 'b, TSVoidKeyword> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSArrayType(s) => {
                AstNode::<'a, 'b, TSArrayType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSConditionalType(s) => {
                AstNode::<'a, 'b, TSConditionalType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSConstructorType(s) => {
                AstNode::<'a, 'b, TSConstructorType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSFunctionType(s) => {
                AstNode::<'a, 'b, TSFunctionType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSImportType(s) => {
                AstNode::<'a, 'b, TSImportType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSIndexedAccessType(s) => AstNode::<'a, 'b, TSIndexedAccessType> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::TSInferType(s) => {
                AstNode::<'a, 'b, TSInferType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSIntersectionType(s) => AstNode::<'a, 'b, TSIntersectionType> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::TSLiteralType(s) => {
                AstNode::<'a, 'b, TSLiteralType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSMappedType(s) => {
                AstNode::<'a, 'b, TSMappedType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSNamedTupleMember(s) => AstNode::<'a, 'b, TSNamedTupleMember> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::TSTemplateLiteralType(s) => AstNode::<'a, 'b, TSTemplateLiteralType> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::TSThisType(s) => {
                AstNode::<'a, 'b, TSThisType> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            TSType::TSTupleType(s) => {
                AstNode::<'a, 'b, TSTupleType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSTypeLiteral(s) => {
                AstNode::<'a, 'b, TSTypeLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSTypeOperatorType(s) => {
                AstNode::<'a, 'b, TSTypeOperator> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSTypePredicate(s) => {
                AstNode::<'a, 'b, TSTypePredicate> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSTypeQuery(s) => {
                AstNode::<'a, 'b, TSTypeQuery> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSTypeReference(s) => {
                AstNode::<'a, 'b, TSTypeReference> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSUnionType(s) => {
                AstNode::<'a, 'b, TSUnionType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::TSParenthesizedType(s) => AstNode::<'a, 'b, TSParenthesizedType> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::JSDocNullableType(s) => {
                AstNode::<'a, 'b, JSDocNullableType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSType::JSDocNonNullableType(s) => AstNode::<'a, 'b, JSDocNonNullableType> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSType::JSDocUnknownType(s) => {
                AstNode::<'a, 'b, JSDocUnknownType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSType<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSConditionalType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn check_type(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.check_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(transmute_self(self))),
        })
    }

    pub fn extends_type(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.extends_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(transmute_self(self))),
        })
    }

    pub fn true_type(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.true_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(transmute_self(self))),
        })
    }

    pub fn false_type(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.false_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSUnionType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn types(&self) -> &AstNode<'a, 'b, Vec<'a, TSType<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSUnionType(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSIntersectionType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn types(&self) -> &AstNode<'a, 'b, Vec<'a, TSType<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIntersectionType(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSParenthesizedType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSParenthesizedType(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeOperator<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn operator(&self) -> TSTypeOperatorOperator {
        self.inner.operator
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSArrayType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn element_type(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.element_type,
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSIndexedAccessType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn object_type(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.object_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIndexedAccessType(transmute_self(self))),
        })
    }

    pub fn index_type(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.index_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIndexedAccessType(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTupleType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn element_types(&self) -> &AstNode<'a, 'b, Vec<'a, TSTupleElement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.element_types,
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSNamedTupleMember<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn label(&self) -> &AstNode<'a, 'b, IdentifierName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.label,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSNamedTupleMember(transmute_self(self))),
        })
    }

    pub fn element_type(&self) -> &AstNode<'a, 'b, TSTupleElement<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.element_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSNamedTupleMember(transmute_self(self))),
        })
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSOptionalType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSRestType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}

impl<'a, 'b> AstNode<'a, 'b, TSTupleElement<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            TSTupleElement::TSOptionalType(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSTupleElement::TSRestType(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            it @ match_ts_type!(TSTupleElement) => {
                return self
                    .allocator
                    .alloc(AstNode { inner: it.to_ts_type(), parent, allocator: self.allocator })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSTupleElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSTupleElement::TSOptionalType(s) => {
                AstNode::<'a, 'b, TSOptionalType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSTupleElement::TSRestType(s) => {
                AstNode::<'a, 'b, TSRestType> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
            it @ match_ts_type!(TSTupleElement) => AstNode::<'a, 'b, TSType> {
                inner: it.to_ts_type(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSTupleElement<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSAnyKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSStringKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSBooleanKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSNumberKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSNeverKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSIntrinsicKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSUnknownKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSNullKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSUndefinedKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSVoidKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSSymbolKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSThisType> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSObjectKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSBigIntKeyword> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeReference<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_name(&self) -> &AstNode<'a, 'b, TSTypeName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeReference(transmute_self(self))),
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation<'a>>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeReference(transmute_self(self))),
            }))
            .as_ref()
    }
}

impl<'a, 'b> AstNode<'a, 'b, TSTypeName<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.allocator.alloc(AstNodes::TSTypeName(transmute_self(self)));
        let node = match self.inner {
            TSTypeName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSTypeName::QualifiedName(s) => {
                AstNodes::TSQualifiedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSTypeName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::TSTypeName(transmute_self(self)));
        match self.inner {
            TSTypeName::IdentifierReference(s) => AstNode::<'a, 'b, IdentifierReference> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSTypeName::QualifiedName(s) => {
                AstNode::<'a, 'b, TSQualifiedName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSTypeName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSQualifiedName<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn left(&self) -> &AstNode<'a, 'b, TSTypeName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSQualifiedName(transmute_self(self))),
        })
    }

    pub fn right(&self) -> &AstNode<'a, 'b, IdentifierName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSQualifiedName(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeParameterInstantiation<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn params(&self) -> &AstNode<'a, 'b, Vec<'a, TSType<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.params,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::TSTypeParameterInstantiation(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeParameter<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> &AstNode<'a, 'b, BindingIdentifier<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeParameter(transmute_self(self))),
        })
    }

    pub fn constraint(&self) -> Option<&AstNode<'a, 'b, TSType<'a>>> {
        self.allocator
            .alloc(self.inner.constraint.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeParameter(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn default(&self) -> Option<&AstNode<'a, 'b, TSType<'a>>> {
        self.allocator
            .alloc(self.inner.default.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeParameter(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn r#in(&self) -> bool {
        self.inner.r#in
    }

    pub fn out(&self) -> bool {
        self.inner.out
    }

    pub fn r#const(&self) -> bool {
        self.inner.r#const
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeParameterDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn params(&self) -> &AstNode<'a, 'b, Vec<'a, TSTypeParameter<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.params,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::TSTypeParameterDeclaration(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeAliasDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn id(&self) -> &AstNode<'a, 'b, BindingIdentifier<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAliasDeclaration(transmute_self(self))),
        })
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration<'a>>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::TSTypeAliasDeclaration(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAliasDeclaration(transmute_self(self))),
        })
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSClassImplements<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, TSTypeName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSClassImplements(transmute_self(self))),
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation<'a>>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSClassImplements(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSInterfaceDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn id(&self) -> &AstNode<'a, 'b, BindingIdentifier<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceDeclaration(transmute_self(self))),
        })
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration<'a>>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::TSInterfaceDeclaration(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn extends(&self) -> &AstNode<'a, 'b, Vec<'a, TSInterfaceHeritage<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.extends,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceDeclaration(transmute_self(self))),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, TSInterfaceBody<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceDeclaration(transmute_self(self))),
        })
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSInterfaceBody<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Vec<'a, TSSignature<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSPropertySignature<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    pub fn key(&self) -> &AstNode<'a, 'b, PropertyKey<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSPropertySignature(transmute_self(self))),
        })
    }

    pub fn type_annotation(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation<'a>>> {
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSPropertySignature(transmute_self(self))),
            }))
            .as_ref()
    }
}

impl<'a, 'b> AstNode<'a, 'b, TSSignature<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            TSSignature::TSIndexSignature(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSSignature::TSPropertySignature(s) => {
                AstNodes::TSPropertySignature(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSSignature::TSCallSignatureDeclaration(s) => {
                panic!(
                    "No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSSignature::TSConstructSignatureDeclaration(s) => {
                AstNodes::TSConstructSignatureDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSSignature::TSMethodSignature(s) => {
                AstNodes::TSMethodSignature(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSSignature<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSSignature::TSIndexSignature(s) => {
                AstNode::<'a, 'b, TSIndexSignature> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSSignature::TSPropertySignature(s) => AstNode::<'a, 'b, TSPropertySignature> {
                inner: s,
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
            TSSignature::TSCallSignatureDeclaration(s) => {
                AstNode::<'a, 'b, TSCallSignatureDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            TSSignature::TSConstructSignatureDeclaration(s) => {
                AstNode::<'a, 'b, TSConstructSignatureDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            TSSignature::TSMethodSignature(s) => {
                AstNode::<'a, 'b, TSMethodSignature> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSSignature<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSIndexSignature<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn parameters(&self) -> &AstNode<'a, 'b, Vec<'a, TSIndexSignatureName<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.parameters,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSTypeAnnotation<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.type_annotation.as_ref(),
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn readonly(&self) -> bool {
        self.inner.readonly
    }

    pub fn r#static(&self) -> bool {
        self.inner.r#static
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSCallSignatureDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration<'a>>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
            }))
            .as_ref()
    }

    pub fn this_param(&self) -> Option<&AstNode<'a, 'b, TSThisParameter<'a>>> {
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
            }))
            .as_ref()
    }

    pub fn params(&self) -> &AstNode<'a, 'b, FormalParameters<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn return_type(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation<'a>>> {
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSMethodSignature<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn key(&self) -> &AstNode<'a, 'b, PropertyKey<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSMethodSignature(transmute_self(self))),
        })
    }

    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn kind(&self) -> TSMethodSignatureKind {
        self.inner.kind
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration<'a>>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMethodSignature(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn this_param(&self) -> Option<&AstNode<'a, 'b, TSThisParameter<'a>>> {
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMethodSignature(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn params(&self) -> &AstNode<'a, 'b, FormalParameters<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSMethodSignature(transmute_self(self))),
        })
    }

    pub fn return_type(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation<'a>>> {
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMethodSignature(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSConstructSignatureDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration<'a>>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| {
                AstNode {
                    inner: inner.as_ref(),
                    allocator: self.allocator,
                    parent: self
                        .allocator
                        .alloc(AstNodes::TSConstructSignatureDeclaration(transmute_self(self))),
                }
            }))
            .as_ref()
    }

    pub fn params(&self) -> &AstNode<'a, 'b, FormalParameters<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::TSConstructSignatureDeclaration(transmute_self(self))),
        })
    }

    pub fn return_type(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation<'a>>> {
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| {
                AstNode {
                    inner: inner.as_ref(),
                    allocator: self.allocator,
                    parent: self
                        .allocator
                        .alloc(AstNodes::TSConstructSignatureDeclaration(transmute_self(self))),
                }
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSIndexSignatureName<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSTypeAnnotation<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.type_annotation.as_ref(),
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSInterfaceHeritage<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceHeritage(transmute_self(self))),
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation<'a>>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSInterfaceHeritage(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypePredicate<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn parameter_name(&self) -> &AstNode<'a, 'b, TSTypePredicateName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.parameter_name,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn asserts(&self) -> bool {
        self.inner.asserts
    }

    pub fn type_annotation(&self) -> Option<&AstNode<'a, 'b, TSTypeAnnotation<'a>>> {
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
            }))
            .as_ref()
    }
}

impl<'a, 'b> AstNode<'a, 'b, TSTypePredicateName<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            TSTypePredicateName::Identifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSTypePredicateName::This(s) => AstNodes::TSThisType(self.allocator.alloc(AstNode {
                inner: s,
                parent,
                allocator: self.allocator,
            })),
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSTypePredicateName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSTypePredicateName::Identifier(s) => {
                AstNode::<'a, 'b, IdentifierName> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSTypePredicateName::This(s) => {
                AstNode::<'a, 'b, TSThisType> { inner: s, parent, allocator: self.allocator }.fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSTypePredicateName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSModuleDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn id(&self) -> &AstNode<'a, 'b, TSModuleDeclarationName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSModuleDeclaration(transmute_self(self))),
        })
    }

    pub fn body(&self) -> Option<&AstNode<'a, 'b, TSModuleDeclarationBody<'a>>> {
        self.allocator
            .alloc(self.inner.body.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSModuleDeclaration(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn kind(&self) -> TSModuleDeclarationKind {
        self.inner.kind
    }

    pub fn declare(&self) -> bool {
        self.inner.declare
    }
}

impl<'a, 'b> AstNode<'a, 'b, TSModuleDeclarationName<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            TSModuleDeclarationName::Identifier(s) => {
                AstNodes::BindingIdentifier(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSModuleDeclarationName::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSModuleDeclarationName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSModuleDeclarationName::Identifier(s) => {
                AstNode::<'a, 'b, BindingIdentifier> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            TSModuleDeclarationName::StringLiteral(s) => {
                AstNode::<'a, 'b, StringLiteral> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSModuleDeclarationName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a, 'b> AstNode<'a, 'b, TSModuleDeclarationBody<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            TSModuleDeclarationBody::TSModuleDeclaration(s) => {
                AstNodes::TSModuleDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            TSModuleDeclarationBody::TSModuleBlock(s) => {
                AstNodes::TSModuleBlock(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSModuleDeclarationBody<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSModuleDeclarationBody::TSModuleDeclaration(s) => {
                AstNode::<'a, 'b, TSModuleDeclaration> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            TSModuleDeclarationBody::TSModuleBlock(s) => {
                AstNode::<'a, 'b, TSModuleBlock> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSModuleDeclarationBody<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSModuleBlock<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn directives(&self) -> &AstNode<'a, 'b, Vec<'a, Directive<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSModuleBlock(transmute_self(self))),
        })
    }

    pub fn body(&self) -> &AstNode<'a, 'b, Vec<'a, Statement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSModuleBlock(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeLiteral<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn members(&self) -> &AstNode<'a, 'b, Vec<'a, TSSignature<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.members,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeLiteral(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSInferType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_parameter(&self) -> &AstNode<'a, 'b, TSTypeParameter<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.type_parameter.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInferType(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeQuery<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expr_name(&self) -> &AstNode<'a, 'b, TSTypeQueryExprName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expr_name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeQuery(transmute_self(self))),
        })
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation<'a>>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeQuery(transmute_self(self))),
            }))
            .as_ref()
    }
}

impl<'a, 'b> AstNode<'a, 'b, TSTypeQueryExprName<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.parent;
        let node = match self.inner {
            TSTypeQueryExprName::TSImportType(s) => {
                AstNodes::TSImportType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            it @ match_ts_type_name!(TSTypeQueryExprName) => {
                AstNodes::TSTypeName(self.allocator.alloc(AstNode {
                    inner: it.to_ts_type_name(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSTypeQueryExprName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent;
        match self.inner {
            TSTypeQueryExprName::TSImportType(s) => {
                AstNode::<'a, 'b, TSImportType> { inner: s, parent, allocator: self.allocator }
                    .fmt(f)
            }
            it @ match_ts_type_name!(TSTypeQueryExprName) => AstNode::<'a, 'b, TSTypeName> {
                inner: it.to_ts_type_name(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSTypeQueryExprName<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSImportType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn argument(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSImportType(transmute_self(self))),
        })
    }

    pub fn options(&self) -> Option<&AstNode<'a, 'b, ObjectExpression<'a>>> {
        self.allocator
            .alloc(self.inner.options.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSImportType(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn qualifier(&self) -> Option<&AstNode<'a, 'b, TSTypeName<'a>>> {
        self.allocator
            .alloc(self.inner.qualifier.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSImportType(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn type_arguments(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterInstantiation<'a>>> {
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSImportType(transmute_self(self))),
            }))
            .as_ref()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSFunctionType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration<'a>>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
            }))
            .as_ref()
    }

    pub fn this_param(&self) -> Option<&AstNode<'a, 'b, TSThisParameter<'a>>> {
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
            }))
            .as_ref()
    }

    pub fn params(&self) -> &AstNode<'a, 'b, FormalParameters<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn return_type(&self) -> &AstNode<'a, 'b, TSTypeAnnotation<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.return_type.as_ref(),
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSConstructorType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn r#abstract(&self) -> bool {
        self.inner.r#abstract
    }

    pub fn type_parameters(&self) -> Option<&AstNode<'a, 'b, TSTypeParameterDeclaration<'a>>> {
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
            }))
            .as_ref()
    }

    pub fn params(&self) -> &AstNode<'a, 'b, FormalParameters<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn return_type(&self) -> &AstNode<'a, 'b, TSTypeAnnotation<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.return_type.as_ref(),
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSMappedType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_parameter(&self) -> &AstNode<'a, 'b, TSTypeParameter<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.type_parameter.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSMappedType(transmute_self(self))),
        })
    }

    pub fn name_type(&self) -> Option<&AstNode<'a, 'b, TSType<'a>>> {
        self.allocator
            .alloc(self.inner.name_type.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMappedType(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn type_annotation(&self) -> Option<&AstNode<'a, 'b, TSType<'a>>> {
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMappedType(transmute_self(self))),
            }))
            .as_ref()
    }

    pub fn optional(&self) -> Option<TSMappedTypeModifierOperator> {
        self.inner.optional
    }

    pub fn readonly(&self) -> Option<TSMappedTypeModifierOperator> {
        self.inner.readonly
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTemplateLiteralType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn quasis(&self) -> &AstNode<'a, 'b, Vec<'a, TemplateElement<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasis,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTemplateLiteralType(transmute_self(self))),
        })
    }

    pub fn types(&self) -> &AstNode<'a, 'b, Vec<'a, TSType<'a>>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTemplateLiteralType(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSAsExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSAsExpression(transmute_self(self))),
        })
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSAsExpression(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSSatisfiesExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSSatisfiesExpression(transmute_self(self))),
        })
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSSatisfiesExpression(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSTypeAssertion<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAssertion(transmute_self(self))),
        })
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAssertion(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSImportEqualsDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn id(&self) -> &AstNode<'a, 'b, BindingIdentifier<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSImportEqualsDeclaration(transmute_self(self))),
        })
    }

    pub fn module_reference(&self) -> &AstNode<'a, 'b, TSModuleReference<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.module_reference,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSImportEqualsDeclaration(transmute_self(self))),
        })
    }

    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }
}

impl<'a, 'b> AstNode<'a, 'b, TSModuleReference<'a>> {
    pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
        let parent = self.allocator.alloc(AstNodes::TSModuleReference(transmute_self(self)));
        let node = match self.inner {
            TSModuleReference::ExternalModuleReference(s) => {
                AstNodes::TSExternalModuleReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                }))
            }
            it @ match_ts_type_name!(TSModuleReference) => {
                AstNodes::TSTypeName(self.allocator.alloc(AstNode {
                    inner: it.to_ts_type_name(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}
impl<'a, 'b> FormatWrite<'a> for AstNode<'a, 'b, TSModuleReference<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.allocator.alloc(AstNodes::TSModuleReference(transmute_self(self)));
        match self.inner {
            TSModuleReference::ExternalModuleReference(s) => {
                AstNode::<'a, 'b, TSExternalModuleReference> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }
                .fmt(f)
            }
            it @ match_ts_type_name!(TSModuleReference) => AstNode::<'a, 'b, TSTypeName> {
                inner: it.to_ts_type_name(),
                parent,
                allocator: self.allocator,
            }
            .fmt(f),
        }
    }
}
impl<'a, 'b> GetSpan for AstNode<'a, 'b, TSModuleReference<'a>> {
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSExternalModuleReference<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, StringLiteral<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSExternalModuleReference(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSNonNullExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSNonNullExpression(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, Decorator<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Decorator(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSExportAssignment<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSExportAssignment(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSNamespaceExportDeclaration<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn id(&self) -> &AstNode<'a, 'b, IdentifierName<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.parent,
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, TSInstantiationExpression<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn expression(&self) -> &AstNode<'a, 'b, Expression<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInstantiationExpression(transmute_self(self))),
        })
    }

    pub fn type_arguments(&self) -> &AstNode<'a, 'b, TSTypeParameterInstantiation<'a>> {
        self.allocator.alloc(AstNode {
            inner: self.inner.type_arguments.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInstantiationExpression(transmute_self(self))),
        })
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSDocNullableType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn postfix(&self) -> bool {
        self.inner.postfix
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSDocNonNullableType<'a>> {
    pub fn span(&self) -> Span {
        self.inner.span
    }

    pub fn type_annotation(&self) -> &AstNode<'a, 'b, TSType<'a>> {
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.parent,
        })
    }

    pub fn postfix(&self) -> bool {
        self.inner.postfix
    }
}
impl<'a, 'b> AstNode<'a, 'b, JSDocUnknownType> {
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
