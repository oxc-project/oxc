// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/replace_with.rs`.

use oxc_allocator::ReplaceWith;

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl<'a> ReplaceWith<'a> for Program<'a> {}

impl<'a> ReplaceWith<'a> for Expression<'a> {}

impl<'a> ReplaceWith<'a> for IdentifierName<'a> {}

impl<'a> ReplaceWith<'a> for IdentifierReference<'a> {}

impl<'a> ReplaceWith<'a> for BindingIdentifier<'a> {}

impl<'a> ReplaceWith<'a> for LabelIdentifier<'a> {}

impl ReplaceWith<'_> for ThisExpression {}

impl<'a> ReplaceWith<'a> for ArrayExpression<'a> {}

impl<'a> ReplaceWith<'a> for ArrayExpressionElement<'a> {}

impl ReplaceWith<'_> for Elision {}

impl<'a> ReplaceWith<'a> for ObjectExpression<'a> {}

impl<'a> ReplaceWith<'a> for ObjectPropertyKind<'a> {}

impl<'a> ReplaceWith<'a> for ObjectProperty<'a> {}

impl<'a> ReplaceWith<'a> for PropertyKey<'a> {}

impl<'a> ReplaceWith<'a> for TemplateLiteral<'a> {}

impl<'a> ReplaceWith<'a> for TaggedTemplateExpression<'a> {}

impl<'a> ReplaceWith<'a> for TemplateElement<'a> {}

impl<'a> ReplaceWith<'a> for TemplateElementValue<'a> {}

impl<'a> ReplaceWith<'a> for MemberExpression<'a> {}

impl<'a> ReplaceWith<'a> for ComputedMemberExpression<'a> {}

impl<'a> ReplaceWith<'a> for StaticMemberExpression<'a> {}

impl<'a> ReplaceWith<'a> for PrivateFieldExpression<'a> {}

impl<'a> ReplaceWith<'a> for CallExpression<'a> {}

impl<'a> ReplaceWith<'a> for NewExpression<'a> {}

impl ReplaceWith<'_> for ImportMeta {}

impl ReplaceWith<'_> for NewTarget {}

impl<'a> ReplaceWith<'a> for SpreadElement<'a> {}

impl<'a> ReplaceWith<'a> for Argument<'a> {}

impl<'a> ReplaceWith<'a> for UpdateExpression<'a> {}

impl<'a> ReplaceWith<'a> for UnaryExpression<'a> {}

impl<'a> ReplaceWith<'a> for BinaryExpression<'a> {}

impl<'a> ReplaceWith<'a> for PrivateInExpression<'a> {}

impl<'a> ReplaceWith<'a> for LogicalExpression<'a> {}

impl<'a> ReplaceWith<'a> for ConditionalExpression<'a> {}

impl<'a> ReplaceWith<'a> for AssignmentExpression<'a> {}

impl<'a> ReplaceWith<'a> for AssignmentTarget<'a> {}

impl<'a> ReplaceWith<'a> for SimpleAssignmentTarget<'a> {}

impl<'a> ReplaceWith<'a> for AssignmentTargetPattern<'a> {}

impl<'a> ReplaceWith<'a> for ArrayAssignmentTarget<'a> {}

impl<'a> ReplaceWith<'a> for ObjectAssignmentTarget<'a> {}

impl<'a> ReplaceWith<'a> for AssignmentTargetRest<'a> {}

impl<'a> ReplaceWith<'a> for AssignmentTargetMaybeDefault<'a> {}

impl<'a> ReplaceWith<'a> for AssignmentTargetWithDefault<'a> {}

impl<'a> ReplaceWith<'a> for AssignmentTargetProperty<'a> {}

impl<'a> ReplaceWith<'a> for AssignmentTargetPropertyIdentifier<'a> {}

impl<'a> ReplaceWith<'a> for AssignmentTargetPropertyProperty<'a> {}

impl<'a> ReplaceWith<'a> for SequenceExpression<'a> {}

impl ReplaceWith<'_> for Super {}

impl<'a> ReplaceWith<'a> for AwaitExpression<'a> {}

impl<'a> ReplaceWith<'a> for ChainExpression<'a> {}

impl<'a> ReplaceWith<'a> for ChainElement<'a> {}

impl<'a> ReplaceWith<'a> for ParenthesizedExpression<'a> {}

impl<'a> ReplaceWith<'a> for Statement<'a> {}

impl<'a> ReplaceWith<'a> for Directive<'a> {}

impl<'a> ReplaceWith<'a> for Hashbang<'a> {}

