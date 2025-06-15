// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter/ast_nodes.rs`.

#![expect(clippy::elidable_lifetime_names)]
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
    ArrayExpressionElement(&'a AstNode<'a, ArrayExpressionElement<'a>>),
    Elision(&'a AstNode<'a, Elision>),
    ObjectExpression(&'a AstNode<'a, ObjectExpression<'a>>),
    ObjectProperty(&'a AstNode<'a, ObjectProperty<'a>>),
    PropertyKey(&'a AstNode<'a, PropertyKey<'a>>),
    TemplateLiteral(&'a AstNode<'a, TemplateLiteral<'a>>),
    TaggedTemplateExpression(&'a AstNode<'a, TaggedTemplateExpression<'a>>),
    MemberExpression(&'a AstNode<'a, MemberExpression<'a>>),
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
    AssignmentTarget(&'a AstNode<'a, AssignmentTarget<'a>>),
    SimpleAssignmentTarget(&'a AstNode<'a, SimpleAssignmentTarget<'a>>),
    AssignmentTargetPattern(&'a AstNode<'a, AssignmentTargetPattern<'a>>),
    ArrayAssignmentTarget(&'a AstNode<'a, ArrayAssignmentTarget<'a>>),
    ObjectAssignmentTarget(&'a AstNode<'a, ObjectAssignmentTarget<'a>>),
    AssignmentTargetWithDefault(&'a AstNode<'a, AssignmentTargetWithDefault<'a>>),
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
    ForStatementInit(&'a AstNode<'a, ForStatementInit<'a>>),
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
    ModuleDeclaration(&'a AstNode<'a, ModuleDeclaration<'a>>),
    ImportExpression(&'a AstNode<'a, ImportExpression<'a>>),
    ImportDeclaration(&'a AstNode<'a, ImportDeclaration<'a>>),
    ImportSpecifier(&'a AstNode<'a, ImportSpecifier<'a>>),
    ImportDefaultSpecifier(&'a AstNode<'a, ImportDefaultSpecifier<'a>>),
    ImportNamespaceSpecifier(&'a AstNode<'a, ImportNamespaceSpecifier<'a>>),
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
    TSIndexedAccessType(&'a AstNode<'a, TSIndexedAccessType<'a>>),
    TSNamedTupleMember(&'a AstNode<'a, TSNamedTupleMember<'a>>),
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
    TSTypeName(&'a AstNode<'a, TSTypeName<'a>>),
    TSQualifiedName(&'a AstNode<'a, TSQualifiedName<'a>>),
    TSTypeParameterInstantiation(&'a AstNode<'a, TSTypeParameterInstantiation<'a>>),
    TSTypeParameter(&'a AstNode<'a, TSTypeParameter<'a>>),
    TSTypeParameterDeclaration(&'a AstNode<'a, TSTypeParameterDeclaration<'a>>),
    TSTypeAliasDeclaration(&'a AstNode<'a, TSTypeAliasDeclaration<'a>>),
    TSClassImplements(&'a AstNode<'a, TSClassImplements<'a>>),
    TSInterfaceDeclaration(&'a AstNode<'a, TSInterfaceDeclaration<'a>>),
    TSPropertySignature(&'a AstNode<'a, TSPropertySignature<'a>>),
    TSMethodSignature(&'a AstNode<'a, TSMethodSignature<'a>>),
    TSConstructSignatureDeclaration(&'a AstNode<'a, TSConstructSignatureDeclaration<'a>>),
    TSInterfaceHeritage(&'a AstNode<'a, TSInterfaceHeritage<'a>>),
    TSModuleDeclaration(&'a AstNode<'a, TSModuleDeclaration<'a>>),
    TSModuleBlock(&'a AstNode<'a, TSModuleBlock<'a>>),
    TSTypeLiteral(&'a AstNode<'a, TSTypeLiteral<'a>>),
    TSInferType(&'a AstNode<'a, TSInferType<'a>>),
    TSTypeQuery(&'a AstNode<'a, TSTypeQuery<'a>>),
    TSImportType(&'a AstNode<'a, TSImportType<'a>>),
    TSMappedType(&'a AstNode<'a, TSMappedType<'a>>),
    TSTemplateLiteralType(&'a AstNode<'a, TSTemplateLiteralType<'a>>),
    TSAsExpression(&'a AstNode<'a, TSAsExpression<'a>>),
    TSSatisfiesExpression(&'a AstNode<'a, TSSatisfiesExpression<'a>>),
    TSTypeAssertion(&'a AstNode<'a, TSTypeAssertion<'a>>),
    TSImportEqualsDeclaration(&'a AstNode<'a, TSImportEqualsDeclaration<'a>>),
    TSModuleReference(&'a AstNode<'a, TSModuleReference<'a>>),
    TSExternalModuleReference(&'a AstNode<'a, TSExternalModuleReference<'a>>),
    TSNonNullExpression(&'a AstNode<'a, TSNonNullExpression<'a>>),
    Decorator(&'a AstNode<'a, Decorator<'a>>),
    TSExportAssignment(&'a AstNode<'a, TSExportAssignment<'a>>),
    TSInstantiationExpression(&'a AstNode<'a, TSInstantiationExpression<'a>>),
    JSDocNullableType(&'a AstNode<'a, JSDocNullableType<'a>>),
    JSDocNonNullableType(&'a AstNode<'a, JSDocNonNullableType<'a>>),
    JSDocUnknownType(&'a AstNode<'a, JSDocUnknownType>),
}
#[derive(Clone)]
pub enum FollowingNode<'a> {
    Program(&'a Program<'a>),
    Expression(&'a Expression<'a>),
    IdentifierName(&'a IdentifierName<'a>),
    IdentifierReference(&'a IdentifierReference<'a>),
    BindingIdentifier(&'a BindingIdentifier<'a>),
    LabelIdentifier(&'a LabelIdentifier<'a>),
    ThisExpression(&'a ThisExpression),
    ArrayExpression(&'a ArrayExpression<'a>),
    ArrayExpressionElement(&'a ArrayExpressionElement<'a>),
    Elision(&'a Elision),
    ObjectExpression(&'a ObjectExpression<'a>),
    ObjectPropertyKind(&'a ObjectPropertyKind<'a>),
    ObjectProperty(&'a ObjectProperty<'a>),
    PropertyKey(&'a PropertyKey<'a>),
    TemplateLiteral(&'a TemplateLiteral<'a>),
    TaggedTemplateExpression(&'a TaggedTemplateExpression<'a>),
    TemplateElement(&'a TemplateElement<'a>),
    MemberExpression(&'a MemberExpression<'a>),
    ComputedMemberExpression(&'a ComputedMemberExpression<'a>),
    StaticMemberExpression(&'a StaticMemberExpression<'a>),
    PrivateFieldExpression(&'a PrivateFieldExpression<'a>),
    CallExpression(&'a CallExpression<'a>),
    NewExpression(&'a NewExpression<'a>),
    MetaProperty(&'a MetaProperty<'a>),
    SpreadElement(&'a SpreadElement<'a>),
    Argument(&'a Argument<'a>),
    UpdateExpression(&'a UpdateExpression<'a>),
    UnaryExpression(&'a UnaryExpression<'a>),
    BinaryExpression(&'a BinaryExpression<'a>),
    PrivateInExpression(&'a PrivateInExpression<'a>),
    LogicalExpression(&'a LogicalExpression<'a>),
    ConditionalExpression(&'a ConditionalExpression<'a>),
    AssignmentExpression(&'a AssignmentExpression<'a>),
    AssignmentTarget(&'a AssignmentTarget<'a>),
    SimpleAssignmentTarget(&'a SimpleAssignmentTarget<'a>),
    AssignmentTargetPattern(&'a AssignmentTargetPattern<'a>),
    ArrayAssignmentTarget(&'a ArrayAssignmentTarget<'a>),
    ObjectAssignmentTarget(&'a ObjectAssignmentTarget<'a>),
    AssignmentTargetRest(&'a AssignmentTargetRest<'a>),
    AssignmentTargetMaybeDefault(&'a AssignmentTargetMaybeDefault<'a>),
    AssignmentTargetWithDefault(&'a AssignmentTargetWithDefault<'a>),
    AssignmentTargetProperty(&'a AssignmentTargetProperty<'a>),
    AssignmentTargetPropertyIdentifier(&'a AssignmentTargetPropertyIdentifier<'a>),
    AssignmentTargetPropertyProperty(&'a AssignmentTargetPropertyProperty<'a>),
    SequenceExpression(&'a SequenceExpression<'a>),
    Super(&'a Super),
    AwaitExpression(&'a AwaitExpression<'a>),
    ChainExpression(&'a ChainExpression<'a>),
    ChainElement(&'a ChainElement<'a>),
    ParenthesizedExpression(&'a ParenthesizedExpression<'a>),
    Statement(&'a Statement<'a>),
    Directive(&'a Directive<'a>),
    Hashbang(&'a Hashbang<'a>),
    BlockStatement(&'a BlockStatement<'a>),
    Declaration(&'a Declaration<'a>),
    VariableDeclaration(&'a VariableDeclaration<'a>),
    VariableDeclarator(&'a VariableDeclarator<'a>),
    EmptyStatement(&'a EmptyStatement),
    ExpressionStatement(&'a ExpressionStatement<'a>),
    IfStatement(&'a IfStatement<'a>),
    DoWhileStatement(&'a DoWhileStatement<'a>),
    WhileStatement(&'a WhileStatement<'a>),
    ForStatement(&'a ForStatement<'a>),
    ForStatementInit(&'a ForStatementInit<'a>),
    ForInStatement(&'a ForInStatement<'a>),
    ForStatementLeft(&'a ForStatementLeft<'a>),
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
    BindingPatternKind(&'a BindingPatternKind<'a>),
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
    ClassElement(&'a ClassElement<'a>),
    MethodDefinition(&'a MethodDefinition<'a>),
    PropertyDefinition(&'a PropertyDefinition<'a>),
    PrivateIdentifier(&'a PrivateIdentifier<'a>),
    StaticBlock(&'a StaticBlock<'a>),
    ModuleDeclaration(&'a ModuleDeclaration<'a>),
    AccessorProperty(&'a AccessorProperty<'a>),
    ImportExpression(&'a ImportExpression<'a>),
    ImportDeclaration(&'a ImportDeclaration<'a>),
    ImportDeclarationSpecifier(&'a ImportDeclarationSpecifier<'a>),
    ImportSpecifier(&'a ImportSpecifier<'a>),
    ImportDefaultSpecifier(&'a ImportDefaultSpecifier<'a>),
    ImportNamespaceSpecifier(&'a ImportNamespaceSpecifier<'a>),
    WithClause(&'a WithClause<'a>),
    ImportAttribute(&'a ImportAttribute<'a>),
    ImportAttributeKey(&'a ImportAttributeKey<'a>),
    ExportNamedDeclaration(&'a ExportNamedDeclaration<'a>),
    ExportDefaultDeclaration(&'a ExportDefaultDeclaration<'a>),
    ExportAllDeclaration(&'a ExportAllDeclaration<'a>),
    ExportSpecifier(&'a ExportSpecifier<'a>),
    ExportDefaultDeclarationKind(&'a ExportDefaultDeclarationKind<'a>),
    ModuleExportName(&'a ModuleExportName<'a>),
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
    JSXElementName(&'a JSXElementName<'a>),
    JSXNamespacedName(&'a JSXNamespacedName<'a>),
    JSXMemberExpression(&'a JSXMemberExpression<'a>),
    JSXMemberExpressionObject(&'a JSXMemberExpressionObject<'a>),
    JSXExpressionContainer(&'a JSXExpressionContainer<'a>),
    JSXExpression(&'a JSXExpression<'a>),
    JSXEmptyExpression(&'a JSXEmptyExpression),
    JSXAttributeItem(&'a JSXAttributeItem<'a>),
    JSXAttribute(&'a JSXAttribute<'a>),
    JSXSpreadAttribute(&'a JSXSpreadAttribute<'a>),
    JSXAttributeName(&'a JSXAttributeName<'a>),
    JSXAttributeValue(&'a JSXAttributeValue<'a>),
    JSXIdentifier(&'a JSXIdentifier<'a>),
    JSXChild(&'a JSXChild<'a>),
    JSXSpreadChild(&'a JSXSpreadChild<'a>),
    JSXText(&'a JSXText<'a>),
    TSThisParameter(&'a TSThisParameter<'a>),
    TSEnumDeclaration(&'a TSEnumDeclaration<'a>),
    TSEnumBody(&'a TSEnumBody<'a>),
    TSEnumMember(&'a TSEnumMember<'a>),
    TSEnumMemberName(&'a TSEnumMemberName<'a>),
    TSTypeAnnotation(&'a TSTypeAnnotation<'a>),
    TSLiteralType(&'a TSLiteralType<'a>),
    TSLiteral(&'a TSLiteral<'a>),
    TSType(&'a TSType<'a>),
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
    TSTupleElement(&'a TSTupleElement<'a>),
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
    TSTypeName(&'a TSTypeName<'a>),
    TSQualifiedName(&'a TSQualifiedName<'a>),
    TSTypeParameterInstantiation(&'a TSTypeParameterInstantiation<'a>),
    TSTypeParameter(&'a TSTypeParameter<'a>),
    TSTypeParameterDeclaration(&'a TSTypeParameterDeclaration<'a>),
    TSTypeAliasDeclaration(&'a TSTypeAliasDeclaration<'a>),
    TSClassImplements(&'a TSClassImplements<'a>),
    TSInterfaceDeclaration(&'a TSInterfaceDeclaration<'a>),
    TSInterfaceBody(&'a TSInterfaceBody<'a>),
    TSPropertySignature(&'a TSPropertySignature<'a>),
    TSSignature(&'a TSSignature<'a>),
    TSIndexSignature(&'a TSIndexSignature<'a>),
    TSCallSignatureDeclaration(&'a TSCallSignatureDeclaration<'a>),
    TSMethodSignature(&'a TSMethodSignature<'a>),
    TSConstructSignatureDeclaration(&'a TSConstructSignatureDeclaration<'a>),
    TSIndexSignatureName(&'a TSIndexSignatureName<'a>),
    TSInterfaceHeritage(&'a TSInterfaceHeritage<'a>),
    TSTypePredicate(&'a TSTypePredicate<'a>),
    TSTypePredicateName(&'a TSTypePredicateName<'a>),
    TSModuleDeclaration(&'a TSModuleDeclaration<'a>),
    TSModuleDeclarationName(&'a TSModuleDeclarationName<'a>),
    TSModuleDeclarationBody(&'a TSModuleDeclarationBody<'a>),
    TSModuleBlock(&'a TSModuleBlock<'a>),
    TSTypeLiteral(&'a TSTypeLiteral<'a>),
    TSInferType(&'a TSInferType<'a>),
    TSTypeQuery(&'a TSTypeQuery<'a>),
    TSTypeQueryExprName(&'a TSTypeQueryExprName<'a>),
    TSImportType(&'a TSImportType<'a>),
    TSFunctionType(&'a TSFunctionType<'a>),
    TSConstructorType(&'a TSConstructorType<'a>),
    TSMappedType(&'a TSMappedType<'a>),
    TSTemplateLiteralType(&'a TSTemplateLiteralType<'a>),
    TSAsExpression(&'a TSAsExpression<'a>),
    TSSatisfiesExpression(&'a TSSatisfiesExpression<'a>),
    TSTypeAssertion(&'a TSTypeAssertion<'a>),
    TSImportEqualsDeclaration(&'a TSImportEqualsDeclaration<'a>),
    TSModuleReference(&'a TSModuleReference<'a>),
    TSExternalModuleReference(&'a TSExternalModuleReference<'a>),
    TSNonNullExpression(&'a TSNonNullExpression<'a>),
    Decorator(&'a Decorator<'a>),
    TSExportAssignment(&'a TSExportAssignment<'a>),
    TSNamespaceExportDeclaration(&'a TSNamespaceExportDeclaration<'a>),
    TSInstantiationExpression(&'a TSInstantiationExpression<'a>),
    JSDocNullableType(&'a JSDocNullableType<'a>),
    JSDocNonNullableType(&'a JSDocNonNullableType<'a>),
    JSDocUnknownType(&'a JSDocUnknownType),
    Span(&'a Span),
}
impl FollowingNode<'_> {
    pub fn span(&self) -> oxc_span::Span {
        match self {
            Self::Program(n) => n.span(),
            Self::Expression(n) => n.span(),
            Self::IdentifierName(n) => n.span(),
            Self::IdentifierReference(n) => n.span(),
            Self::BindingIdentifier(n) => n.span(),
            Self::LabelIdentifier(n) => n.span(),
            Self::ThisExpression(n) => n.span(),
            Self::ArrayExpression(n) => n.span(),
            Self::ArrayExpressionElement(n) => n.span(),
            Self::Elision(n) => n.span(),
            Self::ObjectExpression(n) => n.span(),
            Self::ObjectPropertyKind(n) => n.span(),
            Self::ObjectProperty(n) => n.span(),
            Self::PropertyKey(n) => n.span(),
            Self::TemplateLiteral(n) => n.span(),
            Self::TaggedTemplateExpression(n) => n.span(),
            Self::TemplateElement(n) => n.span(),
            Self::MemberExpression(n) => n.span(),
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
            Self::AssignmentTarget(n) => n.span(),
            Self::SimpleAssignmentTarget(n) => n.span(),
            Self::AssignmentTargetPattern(n) => n.span(),
            Self::ArrayAssignmentTarget(n) => n.span(),
            Self::ObjectAssignmentTarget(n) => n.span(),
            Self::AssignmentTargetRest(n) => n.span(),
            Self::AssignmentTargetMaybeDefault(n) => n.span(),
            Self::AssignmentTargetWithDefault(n) => n.span(),
            Self::AssignmentTargetProperty(n) => n.span(),
            Self::AssignmentTargetPropertyIdentifier(n) => n.span(),
            Self::AssignmentTargetPropertyProperty(n) => n.span(),
            Self::SequenceExpression(n) => n.span(),
            Self::Super(n) => n.span(),
            Self::AwaitExpression(n) => n.span(),
            Self::ChainExpression(n) => n.span(),
            Self::ChainElement(n) => n.span(),
            Self::ParenthesizedExpression(n) => n.span(),
            Self::Statement(n) => n.span(),
            Self::Directive(n) => n.span(),
            Self::Hashbang(n) => n.span(),
            Self::BlockStatement(n) => n.span(),
            Self::Declaration(n) => n.span(),
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
            Self::ForStatementLeft(n) => n.span(),
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
            Self::BindingPatternKind(n) => n.span(),
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
            Self::ClassElement(n) => n.span(),
            Self::MethodDefinition(n) => n.span(),
            Self::PropertyDefinition(n) => n.span(),
            Self::PrivateIdentifier(n) => n.span(),
            Self::StaticBlock(n) => n.span(),
            Self::ModuleDeclaration(n) => n.span(),
            Self::AccessorProperty(n) => n.span(),
            Self::ImportExpression(n) => n.span(),
            Self::ImportDeclaration(n) => n.span(),
            Self::ImportDeclarationSpecifier(n) => n.span(),
            Self::ImportSpecifier(n) => n.span(),
            Self::ImportDefaultSpecifier(n) => n.span(),
            Self::ImportNamespaceSpecifier(n) => n.span(),
            Self::WithClause(n) => n.span(),
            Self::ImportAttribute(n) => n.span(),
            Self::ImportAttributeKey(n) => n.span(),
            Self::ExportNamedDeclaration(n) => n.span(),
            Self::ExportDefaultDeclaration(n) => n.span(),
            Self::ExportAllDeclaration(n) => n.span(),
            Self::ExportSpecifier(n) => n.span(),
            Self::ExportDefaultDeclarationKind(n) => n.span(),
            Self::ModuleExportName(n) => n.span(),
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
            Self::JSXElementName(n) => n.span(),
            Self::JSXNamespacedName(n) => n.span(),
            Self::JSXMemberExpression(n) => n.span(),
            Self::JSXMemberExpressionObject(n) => n.span(),
            Self::JSXExpressionContainer(n) => n.span(),
            Self::JSXExpression(n) => n.span(),
            Self::JSXEmptyExpression(n) => n.span(),
            Self::JSXAttributeItem(n) => n.span(),
            Self::JSXAttribute(n) => n.span(),
            Self::JSXSpreadAttribute(n) => n.span(),
            Self::JSXAttributeName(n) => n.span(),
            Self::JSXAttributeValue(n) => n.span(),
            Self::JSXIdentifier(n) => n.span(),
            Self::JSXChild(n) => n.span(),
            Self::JSXSpreadChild(n) => n.span(),
            Self::JSXText(n) => n.span(),
            Self::TSThisParameter(n) => n.span(),
            Self::TSEnumDeclaration(n) => n.span(),
            Self::TSEnumBody(n) => n.span(),
            Self::TSEnumMember(n) => n.span(),
            Self::TSEnumMemberName(n) => n.span(),
            Self::TSTypeAnnotation(n) => n.span(),
            Self::TSLiteralType(n) => n.span(),
            Self::TSLiteral(n) => n.span(),
            Self::TSType(n) => n.span(),
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
            Self::TSTupleElement(n) => n.span(),
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
            Self::TSInterfaceBody(n) => n.span(),
            Self::TSPropertySignature(n) => n.span(),
            Self::TSSignature(n) => n.span(),
            Self::TSIndexSignature(n) => n.span(),
            Self::TSCallSignatureDeclaration(n) => n.span(),
            Self::TSMethodSignature(n) => n.span(),
            Self::TSConstructSignatureDeclaration(n) => n.span(),
            Self::TSIndexSignatureName(n) => n.span(),
            Self::TSInterfaceHeritage(n) => n.span(),
            Self::TSTypePredicate(n) => n.span(),
            Self::TSTypePredicateName(n) => n.span(),
            Self::TSModuleDeclaration(n) => n.span(),
            Self::TSModuleDeclarationName(n) => n.span(),
            Self::TSModuleDeclarationBody(n) => n.span(),
            Self::TSModuleBlock(n) => n.span(),
            Self::TSTypeLiteral(n) => n.span(),
            Self::TSInferType(n) => n.span(),
            Self::TSTypeQuery(n) => n.span(),
            Self::TSTypeQueryExprName(n) => n.span(),
            Self::TSImportType(n) => n.span(),
            Self::TSFunctionType(n) => n.span(),
            Self::TSConstructorType(n) => n.span(),
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
            Self::TSNamespaceExportDeclaration(n) => n.span(),
            Self::TSInstantiationExpression(n) => n.span(),
            Self::JSDocNullableType(n) => n.span(),
            Self::JSDocNonNullableType(n) => n.span(),
            Self::JSDocUnknownType(n) => n.span(),
            Self::Span(n) => n.span(),
        }
    }
}
impl<'a> AstNodes<'a> {
    #[inline]
    pub fn span(&self) -> Span {
        match self {
            Self::Dummy() => SPAN,
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
            Self::JSDocNullableType(n) => n.parent,
            Self::JSDocNonNullableType(n) => n.parent,
            Self::JSDocUnknownType(n) => n.parent,
        }
    }
    #[inline]
    pub fn following_node(&self) -> Option<&FollowingNode<'a>> {
        match self {
            Self::Dummy() => None,
            Self::Program(n) => n.following_node.as_ref(),
            Self::IdentifierName(n) => n.following_node.as_ref(),
            Self::IdentifierReference(n) => n.following_node.as_ref(),
            Self::BindingIdentifier(n) => n.following_node.as_ref(),
            Self::LabelIdentifier(n) => n.following_node.as_ref(),
            Self::ThisExpression(n) => n.following_node.as_ref(),
            Self::ArrayExpression(n) => n.following_node.as_ref(),
            Self::ArrayExpressionElement(n) => n.following_node.as_ref(),
            Self::Elision(n) => n.following_node.as_ref(),
            Self::ObjectExpression(n) => n.following_node.as_ref(),
            Self::ObjectProperty(n) => n.following_node.as_ref(),
            Self::PropertyKey(n) => n.following_node.as_ref(),
            Self::TemplateLiteral(n) => n.following_node.as_ref(),
            Self::TaggedTemplateExpression(n) => n.following_node.as_ref(),
            Self::MemberExpression(n) => n.following_node.as_ref(),
            Self::CallExpression(n) => n.following_node.as_ref(),
            Self::NewExpression(n) => n.following_node.as_ref(),
            Self::MetaProperty(n) => n.following_node.as_ref(),
            Self::SpreadElement(n) => n.following_node.as_ref(),
            Self::Argument(n) => n.following_node.as_ref(),
            Self::UpdateExpression(n) => n.following_node.as_ref(),
            Self::UnaryExpression(n) => n.following_node.as_ref(),
            Self::BinaryExpression(n) => n.following_node.as_ref(),
            Self::PrivateInExpression(n) => n.following_node.as_ref(),
            Self::LogicalExpression(n) => n.following_node.as_ref(),
            Self::ConditionalExpression(n) => n.following_node.as_ref(),
            Self::AssignmentExpression(n) => n.following_node.as_ref(),
            Self::AssignmentTarget(n) => n.following_node.as_ref(),
            Self::SimpleAssignmentTarget(n) => n.following_node.as_ref(),
            Self::AssignmentTargetPattern(n) => n.following_node.as_ref(),
            Self::ArrayAssignmentTarget(n) => n.following_node.as_ref(),
            Self::ObjectAssignmentTarget(n) => n.following_node.as_ref(),
            Self::AssignmentTargetWithDefault(n) => n.following_node.as_ref(),
            Self::SequenceExpression(n) => n.following_node.as_ref(),
            Self::Super(n) => n.following_node.as_ref(),
            Self::AwaitExpression(n) => n.following_node.as_ref(),
            Self::ChainExpression(n) => n.following_node.as_ref(),
            Self::ParenthesizedExpression(n) => n.following_node.as_ref(),
            Self::Directive(n) => n.following_node.as_ref(),
            Self::Hashbang(n) => n.following_node.as_ref(),
            Self::BlockStatement(n) => n.following_node.as_ref(),
            Self::VariableDeclaration(n) => n.following_node.as_ref(),
            Self::VariableDeclarator(n) => n.following_node.as_ref(),
            Self::EmptyStatement(n) => n.following_node.as_ref(),
            Self::ExpressionStatement(n) => n.following_node.as_ref(),
            Self::IfStatement(n) => n.following_node.as_ref(),
            Self::DoWhileStatement(n) => n.following_node.as_ref(),
            Self::WhileStatement(n) => n.following_node.as_ref(),
            Self::ForStatement(n) => n.following_node.as_ref(),
            Self::ForStatementInit(n) => n.following_node.as_ref(),
            Self::ForInStatement(n) => n.following_node.as_ref(),
            Self::ForOfStatement(n) => n.following_node.as_ref(),
            Self::ContinueStatement(n) => n.following_node.as_ref(),
            Self::BreakStatement(n) => n.following_node.as_ref(),
            Self::ReturnStatement(n) => n.following_node.as_ref(),
            Self::WithStatement(n) => n.following_node.as_ref(),
            Self::SwitchStatement(n) => n.following_node.as_ref(),
            Self::SwitchCase(n) => n.following_node.as_ref(),
            Self::LabeledStatement(n) => n.following_node.as_ref(),
            Self::ThrowStatement(n) => n.following_node.as_ref(),
            Self::TryStatement(n) => n.following_node.as_ref(),
            Self::CatchClause(n) => n.following_node.as_ref(),
            Self::CatchParameter(n) => n.following_node.as_ref(),
            Self::DebuggerStatement(n) => n.following_node.as_ref(),
            Self::AssignmentPattern(n) => n.following_node.as_ref(),
            Self::ObjectPattern(n) => n.following_node.as_ref(),
            Self::ArrayPattern(n) => n.following_node.as_ref(),
            Self::BindingRestElement(n) => n.following_node.as_ref(),
            Self::Function(n) => n.following_node.as_ref(),
            Self::FormalParameters(n) => n.following_node.as_ref(),
            Self::FormalParameter(n) => n.following_node.as_ref(),
            Self::FunctionBody(n) => n.following_node.as_ref(),
            Self::ArrowFunctionExpression(n) => n.following_node.as_ref(),
            Self::YieldExpression(n) => n.following_node.as_ref(),
            Self::Class(n) => n.following_node.as_ref(),
            Self::ClassBody(n) => n.following_node.as_ref(),
            Self::MethodDefinition(n) => n.following_node.as_ref(),
            Self::PropertyDefinition(n) => n.following_node.as_ref(),
            Self::PrivateIdentifier(n) => n.following_node.as_ref(),
            Self::StaticBlock(n) => n.following_node.as_ref(),
            Self::ModuleDeclaration(n) => n.following_node.as_ref(),
            Self::ImportExpression(n) => n.following_node.as_ref(),
            Self::ImportDeclaration(n) => n.following_node.as_ref(),
            Self::ImportSpecifier(n) => n.following_node.as_ref(),
            Self::ImportDefaultSpecifier(n) => n.following_node.as_ref(),
            Self::ImportNamespaceSpecifier(n) => n.following_node.as_ref(),
            Self::ExportNamedDeclaration(n) => n.following_node.as_ref(),
            Self::ExportDefaultDeclaration(n) => n.following_node.as_ref(),
            Self::ExportAllDeclaration(n) => n.following_node.as_ref(),
            Self::ExportSpecifier(n) => n.following_node.as_ref(),
            Self::V8IntrinsicExpression(n) => n.following_node.as_ref(),
            Self::BooleanLiteral(n) => n.following_node.as_ref(),
            Self::NullLiteral(n) => n.following_node.as_ref(),
            Self::NumericLiteral(n) => n.following_node.as_ref(),
            Self::StringLiteral(n) => n.following_node.as_ref(),
            Self::BigIntLiteral(n) => n.following_node.as_ref(),
            Self::RegExpLiteral(n) => n.following_node.as_ref(),
            Self::JSXElement(n) => n.following_node.as_ref(),
            Self::JSXOpeningElement(n) => n.following_node.as_ref(),
            Self::JSXClosingElement(n) => n.following_node.as_ref(),
            Self::JSXFragment(n) => n.following_node.as_ref(),
            Self::JSXOpeningFragment(n) => n.following_node.as_ref(),
            Self::JSXClosingFragment(n) => n.following_node.as_ref(),
            Self::JSXNamespacedName(n) => n.following_node.as_ref(),
            Self::JSXMemberExpression(n) => n.following_node.as_ref(),
            Self::JSXExpressionContainer(n) => n.following_node.as_ref(),
            Self::JSXEmptyExpression(n) => n.following_node.as_ref(),
            Self::JSXAttribute(n) => n.following_node.as_ref(),
            Self::JSXSpreadAttribute(n) => n.following_node.as_ref(),
            Self::JSXIdentifier(n) => n.following_node.as_ref(),
            Self::JSXSpreadChild(n) => n.following_node.as_ref(),
            Self::JSXText(n) => n.following_node.as_ref(),
            Self::TSThisParameter(n) => n.following_node.as_ref(),
            Self::TSEnumDeclaration(n) => n.following_node.as_ref(),
            Self::TSEnumBody(n) => n.following_node.as_ref(),
            Self::TSEnumMember(n) => n.following_node.as_ref(),
            Self::TSTypeAnnotation(n) => n.following_node.as_ref(),
            Self::TSLiteralType(n) => n.following_node.as_ref(),
            Self::TSConditionalType(n) => n.following_node.as_ref(),
            Self::TSUnionType(n) => n.following_node.as_ref(),
            Self::TSIntersectionType(n) => n.following_node.as_ref(),
            Self::TSParenthesizedType(n) => n.following_node.as_ref(),
            Self::TSIndexedAccessType(n) => n.following_node.as_ref(),
            Self::TSNamedTupleMember(n) => n.following_node.as_ref(),
            Self::TSAnyKeyword(n) => n.following_node.as_ref(),
            Self::TSStringKeyword(n) => n.following_node.as_ref(),
            Self::TSBooleanKeyword(n) => n.following_node.as_ref(),
            Self::TSNumberKeyword(n) => n.following_node.as_ref(),
            Self::TSNeverKeyword(n) => n.following_node.as_ref(),
            Self::TSIntrinsicKeyword(n) => n.following_node.as_ref(),
            Self::TSUnknownKeyword(n) => n.following_node.as_ref(),
            Self::TSNullKeyword(n) => n.following_node.as_ref(),
            Self::TSUndefinedKeyword(n) => n.following_node.as_ref(),
            Self::TSVoidKeyword(n) => n.following_node.as_ref(),
            Self::TSSymbolKeyword(n) => n.following_node.as_ref(),
            Self::TSThisType(n) => n.following_node.as_ref(),
            Self::TSObjectKeyword(n) => n.following_node.as_ref(),
            Self::TSBigIntKeyword(n) => n.following_node.as_ref(),
            Self::TSTypeReference(n) => n.following_node.as_ref(),
            Self::TSTypeName(n) => n.following_node.as_ref(),
            Self::TSQualifiedName(n) => n.following_node.as_ref(),
            Self::TSTypeParameterInstantiation(n) => n.following_node.as_ref(),
            Self::TSTypeParameter(n) => n.following_node.as_ref(),
            Self::TSTypeParameterDeclaration(n) => n.following_node.as_ref(),
            Self::TSTypeAliasDeclaration(n) => n.following_node.as_ref(),
            Self::TSClassImplements(n) => n.following_node.as_ref(),
            Self::TSInterfaceDeclaration(n) => n.following_node.as_ref(),
            Self::TSPropertySignature(n) => n.following_node.as_ref(),
            Self::TSMethodSignature(n) => n.following_node.as_ref(),
            Self::TSConstructSignatureDeclaration(n) => n.following_node.as_ref(),
            Self::TSInterfaceHeritage(n) => n.following_node.as_ref(),
            Self::TSModuleDeclaration(n) => n.following_node.as_ref(),
            Self::TSModuleBlock(n) => n.following_node.as_ref(),
            Self::TSTypeLiteral(n) => n.following_node.as_ref(),
            Self::TSInferType(n) => n.following_node.as_ref(),
            Self::TSTypeQuery(n) => n.following_node.as_ref(),
            Self::TSImportType(n) => n.following_node.as_ref(),
            Self::TSMappedType(n) => n.following_node.as_ref(),
            Self::TSTemplateLiteralType(n) => n.following_node.as_ref(),
            Self::TSAsExpression(n) => n.following_node.as_ref(),
            Self::TSSatisfiesExpression(n) => n.following_node.as_ref(),
            Self::TSTypeAssertion(n) => n.following_node.as_ref(),
            Self::TSImportEqualsDeclaration(n) => n.following_node.as_ref(),
            Self::TSModuleReference(n) => n.following_node.as_ref(),
            Self::TSExternalModuleReference(n) => n.following_node.as_ref(),
            Self::TSNonNullExpression(n) => n.following_node.as_ref(),
            Self::Decorator(n) => n.following_node.as_ref(),
            Self::TSExportAssignment(n) => n.following_node.as_ref(),
            Self::TSInstantiationExpression(n) => n.following_node.as_ref(),
            Self::JSDocNullableType(n) => n.following_node.as_ref(),
            Self::JSDocNonNullableType(n) => n.following_node.as_ref(),
            Self::JSDocUnknownType(n) => n.following_node.as_ref(),
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
            Self::ArrayExpressionElement(_) => "ArrayExpressionElement",
            Self::Elision(_) => "Elision",
            Self::ObjectExpression(_) => "ObjectExpression",
            Self::ObjectProperty(_) => "ObjectProperty",
            Self::PropertyKey(_) => "PropertyKey",
            Self::TemplateLiteral(_) => "TemplateLiteral",
            Self::TaggedTemplateExpression(_) => "TaggedTemplateExpression",
            Self::MemberExpression(_) => "MemberExpression",
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
            Self::AssignmentTarget(_) => "AssignmentTarget",
            Self::SimpleAssignmentTarget(_) => "SimpleAssignmentTarget",
            Self::AssignmentTargetPattern(_) => "AssignmentTargetPattern",
            Self::ArrayAssignmentTarget(_) => "ArrayAssignmentTarget",
            Self::ObjectAssignmentTarget(_) => "ObjectAssignmentTarget",
            Self::AssignmentTargetWithDefault(_) => "AssignmentTargetWithDefault",
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
            Self::ForStatementInit(_) => "ForStatementInit",
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
            Self::ModuleDeclaration(_) => "ModuleDeclaration",
            Self::ImportExpression(_) => "ImportExpression",
            Self::ImportDeclaration(_) => "ImportDeclaration",
            Self::ImportSpecifier(_) => "ImportSpecifier",
            Self::ImportDefaultSpecifier(_) => "ImportDefaultSpecifier",
            Self::ImportNamespaceSpecifier(_) => "ImportNamespaceSpecifier",
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
            Self::TSIndexedAccessType(_) => "TSIndexedAccessType",
            Self::TSNamedTupleMember(_) => "TSNamedTupleMember",
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
            Self::TSTypeName(_) => "TSTypeName",
            Self::TSQualifiedName(_) => "TSQualifiedName",
            Self::TSTypeParameterInstantiation(_) => "TSTypeParameterInstantiation",
            Self::TSTypeParameter(_) => "TSTypeParameter",
            Self::TSTypeParameterDeclaration(_) => "TSTypeParameterDeclaration",
            Self::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration",
            Self::TSClassImplements(_) => "TSClassImplements",
            Self::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration",
            Self::TSPropertySignature(_) => "TSPropertySignature",
            Self::TSMethodSignature(_) => "TSMethodSignature",
            Self::TSConstructSignatureDeclaration(_) => "TSConstructSignatureDeclaration",
            Self::TSInterfaceHeritage(_) => "TSInterfaceHeritage",
            Self::TSModuleDeclaration(_) => "TSModuleDeclaration",
            Self::TSModuleBlock(_) => "TSModuleBlock",
            Self::TSTypeLiteral(_) => "TSTypeLiteral",
            Self::TSInferType(_) => "TSInferType",
            Self::TSTypeQuery(_) => "TSTypeQuery",
            Self::TSImportType(_) => "TSImportType",
            Self::TSMappedType(_) => "TSMappedType",
            Self::TSTemplateLiteralType(_) => "TSTemplateLiteralType",
            Self::TSAsExpression(_) => "TSAsExpression",
            Self::TSSatisfiesExpression(_) => "TSSatisfiesExpression",
            Self::TSTypeAssertion(_) => "TSTypeAssertion",
            Self::TSImportEqualsDeclaration(_) => "TSImportEqualsDeclaration",
            Self::TSModuleReference(_) => "TSModuleReference",
            Self::TSExternalModuleReference(_) => "TSExternalModuleReference",
            Self::TSNonNullExpression(_) => "TSNonNullExpression",
            Self::Decorator(_) => "Decorator",
            Self::TSExportAssignment(_) => "TSExportAssignment",
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
    pub(super) following_node: Option<FollowingNode<'a>>,
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
                following_node: self.following_node.clone(),
            }))
            .as_ref()
    }
}

const PROGRAM_OFFSET_HASHBANG: usize = std::mem::offset_of!(Program, hashbang);
const PROGRAM_OFFSET_DIRECTIVES: usize = std::mem::offset_of!(Program, directives);
const PROGRAM_OFFSET_BODY: usize = std::mem::offset_of!(Program, body);

impl<'a> AstNode<'a, Program<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
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
    pub fn hashbang(&self) -> Option<&AstNode<'a, Hashbang<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(PROGRAM_OFFSET_DIRECTIVES) as *const Vec<'a, Directive<'a>>) }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::Directive(t));
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
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(PROGRAM_OFFSET_BODY) as *const Vec<'a, Statement<'a>>) }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::Statement(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Program(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Program(transmute_self(self))),
            following_node,
        })
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
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::NullLiteral(s) => AstNodes::NullLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            Expression::NumericLiteral(s) => {
                AstNodes::NumericLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::BigIntLiteral(s) => {
                AstNodes::BigIntLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::RegExpLiteral(s) => {
                AstNodes::RegExpLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::TemplateLiteral(s) => {
                AstNodes::TemplateLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::Identifier(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::MetaProperty(s) => AstNodes::MetaProperty(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            Expression::Super(s) => AstNodes::Super(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            Expression::ArrayExpression(s) => {
                AstNodes::ArrayExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::ArrowFunctionExpression(s) => {
                AstNodes::ArrowFunctionExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::AssignmentExpression(s) => {
                AstNodes::AssignmentExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::AwaitExpression(s) => {
                AstNodes::AwaitExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::BinaryExpression(s) => {
                AstNodes::BinaryExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::CallExpression(s) => {
                AstNodes::CallExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::ChainExpression(s) => {
                AstNodes::ChainExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::ClassExpression(s) => AstNodes::Class(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            Expression::ConditionalExpression(s) => {
                AstNodes::ConditionalExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::FunctionExpression(s) => {
                AstNodes::Function(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::ImportExpression(s) => {
                AstNodes::ImportExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::LogicalExpression(s) => {
                AstNodes::LogicalExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::NewExpression(s) => {
                AstNodes::NewExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::ObjectExpression(s) => {
                AstNodes::ObjectExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::ParenthesizedExpression(s) => {
                AstNodes::ParenthesizedExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::SequenceExpression(s) => {
                AstNodes::SequenceExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::TaggedTemplateExpression(s) => {
                AstNodes::TaggedTemplateExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::ThisExpression(s) => {
                AstNodes::ThisExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::UnaryExpression(s) => {
                AstNodes::UnaryExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::UpdateExpression(s) => {
                AstNodes::UpdateExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::YieldExpression(s) => {
                AstNodes::YieldExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::PrivateInExpression(s) => {
                AstNodes::PrivateInExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::JSXElement(s) => AstNodes::JSXElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            Expression::JSXFragment(s) => AstNodes::JSXFragment(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            Expression::TSAsExpression(s) => {
                AstNodes::TSAsExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::TSSatisfiesExpression(s) => {
                AstNodes::TSSatisfiesExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::TSTypeAssertion(s) => {
                AstNodes::TSTypeAssertion(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::TSInstantiationExpression(s) => {
                AstNodes::TSInstantiationExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Expression::V8IntrinsicExpression(s) => {
                AstNodes::V8IntrinsicExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            it @ match_member_expression!(Expression) => {
                AstNodes::MemberExpression(self.allocator.alloc(AstNode {
                    inner: it.to_member_expression(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
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
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}

impl<'a> AstNode<'a, IdentifierReference<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}

impl<'a> AstNode<'a, BindingIdentifier<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}

impl<'a> AstNode<'a, LabelIdentifier<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}

impl<'a> AstNode<'a, ThisExpression> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

const ARRAY_EXPRESSION_OFFSET_ELEMENTS: usize = std::mem::offset_of!(ArrayExpression, elements);

impl<'a> AstNode<'a, ArrayExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn elements(&self) -> &AstNode<'a, Vec<'a, ArrayExpressionElement<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrayExpression(transmute_self(self))),
            following_node,
        })
    }
}

impl<'a> AstNode<'a, ArrayExpressionElement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.allocator.alloc(AstNodes::ArrayExpressionElement(transmute_self(self)));
        let node = match self.inner {
            ArrayExpressionElement::SpreadElement(s) => {
                AstNodes::SpreadElement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            ArrayExpressionElement::Elision(s) => {
                AstNodes::Elision(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            it @ match_expression!(ArrayExpressionElement) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node.clone(),
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
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

const OBJECT_EXPRESSION_OFFSET_PROPERTIES: usize =
    std::mem::offset_of!(ObjectExpression, properties);

impl<'a> AstNode<'a, ObjectExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn properties(&self) -> &AstNode<'a, Vec<'a, ObjectPropertyKind<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectExpression(transmute_self(self))),
            following_node,
        })
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
                    following_node: self.following_node.clone(),
                }))
            }
            ObjectPropertyKind::SpreadProperty(s) => {
                AstNodes::SpreadElement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const OBJECT_PROPERTY_OFFSET_KEY: usize = std::mem::offset_of!(ObjectProperty, key);
const OBJECT_PROPERTY_OFFSET_VALUE: usize = std::mem::offset_of!(ObjectProperty, value);

impl<'a> AstNode<'a, ObjectProperty<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn kind(&self) -> PropertyKind {
        self.inner.kind
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = Some(FollowingNode::Expression({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(OBJECT_PROPERTY_OFFSET_VALUE) as *const Expression<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectProperty(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn value(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
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
}

impl<'a> AstNode<'a, PropertyKey<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.allocator.alloc(AstNodes::PropertyKey(transmute_self(self)));
        let node = match self.inner {
            PropertyKey::StaticIdentifier(s) => {
                AstNodes::IdentifierName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            PropertyKey::PrivateIdentifier(s) => {
                AstNodes::PrivateIdentifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            it @ match_expression!(PropertyKey) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node.clone(),
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

const TEMPLATE_LITERAL_OFFSET_QUASIS: usize = std::mem::offset_of!(TemplateLiteral, quasis);
const TEMPLATE_LITERAL_OFFSET_EXPRESSIONS: usize =
    std::mem::offset_of!(TemplateLiteral, expressions);

impl<'a> AstNode<'a, TemplateLiteral<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn quasis(&self) -> &AstNode<'a, Vec<'a, TemplateElement<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TEMPLATE_LITERAL_OFFSET_EXPRESSIONS) as *const Vec<'a, Expression<'a>>)
            }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::Expression(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasis,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TemplateLiteral(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn expressions(&self) -> &AstNode<'a, Vec<'a, Expression<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.expressions,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TemplateLiteral(transmute_self(self))),
            following_node,
        })
    }
}

const TAGGED_TEMPLATE_EXPRESSION_OFFSET_TAG: usize =
    std::mem::offset_of!(TaggedTemplateExpression, tag);
const TAGGED_TEMPLATE_EXPRESSION_OFFSET_TYPEARGUMENTS: usize =
    std::mem::offset_of!(TaggedTemplateExpression, type_arguments);
const TAGGED_TEMPLATE_EXPRESSION_OFFSET_QUASI: usize =
    std::mem::offset_of!(TaggedTemplateExpression, quasi);

impl<'a> AstNode<'a, TaggedTemplateExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn tag(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TAGGED_TEMPLATE_EXPRESSION_OFFSET_TYPEARGUMENTS)
                    as *const Option<Box<'a, TSTypeParameterInstantiation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterInstantiation(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.tag,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TaggedTemplateExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = Some(FollowingNode::TemplateLiteral({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TAGGED_TEMPLATE_EXPRESSION_OFFSET_QUASI) as *const TemplateLiteral<'a>)
            }
        }));
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
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasi,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TaggedTemplateExpression(transmute_self(self))),
            following_node,
        })
    }
}

impl<'a> AstNode<'a, TemplateElement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
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
}

impl<'a> AstNode<'a, MemberExpression<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.allocator.alloc(AstNodes::MemberExpression(transmute_self(self)));
        let node = match self.inner {
            MemberExpression::ComputedMemberExpression(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            MemberExpression::StaticMemberExpression(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            MemberExpression::PrivateFieldExpression(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
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

const COMPUTED_MEMBER_EXPRESSION_OFFSET_OBJECT: usize =
    std::mem::offset_of!(ComputedMemberExpression, object);
const COMPUTED_MEMBER_EXPRESSION_OFFSET_EXPRESSION: usize =
    std::mem::offset_of!(ComputedMemberExpression, expression);

impl<'a> AstNode<'a, ComputedMemberExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn object(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::Expression({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(COMPUTED_MEMBER_EXPRESSION_OFFSET_EXPRESSION) as *const Expression<'a>)
            }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }
}

const STATIC_MEMBER_EXPRESSION_OFFSET_OBJECT: usize =
    std::mem::offset_of!(StaticMemberExpression, object);
const STATIC_MEMBER_EXPRESSION_OFFSET_PROPERTY: usize =
    std::mem::offset_of!(StaticMemberExpression, property);

impl<'a> AstNode<'a, StaticMemberExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn object(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::IdentifierName({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(STATIC_MEMBER_EXPRESSION_OFFSET_PROPERTY) as *const IdentifierName<'a>)
            }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn property(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }
}

const PRIVATE_FIELD_EXPRESSION_OFFSET_OBJECT: usize =
    std::mem::offset_of!(PrivateFieldExpression, object);
const PRIVATE_FIELD_EXPRESSION_OFFSET_FIELD: usize =
    std::mem::offset_of!(PrivateFieldExpression, field);

impl<'a> AstNode<'a, PrivateFieldExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn object(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::PrivateIdentifier({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(PRIVATE_FIELD_EXPRESSION_OFFSET_FIELD) as *const PrivateIdentifier<'a>)
            }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn field(&self) -> &AstNode<'a, PrivateIdentifier<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.field,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn optional(&self) -> bool {
        self.inner.optional
    }
}

const CALL_EXPRESSION_OFFSET_CALLEE: usize = std::mem::offset_of!(CallExpression, callee);
const CALL_EXPRESSION_OFFSET_TYPEARGUMENTS: usize =
    std::mem::offset_of!(CallExpression, type_arguments);
const CALL_EXPRESSION_OFFSET_ARGUMENTS: usize = std::mem::offset_of!(CallExpression, arguments);

impl<'a> AstNode<'a, CallExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn callee(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(CALL_EXPRESSION_OFFSET_TYPEARGUMENTS)
                    as *const Option<Box<'a, TSTypeParameterInstantiation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterInstantiation(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.callee,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::CallExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(CALL_EXPRESSION_OFFSET_ARGUMENTS) as *const Vec<'a, Argument<'a>>) }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::Argument(t));
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
        let following_node = self.following_node.clone();
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
}

const NEW_EXPRESSION_OFFSET_CALLEE: usize = std::mem::offset_of!(NewExpression, callee);
const NEW_EXPRESSION_OFFSET_TYPEARGUMENTS: usize =
    std::mem::offset_of!(NewExpression, type_arguments);
const NEW_EXPRESSION_OFFSET_ARGUMENTS: usize = std::mem::offset_of!(NewExpression, arguments);

impl<'a> AstNode<'a, NewExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn callee(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(NEW_EXPRESSION_OFFSET_TYPEARGUMENTS)
                    as *const Option<Box<'a, TSTypeParameterInstantiation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterInstantiation(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.callee,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::NewExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(NEW_EXPRESSION_OFFSET_ARGUMENTS) as *const Vec<'a, Argument<'a>>) }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::Argument(t));
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
        let following_node = self.following_node.clone();
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
}

const META_PROPERTY_OFFSET_META: usize = std::mem::offset_of!(MetaProperty, meta);
const META_PROPERTY_OFFSET_PROPERTY: usize = std::mem::offset_of!(MetaProperty, property);

impl<'a> AstNode<'a, MetaProperty<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn meta(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = Some(FollowingNode::IdentifierName({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(META_PROPERTY_OFFSET_PROPERTY) as *const IdentifierName<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.meta,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MetaProperty(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn property(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MetaProperty(transmute_self(self))),
            following_node,
        })
    }
}

const SPREAD_ELEMENT_OFFSET_ARGUMENT: usize = std::mem::offset_of!(SpreadElement, argument);

impl<'a> AstNode<'a, SpreadElement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SpreadElement(transmute_self(self))),
            following_node,
        })
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
                following_node: self.following_node.clone(),
            })),
            it @ match_expression!(Argument) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node.clone(),
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

const UPDATE_EXPRESSION_OFFSET_ARGUMENT: usize = std::mem::offset_of!(UpdateExpression, argument);

impl<'a> AstNode<'a, UpdateExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
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
    pub fn argument(&self) -> &AstNode<'a, SimpleAssignmentTarget<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::UpdateExpression(transmute_self(self))),
            following_node,
        })
    }
}

const UNARY_EXPRESSION_OFFSET_ARGUMENT: usize = std::mem::offset_of!(UnaryExpression, argument);

impl<'a> AstNode<'a, UnaryExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn operator(&self) -> UnaryOperator {
        self.inner.operator
    }

    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::UnaryExpression(transmute_self(self))),
            following_node,
        })
    }
}

const BINARY_EXPRESSION_OFFSET_LEFT: usize = std::mem::offset_of!(BinaryExpression, left);
const BINARY_EXPRESSION_OFFSET_RIGHT: usize = std::mem::offset_of!(BinaryExpression, right);

impl<'a> AstNode<'a, BinaryExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn left(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::Expression({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(BINARY_EXPRESSION_OFFSET_RIGHT) as *const Expression<'a>) }
        }));
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
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BinaryExpression(transmute_self(self))),
            following_node,
        })
    }
}

const PRIVATE_IN_EXPRESSION_OFFSET_LEFT: usize = std::mem::offset_of!(PrivateInExpression, left);
const PRIVATE_IN_EXPRESSION_OFFSET_RIGHT: usize = std::mem::offset_of!(PrivateInExpression, right);

impl<'a> AstNode<'a, PrivateInExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn left(&self) -> &AstNode<'a, PrivateIdentifier<'a>> {
        let following_node = Some(FollowingNode::Expression({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(PRIVATE_IN_EXPRESSION_OFFSET_RIGHT) as *const Expression<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PrivateInExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PrivateInExpression(transmute_self(self))),
            following_node,
        })
    }
}

const LOGICAL_EXPRESSION_OFFSET_LEFT: usize = std::mem::offset_of!(LogicalExpression, left);
const LOGICAL_EXPRESSION_OFFSET_RIGHT: usize = std::mem::offset_of!(LogicalExpression, right);

impl<'a> AstNode<'a, LogicalExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn left(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::Expression({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(LOGICAL_EXPRESSION_OFFSET_RIGHT) as *const Expression<'a>) }
        }));
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
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LogicalExpression(transmute_self(self))),
            following_node,
        })
    }
}

const CONDITIONAL_EXPRESSION_OFFSET_TEST: usize = std::mem::offset_of!(ConditionalExpression, test);
const CONDITIONAL_EXPRESSION_OFFSET_CONSEQUENT: usize =
    std::mem::offset_of!(ConditionalExpression, consequent);
const CONDITIONAL_EXPRESSION_OFFSET_ALTERNATE: usize =
    std::mem::offset_of!(ConditionalExpression, alternate);

impl<'a> AstNode<'a, ConditionalExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn test(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::Expression({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(CONDITIONAL_EXPRESSION_OFFSET_CONSEQUENT) as *const Expression<'a>)
            }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ConditionalExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn consequent(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::Expression({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(CONDITIONAL_EXPRESSION_OFFSET_ALTERNATE) as *const Expression<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ConditionalExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn alternate(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.alternate,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ConditionalExpression(transmute_self(self))),
            following_node,
        })
    }
}

const ASSIGNMENT_EXPRESSION_OFFSET_LEFT: usize = std::mem::offset_of!(AssignmentExpression, left);
const ASSIGNMENT_EXPRESSION_OFFSET_RIGHT: usize = std::mem::offset_of!(AssignmentExpression, right);

impl<'a> AstNode<'a, AssignmentExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn operator(&self) -> AssignmentOperator {
        self.inner.operator
    }

    #[inline]
    pub fn left(&self) -> &AstNode<'a, AssignmentTarget<'a>> {
        let following_node = Some(FollowingNode::Expression({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(ASSIGNMENT_EXPRESSION_OFFSET_RIGHT) as *const Expression<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentExpression(transmute_self(self))),
            following_node,
        })
    }
}

impl<'a> AstNode<'a, AssignmentTarget<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.allocator.alloc(AstNodes::AssignmentTarget(transmute_self(self)));
        let node = match self.inner {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                AstNodes::SimpleAssignmentTarget(self.allocator.alloc(AstNode {
                    inner: it.to_simple_assignment_target(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                AstNodes::AssignmentTargetPattern(self.allocator.alloc(AstNode {
                    inner: it.to_assignment_target_pattern(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
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
        let parent = self.allocator.alloc(AstNodes::SimpleAssignmentTarget(transmute_self(self)));
        let node = match self.inner {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            SimpleAssignmentTarget::TSAsExpression(s) => {
                AstNodes::TSAsExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(s) => {
                AstNodes::TSSatisfiesExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            SimpleAssignmentTarget::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            SimpleAssignmentTarget::TSTypeAssertion(s) => {
                AstNodes::TSTypeAssertion(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                AstNodes::MemberExpression(self.allocator.alloc(AstNode {
                    inner: it.to_member_expression(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
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
        let parent = self.allocator.alloc(AstNodes::AssignmentTargetPattern(transmute_self(self)));
        let node = match self.inner {
            AssignmentTargetPattern::ArrayAssignmentTarget(s) => {
                AstNodes::ArrayAssignmentTarget(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(s) => {
                AstNodes::ObjectAssignmentTarget(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const ARRAY_ASSIGNMENT_TARGET_OFFSET_ELEMENTS: usize =
    std::mem::offset_of!(ArrayAssignmentTarget, elements);
const ARRAY_ASSIGNMENT_TARGET_OFFSET_REST: usize =
    std::mem::offset_of!(ArrayAssignmentTarget, rest);

impl<'a> AstNode<'a, ArrayAssignmentTarget<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn elements(&self) -> &AstNode<'a, Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(ARRAY_ASSIGNMENT_TARGET_OFFSET_REST)
                    as *const Option<AssignmentTargetRest<'a>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::AssignmentTargetRest(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrayAssignmentTarget(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, AssignmentTargetRest<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ArrayAssignmentTarget(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const OBJECT_ASSIGNMENT_TARGET_OFFSET_PROPERTIES: usize =
    std::mem::offset_of!(ObjectAssignmentTarget, properties);
const OBJECT_ASSIGNMENT_TARGET_OFFSET_REST: usize =
    std::mem::offset_of!(ObjectAssignmentTarget, rest);

impl<'a> AstNode<'a, ObjectAssignmentTarget<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn properties(&self) -> &AstNode<'a, Vec<'a, AssignmentTargetProperty<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(OBJECT_ASSIGNMENT_TARGET_OFFSET_REST)
                    as *const Option<AssignmentTargetRest<'a>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::AssignmentTargetRest(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectAssignmentTarget(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, AssignmentTargetRest<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent:
                    self.allocator.alloc(AstNodes::ObjectAssignmentTarget(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const ASSIGNMENT_TARGET_REST_OFFSET_TARGET: usize =
    std::mem::offset_of!(AssignmentTargetRest, target);

impl<'a> AstNode<'a, AssignmentTargetRest<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn target(&self) -> &AstNode<'a, AssignmentTarget<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.target,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
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
                    following_node: self.following_node.clone(),
                }))
            }
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                AstNodes::AssignmentTarget(self.allocator.alloc(AstNode {
                    inner: it.to_assignment_target(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
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

const ASSIGNMENT_TARGET_WITH_DEFAULT_OFFSET_BINDING: usize =
    std::mem::offset_of!(AssignmentTargetWithDefault, binding);
const ASSIGNMENT_TARGET_WITH_DEFAULT_OFFSET_INIT: usize =
    std::mem::offset_of!(AssignmentTargetWithDefault, init);

impl<'a> AstNode<'a, AssignmentTargetWithDefault<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn binding(&self) -> &AstNode<'a, AssignmentTarget<'a>> {
        let following_node = Some(FollowingNode::Expression({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(ASSIGNMENT_TARGET_WITH_DEFAULT_OFFSET_INIT) as *const Expression<'a>)
            }
        }));
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
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.init,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::AssignmentTargetWithDefault(transmute_self(self))),
            following_node,
        })
    }
}

impl<'a> AstNode<'a, AssignmentTargetProperty<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
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

const ASSIGNMENT_TARGET_PROPERTY_IDENTIFIER_OFFSET_BINDING: usize =
    std::mem::offset_of!(AssignmentTargetPropertyIdentifier, binding);
const ASSIGNMENT_TARGET_PROPERTY_IDENTIFIER_OFFSET_INIT: usize =
    std::mem::offset_of!(AssignmentTargetPropertyIdentifier, init);

impl<'a> AstNode<'a, AssignmentTargetPropertyIdentifier<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn binding(&self) -> &AstNode<'a, IdentifierReference<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(ASSIGNMENT_TARGET_PROPERTY_IDENTIFIER_OFFSET_INIT)
                    as *const Option<Expression<'a>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::Expression(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn init(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.init.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.parent,
                following_node,
            }))
            .as_ref()
    }
}

const ASSIGNMENT_TARGET_PROPERTY_PROPERTY_OFFSET_NAME: usize =
    std::mem::offset_of!(AssignmentTargetPropertyProperty, name);
const ASSIGNMENT_TARGET_PROPERTY_PROPERTY_OFFSET_BINDING: usize =
    std::mem::offset_of!(AssignmentTargetPropertyProperty, binding);

impl<'a> AstNode<'a, AssignmentTargetPropertyProperty<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn name(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = Some(FollowingNode::AssignmentTargetMaybeDefault({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(ASSIGNMENT_TARGET_PROPERTY_PROPERTY_OFFSET_BINDING)
                    as *const AssignmentTargetMaybeDefault<'a>)
            }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn binding(&self) -> &AstNode<'a, AssignmentTargetMaybeDefault<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.binding,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn computed(&self) -> bool {
        self.inner.computed
    }
}

const SEQUENCE_EXPRESSION_OFFSET_EXPRESSIONS: usize =
    std::mem::offset_of!(SequenceExpression, expressions);

impl<'a> AstNode<'a, SequenceExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expressions(&self) -> &AstNode<'a, Vec<'a, Expression<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.expressions,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SequenceExpression(transmute_self(self))),
            following_node,
        })
    }
}

impl<'a> AstNode<'a, Super> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

const AWAIT_EXPRESSION_OFFSET_ARGUMENT: usize = std::mem::offset_of!(AwaitExpression, argument);

impl<'a> AstNode<'a, AwaitExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AwaitExpression(transmute_self(self))),
            following_node,
        })
    }
}

const CHAIN_EXPRESSION_OFFSET_EXPRESSION: usize = std::mem::offset_of!(ChainExpression, expression);

impl<'a> AstNode<'a, ChainExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, ChainElement<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ChainExpression(transmute_self(self))),
            following_node,
        })
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
                    following_node: self.following_node.clone(),
                }))
            }
            ChainElement::TSNonNullExpression(s) => {
                AstNodes::TSNonNullExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            it @ match_member_expression!(ChainElement) => {
                AstNodes::MemberExpression(self.allocator.alloc(AstNode {
                    inner: it.to_member_expression(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
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

const PARENTHESIZED_EXPRESSION_OFFSET_EXPRESSION: usize =
    std::mem::offset_of!(ParenthesizedExpression, expression);

impl<'a> AstNode<'a, ParenthesizedExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ParenthesizedExpression(transmute_self(self))),
            following_node,
        })
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
                    following_node: self.following_node.clone(),
                }))
            }
            Statement::BreakStatement(s) => {
                AstNodes::BreakStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Statement::ContinueStatement(s) => {
                AstNodes::ContinueStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Statement::DebuggerStatement(s) => {
                AstNodes::DebuggerStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Statement::DoWhileStatement(s) => {
                AstNodes::DoWhileStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Statement::EmptyStatement(s) => {
                AstNodes::EmptyStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Statement::ExpressionStatement(s) => {
                AstNodes::ExpressionStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Statement::ForInStatement(s) => {
                AstNodes::ForInStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Statement::ForOfStatement(s) => {
                AstNodes::ForOfStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Statement::ForStatement(s) => AstNodes::ForStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            Statement::IfStatement(s) => AstNodes::IfStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            Statement::LabeledStatement(s) => {
                AstNodes::LabeledStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Statement::ReturnStatement(s) => {
                AstNodes::ReturnStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Statement::SwitchStatement(s) => {
                AstNodes::SwitchStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Statement::ThrowStatement(s) => {
                AstNodes::ThrowStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Statement::TryStatement(s) => AstNodes::TryStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            Statement::WhileStatement(s) => {
                AstNodes::WhileStatement(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Statement::WithStatement(s) => AstNodes::WithStatement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            it @ match_declaration!(Statement) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_declaration(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node.clone(),
                    })
                    .as_ast_nodes();
            }
            it @ match_module_declaration!(Statement) => {
                AstNodes::ModuleDeclaration(self.allocator.alloc(AstNode {
                    inner: it.to_module_declaration(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
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

const DIRECTIVE_OFFSET_EXPRESSION: usize = std::mem::offset_of!(Directive, expression);

impl<'a> AstNode<'a, Directive<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, StringLiteral<'a>> {
        let following_node = self.following_node.clone();
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
}

impl<'a> AstNode<'a, Hashbang<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }
}

const BLOCK_STATEMENT_OFFSET_BODY: usize = std::mem::offset_of!(BlockStatement, body);

impl<'a> AstNode<'a, BlockStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BlockStatement(transmute_self(self))),
            following_node,
        })
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
                    following_node: self.following_node.clone(),
                }))
            }
            Declaration::FunctionDeclaration(s) => {
                AstNodes::Function(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Declaration::ClassDeclaration(s) => AstNodes::Class(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            Declaration::TSTypeAliasDeclaration(s) => {
                AstNodes::TSTypeAliasDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Declaration::TSInterfaceDeclaration(s) => {
                AstNodes::TSInterfaceDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Declaration::TSEnumDeclaration(s) => {
                AstNodes::TSEnumDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Declaration::TSModuleDeclaration(s) => {
                AstNodes::TSModuleDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            Declaration::TSImportEqualsDeclaration(s) => {
                AstNodes::TSImportEqualsDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const VARIABLE_DECLARATION_OFFSET_DECLARATIONS: usize =
    std::mem::offset_of!(VariableDeclaration, declarations);

impl<'a> AstNode<'a, VariableDeclaration<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn kind(&self) -> VariableDeclarationKind {
        self.inner.kind
    }

    #[inline]
    pub fn declarations(&self) -> &AstNode<'a, Vec<'a, VariableDeclarator<'a>>> {
        let following_node = self.following_node.clone();
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
}

const VARIABLE_DECLARATOR_OFFSET_ID: usize = std::mem::offset_of!(VariableDeclarator, id);
const VARIABLE_DECLARATOR_OFFSET_INIT: usize = std::mem::offset_of!(VariableDeclarator, init);

impl<'a> AstNode<'a, VariableDeclarator<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn kind(&self) -> VariableDeclarationKind {
        self.inner.kind
    }

    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(VARIABLE_DECLARATOR_OFFSET_INIT) as *const Option<Expression<'a>>) }
        })
        .as_ref()
        .map(|t| FollowingNode::Expression(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::VariableDeclarator(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn init(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self.following_node.clone();
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
}

impl<'a> AstNode<'a, EmptyStatement> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

const EXPRESSION_STATEMENT_OFFSET_EXPRESSION: usize =
    std::mem::offset_of!(ExpressionStatement, expression);

impl<'a> AstNode<'a, ExpressionStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExpressionStatement(transmute_self(self))),
            following_node,
        })
    }
}

const IF_STATEMENT_OFFSET_TEST: usize = std::mem::offset_of!(IfStatement, test);
const IF_STATEMENT_OFFSET_CONSEQUENT: usize = std::mem::offset_of!(IfStatement, consequent);
const IF_STATEMENT_OFFSET_ALTERNATE: usize = std::mem::offset_of!(IfStatement, alternate);

impl<'a> AstNode<'a, IfStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn test(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::Statement({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(IF_STATEMENT_OFFSET_CONSEQUENT) as *const Statement<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::IfStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn consequent(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(IF_STATEMENT_OFFSET_ALTERNATE) as *const Option<Statement<'a>>) }
        })
        .as_ref()
        .map(|t| FollowingNode::Statement(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::IfStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn alternate(&self) -> Option<&AstNode<'a, Statement<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.alternate.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::IfStatement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const DO_WHILE_STATEMENT_OFFSET_BODY: usize = std::mem::offset_of!(DoWhileStatement, body);
const DO_WHILE_STATEMENT_OFFSET_TEST: usize = std::mem::offset_of!(DoWhileStatement, test);

impl<'a> AstNode<'a, DoWhileStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = Some(FollowingNode::Expression({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(DO_WHILE_STATEMENT_OFFSET_TEST) as *const Expression<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::DoWhileStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn test(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::DoWhileStatement(transmute_self(self))),
            following_node,
        })
    }
}

const WHILE_STATEMENT_OFFSET_TEST: usize = std::mem::offset_of!(WhileStatement, test);
const WHILE_STATEMENT_OFFSET_BODY: usize = std::mem::offset_of!(WhileStatement, body);

impl<'a> AstNode<'a, WhileStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn test(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::Statement({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(WHILE_STATEMENT_OFFSET_BODY) as *const Statement<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.test,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WhileStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WhileStatement(transmute_self(self))),
            following_node,
        })
    }
}

const FOR_STATEMENT_OFFSET_INIT: usize = std::mem::offset_of!(ForStatement, init);
const FOR_STATEMENT_OFFSET_TEST: usize = std::mem::offset_of!(ForStatement, test);
const FOR_STATEMENT_OFFSET_UPDATE: usize = std::mem::offset_of!(ForStatement, update);
const FOR_STATEMENT_OFFSET_BODY: usize = std::mem::offset_of!(ForStatement, body);

impl<'a> AstNode<'a, ForStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn init(&self) -> Option<&AstNode<'a, ForStatementInit<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(FOR_STATEMENT_OFFSET_TEST) as *const Option<Expression<'a>>) }
        })
        .as_ref()
        .map(|t| FollowingNode::Expression(t));
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
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(FOR_STATEMENT_OFFSET_UPDATE) as *const Option<Expression<'a>>) }
        })
        .as_ref()
        .map(|t| FollowingNode::Expression(t));
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
        let following_node = Some(FollowingNode::Statement({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(FOR_STATEMENT_OFFSET_BODY) as *const Statement<'a>) }
        }));
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
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForStatement(transmute_self(self))),
            following_node,
        })
    }
}

impl<'a> AstNode<'a, ForStatementInit<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.allocator.alloc(AstNodes::ForStatementInit(transmute_self(self)));
        let node = match self.inner {
            ForStatementInit::VariableDeclaration(s) => {
                AstNodes::VariableDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            it @ match_expression!(ForStatementInit) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node.clone(),
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

const FOR_IN_STATEMENT_OFFSET_LEFT: usize = std::mem::offset_of!(ForInStatement, left);
const FOR_IN_STATEMENT_OFFSET_RIGHT: usize = std::mem::offset_of!(ForInStatement, right);
const FOR_IN_STATEMENT_OFFSET_BODY: usize = std::mem::offset_of!(ForInStatement, body);

impl<'a> AstNode<'a, ForInStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn left(&self) -> &AstNode<'a, ForStatementLeft<'a>> {
        let following_node = Some(FollowingNode::Expression({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(FOR_IN_STATEMENT_OFFSET_RIGHT) as *const Expression<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForInStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::Statement({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(FOR_IN_STATEMENT_OFFSET_BODY) as *const Statement<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForInStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForInStatement(transmute_self(self))),
            following_node,
        })
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
                    following_node: self.following_node.clone(),
                }))
            }
            it @ match_assignment_target!(ForStatementLeft) => {
                AstNodes::AssignmentTarget(self.allocator.alloc(AstNode {
                    inner: it.to_assignment_target(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
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

const FOR_OF_STATEMENT_OFFSET_LEFT: usize = std::mem::offset_of!(ForOfStatement, left);
const FOR_OF_STATEMENT_OFFSET_RIGHT: usize = std::mem::offset_of!(ForOfStatement, right);
const FOR_OF_STATEMENT_OFFSET_BODY: usize = std::mem::offset_of!(ForOfStatement, body);

impl<'a> AstNode<'a, ForOfStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn r#await(&self) -> bool {
        self.inner.r#await
    }

    #[inline]
    pub fn left(&self) -> &AstNode<'a, ForStatementLeft<'a>> {
        let following_node = Some(FollowingNode::Expression({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(FOR_OF_STATEMENT_OFFSET_RIGHT) as *const Expression<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForOfStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::Statement({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(FOR_OF_STATEMENT_OFFSET_BODY) as *const Statement<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForOfStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ForOfStatement(transmute_self(self))),
            following_node,
        })
    }
}

const CONTINUE_STATEMENT_OFFSET_LABEL: usize = std::mem::offset_of!(ContinueStatement, label);

impl<'a> AstNode<'a, ContinueStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn label(&self) -> Option<&AstNode<'a, LabelIdentifier<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.label.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ContinueStatement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const BREAK_STATEMENT_OFFSET_LABEL: usize = std::mem::offset_of!(BreakStatement, label);

impl<'a> AstNode<'a, BreakStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn label(&self) -> Option<&AstNode<'a, LabelIdentifier<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.label.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::BreakStatement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const RETURN_STATEMENT_OFFSET_ARGUMENT: usize = std::mem::offset_of!(ReturnStatement, argument);

impl<'a> AstNode<'a, ReturnStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn argument(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.argument.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ReturnStatement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const WITH_STATEMENT_OFFSET_OBJECT: usize = std::mem::offset_of!(WithStatement, object);
const WITH_STATEMENT_OFFSET_BODY: usize = std::mem::offset_of!(WithStatement, body);

impl<'a> AstNode<'a, WithStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn object(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::Statement({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(WITH_STATEMENT_OFFSET_BODY) as *const Statement<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WithStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::WithStatement(transmute_self(self))),
            following_node,
        })
    }
}

const SWITCH_STATEMENT_OFFSET_DISCRIMINANT: usize =
    std::mem::offset_of!(SwitchStatement, discriminant);
const SWITCH_STATEMENT_OFFSET_CASES: usize = std::mem::offset_of!(SwitchStatement, cases);

impl<'a> AstNode<'a, SwitchStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn discriminant(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(SWITCH_STATEMENT_OFFSET_CASES) as *const Vec<'a, SwitchCase<'a>>) }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::SwitchCase(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.discriminant,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SwitchStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn cases(&self) -> &AstNode<'a, Vec<'a, SwitchCase<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.cases,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SwitchStatement(transmute_self(self))),
            following_node,
        })
    }
}

const SWITCH_CASE_OFFSET_TEST: usize = std::mem::offset_of!(SwitchCase, test);
const SWITCH_CASE_OFFSET_CONSEQUENT: usize = std::mem::offset_of!(SwitchCase, consequent);

impl<'a> AstNode<'a, SwitchCase<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn test(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(SWITCH_CASE_OFFSET_CONSEQUENT) as *const Vec<'a, Statement<'a>>) }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::Statement(t));
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
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.consequent,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::SwitchCase(transmute_self(self))),
            following_node,
        })
    }
}

const LABELED_STATEMENT_OFFSET_LABEL: usize = std::mem::offset_of!(LabeledStatement, label);
const LABELED_STATEMENT_OFFSET_BODY: usize = std::mem::offset_of!(LabeledStatement, body);

impl<'a> AstNode<'a, LabeledStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn label(&self) -> &AstNode<'a, LabelIdentifier<'a>> {
        let following_node = Some(FollowingNode::Statement({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(LABELED_STATEMENT_OFFSET_BODY) as *const Statement<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.label,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LabeledStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Statement<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::LabeledStatement(transmute_self(self))),
            following_node,
        })
    }
}

const THROW_STATEMENT_OFFSET_ARGUMENT: usize = std::mem::offset_of!(ThrowStatement, argument);

impl<'a> AstNode<'a, ThrowStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ThrowStatement(transmute_self(self))),
            following_node,
        })
    }
}

const TRY_STATEMENT_OFFSET_BLOCK: usize = std::mem::offset_of!(TryStatement, block);
const TRY_STATEMENT_OFFSET_HANDLER: usize = std::mem::offset_of!(TryStatement, handler);
const TRY_STATEMENT_OFFSET_FINALIZER: usize = std::mem::offset_of!(TryStatement, finalizer);

impl<'a> AstNode<'a, TryStatement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn block(&self) -> &AstNode<'a, BlockStatement<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TRY_STATEMENT_OFFSET_HANDLER) as *const Option<Box<'a, CatchClause<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::CatchClause(t));
        self.allocator.alloc(AstNode {
            inner: self.inner.block.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TryStatement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn handler(&self) -> Option<&AstNode<'a, CatchClause<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TRY_STATEMENT_OFFSET_FINALIZER)
                    as *const Option<Box<'a, BlockStatement<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::BlockStatement(t));
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
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.finalizer.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TryStatement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const CATCH_CLAUSE_OFFSET_PARAM: usize = std::mem::offset_of!(CatchClause, param);
const CATCH_CLAUSE_OFFSET_BODY: usize = std::mem::offset_of!(CatchClause, body);

impl<'a> AstNode<'a, CatchClause<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn param(&self) -> Option<&AstNode<'a, CatchParameter<'a>>> {
        let following_node = Some(FollowingNode::BlockStatement(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(CATCH_CLAUSE_OFFSET_BODY) as *const Box<'a, BlockStatement<'a>>)
                }
            }
            .as_ref(),
        ));
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
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: self.inner.body.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::CatchClause(transmute_self(self))),
            following_node,
        })
    }
}

const CATCH_PARAMETER_OFFSET_PATTERN: usize = std::mem::offset_of!(CatchParameter, pattern);

impl<'a> AstNode<'a, CatchParameter<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn pattern(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.pattern,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::CatchParameter(transmute_self(self))),
            following_node,
        })
    }
}

impl<'a> AstNode<'a, DebuggerStatement> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

const BINDING_PATTERN_OFFSET_KIND: usize = std::mem::offset_of!(BindingPattern, kind);
const BINDING_PATTERN_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(BindingPattern, type_annotation);

impl<'a> AstNode<'a, BindingPattern<'a>> {
    #[inline]
    pub fn kind(&self) -> &AstNode<'a, BindingPatternKind<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(BINDING_PATTERN_OFFSET_TYPEANNOTATION)
                    as *const Option<Box<'a, TSTypeAnnotation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeAnnotation(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.kind,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = self.following_node.clone();
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
                    following_node: self.following_node.clone(),
                }))
            }
            BindingPatternKind::ObjectPattern(s) => {
                AstNodes::ObjectPattern(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            BindingPatternKind::ArrayPattern(s) => {
                AstNodes::ArrayPattern(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            BindingPatternKind::AssignmentPattern(s) => {
                AstNodes::AssignmentPattern(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const ASSIGNMENT_PATTERN_OFFSET_LEFT: usize = std::mem::offset_of!(AssignmentPattern, left);
const ASSIGNMENT_PATTERN_OFFSET_RIGHT: usize = std::mem::offset_of!(AssignmentPattern, right);

impl<'a> AstNode<'a, AssignmentPattern<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn left(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_node = Some(FollowingNode::Expression({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(ASSIGNMENT_PATTERN_OFFSET_RIGHT) as *const Expression<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentPattern(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::AssignmentPattern(transmute_self(self))),
            following_node,
        })
    }
}

const OBJECT_PATTERN_OFFSET_PROPERTIES: usize = std::mem::offset_of!(ObjectPattern, properties);
const OBJECT_PATTERN_OFFSET_REST: usize = std::mem::offset_of!(ObjectPattern, rest);

impl<'a> AstNode<'a, ObjectPattern<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn properties(&self) -> &AstNode<'a, Vec<'a, BindingProperty<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(OBJECT_PATTERN_OFFSET_REST)
                    as *const Option<Box<'a, BindingRestElement<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::BindingRestElement(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.properties,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ObjectPattern(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, BindingRestElement<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ObjectPattern(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const BINDING_PROPERTY_OFFSET_KEY: usize = std::mem::offset_of!(BindingProperty, key);
const BINDING_PROPERTY_OFFSET_VALUE: usize = std::mem::offset_of!(BindingProperty, value);

impl<'a> AstNode<'a, BindingProperty<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = Some(FollowingNode::BindingPattern({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(BINDING_PROPERTY_OFFSET_VALUE) as *const BindingPattern<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn value(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: self.parent,
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
}

const ARRAY_PATTERN_OFFSET_ELEMENTS: usize = std::mem::offset_of!(ArrayPattern, elements);
const ARRAY_PATTERN_OFFSET_REST: usize = std::mem::offset_of!(ArrayPattern, rest);

impl<'a> AstNode<'a, ArrayPattern<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn elements(&self) -> &AstNode<'a, Vec<'a, Option<BindingPattern<'a>>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(ARRAY_PATTERN_OFFSET_REST)
                    as *const Option<Box<'a, BindingRestElement<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::BindingRestElement(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.elements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrayPattern(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, BindingRestElement<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::ArrayPattern(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const BINDING_REST_ELEMENT_OFFSET_ARGUMENT: usize =
    std::mem::offset_of!(BindingRestElement, argument);

impl<'a> AstNode<'a, BindingRestElement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn argument(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::BindingRestElement(transmute_self(self))),
            following_node,
        })
    }
}

const FUNCTION_OFFSET_ID: usize = std::mem::offset_of!(Function, id);
const FUNCTION_OFFSET_TYPEPARAMETERS: usize = std::mem::offset_of!(Function, type_parameters);
const FUNCTION_OFFSET_THISPARAM: usize = std::mem::offset_of!(Function, this_param);
const FUNCTION_OFFSET_PARAMS: usize = std::mem::offset_of!(Function, params);
const FUNCTION_OFFSET_RETURNTYPE: usize = std::mem::offset_of!(Function, return_type);
const FUNCTION_OFFSET_BODY: usize = std::mem::offset_of!(Function, body);

impl<'a> AstNode<'a, Function<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn r#type(&self) -> FunctionType {
        self.inner.r#type
    }

    #[inline]
    pub fn id(&self) -> Option<&AstNode<'a, BindingIdentifier<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(FUNCTION_OFFSET_TYPEPARAMETERS)
                    as *const Option<Box<'a, TSTypeParameterDeclaration<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterDeclaration(t));
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
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(FUNCTION_OFFSET_THISPARAM)
                    as *const Option<Box<'a, TSThisParameter<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSThisParameter(t));
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
        let following_node = Some(FollowingNode::FormalParameters(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(FUNCTION_OFFSET_PARAMS) as *const Box<'a, FormalParameters<'a>>)
                }
            }
            .as_ref(),
        ));
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
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(FUNCTION_OFFSET_RETURNTYPE)
                    as *const Option<Box<'a, TSTypeAnnotation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeAnnotation(t));
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Function(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn return_type(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(FUNCTION_OFFSET_BODY) as *const Option<Box<'a, FunctionBody<'a>>>) }
        })
        .as_ref()
        .map(|t| FollowingNode::FunctionBody(t));
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
        let following_node = self.following_node.clone();
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
}

const FORMAL_PARAMETERS_OFFSET_ITEMS: usize = std::mem::offset_of!(FormalParameters, items);
const FORMAL_PARAMETERS_OFFSET_REST: usize = std::mem::offset_of!(FormalParameters, rest);

impl<'a> AstNode<'a, FormalParameters<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn kind(&self) -> FormalParameterKind {
        self.inner.kind
    }

    #[inline]
    pub fn items(&self) -> &AstNode<'a, Vec<'a, FormalParameter<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(FORMAL_PARAMETERS_OFFSET_REST)
                    as *const Option<Box<'a, BindingRestElement<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::BindingRestElement(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.items,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FormalParameters(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn rest(&self) -> Option<&AstNode<'a, BindingRestElement<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.rest.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::FormalParameters(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const FORMAL_PARAMETER_OFFSET_DECORATORS: usize = std::mem::offset_of!(FormalParameter, decorators);
const FORMAL_PARAMETER_OFFSET_PATTERN: usize = std::mem::offset_of!(FormalParameter, pattern);

impl<'a> AstNode<'a, FormalParameter<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_node = Some(FollowingNode::BindingPattern({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(FORMAL_PARAMETER_OFFSET_PATTERN) as *const BindingPattern<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FormalParameter(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn pattern(&self) -> &AstNode<'a, BindingPattern<'a>> {
        let following_node = self.following_node.clone();
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
}

const FUNCTION_BODY_OFFSET_DIRECTIVES: usize = std::mem::offset_of!(FunctionBody, directives);
const FUNCTION_BODY_OFFSET_STATEMENTS: usize = std::mem::offset_of!(FunctionBody, statements);

impl<'a> AstNode<'a, FunctionBody<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn directives(&self) -> &AstNode<'a, Vec<'a, Directive<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(FUNCTION_BODY_OFFSET_STATEMENTS) as *const Vec<'a, Statement<'a>>) }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::Statement(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FunctionBody(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn statements(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.statements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::FunctionBody(transmute_self(self))),
            following_node,
        })
    }
}

const ARROW_FUNCTION_EXPRESSION_OFFSET_TYPEPARAMETERS: usize =
    std::mem::offset_of!(ArrowFunctionExpression, type_parameters);
const ARROW_FUNCTION_EXPRESSION_OFFSET_PARAMS: usize =
    std::mem::offset_of!(ArrowFunctionExpression, params);
const ARROW_FUNCTION_EXPRESSION_OFFSET_RETURNTYPE: usize =
    std::mem::offset_of!(ArrowFunctionExpression, return_type);
const ARROW_FUNCTION_EXPRESSION_OFFSET_BODY: usize =
    std::mem::offset_of!(ArrowFunctionExpression, body);

impl<'a> AstNode<'a, ArrowFunctionExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
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
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = Some(FollowingNode::FormalParameters(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(ARROW_FUNCTION_EXPRESSION_OFFSET_PARAMS)
                        as *const Box<'a, FormalParameters<'a>>)
                }
            }
            .as_ref(),
        ));
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
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(ARROW_FUNCTION_EXPRESSION_OFFSET_RETURNTYPE)
                    as *const Option<Box<'a, TSTypeAnnotation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeAnnotation(t));
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ArrowFunctionExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn return_type(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = Some(FollowingNode::FunctionBody(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(ARROW_FUNCTION_EXPRESSION_OFFSET_BODY)
                        as *const Box<'a, FunctionBody<'a>>)
                }
            }
            .as_ref(),
        ));
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
        let following_node = self.following_node.clone();
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
}

const YIELD_EXPRESSION_OFFSET_ARGUMENT: usize = std::mem::offset_of!(YieldExpression, argument);

impl<'a> AstNode<'a, YieldExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn delegate(&self) -> bool {
        self.inner.delegate
    }

    #[inline]
    pub fn argument(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.argument.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::YieldExpression(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const CLASS_OFFSET_DECORATORS: usize = std::mem::offset_of!(Class, decorators);
const CLASS_OFFSET_ID: usize = std::mem::offset_of!(Class, id);
const CLASS_OFFSET_TYPEPARAMETERS: usize = std::mem::offset_of!(Class, type_parameters);
const CLASS_OFFSET_SUPERCLASS: usize = std::mem::offset_of!(Class, super_class);
const CLASS_OFFSET_SUPERTYPEARGUMENTS: usize = std::mem::offset_of!(Class, super_type_arguments);
const CLASS_OFFSET_IMPLEMENTS: usize = std::mem::offset_of!(Class, implements);
const CLASS_OFFSET_BODY: usize = std::mem::offset_of!(Class, body);

impl<'a> AstNode<'a, Class<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn r#type(&self) -> ClassType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(CLASS_OFFSET_ID) as *const Option<BindingIdentifier<'a>>) }
        })
        .as_ref()
        .map(|t| FollowingNode::BindingIdentifier(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn id(&self) -> Option<&AstNode<'a, BindingIdentifier<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(CLASS_OFFSET_TYPEPARAMETERS)
                    as *const Option<Box<'a, TSTypeParameterDeclaration<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterDeclaration(t));
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
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(CLASS_OFFSET_SUPERCLASS) as *const Option<Expression<'a>>) }
        })
        .as_ref()
        .map(|t| FollowingNode::Expression(t));
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
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(CLASS_OFFSET_SUPERTYPEARGUMENTS)
                    as *const Option<Box<'a, TSTypeParameterInstantiation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterInstantiation(t));
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
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(CLASS_OFFSET_IMPLEMENTS) as *const Vec<'a, TSClassImplements<'a>>) }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::TSClassImplements(t));
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
        let following_node = Some(FollowingNode::ClassBody(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe { &*(ptr.add(CLASS_OFFSET_BODY) as *const Box<'a, ClassBody<'a>>) }
            }
            .as_ref(),
        ));
        self.allocator.alloc(AstNode {
            inner: &self.inner.implements,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Class(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, ClassBody<'a>> {
        let following_node = self.following_node.clone();
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
}

const CLASS_BODY_OFFSET_BODY: usize = std::mem::offset_of!(ClassBody, body);

impl<'a> AstNode<'a, ClassBody<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, ClassElement<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ClassBody(transmute_self(self))),
            following_node,
        })
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
                following_node: self.following_node.clone(),
            })),
            ClassElement::MethodDefinition(s) => {
                AstNodes::MethodDefinition(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            ClassElement::PropertyDefinition(s) => {
                AstNodes::PropertyDefinition(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            ClassElement::AccessorProperty(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            ClassElement::TSIndexSignature(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
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

const METHOD_DEFINITION_OFFSET_DECORATORS: usize =
    std::mem::offset_of!(MethodDefinition, decorators);
const METHOD_DEFINITION_OFFSET_KEY: usize = std::mem::offset_of!(MethodDefinition, key);
const METHOD_DEFINITION_OFFSET_VALUE: usize = std::mem::offset_of!(MethodDefinition, value);

impl<'a> AstNode<'a, MethodDefinition<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn r#type(&self) -> MethodDefinitionType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_node = Some(FollowingNode::PropertyKey({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(METHOD_DEFINITION_OFFSET_KEY) as *const PropertyKey<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MethodDefinition(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = Some(FollowingNode::Function(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(METHOD_DEFINITION_OFFSET_VALUE) as *const Box<'a, Function<'a>>)
                }
            }
            .as_ref(),
        ));
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::MethodDefinition(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn value(&self) -> &AstNode<'a, Function<'a>> {
        let following_node = self.following_node.clone();
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
}

const PROPERTY_DEFINITION_OFFSET_DECORATORS: usize =
    std::mem::offset_of!(PropertyDefinition, decorators);
const PROPERTY_DEFINITION_OFFSET_KEY: usize = std::mem::offset_of!(PropertyDefinition, key);
const PROPERTY_DEFINITION_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(PropertyDefinition, type_annotation);
const PROPERTY_DEFINITION_OFFSET_VALUE: usize = std::mem::offset_of!(PropertyDefinition, value);

impl<'a> AstNode<'a, PropertyDefinition<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn r#type(&self) -> PropertyDefinitionType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_node = Some(FollowingNode::PropertyKey({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(PROPERTY_DEFINITION_OFFSET_KEY) as *const PropertyKey<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PropertyDefinition(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(PROPERTY_DEFINITION_OFFSET_TYPEANNOTATION)
                    as *const Option<Box<'a, TSTypeAnnotation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeAnnotation(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::PropertyDefinition(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(PROPERTY_DEFINITION_OFFSET_VALUE) as *const Option<Expression<'a>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::Expression(t));
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
        let following_node = self.following_node.clone();
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
}

impl<'a> AstNode<'a, PrivateIdentifier<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }
}

const STATIC_BLOCK_OFFSET_BODY: usize = std::mem::offset_of!(StaticBlock, body);

impl<'a> AstNode<'a, StaticBlock<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::StaticBlock(transmute_self(self))),
            following_node,
        })
    }
}

impl<'a> AstNode<'a, ModuleDeclaration<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.allocator.alloc(AstNodes::ModuleDeclaration(transmute_self(self)));
        let node = match self.inner {
            ModuleDeclaration::ImportDeclaration(s) => {
                AstNodes::ImportDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            ModuleDeclaration::ExportAllDeclaration(s) => {
                AstNodes::ExportAllDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            ModuleDeclaration::ExportDefaultDeclaration(s) => {
                AstNodes::ExportDefaultDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            ModuleDeclaration::ExportNamedDeclaration(s) => {
                AstNodes::ExportNamedDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            ModuleDeclaration::TSExportAssignment(s) => {
                AstNodes::TSExportAssignment(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
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

const ACCESSOR_PROPERTY_OFFSET_DECORATORS: usize =
    std::mem::offset_of!(AccessorProperty, decorators);
const ACCESSOR_PROPERTY_OFFSET_KEY: usize = std::mem::offset_of!(AccessorProperty, key);
const ACCESSOR_PROPERTY_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(AccessorProperty, type_annotation);
const ACCESSOR_PROPERTY_OFFSET_VALUE: usize = std::mem::offset_of!(AccessorProperty, value);

impl<'a> AstNode<'a, AccessorProperty<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn r#type(&self) -> AccessorPropertyType {
        self.inner.r#type
    }

    #[inline]
    pub fn decorators(&self) -> &AstNode<'a, Vec<'a, Decorator<'a>>> {
        let following_node = Some(FollowingNode::PropertyKey({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(ACCESSOR_PROPERTY_OFFSET_KEY) as *const PropertyKey<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.decorators,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(ACCESSOR_PROPERTY_OFFSET_TYPEANNOTATION)
                    as *const Option<Box<'a, TSTypeAnnotation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeAnnotation(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(ACCESSOR_PROPERTY_OFFSET_VALUE) as *const Option<Expression<'a>>) }
        })
        .as_ref()
        .map(|t| FollowingNode::Expression(t));
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
    pub fn value(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.parent,
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
}

const IMPORT_EXPRESSION_OFFSET_SOURCE: usize = std::mem::offset_of!(ImportExpression, source);
const IMPORT_EXPRESSION_OFFSET_OPTIONS: usize = std::mem::offset_of!(ImportExpression, options);

impl<'a> AstNode<'a, ImportExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn source(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(IMPORT_EXPRESSION_OFFSET_OPTIONS) as *const Option<Expression<'a>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::Expression(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn options(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self.following_node.clone();
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
}

const IMPORT_DECLARATION_OFFSET_SPECIFIERS: usize =
    std::mem::offset_of!(ImportDeclaration, specifiers);
const IMPORT_DECLARATION_OFFSET_SOURCE: usize = std::mem::offset_of!(ImportDeclaration, source);
const IMPORT_DECLARATION_OFFSET_WITHCLAUSE: usize =
    std::mem::offset_of!(ImportDeclaration, with_clause);

impl<'a> AstNode<'a, ImportDeclaration<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn specifiers(&self) -> Option<&AstNode<'a, Vec<'a, ImportDeclarationSpecifier<'a>>>> {
        let following_node = Some(FollowingNode::StringLiteral({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(IMPORT_DECLARATION_OFFSET_SOURCE) as *const StringLiteral<'a>) }
        }));
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
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(IMPORT_DECLARATION_OFFSET_WITHCLAUSE)
                    as *const Option<Box<'a, WithClause<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::WithClause(t));
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
        let following_node = self.following_node.clone();
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
                    following_node: self.following_node.clone(),
                }))
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                AstNodes::ImportDefaultSpecifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                AstNodes::ImportNamespaceSpecifier(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const IMPORT_SPECIFIER_OFFSET_IMPORTED: usize = std::mem::offset_of!(ImportSpecifier, imported);
const IMPORT_SPECIFIER_OFFSET_LOCAL: usize = std::mem::offset_of!(ImportSpecifier, local);

impl<'a> AstNode<'a, ImportSpecifier<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn imported(&self) -> &AstNode<'a, ModuleExportName<'a>> {
        let following_node = Some(FollowingNode::BindingIdentifier({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(IMPORT_SPECIFIER_OFFSET_LOCAL) as *const BindingIdentifier<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.imported,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportSpecifier(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn local(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = self.following_node.clone();
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
}

const IMPORT_DEFAULT_SPECIFIER_OFFSET_LOCAL: usize =
    std::mem::offset_of!(ImportDefaultSpecifier, local);

impl<'a> AstNode<'a, ImportDefaultSpecifier<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn local(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportDefaultSpecifier(transmute_self(self))),
            following_node,
        })
    }
}

const IMPORT_NAMESPACE_SPECIFIER_OFFSET_LOCAL: usize =
    std::mem::offset_of!(ImportNamespaceSpecifier, local);

impl<'a> AstNode<'a, ImportNamespaceSpecifier<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn local(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ImportNamespaceSpecifier(transmute_self(self))),
            following_node,
        })
    }
}

const WITH_CLAUSE_OFFSET_ATTRIBUTESKEYWORD: usize =
    std::mem::offset_of!(WithClause, attributes_keyword);
const WITH_CLAUSE_OFFSET_WITHENTRIES: usize = std::mem::offset_of!(WithClause, with_entries);

impl<'a> AstNode<'a, WithClause<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn attributes_keyword(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(WITH_CLAUSE_OFFSET_WITHENTRIES) as *const Vec<'a, ImportAttribute<'a>>)
            }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::ImportAttribute(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.attributes_keyword,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn with_entries(&self) -> &AstNode<'a, Vec<'a, ImportAttribute<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.with_entries,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }
}

const IMPORT_ATTRIBUTE_OFFSET_KEY: usize = std::mem::offset_of!(ImportAttribute, key);
const IMPORT_ATTRIBUTE_OFFSET_VALUE: usize = std::mem::offset_of!(ImportAttribute, value);

impl<'a> AstNode<'a, ImportAttribute<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, ImportAttributeKey<'a>> {
        let following_node = Some(FollowingNode::StringLiteral({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(IMPORT_ATTRIBUTE_OFFSET_VALUE) as *const StringLiteral<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn value(&self) -> &AstNode<'a, StringLiteral<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.value,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
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
                    following_node: self.following_node.clone(),
                }))
            }
            ImportAttributeKey::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const EXPORT_NAMED_DECLARATION_OFFSET_DECLARATION: usize =
    std::mem::offset_of!(ExportNamedDeclaration, declaration);
const EXPORT_NAMED_DECLARATION_OFFSET_SPECIFIERS: usize =
    std::mem::offset_of!(ExportNamedDeclaration, specifiers);
const EXPORT_NAMED_DECLARATION_OFFSET_SOURCE: usize =
    std::mem::offset_of!(ExportNamedDeclaration, source);
const EXPORT_NAMED_DECLARATION_OFFSET_WITHCLAUSE: usize =
    std::mem::offset_of!(ExportNamedDeclaration, with_clause);

impl<'a> AstNode<'a, ExportNamedDeclaration<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn declaration(&self) -> Option<&AstNode<'a, Declaration<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(EXPORT_NAMED_DECLARATION_OFFSET_SPECIFIERS)
                    as *const Vec<'a, ExportSpecifier<'a>>)
            }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::ExportSpecifier(t));
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
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(EXPORT_NAMED_DECLARATION_OFFSET_SOURCE)
                    as *const Option<StringLiteral<'a>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::StringLiteral(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.specifiers,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportNamedDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn source(&self) -> Option<&AstNode<'a, StringLiteral<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(EXPORT_NAMED_DECLARATION_OFFSET_WITHCLAUSE)
                    as *const Option<Box<'a, WithClause<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::WithClause(t));
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
        let following_node = self.following_node.clone();
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
}

