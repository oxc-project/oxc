// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/take_in.rs`.

#![expect(clippy::elidable_lifetime_names)]

use oxc_allocator::TakeIn;

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl<'a> TakeIn<'a> for Program<'a> {}

impl<'a> TakeIn<'a> for Expression<'a> {}

impl<'a> TakeIn<'a> for IdentifierName<'a> {}

impl<'a> TakeIn<'a> for IdentifierReference<'a> {}

impl<'a> TakeIn<'a> for BindingIdentifier<'a> {}

impl<'a> TakeIn<'a> for LabelIdentifier<'a> {}

impl<'a> TakeIn<'a> for ThisExpression {}

impl<'a> TakeIn<'a> for ArrayExpression<'a> {}

impl<'a> TakeIn<'a> for ArrayExpressionElement<'a> {}

impl<'a> TakeIn<'a> for Elision {}

impl<'a> TakeIn<'a> for ObjectExpression<'a> {}

impl<'a> TakeIn<'a> for ObjectPropertyKind<'a> {}

impl<'a> TakeIn<'a> for ObjectProperty<'a> {}

impl<'a> TakeIn<'a> for PropertyKey<'a> {}

impl<'a> TakeIn<'a> for TemplateLiteral<'a> {}

impl<'a> TakeIn<'a> for TaggedTemplateExpression<'a> {}

impl<'a> TakeIn<'a> for TemplateElement<'a> {}

impl<'a> TakeIn<'a> for TemplateElementValue<'a> {}

impl<'a> TakeIn<'a> for MemberExpression<'a> {}

impl<'a> TakeIn<'a> for ComputedMemberExpression<'a> {}

impl<'a> TakeIn<'a> for StaticMemberExpression<'a> {}

impl<'a> TakeIn<'a> for PrivateFieldExpression<'a> {}

impl<'a> TakeIn<'a> for CallExpression<'a> {}

impl<'a> TakeIn<'a> for NewExpression<'a> {}

impl<'a> TakeIn<'a> for MetaProperty<'a> {}

impl<'a> TakeIn<'a> for SpreadElement<'a> {}

impl<'a> TakeIn<'a> for Argument<'a> {}

impl<'a> TakeIn<'a> for UpdateExpression<'a> {}

impl<'a> TakeIn<'a> for UnaryExpression<'a> {}

impl<'a> TakeIn<'a> for BinaryExpression<'a> {}

impl<'a> TakeIn<'a> for PrivateInExpression<'a> {}

impl<'a> TakeIn<'a> for LogicalExpression<'a> {}

impl<'a> TakeIn<'a> for ConditionalExpression<'a> {}

impl<'a> TakeIn<'a> for AssignmentExpression<'a> {}

impl<'a> TakeIn<'a> for AssignmentTarget<'a> {}

impl<'a> TakeIn<'a> for SimpleAssignmentTarget<'a> {}

impl<'a> TakeIn<'a> for AssignmentTargetPattern<'a> {}

impl<'a> TakeIn<'a> for ArrayAssignmentTarget<'a> {}

impl<'a> TakeIn<'a> for ObjectAssignmentTarget<'a> {}

impl<'a> TakeIn<'a> for AssignmentTargetRest<'a> {}

impl<'a> TakeIn<'a> for AssignmentTargetMaybeDefault<'a> {}

impl<'a> TakeIn<'a> for AssignmentTargetWithDefault<'a> {}

impl<'a> TakeIn<'a> for AssignmentTargetProperty<'a> {}

impl<'a> TakeIn<'a> for AssignmentTargetPropertyIdentifier<'a> {}

impl<'a> TakeIn<'a> for AssignmentTargetPropertyProperty<'a> {}

impl<'a> TakeIn<'a> for SequenceExpression<'a> {}

impl<'a> TakeIn<'a> for Super {}

impl<'a> TakeIn<'a> for AwaitExpression<'a> {}

impl<'a> TakeIn<'a> for ChainExpression<'a> {}

impl<'a> TakeIn<'a> for ChainElement<'a> {}

impl<'a> TakeIn<'a> for ParenthesizedExpression<'a> {}

impl<'a> TakeIn<'a> for Statement<'a> {}

impl<'a> TakeIn<'a> for Directive<'a> {}

impl<'a> TakeIn<'a> for Hashbang<'a> {}

impl<'a> TakeIn<'a> for BlockStatement<'a> {}

impl<'a> TakeIn<'a> for Declaration<'a> {}

impl<'a> TakeIn<'a> for VariableDeclaration<'a> {}