impl<'a> ReplaceWith<'a> for BlockStatement<'a> {}

impl<'a> ReplaceWith<'a> for Declaration<'a> {}

impl<'a> ReplaceWith<'a> for VariableDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for VariableDeclarator<'a> {}

impl ReplaceWith<'_> for EmptyStatement {}

impl<'a> ReplaceWith<'a> for ExpressionStatement<'a> {}

impl<'a> ReplaceWith<'a> for IfStatement<'a> {}

impl<'a> ReplaceWith<'a> for DoWhileStatement<'a> {}

impl<'a> ReplaceWith<'a> for WhileStatement<'a> {}

impl<'a> ReplaceWith<'a> for ForStatement<'a> {}

impl<'a> ReplaceWith<'a> for ForStatementInit<'a> {}

impl<'a> ReplaceWith<'a> for ForInStatement<'a> {}

impl<'a> ReplaceWith<'a> for ForStatementLeft<'a> {}

impl<'a> ReplaceWith<'a> for ForOfStatement<'a> {}

impl<'a> ReplaceWith<'a> for ContinueStatement<'a> {}

impl<'a> ReplaceWith<'a> for BreakStatement<'a> {}

impl<'a> ReplaceWith<'a> for ReturnStatement<'a> {}

impl<'a> ReplaceWith<'a> for WithStatement<'a> {}

impl<'a> ReplaceWith<'a> for SwitchStatement<'a> {}

impl<'a> ReplaceWith<'a> for SwitchCase<'a> {}

impl<'a> ReplaceWith<'a> for LabeledStatement<'a> {}

impl<'a> ReplaceWith<'a> for ThrowStatement<'a> {}

impl<'a> ReplaceWith<'a> for TryStatement<'a> {}

impl<'a> ReplaceWith<'a> for CatchClause<'a> {}

impl<'a> ReplaceWith<'a> for CatchParameter<'a> {}

impl ReplaceWith<'_> for DebuggerStatement {}

impl<'a> ReplaceWith<'a> for BindingPattern<'a> {}

impl<'a> ReplaceWith<'a> for AssignmentPattern<'a> {}

impl<'a> ReplaceWith<'a> for ObjectPattern<'a> {}

impl<'a> ReplaceWith<'a> for BindingProperty<'a> {}

impl<'a> ReplaceWith<'a> for ArrayPattern<'a> {}

impl<'a> ReplaceWith<'a> for BindingRestElement<'a> {}

impl<'a> ReplaceWith<'a> for Function<'a> {}

impl<'a> ReplaceWith<'a> for FormalParameters<'a> {}

impl<'a> ReplaceWith<'a> for FormalParameter<'a> {}

impl<'a> ReplaceWith<'a> for FormalParameterRest<'a> {}

impl<'a> ReplaceWith<'a> for FunctionBody<'a> {}

impl<'a> ReplaceWith<'a> for ArrowFunctionExpression<'a> {}

impl<'a> ReplaceWith<'a> for YieldExpression<'a> {}

impl<'a> ReplaceWith<'a> for Class<'a> {}

impl<'a> ReplaceWith<'a> for ClassBody<'a> {}

impl<'a> ReplaceWith<'a> for ClassElement<'a> {}

impl<'a> ReplaceWith<'a> for ClassConstructor<'a> {}

impl<'a> ReplaceWith<'a> for ClassConstructorKey<'a> {}

impl<'a> ReplaceWith<'a> for MethodDefinition<'a> {}

impl<'a> ReplaceWith<'a> for PropertyDefinition<'a> {}

impl<'a> ReplaceWith<'a> for PrivateIdentifier<'a> {}

impl<'a> ReplaceWith<'a> for StaticBlock<'a> {}

impl<'a> ReplaceWith<'a> for ModuleDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for AccessorProperty<'a> {}

impl<'a> ReplaceWith<'a> for ImportExpression<'a> {}

impl<'a> ReplaceWith<'a> for ImportDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for ImportDeclarationSpecifier<'a> {}

impl<'a> ReplaceWith<'a> for ImportSpecifier<'a> {}

impl<'a> ReplaceWith<'a> for ImportDefaultSpecifier<'a> {}

impl<'a> ReplaceWith<'a> for ImportNamespaceSpecifier<'a> {}

impl<'a> ReplaceWith<'a> for WithClause<'a> {}

impl<'a> ReplaceWith<'a> for ImportAttribute<'a> {}

impl<'a> ReplaceWith<'a> for ImportAttributeKey<'a> {}

