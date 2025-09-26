// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter/ast_nodes.rs`.

#![expect(clippy::elidable_lifetime_names, clippy::match_same_arms)]
use std::{fmt, mem::transmute, ops::Deref};

use oxc_allocator::{Allocator, Box, Vec};
use oxc_ast::ast::*;
use oxc_span::{GetSpan, SPAN};

use crate::{
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        trivia::{format_leading_comments, format_trailing_comments},
    },
    parentheses::NeedsParentheses,
    write::FormatWrite,
};

#[inline]
pub(super) fn transmute_self<'a, T>(s: &AstNode<'a, T>) -> &'a AstNode<'a, T> {
    /// * SAFETY: `s` is already allocated in Arena, so transmute from `&` to `&'a` is safe.
    unsafe {
        transmute(s)
    }
}

pub enum AstNodes<'a> {
    Dummy(),
    Program(&'a AstNode<'a, Program<'a>>),
    IdentifierName(&'a AstNode<'a, IdentifierName<'a>>),
    IdentifierReference(&'a AstNode<'a, IdentifierReference<'a>>),
    BindingIdentifier(&'a AstNode<'a, BindingIdentifier<'a>>),
    LabelIdentifier(&'a AstNode<'a, LabelIdentifier<'a>>),
    ThisExpression(&'a AstNode<'a, ThisExpression>),
    ArrayExpression(&'a AstNode<'a, ArrayExpression<'a>>),
    Elision(&'a AstNode<'a, Elision>),
    ObjectExpression(&'a AstNode<'a, ObjectExpression<'a>>),
    ObjectProperty(&'a AstNode<'a, ObjectProperty<'a>>),
    TemplateLiteral(&'a AstNode<'a, TemplateLiteral<'a>>),
    TaggedTemplateExpression(&'a AstNode<'a, TaggedTemplateExpression<'a>>),
    TemplateElement(&'a AstNode<'a, TemplateElement<'a>>),
    ComputedMemberExpression(&'a AstNode<'a, ComputedMemberExpression<'a>>),
    StaticMemberExpression(&'a AstNode<'a, StaticMemberExpression<'a>>),
    PrivateFieldExpression(&'a AstNode<'a, PrivateFieldExpression<'a>>),
    CallExpression(&'a AstNode<'a, CallExpression<'a>>),
    NewExpression(&'a AstNode<'a, NewExpression<'a>>),
    MetaProperty(&'a AstNode<'a, MetaProperty<'a>>),
    SpreadElement(&'a AstNode<'a, SpreadElement<'a>>),
    Argument(&'a AstNode<'a, Argument<'a>>),
    UpdateExpression(&'a AstNode<'a, UpdateExpression<'a>>),
    UnaryExpression(&'a AstNode<'a, UnaryExpression<'a>>),
    BinaryExpression(&'a AstNode<'a, BinaryExpression<'a>>),
    PrivateInExpression(&'a AstNode<'a, PrivateInExpression<'a>>),
    LogicalExpression(&'a AstNode<'a, LogicalExpression<'a>>),
    ConditionalExpression(&'a AstNode<'a, ConditionalExpression<'a>>),
    AssignmentExpression(&'a AstNode<'a, AssignmentExpression<'a>>),
    ArrayAssignmentTarget(&'a AstNode<'a, ArrayAssignmentTarget<'a>>),
    ObjectAssignmentTarget(&'a AstNode<'a, ObjectAssignmentTarget<'a>>),
    AssignmentTargetRest(&'a AstNode<'a, AssignmentTargetRest<'a>>),
    AssignmentTargetWithDefault(&'a AstNode<'a, AssignmentTargetWithDefault<'a>>),
    AssignmentTargetPropertyIdentifier(&'a AstNode<'a, AssignmentTargetPropertyIdentifier<'a>>),
    AssignmentTargetPropertyProperty(&'a AstNode<'a, AssignmentTargetPropertyProperty<'a>>),
    SequenceExpression(&'a AstNode<'a, SequenceExpression<'a>>),
    Super(&'a AstNode<'a, Super>),
    AwaitExpression(&'a AstNode<'a, AwaitExpression<'a>>),
    ChainExpression(&'a AstNode<'a, ChainExpression<'a>>),
    ParenthesizedExpression(&'a AstNode<'a, ParenthesizedExpression<'a>>),
    Directive(&'a AstNode<'a, Directive<'a>>),
    Hashbang(&'a AstNode<'a, Hashbang<'a>>),
    BlockStatement(&'a AstNode<'a, BlockStatement<'a>>),
    VariableDeclaration(&'a AstNode<'a, VariableDeclaration<'a>>),
    VariableDeclarator(&'a AstNode<'a, VariableDeclarator<'a>>),
    EmptyStatement(&'a AstNode<'a, EmptyStatement>),
    ExpressionStatement(&'a AstNode<'a, ExpressionStatement<'a>>),
    IfStatement(&'a AstNode<'a, IfStatement<'a>>),
    DoWhileStatement(&'a AstNode<'a, DoWhileStatement<'a>>),
    WhileStatement(&'a AstNode<'a, WhileStatement<'a>>),
    ForStatement(&'a AstNode<'a, ForStatement<'a>>),
    ForInStatement(&'a AstNode<'a, ForInStatement<'a>>),
    ForOfStatement(&'a AstNode<'a, ForOfStatement<'a>>),
    ContinueStatement(&'a AstNode<'a, ContinueStatement<'a>>),
    BreakStatement(&'a AstNode<'a, BreakStatement<'a>>),
    ReturnStatement(&'a AstNode<'a, ReturnStatement<'a>>),
    WithStatement(&'a AstNode<'a, WithStatement<'a>>),
    SwitchStatement(&'a AstNode<'a, SwitchStatement<'a>>),
    SwitchCase(&'a AstNode<'a, SwitchCase<'a>>),
    LabeledStatement(&'a AstNode<'a, LabeledStatement<'a>>),
    ThrowStatement(&'a AstNode<'a, ThrowStatement<'a>>),
    TryStatement(&'a AstNode<'a, TryStatement<'a>>),
    CatchClause(&'a AstNode<'a, CatchClause<'a>>),
    CatchParameter(&'a AstNode<'a, CatchParameter<'a>>),
    DebuggerStatement(&'a AstNode<'a, DebuggerStatement>),
    AssignmentPattern(&'a AstNode<'a, AssignmentPattern<'a>>),
    ObjectPattern(&'a AstNode<'a, ObjectPattern<'a>>),
    BindingProperty(&'a AstNode<'a, BindingProperty<'a>>),
    ArrayPattern(&'a AstNode<'a, ArrayPattern<'a>>),
    BindingRestElement(&'a AstNode<'a, BindingRestElement<'a>>),
    Function(&'a AstNode<'a, Function<'a>>),
    FormalParameters(&'a AstNode<'a, FormalParameters<'a>>),
    FormalParameter(&'a AstNode<'a, FormalParameter<'a>>),
    FunctionBody(&'a AstNode<'a, FunctionBody<'a>>),
    ArrowFunctionExpression(&'a AstNode<'a, ArrowFunctionExpression<'a>>),
    YieldExpression(&'a AstNode<'a, YieldExpression<'a>>),
    Class(&'a AstNode<'a, Class<'a>>),
    ClassBody(&'a AstNode<'a, ClassBody<'a>>),
    MethodDefinition(&'a AstNode<'a, MethodDefinition<'a>>),
    PropertyDefinition(&'a AstNode<'a, PropertyDefinition<'a>>),
    PrivateIdentifier(&'a AstNode<'a, PrivateIdentifier<'a>>),
    StaticBlock(&'a AstNode<'a, StaticBlock<'a>>),
    AccessorProperty(&'a AstNode<'a, AccessorProperty<'a>>),
    ImportExpression(&'a AstNode<'a, ImportExpression<'a>>),
    ImportDeclaration(&'a AstNode<'a, ImportDeclaration<'a>>),
    ImportSpecifier(&'a AstNode<'a, ImportSpecifier<'a>>),
    ImportDefaultSpecifier(&'a AstNode<'a, ImportDefaultSpecifier<'a>>),
    ImportNamespaceSpecifier(&'a AstNode<'a, ImportNamespaceSpecifier<'a>>),
    WithClause(&'a AstNode<'a, WithClause<'a>>),
    ImportAttribute(&'a AstNode<'a, ImportAttribute<'a>>),
    ExportNamedDeclaration(&'a AstNode<'a, ExportNamedDeclaration<'a>>),
    ExportDefaultDeclaration(&'a AstNode<'a, ExportDefaultDeclaration<'a>>),
    ExportAllDeclaration(&'a AstNode<'a, ExportAllDeclaration<'a>>),
    ExportSpecifier(&'a AstNode<'a, ExportSpecifier<'a>>),
    V8IntrinsicExpression(&'a AstNode<'a, V8IntrinsicExpression<'a>>),
    BooleanLiteral(&'a AstNode<'a, BooleanLiteral>),
    NullLiteral(&'a AstNode<'a, NullLiteral>),
    NumericLiteral(&'a AstNode<'a, NumericLiteral<'a>>),
    StringLiteral(&'a AstNode<'a, StringLiteral<'a>>),
    BigIntLiteral(&'a AstNode<'a, BigIntLiteral<'a>>),
    RegExpLiteral(&'a AstNode<'a, RegExpLiteral<'a>>),
    JSXElement(&'a AstNode<'a, JSXElement<'a>>),
    JSXOpeningElement(&'a AstNode<'a, JSXOpeningElement<'a>>),
    JSXClosingElement(&'a AstNode<'a, JSXClosingElement<'a>>),
    JSXFragment(&'a AstNode<'a, JSXFragment<'a>>),
    JSXOpeningFragment(&'a AstNode<'a, JSXOpeningFragment>),
    JSXClosingFragment(&'a AstNode<'a, JSXClosingFragment>),
    JSXNamespacedName(&'a AstNode<'a, JSXNamespacedName<'a>>),
    JSXMemberExpression(&'a AstNode<'a, JSXMemberExpression<'a>>),
    JSXExpressionContainer(&'a AstNode<'a, JSXExpressionContainer<'a>>),
    JSXEmptyExpression(&'a AstNode<'a, JSXEmptyExpression>),
    JSXAttribute(&'a AstNode<'a, JSXAttribute<'a>>),
    JSXSpreadAttribute(&'a AstNode<'a, JSXSpreadAttribute<'a>>),
    JSXIdentifier(&'a AstNode<'a, JSXIdentifier<'a>>),
    JSXSpreadChild(&'a AstNode<'a, JSXSpreadChild<'a>>),
    JSXText(&'a AstNode<'a, JSXText<'a>>),
    TSThisParameter(&'a AstNode<'a, TSThisParameter<'a>>),
    TSEnumDeclaration(&'a AstNode<'a, TSEnumDeclaration<'a>>),
    TSEnumBody(&'a AstNode<'a, TSEnumBody<'a>>),
    TSEnumMember(&'a AstNode<'a, TSEnumMember<'a>>),
    TSTypeAnnotation(&'a AstNode<'a, TSTypeAnnotation<'a>>),
    TSLiteralType(&'a AstNode<'a, TSLiteralType<'a>>),
    TSConditionalType(&'a AstNode<'a, TSConditionalType<'a>>),
    TSUnionType(&'a AstNode<'a, TSUnionType<'a>>),
    TSIntersectionType(&'a AstNode<'a, TSIntersectionType<'a>>),
    TSParenthesizedType(&'a AstNode<'a, TSParenthesizedType<'a>>),
    TSTypeOperator(&'a AstNode<'a, TSTypeOperator<'a>>),
    TSArrayType(&'a AstNode<'a, TSArrayType<'a>>),
    TSIndexedAccessType(&'a AstNode<'a, TSIndexedAccessType<'a>>),
    TSTupleType(&'a AstNode<'a, TSTupleType<'a>>),
    TSNamedTupleMember(&'a AstNode<'a, TSNamedTupleMember<'a>>),
    TSOptionalType(&'a AstNode<'a, TSOptionalType<'a>>),
    TSRestType(&'a AstNode<'a, TSRestType<'a>>),
    TSAnyKeyword(&'a AstNode<'a, TSAnyKeyword>),
    TSStringKeyword(&'a AstNode<'a, TSStringKeyword>),
    TSBooleanKeyword(&'a AstNode<'a, TSBooleanKeyword>),
    TSNumberKeyword(&'a AstNode<'a, TSNumberKeyword>),
    TSNeverKeyword(&'a AstNode<'a, TSNeverKeyword>),
    TSIntrinsicKeyword(&'a AstNode<'a, TSIntrinsicKeyword>),
    TSUnknownKeyword(&'a AstNode<'a, TSUnknownKeyword>),
    TSNullKeyword(&'a AstNode<'a, TSNullKeyword>),
    TSUndefinedKeyword(&'a AstNode<'a, TSUndefinedKeyword>),
    TSVoidKeyword(&'a AstNode<'a, TSVoidKeyword>),
    TSSymbolKeyword(&'a AstNode<'a, TSSymbolKeyword>),
    TSThisType(&'a AstNode<'a, TSThisType>),
    TSObjectKeyword(&'a AstNode<'a, TSObjectKeyword>),
    TSBigIntKeyword(&'a AstNode<'a, TSBigIntKeyword>),
    TSTypeReference(&'a AstNode<'a, TSTypeReference<'a>>),
    TSQualifiedName(&'a AstNode<'a, TSQualifiedName<'a>>),
    TSTypeParameterInstantiation(&'a AstNode<'a, TSTypeParameterInstantiation<'a>>),
    TSTypeParameter(&'a AstNode<'a, TSTypeParameter<'a>>),
    TSTypeParameterDeclaration(&'a AstNode<'a, TSTypeParameterDeclaration<'a>>),
    TSTypeAliasDeclaration(&'a AstNode<'a, TSTypeAliasDeclaration<'a>>),
    TSClassImplements(&'a AstNode<'a, TSClassImplements<'a>>),
    TSInterfaceDeclaration(&'a AstNode<'a, TSInterfaceDeclaration<'a>>),
    TSInterfaceBody(&'a AstNode<'a, TSInterfaceBody<'a>>),
    TSPropertySignature(&'a AstNode<'a, TSPropertySignature<'a>>),
    TSIndexSignature(&'a AstNode<'a, TSIndexSignature<'a>>),
    TSCallSignatureDeclaration(&'a AstNode<'a, TSCallSignatureDeclaration<'a>>),
    TSMethodSignature(&'a AstNode<'a, TSMethodSignature<'a>>),
    TSConstructSignatureDeclaration(&'a AstNode<'a, TSConstructSignatureDeclaration<'a>>),
    TSIndexSignatureName(&'a AstNode<'a, TSIndexSignatureName<'a>>),
    TSInterfaceHeritage(&'a AstNode<'a, TSInterfaceHeritage<'a>>),
    TSTypePredicate(&'a AstNode<'a, TSTypePredicate<'a>>),
    TSModuleDeclaration(&'a AstNode<'a, TSModuleDeclaration<'a>>),
    TSModuleBlock(&'a AstNode<'a, TSModuleBlock<'a>>),
    TSTypeLiteral(&'a AstNode<'a, TSTypeLiteral<'a>>),
    TSInferType(&'a AstNode<'a, TSInferType<'a>>),
    TSTypeQuery(&'a AstNode<'a, TSTypeQuery<'a>>),
    TSImportType(&'a AstNode<'a, TSImportType<'a>>),
    TSImportTypeQualifiedName(&'a AstNode<'a, TSImportTypeQualifiedName<'a>>),
    TSFunctionType(&'a AstNode<'a, TSFunctionType<'a>>),
    TSConstructorType(&'a AstNode<'a, TSConstructorType<'a>>),
    TSMappedType(&'a AstNode<'a, TSMappedType<'a>>),
    TSTemplateLiteralType(&'a AstNode<'a, TSTemplateLiteralType<'a>>),
    TSAsExpression(&'a AstNode<'a, TSAsExpression<'a>>),
    TSSatisfiesExpression(&'a AstNode<'a, TSSatisfiesExpression<'a>>),
    TSTypeAssertion(&'a AstNode<'a, TSTypeAssertion<'a>>),
    TSImportEqualsDeclaration(&'a AstNode<'a, TSImportEqualsDeclaration<'a>>),
    TSExternalModuleReference(&'a AstNode<'a, TSExternalModuleReference<'a>>),
    TSNonNullExpression(&'a AstNode<'a, TSNonNullExpression<'a>>),
    Decorator(&'a AstNode<'a, Decorator<'a>>),
    TSExportAssignment(&'a AstNode<'a, TSExportAssignment<'a>>),
    TSNamespaceExportDeclaration(&'a AstNode<'a, TSNamespaceExportDeclaration<'a>>),
    TSInstantiationExpression(&'a AstNode<'a, TSInstantiationExpression<'a>>),
    JSDocNullableType(&'a AstNode<'a, JSDocNullableType<'a>>),
    JSDocNonNullableType(&'a AstNode<'a, JSDocNonNullableType<'a>>),
    JSDocUnknownType(&'a AstNode<'a, JSDocUnknownType>),
}

#[derive(Debug, Copy, Clone)]
pub enum SiblingNode<'a> {
    Program(&'a Program<'a>),
    IdentifierName(&'a IdentifierName<'a>),
    IdentifierReference(&'a IdentifierReference<'a>),
    BindingIdentifier(&'a BindingIdentifier<'a>),
    LabelIdentifier(&'a LabelIdentifier<'a>),
    ThisExpression(&'a ThisExpression),
    ArrayExpression(&'a ArrayExpression<'a>),
    Elision(&'a Elision),
    ObjectExpression(&'a ObjectExpression<'a>),
    ObjectProperty(&'a ObjectProperty<'a>),
    TemplateLiteral(&'a TemplateLiteral<'a>),
    TaggedTemplateExpression(&'a TaggedTemplateExpression<'a>),
    TemplateElement(&'a TemplateElement<'a>),
    ComputedMemberExpression(&'a ComputedMemberExpression<'a>),
    StaticMemberExpression(&'a StaticMemberExpression<'a>),
    PrivateFieldExpression(&'a PrivateFieldExpression<'a>),
    CallExpression(&'a CallExpression<'a>),
    NewExpression(&'a NewExpression<'a>),
    MetaProperty(&'a MetaProperty<'a>),
    SpreadElement(&'a SpreadElement<'a>),
    UpdateExpression(&'a UpdateExpression<'a>),
    UnaryExpression(&'a UnaryExpression<'a>),
    BinaryExpression(&'a BinaryExpression<'a>),
    PrivateInExpression(&'a PrivateInExpression<'a>),
    LogicalExpression(&'a LogicalExpression<'a>),
    ConditionalExpression(&'a ConditionalExpression<'a>),
    AssignmentExpression(&'a AssignmentExpression<'a>),
    ArrayAssignmentTarget(&'a ArrayAssignmentTarget<'a>),
    ObjectAssignmentTarget(&'a ObjectAssignmentTarget<'a>),
    AssignmentTargetRest(&'a AssignmentTargetRest<'a>),
    AssignmentTargetWithDefault(&'a AssignmentTargetWithDefault<'a>),
    AssignmentTargetPropertyIdentifier(&'a AssignmentTargetPropertyIdentifier<'a>),
    AssignmentTargetPropertyProperty(&'a AssignmentTargetPropertyProperty<'a>),
    SequenceExpression(&'a SequenceExpression<'a>),
    Super(&'a Super),
    AwaitExpression(&'a AwaitExpression<'a>),
    ChainExpression(&'a ChainExpression<'a>),
    ParenthesizedExpression(&'a ParenthesizedExpression<'a>),
    Directive(&'a Directive<'a>),
    Hashbang(&'a Hashbang<'a>),
    BlockStatement(&'a BlockStatement<'a>),
    VariableDeclaration(&'a VariableDeclaration<'a>),
    VariableDeclarator(&'a VariableDeclarator<'a>),
    EmptyStatement(&'a EmptyStatement),
    ExpressionStatement(&'a ExpressionStatement<'a>),
    IfStatement(&'a IfStatement<'a>),
    DoWhileStatement(&'a DoWhileStatement<'a>),
    WhileStatement(&'a WhileStatement<'a>),
    ForStatement(&'a ForStatement<'a>),
    ForInStatement(&'a ForInStatement<'a>),
    ForOfStatement(&'a ForOfStatement<'a>),
    ContinueStatement(&'a ContinueStatement<'a>),
    BreakStatement(&'a BreakStatement<'a>),
    ReturnStatement(&'a ReturnStatement<'a>),
    WithStatement(&'a WithStatement<'a>),
    SwitchStatement(&'a SwitchStatement<'a>),
    SwitchCase(&'a SwitchCase<'a>),
    LabeledStatement(&'a LabeledStatement<'a>),
    ThrowStatement(&'a ThrowStatement<'a>),
    TryStatement(&'a TryStatement<'a>),
    CatchClause(&'a CatchClause<'a>),
    CatchParameter(&'a CatchParameter<'a>),
    DebuggerStatement(&'a DebuggerStatement),
    BindingPattern(&'a BindingPattern<'a>),
    AssignmentPattern(&'a AssignmentPattern<'a>),
    ObjectPattern(&'a ObjectPattern<'a>),
    BindingProperty(&'a BindingProperty<'a>),
    ArrayPattern(&'a ArrayPattern<'a>),
    BindingRestElement(&'a BindingRestElement<'a>),
    Function(&'a Function<'a>),
    FormalParameters(&'a FormalParameters<'a>),
    FormalParameter(&'a FormalParameter<'a>),
    FunctionBody(&'a FunctionBody<'a>),
    ArrowFunctionExpression(&'a ArrowFunctionExpression<'a>),
    YieldExpression(&'a YieldExpression<'a>),
    Class(&'a Class<'a>),
    ClassBody(&'a ClassBody<'a>),
    MethodDefinition(&'a MethodDefinition<'a>),
    PropertyDefinition(&'a PropertyDefinition<'a>),
    PrivateIdentifier(&'a PrivateIdentifier<'a>),
    StaticBlock(&'a StaticBlock<'a>),
    AccessorProperty(&'a AccessorProperty<'a>),
    ImportExpression(&'a ImportExpression<'a>),
    ImportDeclaration(&'a ImportDeclaration<'a>),
    ImportSpecifier(&'a ImportSpecifier<'a>),
    ImportDefaultSpecifier(&'a ImportDefaultSpecifier<'a>),
    ImportNamespaceSpecifier(&'a ImportNamespaceSpecifier<'a>),
    WithClause(&'a WithClause<'a>),
    ImportAttribute(&'a ImportAttribute<'a>),
    ExportNamedDeclaration(&'a ExportNamedDeclaration<'a>),
    ExportDefaultDeclaration(&'a ExportDefaultDeclaration<'a>),
    ExportAllDeclaration(&'a ExportAllDeclaration<'a>),
    ExportSpecifier(&'a ExportSpecifier<'a>),
    V8IntrinsicExpression(&'a V8IntrinsicExpression<'a>),
    BooleanLiteral(&'a BooleanLiteral),
    NullLiteral(&'a NullLiteral),
    NumericLiteral(&'a NumericLiteral<'a>),
    StringLiteral(&'a StringLiteral<'a>),
    BigIntLiteral(&'a BigIntLiteral<'a>),
    RegExpLiteral(&'a RegExpLiteral<'a>),
    JSXElement(&'a JSXElement<'a>),
    JSXOpeningElement(&'a JSXOpeningElement<'a>),
    JSXClosingElement(&'a JSXClosingElement<'a>),
    JSXFragment(&'a JSXFragment<'a>),
    JSXOpeningFragment(&'a JSXOpeningFragment),
    JSXClosingFragment(&'a JSXClosingFragment),
    JSXNamespacedName(&'a JSXNamespacedName<'a>),
    JSXMemberExpression(&'a JSXMemberExpression<'a>),
    JSXExpressionContainer(&'a JSXExpressionContainer<'a>),
    JSXEmptyExpression(&'a JSXEmptyExpression),
    JSXAttribute(&'a JSXAttribute<'a>),
    JSXSpreadAttribute(&'a JSXSpreadAttribute<'a>),
    JSXIdentifier(&'a JSXIdentifier<'a>),
    JSXSpreadChild(&'a JSXSpreadChild<'a>),
    JSXText(&'a JSXText<'a>),
    TSThisParameter(&'a TSThisParameter<'a>),
    TSEnumDeclaration(&'a TSEnumDeclaration<'a>),
    TSEnumBody(&'a TSEnumBody<'a>),
    TSEnumMember(&'a TSEnumMember<'a>),
    TSTypeAnnotation(&'a TSTypeAnnotation<'a>),
    TSLiteralType(&'a TSLiteralType<'a>),
    TSConditionalType(&'a TSConditionalType<'a>),
    TSUnionType(&'a TSUnionType<'a>),
    TSIntersectionType(&'a TSIntersectionType<'a>),
    TSParenthesizedType(&'a TSParenthesizedType<'a>),
    TSTypeOperator(&'a TSTypeOperator<'a>),
    TSArrayType(&'a TSArrayType<'a>),
    TSIndexedAccessType(&'a TSIndexedAccessType<'a>),
    TSTupleType(&'a TSTupleType<'a>),
    TSNamedTupleMember(&'a TSNamedTupleMember<'a>),
    TSOptionalType(&'a TSOptionalType<'a>),
    TSRestType(&'a TSRestType<'a>),
    TSAnyKeyword(&'a TSAnyKeyword),
    TSStringKeyword(&'a TSStringKeyword),
    TSBooleanKeyword(&'a TSBooleanKeyword),
    TSNumberKeyword(&'a TSNumberKeyword),
    TSNeverKeyword(&'a TSNeverKeyword),
    TSIntrinsicKeyword(&'a TSIntrinsicKeyword),
    TSUnknownKeyword(&'a TSUnknownKeyword),
    TSNullKeyword(&'a TSNullKeyword),
    TSUndefinedKeyword(&'a TSUndefinedKeyword),
    TSVoidKeyword(&'a TSVoidKeyword),
    TSSymbolKeyword(&'a TSSymbolKeyword),
    TSThisType(&'a TSThisType),
    TSObjectKeyword(&'a TSObjectKeyword),
    TSBigIntKeyword(&'a TSBigIntKeyword),
    TSTypeReference(&'a TSTypeReference<'a>),
    TSQualifiedName(&'a TSQualifiedName<'a>),
    TSTypeParameterInstantiation(&'a TSTypeParameterInstantiation<'a>),
    TSTypeParameter(&'a TSTypeParameter<'a>),
    TSTypeParameterDeclaration(&'a TSTypeParameterDeclaration<'a>),
    TSTypeAliasDeclaration(&'a TSTypeAliasDeclaration<'a>),
    TSClassImplements(&'a TSClassImplements<'a>),
    TSInterfaceDeclaration(&'a TSInterfaceDeclaration<'a>),
    TSInterfaceBody(&'a TSInterfaceBody<'a>),
    TSPropertySignature(&'a TSPropertySignature<'a>),
    TSIndexSignature(&'a TSIndexSignature<'a>),
    TSCallSignatureDeclaration(&'a TSCallSignatureDeclaration<'a>),
    TSMethodSignature(&'a TSMethodSignature<'a>),
    TSConstructSignatureDeclaration(&'a TSConstructSignatureDeclaration<'a>),
    TSIndexSignatureName(&'a TSIndexSignatureName<'a>),
    TSInterfaceHeritage(&'a TSInterfaceHeritage<'a>),
    TSTypePredicate(&'a TSTypePredicate<'a>),
    TSModuleDeclaration(&'a TSModuleDeclaration<'a>),
    TSModuleBlock(&'a TSModuleBlock<'a>),
    TSTypeLiteral(&'a TSTypeLiteral<'a>),
    TSInferType(&'a TSInferType<'a>),
    TSTypeQuery(&'a TSTypeQuery<'a>),
    TSImportType(&'a TSImportType<'a>),
    TSImportTypeQualifiedName(&'a TSImportTypeQualifiedName<'a>),
    TSFunctionType(&'a TSFunctionType<'a>),
    TSConstructorType(&'a TSConstructorType<'a>),
    TSMappedType(&'a TSMappedType<'a>),
    TSTemplateLiteralType(&'a TSTemplateLiteralType<'a>),
    TSAsExpression(&'a TSAsExpression<'a>),
    TSSatisfiesExpression(&'a TSSatisfiesExpression<'a>),
    TSTypeAssertion(&'a TSTypeAssertion<'a>),
    TSImportEqualsDeclaration(&'a TSImportEqualsDeclaration<'a>),
    TSExternalModuleReference(&'a TSExternalModuleReference<'a>),
    TSNonNullExpression(&'a TSNonNullExpression<'a>),
    Decorator(&'a Decorator<'a>),
    TSExportAssignment(&'a TSExportAssignment<'a>),
    TSNamespaceExportDeclaration(&'a TSNamespaceExportDeclaration<'a>),
    TSInstantiationExpression(&'a TSInstantiationExpression<'a>),
    JSDocNullableType(&'a JSDocNullableType<'a>),
    JSDocNonNullableType(&'a JSDocNonNullableType<'a>),
    JSDocUnknownType(&'a JSDocUnknownType),
}

impl<'a> From<&'a Program<'a>> for SiblingNode<'a> {
    fn from(node: &'a Program<'a>) -> Self {
        SiblingNode::Program(node)
    }
}

impl<'a> From<&'a IdentifierName<'a>> for SiblingNode<'a> {
    fn from(node: &'a IdentifierName<'a>) -> Self {
        SiblingNode::IdentifierName(node)
    }
}

impl<'a> From<&'a IdentifierReference<'a>> for SiblingNode<'a> {
    fn from(node: &'a IdentifierReference<'a>) -> Self {
        SiblingNode::IdentifierReference(node)
    }
}

impl<'a> From<&'a BindingIdentifier<'a>> for SiblingNode<'a> {
    fn from(node: &'a BindingIdentifier<'a>) -> Self {
        SiblingNode::BindingIdentifier(node)
    }
}

impl<'a> From<&'a LabelIdentifier<'a>> for SiblingNode<'a> {
    fn from(node: &'a LabelIdentifier<'a>) -> Self {
        SiblingNode::LabelIdentifier(node)
    }
}

impl<'a> From<&'a ThisExpression> for SiblingNode<'a> {
    fn from(node: &'a ThisExpression) -> Self {
        SiblingNode::ThisExpression(node)
    }
}

impl<'a> From<&'a ArrayExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a ArrayExpression<'a>) -> Self {
        SiblingNode::ArrayExpression(node)
    }
}

impl<'a> From<&'a Elision> for SiblingNode<'a> {
    fn from(node: &'a Elision) -> Self {
        SiblingNode::Elision(node)
    }
}

impl<'a> From<&'a ObjectExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a ObjectExpression<'a>) -> Self {
        SiblingNode::ObjectExpression(node)
    }
}

impl<'a> From<&'a ObjectProperty<'a>> for SiblingNode<'a> {
    fn from(node: &'a ObjectProperty<'a>) -> Self {
        SiblingNode::ObjectProperty(node)
    }
}

impl<'a> From<&'a TemplateLiteral<'a>> for SiblingNode<'a> {
    fn from(node: &'a TemplateLiteral<'a>) -> Self {
        SiblingNode::TemplateLiteral(node)
    }
}

impl<'a> From<&'a TaggedTemplateExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a TaggedTemplateExpression<'a>) -> Self {
        SiblingNode::TaggedTemplateExpression(node)
    }
}

impl<'a> From<&'a TemplateElement<'a>> for SiblingNode<'a> {
    fn from(node: &'a TemplateElement<'a>) -> Self {
        SiblingNode::TemplateElement(node)
    }
}

impl<'a> From<&'a ComputedMemberExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a ComputedMemberExpression<'a>) -> Self {
        SiblingNode::ComputedMemberExpression(node)
    }
}

impl<'a> From<&'a StaticMemberExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a StaticMemberExpression<'a>) -> Self {
        SiblingNode::StaticMemberExpression(node)
    }
}

impl<'a> From<&'a PrivateFieldExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a PrivateFieldExpression<'a>) -> Self {
        SiblingNode::PrivateFieldExpression(node)
    }
}

impl<'a> From<&'a CallExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a CallExpression<'a>) -> Self {
        SiblingNode::CallExpression(node)
    }
}

impl<'a> From<&'a NewExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a NewExpression<'a>) -> Self {
        SiblingNode::NewExpression(node)
    }
}

impl<'a> From<&'a MetaProperty<'a>> for SiblingNode<'a> {
    fn from(node: &'a MetaProperty<'a>) -> Self {
        SiblingNode::MetaProperty(node)
    }
}

impl<'a> From<&'a SpreadElement<'a>> for SiblingNode<'a> {
    fn from(node: &'a SpreadElement<'a>) -> Self {
        SiblingNode::SpreadElement(node)
    }
}

impl<'a> From<&'a UpdateExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a UpdateExpression<'a>) -> Self {
        SiblingNode::UpdateExpression(node)
    }
}

impl<'a> From<&'a UnaryExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a UnaryExpression<'a>) -> Self {
        SiblingNode::UnaryExpression(node)
    }
}

impl<'a> From<&'a BinaryExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a BinaryExpression<'a>) -> Self {
        SiblingNode::BinaryExpression(node)
    }
}

impl<'a> From<&'a PrivateInExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a PrivateInExpression<'a>) -> Self {
        SiblingNode::PrivateInExpression(node)
    }
}

impl<'a> From<&'a LogicalExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a LogicalExpression<'a>) -> Self {
        SiblingNode::LogicalExpression(node)
    }
}

impl<'a> From<&'a ConditionalExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a ConditionalExpression<'a>) -> Self {
        SiblingNode::ConditionalExpression(node)
    }
}

impl<'a> From<&'a AssignmentExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a AssignmentExpression<'a>) -> Self {
        SiblingNode::AssignmentExpression(node)
    }
}

impl<'a> From<&'a ArrayAssignmentTarget<'a>> for SiblingNode<'a> {
    fn from(node: &'a ArrayAssignmentTarget<'a>) -> Self {
        SiblingNode::ArrayAssignmentTarget(node)
    }
}

impl<'a> From<&'a ObjectAssignmentTarget<'a>> for SiblingNode<'a> {
    fn from(node: &'a ObjectAssignmentTarget<'a>) -> Self {
        SiblingNode::ObjectAssignmentTarget(node)
    }
}

impl<'a> From<&'a AssignmentTargetRest<'a>> for SiblingNode<'a> {
    fn from(node: &'a AssignmentTargetRest<'a>) -> Self {
        SiblingNode::AssignmentTargetRest(node)
    }
}

impl<'a> From<&'a AssignmentTargetWithDefault<'a>> for SiblingNode<'a> {
    fn from(node: &'a AssignmentTargetWithDefault<'a>) -> Self {
        SiblingNode::AssignmentTargetWithDefault(node)
    }
}

impl<'a> From<&'a AssignmentTargetPropertyIdentifier<'a>> for SiblingNode<'a> {
    fn from(node: &'a AssignmentTargetPropertyIdentifier<'a>) -> Self {
        SiblingNode::AssignmentTargetPropertyIdentifier(node)
    }
}

impl<'a> From<&'a AssignmentTargetPropertyProperty<'a>> for SiblingNode<'a> {
    fn from(node: &'a AssignmentTargetPropertyProperty<'a>) -> Self {
        SiblingNode::AssignmentTargetPropertyProperty(node)
    }
}

impl<'a> From<&'a SequenceExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a SequenceExpression<'a>) -> Self {
        SiblingNode::SequenceExpression(node)
    }
}

impl<'a> From<&'a Super> for SiblingNode<'a> {
    fn from(node: &'a Super) -> Self {
        SiblingNode::Super(node)
    }
}

impl<'a> From<&'a AwaitExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a AwaitExpression<'a>) -> Self {
        SiblingNode::AwaitExpression(node)
    }
}

impl<'a> From<&'a ChainExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a ChainExpression<'a>) -> Self {
        SiblingNode::ChainExpression(node)
    }
}

impl<'a> From<&'a ParenthesizedExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a ParenthesizedExpression<'a>) -> Self {
        SiblingNode::ParenthesizedExpression(node)
    }
}

impl<'a> From<&'a Directive<'a>> for SiblingNode<'a> {
    fn from(node: &'a Directive<'a>) -> Self {
        SiblingNode::Directive(node)
    }
}

impl<'a> From<&'a Hashbang<'a>> for SiblingNode<'a> {
    fn from(node: &'a Hashbang<'a>) -> Self {
        SiblingNode::Hashbang(node)
    }
}

impl<'a> From<&'a BlockStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a BlockStatement<'a>) -> Self {
        SiblingNode::BlockStatement(node)
    }
}

impl<'a> From<&'a VariableDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a VariableDeclaration<'a>) -> Self {
        SiblingNode::VariableDeclaration(node)
    }
}

impl<'a> From<&'a VariableDeclarator<'a>> for SiblingNode<'a> {
    fn from(node: &'a VariableDeclarator<'a>) -> Self {
        SiblingNode::VariableDeclarator(node)
    }
}

impl<'a> From<&'a EmptyStatement> for SiblingNode<'a> {
    fn from(node: &'a EmptyStatement) -> Self {
        SiblingNode::EmptyStatement(node)
    }
}

impl<'a> From<&'a ExpressionStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a ExpressionStatement<'a>) -> Self {
        SiblingNode::ExpressionStatement(node)
    }
}

impl<'a> From<&'a IfStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a IfStatement<'a>) -> Self {
        SiblingNode::IfStatement(node)
    }
}

impl<'a> From<&'a DoWhileStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a DoWhileStatement<'a>) -> Self {
        SiblingNode::DoWhileStatement(node)
    }
}

impl<'a> From<&'a WhileStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a WhileStatement<'a>) -> Self {
        SiblingNode::WhileStatement(node)
    }
}

impl<'a> From<&'a ForStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a ForStatement<'a>) -> Self {
        SiblingNode::ForStatement(node)
    }
}

impl<'a> From<&'a ForInStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a ForInStatement<'a>) -> Self {
        SiblingNode::ForInStatement(node)
    }
}

impl<'a> From<&'a ForOfStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a ForOfStatement<'a>) -> Self {
        SiblingNode::ForOfStatement(node)
    }
}

impl<'a> From<&'a ContinueStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a ContinueStatement<'a>) -> Self {
        SiblingNode::ContinueStatement(node)
    }
}

impl<'a> From<&'a BreakStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a BreakStatement<'a>) -> Self {
        SiblingNode::BreakStatement(node)
    }
}

impl<'a> From<&'a ReturnStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a ReturnStatement<'a>) -> Self {
        SiblingNode::ReturnStatement(node)
    }
}

impl<'a> From<&'a WithStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a WithStatement<'a>) -> Self {
        SiblingNode::WithStatement(node)
    }
}

impl<'a> From<&'a SwitchStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a SwitchStatement<'a>) -> Self {
        SiblingNode::SwitchStatement(node)
    }
}

impl<'a> From<&'a SwitchCase<'a>> for SiblingNode<'a> {
    fn from(node: &'a SwitchCase<'a>) -> Self {
        SiblingNode::SwitchCase(node)
    }
}

impl<'a> From<&'a LabeledStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a LabeledStatement<'a>) -> Self {
        SiblingNode::LabeledStatement(node)
    }
}

impl<'a> From<&'a ThrowStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a ThrowStatement<'a>) -> Self {
        SiblingNode::ThrowStatement(node)
    }
}

impl<'a> From<&'a TryStatement<'a>> for SiblingNode<'a> {
    fn from(node: &'a TryStatement<'a>) -> Self {
        SiblingNode::TryStatement(node)
    }
}

impl<'a> From<&'a CatchClause<'a>> for SiblingNode<'a> {
    fn from(node: &'a CatchClause<'a>) -> Self {
        SiblingNode::CatchClause(node)
    }
}

impl<'a> From<&'a CatchParameter<'a>> for SiblingNode<'a> {
    fn from(node: &'a CatchParameter<'a>) -> Self {
        SiblingNode::CatchParameter(node)
    }
}

impl<'a> From<&'a DebuggerStatement> for SiblingNode<'a> {
    fn from(node: &'a DebuggerStatement) -> Self {
        SiblingNode::DebuggerStatement(node)
    }
}

impl<'a> From<&'a BindingPattern<'a>> for SiblingNode<'a> {
    fn from(node: &'a BindingPattern<'a>) -> Self {
        SiblingNode::BindingPattern(node)
    }
}

impl<'a> From<&'a AssignmentPattern<'a>> for SiblingNode<'a> {
    fn from(node: &'a AssignmentPattern<'a>) -> Self {
        SiblingNode::AssignmentPattern(node)
    }
}

impl<'a> From<&'a ObjectPattern<'a>> for SiblingNode<'a> {
    fn from(node: &'a ObjectPattern<'a>) -> Self {
        SiblingNode::ObjectPattern(node)
    }
}

impl<'a> From<&'a BindingProperty<'a>> for SiblingNode<'a> {
    fn from(node: &'a BindingProperty<'a>) -> Self {
        SiblingNode::BindingProperty(node)
    }
}

impl<'a> From<&'a ArrayPattern<'a>> for SiblingNode<'a> {
    fn from(node: &'a ArrayPattern<'a>) -> Self {
        SiblingNode::ArrayPattern(node)
    }
}

impl<'a> From<&'a BindingRestElement<'a>> for SiblingNode<'a> {
    fn from(node: &'a BindingRestElement<'a>) -> Self {
        SiblingNode::BindingRestElement(node)
    }
}

impl<'a> From<&'a Function<'a>> for SiblingNode<'a> {
    fn from(node: &'a Function<'a>) -> Self {
        SiblingNode::Function(node)
    }
}

impl<'a> From<&'a FormalParameters<'a>> for SiblingNode<'a> {
    fn from(node: &'a FormalParameters<'a>) -> Self {
        SiblingNode::FormalParameters(node)
    }
}

impl<'a> From<&'a FormalParameter<'a>> for SiblingNode<'a> {
    fn from(node: &'a FormalParameter<'a>) -> Self {
        SiblingNode::FormalParameter(node)
    }
}

impl<'a> From<&'a FunctionBody<'a>> for SiblingNode<'a> {
    fn from(node: &'a FunctionBody<'a>) -> Self {
        SiblingNode::FunctionBody(node)
    }
}

impl<'a> From<&'a ArrowFunctionExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a ArrowFunctionExpression<'a>) -> Self {
        SiblingNode::ArrowFunctionExpression(node)
    }
}

impl<'a> From<&'a YieldExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a YieldExpression<'a>) -> Self {
        SiblingNode::YieldExpression(node)
    }
}

impl<'a> From<&'a Class<'a>> for SiblingNode<'a> {
    fn from(node: &'a Class<'a>) -> Self {
        SiblingNode::Class(node)
    }
}

impl<'a> From<&'a ClassBody<'a>> for SiblingNode<'a> {
    fn from(node: &'a ClassBody<'a>) -> Self {
        SiblingNode::ClassBody(node)
    }
}

impl<'a> From<&'a MethodDefinition<'a>> for SiblingNode<'a> {
    fn from(node: &'a MethodDefinition<'a>) -> Self {
        SiblingNode::MethodDefinition(node)
    }
}

impl<'a> From<&'a PropertyDefinition<'a>> for SiblingNode<'a> {
    fn from(node: &'a PropertyDefinition<'a>) -> Self {
        SiblingNode::PropertyDefinition(node)
    }
}

impl<'a> From<&'a PrivateIdentifier<'a>> for SiblingNode<'a> {
    fn from(node: &'a PrivateIdentifier<'a>) -> Self {
        SiblingNode::PrivateIdentifier(node)
    }
}

impl<'a> From<&'a StaticBlock<'a>> for SiblingNode<'a> {
    fn from(node: &'a StaticBlock<'a>) -> Self {
        SiblingNode::StaticBlock(node)
    }
}

impl<'a> From<&'a AccessorProperty<'a>> for SiblingNode<'a> {
    fn from(node: &'a AccessorProperty<'a>) -> Self {
        SiblingNode::AccessorProperty(node)
    }
}

impl<'a> From<&'a ImportExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a ImportExpression<'a>) -> Self {
        SiblingNode::ImportExpression(node)
    }
}

impl<'a> From<&'a ImportDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a ImportDeclaration<'a>) -> Self {
        SiblingNode::ImportDeclaration(node)
    }
}

impl<'a> From<&'a ImportSpecifier<'a>> for SiblingNode<'a> {
    fn from(node: &'a ImportSpecifier<'a>) -> Self {
        SiblingNode::ImportSpecifier(node)
    }
}

impl<'a> From<&'a ImportDefaultSpecifier<'a>> for SiblingNode<'a> {
    fn from(node: &'a ImportDefaultSpecifier<'a>) -> Self {
        SiblingNode::ImportDefaultSpecifier(node)
    }
}

impl<'a> From<&'a ImportNamespaceSpecifier<'a>> for SiblingNode<'a> {
    fn from(node: &'a ImportNamespaceSpecifier<'a>) -> Self {
        SiblingNode::ImportNamespaceSpecifier(node)
    }
}

impl<'a> From<&'a WithClause<'a>> for SiblingNode<'a> {
    fn from(node: &'a WithClause<'a>) -> Self {
        SiblingNode::WithClause(node)
    }
}

impl<'a> From<&'a ImportAttribute<'a>> for SiblingNode<'a> {
    fn from(node: &'a ImportAttribute<'a>) -> Self {
        SiblingNode::ImportAttribute(node)
    }
}

impl<'a> From<&'a ExportNamedDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a ExportNamedDeclaration<'a>) -> Self {
        SiblingNode::ExportNamedDeclaration(node)
    }
}

impl<'a> From<&'a ExportDefaultDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a ExportDefaultDeclaration<'a>) -> Self {
        SiblingNode::ExportDefaultDeclaration(node)
    }
}

impl<'a> From<&'a ExportAllDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a ExportAllDeclaration<'a>) -> Self {
        SiblingNode::ExportAllDeclaration(node)
    }
}

impl<'a> From<&'a ExportSpecifier<'a>> for SiblingNode<'a> {
    fn from(node: &'a ExportSpecifier<'a>) -> Self {
        SiblingNode::ExportSpecifier(node)
    }
}

impl<'a> From<&'a V8IntrinsicExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a V8IntrinsicExpression<'a>) -> Self {
        SiblingNode::V8IntrinsicExpression(node)
    }
}

impl<'a> From<&'a BooleanLiteral> for SiblingNode<'a> {
    fn from(node: &'a BooleanLiteral) -> Self {
        SiblingNode::BooleanLiteral(node)
    }
}

impl<'a> From<&'a NullLiteral> for SiblingNode<'a> {
    fn from(node: &'a NullLiteral) -> Self {
        SiblingNode::NullLiteral(node)
    }
}

impl<'a> From<&'a NumericLiteral<'a>> for SiblingNode<'a> {
    fn from(node: &'a NumericLiteral<'a>) -> Self {
        SiblingNode::NumericLiteral(node)
    }
}

impl<'a> From<&'a StringLiteral<'a>> for SiblingNode<'a> {
    fn from(node: &'a StringLiteral<'a>) -> Self {
        SiblingNode::StringLiteral(node)
    }
}

impl<'a> From<&'a BigIntLiteral<'a>> for SiblingNode<'a> {
    fn from(node: &'a BigIntLiteral<'a>) -> Self {
        SiblingNode::BigIntLiteral(node)
    }
}

impl<'a> From<&'a RegExpLiteral<'a>> for SiblingNode<'a> {
    fn from(node: &'a RegExpLiteral<'a>) -> Self {
        SiblingNode::RegExpLiteral(node)
    }
}

impl<'a> From<&'a JSXElement<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXElement<'a>) -> Self {
        SiblingNode::JSXElement(node)
    }
}

impl<'a> From<&'a JSXOpeningElement<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXOpeningElement<'a>) -> Self {
        SiblingNode::JSXOpeningElement(node)
    }
}

impl<'a> From<&'a JSXClosingElement<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXClosingElement<'a>) -> Self {
        SiblingNode::JSXClosingElement(node)
    }
}

impl<'a> From<&'a JSXFragment<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXFragment<'a>) -> Self {
        SiblingNode::JSXFragment(node)
    }
}

impl<'a> From<&'a JSXOpeningFragment> for SiblingNode<'a> {
    fn from(node: &'a JSXOpeningFragment) -> Self {
        SiblingNode::JSXOpeningFragment(node)
    }
}

impl<'a> From<&'a JSXClosingFragment> for SiblingNode<'a> {
    fn from(node: &'a JSXClosingFragment) -> Self {
        SiblingNode::JSXClosingFragment(node)
    }
}

impl<'a> From<&'a JSXNamespacedName<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXNamespacedName<'a>) -> Self {
        SiblingNode::JSXNamespacedName(node)
    }
}

impl<'a> From<&'a JSXMemberExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXMemberExpression<'a>) -> Self {
        SiblingNode::JSXMemberExpression(node)
    }
}

impl<'a> From<&'a JSXExpressionContainer<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXExpressionContainer<'a>) -> Self {
        SiblingNode::JSXExpressionContainer(node)
    }
}

impl<'a> From<&'a JSXEmptyExpression> for SiblingNode<'a> {
    fn from(node: &'a JSXEmptyExpression) -> Self {
        SiblingNode::JSXEmptyExpression(node)
    }
}

impl<'a> From<&'a JSXAttribute<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXAttribute<'a>) -> Self {
        SiblingNode::JSXAttribute(node)
    }
}

impl<'a> From<&'a JSXSpreadAttribute<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXSpreadAttribute<'a>) -> Self {
        SiblingNode::JSXSpreadAttribute(node)
    }
}

impl<'a> From<&'a JSXIdentifier<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXIdentifier<'a>) -> Self {
        SiblingNode::JSXIdentifier(node)
    }
}

impl<'a> From<&'a JSXSpreadChild<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXSpreadChild<'a>) -> Self {
        SiblingNode::JSXSpreadChild(node)
    }
}

impl<'a> From<&'a JSXText<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXText<'a>) -> Self {
        SiblingNode::JSXText(node)
    }
}

impl<'a> From<&'a TSThisParameter<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSThisParameter<'a>) -> Self {
        SiblingNode::TSThisParameter(node)
    }
}

impl<'a> From<&'a TSEnumDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSEnumDeclaration<'a>) -> Self {
        SiblingNode::TSEnumDeclaration(node)
    }
}

impl<'a> From<&'a TSEnumBody<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSEnumBody<'a>) -> Self {
        SiblingNode::TSEnumBody(node)
    }
}

impl<'a> From<&'a TSEnumMember<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSEnumMember<'a>) -> Self {
        SiblingNode::TSEnumMember(node)
    }
}

impl<'a> From<&'a TSTypeAnnotation<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTypeAnnotation<'a>) -> Self {
        SiblingNode::TSTypeAnnotation(node)
    }
}

impl<'a> From<&'a TSLiteralType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSLiteralType<'a>) -> Self {
        SiblingNode::TSLiteralType(node)
    }
}

impl<'a> From<&'a TSConditionalType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSConditionalType<'a>) -> Self {
        SiblingNode::TSConditionalType(node)
    }
}

impl<'a> From<&'a TSUnionType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSUnionType<'a>) -> Self {
        SiblingNode::TSUnionType(node)
    }
}

impl<'a> From<&'a TSIntersectionType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSIntersectionType<'a>) -> Self {
        SiblingNode::TSIntersectionType(node)
    }
}

impl<'a> From<&'a TSParenthesizedType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSParenthesizedType<'a>) -> Self {
        SiblingNode::TSParenthesizedType(node)
    }
}

impl<'a> From<&'a TSTypeOperator<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTypeOperator<'a>) -> Self {
        SiblingNode::TSTypeOperator(node)
    }
}

impl<'a> From<&'a TSArrayType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSArrayType<'a>) -> Self {
        SiblingNode::TSArrayType(node)
    }
}

impl<'a> From<&'a TSIndexedAccessType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSIndexedAccessType<'a>) -> Self {
        SiblingNode::TSIndexedAccessType(node)
    }
}

impl<'a> From<&'a TSTupleType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTupleType<'a>) -> Self {
        SiblingNode::TSTupleType(node)
    }
}

impl<'a> From<&'a TSNamedTupleMember<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSNamedTupleMember<'a>) -> Self {
        SiblingNode::TSNamedTupleMember(node)
    }
}

impl<'a> From<&'a TSOptionalType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSOptionalType<'a>) -> Self {
        SiblingNode::TSOptionalType(node)
    }
}

impl<'a> From<&'a TSRestType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSRestType<'a>) -> Self {
        SiblingNode::TSRestType(node)
    }
}

impl<'a> From<&'a TSAnyKeyword> for SiblingNode<'a> {
    fn from(node: &'a TSAnyKeyword) -> Self {
        SiblingNode::TSAnyKeyword(node)
    }
}

impl<'a> From<&'a TSStringKeyword> for SiblingNode<'a> {
    fn from(node: &'a TSStringKeyword) -> Self {
        SiblingNode::TSStringKeyword(node)
    }
}

impl<'a> From<&'a TSBooleanKeyword> for SiblingNode<'a> {
    fn from(node: &'a TSBooleanKeyword) -> Self {
        SiblingNode::TSBooleanKeyword(node)
    }
}

impl<'a> From<&'a TSNumberKeyword> for SiblingNode<'a> {
    fn from(node: &'a TSNumberKeyword) -> Self {
        SiblingNode::TSNumberKeyword(node)
    }
}

impl<'a> From<&'a TSNeverKeyword> for SiblingNode<'a> {
    fn from(node: &'a TSNeverKeyword) -> Self {
        SiblingNode::TSNeverKeyword(node)
    }
}

impl<'a> From<&'a TSIntrinsicKeyword> for SiblingNode<'a> {
    fn from(node: &'a TSIntrinsicKeyword) -> Self {
        SiblingNode::TSIntrinsicKeyword(node)
    }
}

impl<'a> From<&'a TSUnknownKeyword> for SiblingNode<'a> {
    fn from(node: &'a TSUnknownKeyword) -> Self {
        SiblingNode::TSUnknownKeyword(node)
    }
}

impl<'a> From<&'a TSNullKeyword> for SiblingNode<'a> {
    fn from(node: &'a TSNullKeyword) -> Self {
        SiblingNode::TSNullKeyword(node)
    }
}

impl<'a> From<&'a TSUndefinedKeyword> for SiblingNode<'a> {
    fn from(node: &'a TSUndefinedKeyword) -> Self {
        SiblingNode::TSUndefinedKeyword(node)
    }
}

impl<'a> From<&'a TSVoidKeyword> for SiblingNode<'a> {
    fn from(node: &'a TSVoidKeyword) -> Self {
        SiblingNode::TSVoidKeyword(node)
    }
}

impl<'a> From<&'a TSSymbolKeyword> for SiblingNode<'a> {
    fn from(node: &'a TSSymbolKeyword) -> Self {
        SiblingNode::TSSymbolKeyword(node)
    }
}

impl<'a> From<&'a TSThisType> for SiblingNode<'a> {
    fn from(node: &'a TSThisType) -> Self {
        SiblingNode::TSThisType(node)
    }
}

impl<'a> From<&'a TSObjectKeyword> for SiblingNode<'a> {
    fn from(node: &'a TSObjectKeyword) -> Self {
        SiblingNode::TSObjectKeyword(node)
    }
}

impl<'a> From<&'a TSBigIntKeyword> for SiblingNode<'a> {
    fn from(node: &'a TSBigIntKeyword) -> Self {
        SiblingNode::TSBigIntKeyword(node)
    }
}

impl<'a> From<&'a TSTypeReference<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTypeReference<'a>) -> Self {
        SiblingNode::TSTypeReference(node)
    }
}

impl<'a> From<&'a TSQualifiedName<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSQualifiedName<'a>) -> Self {
        SiblingNode::TSQualifiedName(node)
    }
}

impl<'a> From<&'a TSTypeParameterInstantiation<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTypeParameterInstantiation<'a>) -> Self {
        SiblingNode::TSTypeParameterInstantiation(node)
    }
}

impl<'a> From<&'a TSTypeParameter<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTypeParameter<'a>) -> Self {
        SiblingNode::TSTypeParameter(node)
    }
}

impl<'a> From<&'a TSTypeParameterDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTypeParameterDeclaration<'a>) -> Self {
        SiblingNode::TSTypeParameterDeclaration(node)
    }
}

impl<'a> From<&'a TSTypeAliasDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTypeAliasDeclaration<'a>) -> Self {
        SiblingNode::TSTypeAliasDeclaration(node)
    }
}

impl<'a> From<&'a TSClassImplements<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSClassImplements<'a>) -> Self {
        SiblingNode::TSClassImplements(node)
    }
}

impl<'a> From<&'a TSInterfaceDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSInterfaceDeclaration<'a>) -> Self {
        SiblingNode::TSInterfaceDeclaration(node)
    }
}

impl<'a> From<&'a TSInterfaceBody<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSInterfaceBody<'a>) -> Self {
        SiblingNode::TSInterfaceBody(node)
    }
}

