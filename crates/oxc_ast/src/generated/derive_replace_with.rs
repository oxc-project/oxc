// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/replace_with.rs`.

use oxc_allocator::ReplaceWith;

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl ReplaceWith for Program<'_> {}

impl ReplaceWith for Expression<'_> {}

impl ReplaceWith for IdentifierName<'_> {}

impl ReplaceWith for IdentifierReference<'_> {}

impl ReplaceWith for BindingIdentifier<'_> {}

impl ReplaceWith for LabelIdentifier<'_> {}

impl ReplaceWith for ThisExpression {}

impl ReplaceWith for ArrayExpression<'_> {}

impl ReplaceWith for ArrayExpressionElement<'_> {}

impl ReplaceWith for Elision {}

impl ReplaceWith for ObjectExpression<'_> {}

impl ReplaceWith for ObjectPropertyKind<'_> {}

impl ReplaceWith for ObjectProperty<'_> {}

impl ReplaceWith for PropertyKey<'_> {}

impl ReplaceWith for TemplateLiteral<'_> {}

impl ReplaceWith for TaggedTemplateExpression<'_> {}

impl ReplaceWith for TemplateElement<'_> {}

impl ReplaceWith for TemplateElementValue<'_> {}

impl ReplaceWith for MemberExpression<'_> {}

impl ReplaceWith for ComputedMemberExpression<'_> {}

impl ReplaceWith for StaticMemberExpression<'_> {}

impl ReplaceWith for PrivateFieldExpression<'_> {}

impl ReplaceWith for CallExpression<'_> {}

impl ReplaceWith for NewExpression<'_> {}

impl ReplaceWith for MetaProperty<'_> {}

impl ReplaceWith for SpreadElement<'_> {}

impl ReplaceWith for Argument<'_> {}

impl ReplaceWith for UpdateExpression<'_> {}

impl ReplaceWith for UnaryExpression<'_> {}

impl ReplaceWith for BinaryExpression<'_> {}

impl ReplaceWith for PrivateInExpression<'_> {}

impl ReplaceWith for LogicalExpression<'_> {}

impl ReplaceWith for ConditionalExpression<'_> {}

impl ReplaceWith for AssignmentExpression<'_> {}

impl ReplaceWith for AssignmentTarget<'_> {}

impl ReplaceWith for SimpleAssignmentTarget<'_> {}

impl ReplaceWith for AssignmentTargetPattern<'_> {}

impl ReplaceWith for ArrayAssignmentTarget<'_> {}

impl ReplaceWith for ObjectAssignmentTarget<'_> {}

impl ReplaceWith for AssignmentTargetRest<'_> {}

impl ReplaceWith for AssignmentTargetMaybeDefault<'_> {}

impl ReplaceWith for AssignmentTargetWithDefault<'_> {}

impl ReplaceWith for AssignmentTargetProperty<'_> {}

impl ReplaceWith for AssignmentTargetPropertyIdentifier<'_> {}

impl ReplaceWith for AssignmentTargetPropertyProperty<'_> {}

impl ReplaceWith for SequenceExpression<'_> {}

impl ReplaceWith for Super {}

impl ReplaceWith for AwaitExpression<'_> {}

impl ReplaceWith for ChainExpression<'_> {}

impl ReplaceWith for ChainElement<'_> {}

impl ReplaceWith for ParenthesizedExpression<'_> {}

impl ReplaceWith for Statement<'_> {}

impl ReplaceWith for Directive<'_> {}

impl ReplaceWith for Hashbang<'_> {}

impl ReplaceWith for BlockStatement<'_> {}

impl ReplaceWith for Declaration<'_> {}

impl ReplaceWith for VariableDeclaration<'_> {}

impl ReplaceWith for VariableDeclarator<'_> {}

impl ReplaceWith for EmptyStatement {}

impl ReplaceWith for ExpressionStatement<'_> {}

impl ReplaceWith for IfStatement<'_> {}

impl ReplaceWith for DoWhileStatement<'_> {}

impl ReplaceWith for WhileStatement<'_> {}

impl ReplaceWith for ForStatement<'_> {}

impl ReplaceWith for ForStatementInit<'_> {}

impl ReplaceWith for ForInStatement<'_> {}

impl ReplaceWith for ForStatementLeft<'_> {}

impl ReplaceWith for ForOfStatement<'_> {}

impl ReplaceWith for ContinueStatement<'_> {}

impl ReplaceWith for BreakStatement<'_> {}

impl ReplaceWith for ReturnStatement<'_> {}

impl ReplaceWith for WithStatement<'_> {}

impl ReplaceWith for SwitchStatement<'_> {}

impl ReplaceWith for SwitchCase<'_> {}

impl ReplaceWith for LabeledStatement<'_> {}

impl ReplaceWith for ThrowStatement<'_> {}

impl ReplaceWith for TryStatement<'_> {}