impl<'a> ReplaceWith<'a> for ExportNamedDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for ExportDefaultDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for ExportAllDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for ExportSpecifier<'a> {}

impl<'a> ReplaceWith<'a> for ExportDefaultDeclarationKind<'a> {}

impl<'a> ReplaceWith<'a> for ModuleExportName<'a> {}

impl<'a> ReplaceWith<'a> for V8IntrinsicExpression<'a> {}

impl ReplaceWith<'_> for BooleanLiteral {}

impl ReplaceWith<'_> for NullLiteral {}

impl<'a> ReplaceWith<'a> for NumericLiteral<'a> {}

impl<'a> ReplaceWith<'a> for StringLiteral<'a> {}

impl<'a> ReplaceWith<'a> for BigIntLiteral<'a> {}

impl<'a> ReplaceWith<'a> for RegExpLiteral<'a> {}

impl<'a> ReplaceWith<'a> for RegExp<'a> {}

impl<'a> ReplaceWith<'a> for RegExpPattern<'a> {}

impl<'a> ReplaceWith<'a> for JSXElement<'a> {}

impl<'a> ReplaceWith<'a> for JSXOpeningElement<'a> {}

impl<'a> ReplaceWith<'a> for JSXClosingElement<'a> {}

impl<'a> ReplaceWith<'a> for JSXFragment<'a> {}

impl ReplaceWith<'_> for JSXOpeningFragment {}

impl ReplaceWith<'_> for JSXClosingFragment {}

impl<'a> ReplaceWith<'a> for JSXElementName<'a> {}

impl<'a> ReplaceWith<'a> for JSXNamespacedName<'a> {}

impl<'a> ReplaceWith<'a> for JSXMemberExpression<'a> {}

impl<'a> ReplaceWith<'a> for JSXMemberExpressionObject<'a> {}

impl<'a> ReplaceWith<'a> for JSXExpressionContainer<'a> {}

impl<'a> ReplaceWith<'a> for JSXExpression<'a> {}

impl ReplaceWith<'_> for JSXEmptyExpression {}

impl<'a> ReplaceWith<'a> for JSXAttributeItem<'a> {}

impl<'a> ReplaceWith<'a> for JSXAttribute<'a> {}

impl<'a> ReplaceWith<'a> for JSXSpreadAttribute<'a> {}

impl<'a> ReplaceWith<'a> for JSXAttributeName<'a> {}

impl<'a> ReplaceWith<'a> for JSXAttributeValue<'a> {}

impl<'a> ReplaceWith<'a> for JSXIdentifier<'a> {}

impl<'a> ReplaceWith<'a> for JSXChild<'a> {}

impl<'a> ReplaceWith<'a> for JSXSpreadChild<'a> {}

impl<'a> ReplaceWith<'a> for JSXText<'a> {}

impl<'a> ReplaceWith<'a> for TSThisParameter<'a> {}

impl<'a> ReplaceWith<'a> for TSEnumDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for TSEnumBody<'a> {}

impl<'a> ReplaceWith<'a> for TSEnumMember<'a> {}

impl<'a> ReplaceWith<'a> for TSEnumMemberName<'a> {}

impl<'a> ReplaceWith<'a> for TSTypeAnnotation<'a> {}

impl<'a> ReplaceWith<'a> for TSLiteralType<'a> {}

impl<'a> ReplaceWith<'a> for TSLiteral<'a> {}

impl<'a> ReplaceWith<'a> for TSType<'a> {}

impl<'a> ReplaceWith<'a> for TSConditionalType<'a> {}

impl<'a> ReplaceWith<'a> for TSUnionType<'a> {}

impl<'a> ReplaceWith<'a> for TSIntersectionType<'a> {}

impl<'a> ReplaceWith<'a> for TSParenthesizedType<'a> {}

impl<'a> ReplaceWith<'a> for TSTypeOperator<'a> {}

impl<'a> ReplaceWith<'a> for TSArrayType<'a> {}

impl<'a> ReplaceWith<'a> for TSIndexedAccessType<'a> {}

impl<'a> ReplaceWith<'a> for TSTupleType<'a> {}

impl<'a> ReplaceWith<'a> for TSNamedTupleMember<'a> {}

impl<'a> ReplaceWith<'a> for TSOptionalType<'a> {}

impl<'a> ReplaceWith<'a> for TSRestType<'a> {}

impl<'a> ReplaceWith<'a> for TSTupleElement<'a> {}

impl ReplaceWith<'_> for TSAnyKeyword {}

impl ReplaceWith<'_> for TSStringKeyword {}