impl<'a> From<&'a TSPropertySignature<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSPropertySignature<'a>) -> Self {
        SiblingNode::TSPropertySignature(node)
    }
}

impl<'a> From<&'a TSIndexSignature<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSIndexSignature<'a>) -> Self {
        SiblingNode::TSIndexSignature(node)
    }
}

impl<'a> From<&'a TSCallSignatureDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSCallSignatureDeclaration<'a>) -> Self {
        SiblingNode::TSCallSignatureDeclaration(node)
    }
}

impl<'a> From<&'a TSMethodSignature<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSMethodSignature<'a>) -> Self {
        SiblingNode::TSMethodSignature(node)
    }
}

impl<'a> From<&'a TSConstructSignatureDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSConstructSignatureDeclaration<'a>) -> Self {
        SiblingNode::TSConstructSignatureDeclaration(node)
    }
}

impl<'a> From<&'a TSIndexSignatureName<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSIndexSignatureName<'a>) -> Self {
        SiblingNode::TSIndexSignatureName(node)
    }
}

impl<'a> From<&'a TSInterfaceHeritage<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSInterfaceHeritage<'a>) -> Self {
        SiblingNode::TSInterfaceHeritage(node)
    }
}

impl<'a> From<&'a TSTypePredicate<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTypePredicate<'a>) -> Self {
        SiblingNode::TSTypePredicate(node)
    }
}