const EXPORT_DEFAULT_DECLARATION_OFFSET_EXPORTED: usize =
    std::mem::offset_of!(ExportDefaultDeclaration, exported);
const EXPORT_DEFAULT_DECLARATION_OFFSET_DECLARATION: usize =
    std::mem::offset_of!(ExportDefaultDeclaration, declaration);

impl<'a> AstNode<'a, ExportDefaultDeclaration<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn exported(&self) -> &AstNode<'a, ModuleExportName<'a>> {
        let following_node = Some(FollowingNode::ExportDefaultDeclarationKind({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(EXPORT_DEFAULT_DECLARATION_OFFSET_DECLARATION)
                    as *const ExportDefaultDeclarationKind<'a>)
            }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.exported,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportDefaultDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn declaration(&self) -> &AstNode<'a, ExportDefaultDeclarationKind<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.declaration,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportDefaultDeclaration(transmute_self(self))),
            following_node,
        })
    }
}

const EXPORT_ALL_DECLARATION_OFFSET_EXPORTED: usize =
    std::mem::offset_of!(ExportAllDeclaration, exported);
const EXPORT_ALL_DECLARATION_OFFSET_SOURCE: usize =
    std::mem::offset_of!(ExportAllDeclaration, source);
const EXPORT_ALL_DECLARATION_OFFSET_WITHCLAUSE: usize =
    std::mem::offset_of!(ExportAllDeclaration, with_clause);