impl<'a> TakeIn<'a> for VariableDeclarator<'a> {}

impl<'a> TakeIn<'a> for EmptyStatement {}

impl<'a> TakeIn<'a> for ExpressionStatement<'a> {}

impl<'a> TakeIn<'a> for IfStatement<'a> {}

impl<'a> TakeIn<'a> for DoWhileStatement<'a> {}

impl<'a> TakeIn<'a> for WhileStatement<'a> {}

impl<'a> TakeIn<'a> for ForStatement<'a> {}

impl<'a> TakeIn<'a> for ForStatementInit<'a> {}

impl<'a> TakeIn<'a> for ForInStatement<'a> {}

impl<'a> TakeIn<'a> for ForStatementLeft<'a> {}

impl<'a> TakeIn<'a> for ForOfStatement<'a> {}

impl<'a> TakeIn<'a> for ContinueStatement<'a> {}

impl<'a> TakeIn<'a> for BreakStatement<'a> {}

impl<'a> TakeIn<'a> for ReturnStatement<'a> {}

impl<'a> TakeIn<'a> for WithStatement<'a> {}

impl<'a> TakeIn<'a> for SwitchStatement<'a> {}

impl<'a> TakeIn<'a> for SwitchCase<'a> {}

impl<'a> TakeIn<'a> for LabeledStatement<'a> {}

impl<'a> TakeIn<'a> for ThrowStatement<'a> {}

impl<'a> TakeIn<'a> for TryStatement<'a> {}

impl<'a> TakeIn<'a> for CatchClause<'a> {}

impl<'a> TakeIn<'a> for CatchParameter<'a> {}

impl<'a> TakeIn<'a> for DebuggerStatement {}

impl<'a> TakeIn<'a> for BindingPattern<'a> {}

impl<'a> TakeIn<'a> for BindingPatternKind<'a> {}

impl<'a> TakeIn<'a> for AssignmentPattern<'a> {}

impl<'a> TakeIn<'a> for ObjectPattern<'a> {}

impl<'a> TakeIn<'a> for BindingProperty<'a> {}

impl<'a> TakeIn<'a> for ArrayPattern<'a> {}

impl<'a> TakeIn<'a> for BindingRestElement<'a> {}

impl<'a> TakeIn<'a> for Function<'a> {}

impl<'a> TakeIn<'a> for FormalParameters<'a> {}

impl<'a> TakeIn<'a> for FormalParameter<'a> {}

impl<'a> TakeIn<'a> for FunctionBody<'a> {}

impl<'a> TakeIn<'a> for ArrowFunctionExpression<'a> {}

impl<'a> TakeIn<'a> for YieldExpression<'a> {}

impl<'a> TakeIn<'a> for Class<'a> {}

impl<'a> TakeIn<'a> for ClassBody<'a> {}

impl<'a> TakeIn<'a> for ClassElement<'a> {}

impl<'a> TakeIn<'a> for MethodDefinition<'a> {}

impl<'a> TakeIn<'a> for PropertyDefinition<'a> {}

impl<'a> TakeIn<'a> for PrivateIdentifier<'a> {}

impl<'a> TakeIn<'a> for StaticBlock<'a> {}

impl<'a> TakeIn<'a> for ModuleDeclaration<'a> {}

impl<'a> TakeIn<'a> for AccessorProperty<'a> {}

impl<'a> TakeIn<'a> for ImportExpression<'a> {}

impl<'a> TakeIn<'a> for ImportDeclaration<'a> {}

impl<'a> TakeIn<'a> for ImportDeclarationSpecifier<'a> {}

impl<'a> TakeIn<'a> for ImportSpecifier<'a> {}

impl<'a> TakeIn<'a> for ImportDefaultSpecifier<'a> {}

impl<'a> TakeIn<'a> for ImportNamespaceSpecifier<'a> {}

impl<'a> TakeIn<'a> for WithClause<'a> {}

impl<'a> TakeIn<'a> for ImportAttribute<'a> {}

impl<'a> TakeIn<'a> for ImportAttributeKey<'a> {}

impl<'a> TakeIn<'a> for ExportNamedDeclaration<'a> {}

impl<'a> TakeIn<'a> for ExportDefaultDeclaration<'a> {}

impl<'a> TakeIn<'a> for ExportAllDeclaration<'a> {}

impl<'a> TakeIn<'a> for ExportSpecifier<'a> {}

impl<'a> TakeIn<'a> for ExportDefaultDeclarationKind<'a> {}