impl<'a> From<&'a TSModuleDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSModuleDeclaration<'a>) -> Self {
        SiblingNode::TSModuleDeclaration(node)
    }
}

impl<'a> From<&'a TSModuleBlock<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSModuleBlock<'a>) -> Self {
        SiblingNode::TSModuleBlock(node)
    }
}

impl<'a> From<&'a TSTypeLiteral<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTypeLiteral<'a>) -> Self {
        SiblingNode::TSTypeLiteral(node)
    }
}

impl<'a> From<&'a TSInferType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSInferType<'a>) -> Self {
        SiblingNode::TSInferType(node)
    }
}

impl<'a> From<&'a TSTypeQuery<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTypeQuery<'a>) -> Self {
        SiblingNode::TSTypeQuery(node)
    }
}

impl<'a> From<&'a TSImportType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSImportType<'a>) -> Self {
        SiblingNode::TSImportType(node)
    }
}

impl<'a> From<&'a TSImportTypeQualifiedName<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSImportTypeQualifiedName<'a>) -> Self {
        SiblingNode::TSImportTypeQualifiedName(node)
    }
}

impl<'a> From<&'a TSFunctionType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSFunctionType<'a>) -> Self {
        SiblingNode::TSFunctionType(node)
    }
}

impl<'a> From<&'a TSConstructorType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSConstructorType<'a>) -> Self {
        SiblingNode::TSConstructorType(node)
    }
}

impl<'a> From<&'a TSMappedType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSMappedType<'a>) -> Self {
        SiblingNode::TSMappedType(node)
    }
}

impl<'a> From<&'a TSTemplateLiteralType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTemplateLiteralType<'a>) -> Self {
        SiblingNode::TSTemplateLiteralType(node)
    }
}

impl<'a> From<&'a TSAsExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSAsExpression<'a>) -> Self {
        SiblingNode::TSAsExpression(node)
    }
}

impl<'a> From<&'a TSSatisfiesExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSSatisfiesExpression<'a>) -> Self {
        SiblingNode::TSSatisfiesExpression(node)
    }
}

impl<'a> From<&'a TSTypeAssertion<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTypeAssertion<'a>) -> Self {
        SiblingNode::TSTypeAssertion(node)
    }
}

impl<'a> From<&'a TSImportEqualsDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSImportEqualsDeclaration<'a>) -> Self {
        SiblingNode::TSImportEqualsDeclaration(node)
    }
}

impl<'a> From<&'a TSExternalModuleReference<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSExternalModuleReference<'a>) -> Self {
        SiblingNode::TSExternalModuleReference(node)
    }
}

impl<'a> From<&'a TSNonNullExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSNonNullExpression<'a>) -> Self {
        SiblingNode::TSNonNullExpression(node)
    }
}

impl<'a> From<&'a Decorator<'a>> for SiblingNode<'a> {
    fn from(node: &'a Decorator<'a>) -> Self {
        SiblingNode::Decorator(node)
    }
}

impl<'a> From<&'a TSExportAssignment<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSExportAssignment<'a>) -> Self {
        SiblingNode::TSExportAssignment(node)
    }
}

impl<'a> From<&'a TSNamespaceExportDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSNamespaceExportDeclaration<'a>) -> Self {
        SiblingNode::TSNamespaceExportDeclaration(node)
    }
}

impl<'a> From<&'a TSInstantiationExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSInstantiationExpression<'a>) -> Self {
        SiblingNode::TSInstantiationExpression(node)
    }
}

impl<'a> From<&'a JSDocNullableType<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSDocNullableType<'a>) -> Self {
        SiblingNode::JSDocNullableType(node)
    }
}

impl<'a> From<&'a JSDocNonNullableType<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSDocNonNullableType<'a>) -> Self {
        SiblingNode::JSDocNonNullableType(node)
    }
}

impl<'a> From<&'a JSDocUnknownType> for SiblingNode<'a> {
    fn from(node: &'a JSDocUnknownType) -> Self {
        SiblingNode::JSDocUnknownType(node)
    }
}

impl<'a> From<&'a Expression<'a>> for SiblingNode<'a> {
    fn from(node: &'a Expression<'a>) -> Self {
        match node {
            Expression::BooleanLiteral(inner) => SiblingNode::BooleanLiteral(inner),
            Expression::NullLiteral(inner) => SiblingNode::NullLiteral(inner),
            Expression::NumericLiteral(inner) => SiblingNode::NumericLiteral(inner),
            Expression::BigIntLiteral(inner) => SiblingNode::BigIntLiteral(inner),
            Expression::RegExpLiteral(inner) => SiblingNode::RegExpLiteral(inner),
            Expression::StringLiteral(inner) => SiblingNode::StringLiteral(inner),
            Expression::TemplateLiteral(inner) => SiblingNode::TemplateLiteral(inner),
            Expression::Identifier(inner) => SiblingNode::IdentifierReference(inner),
            Expression::MetaProperty(inner) => SiblingNode::MetaProperty(inner),
            Expression::Super(inner) => SiblingNode::Super(inner),
            Expression::ArrayExpression(inner) => SiblingNode::ArrayExpression(inner),
            Expression::ArrowFunctionExpression(inner) => {
                SiblingNode::ArrowFunctionExpression(inner)
            }
            Expression::AssignmentExpression(inner) => SiblingNode::AssignmentExpression(inner),
            Expression::AwaitExpression(inner) => SiblingNode::AwaitExpression(inner),
            Expression::BinaryExpression(inner) => SiblingNode::BinaryExpression(inner),
            Expression::CallExpression(inner) => SiblingNode::CallExpression(inner),
            Expression::ChainExpression(inner) => SiblingNode::ChainExpression(inner),
            Expression::ClassExpression(inner) => SiblingNode::Class(inner),
            Expression::ConditionalExpression(inner) => SiblingNode::ConditionalExpression(inner),
            Expression::FunctionExpression(inner) => SiblingNode::Function(inner),
            Expression::ImportExpression(inner) => SiblingNode::ImportExpression(inner),
            Expression::LogicalExpression(inner) => SiblingNode::LogicalExpression(inner),
            Expression::NewExpression(inner) => SiblingNode::NewExpression(inner),
            Expression::ObjectExpression(inner) => SiblingNode::ObjectExpression(inner),
            Expression::ParenthesizedExpression(inner) => {
                SiblingNode::ParenthesizedExpression(inner)
            }
            Expression::SequenceExpression(inner) => SiblingNode::SequenceExpression(inner),
            Expression::TaggedTemplateExpression(inner) => {
                SiblingNode::TaggedTemplateExpression(inner)
            }
            Expression::ThisExpression(inner) => SiblingNode::ThisExpression(inner),
            Expression::UnaryExpression(inner) => SiblingNode::UnaryExpression(inner),
            Expression::UpdateExpression(inner) => SiblingNode::UpdateExpression(inner),
            Expression::YieldExpression(inner) => SiblingNode::YieldExpression(inner),
            Expression::PrivateInExpression(inner) => SiblingNode::PrivateInExpression(inner),
            Expression::JSXElement(inner) => SiblingNode::JSXElement(inner),
            Expression::JSXFragment(inner) => SiblingNode::JSXFragment(inner),
            Expression::TSAsExpression(inner) => SiblingNode::TSAsExpression(inner),
            Expression::TSSatisfiesExpression(inner) => SiblingNode::TSSatisfiesExpression(inner),
            Expression::TSTypeAssertion(inner) => SiblingNode::TSTypeAssertion(inner),
            Expression::TSNonNullExpression(inner) => SiblingNode::TSNonNullExpression(inner),
            Expression::TSInstantiationExpression(inner) => {
                SiblingNode::TSInstantiationExpression(inner)
            }
            Expression::V8IntrinsicExpression(inner) => SiblingNode::V8IntrinsicExpression(inner),
            it @ match_member_expression!(Expression) => {
                SiblingNode::from(it.to_member_expression())
            }
        }
    }
}

impl<'a> From<&'a ArrayExpressionElement<'a>> for SiblingNode<'a> {
    fn from(node: &'a ArrayExpressionElement<'a>) -> Self {
        match node {
            ArrayExpressionElement::SpreadElement(inner) => SiblingNode::SpreadElement(inner),
            ArrayExpressionElement::Elision(inner) => SiblingNode::Elision(inner),
            it @ match_expression!(ArrayExpressionElement) => SiblingNode::from(it.to_expression()),
        }
    }
}

impl<'a> From<&'a ObjectPropertyKind<'a>> for SiblingNode<'a> {
    fn from(node: &'a ObjectPropertyKind<'a>) -> Self {
        match node {
            ObjectPropertyKind::ObjectProperty(inner) => SiblingNode::ObjectProperty(inner),
            ObjectPropertyKind::SpreadProperty(inner) => SiblingNode::SpreadElement(inner),
        }
    }
}

impl<'a> From<&'a PropertyKey<'a>> for SiblingNode<'a> {
    fn from(node: &'a PropertyKey<'a>) -> Self {
        match node {
            PropertyKey::StaticIdentifier(inner) => SiblingNode::IdentifierName(inner),
            PropertyKey::PrivateIdentifier(inner) => SiblingNode::PrivateIdentifier(inner),
            it @ match_expression!(PropertyKey) => SiblingNode::from(it.to_expression()),
        }
    }
}

impl<'a> From<&'a MemberExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a MemberExpression<'a>) -> Self {
        match node {
            MemberExpression::ComputedMemberExpression(inner) => {
                SiblingNode::ComputedMemberExpression(inner)
            }
            MemberExpression::StaticMemberExpression(inner) => {
                SiblingNode::StaticMemberExpression(inner)
            }
            MemberExpression::PrivateFieldExpression(inner) => {
                SiblingNode::PrivateFieldExpression(inner)
            }
        }
    }
}

impl<'a> From<&'a Argument<'a>> for SiblingNode<'a> {
    fn from(node: &'a Argument<'a>) -> Self {
        match node {
            Argument::SpreadElement(inner) => SiblingNode::SpreadElement(inner),
            it @ match_expression!(Argument) => SiblingNode::from(it.to_expression()),
        }
    }
}

impl<'a> From<&'a AssignmentTarget<'a>> for SiblingNode<'a> {
    fn from(node: &'a AssignmentTarget<'a>) -> Self {
        match node {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                SiblingNode::from(it.to_simple_assignment_target())
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                SiblingNode::from(it.to_assignment_target_pattern())
            }
        }
    }
}

impl<'a> From<&'a SimpleAssignmentTarget<'a>> for SiblingNode<'a> {
    fn from(node: &'a SimpleAssignmentTarget<'a>) -> Self {
        match node {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(inner) => {
                SiblingNode::IdentifierReference(inner)
            }
            SimpleAssignmentTarget::TSAsExpression(inner) => SiblingNode::TSAsExpression(inner),
            SimpleAssignmentTarget::TSSatisfiesExpression(inner) => {
                SiblingNode::TSSatisfiesExpression(inner)
            }
            SimpleAssignmentTarget::TSNonNullExpression(inner) => {
                SiblingNode::TSNonNullExpression(inner)
            }
            SimpleAssignmentTarget::TSTypeAssertion(inner) => SiblingNode::TSTypeAssertion(inner),
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                SiblingNode::from(it.to_member_expression())
            }
        }
    }
}

impl<'a> From<&'a AssignmentTargetPattern<'a>> for SiblingNode<'a> {
    fn from(node: &'a AssignmentTargetPattern<'a>) -> Self {
        match node {
            AssignmentTargetPattern::ArrayAssignmentTarget(inner) => {
                SiblingNode::ArrayAssignmentTarget(inner)
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(inner) => {
                SiblingNode::ObjectAssignmentTarget(inner)
            }
        }
    }
}

impl<'a> From<&'a AssignmentTargetMaybeDefault<'a>> for SiblingNode<'a> {
    fn from(node: &'a AssignmentTargetMaybeDefault<'a>) -> Self {
        match node {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(inner) => {
                SiblingNode::AssignmentTargetWithDefault(inner)
            }
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                SiblingNode::from(it.to_assignment_target())
            }
        }
    }
}