impl<'a> AstNode<'a, ExportAllDeclaration<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn exported(&self) -> Option<&AstNode<'a, ModuleExportName<'a>>> {
        let following_node = Some(FollowingNode::StringLiteral({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(EXPORT_ALL_DECLARATION_OFFSET_SOURCE) as *const StringLiteral<'a>) }
        }));
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
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(EXPORT_ALL_DECLARATION_OFFSET_WITHCLAUSE)
                    as *const Option<Box<'a, WithClause<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::WithClause(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.source,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportAllDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn with_clause(&self) -> Option<&AstNode<'a, WithClause<'a>>> {
        let following_node = self.following_node.clone();
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
}

const EXPORT_SPECIFIER_OFFSET_LOCAL: usize = std::mem::offset_of!(ExportSpecifier, local);
const EXPORT_SPECIFIER_OFFSET_EXPORTED: usize = std::mem::offset_of!(ExportSpecifier, exported);

impl<'a> AstNode<'a, ExportSpecifier<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn local(&self) -> &AstNode<'a, ModuleExportName<'a>> {
        let following_node = Some(FollowingNode::ModuleExportName({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(EXPORT_SPECIFIER_OFFSET_EXPORTED) as *const ModuleExportName<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.local,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::ExportSpecifier(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn exported(&self) -> &AstNode<'a, ModuleExportName<'a>> {
        let following_node = self.following_node.clone();
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
                    following_node: self.following_node.clone(),
                }))
            }
            ExportDefaultDeclarationKind::ClassDeclaration(s) => {
                AstNodes::Class(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(s) => {
                AstNodes::TSInterfaceDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            it @ match_expression!(ExportDefaultDeclarationKind) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node.clone(),
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
                    following_node: self.following_node.clone(),
                }))
            }
            ModuleExportName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            ModuleExportName::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const V_8_INTRINSIC_EXPRESSION_OFFSET_NAME: usize =
    std::mem::offset_of!(V8IntrinsicExpression, name);
const V_8_INTRINSIC_EXPRESSION_OFFSET_ARGUMENTS: usize =
    std::mem::offset_of!(V8IntrinsicExpression, arguments);

impl<'a> AstNode<'a, V8IntrinsicExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn name(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(V_8_INTRINSIC_EXPRESSION_OFFSET_ARGUMENTS)
                    as *const Vec<'a, Argument<'a>>)
            }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::Argument(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::V8IntrinsicExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn arguments(&self) -> &AstNode<'a, Vec<'a, Argument<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.arguments,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::V8IntrinsicExpression(transmute_self(self))),
            following_node,
        })
    }
}