impl<'a> TakeIn<'a> for ModuleExportName<'a> {}

impl<'a> TakeIn<'a> for V8IntrinsicExpression<'a> {}

impl<'a> TakeIn<'a> for BooleanLiteral {}

impl<'a> TakeIn<'a> for NullLiteral {}

impl<'a> TakeIn<'a> for NumericLiteral<'a> {}

impl<'a> TakeIn<'a> for StringLiteral<'a> {}

impl<'a> TakeIn<'a> for BigIntLiteral<'a> {}

impl<'a> TakeIn<'a> for RegExpLiteral<'a> {}

impl<'a> TakeIn<'a> for RegExp<'a> {}

impl<'a> TakeIn<'a> for RegExpPattern<'a> {}

impl<'a> TakeIn<'a> for JSXElement<'a> {}

impl<'a> TakeIn<'a> for JSXOpeningElement<'a> {}

impl<'a> TakeIn<'a> for JSXClosingElement<'a> {}

impl<'a> TakeIn<'a> for JSXFragment<'a> {}

impl<'a> TakeIn<'a> for JSXOpeningFragment {}

impl<'a> TakeIn<'a> for JSXClosingFragment {}

impl<'a> TakeIn<'a> for JSXElementName<'a> {}

impl<'a> TakeIn<'a> for JSXNamespacedName<'a> {}

impl<'a> TakeIn<'a> for JSXMemberExpression<'a> {}

impl<'a> TakeIn<'a> for JSXMemberExpressionObject<'a> {}

impl<'a> TakeIn<'a> for JSXExpressionContainer<'a> {}

impl<'a> TakeIn<'a> for JSXExpression<'a> {}

impl<'a> TakeIn<'a> for JSXEmptyExpression {}

impl<'a> TakeIn<'a> for JSXAttributeItem<'a> {}

impl<'a> TakeIn<'a> for JSXAttribute<'a> {}

impl<'a> TakeIn<'a> for JSXSpreadAttribute<'a> {}

impl<'a> TakeIn<'a> for JSXAttributeName<'a> {}

impl<'a> TakeIn<'a> for JSXAttributeValue<'a> {}

impl<'a> TakeIn<'a> for JSXIdentifier<'a> {}

impl<'a> TakeIn<'a> for JSXChild<'a> {}

impl<'a> TakeIn<'a> for JSXSpreadChild<'a> {}

impl<'a> TakeIn<'a> for JSXText<'a> {}

impl<'a> TakeIn<'a> for TSThisParameter<'a> {}

impl<'a> TakeIn<'a> for TSEnumDeclaration<'a> {}

impl<'a> TakeIn<'a> for TSEnumBody<'a> {}

impl<'a> TakeIn<'a> for TSEnumMember<'a> {}

impl<'a> TakeIn<'a> for TSEnumMemberName<'a> {}

impl<'a> TakeIn<'a> for TSTypeAnnotation<'a> {}

impl<'a> TakeIn<'a> for TSLiteralType<'a> {}

impl<'a> TakeIn<'a> for TSLiteral<'a> {}

impl<'a> TakeIn<'a> for TSType<'a> {}

impl<'a> TakeIn<'a> for TSConditionalType<'a> {}

impl<'a> TakeIn<'a> for TSUnionType<'a> {}

impl<'a> TakeIn<'a> for TSIntersectionType<'a> {}

impl<'a> TakeIn<'a> for TSParenthesizedType<'a> {}

impl<'a> TakeIn<'a> for TSTypeOperator<'a> {}

impl<'a> TakeIn<'a> for TSArrayType<'a> {}

impl<'a> TakeIn<'a> for TSIndexedAccessType<'a> {}

impl<'a> TakeIn<'a> for TSTupleType<'a> {}

impl<'a> TakeIn<'a> for TSNamedTupleMember<'a> {}

impl<'a> TakeIn<'a> for TSOptionalType<'a> {}

impl<'a> TakeIn<'a> for TSRestType<'a> {}

impl<'a> TakeIn<'a> for TSTupleElement<'a> {}

impl<'a> TakeIn<'a> for TSAnyKeyword {}

impl<'a> TakeIn<'a> for TSStringKeyword {}

impl<'a> TakeIn<'a> for TSBooleanKeyword {}

impl<'a> TakeIn<'a> for TSNumberKeyword {}

impl<'a> TakeIn<'a> for TSNeverKeyword {}

impl<'a> TakeIn<'a> for TSIntrinsicKeyword {}