impl<'a> From<&'a AssignmentTargetProperty<'a>> for SiblingNode<'a> {
    fn from(node: &'a AssignmentTargetProperty<'a>) -> Self {
        match node {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(inner) => {
                SiblingNode::AssignmentTargetPropertyIdentifier(inner)
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(inner) => {
                SiblingNode::AssignmentTargetPropertyProperty(inner)
            }
        }
    }
}

impl<'a> From<&'a ChainElement<'a>> for SiblingNode<'a> {
    fn from(node: &'a ChainElement<'a>) -> Self {
        match node {
            ChainElement::CallExpression(inner) => SiblingNode::CallExpression(inner),
            ChainElement::TSNonNullExpression(inner) => SiblingNode::TSNonNullExpression(inner),
            it @ match_member_expression!(ChainElement) => {
                SiblingNode::from(it.to_member_expression())
            }
        }
    }
}

impl<'a> From<&'a Statement<'a>> for SiblingNode<'a> {
    fn from(node: &'a Statement<'a>) -> Self {
        match node {
            Statement::BlockStatement(inner) => SiblingNode::BlockStatement(inner),
            Statement::BreakStatement(inner) => SiblingNode::BreakStatement(inner),
            Statement::ContinueStatement(inner) => SiblingNode::ContinueStatement(inner),
            Statement::DebuggerStatement(inner) => SiblingNode::DebuggerStatement(inner),
            Statement::DoWhileStatement(inner) => SiblingNode::DoWhileStatement(inner),
            Statement::EmptyStatement(inner) => SiblingNode::EmptyStatement(inner),
            Statement::ExpressionStatement(inner) => SiblingNode::ExpressionStatement(inner),
            Statement::ForInStatement(inner) => SiblingNode::ForInStatement(inner),
            Statement::ForOfStatement(inner) => SiblingNode::ForOfStatement(inner),
            Statement::ForStatement(inner) => SiblingNode::ForStatement(inner),
            Statement::IfStatement(inner) => SiblingNode::IfStatement(inner),
            Statement::LabeledStatement(inner) => SiblingNode::LabeledStatement(inner),
            Statement::ReturnStatement(inner) => SiblingNode::ReturnStatement(inner),
            Statement::SwitchStatement(inner) => SiblingNode::SwitchStatement(inner),
            Statement::ThrowStatement(inner) => SiblingNode::ThrowStatement(inner),
            Statement::TryStatement(inner) => SiblingNode::TryStatement(inner),
            Statement::WhileStatement(inner) => SiblingNode::WhileStatement(inner),
            Statement::WithStatement(inner) => SiblingNode::WithStatement(inner),
            it @ match_declaration!(Statement) => SiblingNode::from(it.to_declaration()),
            it @ match_module_declaration!(Statement) => {
                SiblingNode::from(it.to_module_declaration())
            }
        }
    }
}

impl<'a> From<&'a Declaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a Declaration<'a>) -> Self {
        match node {
            Declaration::VariableDeclaration(inner) => SiblingNode::VariableDeclaration(inner),
            Declaration::FunctionDeclaration(inner) => SiblingNode::Function(inner),
            Declaration::ClassDeclaration(inner) => SiblingNode::Class(inner),
            Declaration::TSTypeAliasDeclaration(inner) => {
                SiblingNode::TSTypeAliasDeclaration(inner)
            }
            Declaration::TSInterfaceDeclaration(inner) => {
                SiblingNode::TSInterfaceDeclaration(inner)
            }
            Declaration::TSEnumDeclaration(inner) => SiblingNode::TSEnumDeclaration(inner),
            Declaration::TSModuleDeclaration(inner) => SiblingNode::TSModuleDeclaration(inner),
            Declaration::TSImportEqualsDeclaration(inner) => {
                SiblingNode::TSImportEqualsDeclaration(inner)
            }
        }
    }
}

impl<'a> From<&'a ForStatementInit<'a>> for SiblingNode<'a> {
    fn from(node: &'a ForStatementInit<'a>) -> Self {
        match node {
            ForStatementInit::VariableDeclaration(inner) => SiblingNode::VariableDeclaration(inner),
            it @ match_expression!(ForStatementInit) => SiblingNode::from(it.to_expression()),
        }
    }
}

impl<'a> From<&'a ForStatementLeft<'a>> for SiblingNode<'a> {
    fn from(node: &'a ForStatementLeft<'a>) -> Self {
        match node {
            ForStatementLeft::VariableDeclaration(inner) => SiblingNode::VariableDeclaration(inner),
            it @ match_assignment_target!(ForStatementLeft) => {
                SiblingNode::from(it.to_assignment_target())
            }
        }
    }
}

impl<'a> From<&'a BindingPatternKind<'a>> for SiblingNode<'a> {
    fn from(node: &'a BindingPatternKind<'a>) -> Self {
        match node {
            BindingPatternKind::BindingIdentifier(inner) => SiblingNode::BindingIdentifier(inner),
            BindingPatternKind::ObjectPattern(inner) => SiblingNode::ObjectPattern(inner),
            BindingPatternKind::ArrayPattern(inner) => SiblingNode::ArrayPattern(inner),
            BindingPatternKind::AssignmentPattern(inner) => SiblingNode::AssignmentPattern(inner),
        }
    }
}

impl<'a> From<&'a ClassElement<'a>> for SiblingNode<'a> {
    fn from(node: &'a ClassElement<'a>) -> Self {
        match node {
            ClassElement::StaticBlock(inner) => SiblingNode::StaticBlock(inner),
            ClassElement::MethodDefinition(inner) => SiblingNode::MethodDefinition(inner),
            ClassElement::PropertyDefinition(inner) => SiblingNode::PropertyDefinition(inner),
            ClassElement::AccessorProperty(inner) => SiblingNode::AccessorProperty(inner),
            ClassElement::TSIndexSignature(inner) => SiblingNode::TSIndexSignature(inner),
        }
    }
}

impl<'a> From<&'a ModuleDeclaration<'a>> for SiblingNode<'a> {
    fn from(node: &'a ModuleDeclaration<'a>) -> Self {
        match node {
            ModuleDeclaration::ImportDeclaration(inner) => SiblingNode::ImportDeclaration(inner),
            ModuleDeclaration::ExportAllDeclaration(inner) => {
                SiblingNode::ExportAllDeclaration(inner)
            }
            ModuleDeclaration::ExportDefaultDeclaration(inner) => {
                SiblingNode::ExportDefaultDeclaration(inner)
            }
            ModuleDeclaration::ExportNamedDeclaration(inner) => {
                SiblingNode::ExportNamedDeclaration(inner)
            }
            ModuleDeclaration::TSExportAssignment(inner) => SiblingNode::TSExportAssignment(inner),
            ModuleDeclaration::TSNamespaceExportDeclaration(inner) => {
                SiblingNode::TSNamespaceExportDeclaration(inner)
            }
        }
    }
}

impl<'a> From<&'a ImportDeclarationSpecifier<'a>> for SiblingNode<'a> {
    fn from(node: &'a ImportDeclarationSpecifier<'a>) -> Self {
        match node {
            ImportDeclarationSpecifier::ImportSpecifier(inner) => {
                SiblingNode::ImportSpecifier(inner)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(inner) => {
                SiblingNode::ImportDefaultSpecifier(inner)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(inner) => {
                SiblingNode::ImportNamespaceSpecifier(inner)
            }
        }
    }
}

impl<'a> From<&'a ImportAttributeKey<'a>> for SiblingNode<'a> {
    fn from(node: &'a ImportAttributeKey<'a>) -> Self {
        match node {
            ImportAttributeKey::Identifier(inner) => SiblingNode::IdentifierName(inner),
            ImportAttributeKey::StringLiteral(inner) => SiblingNode::StringLiteral(inner),
        }
    }
}

impl<'a> From<&'a ExportDefaultDeclarationKind<'a>> for SiblingNode<'a> {
    fn from(node: &'a ExportDefaultDeclarationKind<'a>) -> Self {
        match node {
            ExportDefaultDeclarationKind::FunctionDeclaration(inner) => {
                SiblingNode::Function(inner)
            }
            ExportDefaultDeclarationKind::ClassDeclaration(inner) => SiblingNode::Class(inner),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(inner) => {
                SiblingNode::TSInterfaceDeclaration(inner)
            }
            it @ match_expression!(ExportDefaultDeclarationKind) => {
                SiblingNode::from(it.to_expression())
            }
        }
    }
}

impl<'a> From<&'a ModuleExportName<'a>> for SiblingNode<'a> {
    fn from(node: &'a ModuleExportName<'a>) -> Self {
        match node {
            ModuleExportName::IdentifierName(inner) => SiblingNode::IdentifierName(inner),
            ModuleExportName::IdentifierReference(inner) => SiblingNode::IdentifierReference(inner),
            ModuleExportName::StringLiteral(inner) => SiblingNode::StringLiteral(inner),
        }
    }
}

impl<'a> From<&'a JSXElementName<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXElementName<'a>) -> Self {
        match node {
            JSXElementName::Identifier(inner) => SiblingNode::JSXIdentifier(inner),
            JSXElementName::IdentifierReference(inner) => SiblingNode::IdentifierReference(inner),
            JSXElementName::NamespacedName(inner) => SiblingNode::JSXNamespacedName(inner),
            JSXElementName::MemberExpression(inner) => SiblingNode::JSXMemberExpression(inner),
            JSXElementName::ThisExpression(inner) => SiblingNode::ThisExpression(inner),
        }
    }
}

impl<'a> From<&'a JSXMemberExpressionObject<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXMemberExpressionObject<'a>) -> Self {
        match node {
            JSXMemberExpressionObject::IdentifierReference(inner) => {
                SiblingNode::IdentifierReference(inner)
            }
            JSXMemberExpressionObject::MemberExpression(inner) => {
                SiblingNode::JSXMemberExpression(inner)
            }
            JSXMemberExpressionObject::ThisExpression(inner) => SiblingNode::ThisExpression(inner),
        }
    }
}

impl<'a> From<&'a JSXExpression<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXExpression<'a>) -> Self {
        match node {
            JSXExpression::EmptyExpression(inner) => SiblingNode::JSXEmptyExpression(inner),
            it @ match_expression!(JSXExpression) => SiblingNode::from(it.to_expression()),
        }
    }
}

impl<'a> From<&'a JSXAttributeItem<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXAttributeItem<'a>) -> Self {
        match node {
            JSXAttributeItem::Attribute(inner) => SiblingNode::JSXAttribute(inner),
            JSXAttributeItem::SpreadAttribute(inner) => SiblingNode::JSXSpreadAttribute(inner),
        }
    }
}

impl<'a> From<&'a JSXAttributeName<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXAttributeName<'a>) -> Self {
        match node {
            JSXAttributeName::Identifier(inner) => SiblingNode::JSXIdentifier(inner),
            JSXAttributeName::NamespacedName(inner) => SiblingNode::JSXNamespacedName(inner),
        }
    }
}

impl<'a> From<&'a JSXAttributeValue<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXAttributeValue<'a>) -> Self {
        match node {
            JSXAttributeValue::StringLiteral(inner) => SiblingNode::StringLiteral(inner),
            JSXAttributeValue::ExpressionContainer(inner) => {
                SiblingNode::JSXExpressionContainer(inner)
            }
            JSXAttributeValue::Element(inner) => SiblingNode::JSXElement(inner),
            JSXAttributeValue::Fragment(inner) => SiblingNode::JSXFragment(inner),
        }
    }
}

impl<'a> From<&'a JSXChild<'a>> for SiblingNode<'a> {
    fn from(node: &'a JSXChild<'a>) -> Self {
        match node {
            JSXChild::Text(inner) => SiblingNode::JSXText(inner),
            JSXChild::Element(inner) => SiblingNode::JSXElement(inner),
            JSXChild::Fragment(inner) => SiblingNode::JSXFragment(inner),
            JSXChild::ExpressionContainer(inner) => SiblingNode::JSXExpressionContainer(inner),
            JSXChild::Spread(inner) => SiblingNode::JSXSpreadChild(inner),
        }
    }
}

impl<'a> From<&'a TSEnumMemberName<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSEnumMemberName<'a>) -> Self {
        match node {
            TSEnumMemberName::Identifier(inner) => SiblingNode::IdentifierName(inner),
            TSEnumMemberName::String(inner) => SiblingNode::StringLiteral(inner),
            TSEnumMemberName::ComputedString(inner) => SiblingNode::StringLiteral(inner),
            TSEnumMemberName::ComputedTemplateString(inner) => SiblingNode::TemplateLiteral(inner),
        }
    }
}

impl<'a> From<&'a TSLiteral<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSLiteral<'a>) -> Self {
        match node {
            TSLiteral::BooleanLiteral(inner) => SiblingNode::BooleanLiteral(inner),
            TSLiteral::NumericLiteral(inner) => SiblingNode::NumericLiteral(inner),
            TSLiteral::BigIntLiteral(inner) => SiblingNode::BigIntLiteral(inner),
            TSLiteral::StringLiteral(inner) => SiblingNode::StringLiteral(inner),
            TSLiteral::TemplateLiteral(inner) => SiblingNode::TemplateLiteral(inner),
            TSLiteral::UnaryExpression(inner) => SiblingNode::UnaryExpression(inner),
        }
    }
}

impl<'a> From<&'a TSType<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSType<'a>) -> Self {
        match node {
            TSType::TSAnyKeyword(inner) => SiblingNode::TSAnyKeyword(inner),
            TSType::TSBigIntKeyword(inner) => SiblingNode::TSBigIntKeyword(inner),
            TSType::TSBooleanKeyword(inner) => SiblingNode::TSBooleanKeyword(inner),
            TSType::TSIntrinsicKeyword(inner) => SiblingNode::TSIntrinsicKeyword(inner),
            TSType::TSNeverKeyword(inner) => SiblingNode::TSNeverKeyword(inner),
            TSType::TSNullKeyword(inner) => SiblingNode::TSNullKeyword(inner),
            TSType::TSNumberKeyword(inner) => SiblingNode::TSNumberKeyword(inner),
            TSType::TSObjectKeyword(inner) => SiblingNode::TSObjectKeyword(inner),
            TSType::TSStringKeyword(inner) => SiblingNode::TSStringKeyword(inner),
            TSType::TSSymbolKeyword(inner) => SiblingNode::TSSymbolKeyword(inner),
            TSType::TSUndefinedKeyword(inner) => SiblingNode::TSUndefinedKeyword(inner),
            TSType::TSUnknownKeyword(inner) => SiblingNode::TSUnknownKeyword(inner),
            TSType::TSVoidKeyword(inner) => SiblingNode::TSVoidKeyword(inner),
            TSType::TSArrayType(inner) => SiblingNode::TSArrayType(inner),
            TSType::TSConditionalType(inner) => SiblingNode::TSConditionalType(inner),
            TSType::TSConstructorType(inner) => SiblingNode::TSConstructorType(inner),
            TSType::TSFunctionType(inner) => SiblingNode::TSFunctionType(inner),
            TSType::TSImportType(inner) => SiblingNode::TSImportType(inner),
            TSType::TSIndexedAccessType(inner) => SiblingNode::TSIndexedAccessType(inner),
            TSType::TSInferType(inner) => SiblingNode::TSInferType(inner),
            TSType::TSIntersectionType(inner) => SiblingNode::TSIntersectionType(inner),
            TSType::TSLiteralType(inner) => SiblingNode::TSLiteralType(inner),
            TSType::TSMappedType(inner) => SiblingNode::TSMappedType(inner),
            TSType::TSNamedTupleMember(inner) => SiblingNode::TSNamedTupleMember(inner),
            TSType::TSTemplateLiteralType(inner) => SiblingNode::TSTemplateLiteralType(inner),
            TSType::TSThisType(inner) => SiblingNode::TSThisType(inner),
            TSType::TSTupleType(inner) => SiblingNode::TSTupleType(inner),
            TSType::TSTypeLiteral(inner) => SiblingNode::TSTypeLiteral(inner),
            TSType::TSTypeOperatorType(inner) => SiblingNode::TSTypeOperator(inner),
            TSType::TSTypePredicate(inner) => SiblingNode::TSTypePredicate(inner),
            TSType::TSTypeQuery(inner) => SiblingNode::TSTypeQuery(inner),
            TSType::TSTypeReference(inner) => SiblingNode::TSTypeReference(inner),
            TSType::TSUnionType(inner) => SiblingNode::TSUnionType(inner),
            TSType::TSParenthesizedType(inner) => SiblingNode::TSParenthesizedType(inner),
            TSType::JSDocNullableType(inner) => SiblingNode::JSDocNullableType(inner),
            TSType::JSDocNonNullableType(inner) => SiblingNode::JSDocNonNullableType(inner),
            TSType::JSDocUnknownType(inner) => SiblingNode::JSDocUnknownType(inner),
        }
    }
}

impl<'a> From<&'a TSTupleElement<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTupleElement<'a>) -> Self {
        match node {
            TSTupleElement::TSOptionalType(inner) => SiblingNode::TSOptionalType(inner),
            TSTupleElement::TSRestType(inner) => SiblingNode::TSRestType(inner),
            it @ match_ts_type!(TSTupleElement) => SiblingNode::from(it.to_ts_type()),
        }
    }
}

impl<'a> From<&'a TSTypeName<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTypeName<'a>) -> Self {
        match node {
            TSTypeName::IdentifierReference(inner) => SiblingNode::IdentifierReference(inner),
            TSTypeName::QualifiedName(inner) => SiblingNode::TSQualifiedName(inner),
            TSTypeName::ThisExpression(inner) => SiblingNode::ThisExpression(inner),
        }
    }
}

impl<'a> From<&'a TSSignature<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSSignature<'a>) -> Self {
        match node {
            TSSignature::TSIndexSignature(inner) => SiblingNode::TSIndexSignature(inner),
            TSSignature::TSPropertySignature(inner) => SiblingNode::TSPropertySignature(inner),
            TSSignature::TSCallSignatureDeclaration(inner) => {
                SiblingNode::TSCallSignatureDeclaration(inner)
            }
            TSSignature::TSConstructSignatureDeclaration(inner) => {
                SiblingNode::TSConstructSignatureDeclaration(inner)
            }
            TSSignature::TSMethodSignature(inner) => SiblingNode::TSMethodSignature(inner),
        }
    }
}

impl<'a> From<&'a TSTypePredicateName<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTypePredicateName<'a>) -> Self {
        match node {
            TSTypePredicateName::Identifier(inner) => SiblingNode::IdentifierName(inner),
            TSTypePredicateName::This(inner) => SiblingNode::TSThisType(inner),
        }
    }
}

impl<'a> From<&'a TSModuleDeclarationName<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSModuleDeclarationName<'a>) -> Self {
        match node {
            TSModuleDeclarationName::Identifier(inner) => SiblingNode::BindingIdentifier(inner),
            TSModuleDeclarationName::StringLiteral(inner) => SiblingNode::StringLiteral(inner),
        }
    }
}

impl<'a> From<&'a TSModuleDeclarationBody<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSModuleDeclarationBody<'a>) -> Self {
        match node {
            TSModuleDeclarationBody::TSModuleDeclaration(inner) => {
                SiblingNode::TSModuleDeclaration(inner)
            }
            TSModuleDeclarationBody::TSModuleBlock(inner) => SiblingNode::TSModuleBlock(inner),
        }
    }
}

impl<'a> From<&'a TSTypeQueryExprName<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSTypeQueryExprName<'a>) -> Self {
        match node {
            TSTypeQueryExprName::TSImportType(inner) => SiblingNode::TSImportType(inner),
            it @ match_ts_type_name!(TSTypeQueryExprName) => {
                SiblingNode::from(it.to_ts_type_name())
            }
        }
    }
}

impl<'a> From<&'a TSImportTypeQualifier<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSImportTypeQualifier<'a>) -> Self {
        match node {
            TSImportTypeQualifier::Identifier(inner) => SiblingNode::IdentifierName(inner),
            TSImportTypeQualifier::QualifiedName(inner) => {
                SiblingNode::TSImportTypeQualifiedName(inner)
            }
        }
    }
}

impl<'a> From<&'a TSModuleReference<'a>> for SiblingNode<'a> {
    fn from(node: &'a TSModuleReference<'a>) -> Self {
        match node {
            TSModuleReference::ExternalModuleReference(inner) => {
                SiblingNode::TSExternalModuleReference(inner)
            }
            it @ match_ts_type_name!(TSModuleReference) => SiblingNode::from(it.to_ts_type_name()),
        }
    }
}