impl<'a> AstNode<'a, BooleanLiteral> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn value(&self) -> bool {
        self.inner.value
    }
}

impl<'a> AstNode<'a, NullLiteral> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, NumericLiteral<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

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
}

impl<'a> AstNode<'a, StringLiteral<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

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
}

impl<'a> AstNode<'a, BigIntLiteral<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

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
}

impl<'a> AstNode<'a, RegExpLiteral<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn regex(&self) -> &RegExp<'a> {
        &self.inner.regex
    }

    #[inline]
    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }
}

const JSX_ELEMENT_OFFSET_OPENINGELEMENT: usize = std::mem::offset_of!(JSXElement, opening_element);
const JSX_ELEMENT_OFFSET_CHILDREN: usize = std::mem::offset_of!(JSXElement, children);
const JSX_ELEMENT_OFFSET_CLOSINGELEMENT: usize = std::mem::offset_of!(JSXElement, closing_element);

impl<'a> AstNode<'a, JSXElement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn opening_element(&self) -> &AstNode<'a, JSXOpeningElement<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(JSX_ELEMENT_OFFSET_CHILDREN) as *const Vec<'a, JSXChild<'a>>) }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::JSXChild(t));
        self.allocator.alloc(AstNode {
            inner: self.inner.opening_element.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXElement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn children(&self) -> &AstNode<'a, Vec<'a, JSXChild<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(JSX_ELEMENT_OFFSET_CLOSINGELEMENT)
                    as *const Option<Box<'a, JSXClosingElement<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::JSXClosingElement(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.children,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXElement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn closing_element(&self) -> Option<&AstNode<'a, JSXClosingElement<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.closing_element.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::JSXElement(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const JSX_OPENING_ELEMENT_OFFSET_NAME: usize = std::mem::offset_of!(JSXOpeningElement, name);
const JSX_OPENING_ELEMENT_OFFSET_TYPEARGUMENTS: usize =
    std::mem::offset_of!(JSXOpeningElement, type_arguments);
const JSX_OPENING_ELEMENT_OFFSET_ATTRIBUTES: usize =
    std::mem::offset_of!(JSXOpeningElement, attributes);

impl<'a> AstNode<'a, JSXOpeningElement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn name(&self) -> &AstNode<'a, JSXElementName<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(JSX_OPENING_ELEMENT_OFFSET_TYPEARGUMENTS)
                    as *const Option<Box<'a, TSTypeParameterInstantiation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterInstantiation(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXOpeningElement(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(JSX_OPENING_ELEMENT_OFFSET_ATTRIBUTES)
                    as *const Vec<'a, JSXAttributeItem<'a>>)
            }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::JSXAttributeItem(t));
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
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.attributes,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXOpeningElement(transmute_self(self))),
            following_node,
        })
    }
}