impl<'a> TakeIn<'a> for TSUnknownKeyword {}

impl<'a> TakeIn<'a> for TSNullKeyword {}

impl<'a> TakeIn<'a> for TSUndefinedKeyword {}

impl<'a> TakeIn<'a> for TSVoidKeyword {}

impl<'a> TakeIn<'a> for TSSymbolKeyword {}

impl<'a> TakeIn<'a> for TSThisType {}

impl<'a> TakeIn<'a> for TSObjectKeyword {}

impl<'a> TakeIn<'a> for TSBigIntKeyword {}

impl<'a> TakeIn<'a> for TSTypeReference<'a> {}

impl<'a> TakeIn<'a> for TSTypeName<'a> {}

impl<'a> TakeIn<'a> for TSQualifiedName<'a> {}

impl<'a> TakeIn<'a> for TSTypeParameterInstantiation<'a> {}

impl<'a> TakeIn<'a> for TSTypeParameter<'a> {}

impl<'a> TakeIn<'a> for TSTypeParameterDeclaration<'a> {}

impl<'a> TakeIn<'a> for TSTypeAliasDeclaration<'a> {}

impl<'a> TakeIn<'a> for TSClassImplements<'a> {}

impl<'a> TakeIn<'a> for TSInterfaceDeclaration<'a> {}

impl<'a> TakeIn<'a> for TSInterfaceBody<'a> {}

impl<'a> TakeIn<'a> for TSPropertySignature<'a> {}

impl<'a> TakeIn<'a> for TSSignature<'a> {}

impl<'a> TakeIn<'a> for TSIndexSignature<'a> {}

impl<'a> TakeIn<'a> for TSCallSignatureDeclaration<'a> {}

impl<'a> TakeIn<'a> for TSMethodSignature<'a> {}

impl<'a> TakeIn<'a> for TSConstructSignatureDeclaration<'a> {}

impl<'a> TakeIn<'a> for TSIndexSignatureName<'a> {}

impl<'a> TakeIn<'a> for TSInterfaceHeritage<'a> {}

impl<'a> TakeIn<'a> for TSTypePredicate<'a> {}

impl<'a> TakeIn<'a> for TSTypePredicateName<'a> {}

impl<'a> TakeIn<'a> for TSModuleDeclaration<'a> {}

impl<'a> TakeIn<'a> for TSModuleDeclarationName<'a> {}

impl<'a> TakeIn<'a> for TSModuleDeclarationBody<'a> {}

impl<'a> TakeIn<'a> for TSModuleBlock<'a> {}

impl<'a> TakeIn<'a> for TSTypeLiteral<'a> {}

impl<'a> TakeIn<'a> for TSInferType<'a> {}

impl<'a> TakeIn<'a> for TSTypeQuery<'a> {}

impl<'a> TakeIn<'a> for TSTypeQueryExprName<'a> {}

impl<'a> TakeIn<'a> for TSImportType<'a> {}

impl<'a> TakeIn<'a> for TSImportTypeQualifier<'a> {}

impl<'a> TakeIn<'a> for TSImportTypeQualifiedName<'a> {}

impl<'a> TakeIn<'a> for TSFunctionType<'a> {}

impl<'a> TakeIn<'a> for TSConstructorType<'a> {}

impl<'a> TakeIn<'a> for TSMappedType<'a> {}

impl<'a> TakeIn<'a> for TSTemplateLiteralType<'a> {}

impl<'a> TakeIn<'a> for TSAsExpression<'a> {}

impl<'a> TakeIn<'a> for TSSatisfiesExpression<'a> {}

impl<'a> TakeIn<'a> for TSTypeAssertion<'a> {}

impl<'a> TakeIn<'a> for TSImportEqualsDeclaration<'a> {}

impl<'a> TakeIn<'a> for TSModuleReference<'a> {}

impl<'a> TakeIn<'a> for TSExternalModuleReference<'a> {}

impl<'a> TakeIn<'a> for TSNonNullExpression<'a> {}

impl<'a> TakeIn<'a> for Decorator<'a> {}

impl<'a> TakeIn<'a> for TSExportAssignment<'a> {}

impl<'a> TakeIn<'a> for TSNamespaceExportDeclaration<'a> {}

impl<'a> TakeIn<'a> for TSInstantiationExpression<'a> {}

impl<'a> TakeIn<'a> for JSDocNullableType<'a> {}

impl<'a> TakeIn<'a> for JSDocNonNullableType<'a> {}

impl<'a> TakeIn<'a> for JSDocUnknownType {}