impl SiblingNode<'_> {
    pub fn span(&self) -> oxc_span::Span {
        match self {
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
            Self::BindingPattern(n) => n.span(),
            Self::AssignmentPattern(n) => n.span(),
            Self::ObjectPattern(n) => n.span(),
            Self::BindingProperty(n) => n.span(),
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
}
impl<'a> AstNodes<'a> {
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
            Self::Argument(n) => n.span(),
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
            Self::Elision(n) => n.parent,
            Self::ObjectExpression(n) => n.parent,
            Self::ObjectProperty(n) => n.parent,
            Self::TemplateLiteral(n) => n.parent,
            Self::TaggedTemplateExpression(n) => n.parent,
            Self::TemplateElement(n) => n.parent,
            Self::ComputedMemberExpression(n) => n.parent,
            Self::StaticMemberExpression(n) => n.parent,
            Self::PrivateFieldExpression(n) => n.parent,
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
            Self::ArrayAssignmentTarget(n) => n.parent,
            Self::ObjectAssignmentTarget(n) => n.parent,
            Self::AssignmentTargetRest(n) => n.parent,
            Self::AssignmentTargetWithDefault(n) => n.parent,
            Self::AssignmentTargetPropertyIdentifier(n) => n.parent,
            Self::AssignmentTargetPropertyProperty(n) => n.parent,
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
            Self::BindingProperty(n) => n.parent,
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
            Self::AccessorProperty(n) => n.parent,
            Self::ImportExpression(n) => n.parent,
            Self::ImportDeclaration(n) => n.parent,
            Self::ImportSpecifier(n) => n.parent,
            Self::ImportDefaultSpecifier(n) => n.parent,
            Self::ImportNamespaceSpecifier(n) => n.parent,
            Self::WithClause(n) => n.parent,
            Self::ImportAttribute(n) => n.parent,
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
            Self::JSXOpeningFragment(n) => n.parent,
            Self::JSXClosingFragment(n) => n.parent,
            Self::JSXNamespacedName(n) => n.parent,
            Self::JSXMemberExpression(n) => n.parent,
            Self::JSXExpressionContainer(n) => n.parent,
            Self::JSXEmptyExpression(n) => n.parent,
            Self::JSXAttribute(n) => n.parent,
            Self::JSXSpreadAttribute(n) => n.parent,
            Self::JSXIdentifier(n) => n.parent,
            Self::JSXSpreadChild(n) => n.parent,
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
            Self::TSTypeOperator(n) => n.parent,
            Self::TSArrayType(n) => n.parent,
            Self::TSIndexedAccessType(n) => n.parent,
            Self::TSTupleType(n) => n.parent,
            Self::TSNamedTupleMember(n) => n.parent,
            Self::TSOptionalType(n) => n.parent,
            Self::TSRestType(n) => n.parent,
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
            Self::TSQualifiedName(n) => n.parent,
            Self::TSTypeParameterInstantiation(n) => n.parent,
            Self::TSTypeParameter(n) => n.parent,
            Self::TSTypeParameterDeclaration(n) => n.parent,
            Self::TSTypeAliasDeclaration(n) => n.parent,
            Self::TSClassImplements(n) => n.parent,
            Self::TSInterfaceDeclaration(n) => n.parent,
            Self::TSInterfaceBody(n) => n.parent,
            Self::TSPropertySignature(n) => n.parent,
            Self::TSIndexSignature(n) => n.parent,
            Self::TSCallSignatureDeclaration(n) => n.parent,
            Self::TSMethodSignature(n) => n.parent,
            Self::TSConstructSignatureDeclaration(n) => n.parent,
            Self::TSIndexSignatureName(n) => n.parent,
            Self::TSInterfaceHeritage(n) => n.parent,
            Self::TSTypePredicate(n) => n.parent,
            Self::TSModuleDeclaration(n) => n.parent,
            Self::TSModuleBlock(n) => n.parent,
            Self::TSTypeLiteral(n) => n.parent,
            Self::TSInferType(n) => n.parent,
            Self::TSTypeQuery(n) => n.parent,
            Self::TSImportType(n) => n.parent,
            Self::TSImportTypeQualifiedName(n) => n.parent,
            Self::TSFunctionType(n) => n.parent,
            Self::TSConstructorType(n) => n.parent,
            Self::TSMappedType(n) => n.parent,
            Self::TSTemplateLiteralType(n) => n.parent,
            Self::TSAsExpression(n) => n.parent,
            Self::TSSatisfiesExpression(n) => n.parent,
            Self::TSTypeAssertion(n) => n.parent,
            Self::TSImportEqualsDeclaration(n) => n.parent,
            Self::TSExternalModuleReference(n) => n.parent,
            Self::TSNonNullExpression(n) => n.parent,
            Self::Decorator(n) => n.parent,
            Self::TSExportAssignment(n) => n.parent,
            Self::TSNamespaceExportDeclaration(n) => n.parent,
            Self::TSInstantiationExpression(n) => n.parent,
            Self::JSDocNullableType(n) => n.parent,
            Self::JSDocNonNullableType(n) => n.parent,
            Self::JSDocUnknownType(n) => n.parent,
        }
    }
    #[inline]
    pub fn as_sibling_node(&self) -> SiblingNode<'a> {
        match self {
            Self::Dummy() => panic!("Should never be called on a dummy node"),
            Self::Program(n) => SiblingNode::from(n.inner),
            Self::IdentifierName(n) => SiblingNode::from(n.inner),
            Self::IdentifierReference(n) => SiblingNode::from(n.inner),
            Self::BindingIdentifier(n) => SiblingNode::from(n.inner),
            Self::LabelIdentifier(n) => SiblingNode::from(n.inner),
            Self::ThisExpression(n) => SiblingNode::from(n.inner),
            Self::ArrayExpression(n) => SiblingNode::from(n.inner),
            Self::Elision(n) => SiblingNode::from(n.inner),
            Self::ObjectExpression(n) => SiblingNode::from(n.inner),
            Self::ObjectProperty(n) => SiblingNode::from(n.inner),
            Self::TemplateLiteral(n) => SiblingNode::from(n.inner),
            Self::TaggedTemplateExpression(n) => SiblingNode::from(n.inner),
            Self::TemplateElement(n) => SiblingNode::from(n.inner),
            Self::ComputedMemberExpression(n) => SiblingNode::from(n.inner),
            Self::StaticMemberExpression(n) => SiblingNode::from(n.inner),
            Self::PrivateFieldExpression(n) => SiblingNode::from(n.inner),
            Self::CallExpression(n) => SiblingNode::from(n.inner),
            Self::NewExpression(n) => SiblingNode::from(n.inner),
            Self::MetaProperty(n) => SiblingNode::from(n.inner),
            Self::SpreadElement(n) => SiblingNode::from(n.inner),
            Self::Argument(n) => n.parent.as_sibling_node(),
            Self::UpdateExpression(n) => SiblingNode::from(n.inner),
            Self::UnaryExpression(n) => SiblingNode::from(n.inner),
            Self::BinaryExpression(n) => SiblingNode::from(n.inner),
            Self::PrivateInExpression(n) => SiblingNode::from(n.inner),
            Self::LogicalExpression(n) => SiblingNode::from(n.inner),
            Self::ConditionalExpression(n) => SiblingNode::from(n.inner),
            Self::AssignmentExpression(n) => SiblingNode::from(n.inner),
            Self::ArrayAssignmentTarget(n) => SiblingNode::from(n.inner),
            Self::ObjectAssignmentTarget(n) => SiblingNode::from(n.inner),
            Self::AssignmentTargetRest(n) => SiblingNode::from(n.inner),
            Self::AssignmentTargetWithDefault(n) => SiblingNode::from(n.inner),
            Self::AssignmentTargetPropertyIdentifier(n) => SiblingNode::from(n.inner),
            Self::AssignmentTargetPropertyProperty(n) => SiblingNode::from(n.inner),
            Self::SequenceExpression(n) => SiblingNode::from(n.inner),
            Self::Super(n) => SiblingNode::from(n.inner),
            Self::AwaitExpression(n) => SiblingNode::from(n.inner),
            Self::ChainExpression(n) => SiblingNode::from(n.inner),
            Self::ParenthesizedExpression(n) => SiblingNode::from(n.inner),
            Self::Directive(n) => SiblingNode::from(n.inner),
            Self::Hashbang(n) => SiblingNode::from(n.inner),
            Self::BlockStatement(n) => SiblingNode::from(n.inner),
            Self::VariableDeclaration(n) => SiblingNode::from(n.inner),
            Self::VariableDeclarator(n) => SiblingNode::from(n.inner),
            Self::EmptyStatement(n) => SiblingNode::from(n.inner),
            Self::ExpressionStatement(n) => SiblingNode::from(n.inner),
            Self::IfStatement(n) => SiblingNode::from(n.inner),
            Self::DoWhileStatement(n) => SiblingNode::from(n.inner),
            Self::WhileStatement(n) => SiblingNode::from(n.inner),
            Self::ForStatement(n) => SiblingNode::from(n.inner),
            Self::ForInStatement(n) => SiblingNode::from(n.inner),
            Self::ForOfStatement(n) => SiblingNode::from(n.inner),
            Self::ContinueStatement(n) => SiblingNode::from(n.inner),
            Self::BreakStatement(n) => SiblingNode::from(n.inner),
            Self::ReturnStatement(n) => SiblingNode::from(n.inner),
            Self::WithStatement(n) => SiblingNode::from(n.inner),
            Self::SwitchStatement(n) => SiblingNode::from(n.inner),
            Self::SwitchCase(n) => SiblingNode::from(n.inner),
            Self::LabeledStatement(n) => SiblingNode::from(n.inner),
            Self::ThrowStatement(n) => SiblingNode::from(n.inner),
            Self::TryStatement(n) => SiblingNode::from(n.inner),
            Self::CatchClause(n) => SiblingNode::from(n.inner),
            Self::CatchParameter(n) => SiblingNode::from(n.inner),
            Self::DebuggerStatement(n) => SiblingNode::from(n.inner),
            Self::AssignmentPattern(n) => SiblingNode::from(n.inner),
            Self::ObjectPattern(n) => SiblingNode::from(n.inner),
            Self::BindingProperty(n) => SiblingNode::from(n.inner),
            Self::ArrayPattern(n) => SiblingNode::from(n.inner),
            Self::BindingRestElement(n) => SiblingNode::from(n.inner),
            Self::Function(n) => SiblingNode::from(n.inner),
            Self::FormalParameters(n) => SiblingNode::from(n.inner),
            Self::FormalParameter(n) => SiblingNode::from(n.inner),
            Self::FunctionBody(n) => SiblingNode::from(n.inner),
            Self::ArrowFunctionExpression(n) => SiblingNode::from(n.inner),
            Self::YieldExpression(n) => SiblingNode::from(n.inner),
            Self::Class(n) => SiblingNode::from(n.inner),
            Self::ClassBody(n) => SiblingNode::from(n.inner),
            Self::MethodDefinition(n) => SiblingNode::from(n.inner),
            Self::PropertyDefinition(n) => SiblingNode::from(n.inner),
            Self::PrivateIdentifier(n) => SiblingNode::from(n.inner),
            Self::StaticBlock(n) => SiblingNode::from(n.inner),
            Self::AccessorProperty(n) => SiblingNode::from(n.inner),
            Self::ImportExpression(n) => SiblingNode::from(n.inner),
            Self::ImportDeclaration(n) => SiblingNode::from(n.inner),
            Self::ImportSpecifier(n) => SiblingNode::from(n.inner),
            Self::ImportDefaultSpecifier(n) => SiblingNode::from(n.inner),
            Self::ImportNamespaceSpecifier(n) => SiblingNode::from(n.inner),
            Self::WithClause(n) => SiblingNode::from(n.inner),
            Self::ImportAttribute(n) => SiblingNode::from(n.inner),
            Self::ExportNamedDeclaration(n) => SiblingNode::from(n.inner),
            Self::ExportDefaultDeclaration(n) => SiblingNode::from(n.inner),
            Self::ExportAllDeclaration(n) => SiblingNode::from(n.inner),
            Self::ExportSpecifier(n) => SiblingNode::from(n.inner),
            Self::V8IntrinsicExpression(n) => SiblingNode::from(n.inner),
            Self::BooleanLiteral(n) => SiblingNode::from(n.inner),
            Self::NullLiteral(n) => SiblingNode::from(n.inner),
            Self::NumericLiteral(n) => SiblingNode::from(n.inner),
            Self::StringLiteral(n) => SiblingNode::from(n.inner),
            Self::BigIntLiteral(n) => SiblingNode::from(n.inner),
            Self::RegExpLiteral(n) => SiblingNode::from(n.inner),
            Self::JSXElement(n) => SiblingNode::from(n.inner),
            Self::JSXOpeningElement(n) => SiblingNode::from(n.inner),
            Self::JSXClosingElement(n) => SiblingNode::from(n.inner),
            Self::JSXFragment(n) => SiblingNode::from(n.inner),
            Self::JSXOpeningFragment(n) => SiblingNode::from(n.inner),
            Self::JSXClosingFragment(n) => SiblingNode::from(n.inner),
            Self::JSXNamespacedName(n) => SiblingNode::from(n.inner),
            Self::JSXMemberExpression(n) => SiblingNode::from(n.inner),
            Self::JSXExpressionContainer(n) => SiblingNode::from(n.inner),
            Self::JSXEmptyExpression(n) => SiblingNode::from(n.inner),
            Self::JSXAttribute(n) => SiblingNode::from(n.inner),
            Self::JSXSpreadAttribute(n) => SiblingNode::from(n.inner),
            Self::JSXIdentifier(n) => SiblingNode::from(n.inner),
            Self::JSXSpreadChild(n) => SiblingNode::from(n.inner),
            Self::JSXText(n) => SiblingNode::from(n.inner),
            Self::TSThisParameter(n) => SiblingNode::from(n.inner),
            Self::TSEnumDeclaration(n) => SiblingNode::from(n.inner),
            Self::TSEnumBody(n) => SiblingNode::from(n.inner),
            Self::TSEnumMember(n) => SiblingNode::from(n.inner),
            Self::TSTypeAnnotation(n) => SiblingNode::from(n.inner),
            Self::TSLiteralType(n) => SiblingNode::from(n.inner),
            Self::TSConditionalType(n) => SiblingNode::from(n.inner),
            Self::TSUnionType(n) => SiblingNode::from(n.inner),
            Self::TSIntersectionType(n) => SiblingNode::from(n.inner),
            Self::TSParenthesizedType(n) => SiblingNode::from(n.inner),
            Self::TSTypeOperator(n) => SiblingNode::from(n.inner),
            Self::TSArrayType(n) => SiblingNode::from(n.inner),
            Self::TSIndexedAccessType(n) => SiblingNode::from(n.inner),
            Self::TSTupleType(n) => SiblingNode::from(n.inner),
            Self::TSNamedTupleMember(n) => SiblingNode::from(n.inner),
            Self::TSOptionalType(n) => SiblingNode::from(n.inner),
            Self::TSRestType(n) => SiblingNode::from(n.inner),
            Self::TSAnyKeyword(n) => SiblingNode::from(n.inner),
            Self::TSStringKeyword(n) => SiblingNode::from(n.inner),
            Self::TSBooleanKeyword(n) => SiblingNode::from(n.inner),
            Self::TSNumberKeyword(n) => SiblingNode::from(n.inner),
            Self::TSNeverKeyword(n) => SiblingNode::from(n.inner),
            Self::TSIntrinsicKeyword(n) => SiblingNode::from(n.inner),
            Self::TSUnknownKeyword(n) => SiblingNode::from(n.inner),
            Self::TSNullKeyword(n) => SiblingNode::from(n.inner),
            Self::TSUndefinedKeyword(n) => SiblingNode::from(n.inner),
            Self::TSVoidKeyword(n) => SiblingNode::from(n.inner),
            Self::TSSymbolKeyword(n) => SiblingNode::from(n.inner),
            Self::TSThisType(n) => SiblingNode::from(n.inner),
            Self::TSObjectKeyword(n) => SiblingNode::from(n.inner),
            Self::TSBigIntKeyword(n) => SiblingNode::from(n.inner),
            Self::TSTypeReference(n) => SiblingNode::from(n.inner),
            Self::TSQualifiedName(n) => SiblingNode::from(n.inner),
            Self::TSTypeParameterInstantiation(n) => SiblingNode::from(n.inner),
            Self::TSTypeParameter(n) => SiblingNode::from(n.inner),
            Self::TSTypeParameterDeclaration(n) => SiblingNode::from(n.inner),
            Self::TSTypeAliasDeclaration(n) => SiblingNode::from(n.inner),
            Self::TSClassImplements(n) => SiblingNode::from(n.inner),
            Self::TSInterfaceDeclaration(n) => SiblingNode::from(n.inner),
            Self::TSInterfaceBody(n) => SiblingNode::from(n.inner),
            Self::TSPropertySignature(n) => SiblingNode::from(n.inner),
            Self::TSIndexSignature(n) => SiblingNode::from(n.inner),
            Self::TSCallSignatureDeclaration(n) => SiblingNode::from(n.inner),
            Self::TSMethodSignature(n) => SiblingNode::from(n.inner),
            Self::TSConstructSignatureDeclaration(n) => SiblingNode::from(n.inner),
            Self::TSIndexSignatureName(n) => SiblingNode::from(n.inner),
            Self::TSInterfaceHeritage(n) => SiblingNode::from(n.inner),
            Self::TSTypePredicate(n) => SiblingNode::from(n.inner),
            Self::TSModuleDeclaration(n) => SiblingNode::from(n.inner),
            Self::TSModuleBlock(n) => SiblingNode::from(n.inner),
            Self::TSTypeLiteral(n) => SiblingNode::from(n.inner),
            Self::TSInferType(n) => SiblingNode::from(n.inner),
            Self::TSTypeQuery(n) => SiblingNode::from(n.inner),
            Self::TSImportType(n) => SiblingNode::from(n.inner),
            Self::TSImportTypeQualifiedName(n) => SiblingNode::from(n.inner),
            Self::TSFunctionType(n) => SiblingNode::from(n.inner),
            Self::TSConstructorType(n) => SiblingNode::from(n.inner),
            Self::TSMappedType(n) => SiblingNode::from(n.inner),
            Self::TSTemplateLiteralType(n) => SiblingNode::from(n.inner),
            Self::TSAsExpression(n) => SiblingNode::from(n.inner),
            Self::TSSatisfiesExpression(n) => SiblingNode::from(n.inner),
            Self::TSTypeAssertion(n) => SiblingNode::from(n.inner),
            Self::TSImportEqualsDeclaration(n) => SiblingNode::from(n.inner),
            Self::TSExternalModuleReference(n) => SiblingNode::from(n.inner),
            Self::TSNonNullExpression(n) => SiblingNode::from(n.inner),
            Self::Decorator(n) => SiblingNode::from(n.inner),
            Self::TSExportAssignment(n) => SiblingNode::from(n.inner),
            Self::TSNamespaceExportDeclaration(n) => SiblingNode::from(n.inner),
            Self::TSInstantiationExpression(n) => SiblingNode::from(n.inner),
            Self::JSDocNullableType(n) => SiblingNode::from(n.inner),
            Self::JSDocNonNullableType(n) => SiblingNode::from(n.inner),
            Self::JSDocUnknownType(n) => SiblingNode::from(n.inner),
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
            Self::Argument(_) => "Argument",
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

pub struct AstNode<'a, T> {
    pub(super) inner: &'a T,
    pub parent: &'a AstNodes<'a>,
    pub(super) allocator: &'a Allocator,
    pub(super) following_node: Option<SiblingNode<'a>>,
}
impl<T: GetSpan> GetSpan for &AstNode<'_, T> {
    fn span(&self) -> Span {
        self.inner.span()
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for AstNode<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AstNode")
            .field("inner", &self.inner)
            .field("parent", &self.parent.debug_name())
            .finish_non_exhaustive()
    }
}

impl<'a, T> Deref for AstNode<'a, T> {
    type Target = T;
    fn deref(&self) -> &'a Self::Target {
        self.inner
    }
}

impl<'a, T> AsRef<T> for AstNode<'a, T> {
    fn as_ref(&self) -> &'a T {
        self.inner
    }
}

impl<'a> AstNode<'a, Program<'a>> {
    pub fn new(inner: &'a Program<'a>, parent: &'a AstNodes<'a>, allocator: &'a Allocator) -> Self {
        AstNode { inner, parent, allocator, following_node: None }
    }
}

impl<'a, T> AstNode<'a, Option<T>> {
    pub fn as_ref(&self) -> Option<&'a AstNode<'a, T>> {
        self.allocator
            .alloc(self.inner.as_ref().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node,
            }))
            .as_ref()
    }
}