const JSX_CLOSING_ELEMENT_OFFSET_NAME: usize = std::mem::offset_of!(JSXClosingElement, name);

impl<'a> AstNode<'a, JSXClosingElement<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn name(&self) -> &AstNode<'a, JSXElementName<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXClosingElement(transmute_self(self))),
            following_node,
        })
    }
}

const JSX_FRAGMENT_OFFSET_OPENINGFRAGMENT: usize =
    std::mem::offset_of!(JSXFragment, opening_fragment);
const JSX_FRAGMENT_OFFSET_CHILDREN: usize = std::mem::offset_of!(JSXFragment, children);
const JSX_FRAGMENT_OFFSET_CLOSINGFRAGMENT: usize =
    std::mem::offset_of!(JSXFragment, closing_fragment);

impl<'a> AstNode<'a, JSXFragment<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn opening_fragment(&self) -> &AstNode<'a, JSXOpeningFragment> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(JSX_FRAGMENT_OFFSET_CHILDREN) as *const Vec<'a, JSXChild<'a>>) }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::JSXChild(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.opening_fragment,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXFragment(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn children(&self) -> &AstNode<'a, Vec<'a, JSXChild<'a>>> {
        let following_node = Some(FollowingNode::JSXClosingFragment({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(JSX_FRAGMENT_OFFSET_CLOSINGFRAGMENT) as *const JSXClosingFragment) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.children,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXFragment(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn closing_fragment(&self) -> &AstNode<'a, JSXClosingFragment> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.closing_fragment,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXFragment(transmute_self(self))),
            following_node,
        })
    }
}