impl ReplaceWith for CatchClause<'_> {}

impl ReplaceWith for CatchParameter<'_> {}

impl ReplaceWith for DebuggerStatement {}

impl ReplaceWith for BindingPattern<'_> {}

impl ReplaceWith for AssignmentPattern<'_> {}

impl ReplaceWith for ObjectPattern<'_> {}

impl ReplaceWith for BindingProperty<'_> {}

impl ReplaceWith for ArrayPattern<'_> {}

impl ReplaceWith for BindingRestElement<'_> {}

impl ReplaceWith for Function<'_> {}

impl ReplaceWith for FormalParameters<'_> {}

impl ReplaceWith for FormalParameter<'_> {}

impl ReplaceWith for FormalParameterRest<'_> {}

impl ReplaceWith for FunctionBody<'_> {}

impl ReplaceWith for ArrowFunctionExpression<'_> {}

impl ReplaceWith for YieldExpression<'_> {}

impl ReplaceWith for Class<'_> {}

impl ReplaceWith for ClassBody<'_> {}

impl ReplaceWith for ClassElement<'_> {}

impl ReplaceWith for MethodDefinition<'_> {}

impl ReplaceWith for PropertyDefinition<'_> {}

impl ReplaceWith for PrivateIdentifier<'_> {}

impl ReplaceWith for StaticBlock<'_> {}

impl ReplaceWith for ModuleDeclaration<'_> {}

impl ReplaceWith for AccessorProperty<'_> {}

impl ReplaceWith for ImportExpression<'_> {}

impl ReplaceWith for ImportDeclaration<'_> {}

impl ReplaceWith for ImportDeclarationSpecifier<'_> {}

impl ReplaceWith for ImportSpecifier<'_> {}

impl ReplaceWith for ImportDefaultSpecifier<'_> {}

impl ReplaceWith for ImportNamespaceSpecifier<'_> {}

impl ReplaceWith for WithClause<'_> {}

impl ReplaceWith for ImportAttribute<'_> {}

impl ReplaceWith for ImportAttributeKey<'_> {}

impl ReplaceWith for ExportNamedDeclaration<'_> {}

impl ReplaceWith for ExportDefaultDeclaration<'_> {}

impl ReplaceWith for ExportAllDeclaration<'_> {}

impl ReplaceWith for ExportSpecifier<'_> {}

impl ReplaceWith for ExportDefaultDeclarationKind<'_> {}

impl ReplaceWith for ModuleExportName<'_> {}

impl ReplaceWith for V8IntrinsicExpression<'_> {}

impl ReplaceWith for BooleanLiteral {}

impl ReplaceWith for NullLiteral {}

impl ReplaceWith for NumericLiteral<'_> {}

impl ReplaceWith for StringLiteral<'_> {}

impl ReplaceWith for BigIntLiteral<'_> {}

impl ReplaceWith for RegExpLiteral<'_> {}

impl ReplaceWith for RegExp<'_> {}

impl ReplaceWith for RegExpPattern<'_> {}

impl ReplaceWith for JSXElement<'_> {}

impl ReplaceWith for JSXOpeningElement<'_> {}

impl ReplaceWith for JSXClosingElement<'_> {}

impl ReplaceWith for JSXFragment<'_> {}

impl ReplaceWith for JSXOpeningFragment {}

impl ReplaceWith for JSXClosingFragment {}

impl ReplaceWith for JSXElementName<'_> {}

impl ReplaceWith for JSXNamespacedName<'_> {}

impl ReplaceWith for JSXMemberExpression<'_> {}

impl ReplaceWith for JSXMemberExpressionObject<'_> {}

impl ReplaceWith for JSXExpressionContainer<'_> {}

impl ReplaceWith for JSXExpression<'_> {}

impl ReplaceWith for JSXEmptyExpression {}

impl ReplaceWith for JSXAttributeItem<'_> {}

impl ReplaceWith for JSXAttribute<'_> {}

impl ReplaceWith for JSXSpreadAttribute<'_> {}

impl ReplaceWith for JSXAttributeName<'_> {}

impl ReplaceWith for JSXAttributeValue<'_> {}

impl ReplaceWith for JSXIdentifier<'_> {}

impl ReplaceWith for JSXChild<'_> {}

impl ReplaceWith for JSXSpreadChild<'_> {}

impl ReplaceWith for JSXText<'_> {}

impl ReplaceWith for TSThisParameter<'_> {}

impl ReplaceWith for TSEnumDeclaration<'_> {}

impl ReplaceWith for TSEnumBody<'_> {}

impl ReplaceWith for TSEnumMember<'_> {}

impl ReplaceWith for TSEnumMemberName<'_> {}

impl ReplaceWith for TSTypeAnnotation<'_> {}

impl ReplaceWith for TSLiteralType<'_> {}

impl ReplaceWith for TSLiteral<'_> {}