impl<'a> AstNode<'a, Program<'a>> {
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
    pub fn hashbang(&self) -> Option<&AstNode<'a, Hashbang<'a>>> {
        let following_node = self
            .inner
            .directives
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or_else(|| self.inner.body.first().as_ref().copied().map(SiblingNode::from))
            .or(self.following_node);
        self.allocator
            .alloc(self.inner.hashbang.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Program(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn directives(&self) -> &AstNode<'a, Vec<'a, Directive<'a>>> {
        let following_node = self
            .inner
            .body
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Program(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Program(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, Program<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, Expression<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            Expression::BooleanLiteral(s) => {
                AstNodes::BooleanLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::NullLiteral(s) => AstNodes::NullLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            Expression::NumericLiteral(s) => {
                AstNodes::NumericLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::BigIntLiteral(s) => {
                AstNodes::BigIntLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::RegExpLiteral(s) => {
                AstNodes::RegExpLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::TemplateLiteral(s) => {
                AstNodes::TemplateLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::Identifier(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::MetaProperty(s) => AstNodes::MetaProperty(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            Expression::Super(s) => AstNodes::Super(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            Expression::ArrayExpression(s) => {
                AstNodes::ArrayExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::ArrowFunctionExpression(s) => {
                AstNodes::ArrowFunctionExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::AssignmentExpression(s) => {
                AstNodes::AssignmentExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::AwaitExpression(s) => {
                AstNodes::AwaitExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::BinaryExpression(s) => {
                AstNodes::BinaryExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::CallExpression(s) => {
                AstNodes::CallExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::ChainExpression(s) => {
                AstNodes::ChainExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::ClassExpression(s) => AstNodes::Class(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            Expression::ConditionalExpression(s) => {
                AstNodes::ConditionalExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::FunctionExpression(s) => {
                AstNodes::Function(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::ImportExpression(s) => {
                AstNodes::ImportExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::LogicalExpression(s) => {
                AstNodes::LogicalExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::NewExpression(s) => {
                AstNodes::NewExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::ObjectExpression(s) => {
                AstNodes::ObjectExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::ParenthesizedExpression(s) => {
                AstNodes::ParenthesizedExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::SequenceExpression(s) => {
                AstNodes::SequenceExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::TaggedTemplateExpression(s) => {
                AstNodes::TaggedTemplateExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::ThisExpression(s) => {
                AstNodes::ThisExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::UnaryExpression(s) => {
                AstNodes::UnaryExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::UpdateExpression(s) => {
                AstNodes::UpdateExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::YieldExpression(s) => {
                AstNodes::YieldExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::PrivateInExpression(s) => {
                AstNodes::PrivateInExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::JSXElement(s) => AstNodes::JSXElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            Expression::JSXFragment(s) => AstNodes::JSXFragment(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            Expression::TSAsExpression(s) => {
                AstNodes::TSAsExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::TSSatisfiesExpression(s) => {
                AstNodes::TSSatisfiesExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::TSTypeAssertion(s) => {
                AstNodes::TSTypeAssertion(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::TSInstantiationExpression(s) => {
                AstNodes::TSInstantiationExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Expression::V8IntrinsicExpression(s) => {
                AstNodes::V8IntrinsicExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            it @ match_member_expression!(Expression) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_member_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, Expression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, IdentifierName<'a>> {
    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, IdentifierName<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, IdentifierReference<'a>> {
    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, IdentifierReference<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, BindingIdentifier<'a>> {
    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, BindingIdentifier<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, LabelIdentifier<'a>> {
    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, LabelIdentifier<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ThisExpression> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ThisExpression> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ArrayExpression<'a>> {
    #[inline]
    pub fn elements(&self) -> &AstNode<'a, Vec<'a, ArrayExpressionElement<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrayExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ArrayExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ArrayExpressionElement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ArrayExpressionElement::SpreadElement(s) => {
                AstNodes::SpreadElement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ArrayExpressionElement::Elision(s) => {
                AstNodes::Elision(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            it @ match_expression!(ArrayExpressionElement) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, ArrayExpressionElement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, Elision> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, Elision> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ObjectExpression<'a>> {
    #[inline]
    pub fn properties(&self) -> &AstNode<'a, Vec<'a, ObjectPropertyKind<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ObjectExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ObjectPropertyKind<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ObjectPropertyKind::ObjectProperty(s) => {
                AstNodes::ObjectProperty(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ObjectPropertyKind::SpreadProperty(s) => {
                AstNodes::SpreadElement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, ObjectPropertyKind<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ObjectProperty<'a>> {
    #[inline]
    pub fn kind(&self) -> PropertyKind {
        self.inner.kind
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.value));
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectProperty(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn value(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectProperty(transmute_self(self))),
            following_node,
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ObjectProperty<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, PropertyKey<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            PropertyKey::StaticIdentifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            PropertyKey::PrivateIdentifier(s) => {
                AstNodes::PrivateIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            it @ match_expression!(PropertyKey) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, PropertyKey<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TemplateLiteral<'a>> {
    #[inline]
    pub fn quasis(&self) -> &AstNode<'a, Vec<'a, TemplateElement<'a>>> {
        let following_node = self
            .inner
            .expressions
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasis,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TemplateLiteral(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn expressions(&self) -> &AstNode<'a, Vec<'a, Expression<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expressions,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TemplateLiteral(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TemplateLiteral<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TaggedTemplateExpression<'a>> {
    #[inline]
    pub fn tag(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self
            .inner
            .type_arguments
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| Some(SiblingNode::from(&self.inner.quasi)));
        self.allocator.alloc(AstNode {
            inner: &self.inner.tag,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TaggedTemplateExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = Some(SiblingNode::from(&self.inner.quasi));
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::TaggedTemplateExpression(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn quasi(&self) -> &AstNode<'a, TemplateLiteral<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasi,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TaggedTemplateExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TaggedTemplateExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TemplateElement<'a>> {
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TemplateElement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, MemberExpression<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            MemberExpression::ComputedMemberExpression(s) => {
                AstNodes::ComputedMemberExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            MemberExpression::StaticMemberExpression(s) => {
                AstNodes::StaticMemberExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            MemberExpression::PrivateFieldExpression(s) => {
                AstNodes::PrivateFieldExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, MemberExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ComputedMemberExpression<'a>> {
    #[inline]
    pub fn object(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.expression));
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ComputedMemberExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ComputedMemberExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ComputedMemberExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, StaticMemberExpression<'a>> {
    #[inline]
    pub fn object(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.property));
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::StaticMemberExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn property(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::StaticMemberExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, StaticMemberExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, PrivateFieldExpression<'a>> {
    #[inline]
    pub fn object(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.field));
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PrivateFieldExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn field(&self) -> &AstNode<'a, PrivateIdentifier<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.field,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PrivateFieldExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, PrivateFieldExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, CallExpression<'a>> {
    #[inline]
    pub fn callee(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self
            .inner
            .type_arguments
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.arguments.first().as_ref().copied().map(SiblingNode::from))
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.callee,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::CallExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = self
            .inner
            .arguments
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or(self.following_node);
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::CallExpression(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn arguments(&self) -> &AstNode<'a, Vec<'a, Argument<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::CallExpression(transmute_self(self))),
            following_node,
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, CallExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, NewExpression<'a>> {
    #[inline]
    pub fn callee(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self
            .inner
            .type_arguments
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.arguments.first().as_ref().copied().map(SiblingNode::from))
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.callee,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::NewExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = self
            .inner
            .arguments
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or(self.following_node);
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::NewExpression(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn arguments(&self) -> &AstNode<'a, Vec<'a, Argument<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::NewExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn pure(&self) -> bool {
        self.inner.pure
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, NewExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, MetaProperty<'a>> {
    #[inline]
    pub fn meta(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.property));
        self.allocator.alloc(AstNode {
            inner: &self.inner.meta,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MetaProperty(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn property(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MetaProperty(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, MetaProperty<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, SpreadElement<'a>> {
    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SpreadElement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, SpreadElement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, Argument<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.allocator.alloc(AstNodes::Argument(transmute_self(self)));
        let node = match self.inner {
            Argument::SpreadElement(s) => AstNodes::SpreadElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            it @ match_expression!(Argument) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, Argument<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, UpdateExpression<'a>> {
    #[inline]
    pub fn operator(&self) -> UpdateOperator {
        self.inner.operator
    }

    #[inline]
    pub fn prefix(&self) -> bool {
        self.inner.prefix
    }

    #[inline]
    pub fn argument(&self) -> &AstNode<'a, SimpleAssignmentTarget<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::UpdateExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, UpdateExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, UnaryExpression<'a>> {
    #[inline]
    pub fn operator(&self) -> UnaryOperator {
        self.inner.operator
    }

    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::UnaryExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, UnaryExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, BinaryExpression<'a>> {
    #[inline]
    pub fn left(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.right));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BinaryExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn operator(&self) -> BinaryOperator {
        self.inner.operator
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BinaryExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, BinaryExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, PrivateInExpression<'a>> {
    #[inline]
    pub fn left(&self) -> &AstNode<'a, PrivateIdentifier<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.right));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PrivateInExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PrivateInExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, PrivateInExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, LogicalExpression<'a>> {
    #[inline]
    pub fn left(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.right));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LogicalExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn operator(&self) -> LogicalOperator {
        self.inner.operator
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LogicalExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, LogicalExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ConditionalExpression<'a>> {
    #[inline]
    pub fn test(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.consequent));
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ConditionalExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn consequent(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.alternate));
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ConditionalExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn alternate(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.alternate,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ConditionalExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ConditionalExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, AssignmentExpression<'a>> {
    #[inline]
    pub fn operator(&self) -> AssignmentOperator {
        self.inner.operator
    }

    #[inline]
    pub fn left(&self) -> &AstNode<'a, AssignmentTarget<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.right));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, AssignmentExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, AssignmentTarget<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_simple_assignment_target(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_assignment_target_pattern(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, AssignmentTarget<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, SimpleAssignmentTarget<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            SimpleAssignmentTarget::TSAsExpression(s) => {
                AstNodes::TSAsExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(s) => {
                AstNodes::TSSatisfiesExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            SimpleAssignmentTarget::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            SimpleAssignmentTarget::TSTypeAssertion(s) => {
                AstNodes::TSTypeAssertion(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_member_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, SimpleAssignmentTarget<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, AssignmentTargetPattern<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            AssignmentTargetPattern::ArrayAssignmentTarget(s) => {
                AstNodes::ArrayAssignmentTarget(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(s) => {
                AstNodes::ObjectAssignmentTarget(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, AssignmentTargetPattern<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ArrayAssignmentTarget<'a>> {
    #[inline]
    pub fn elements(&self) -> &AstNode<'a, Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>> {
        let following_node =
            self.inner.rest.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrayAssignmentTarget(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, AssignmentTargetRest<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ArrayAssignmentTarget(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ArrayAssignmentTarget<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ObjectAssignmentTarget<'a>> {
    #[inline]
    pub fn properties(&self) -> &AstNode<'a, Vec<'a, AssignmentTargetProperty<'a>>> {
        let following_node =
            self.inner.rest.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectAssignmentTarget(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, AssignmentTargetRest<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::ObjectAssignmentTarget(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ObjectAssignmentTarget<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, AssignmentTargetRest<'a>> {
    #[inline]
    pub fn target(&self) -> &AstNode<'a, AssignmentTarget<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.target,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentTargetRest(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, AssignmentTargetRest<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, AssignmentTargetMaybeDefault<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(s) => {
                AstNodes::AssignmentTargetWithDefault(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_assignment_target(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, AssignmentTargetMaybeDefault<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, AssignmentTargetWithDefault<'a>> {
    #[inline]
    pub fn binding(&self) -> &AstNode<'a, AssignmentTarget<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.init));
        self.allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::AssignmentTargetWithDefault(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn init(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.init,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::AssignmentTargetWithDefault(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, AssignmentTargetWithDefault<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, AssignmentTargetProperty<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(s) => {
                AstNodes::AssignmentTargetPropertyIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(s) => {
                AstNodes::AssignmentTargetPropertyProperty(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, AssignmentTargetProperty<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, AssignmentTargetPropertyIdentifier<'a>> {
    #[inline]
    pub fn binding(&self) -> &AstNode<'a, IdentifierReference<'a>> {
        let following_node =
            self.inner.init.as_ref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::AssignmentTargetPropertyIdentifier(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn init(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.init.as_ref().map(|inner| {
                AstNode {
                    inner,
                    allocator: self.allocator,
                    parent: self
                        .allocator
                        .alloc(AstNodes::AssignmentTargetPropertyIdentifier(transmute_self(self))),
                    following_node,
                }
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, AssignmentTargetPropertyIdentifier<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, AssignmentTargetPropertyProperty<'a>> {
    #[inline]
    pub fn name(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.binding));
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::AssignmentTargetPropertyProperty(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn binding(&self) -> &AstNode<'a, AssignmentTargetMaybeDefault<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::AssignmentTargetPropertyProperty(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, AssignmentTargetPropertyProperty<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, SequenceExpression<'a>> {
    #[inline]
    pub fn expressions(&self) -> &AstNode<'a, Vec<'a, Expression<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expressions,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SequenceExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, SequenceExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, Super> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, Super> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, AwaitExpression<'a>> {
    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AwaitExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, AwaitExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ChainExpression<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, ChainElement<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ChainExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ChainExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ChainElement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ChainElement::CallExpression(s) => {
                AstNodes::CallExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ChainElement::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            it @ match_member_expression!(ChainElement) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_member_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, ChainElement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ParenthesizedExpression<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ParenthesizedExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ParenthesizedExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, Statement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            Statement::BlockStatement(s) => {
                AstNodes::BlockStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Statement::BreakStatement(s) => {
                AstNodes::BreakStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Statement::ContinueStatement(s) => {
                AstNodes::ContinueStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Statement::DebuggerStatement(s) => {
                AstNodes::DebuggerStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Statement::DoWhileStatement(s) => {
                AstNodes::DoWhileStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Statement::EmptyStatement(s) => {
                AstNodes::EmptyStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Statement::ExpressionStatement(s) => {
                AstNodes::ExpressionStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Statement::ForInStatement(s) => {
                AstNodes::ForInStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Statement::ForOfStatement(s) => {
                AstNodes::ForOfStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Statement::ForStatement(s) => AstNodes::ForStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            Statement::IfStatement(s) => AstNodes::IfStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            Statement::LabeledStatement(s) => {
                AstNodes::LabeledStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Statement::ReturnStatement(s) => {
                AstNodes::ReturnStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Statement::SwitchStatement(s) => {
                AstNodes::SwitchStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Statement::ThrowStatement(s) => {
                AstNodes::ThrowStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Statement::TryStatement(s) => AstNodes::TryStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            Statement::WhileStatement(s) => {
                AstNodes::WhileStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Statement::WithStatement(s) => AstNodes::WithStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            it @ match_declaration!(Statement) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_declaration(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
            it @ match_module_declaration!(Statement) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_module_declaration(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, Statement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, Directive<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, StringLiteral<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Directive(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn directive(&self) -> Atom<'a> {
        self.inner.directive
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, Directive<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, Hashbang<'a>> {
    #[inline]
    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, Hashbang<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, BlockStatement<'a>> {
    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BlockStatement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, BlockStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, Declaration<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            Declaration::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Declaration::FunctionDeclaration(s) => {
                AstNodes::Function(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Declaration::ClassDeclaration(s) => AstNodes::Class(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            Declaration::TSTypeAliasDeclaration(s) => {
                AstNodes::TSTypeAliasDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Declaration::TSInterfaceDeclaration(s) => {
                AstNodes::TSInterfaceDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Declaration::TSEnumDeclaration(s) => {
                AstNodes::TSEnumDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Declaration::TSModuleDeclaration(s) => {
                AstNodes::TSModuleDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            Declaration::TSImportEqualsDeclaration(s) => {
                AstNodes::TSImportEqualsDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, Declaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, VariableDeclaration<'a>> {
    #[inline]
    pub fn kind(&self) -> VariableDeclarationKind {
        self.inner.kind
    }

    #[inline]
    pub fn declarations(&self) -> &AstNode<'a, Vec<'a, VariableDeclarator<'a>>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.declarations,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::VariableDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, VariableDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, VariableDeclarator<'a>> {
    #[inline]
    pub fn kind(&self) -> VariableDeclarationKind {
        self.inner.kind
    }

    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_node =
            self.inner.init.as_ref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::VariableDeclarator(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn init(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::VariableDeclarator(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn definite(&self) -> bool {
        self.inner.definite
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, VariableDeclarator<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, EmptyStatement> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, EmptyStatement> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ExpressionStatement<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExpressionStatement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ExpressionStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, IfStatement<'a>> {
    #[inline]
    pub fn test(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.consequent));
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::IfStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn consequent(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = self.inner.alternate.as_ref().map(SiblingNode::from);
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::IfStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn alternate(&self) -> Option<&AstNode<'a, Statement<'a>>> {
        let following_node = None;
        self.allocator
            .alloc(self.inner.alternate.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::IfStatement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, IfStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, DoWhileStatement<'a>> {
    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.test));
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::DoWhileStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn test(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::DoWhileStatement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, DoWhileStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, WhileStatement<'a>> {
    #[inline]
    pub fn test(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.body));
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WhileStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WhileStatement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, WhileStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ForStatement<'a>> {
    #[inline]
    pub fn init(&self) -> Option<&AstNode<'a, ForStatementInit<'a>>> {
        let following_node = self
            .inner
            .test
            .as_ref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.update.as_ref().map(SiblingNode::from))
            .or_else(|| Some(SiblingNode::from(&self.inner.body)));
        self.allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ForStatement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn test(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self
            .inner
            .update
            .as_ref()
            .map(SiblingNode::from)
            .or_else(|| Some(SiblingNode::from(&self.inner.body)));
        self.allocator
            .alloc(self.inner.test.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ForStatement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn update(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = Some(SiblingNode::from(&self.inner.body));
        self.allocator
            .alloc(self.inner.update.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ForStatement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForStatement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ForStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ForStatementInit<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ForStatementInit::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            it @ match_expression!(ForStatementInit) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, ForStatementInit<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ForInStatement<'a>> {
    #[inline]
    pub fn left(&self) -> &AstNode<'a, ForStatementLeft<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.right));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForInStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.body));
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForInStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForInStatement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ForInStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ForStatementLeft<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ForStatementLeft::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            it @ match_assignment_target!(ForStatementLeft) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_assignment_target(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, ForStatementLeft<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ForOfStatement<'a>> {
    #[inline]
    pub fn r#await(&self) -> bool {
        self.inner.r#await
    }

    #[inline]
    pub fn left(&self) -> &AstNode<'a, ForStatementLeft<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.right));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForOfStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.body));
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForOfStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForOfStatement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ForOfStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ContinueStatement<'a>> {
    #[inline]
    pub fn label(&self) -> Option<&AstNode<'a, LabelIdentifier<'a>>> {
        let following_node = None;
        self.allocator
            .alloc(self.inner.label.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ContinueStatement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ContinueStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, BreakStatement<'a>> {
    #[inline]
    pub fn label(&self) -> Option<&AstNode<'a, LabelIdentifier<'a>>> {
        let following_node = None;
        self.allocator
            .alloc(self.inner.label.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::BreakStatement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, BreakStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ReturnStatement<'a>> {
    #[inline]
    pub fn argument(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = None;
        self.allocator
            .alloc(self.inner.argument.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ReturnStatement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ReturnStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, WithStatement<'a>> {
    #[inline]
    pub fn object(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.body));
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WithStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WithStatement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, WithStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, SwitchStatement<'a>> {
    #[inline]
    pub fn discriminant(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.inner.cases.first().as_ref().copied().map(SiblingNode::from);
        self.allocator.alloc(AstNode {
            inner: &self.inner.discriminant,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SwitchStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn cases(&self) -> &AstNode<'a, Vec<'a, SwitchCase<'a>>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.cases,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SwitchStatement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, SwitchStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, SwitchCase<'a>> {
    #[inline]
    pub fn test(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self
            .inner
            .consequent
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or(self.following_node);
        self.allocator
            .alloc(self.inner.test.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::SwitchCase(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn consequent(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SwitchCase(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, SwitchCase<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, LabeledStatement<'a>> {
    #[inline]
    pub fn label(&self) -> &AstNode<'a, LabelIdentifier<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.body));
        self.allocator.alloc(AstNode {
            inner: &self.inner.label,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LabeledStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LabeledStatement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, LabeledStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ThrowStatement<'a>> {
    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ThrowStatement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ThrowStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TryStatement<'a>> {
    #[inline]
    pub fn block(&self) -> &AstNode<'a, BlockStatement<'a>> {
        let following_node = self
            .inner
            .handler
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.finalizer.as_deref().map(SiblingNode::from));
        self.allocator.alloc(AstNode {
            inner: self.inner.block.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TryStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn handler(&self) -> Option<&AstNode<'a, CatchClause<'a>>> {
        let following_node = self.inner.finalizer.as_deref().map(SiblingNode::from);
        self.allocator
            .alloc(self.inner.handler.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TryStatement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn finalizer(&self) -> Option<&AstNode<'a, BlockStatement<'a>>> {
        let following_node = None;
        self.allocator
            .alloc(self.inner.finalizer.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TryStatement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TryStatement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, CatchClause<'a>> {
    #[inline]
    pub fn param(&self) -> Option<&AstNode<'a, CatchParameter<'a>>> {
        let following_node = Some(SiblingNode::from(self.inner.body.as_ref()));
        self.allocator
            .alloc(self.inner.param.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::CatchClause(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, BlockStatement<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::CatchClause(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, CatchClause<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, CatchParameter<'a>> {
    #[inline]
    pub fn pattern(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.pattern,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::CatchParameter(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, CatchParameter<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, DebuggerStatement> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, DebuggerStatement> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, BindingPattern<'a>> {
    #[inline]
    pub fn kind(&self) -> &AstNode<'a, BindingPatternKind<'a>> {
        let following_node =
            self.inner.type_annotation.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.kind,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, BindingPattern<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, BindingPatternKind<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            BindingPatternKind::BindingIdentifier(s) => {
                AstNodes::BindingIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            BindingPatternKind::ObjectPattern(s) => {
                AstNodes::ObjectPattern(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            BindingPatternKind::ArrayPattern(s) => {
                AstNodes::ArrayPattern(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            BindingPatternKind::AssignmentPattern(s) => {
                AstNodes::AssignmentPattern(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, BindingPatternKind<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, AssignmentPattern<'a>> {
    #[inline]
    pub fn left(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.right));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentPattern(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentPattern(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, AssignmentPattern<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ObjectPattern<'a>> {
    #[inline]
    pub fn properties(&self) -> &AstNode<'a, Vec<'a, BindingProperty<'a>>> {
        let following_node =
            self.inner.rest.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectPattern(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, BindingRestElement<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ObjectPattern(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ObjectPattern<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, BindingProperty<'a>> {
    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.value));
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BindingProperty(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn value(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BindingProperty(transmute_self(self))),
            following_node,
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, BindingProperty<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ArrayPattern<'a>> {
    #[inline]
    pub fn elements(&self) -> &AstNode<'a, Vec<'a, Option<BindingPattern<'a>>>> {
        let following_node =
            self.inner.rest.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrayPattern(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, BindingRestElement<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ArrayPattern(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ArrayPattern<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, BindingRestElement<'a>> {
    #[inline]
    pub fn argument(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BindingRestElement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, BindingRestElement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, Function<'a>> {
    #[inline]
    pub fn r#type(&self) -> FunctionType {
        self.inner.r#type
    }

    #[inline]
    pub fn id(&self) -> Option<&AstNode<'a, BindingIdentifier<'a>>> {
        let following_node = self
            .inner
            .type_parameters
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.this_param.as_deref().map(SiblingNode::from))
            .or_else(|| Some(SiblingNode::from(self.inner.params.as_ref())));
        self.allocator
            .alloc(self.inner.id.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(transmute_self(self))),
                following_node,
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
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = self
            .inner
            .this_param
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| Some(SiblingNode::from(self.inner.params.as_ref())));
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn this_param(&self) -> Option<&AstNode<'a, TSThisParameter<'a>>> {
        let following_node = Some(SiblingNode::from(self.inner.params.as_ref()));
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_node = self
            .inner
            .return_type
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.body.as_deref().map(SiblingNode::from))
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Function(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn return_type(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node =
            self.inner.body.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn body(&self) -> Option<&AstNode<'a, FunctionBody<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.body.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Function(transmute_self(self))),
                following_node,
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, Function<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, FormalParameters<'a>> {
    #[inline]
    pub fn kind(&self) -> FormalParameterKind {
        self.inner.kind
    }

    #[inline]
    pub fn items(&self) -> &AstNode<'a, Vec<'a, FormalParameter<'a>>> {
        let following_node = self.inner.rest.as_deref().map(SiblingNode::from);
        self.allocator.alloc(AstNode {
            inner: &self.inner.items,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FormalParameters(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, BindingRestElement<'a>>> {
        let following_node = None;
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::FormalParameters(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, FormalParameters<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, FormalParameter<'a>> {
    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_node = Some(SiblingNode::from(&self.inner.pattern));
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FormalParameter(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn pattern(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.pattern,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FormalParameter(transmute_self(self))),
            following_node,
        })
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, FormalParameter<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, FunctionBody<'a>> {
    #[inline]
    pub fn directives(&self) -> &AstNode<'a, Vec<'a, Directive<'a>>> {
        let following_node = self
            .inner
            .statements
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FunctionBody(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn statements(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.statements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FunctionBody(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, FunctionBody<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ArrowFunctionExpression<'a>> {
    #[inline]
    pub fn expression(&self) -> bool {
        self.inner.expression
    }

    #[inline]
    pub fn r#async(&self) -> bool {
        self.inner.r#async
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = Some(SiblingNode::from(self.inner.params.as_ref()));
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::ArrowFunctionExpression(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_node = self
            .inner
            .return_type
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| Some(SiblingNode::from(self.inner.body.as_ref())));
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrowFunctionExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn return_type(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = Some(SiblingNode::from(self.inner.body.as_ref()));
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::ArrowFunctionExpression(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, FunctionBody<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrowFunctionExpression(transmute_self(self))),
            following_node,
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ArrowFunctionExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, YieldExpression<'a>> {
    #[inline]
    pub fn delegate(&self) -> bool {
        self.inner.delegate
    }

    #[inline]
    pub fn argument(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.argument.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::YieldExpression(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, YieldExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, Class<'a>> {
    #[inline]
    pub fn r#type(&self) -> ClassType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_node = self
            .inner
            .id
            .as_ref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.type_parameters.as_deref().map(SiblingNode::from))
            .or_else(|| self.inner.super_class.as_ref().map(SiblingNode::from))
            .or_else(|| self.inner.super_type_arguments.as_deref().map(SiblingNode::from))
            .or_else(|| self.inner.implements.first().as_ref().copied().map(SiblingNode::from))
            .or_else(|| Some(SiblingNode::from(self.inner.body.as_ref())));
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn id(&self) -> Option<&AstNode<'a, BindingIdentifier<'a>>> {
        let following_node = self
            .inner
            .type_parameters
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.super_class.as_ref().map(SiblingNode::from))
            .or_else(|| self.inner.super_type_arguments.as_deref().map(SiblingNode::from))
            .or_else(|| self.inner.implements.first().as_ref().copied().map(SiblingNode::from))
            .or_else(|| Some(SiblingNode::from(self.inner.body.as_ref())));
        self.allocator
            .alloc(self.inner.id.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = self
            .inner
            .super_class
            .as_ref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.super_type_arguments.as_deref().map(SiblingNode::from))
            .or_else(|| self.inner.implements.first().as_ref().copied().map(SiblingNode::from))
            .or_else(|| Some(SiblingNode::from(self.inner.body.as_ref())));
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn super_class(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self
            .inner
            .super_type_arguments
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.implements.first().as_ref().copied().map(SiblingNode::from))
            .or_else(|| Some(SiblingNode::from(self.inner.body.as_ref())));
        self.allocator
            .alloc(self.inner.super_class.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn super_type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = self
            .inner
            .implements
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or_else(|| Some(SiblingNode::from(self.inner.body.as_ref())));
        self.allocator
            .alloc(self.inner.super_type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn implements(&self) -> &AstNode<'a, Vec<'a, TSClassImplements<'a>>> {
        let following_node = Some(SiblingNode::from(self.inner.body.as_ref()));
        self.allocator.alloc(AstNode {
            inner: &self.inner.implements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, ClassBody<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
            following_node,
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, Class<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ClassBody<'a>> {
    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, ClassElement<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ClassBody(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ClassBody<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ClassElement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ClassElement::StaticBlock(s) => AstNodes::StaticBlock(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            ClassElement::MethodDefinition(s) => {
                AstNodes::MethodDefinition(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ClassElement::PropertyDefinition(s) => {
                AstNodes::PropertyDefinition(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ClassElement::AccessorProperty(s) => {
                AstNodes::AccessorProperty(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ClassElement::TSIndexSignature(s) => {
                AstNodes::TSIndexSignature(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, ClassElement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, MethodDefinition<'a>> {
    #[inline]
    pub fn r#type(&self) -> MethodDefinitionType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_node = Some(SiblingNode::from(&self.inner.key));
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MethodDefinition(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = Some(SiblingNode::from(self.inner.value.as_ref()));
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MethodDefinition(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn value(&self) -> &AstNode<'a, Function<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: self.inner.value.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MethodDefinition(transmute_self(self))),
            following_node,
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, MethodDefinition<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, PropertyDefinition<'a>> {
    #[inline]
    pub fn r#type(&self) -> PropertyDefinitionType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_node = Some(SiblingNode::from(&self.inner.key));
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PropertyDefinition(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = self
            .inner
            .type_annotation
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.value.as_ref().map(SiblingNode::from))
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PropertyDefinition(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node =
            self.inner.value.as_ref().map(SiblingNode::from).or(self.following_node);
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::PropertyDefinition(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn value(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::PropertyDefinition(transmute_self(self))),
                following_node,
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, PropertyDefinition<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, PrivateIdentifier<'a>> {
    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, PrivateIdentifier<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, StaticBlock<'a>> {
    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::StaticBlock(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, StaticBlock<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ModuleDeclaration<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ModuleDeclaration::ImportDeclaration(s) => {
                AstNodes::ImportDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ModuleDeclaration::ExportAllDeclaration(s) => {
                AstNodes::ExportAllDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ModuleDeclaration::ExportDefaultDeclaration(s) => {
                AstNodes::ExportDefaultDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ModuleDeclaration::ExportNamedDeclaration(s) => {
                AstNodes::ExportNamedDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ModuleDeclaration::TSExportAssignment(s) => {
                AstNodes::TSExportAssignment(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(s) => {
                AstNodes::TSNamespaceExportDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, ModuleDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, AccessorProperty<'a>> {
    #[inline]
    pub fn r#type(&self) -> AccessorPropertyType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_node = Some(SiblingNode::from(&self.inner.key));
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AccessorProperty(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = self
            .inner
            .type_annotation
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.value.as_ref().map(SiblingNode::from))
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AccessorProperty(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node =
            self.inner.value.as_ref().map(SiblingNode::from).or(self.following_node);
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::AccessorProperty(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn value(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::AccessorProperty(transmute_self(self))),
                following_node,
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, AccessorProperty<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ImportExpression<'a>> {
    #[inline]
    pub fn source(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node =
            self.inner.options.as_ref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn options(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.options.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ImportExpression(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn phase(&self) -> Option<ImportPhase> {
        self.inner.phase
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ImportExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ImportDeclaration<'a>> {
    #[inline]
    pub fn specifiers(&self) -> Option<&AstNode<'a, Vec<'a, ImportDeclarationSpecifier<'a>>>> {
        let following_node = Some(SiblingNode::from(&self.inner.source));
        self.allocator
            .alloc(self.inner.specifiers.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ImportDeclaration(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn source(&self) -> &AstNode<'a, StringLiteral<'a>> {
        let following_node = self.inner.with_clause.as_deref().map(SiblingNode::from);
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn phase(&self) -> Option<ImportPhase> {
        self.inner.phase
    }

    #[inline]
    pub fn with_clause(&self) -> Option<&AstNode<'a, WithClause<'a>>> {
        let following_node = None;
        self.allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ImportDeclaration(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ImportDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ImportDeclarationSpecifier<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                AstNodes::ImportSpecifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                AstNodes::ImportDefaultSpecifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                AstNodes::ImportNamespaceSpecifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, ImportDeclarationSpecifier<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ImportSpecifier<'a>> {
    #[inline]
    pub fn imported(&self) -> &AstNode<'a, ModuleExportName<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.local));
        self.allocator.alloc(AstNode {
            inner: &self.inner.imported,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportSpecifier(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn local(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportSpecifier(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ImportSpecifier<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ImportDefaultSpecifier<'a>> {
    #[inline]
    pub fn local(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportDefaultSpecifier(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ImportDefaultSpecifier<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ImportNamespaceSpecifier<'a>> {
    #[inline]
    pub fn local(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportNamespaceSpecifier(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ImportNamespaceSpecifier<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, WithClause<'a>> {
    #[inline]
    pub fn keyword(&self) -> WithClauseKeyword {
        self.inner.keyword
    }

    #[inline]
    pub fn with_entries(&self) -> &AstNode<'a, Vec<'a, ImportAttribute<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.with_entries,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WithClause(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, WithClause<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ImportAttribute<'a>> {
    #[inline]
    pub fn key(&self) -> &AstNode<'a, ImportAttributeKey<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.value));
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportAttribute(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn value(&self) -> &AstNode<'a, StringLiteral<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportAttribute(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ImportAttribute<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ImportAttributeKey<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ImportAttributeKey::Identifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ImportAttributeKey::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, ImportAttributeKey<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ExportNamedDeclaration<'a>> {
    #[inline]
    pub fn declaration(&self) -> Option<&AstNode<'a, Declaration<'a>>> {
        let following_node = self
            .inner
            .specifiers
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or_else(|| self.inner.source.as_ref().map(SiblingNode::from))
            .or_else(|| self.inner.with_clause.as_deref().map(SiblingNode::from));
        self.allocator
            .alloc(self.inner.declaration.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::ExportNamedDeclaration(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn specifiers(&self) -> &AstNode<'a, Vec<'a, ExportSpecifier<'a>>> {
        let following_node = self
            .inner
            .source
            .as_ref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.with_clause.as_deref().map(SiblingNode::from));
        self.allocator.alloc(AstNode {
            inner: &self.inner.specifiers,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportNamedDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn source(&self) -> Option<&AstNode<'a, StringLiteral<'a>>> {
        let following_node = self.inner.with_clause.as_deref().map(SiblingNode::from);
        self.allocator
            .alloc(self.inner.source.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::ExportNamedDeclaration(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    #[inline]
    pub fn with_clause(&self) -> Option<&AstNode<'a, WithClause<'a>>> {
        let following_node = None;
        self.allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::ExportNamedDeclaration(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ExportNamedDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ExportDefaultDeclaration<'a>> {
    #[inline]
    pub fn declaration(&self) -> &AstNode<'a, ExportDefaultDeclarationKind<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.declaration,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportDefaultDeclaration(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ExportDefaultDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ExportAllDeclaration<'a>> {
    #[inline]
    pub fn exported(&self) -> Option<&AstNode<'a, ModuleExportName<'a>>> {
        let following_node = Some(SiblingNode::from(&self.inner.source));
        self.allocator
            .alloc(self.inner.exported.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ExportAllDeclaration(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn source(&self) -> &AstNode<'a, StringLiteral<'a>> {
        let following_node = self.inner.with_clause.as_deref().map(SiblingNode::from);
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportAllDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn with_clause(&self) -> Option<&AstNode<'a, WithClause<'a>>> {
        let following_node = None;
        self.allocator
            .alloc(self.inner.with_clause.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ExportAllDeclaration(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ExportAllDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ExportSpecifier<'a>> {
    #[inline]
    pub fn local(&self) -> &AstNode<'a, ModuleExportName<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.exported));
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportSpecifier(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn exported(&self) -> &AstNode<'a, ModuleExportName<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.exported,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportSpecifier(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn export_kind(&self) -> ImportOrExportKind {
        self.inner.export_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, ExportSpecifier<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ExportDefaultDeclarationKind<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ExportDefaultDeclarationKind::FunctionDeclaration(s) => {
                AstNodes::Function(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ExportDefaultDeclarationKind::ClassDeclaration(s) => {
                AstNodes::Class(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(s) => {
                AstNodes::TSInterfaceDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            it @ match_expression!(ExportDefaultDeclarationKind) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, ExportDefaultDeclarationKind<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, ModuleExportName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            ModuleExportName::IdentifierName(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ModuleExportName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            ModuleExportName::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, ModuleExportName<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, V8IntrinsicExpression<'a>> {
    #[inline]
    pub fn name(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = self
            .inner
            .arguments
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::V8IntrinsicExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn arguments(&self) -> &AstNode<'a, Vec<'a, Argument<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::V8IntrinsicExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, V8IntrinsicExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, BooleanLiteral> {
    #[inline]
    pub fn value(&self) -> bool {
        self.inner.value
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, BooleanLiteral> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, NullLiteral> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, NullLiteral> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, NumericLiteral<'a>> {
    #[inline]
    pub fn value(&self) -> f64 {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }

    #[inline]
    pub fn base(&self) -> NumberBase {
        self.inner.base
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, NumericLiteral<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, StringLiteral<'a>> {
    #[inline]
    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }

    #[inline]
    pub fn lone_surrogates(&self) -> bool {
        self.inner.lone_surrogates
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, StringLiteral<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, BigIntLiteral<'a>> {
    #[inline]
    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }

    #[inline]
    pub fn base(&self) -> BigintBase {
        self.inner.base
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, BigIntLiteral<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, RegExpLiteral<'a>> {
    #[inline]
    pub fn regex(&self) -> &RegExp<'a> {
        &self.inner.regex
    }

    #[inline]
    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, RegExpLiteral<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXElement<'a>> {
    #[inline]
    pub fn opening_element(&self) -> &AstNode<'a, JSXOpeningElement<'a>> {
        let following_node = self
            .inner
            .children
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or_else(|| self.inner.closing_element.as_deref().map(SiblingNode::from))
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: self.inner.opening_element.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXElement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn children(&self) -> &AstNode<'a, Vec<'a, JSXChild<'a>>> {
        let following_node =
            self.inner.closing_element.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.children,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXElement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn closing_element(&self) -> Option<&AstNode<'a, JSXClosingElement<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.closing_element.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::JSXElement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXElement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXOpeningElement<'a>> {
    #[inline]
    pub fn name(&self) -> &AstNode<'a, JSXElementName<'a>> {
        let following_node = self
            .inner
            .type_arguments
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.attributes.first().as_ref().copied().map(SiblingNode::from))
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXOpeningElement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = self
            .inner
            .attributes
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or(self.following_node);
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::JSXOpeningElement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn attributes(&self) -> &AstNode<'a, Vec<'a, JSXAttributeItem<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.attributes,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXOpeningElement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXOpeningElement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXClosingElement<'a>> {
    #[inline]
    pub fn name(&self) -> &AstNode<'a, JSXElementName<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXClosingElement(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXClosingElement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXFragment<'a>> {
    #[inline]
    pub fn opening_fragment(&self) -> &AstNode<'a, JSXOpeningFragment> {
        let following_node = self
            .inner
            .children
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or_else(|| Some(SiblingNode::from(&self.inner.closing_fragment)));
        self.allocator.alloc(AstNode {
            inner: &self.inner.opening_fragment,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXFragment(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn children(&self) -> &AstNode<'a, Vec<'a, JSXChild<'a>>> {
        let following_node = Some(SiblingNode::from(&self.inner.closing_fragment));
        self.allocator.alloc(AstNode {
            inner: &self.inner.children,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXFragment(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn closing_fragment(&self) -> &AstNode<'a, JSXClosingFragment> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.closing_fragment,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXFragment(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXFragment<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXOpeningFragment> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXOpeningFragment> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXClosingFragment> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXClosingFragment> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXElementName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            JSXElementName::Identifier(s) => {
                AstNodes::JSXIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            JSXElementName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            JSXElementName::NamespacedName(s) => {
                AstNodes::JSXNamespacedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            JSXElementName::MemberExpression(s) => {
                AstNodes::JSXMemberExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            JSXElementName::ThisExpression(s) => {
                AstNodes::ThisExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXElementName<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXNamespacedName<'a>> {
    #[inline]
    pub fn namespace(&self) -> &AstNode<'a, JSXIdentifier<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.name));
        self.allocator.alloc(AstNode {
            inner: &self.inner.namespace,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXNamespacedName(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn name(&self) -> &AstNode<'a, JSXIdentifier<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXNamespacedName(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXNamespacedName<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXMemberExpression<'a>> {
    #[inline]
    pub fn object(&self) -> &AstNode<'a, JSXMemberExpressionObject<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.property));
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXMemberExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn property(&self) -> &AstNode<'a, JSXIdentifier<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXMemberExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXMemberExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXMemberExpressionObject<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            JSXMemberExpressionObject::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            JSXMemberExpressionObject::MemberExpression(s) => {
                AstNodes::JSXMemberExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            JSXMemberExpressionObject::ThisExpression(s) => {
                AstNodes::ThisExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXMemberExpressionObject<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXExpressionContainer<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, JSXExpression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXExpressionContainer(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXExpressionContainer<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXExpression<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            JSXExpression::EmptyExpression(s) => {
                AstNodes::JSXEmptyExpression(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            it @ match_expression!(JSXExpression) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXEmptyExpression> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXEmptyExpression> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXAttributeItem<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            JSXAttributeItem::Attribute(s) => {
                AstNodes::JSXAttribute(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            JSXAttributeItem::SpreadAttribute(s) => {
                AstNodes::JSXSpreadAttribute(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXAttributeItem<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXAttribute<'a>> {
    #[inline]
    pub fn name(&self) -> &AstNode<'a, JSXAttributeName<'a>> {
        let following_node =
            self.inner.value.as_ref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXAttribute(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn value(&self) -> Option<&AstNode<'a, JSXAttributeValue<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::JSXAttribute(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXAttribute<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXSpreadAttribute<'a>> {
    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXSpreadAttribute(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXSpreadAttribute<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXAttributeName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            JSXAttributeName::Identifier(s) => {
                AstNodes::JSXIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            JSXAttributeName::NamespacedName(s) => {
                AstNodes::JSXNamespacedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXAttributeName<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXAttributeValue<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            JSXAttributeValue::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            JSXAttributeValue::ExpressionContainer(s) => {
                AstNodes::JSXExpressionContainer(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            JSXAttributeValue::Element(s) => AstNodes::JSXElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            JSXAttributeValue::Fragment(s) => {
                AstNodes::JSXFragment(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXAttributeValue<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXIdentifier<'a>> {
    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXIdentifier<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXChild<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            JSXChild::Text(s) => AstNodes::JSXText(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            JSXChild::Element(s) => AstNodes::JSXElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            JSXChild::Fragment(s) => AstNodes::JSXFragment(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            JSXChild::ExpressionContainer(s) => {
                AstNodes::JSXExpressionContainer(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            JSXChild::Spread(s) => AstNodes::JSXSpreadChild(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXChild<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXSpreadChild<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXSpreadChild(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXSpreadChild<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSXText<'a>> {
    #[inline]
    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSXText<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSThisParameter<'a>> {
    #[inline]
    pub fn this_span(&self) -> Span {
        self.inner.this_span
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSThisParameter(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSThisParameter<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSEnumDeclaration<'a>> {
    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.body));
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, TSEnumBody<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumDeclaration(transmute_self(self))),
            following_node,
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSEnumDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSEnumBody<'a>> {
    #[inline]
    pub fn members(&self) -> &AstNode<'a, Vec<'a, TSEnumMember<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.members,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumBody(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSEnumBody<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSEnumMember<'a>> {
    #[inline]
    pub fn id(&self) -> &AstNode<'a, TSEnumMemberName<'a>> {
        let following_node =
            self.inner.initializer.as_ref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumMember(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn initializer(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.initializer.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSEnumMember(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSEnumMember<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSEnumMemberName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSEnumMemberName::Identifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSEnumMemberName::String(s) => AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSEnumMemberName::ComputedString(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSEnumMemberName::ComputedTemplateString(s) => {
                AstNodes::TemplateLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, TSEnumMemberName<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTypeAnnotation<'a>> {
    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAnnotation(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTypeAnnotation<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSLiteralType<'a>> {
    #[inline]
    pub fn literal(&self) -> &AstNode<'a, TSLiteral<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.literal,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSLiteralType(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSLiteralType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSLiteral<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSLiteral::BooleanLiteral(s) => {
                AstNodes::BooleanLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSLiteral::NumericLiteral(s) => {
                AstNodes::NumericLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSLiteral::BigIntLiteral(s) => AstNodes::BigIntLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSLiteral::StringLiteral(s) => AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSLiteral::TemplateLiteral(s) => {
                AstNodes::TemplateLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSLiteral::UnaryExpression(s) => {
                AstNodes::UnaryExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, TSLiteral<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSType<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSType::TSAnyKeyword(s) => AstNodes::TSAnyKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSBigIntKeyword(s) => {
                AstNodes::TSBigIntKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSBooleanKeyword(s) => {
                AstNodes::TSBooleanKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSIntrinsicKeyword(s) => {
                AstNodes::TSIntrinsicKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSNeverKeyword(s) => AstNodes::TSNeverKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSNullKeyword(s) => AstNodes::TSNullKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSNumberKeyword(s) => {
                AstNodes::TSNumberKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSObjectKeyword(s) => {
                AstNodes::TSObjectKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSStringKeyword(s) => {
                AstNodes::TSStringKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSSymbolKeyword(s) => {
                AstNodes::TSSymbolKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSUndefinedKeyword(s) => {
                AstNodes::TSUndefinedKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSUnknownKeyword(s) => {
                AstNodes::TSUnknownKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSVoidKeyword(s) => AstNodes::TSVoidKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSArrayType(s) => AstNodes::TSArrayType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSConditionalType(s) => {
                AstNodes::TSConditionalType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSConstructorType(s) => {
                AstNodes::TSConstructorType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSFunctionType(s) => AstNodes::TSFunctionType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSImportType(s) => AstNodes::TSImportType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSIndexedAccessType(s) => {
                AstNodes::TSIndexedAccessType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSInferType(s) => AstNodes::TSInferType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSIntersectionType(s) => {
                AstNodes::TSIntersectionType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSLiteralType(s) => AstNodes::TSLiteralType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSMappedType(s) => AstNodes::TSMappedType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSNamedTupleMember(s) => {
                AstNodes::TSNamedTupleMember(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSTemplateLiteralType(s) => {
                AstNodes::TSTemplateLiteralType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSThisType(s) => AstNodes::TSThisType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSTupleType(s) => AstNodes::TSTupleType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSTypeLiteral(s) => AstNodes::TSTypeLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSTypeOperatorType(s) => {
                AstNodes::TSTypeOperator(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSTypePredicate(s) => {
                AstNodes::TSTypePredicate(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSTypeQuery(s) => AstNodes::TSTypeQuery(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSTypeReference(s) => {
                AstNodes::TSTypeReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::TSUnionType(s) => AstNodes::TSUnionType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            TSType::TSParenthesizedType(s) => {
                AstNodes::TSParenthesizedType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::JSDocNullableType(s) => {
                AstNodes::JSDocNullableType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::JSDocNonNullableType(s) => {
                AstNodes::JSDocNonNullableType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSType::JSDocUnknownType(s) => {
                AstNodes::JSDocUnknownType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, TSType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSConditionalType<'a>> {
    #[inline]
    pub fn check_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.extends_type));
        self.allocator.alloc(AstNode {
            inner: &self.inner.check_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn extends_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.true_type));
        self.allocator.alloc(AstNode {
            inner: &self.inner.extends_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn true_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.false_type));
        self.allocator.alloc(AstNode {
            inner: &self.inner.true_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn false_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.false_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSConditionalType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSUnionType<'a>> {
    #[inline]
    pub fn types(&self) -> &AstNode<'a, Vec<'a, TSType<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSUnionType(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSUnionType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSIntersectionType<'a>> {
    #[inline]
    pub fn types(&self) -> &AstNode<'a, Vec<'a, TSType<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIntersectionType(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSIntersectionType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSParenthesizedType<'a>> {
    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSParenthesizedType(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSParenthesizedType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTypeOperator<'a>> {
    #[inline]
    pub fn operator(&self) -> TSTypeOperatorOperator {
        self.inner.operator
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeOperator(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTypeOperator<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSArrayType<'a>> {
    #[inline]
    pub fn element_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.element_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSArrayType(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSArrayType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSIndexedAccessType<'a>> {
    #[inline]
    pub fn object_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.index_type));
        self.allocator.alloc(AstNode {
            inner: &self.inner.object_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIndexedAccessType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn index_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.index_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIndexedAccessType(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSIndexedAccessType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTupleType<'a>> {
    #[inline]
    pub fn element_types(&self) -> &AstNode<'a, Vec<'a, TSTupleElement<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.element_types,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTupleType(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTupleType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSNamedTupleMember<'a>> {
    #[inline]
    pub fn label(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.element_type));
        self.allocator.alloc(AstNode {
            inner: &self.inner.label,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSNamedTupleMember(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn element_type(&self) -> &AstNode<'a, TSTupleElement<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.element_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSNamedTupleMember(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSNamedTupleMember<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSOptionalType<'a>> {
    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSOptionalType(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSOptionalType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSRestType<'a>> {
    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSRestType(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSRestType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTupleElement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSTupleElement::TSOptionalType(s) => {
                AstNodes::TSOptionalType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSTupleElement::TSRestType(s) => AstNodes::TSRestType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
            it @ match_ts_type!(TSTupleElement) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_ts_type(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTupleElement<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSAnyKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSAnyKeyword> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSStringKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSStringKeyword> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSBooleanKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSBooleanKeyword> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSNumberKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSNumberKeyword> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSNeverKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSNeverKeyword> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSIntrinsicKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSIntrinsicKeyword> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSUnknownKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSUnknownKeyword> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSNullKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSNullKeyword> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSUndefinedKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSUndefinedKeyword> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSVoidKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSVoidKeyword> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSSymbolKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSSymbolKeyword> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSThisType> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSThisType> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSObjectKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSObjectKeyword> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSBigIntKeyword> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSBigIntKeyword> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTypeReference<'a>> {
    #[inline]
    pub fn type_name(&self) -> &AstNode<'a, TSTypeName<'a>> {
        let following_node =
            self.inner.type_arguments.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeReference(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeReference(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTypeReference<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTypeName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSTypeName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSTypeName::QualifiedName(s) => {
                AstNodes::TSQualifiedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSTypeName::ThisExpression(s) => {
                AstNodes::ThisExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTypeName<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSQualifiedName<'a>> {
    #[inline]
    pub fn left(&self) -> &AstNode<'a, TSTypeName<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.right));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSQualifiedName(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSQualifiedName(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSQualifiedName<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTypeParameterInstantiation<'a>> {
    #[inline]
    pub fn params(&self) -> &AstNode<'a, Vec<'a, TSType<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.params,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::TSTypeParameterInstantiation(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTypeParameterInstantiation<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTypeParameter<'a>> {
    #[inline]
    pub fn name(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = self
            .inner
            .constraint
            .as_ref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.default.as_ref().map(SiblingNode::from))
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeParameter(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn constraint(&self) -> Option<&AstNode<'a, TSType<'a>>> {
        let following_node =
            self.inner.default.as_ref().map(SiblingNode::from).or(self.following_node);
        self.allocator
            .alloc(self.inner.constraint.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeParameter(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn default(&self) -> Option<&AstNode<'a, TSType<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.default.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeParameter(transmute_self(self))),
                following_node,
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTypeParameter<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTypeParameterDeclaration<'a>> {
    #[inline]
    pub fn params(&self) -> &AstNode<'a, Vec<'a, TSTypeParameter<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.params,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::TSTypeParameterDeclaration(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTypeParameterDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTypeAliasDeclaration<'a>> {
    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = self
            .inner
            .type_parameters
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| Some(SiblingNode::from(&self.inner.type_annotation)));
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAliasDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = Some(SiblingNode::from(&self.inner.type_annotation));
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::TSTypeAliasDeclaration(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAliasDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTypeAliasDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSClassImplements<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, TSTypeName<'a>> {
        let following_node =
            self.inner.type_arguments.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSClassImplements(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSClassImplements(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSClassImplements<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSInterfaceDeclaration<'a>> {
    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = self
            .inner
            .type_parameters
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.extends.first().as_ref().copied().map(SiblingNode::from))
            .or_else(|| Some(SiblingNode::from(self.inner.body.as_ref())));
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = self
            .inner
            .extends
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or_else(|| Some(SiblingNode::from(self.inner.body.as_ref())));
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::TSInterfaceDeclaration(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn extends(&self) -> &AstNode<'a, Vec<'a, TSInterfaceHeritage<'a>>> {
        let following_node = Some(SiblingNode::from(self.inner.body.as_ref()));
        self.allocator.alloc(AstNode {
            inner: &self.inner.extends,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, TSInterfaceBody<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn declare(&self) -> bool {
        self.inner.declare
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSInterfaceDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSInterfaceBody<'a>> {
    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, TSSignature<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceBody(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSInterfaceBody<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSPropertySignature<'a>> {
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
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node =
            self.inner.type_annotation.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSPropertySignature(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSPropertySignature(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSPropertySignature<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSSignature<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSSignature::TSIndexSignature(s) => {
                AstNodes::TSIndexSignature(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSSignature::TSPropertySignature(s) => {
                AstNodes::TSPropertySignature(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSSignature::TSCallSignatureDeclaration(s) => {
                AstNodes::TSCallSignatureDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSSignature::TSConstructSignatureDeclaration(s) => {
                AstNodes::TSConstructSignatureDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSSignature::TSMethodSignature(s) => {
                AstNodes::TSMethodSignature(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, TSSignature<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSIndexSignature<'a>> {
    #[inline]
    pub fn parameters(&self) -> &AstNode<'a, Vec<'a, TSIndexSignatureName<'a>>> {
        let following_node = Some(SiblingNode::from(self.inner.type_annotation.as_ref()));
        self.allocator.alloc(AstNode {
            inner: &self.inner.parameters,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIndexSignature(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSTypeAnnotation<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: self.inner.type_annotation.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIndexSignature(transmute_self(self))),
            following_node,
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSIndexSignature<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSCallSignatureDeclaration<'a>> {
    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = self
            .inner
            .this_param
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| Some(SiblingNode::from(self.inner.params.as_ref())));
        self.allocator
            .alloc(
                self.inner.type_parameters.as_ref().map(|inner| AstNode {
                    inner: inner.as_ref(),
                    allocator: self.allocator,
                    parent: self
                        .allocator
                        .alloc(AstNodes::TSCallSignatureDeclaration(transmute_self(self))),
                    following_node,
                }),
            )
            .as_ref()
    }

    #[inline]
    pub fn this_param(&self) -> Option<&AstNode<'a, TSThisParameter<'a>>> {
        let following_node = Some(SiblingNode::from(self.inner.params.as_ref()));
        self.allocator
            .alloc(
                self.inner.this_param.as_ref().map(|inner| AstNode {
                    inner: inner.as_ref(),
                    allocator: self.allocator,
                    parent: self
                        .allocator
                        .alloc(AstNodes::TSCallSignatureDeclaration(transmute_self(self))),
                    following_node,
                }),
            )
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_node =
            self.inner.return_type.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::TSCallSignatureDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn return_type(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(
                self.inner.return_type.as_ref().map(|inner| AstNode {
                    inner: inner.as_ref(),
                    allocator: self.allocator,
                    parent: self
                        .allocator
                        .alloc(AstNodes::TSCallSignatureDeclaration(transmute_self(self))),
                    following_node,
                }),
            )
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSCallSignatureDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSMethodSignature<'a>> {
    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = self
            .inner
            .type_parameters
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.this_param.as_deref().map(SiblingNode::from))
            .or_else(|| Some(SiblingNode::from(self.inner.params.as_ref())));
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSMethodSignature(transmute_self(self))),
            following_node,
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
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = self
            .inner
            .this_param
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| Some(SiblingNode::from(self.inner.params.as_ref())));
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMethodSignature(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn this_param(&self) -> Option<&AstNode<'a, TSThisParameter<'a>>> {
        let following_node = Some(SiblingNode::from(self.inner.params.as_ref()));
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMethodSignature(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_node =
            self.inner.return_type.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSMethodSignature(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn return_type(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMethodSignature(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSMethodSignature<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSConstructSignatureDeclaration<'a>> {
    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = Some(SiblingNode::from(self.inner.params.as_ref()));
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| {
                AstNode {
                    inner: inner.as_ref(),
                    allocator: self.allocator,
                    parent: self
                        .allocator
                        .alloc(AstNodes::TSConstructSignatureDeclaration(transmute_self(self))),
                    following_node,
                }
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_node =
            self.inner.return_type.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::TSConstructSignatureDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn return_type(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| {
                AstNode {
                    inner: inner.as_ref(),
                    allocator: self.allocator,
                    parent: self
                        .allocator
                        .alloc(AstNodes::TSConstructSignatureDeclaration(transmute_self(self))),
                    following_node,
                }
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSConstructSignatureDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSIndexSignatureName<'a>> {
    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSTypeAnnotation<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: self.inner.type_annotation.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIndexSignatureName(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSIndexSignatureName<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSInterfaceHeritage<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node =
            self.inner.type_arguments.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceHeritage(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSInterfaceHeritage(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSInterfaceHeritage<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTypePredicate<'a>> {
    #[inline]
    pub fn parameter_name(&self) -> &AstNode<'a, TSTypePredicateName<'a>> {
        let following_node =
            self.inner.type_annotation.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.parameter_name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypePredicate(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn asserts(&self) -> bool {
        self.inner.asserts
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypePredicate(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTypePredicate<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTypePredicateName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSTypePredicateName::Identifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSTypePredicateName::This(s) => AstNodes::TSThisType(self.allocator.alloc(AstNode {
                inner: s,
                parent,
                allocator: self.allocator,
                following_node: self.following_node,
            })),
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTypePredicateName<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSModuleDeclaration<'a>> {
    #[inline]
    pub fn id(&self) -> &AstNode<'a, TSModuleDeclarationName<'a>> {
        let following_node = self.inner.body.as_ref().map(SiblingNode::from);
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSModuleDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> Option<&AstNode<'a, TSModuleDeclarationBody<'a>>> {
        let following_node = None;
        self.allocator
            .alloc(self.inner.body.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSModuleDeclaration(transmute_self(self))),
                following_node,
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSModuleDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSModuleDeclarationName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSModuleDeclarationName::Identifier(s) => {
                AstNodes::BindingIdentifier(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSModuleDeclarationName::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, TSModuleDeclarationName<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSModuleDeclarationBody<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSModuleDeclarationBody::TSModuleDeclaration(s) => {
                AstNodes::TSModuleDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSModuleDeclarationBody::TSModuleBlock(s) => {
                AstNodes::TSModuleBlock(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, TSModuleDeclarationBody<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSModuleBlock<'a>> {
    #[inline]
    pub fn directives(&self) -> &AstNode<'a, Vec<'a, Directive<'a>>> {
        let following_node = self
            .inner
            .body
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSModuleBlock(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSModuleBlock(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSModuleBlock<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTypeLiteral<'a>> {
    #[inline]
    pub fn members(&self) -> &AstNode<'a, Vec<'a, TSSignature<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.members,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeLiteral(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTypeLiteral<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSInferType<'a>> {
    #[inline]
    pub fn type_parameter(&self) -> &AstNode<'a, TSTypeParameter<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: self.inner.type_parameter.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInferType(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSInferType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTypeQuery<'a>> {
    #[inline]
    pub fn expr_name(&self) -> &AstNode<'a, TSTypeQueryExprName<'a>> {
        let following_node =
            self.inner.type_arguments.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.expr_name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeQuery(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeQuery(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTypeQuery<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTypeQueryExprName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSTypeQueryExprName::TSImportType(s) => {
                AstNodes::TSImportType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            it @ match_ts_type_name!(TSTypeQueryExprName) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_ts_type_name(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTypeQueryExprName<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSImportType<'a>> {
    #[inline]
    pub fn argument(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self
            .inner
            .options
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.qualifier.as_ref().map(SiblingNode::from))
            .or_else(|| self.inner.type_arguments.as_deref().map(SiblingNode::from))
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSImportType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn options(&self) -> Option<&AstNode<'a, ObjectExpression<'a>>> {
        let following_node = self
            .inner
            .qualifier
            .as_ref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.type_arguments.as_deref().map(SiblingNode::from))
            .or(self.following_node);
        self.allocator
            .alloc(self.inner.options.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSImportType(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn qualifier(&self) -> Option<&AstNode<'a, TSImportTypeQualifier<'a>>> {
        let following_node =
            self.inner.type_arguments.as_deref().map(SiblingNode::from).or(self.following_node);
        self.allocator
            .alloc(self.inner.qualifier.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSImportType(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSImportType(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSImportType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSImportTypeQualifier<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSImportTypeQualifier::Identifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            TSImportTypeQualifier::QualifiedName(s) => {
                AstNodes::TSImportTypeQualifiedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, TSImportTypeQualifier<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSImportTypeQualifiedName<'a>> {
    #[inline]
    pub fn left(&self) -> &AstNode<'a, TSImportTypeQualifier<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.right));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSImportTypeQualifiedName(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSImportTypeQualifiedName(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSImportTypeQualifiedName<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSFunctionType<'a>> {
    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = self
            .inner
            .this_param
            .as_deref()
            .map(SiblingNode::from)
            .or_else(|| Some(SiblingNode::from(self.inner.params.as_ref())));
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSFunctionType(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn this_param(&self) -> Option<&AstNode<'a, TSThisParameter<'a>>> {
        let following_node = Some(SiblingNode::from(self.inner.params.as_ref()));
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSFunctionType(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_node = Some(SiblingNode::from(self.inner.return_type.as_ref()));
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSFunctionType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn return_type(&self) -> &AstNode<'a, TSTypeAnnotation<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: self.inner.return_type.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSFunctionType(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSFunctionType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSConstructorType<'a>> {
    #[inline]
    pub fn r#abstract(&self) -> bool {
        self.inner.r#abstract
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = Some(SiblingNode::from(self.inner.params.as_ref()));
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSConstructorType(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_node = Some(SiblingNode::from(self.inner.return_type.as_ref()));
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConstructorType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn return_type(&self) -> &AstNode<'a, TSTypeAnnotation<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: self.inner.return_type.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConstructorType(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSConstructorType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSMappedType<'a>> {
    #[inline]
    pub fn type_parameter(&self) -> &AstNode<'a, TSTypeParameter<'a>> {
        let following_node = self
            .inner
            .name_type
            .as_ref()
            .map(SiblingNode::from)
            .or_else(|| self.inner.type_annotation.as_ref().map(SiblingNode::from))
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: self.inner.type_parameter.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSMappedType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn name_type(&self) -> Option<&AstNode<'a, TSType<'a>>> {
        let following_node =
            self.inner.type_annotation.as_ref().map(SiblingNode::from).or(self.following_node);
        self.allocator
            .alloc(self.inner.name_type.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMappedType(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSType<'a>>> {
        let following_node = self.following_node;
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMappedType(transmute_self(self))),
                following_node,
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

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSMappedType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTemplateLiteralType<'a>> {
    #[inline]
    pub fn quasis(&self) -> &AstNode<'a, Vec<'a, TemplateElement<'a>>> {
        let following_node = self
            .inner
            .types
            .first()
            .as_ref()
            .copied()
            .map(SiblingNode::from)
            .or(self.following_node);
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasis,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTemplateLiteralType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn types(&self) -> &AstNode<'a, Vec<'a, TSType<'a>>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTemplateLiteralType(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTemplateLiteralType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSAsExpression<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.type_annotation));
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSAsExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSAsExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSAsExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSSatisfiesExpression<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.type_annotation));
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSSatisfiesExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSSatisfiesExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSSatisfiesExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSTypeAssertion<'a>> {
    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.expression));
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAssertion(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAssertion(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSTypeAssertion<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSImportEqualsDeclaration<'a>> {
    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = Some(SiblingNode::from(&self.inner.module_reference));
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSImportEqualsDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn module_reference(&self) -> &AstNode<'a, TSModuleReference<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.module_reference,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSImportEqualsDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn import_kind(&self) -> ImportOrExportKind {
        self.inner.import_kind
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSImportEqualsDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSModuleReference<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSModuleReference::ExternalModuleReference(s) => {
                AstNodes::TSExternalModuleReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node,
                }))
            }
            it @ match_ts_type_name!(TSModuleReference) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_ts_type_name(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node,
                    })
                    .as_ast_nodes();
            }
        };
        self.allocator.alloc(node)
    }
}

impl<'a> GetSpan for AstNode<'a, TSModuleReference<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSExternalModuleReference<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, StringLiteral<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSExternalModuleReference(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSExternalModuleReference<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSNonNullExpression<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSNonNullExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSNonNullExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, Decorator<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Decorator(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, Decorator<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSExportAssignment<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSExportAssignment(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSExportAssignment<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSNamespaceExportDeclaration<'a>> {
    #[inline]
    pub fn id(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = None;
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::TSNamespaceExportDeclaration(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSNamespaceExportDeclaration<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, TSInstantiationExpression<'a>> {
    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(SiblingNode::from(self.inner.type_arguments.as_ref()));
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInstantiationExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> &AstNode<'a, TSTypeParameterInstantiation<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: self.inner.type_arguments.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInstantiationExpression(transmute_self(self))),
            following_node,
        })
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, TSInstantiationExpression<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSDocNullableType<'a>> {
    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSDocNullableType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn postfix(&self) -> bool {
        self.inner.postfix
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSDocNullableType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSDocNonNullableType<'a>> {
    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node;
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSDocNonNullableType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn postfix(&self) -> bool {
        self.inner.postfix
    }

    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSDocNonNullableType<'a>> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

impl<'a> AstNode<'a, JSDocUnknownType> {
    pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span()).fmt(f)
    }

    pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_trailing_comments(
            &self.parent.as_sibling_node(),
            &SiblingNode::from(self.inner),
            self.following_node.as_ref(),
        )
        .fmt(f)
    }
}

impl<'a> GetSpan for AstNode<'a, JSDocUnknownType> {
    #[inline]
    fn span(&self) -> oxc_span::Span {
        self.inner.span()
    }
}

pub struct AstNodeIterator<'a, T> {
    inner: std::iter::Peekable<std::slice::Iter<'a, T>>,
    parent: &'a AstNodes<'a>,
    allocator: &'a Allocator,
}

impl<'a> AstNode<'a, Vec<'a, Expression<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Expression<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, Expression<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, Expression<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, Expression<'a>> {
    type Item = &'a AstNode<'a, Expression<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, Expression<'a>>> {
    type Item = &'a AstNode<'a, Expression<'a>>;
    type IntoIter = AstNodeIterator<'a, Expression<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<Expression<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, ArrayExpressionElement<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, ArrayExpressionElement<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, ArrayExpressionElement<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, ArrayExpressionElement<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, ArrayExpressionElement<'a>> {
    type Item = &'a AstNode<'a, ArrayExpressionElement<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, ArrayExpressionElement<'a>>> {
    type Item = &'a AstNode<'a, ArrayExpressionElement<'a>>;
    type IntoIter = AstNodeIterator<'a, ArrayExpressionElement<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<ArrayExpressionElement<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, ObjectPropertyKind<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, ObjectPropertyKind<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, ObjectPropertyKind<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, ObjectPropertyKind<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, ObjectPropertyKind<'a>> {
    type Item = &'a AstNode<'a, ObjectPropertyKind<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, ObjectPropertyKind<'a>>> {
    type Item = &'a AstNode<'a, ObjectPropertyKind<'a>>;
    type IntoIter = AstNodeIterator<'a, ObjectPropertyKind<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<ObjectPropertyKind<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, TemplateElement<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TemplateElement<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, TemplateElement<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, TemplateElement<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, TemplateElement<'a>> {
    type Item = &'a AstNode<'a, TemplateElement<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, TemplateElement<'a>>> {
    type Item = &'a AstNode<'a, TemplateElement<'a>>;
    type IntoIter = AstNodeIterator<'a, TemplateElement<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<TemplateElement<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, Argument<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Argument<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, Argument<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, Argument<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, Argument<'a>> {
    type Item = &'a AstNode<'a, Argument<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, Argument<'a>>> {
    type Item = &'a AstNode<'a, Argument<'a>>;
    type IntoIter = AstNodeIterator<'a, Argument<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<Argument<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, AssignmentTargetProperty<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, AssignmentTargetProperty<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, AssignmentTargetProperty<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, AssignmentTargetProperty<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, AssignmentTargetProperty<'a>> {
    type Item = &'a AstNode<'a, AssignmentTargetProperty<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, AssignmentTargetProperty<'a>>> {
    type Item = &'a AstNode<'a, AssignmentTargetProperty<'a>>;
    type IntoIter = AstNodeIterator<'a, AssignmentTargetProperty<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<AssignmentTargetProperty<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, Statement<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Statement<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, Statement<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, Statement<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, Statement<'a>> {
    type Item = &'a AstNode<'a, Statement<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, Statement<'a>>> {
    type Item = &'a AstNode<'a, Statement<'a>>;
    type IntoIter = AstNodeIterator<'a, Statement<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<Statement<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, Directive<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Directive<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, Directive<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, Directive<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, Directive<'a>> {
    type Item = &'a AstNode<'a, Directive<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, Directive<'a>>> {
    type Item = &'a AstNode<'a, Directive<'a>>;
    type IntoIter = AstNodeIterator<'a, Directive<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<Directive<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, VariableDeclarator<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, VariableDeclarator<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, VariableDeclarator<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, VariableDeclarator<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, VariableDeclarator<'a>> {
    type Item = &'a AstNode<'a, VariableDeclarator<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, VariableDeclarator<'a>>> {
    type Item = &'a AstNode<'a, VariableDeclarator<'a>>;
    type IntoIter = AstNodeIterator<'a, VariableDeclarator<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<VariableDeclarator<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, SwitchCase<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, SwitchCase<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, SwitchCase<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, SwitchCase<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, SwitchCase<'a>> {
    type Item = &'a AstNode<'a, SwitchCase<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, SwitchCase<'a>>> {
    type Item = &'a AstNode<'a, SwitchCase<'a>>;
    type IntoIter = AstNodeIterator<'a, SwitchCase<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<SwitchCase<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, BindingProperty<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, BindingProperty<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, BindingProperty<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, BindingProperty<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, BindingProperty<'a>> {
    type Item = &'a AstNode<'a, BindingProperty<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, BindingProperty<'a>>> {
    type Item = &'a AstNode<'a, BindingProperty<'a>>;
    type IntoIter = AstNodeIterator<'a, BindingProperty<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<BindingProperty<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, FormalParameter<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, FormalParameter<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, FormalParameter<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, FormalParameter<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, FormalParameter<'a>> {
    type Item = &'a AstNode<'a, FormalParameter<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, FormalParameter<'a>>> {
    type Item = &'a AstNode<'a, FormalParameter<'a>>;
    type IntoIter = AstNodeIterator<'a, FormalParameter<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<FormalParameter<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, ClassElement<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, ClassElement<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, ClassElement<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, ClassElement<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, ClassElement<'a>> {
    type Item = &'a AstNode<'a, ClassElement<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, ClassElement<'a>>> {
    type Item = &'a AstNode<'a, ClassElement<'a>>;
    type IntoIter = AstNodeIterator<'a, ClassElement<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<ClassElement<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, ImportDeclarationSpecifier<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, ImportDeclarationSpecifier<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, ImportDeclarationSpecifier<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, ImportDeclarationSpecifier<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, ImportDeclarationSpecifier<'a>> {
    type Item = &'a AstNode<'a, ImportDeclarationSpecifier<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, ImportDeclarationSpecifier<'a>>> {
    type Item = &'a AstNode<'a, ImportDeclarationSpecifier<'a>>;
    type IntoIter = AstNodeIterator<'a, ImportDeclarationSpecifier<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<ImportDeclarationSpecifier<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, ImportAttribute<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, ImportAttribute<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, ImportAttribute<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, ImportAttribute<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, ImportAttribute<'a>> {
    type Item = &'a AstNode<'a, ImportAttribute<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, ImportAttribute<'a>>> {
    type Item = &'a AstNode<'a, ImportAttribute<'a>>;
    type IntoIter = AstNodeIterator<'a, ImportAttribute<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<ImportAttribute<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, ExportSpecifier<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, ExportSpecifier<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, ExportSpecifier<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, ExportSpecifier<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, ExportSpecifier<'a>> {
    type Item = &'a AstNode<'a, ExportSpecifier<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, ExportSpecifier<'a>>> {
    type Item = &'a AstNode<'a, ExportSpecifier<'a>>;
    type IntoIter = AstNodeIterator<'a, ExportSpecifier<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<ExportSpecifier<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, JSXAttributeItem<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, JSXAttributeItem<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, JSXAttributeItem<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, JSXAttributeItem<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, JSXAttributeItem<'a>> {
    type Item = &'a AstNode<'a, JSXAttributeItem<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, JSXAttributeItem<'a>>> {
    type Item = &'a AstNode<'a, JSXAttributeItem<'a>>;
    type IntoIter = AstNodeIterator<'a, JSXAttributeItem<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<JSXAttributeItem<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, JSXChild<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, JSXChild<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, JSXChild<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, JSXChild<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, JSXChild<'a>> {
    type Item = &'a AstNode<'a, JSXChild<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, JSXChild<'a>>> {
    type Item = &'a AstNode<'a, JSXChild<'a>>;
    type IntoIter = AstNodeIterator<'a, JSXChild<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<JSXChild<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, TSEnumMember<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSEnumMember<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, TSEnumMember<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, TSEnumMember<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, TSEnumMember<'a>> {
    type Item = &'a AstNode<'a, TSEnumMember<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, TSEnumMember<'a>>> {
    type Item = &'a AstNode<'a, TSEnumMember<'a>>;
    type IntoIter = AstNodeIterator<'a, TSEnumMember<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<TSEnumMember<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, TSType<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSType<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, TSType<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, TSType<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, TSType<'a>> {
    type Item = &'a AstNode<'a, TSType<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, TSType<'a>>> {
    type Item = &'a AstNode<'a, TSType<'a>>;
    type IntoIter = AstNodeIterator<'a, TSType<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<TSType<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, TSTupleElement<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSTupleElement<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, TSTupleElement<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, TSTupleElement<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, TSTupleElement<'a>> {
    type Item = &'a AstNode<'a, TSTupleElement<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, TSTupleElement<'a>>> {
    type Item = &'a AstNode<'a, TSTupleElement<'a>>;
    type IntoIter = AstNodeIterator<'a, TSTupleElement<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<TSTupleElement<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, TSTypeParameter<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSTypeParameter<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, TSTypeParameter<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, TSTypeParameter<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, TSTypeParameter<'a>> {
    type Item = &'a AstNode<'a, TSTypeParameter<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, TSTypeParameter<'a>>> {
    type Item = &'a AstNode<'a, TSTypeParameter<'a>>;
    type IntoIter = AstNodeIterator<'a, TSTypeParameter<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<TSTypeParameter<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, TSClassImplements<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSClassImplements<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, TSClassImplements<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, TSClassImplements<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, TSClassImplements<'a>> {
    type Item = &'a AstNode<'a, TSClassImplements<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, TSClassImplements<'a>>> {
    type Item = &'a AstNode<'a, TSClassImplements<'a>>;
    type IntoIter = AstNodeIterator<'a, TSClassImplements<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<TSClassImplements<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, TSSignature<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSSignature<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, TSSignature<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, TSSignature<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, TSSignature<'a>> {
    type Item = &'a AstNode<'a, TSSignature<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, TSSignature<'a>>> {
    type Item = &'a AstNode<'a, TSSignature<'a>>;
    type IntoIter = AstNodeIterator<'a, TSSignature<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<TSSignature<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, TSIndexSignatureName<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSIndexSignatureName<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, TSIndexSignatureName<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, TSIndexSignatureName<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, TSIndexSignatureName<'a>> {
    type Item = &'a AstNode<'a, TSIndexSignatureName<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, TSIndexSignatureName<'a>>> {
    type Item = &'a AstNode<'a, TSIndexSignatureName<'a>>;
    type IntoIter = AstNodeIterator<'a, TSIndexSignatureName<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<TSIndexSignatureName<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, TSInterfaceHeritage<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSInterfaceHeritage<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, TSInterfaceHeritage<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, TSInterfaceHeritage<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, TSInterfaceHeritage<'a>> {
    type Item = &'a AstNode<'a, TSInterfaceHeritage<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, TSInterfaceHeritage<'a>>> {
    type Item = &'a AstNode<'a, TSInterfaceHeritage<'a>>;
    type IntoIter = AstNodeIterator<'a, TSInterfaceHeritage<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<TSInterfaceHeritage<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, Decorator<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Decorator<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, Decorator<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: inner_iter.next().map(SiblingNode::from),
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, Decorator<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, Decorator<'a>> {
    type Item = &'a AstNode<'a, Decorator<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self.inner.peek().copied().map(SiblingNode::from);
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, Decorator<'a>>> {
    type Item = &'a AstNode<'a, Decorator<'a>>;
    type IntoIter = AstNodeIterator<'a, Decorator<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<Decorator<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Option<AssignmentTargetMaybeDefault<'a>>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, Option<AssignmentTargetMaybeDefault<'a>>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|next| next.as_ref().map(SiblingNode::from))
                        .unwrap_or_default(),
                }
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, Option<AssignmentTargetMaybeDefault<'a>>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, Option<AssignmentTargetMaybeDefault<'a>>> {
    type Item = &'a AstNode<'a, Option<AssignmentTargetMaybeDefault<'a>>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self
                    .inner
                    .peek()
                    .copied()
                    .map(|next| next.as_ref().map(SiblingNode::from))
                    .unwrap_or_default();
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>> {
    type Item = &'a AstNode<'a, Option<AssignmentTargetMaybeDefault<'a>>>;
    type IntoIter = AstNodeIterator<'a, Option<AssignmentTargetMaybeDefault<'a>>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<Option<AssignmentTargetMaybeDefault<'a>>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

impl<'a> AstNode<'a, Vec<'a, Option<BindingPattern<'a>>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Option<BindingPattern<'a>>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }

    pub fn first(&self) -> Option<&'a AstNode<'a, Option<BindingPattern<'a>>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|next| next.as_ref().map(SiblingNode::from))
                        .unwrap_or_default(),
                }
            }))
            .as_ref()
    }

    pub fn last(&self) -> Option<&'a AstNode<'a, Option<BindingPattern<'a>>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: None,
            }))
            .as_ref()
    }
}

impl<'a> Iterator for AstNodeIterator<'a, Option<BindingPattern<'a>>> {
    type Item = &'a AstNode<'a, Option<BindingPattern<'a>>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                let following_node = self
                    .inner
                    .peek()
                    .copied()
                    .map(|next| next.as_ref().map(SiblingNode::from))
                    .unwrap_or_default();
                AstNode { parent: self.parent, inner, allocator, following_node }
            }))
            .as_ref()
    }
}

impl<'a> IntoIterator for &AstNode<'a, Vec<'a, Option<BindingPattern<'a>>>> {
    type Item = &'a AstNode<'a, Option<BindingPattern<'a>>>;
    type IntoIter = AstNodeIterator<'a, Option<BindingPattern<'a>>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<Option<BindingPattern<'a>>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}