impl<'a> AstNode<'a, JSXOpeningFragment> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, JSXClosingFragment> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
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
                    following_node: self.following_node.clone(),
                }))
            }
            JSXElementName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            JSXElementName::NamespacedName(s) => {
                AstNodes::JSXNamespacedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            JSXElementName::MemberExpression(s) => {
                AstNodes::JSXMemberExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            JSXElementName::ThisExpression(s) => {
                AstNodes::ThisExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const JSX_NAMESPACED_NAME_OFFSET_NAMESPACE: usize =
    std::mem::offset_of!(JSXNamespacedName, namespace);
const JSX_NAMESPACED_NAME_OFFSET_NAME: usize = std::mem::offset_of!(JSXNamespacedName, name);

impl<'a> AstNode<'a, JSXNamespacedName<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn namespace(&self) -> &AstNode<'a, JSXIdentifier<'a>> {
        let following_node = Some(FollowingNode::JSXIdentifier({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(JSX_NAMESPACED_NAME_OFFSET_NAME) as *const JSXIdentifier<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.namespace,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXNamespacedName(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn name(&self) -> &AstNode<'a, JSXIdentifier<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXNamespacedName(transmute_self(self))),
            following_node,
        })
    }
}

const JSX_MEMBER_EXPRESSION_OFFSET_OBJECT: usize =
    std::mem::offset_of!(JSXMemberExpression, object);
const JSX_MEMBER_EXPRESSION_OFFSET_PROPERTY: usize =
    std::mem::offset_of!(JSXMemberExpression, property);

impl<'a> AstNode<'a, JSXMemberExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn object(&self) -> &AstNode<'a, JSXMemberExpressionObject<'a>> {
        let following_node = Some(FollowingNode::JSXIdentifier({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(JSX_MEMBER_EXPRESSION_OFFSET_PROPERTY) as *const JSXIdentifier<'a>)
            }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.object,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXMemberExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn property(&self) -> &AstNode<'a, JSXIdentifier<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.property,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXMemberExpression(transmute_self(self))),
            following_node,
        })
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
                    following_node: self.following_node.clone(),
                }))
            }
            JSXMemberExpressionObject::MemberExpression(s) => {
                AstNodes::JSXMemberExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            JSXMemberExpressionObject::ThisExpression(s) => {
                AstNodes::ThisExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const JSX_EXPRESSION_CONTAINER_OFFSET_EXPRESSION: usize =
    std::mem::offset_of!(JSXExpressionContainer, expression);

impl<'a> AstNode<'a, JSXExpressionContainer<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, JSXExpression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXExpressionContainer(transmute_self(self))),
            following_node,
        })
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
                    following_node: self.following_node.clone(),
                }))
            }
            it @ match_expression!(JSXExpression) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_expression(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node.clone(),
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
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
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
                    following_node: self.following_node.clone(),
                }))
            }
            JSXAttributeItem::SpreadAttribute(s) => {
                AstNodes::JSXSpreadAttribute(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const JSX_ATTRIBUTE_OFFSET_NAME: usize = std::mem::offset_of!(JSXAttribute, name);
const JSX_ATTRIBUTE_OFFSET_VALUE: usize = std::mem::offset_of!(JSXAttribute, value);

impl<'a> AstNode<'a, JSXAttribute<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn name(&self) -> &AstNode<'a, JSXAttributeName<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(JSX_ATTRIBUTE_OFFSET_VALUE) as *const Option<JSXAttributeValue<'a>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::JSXAttributeValue(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXAttribute(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn value(&self) -> Option<&AstNode<'a, JSXAttributeValue<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.value.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::JSXAttribute(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const JSX_SPREAD_ATTRIBUTE_OFFSET_ARGUMENT: usize =
    std::mem::offset_of!(JSXSpreadAttribute, argument);

impl<'a> AstNode<'a, JSXSpreadAttribute<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn argument(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXSpreadAttribute(transmute_self(self))),
            following_node,
        })
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
                    following_node: self.following_node.clone(),
                }))
            }
            JSXAttributeName::NamespacedName(s) => {
                AstNodes::JSXNamespacedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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
                    following_node: self.following_node.clone(),
                }))
            }
            JSXAttributeValue::ExpressionContainer(s) => {
                AstNodes::JSXExpressionContainer(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            JSXAttributeValue::Element(s) => AstNodes::JSXElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            JSXAttributeValue::Fragment(s) => {
                AstNodes::JSXFragment(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
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
                following_node: self.following_node.clone(),
            })),
            JSXChild::Element(s) => AstNodes::JSXElement(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            JSXChild::Fragment(s) => AstNodes::JSXFragment(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            JSXChild::ExpressionContainer(s) => {
                AstNodes::JSXExpressionContainer(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            JSXChild::Spread(s) => AstNodes::JSXSpreadChild(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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

const JSX_SPREAD_CHILD_OFFSET_EXPRESSION: usize = std::mem::offset_of!(JSXSpreadChild, expression);

impl<'a> AstNode<'a, JSXSpreadChild<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::JSXSpreadChild(transmute_self(self))),
            following_node,
        })
    }
}

impl<'a> AstNode<'a, JSXText<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn value(&self) -> Atom<'a> {
        self.inner.value
    }

    #[inline]
    pub fn raw(&self) -> Option<Atom<'a>> {
        self.inner.raw
    }
}

const TS_THIS_PARAMETER_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(TSThisParameter, type_annotation);

impl<'a> AstNode<'a, TSThisParameter<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn this_span(&self) -> Span {
        self.inner.this_span
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSThisParameter(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const TS_ENUM_DECLARATION_OFFSET_ID: usize = std::mem::offset_of!(TSEnumDeclaration, id);
const TS_ENUM_DECLARATION_OFFSET_BODY: usize = std::mem::offset_of!(TSEnumDeclaration, body);

impl<'a> AstNode<'a, TSEnumDeclaration<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = Some(FollowingNode::TSEnumBody({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(TS_ENUM_DECLARATION_OFFSET_BODY) as *const TSEnumBody<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, TSEnumBody<'a>> {
        let following_node = self.following_node.clone();
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
}

const TS_ENUM_BODY_OFFSET_MEMBERS: usize = std::mem::offset_of!(TSEnumBody, members);

impl<'a> AstNode<'a, TSEnumBody<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn members(&self) -> &AstNode<'a, Vec<'a, TSEnumMember<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.members,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumBody(transmute_self(self))),
            following_node,
        })
    }
}

const TS_ENUM_MEMBER_OFFSET_ID: usize = std::mem::offset_of!(TSEnumMember, id);
const TS_ENUM_MEMBER_OFFSET_INITIALIZER: usize = std::mem::offset_of!(TSEnumMember, initializer);

impl<'a> AstNode<'a, TSEnumMember<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn id(&self) -> &AstNode<'a, TSEnumMemberName<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_ENUM_MEMBER_OFFSET_INITIALIZER) as *const Option<Expression<'a>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::Expression(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSEnumMember(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn initializer(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.initializer.as_ref().map(|inner| AstNode {
                inner,
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSEnumMember(transmute_self(self))),
                following_node,
            }))
            .as_ref()
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
                    following_node: self.following_node.clone(),
                }))
            }
            TSEnumMemberName::String(s) => AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            TSEnumMemberName::ComputedString(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSEnumMemberName::ComputedTemplateString(s) => {
                AstNodes::TemplateLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const TS_TYPE_ANNOTATION_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(TSTypeAnnotation, type_annotation);

impl<'a> AstNode<'a, TSTypeAnnotation<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAnnotation(transmute_self(self))),
            following_node,
        })
    }
}

const TS_LITERAL_TYPE_OFFSET_LITERAL: usize = std::mem::offset_of!(TSLiteralType, literal);