impl ReplaceWith<'_> for TSBooleanKeyword {}

impl ReplaceWith<'_> for TSNumberKeyword {}

impl ReplaceWith<'_> for TSNeverKeyword {}

impl ReplaceWith<'_> for TSIntrinsicKeyword {}

impl ReplaceWith<'_> for TSUnknownKeyword {}

impl ReplaceWith<'_> for TSNullKeyword {}

impl ReplaceWith<'_> for TSUndefinedKeyword {}

impl ReplaceWith<'_> for TSVoidKeyword {}

impl ReplaceWith<'_> for TSSymbolKeyword {}

impl ReplaceWith<'_> for TSThisType {}

impl ReplaceWith<'_> for TSObjectKeyword {}

impl ReplaceWith<'_> for TSBigIntKeyword {}

impl<'a> ReplaceWith<'a> for TSTypeReference<'a> {}

impl<'a> ReplaceWith<'a> for TSTypeName<'a> {}

impl<'a> ReplaceWith<'a> for TSQualifiedName<'a> {}

impl<'a> ReplaceWith<'a> for TSTypeParameterInstantiation<'a> {}

impl<'a> ReplaceWith<'a> for TSTypeParameter<'a> {}

impl<'a> ReplaceWith<'a> for TSTypeParameterDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for TSTypeAliasDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for TSClassImplements<'a> {}

impl<'a> ReplaceWith<'a> for TSInterfaceDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for TSInterfaceBody<'a> {}

impl<'a> ReplaceWith<'a> for TSPropertySignature<'a> {}

impl<'a> ReplaceWith<'a> for TSSignature<'a> {}

impl<'a> ReplaceWith<'a> for TSIndexSignature<'a> {}

impl<'a> ReplaceWith<'a> for TSCallSignatureDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for TSMethodSignature<'a> {}

impl<'a> ReplaceWith<'a> for TSConstructSignatureDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for TSIndexSignatureName<'a> {}

impl<'a> ReplaceWith<'a> for TSInterfaceHeritage<'a> {}

impl<'a> ReplaceWith<'a> for TSTypePredicate<'a> {}

impl<'a> ReplaceWith<'a> for TSTypePredicateName<'a> {}

impl<'a> ReplaceWith<'a> for TSModuleDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for TSModuleDeclarationName<'a> {}

impl<'a> ReplaceWith<'a> for TSModuleDeclarationBody<'a> {}

impl<'a> ReplaceWith<'a> for TSGlobalDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for TSModuleBlock<'a> {}

impl<'a> ReplaceWith<'a> for TSTypeLiteral<'a> {}

impl<'a> ReplaceWith<'a> for TSInferType<'a> {}

impl<'a> ReplaceWith<'a> for TSTypeQuery<'a> {}

impl<'a> ReplaceWith<'a> for TSTypeQueryExprName<'a> {}

impl<'a> ReplaceWith<'a> for TSImportType<'a> {}

impl<'a> ReplaceWith<'a> for TSImportTypeQualifier<'a> {}

impl<'a> ReplaceWith<'a> for TSImportTypeQualifiedName<'a> {}

impl<'a> ReplaceWith<'a> for TSFunctionType<'a> {}

impl<'a> ReplaceWith<'a> for TSConstructorType<'a> {}

impl<'a> ReplaceWith<'a> for TSMappedType<'a> {}

impl<'a> ReplaceWith<'a> for TSTemplateLiteralType<'a> {}

impl<'a> ReplaceWith<'a> for TSAsExpression<'a> {}

impl<'a> ReplaceWith<'a> for TSSatisfiesExpression<'a> {}

impl<'a> ReplaceWith<'a> for TSTypeAssertion<'a> {}

impl<'a> ReplaceWith<'a> for TSImportEqualsDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for TSModuleReference<'a> {}

impl<'a> ReplaceWith<'a> for TSExternalModuleReference<'a> {}

impl<'a> ReplaceWith<'a> for TSNonNullExpression<'a> {}

impl<'a> ReplaceWith<'a> for Decorator<'a> {}

impl<'a> ReplaceWith<'a> for TSExportAssignment<'a> {}

impl<'a> ReplaceWith<'a> for TSNamespaceExportDeclaration<'a> {}

impl<'a> ReplaceWith<'a> for TSInstantiationExpression<'a> {}

impl<'a> ReplaceWith<'a> for JSDocNullableType<'a> {}

impl<'a> ReplaceWith<'a> for JSDocNonNullableType<'a> {}

impl ReplaceWith<'_> for JSDocUnknownType {}