impl ReplaceWith for TSType<'_> {}

impl ReplaceWith for TSConditionalType<'_> {}

impl ReplaceWith for TSUnionType<'_> {}

impl ReplaceWith for TSIntersectionType<'_> {}

impl ReplaceWith for TSParenthesizedType<'_> {}

impl ReplaceWith for TSTypeOperator<'_> {}

impl ReplaceWith for TSArrayType<'_> {}

impl ReplaceWith for TSIndexedAccessType<'_> {}

impl ReplaceWith for TSTupleType<'_> {}

impl ReplaceWith for TSNamedTupleMember<'_> {}

impl ReplaceWith for TSOptionalType<'_> {}

impl ReplaceWith for TSRestType<'_> {}

impl ReplaceWith for TSTupleElement<'_> {}

impl ReplaceWith for TSAnyKeyword {}

impl ReplaceWith for TSStringKeyword {}

impl ReplaceWith for TSBooleanKeyword {}

impl ReplaceWith for TSNumberKeyword {}

impl ReplaceWith for TSNeverKeyword {}

impl ReplaceWith for TSIntrinsicKeyword {}

impl ReplaceWith for TSUnknownKeyword {}

impl ReplaceWith for TSNullKeyword {}

impl ReplaceWith for TSUndefinedKeyword {}

impl ReplaceWith for TSVoidKeyword {}

impl ReplaceWith for TSSymbolKeyword {}

impl ReplaceWith for TSThisType {}

impl ReplaceWith for TSObjectKeyword {}

impl ReplaceWith for TSBigIntKeyword {}

impl ReplaceWith for TSTypeReference<'_> {}

impl ReplaceWith for TSTypeName<'_> {}

impl ReplaceWith for TSQualifiedName<'_> {}

impl ReplaceWith for TSTypeParameterInstantiation<'_> {}

impl ReplaceWith for TSTypeParameter<'_> {}

impl ReplaceWith for TSTypeParameterDeclaration<'_> {}

impl ReplaceWith for TSTypeAliasDeclaration<'_> {}

impl ReplaceWith for TSClassImplements<'_> {}

impl ReplaceWith for TSInterfaceDeclaration<'_> {}

impl ReplaceWith for TSInterfaceBody<'_> {}

impl ReplaceWith for TSPropertySignature<'_> {}

impl ReplaceWith for TSSignature<'_> {}

impl ReplaceWith for TSIndexSignature<'_> {}

impl ReplaceWith for TSCallSignatureDeclaration<'_> {}

impl ReplaceWith for TSMethodSignature<'_> {}

impl ReplaceWith for TSConstructSignatureDeclaration<'_> {}

impl ReplaceWith for TSIndexSignatureName<'_> {}

impl ReplaceWith for TSInterfaceHeritage<'_> {}

impl ReplaceWith for TSTypePredicate<'_> {}

impl ReplaceWith for TSTypePredicateName<'_> {}

impl ReplaceWith for TSModuleDeclaration<'_> {}

impl ReplaceWith for TSModuleDeclarationName<'_> {}

impl ReplaceWith for TSModuleDeclarationBody<'_> {}

impl ReplaceWith for TSGlobalDeclaration<'_> {}

impl ReplaceWith for TSModuleBlock<'_> {}

impl ReplaceWith for TSTypeLiteral<'_> {}

impl ReplaceWith for TSInferType<'_> {}

impl ReplaceWith for TSTypeQuery<'_> {}

impl ReplaceWith for TSTypeQueryExprName<'_> {}

impl ReplaceWith for TSImportType<'_> {}

impl ReplaceWith for TSImportTypeQualifier<'_> {}

impl ReplaceWith for TSImportTypeQualifiedName<'_> {}

impl ReplaceWith for TSFunctionType<'_> {}

impl ReplaceWith for TSConstructorType<'_> {}

impl ReplaceWith for TSMappedType<'_> {}

impl ReplaceWith for TSTemplateLiteralType<'_> {}

impl ReplaceWith for TSAsExpression<'_> {}

impl ReplaceWith for TSSatisfiesExpression<'_> {}

impl ReplaceWith for TSTypeAssertion<'_> {}

impl ReplaceWith for TSImportEqualsDeclaration<'_> {}

impl ReplaceWith for TSModuleReference<'_> {}

impl ReplaceWith for TSExternalModuleReference<'_> {}

impl ReplaceWith for TSNonNullExpression<'_> {}

impl ReplaceWith for Decorator<'_> {}

impl ReplaceWith for TSExportAssignment<'_> {}

impl ReplaceWith for TSNamespaceExportDeclaration<'_> {}

impl ReplaceWith for TSInstantiationExpression<'_> {}

impl ReplaceWith for JSDocNullableType<'_> {}

impl ReplaceWith for JSDocNonNullableType<'_> {}

impl ReplaceWith for JSDocUnknownType {}