impl<'a> AstNode<'a, TSLiteralType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn literal(&self) -> &AstNode<'a, TSLiteral<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.literal,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSLiteralType(transmute_self(self))),
            following_node,
        })
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
                    following_node: self.following_node.clone(),
                }))
            }
            TSLiteral::NumericLiteral(s) => {
                AstNodes::NumericLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSLiteral::BigIntLiteral(s) => AstNodes::BigIntLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            TSLiteral::StringLiteral(s) => AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            TSLiteral::TemplateLiteral(s) => {
                AstNodes::TemplateLiteral(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSLiteral::UnaryExpression(s) => {
                AstNodes::UnaryExpression(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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
                following_node: self.following_node.clone(),
            })),
            TSType::TSBigIntKeyword(s) => {
                AstNodes::TSBigIntKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSBooleanKeyword(s) => {
                AstNodes::TSBooleanKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSIntrinsicKeyword(s) => {
                AstNodes::TSIntrinsicKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSNeverKeyword(s) => AstNodes::TSNeverKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            TSType::TSNullKeyword(s) => AstNodes::TSNullKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            TSType::TSNumberKeyword(s) => {
                AstNodes::TSNumberKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSObjectKeyword(s) => {
                AstNodes::TSObjectKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSStringKeyword(s) => {
                AstNodes::TSStringKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSSymbolKeyword(s) => {
                AstNodes::TSSymbolKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSUndefinedKeyword(s) => {
                AstNodes::TSUndefinedKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSUnknownKeyword(s) => {
                AstNodes::TSUnknownKeyword(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSVoidKeyword(s) => AstNodes::TSVoidKeyword(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            TSType::TSArrayType(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSType::TSConditionalType(s) => {
                AstNodes::TSConditionalType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSConstructorType(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSType::TSFunctionType(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSType::TSImportType(s) => AstNodes::TSImportType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            TSType::TSIndexedAccessType(s) => {
                AstNodes::TSIndexedAccessType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSInferType(s) => AstNodes::TSInferType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            TSType::TSIntersectionType(s) => {
                AstNodes::TSIntersectionType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSLiteralType(s) => AstNodes::TSLiteralType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            TSType::TSMappedType(s) => AstNodes::TSMappedType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            TSType::TSNamedTupleMember(s) => {
                AstNodes::TSNamedTupleMember(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSTemplateLiteralType(s) => {
                AstNodes::TSTemplateLiteralType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSThisType(s) => AstNodes::TSThisType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            TSType::TSTupleType(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSType::TSTypeLiteral(s) => AstNodes::TSTypeLiteral(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            TSType::TSTypeOperatorType(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSType::TSTypePredicate(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSType::TSTypeQuery(s) => AstNodes::TSTypeQuery(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            TSType::TSTypeReference(s) => {
                AstNodes::TSTypeReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::TSUnionType(s) => AstNodes::TSUnionType(self.allocator.alloc(AstNode {
                inner: s.as_ref(),
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
            })),
            TSType::TSParenthesizedType(s) => {
                AstNodes::TSParenthesizedType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::JSDocNullableType(s) => {
                AstNodes::JSDocNullableType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::JSDocNonNullableType(s) => {
                AstNodes::JSDocNonNullableType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSType::JSDocUnknownType(s) => {
                AstNodes::JSDocUnknownType(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const TS_CONDITIONAL_TYPE_OFFSET_CHECKTYPE: usize =
    std::mem::offset_of!(TSConditionalType, check_type);
const TS_CONDITIONAL_TYPE_OFFSET_EXTENDSTYPE: usize =
    std::mem::offset_of!(TSConditionalType, extends_type);
const TS_CONDITIONAL_TYPE_OFFSET_TRUETYPE: usize =
    std::mem::offset_of!(TSConditionalType, true_type);
const TS_CONDITIONAL_TYPE_OFFSET_FALSETYPE: usize =
    std::mem::offset_of!(TSConditionalType, false_type);

impl<'a> AstNode<'a, TSConditionalType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn check_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = Some(FollowingNode::TSType({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(TS_CONDITIONAL_TYPE_OFFSET_EXTENDSTYPE) as *const TSType<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.check_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn extends_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = Some(FollowingNode::TSType({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(TS_CONDITIONAL_TYPE_OFFSET_TRUETYPE) as *const TSType<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.extends_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn true_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = Some(FollowingNode::TSType({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(TS_CONDITIONAL_TYPE_OFFSET_FALSETYPE) as *const TSType<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.true_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn false_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.false_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSConditionalType(transmute_self(self))),
            following_node,
        })
    }
}

const TS_UNION_TYPE_OFFSET_TYPES: usize = std::mem::offset_of!(TSUnionType, types);

impl<'a> AstNode<'a, TSUnionType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn types(&self) -> &AstNode<'a, Vec<'a, TSType<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSUnionType(transmute_self(self))),
            following_node,
        })
    }
}

const TS_INTERSECTION_TYPE_OFFSET_TYPES: usize = std::mem::offset_of!(TSIntersectionType, types);

impl<'a> AstNode<'a, TSIntersectionType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn types(&self) -> &AstNode<'a, Vec<'a, TSType<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIntersectionType(transmute_self(self))),
            following_node,
        })
    }
}

const TS_PARENTHESIZED_TYPE_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(TSParenthesizedType, type_annotation);

impl<'a> AstNode<'a, TSParenthesizedType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSParenthesizedType(transmute_self(self))),
            following_node,
        })
    }
}

const TS_TYPE_OPERATOR_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(TSTypeOperator, type_annotation);

impl<'a> AstNode<'a, TSTypeOperator<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn operator(&self) -> TSTypeOperatorOperator {
        self.inner.operator
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }
}

const TS_ARRAY_TYPE_OFFSET_ELEMENTTYPE: usize = std::mem::offset_of!(TSArrayType, element_type);

impl<'a> AstNode<'a, TSArrayType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn element_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.element_type,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }
}

const TS_INDEXED_ACCESS_TYPE_OFFSET_OBJECTTYPE: usize =
    std::mem::offset_of!(TSIndexedAccessType, object_type);
const TS_INDEXED_ACCESS_TYPE_OFFSET_INDEXTYPE: usize =
    std::mem::offset_of!(TSIndexedAccessType, index_type);

impl<'a> AstNode<'a, TSIndexedAccessType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn object_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = Some(FollowingNode::TSType({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(TS_INDEXED_ACCESS_TYPE_OFFSET_INDEXTYPE) as *const TSType<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.object_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIndexedAccessType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn index_type(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.index_type,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSIndexedAccessType(transmute_self(self))),
            following_node,
        })
    }
}

const TS_TUPLE_TYPE_OFFSET_ELEMENTTYPES: usize = std::mem::offset_of!(TSTupleType, element_types);

impl<'a> AstNode<'a, TSTupleType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn element_types(&self) -> &AstNode<'a, Vec<'a, TSTupleElement<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.element_types,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }
}

const TS_NAMED_TUPLE_MEMBER_OFFSET_LABEL: usize = std::mem::offset_of!(TSNamedTupleMember, label);
const TS_NAMED_TUPLE_MEMBER_OFFSET_ELEMENTTYPE: usize =
    std::mem::offset_of!(TSNamedTupleMember, element_type);

impl<'a> AstNode<'a, TSNamedTupleMember<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn label(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = Some(FollowingNode::TSTupleElement({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_NAMED_TUPLE_MEMBER_OFFSET_ELEMENTTYPE) as *const TSTupleElement<'a>)
            }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.label,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSNamedTupleMember(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn element_type(&self) -> &AstNode<'a, TSTupleElement<'a>> {
        let following_node = self.following_node.clone();
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
}

const TS_OPTIONAL_TYPE_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(TSOptionalType, type_annotation);

impl<'a> AstNode<'a, TSOptionalType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }
}

const TS_REST_TYPE_OFFSET_TYPEANNOTATION: usize = std::mem::offset_of!(TSRestType, type_annotation);

impl<'a> AstNode<'a, TSRestType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }
}

impl<'a> AstNode<'a, TSTupleElement<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSTupleElement::TSOptionalType(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSTupleElement::TSRestType(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            it @ match_ts_type!(TSTupleElement) => {
                return self
                    .allocator
                    .alloc(AstNode {
                        inner: it.to_ts_type(),
                        parent,
                        allocator: self.allocator,
                        following_node: self.following_node.clone(),
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
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, TSStringKeyword> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, TSBooleanKeyword> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, TSNumberKeyword> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, TSNeverKeyword> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, TSIntrinsicKeyword> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, TSUnknownKeyword> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, TSNullKeyword> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, TSUndefinedKeyword> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, TSVoidKeyword> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, TSSymbolKeyword> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, TSThisType> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, TSObjectKeyword> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

impl<'a> AstNode<'a, TSBigIntKeyword> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}

const TS_TYPE_REFERENCE_OFFSET_TYPENAME: usize = std::mem::offset_of!(TSTypeReference, type_name);
const TS_TYPE_REFERENCE_OFFSET_TYPEARGUMENTS: usize =
    std::mem::offset_of!(TSTypeReference, type_arguments);

impl<'a> AstNode<'a, TSTypeReference<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn type_name(&self) -> &AstNode<'a, TSTypeName<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_TYPE_REFERENCE_OFFSET_TYPEARGUMENTS)
                    as *const Option<Box<'a, TSTypeParameterInstantiation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterInstantiation(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeReference(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeReference(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

impl<'a> AstNode<'a, TSTypeName<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.allocator.alloc(AstNodes::TSTypeName(transmute_self(self)));
        let node = match self.inner {
            TSTypeName::IdentifierReference(s) => {
                AstNodes::IdentifierReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSTypeName::QualifiedName(s) => {
                AstNodes::TSQualifiedName(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const TS_QUALIFIED_NAME_OFFSET_LEFT: usize = std::mem::offset_of!(TSQualifiedName, left);
const TS_QUALIFIED_NAME_OFFSET_RIGHT: usize = std::mem::offset_of!(TSQualifiedName, right);

impl<'a> AstNode<'a, TSQualifiedName<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn left(&self) -> &AstNode<'a, TSTypeName<'a>> {
        let following_node = Some(FollowingNode::IdentifierName({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(TS_QUALIFIED_NAME_OFFSET_RIGHT) as *const IdentifierName<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.left,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSQualifiedName(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn right(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.right,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSQualifiedName(transmute_self(self))),
            following_node,
        })
    }
}

const TS_TYPE_PARAMETER_INSTANTIATION_OFFSET_PARAMS: usize =
    std::mem::offset_of!(TSTypeParameterInstantiation, params);

impl<'a> AstNode<'a, TSTypeParameterInstantiation<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, Vec<'a, TSType<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.params,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::TSTypeParameterInstantiation(transmute_self(self))),
            following_node,
        })
    }
}

const TS_TYPE_PARAMETER_OFFSET_NAME: usize = std::mem::offset_of!(TSTypeParameter, name);
const TS_TYPE_PARAMETER_OFFSET_CONSTRAINT: usize =
    std::mem::offset_of!(TSTypeParameter, constraint);
const TS_TYPE_PARAMETER_OFFSET_DEFAULT: usize = std::mem::offset_of!(TSTypeParameter, default);

impl<'a> AstNode<'a, TSTypeParameter<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn name(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(TS_TYPE_PARAMETER_OFFSET_CONSTRAINT) as *const Option<TSType<'a>>) }
        })
        .as_ref()
        .map(|t| FollowingNode::TSType(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeParameter(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn constraint(&self) -> Option<&AstNode<'a, TSType<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(TS_TYPE_PARAMETER_OFFSET_DEFAULT) as *const Option<TSType<'a>>) }
        })
        .as_ref()
        .map(|t| FollowingNode::TSType(t));
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
        let following_node = self.following_node.clone();
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
}

const TS_TYPE_PARAMETER_DECLARATION_OFFSET_PARAMS: usize =
    std::mem::offset_of!(TSTypeParameterDeclaration, params);

impl<'a> AstNode<'a, TSTypeParameterDeclaration<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, Vec<'a, TSTypeParameter<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.params,
            allocator: self.allocator,
            parent: self
                .allocator
                .alloc(AstNodes::TSTypeParameterDeclaration(transmute_self(self))),
            following_node,
        })
    }
}

const TS_TYPE_ALIAS_DECLARATION_OFFSET_ID: usize = std::mem::offset_of!(TSTypeAliasDeclaration, id);
const TS_TYPE_ALIAS_DECLARATION_OFFSET_TYPEPARAMETERS: usize =
    std::mem::offset_of!(TSTypeAliasDeclaration, type_parameters);
const TS_TYPE_ALIAS_DECLARATION_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(TSTypeAliasDeclaration, type_annotation);

impl<'a> AstNode<'a, TSTypeAliasDeclaration<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_TYPE_ALIAS_DECLARATION_OFFSET_TYPEPARAMETERS)
                    as *const Option<Box<'a, TSTypeParameterDeclaration<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterDeclaration(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAliasDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = Some(FollowingNode::TSType({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_TYPE_ALIAS_DECLARATION_OFFSET_TYPEANNOTATION) as *const TSType<'a>)
            }
        }));
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
        let following_node = self.following_node.clone();
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
}

const TS_CLASS_IMPLEMENTS_OFFSET_EXPRESSION: usize =
    std::mem::offset_of!(TSClassImplements, expression);
const TS_CLASS_IMPLEMENTS_OFFSET_TYPEARGUMENTS: usize =
    std::mem::offset_of!(TSClassImplements, type_arguments);

impl<'a> AstNode<'a, TSClassImplements<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, TSTypeName<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_CLASS_IMPLEMENTS_OFFSET_TYPEARGUMENTS)
                    as *const Option<Box<'a, TSTypeParameterInstantiation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterInstantiation(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSClassImplements(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSClassImplements(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const TS_INTERFACE_DECLARATION_OFFSET_ID: usize = std::mem::offset_of!(TSInterfaceDeclaration, id);
const TS_INTERFACE_DECLARATION_OFFSET_TYPEPARAMETERS: usize =
    std::mem::offset_of!(TSInterfaceDeclaration, type_parameters);
const TS_INTERFACE_DECLARATION_OFFSET_EXTENDS: usize =
    std::mem::offset_of!(TSInterfaceDeclaration, extends);
const TS_INTERFACE_DECLARATION_OFFSET_BODY: usize =
    std::mem::offset_of!(TSInterfaceDeclaration, body);

impl<'a> AstNode<'a, TSInterfaceDeclaration<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_INTERFACE_DECLARATION_OFFSET_TYPEPARAMETERS)
                    as *const Option<Box<'a, TSTypeParameterDeclaration<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterDeclaration(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_INTERFACE_DECLARATION_OFFSET_EXTENDS)
                    as *const Vec<'a, TSInterfaceHeritage<'a>>)
            }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::TSInterfaceHeritage(t));
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
        let following_node = Some(FollowingNode::TSInterfaceBody(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(TS_INTERFACE_DECLARATION_OFFSET_BODY)
                        as *const Box<'a, TSInterfaceBody<'a>>)
                }
            }
            .as_ref(),
        ));
        self.allocator.alloc(AstNode {
            inner: &self.inner.extends,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, TSInterfaceBody<'a>> {
        let following_node = self.following_node.clone();
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
}

const TS_INTERFACE_BODY_OFFSET_BODY: usize = std::mem::offset_of!(TSInterfaceBody, body);

impl<'a> AstNode<'a, TSInterfaceBody<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, TSSignature<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }
}

const TS_PROPERTY_SIGNATURE_OFFSET_KEY: usize = std::mem::offset_of!(TSPropertySignature, key);
const TS_PROPERTY_SIGNATURE_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(TSPropertySignature, type_annotation);

impl<'a> AstNode<'a, TSPropertySignature<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
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
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_PROPERTY_SIGNATURE_OFFSET_TYPEANNOTATION)
                    as *const Option<Box<'a, TSTypeAnnotation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeAnnotation(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.key,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSPropertySignature(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSPropertySignature(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

impl<'a> AstNode<'a, TSSignature<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.parent;
        let node = match self.inner {
            TSSignature::TSIndexSignature(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSSignature::TSPropertySignature(s) => {
                AstNodes::TSPropertySignature(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSSignature::TSCallSignatureDeclaration(s) => {
                panic!(
                    "No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`"
                )
            }
            TSSignature::TSConstructSignatureDeclaration(s) => {
                AstNodes::TSConstructSignatureDeclaration(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            TSSignature::TSMethodSignature(s) => {
                AstNodes::TSMethodSignature(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const TS_INDEX_SIGNATURE_OFFSET_PARAMETERS: usize =
    std::mem::offset_of!(TSIndexSignature, parameters);
const TS_INDEX_SIGNATURE_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(TSIndexSignature, type_annotation);

impl<'a> AstNode<'a, TSIndexSignature<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn parameters(&self) -> &AstNode<'a, Vec<'a, TSIndexSignatureName<'a>>> {
        let following_node = Some(FollowingNode::TSTypeAnnotation(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(TS_INDEX_SIGNATURE_OFFSET_TYPEANNOTATION)
                        as *const Box<'a, TSTypeAnnotation<'a>>)
                }
            }
            .as_ref(),
        ));
        self.allocator.alloc(AstNode {
            inner: &self.inner.parameters,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSTypeAnnotation<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: self.inner.type_annotation.as_ref(),
            allocator: self.allocator,
            parent: self.parent,
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
}

const TS_CALL_SIGNATURE_DECLARATION_OFFSET_TYPEPARAMETERS: usize =
    std::mem::offset_of!(TSCallSignatureDeclaration, type_parameters);
const TS_CALL_SIGNATURE_DECLARATION_OFFSET_THISPARAM: usize =
    std::mem::offset_of!(TSCallSignatureDeclaration, this_param);
const TS_CALL_SIGNATURE_DECLARATION_OFFSET_PARAMS: usize =
    std::mem::offset_of!(TSCallSignatureDeclaration, params);
const TS_CALL_SIGNATURE_DECLARATION_OFFSET_RETURNTYPE: usize =
    std::mem::offset_of!(TSCallSignatureDeclaration, return_type);

impl<'a> AstNode<'a, TSCallSignatureDeclaration<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_CALL_SIGNATURE_DECLARATION_OFFSET_THISPARAM)
                    as *const Option<Box<'a, TSThisParameter<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSThisParameter(t));
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn this_param(&self) -> Option<&AstNode<'a, TSThisParameter<'a>>> {
        let following_node = Some(FollowingNode::FormalParameters(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(TS_CALL_SIGNATURE_DECLARATION_OFFSET_PARAMS)
                        as *const Box<'a, FormalParameters<'a>>)
                }
            }
            .as_ref(),
        ));
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_CALL_SIGNATURE_DECLARATION_OFFSET_RETURNTYPE)
                    as *const Option<Box<'a, TSTypeAnnotation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeAnnotation(t));
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn return_type(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
                following_node,
            }))
            .as_ref()
    }
}

const TS_METHOD_SIGNATURE_OFFSET_KEY: usize = std::mem::offset_of!(TSMethodSignature, key);
const TS_METHOD_SIGNATURE_OFFSET_TYPEPARAMETERS: usize =
    std::mem::offset_of!(TSMethodSignature, type_parameters);
const TS_METHOD_SIGNATURE_OFFSET_THISPARAM: usize =
    std::mem::offset_of!(TSMethodSignature, this_param);
const TS_METHOD_SIGNATURE_OFFSET_PARAMS: usize = std::mem::offset_of!(TSMethodSignature, params);
const TS_METHOD_SIGNATURE_OFFSET_RETURNTYPE: usize =
    std::mem::offset_of!(TSMethodSignature, return_type);

impl<'a> AstNode<'a, TSMethodSignature<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn key(&self) -> &AstNode<'a, PropertyKey<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_METHOD_SIGNATURE_OFFSET_TYPEPARAMETERS)
                    as *const Option<Box<'a, TSTypeParameterDeclaration<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterDeclaration(t));
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
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_METHOD_SIGNATURE_OFFSET_THISPARAM)
                    as *const Option<Box<'a, TSThisParameter<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSThisParameter(t));
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
        let following_node = Some(FollowingNode::FormalParameters(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(TS_METHOD_SIGNATURE_OFFSET_PARAMS)
                        as *const Box<'a, FormalParameters<'a>>)
                }
            }
            .as_ref(),
        ));
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
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_METHOD_SIGNATURE_OFFSET_RETURNTYPE)
                    as *const Option<Box<'a, TSTypeAnnotation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeAnnotation(t));
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSMethodSignature(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn return_type(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.return_type.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSMethodSignature(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const TS_CONSTRUCT_SIGNATURE_DECLARATION_OFFSET_TYPEPARAMETERS: usize =
    std::mem::offset_of!(TSConstructSignatureDeclaration, type_parameters);
const TS_CONSTRUCT_SIGNATURE_DECLARATION_OFFSET_PARAMS: usize =
    std::mem::offset_of!(TSConstructSignatureDeclaration, params);
const TS_CONSTRUCT_SIGNATURE_DECLARATION_OFFSET_RETURNTYPE: usize =
    std::mem::offset_of!(TSConstructSignatureDeclaration, return_type);

impl<'a> AstNode<'a, TSConstructSignatureDeclaration<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = Some(FollowingNode::FormalParameters(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(TS_CONSTRUCT_SIGNATURE_DECLARATION_OFFSET_PARAMS)
                        as *const Box<'a, FormalParameters<'a>>)
                }
            }
            .as_ref(),
        ));
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
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_CONSTRUCT_SIGNATURE_DECLARATION_OFFSET_RETURNTYPE)
                    as *const Option<Box<'a, TSTypeAnnotation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeAnnotation(t));
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
        let following_node = self.following_node.clone();
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
}

const TS_INDEX_SIGNATURE_NAME_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(TSIndexSignatureName, type_annotation);

impl<'a> AstNode<'a, TSIndexSignatureName<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn name(&self) -> Atom<'a> {
        self.inner.name
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSTypeAnnotation<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: self.inner.type_annotation.as_ref(),
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }
}

const TS_INTERFACE_HERITAGE_OFFSET_EXPRESSION: usize =
    std::mem::offset_of!(TSInterfaceHeritage, expression);
const TS_INTERFACE_HERITAGE_OFFSET_TYPEARGUMENTS: usize =
    std::mem::offset_of!(TSInterfaceHeritage, type_arguments);

impl<'a> AstNode<'a, TSInterfaceHeritage<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_INTERFACE_HERITAGE_OFFSET_TYPEARGUMENTS)
                    as *const Option<Box<'a, TSTypeParameterInstantiation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterInstantiation(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInterfaceHeritage(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSInterfaceHeritage(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const TS_TYPE_PREDICATE_OFFSET_PARAMETERNAME: usize =
    std::mem::offset_of!(TSTypePredicate, parameter_name);
const TS_TYPE_PREDICATE_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(TSTypePredicate, type_annotation);

impl<'a> AstNode<'a, TSTypePredicate<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn parameter_name(&self) -> &AstNode<'a, TSTypePredicateName<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_TYPE_PREDICATE_OFFSET_TYPEANNOTATION)
                    as *const Option<Box<'a, TSTypeAnnotation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeAnnotation(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.parameter_name,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn asserts(&self) -> bool {
        self.inner.asserts
    }

    #[inline]
    pub fn type_annotation(&self) -> Option<&AstNode<'a, TSTypeAnnotation<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.type_annotation.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
                following_node,
            }))
            .as_ref()
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
                    following_node: self.following_node.clone(),
                }))
            }
            TSTypePredicateName::This(s) => AstNodes::TSThisType(self.allocator.alloc(AstNode {
                inner: s,
                parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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

const TS_MODULE_DECLARATION_OFFSET_ID: usize = std::mem::offset_of!(TSModuleDeclaration, id);
const TS_MODULE_DECLARATION_OFFSET_BODY: usize = std::mem::offset_of!(TSModuleDeclaration, body);

impl<'a> AstNode<'a, TSModuleDeclaration<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn id(&self) -> &AstNode<'a, TSModuleDeclarationName<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_MODULE_DECLARATION_OFFSET_BODY)
                    as *const Option<TSModuleDeclarationBody<'a>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSModuleDeclarationBody(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSModuleDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> Option<&AstNode<'a, TSModuleDeclarationBody<'a>>> {
        let following_node = self.following_node.clone();
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
                    following_node: self.following_node.clone(),
                }))
            }
            TSModuleDeclarationName::StringLiteral(s) => {
                AstNodes::StringLiteral(self.allocator.alloc(AstNode {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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
                    following_node: self.following_node.clone(),
                }))
            }
            TSModuleDeclarationBody::TSModuleBlock(s) => {
                AstNodes::TSModuleBlock(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
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

const TS_MODULE_BLOCK_OFFSET_DIRECTIVES: usize = std::mem::offset_of!(TSModuleBlock, directives);
const TS_MODULE_BLOCK_OFFSET_BODY: usize = std::mem::offset_of!(TSModuleBlock, body);

impl<'a> AstNode<'a, TSModuleBlock<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn directives(&self) -> &AstNode<'a, Vec<'a, Directive<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(TS_MODULE_BLOCK_OFFSET_BODY) as *const Vec<'a, Statement<'a>>) }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::Statement(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.directives,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSModuleBlock(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn body(&self) -> &AstNode<'a, Vec<'a, Statement<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.body,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSModuleBlock(transmute_self(self))),
            following_node,
        })
    }
}

const TS_TYPE_LITERAL_OFFSET_MEMBERS: usize = std::mem::offset_of!(TSTypeLiteral, members);

impl<'a> AstNode<'a, TSTypeLiteral<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn members(&self) -> &AstNode<'a, Vec<'a, TSSignature<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.members,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeLiteral(transmute_self(self))),
            following_node,
        })
    }
}

const TS_INFER_TYPE_OFFSET_TYPEPARAMETER: usize = std::mem::offset_of!(TSInferType, type_parameter);

impl<'a> AstNode<'a, TSInferType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn type_parameter(&self) -> &AstNode<'a, TSTypeParameter<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: self.inner.type_parameter.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInferType(transmute_self(self))),
            following_node,
        })
    }
}

const TS_TYPE_QUERY_OFFSET_EXPRNAME: usize = std::mem::offset_of!(TSTypeQuery, expr_name);
const TS_TYPE_QUERY_OFFSET_TYPEARGUMENTS: usize = std::mem::offset_of!(TSTypeQuery, type_arguments);

impl<'a> AstNode<'a, TSTypeQuery<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expr_name(&self) -> &AstNode<'a, TSTypeQueryExprName<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_TYPE_QUERY_OFFSET_TYPEARGUMENTS)
                    as *const Option<Box<'a, TSTypeParameterInstantiation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterInstantiation(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.expr_name,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeQuery(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSTypeQuery(transmute_self(self))),
                following_node,
            }))
            .as_ref()
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
                    following_node: self.following_node.clone(),
                }))
            }
            it @ match_ts_type_name!(TSTypeQueryExprName) => {
                AstNodes::TSTypeName(self.allocator.alloc(AstNode {
                    inner: it.to_ts_type_name(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
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

const TS_IMPORT_TYPE_OFFSET_ARGUMENT: usize = std::mem::offset_of!(TSImportType, argument);
const TS_IMPORT_TYPE_OFFSET_OPTIONS: usize = std::mem::offset_of!(TSImportType, options);
const TS_IMPORT_TYPE_OFFSET_QUALIFIER: usize = std::mem::offset_of!(TSImportType, qualifier);
const TS_IMPORT_TYPE_OFFSET_TYPEARGUMENTS: usize =
    std::mem::offset_of!(TSImportType, type_arguments);

impl<'a> AstNode<'a, TSImportType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn argument(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_IMPORT_TYPE_OFFSET_OPTIONS)
                    as *const Option<Box<'a, ObjectExpression<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::ObjectExpression(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.argument,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSImportType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn options(&self) -> Option<&AstNode<'a, ObjectExpression<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(TS_IMPORT_TYPE_OFFSET_QUALIFIER) as *const Option<TSTypeName<'a>>) }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeName(t));
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
    pub fn qualifier(&self) -> Option<&AstNode<'a, TSTypeName<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_IMPORT_TYPE_OFFSET_TYPEARGUMENTS)
                    as *const Option<Box<'a, TSTypeParameterInstantiation<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSTypeParameterInstantiation(t));
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
        let following_node = self.following_node.clone();
        self.allocator
            .alloc(self.inner.type_arguments.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.allocator.alloc(AstNodes::TSImportType(transmute_self(self))),
                following_node,
            }))
            .as_ref()
    }
}

const TS_FUNCTION_TYPE_OFFSET_TYPEPARAMETERS: usize =
    std::mem::offset_of!(TSFunctionType, type_parameters);
const TS_FUNCTION_TYPE_OFFSET_THISPARAM: usize = std::mem::offset_of!(TSFunctionType, this_param);
const TS_FUNCTION_TYPE_OFFSET_PARAMS: usize = std::mem::offset_of!(TSFunctionType, params);
const TS_FUNCTION_TYPE_OFFSET_RETURNTYPE: usize = std::mem::offset_of!(TSFunctionType, return_type);

impl<'a> AstNode<'a, TSFunctionType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_FUNCTION_TYPE_OFFSET_THISPARAM)
                    as *const Option<Box<'a, TSThisParameter<'a>>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSThisParameter(t));
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn this_param(&self) -> Option<&AstNode<'a, TSThisParameter<'a>>> {
        let following_node = Some(FollowingNode::FormalParameters(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(TS_FUNCTION_TYPE_OFFSET_PARAMS)
                        as *const Box<'a, FormalParameters<'a>>)
                }
            }
            .as_ref(),
        ));
        self.allocator
            .alloc(self.inner.this_param.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_node = Some(FollowingNode::TSTypeAnnotation(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(TS_FUNCTION_TYPE_OFFSET_RETURNTYPE)
                        as *const Box<'a, TSTypeAnnotation<'a>>)
                }
            }
            .as_ref(),
        ));
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn return_type(&self) -> &AstNode<'a, TSTypeAnnotation<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: self.inner.return_type.as_ref(),
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }
}

const TS_CONSTRUCTOR_TYPE_OFFSET_TYPEPARAMETERS: usize =
    std::mem::offset_of!(TSConstructorType, type_parameters);
const TS_CONSTRUCTOR_TYPE_OFFSET_PARAMS: usize = std::mem::offset_of!(TSConstructorType, params);
const TS_CONSTRUCTOR_TYPE_OFFSET_RETURNTYPE: usize =
    std::mem::offset_of!(TSConstructorType, return_type);

impl<'a> AstNode<'a, TSConstructorType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn r#abstract(&self) -> bool {
        self.inner.r#abstract
    }

    #[inline]
    pub fn type_parameters(&self) -> Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>> {
        let following_node = Some(FollowingNode::FormalParameters(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(TS_CONSTRUCTOR_TYPE_OFFSET_PARAMS)
                        as *const Box<'a, FormalParameters<'a>>)
                }
            }
            .as_ref(),
        ));
        self.allocator
            .alloc(self.inner.type_parameters.as_ref().map(|inner| AstNode {
                inner: inner.as_ref(),
                allocator: self.allocator,
                parent: self.parent,
                following_node,
            }))
            .as_ref()
    }

    #[inline]
    pub fn params(&self) -> &AstNode<'a, FormalParameters<'a>> {
        let following_node = Some(FollowingNode::TSTypeAnnotation(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(TS_CONSTRUCTOR_TYPE_OFFSET_RETURNTYPE)
                        as *const Box<'a, TSTypeAnnotation<'a>>)
                }
            }
            .as_ref(),
        ));
        self.allocator.alloc(AstNode {
            inner: self.inner.params.as_ref(),
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }

    #[inline]
    pub fn return_type(&self) -> &AstNode<'a, TSTypeAnnotation<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: self.inner.return_type.as_ref(),
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }
}

const TS_MAPPED_TYPE_OFFSET_TYPEPARAMETER: usize =
    std::mem::offset_of!(TSMappedType, type_parameter);
const TS_MAPPED_TYPE_OFFSET_NAMETYPE: usize = std::mem::offset_of!(TSMappedType, name_type);
const TS_MAPPED_TYPE_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(TSMappedType, type_annotation);

impl<'a> AstNode<'a, TSMappedType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn type_parameter(&self) -> &AstNode<'a, TSTypeParameter<'a>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(TS_MAPPED_TYPE_OFFSET_NAMETYPE) as *const Option<TSType<'a>>) }
        })
        .as_ref()
        .map(|t| FollowingNode::TSType(t));
        self.allocator.alloc(AstNode {
            inner: self.inner.type_parameter.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSMappedType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn name_type(&self) -> Option<&AstNode<'a, TSType<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_MAPPED_TYPE_OFFSET_TYPEANNOTATION) as *const Option<TSType<'a>>)
            }
        })
        .as_ref()
        .map(|t| FollowingNode::TSType(t));
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
        let following_node = self.following_node.clone();
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
}

const TS_TEMPLATE_LITERAL_TYPE_OFFSET_QUASIS: usize =
    std::mem::offset_of!(TSTemplateLiteralType, quasis);
const TS_TEMPLATE_LITERAL_TYPE_OFFSET_TYPES: usize =
    std::mem::offset_of!(TSTemplateLiteralType, types);

impl<'a> AstNode<'a, TSTemplateLiteralType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn quasis(&self) -> &AstNode<'a, Vec<'a, TemplateElement<'a>>> {
        let following_node = ({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_TEMPLATE_LITERAL_TYPE_OFFSET_TYPES) as *const Vec<'a, TSType<'a>>)
            }
        })
        .first()
        .as_ref()
        .copied()
        .map(|t| FollowingNode::TSType(t));
        self.allocator.alloc(AstNode {
            inner: &self.inner.quasis,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTemplateLiteralType(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn types(&self) -> &AstNode<'a, Vec<'a, TSType<'a>>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.types,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTemplateLiteralType(transmute_self(self))),
            following_node,
        })
    }
}

const TS_AS_EXPRESSION_OFFSET_EXPRESSION: usize = std::mem::offset_of!(TSAsExpression, expression);
const TS_AS_EXPRESSION_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(TSAsExpression, type_annotation);

impl<'a> AstNode<'a, TSAsExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::TSType({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(TS_AS_EXPRESSION_OFFSET_TYPEANNOTATION) as *const TSType<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSAsExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSAsExpression(transmute_self(self))),
            following_node,
        })
    }
}

const TS_SATISFIES_EXPRESSION_OFFSET_EXPRESSION: usize =
    std::mem::offset_of!(TSSatisfiesExpression, expression);
const TS_SATISFIES_EXPRESSION_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(TSSatisfiesExpression, type_annotation);

impl<'a> AstNode<'a, TSSatisfiesExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::TSType({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_SATISFIES_EXPRESSION_OFFSET_TYPEANNOTATION) as *const TSType<'a>)
            }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSSatisfiesExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSSatisfiesExpression(transmute_self(self))),
            following_node,
        })
    }
}

const TS_TYPE_ASSERTION_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(TSTypeAssertion, type_annotation);
const TS_TYPE_ASSERTION_OFFSET_EXPRESSION: usize =
    std::mem::offset_of!(TSTypeAssertion, expression);

impl<'a> AstNode<'a, TSTypeAssertion<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = Some(FollowingNode::Expression({
            let ptr = self.inner as *const _ as *const u8;
            unsafe { &*(ptr.add(TS_TYPE_ASSERTION_OFFSET_EXPRESSION) as *const Expression<'a>) }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.type_annotation,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAssertion(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSTypeAssertion(transmute_self(self))),
            following_node,
        })
    }
}

const TS_IMPORT_EQUALS_DECLARATION_OFFSET_ID: usize =
    std::mem::offset_of!(TSImportEqualsDeclaration, id);
const TS_IMPORT_EQUALS_DECLARATION_OFFSET_MODULEREFERENCE: usize =
    std::mem::offset_of!(TSImportEqualsDeclaration, module_reference);

impl<'a> AstNode<'a, TSImportEqualsDeclaration<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn id(&self) -> &AstNode<'a, BindingIdentifier<'a>> {
        let following_node = Some(FollowingNode::TSModuleReference({
            let ptr = self.inner as *const _ as *const u8;
            unsafe {
                &*(ptr.add(TS_IMPORT_EQUALS_DECLARATION_OFFSET_MODULEREFERENCE)
                    as *const TSModuleReference<'a>)
            }
        }));
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSImportEqualsDeclaration(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn module_reference(&self) -> &AstNode<'a, TSModuleReference<'a>> {
        let following_node = self.following_node.clone();
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
}

impl<'a> AstNode<'a, TSModuleReference<'a>> {
    #[inline]
    pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
        let parent = self.allocator.alloc(AstNodes::TSModuleReference(transmute_self(self)));
        let node = match self.inner {
            TSModuleReference::ExternalModuleReference(s) => {
                AstNodes::TSExternalModuleReference(self.allocator.alloc(AstNode {
                    inner: s.as_ref(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
            }
            it @ match_ts_type_name!(TSModuleReference) => {
                AstNodes::TSTypeName(self.allocator.alloc(AstNode {
                    inner: it.to_ts_type_name(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }))
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

const TS_EXTERNAL_MODULE_REFERENCE_OFFSET_EXPRESSION: usize =
    std::mem::offset_of!(TSExternalModuleReference, expression);

impl<'a> AstNode<'a, TSExternalModuleReference<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, StringLiteral<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSExternalModuleReference(transmute_self(self))),
            following_node,
        })
    }
}

const TS_NON_NULL_EXPRESSION_OFFSET_EXPRESSION: usize =
    std::mem::offset_of!(TSNonNullExpression, expression);

impl<'a> AstNode<'a, TSNonNullExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSNonNullExpression(transmute_self(self))),
            following_node,
        })
    }
}

const DECORATOR_OFFSET_EXPRESSION: usize = std::mem::offset_of!(Decorator, expression);

impl<'a> AstNode<'a, Decorator<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::Decorator(transmute_self(self))),
            following_node,
        })
    }
}

const TS_EXPORT_ASSIGNMENT_OFFSET_EXPRESSION: usize =
    std::mem::offset_of!(TSExportAssignment, expression);

impl<'a> AstNode<'a, TSExportAssignment<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSExportAssignment(transmute_self(self))),
            following_node,
        })
    }
}

const TS_NAMESPACE_EXPORT_DECLARATION_OFFSET_ID: usize =
    std::mem::offset_of!(TSNamespaceExportDeclaration, id);

impl<'a> AstNode<'a, TSNamespaceExportDeclaration<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn id(&self) -> &AstNode<'a, IdentifierName<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: &self.inner.id,
            allocator: self.allocator,
            parent: self.parent,
            following_node,
        })
    }
}

const TS_INSTANTIATION_EXPRESSION_OFFSET_EXPRESSION: usize =
    std::mem::offset_of!(TSInstantiationExpression, expression);
const TS_INSTANTIATION_EXPRESSION_OFFSET_TYPEARGUMENTS: usize =
    std::mem::offset_of!(TSInstantiationExpression, type_arguments);

impl<'a> AstNode<'a, TSInstantiationExpression<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn expression(&self) -> &AstNode<'a, Expression<'a>> {
        let following_node = Some(FollowingNode::TSTypeParameterInstantiation(
            {
                let ptr = self.inner as *const _ as *const u8;
                unsafe {
                    &*(ptr.add(TS_INSTANTIATION_EXPRESSION_OFFSET_TYPEARGUMENTS)
                        as *const Box<'a, TSTypeParameterInstantiation<'a>>)
                }
            }
            .as_ref(),
        ));
        self.allocator.alloc(AstNode {
            inner: &self.inner.expression,
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInstantiationExpression(transmute_self(self))),
            following_node,
        })
    }

    #[inline]
    pub fn type_arguments(&self) -> &AstNode<'a, TSTypeParameterInstantiation<'a>> {
        let following_node = self.following_node.clone();
        self.allocator.alloc(AstNode {
            inner: self.inner.type_arguments.as_ref(),
            allocator: self.allocator,
            parent: self.allocator.alloc(AstNodes::TSInstantiationExpression(transmute_self(self))),
            following_node,
        })
    }
}

const JS_DOC_NULLABLE_TYPE_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(JSDocNullableType, type_annotation);

impl<'a> AstNode<'a, JSDocNullableType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node.clone();
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
}

const JS_DOC_NON_NULLABLE_TYPE_OFFSET_TYPEANNOTATION: usize =
    std::mem::offset_of!(JSDocNonNullableType, type_annotation);

impl<'a> AstNode<'a, JSDocNonNullableType<'a>> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }

    #[inline]
    pub fn type_annotation(&self) -> &AstNode<'a, TSType<'a>> {
        let following_node = self.following_node.clone();
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
}

impl<'a> AstNode<'a, JSDocUnknownType> {
    #[inline]
    pub fn span(&self) -> Span {
        self.inner.span
    }
}
pub struct AstNodeIterator<'a, T> {
    inner: std::iter::Peekable<std::slice::Iter<'a, T>>,
    parent: &'a AstNodes<'a>,
    following_node: Option<FollowingNode<'a>>,
    allocator: &'a Allocator,
}
impl<'a> AstNode<'a, Vec<'a, Expression<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Expression<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, Expression<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::Expression(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, Expression<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::Expression(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, ArrayExpressionElement<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, ArrayExpressionElement<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, ArrayExpressionElement<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::ArrayExpressionElement(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, ArrayExpressionElement<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::ArrayExpressionElement(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, ObjectPropertyKind<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, ObjectPropertyKind<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, ObjectPropertyKind<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::ObjectPropertyKind(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, ObjectPropertyKind<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::ObjectPropertyKind(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, TemplateElement<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TemplateElement<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, TemplateElement<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::TemplateElement(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, TemplateElement<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::TemplateElement(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, Argument<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Argument<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, Argument<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::Argument(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, Argument<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::Argument(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, AssignmentTargetProperty<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, AssignmentTargetProperty<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, AssignmentTargetProperty<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::AssignmentTargetProperty(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, AssignmentTargetProperty<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::AssignmentTargetProperty(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, Statement<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Statement<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, Statement<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::Statement(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, Statement<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::Statement(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, Directive<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Directive<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, Directive<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::Directive(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, Directive<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::Directive(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, VariableDeclarator<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, VariableDeclarator<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, VariableDeclarator<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::VariableDeclarator(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, VariableDeclarator<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::VariableDeclarator(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, SwitchCase<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, SwitchCase<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, SwitchCase<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::SwitchCase(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, SwitchCase<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::SwitchCase(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, BindingProperty<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, BindingProperty<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, BindingProperty<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::BindingProperty(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, BindingProperty<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::BindingProperty(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, FormalParameter<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, FormalParameter<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, FormalParameter<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::FormalParameter(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, FormalParameter<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::FormalParameter(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, ClassElement<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, ClassElement<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, ClassElement<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::ClassElement(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, ClassElement<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::ClassElement(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, ImportDeclarationSpecifier<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, ImportDeclarationSpecifier<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, ImportDeclarationSpecifier<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::ImportDeclarationSpecifier(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, ImportDeclarationSpecifier<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::ImportDeclarationSpecifier(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, ImportAttribute<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, ImportAttribute<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, ImportAttribute<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::ImportAttribute(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, ImportAttribute<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::ImportAttribute(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, ExportSpecifier<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, ExportSpecifier<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, ExportSpecifier<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::ExportSpecifier(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, ExportSpecifier<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::ExportSpecifier(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, JSXAttributeItem<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, JSXAttributeItem<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, JSXAttributeItem<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::JSXAttributeItem(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, JSXAttributeItem<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::JSXAttributeItem(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, JSXChild<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, JSXChild<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, JSXChild<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::JSXChild(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, JSXChild<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::JSXChild(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, TSEnumMember<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSEnumMember<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, TSEnumMember<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::TSEnumMember(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, TSEnumMember<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::TSEnumMember(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, TSType<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSType<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, TSType<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::TSType(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, TSType<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::TSType(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, TSTupleElement<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSTupleElement<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, TSTupleElement<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::TSTupleElement(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, TSTupleElement<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::TSTupleElement(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, TSTypeParameter<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSTypeParameter<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, TSTypeParameter<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::TSTypeParameter(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, TSTypeParameter<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::TSTypeParameter(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, TSClassImplements<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSClassImplements<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, TSClassImplements<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::TSClassImplements(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, TSClassImplements<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::TSClassImplements(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, TSSignature<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSSignature<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, TSSignature<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::TSSignature(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, TSSignature<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::TSSignature(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, TSIndexSignatureName<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSIndexSignatureName<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, TSIndexSignatureName<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::TSIndexSignatureName(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, TSIndexSignatureName<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::TSIndexSignatureName(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, TSInterfaceHeritage<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, TSInterfaceHeritage<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, TSInterfaceHeritage<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::TSInterfaceHeritage(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, TSInterfaceHeritage<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::TSInterfaceHeritage(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, Decorator<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Decorator<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, Decorator<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_node: inner_iter
                        .next()
                        .map(|t| FollowingNode::Decorator(t))
                        .or_else(|| self.following_node.clone()),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, Decorator<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|t| FollowingNode::Decorator(t))
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Option<AssignmentTargetMaybeDefault<'a>>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
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
                        .map(|next| next.as_ref().map(FollowingNode::AssignmentTargetMaybeDefault))
                        .unwrap_or_default()
                        .or_else(|| self.following_node.clone()),
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
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|next| next.as_ref().map(FollowingNode::AssignmentTargetMaybeDefault))
                        .unwrap_or_default()
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
impl<'a> AstNode<'a, Vec<'a, Option<BindingPattern<'a>>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Option<BindingPattern<'a>>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            following_node: self.following_node.clone(),
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
                        .map(|next| next.as_ref().map(FollowingNode::BindingPattern))
                        .unwrap_or_default()
                        .or_else(|| self.following_node.clone()),
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
                following_node: self.following_node.clone(),
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
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_node: self
                        .inner
                        .peek()
                        .map(|next| next.as_ref().map(FollowingNode::BindingPattern))
                        .unwrap_or_default()
                        .or_else(|| self.following_node.clone()),
                }
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
            following_node: self.following_node.clone(),
            allocator: self.allocator,
        }
    }
}
