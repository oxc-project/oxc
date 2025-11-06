// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/unstable_address.rs`.

use oxc_allocator::UnstableAddress;

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl UnstableAddress for Program<'_> {}

impl UnstableAddress for IdentifierName<'_> {}

impl UnstableAddress for IdentifierReference<'_> {}

impl UnstableAddress for BindingIdentifier<'_> {}

impl UnstableAddress for LabelIdentifier<'_> {}

impl UnstableAddress for ThisExpression {}

impl UnstableAddress for ArrayExpression<'_> {}

impl UnstableAddress for Elision {}

impl UnstableAddress for ObjectExpression<'_> {}

impl UnstableAddress for ObjectProperty<'_> {}

impl UnstableAddress for TemplateLiteral<'_> {}

impl UnstableAddress for TaggedTemplateExpression<'_> {}

impl UnstableAddress for TemplateElement<'_> {}

impl UnstableAddress for ComputedMemberExpression<'_> {}

impl UnstableAddress for StaticMemberExpression<'_> {}

impl UnstableAddress for PrivateFieldExpression<'_> {}

impl UnstableAddress for CallExpression<'_> {}

impl UnstableAddress for NewExpression<'_> {}

impl UnstableAddress for MetaProperty<'_> {}

impl UnstableAddress for SpreadElement<'_> {}

impl UnstableAddress for UpdateExpression<'_> {}

impl UnstableAddress for UnaryExpression<'_> {}

impl UnstableAddress for BinaryExpression<'_> {}

impl UnstableAddress for PrivateInExpression<'_> {}

impl UnstableAddress for LogicalExpression<'_> {}

impl UnstableAddress for ConditionalExpression<'_> {}

impl UnstableAddress for AssignmentExpression<'_> {}

impl UnstableAddress for ArrayAssignmentTarget<'_> {}

impl UnstableAddress for ObjectAssignmentTarget<'_> {}

impl UnstableAddress for AssignmentTargetRest<'_> {}

impl UnstableAddress for AssignmentTargetWithDefault<'_> {}

impl UnstableAddress for AssignmentTargetPropertyIdentifier<'_> {}

impl UnstableAddress for AssignmentTargetPropertyProperty<'_> {}

impl UnstableAddress for SequenceExpression<'_> {}

impl UnstableAddress for Super {}

impl UnstableAddress for AwaitExpression<'_> {}

impl UnstableAddress for ChainExpression<'_> {}

impl UnstableAddress for ParenthesizedExpression<'_> {}

impl UnstableAddress for Directive<'_> {}

impl UnstableAddress for Hashbang<'_> {}

impl UnstableAddress for BlockStatement<'_> {}

impl UnstableAddress for VariableDeclaration<'_> {}

impl UnstableAddress for VariableDeclarator<'_> {}

impl UnstableAddress for EmptyStatement {}

impl UnstableAddress for ExpressionStatement<'_> {}

impl UnstableAddress for IfStatement<'_> {}

impl UnstableAddress for DoWhileStatement<'_> {}

impl UnstableAddress for WhileStatement<'_> {}

impl UnstableAddress for ForStatement<'_> {}

impl UnstableAddress for ForInStatement<'_> {}

impl UnstableAddress for ForOfStatement<'_> {}

impl UnstableAddress for ContinueStatement<'_> {}

impl UnstableAddress for BreakStatement<'_> {}

impl UnstableAddress for ReturnStatement<'_> {}

impl UnstableAddress for WithStatement<'_> {}

impl UnstableAddress for SwitchStatement<'_> {}

impl UnstableAddress for SwitchCase<'_> {}

impl UnstableAddress for LabeledStatement<'_> {}

impl UnstableAddress for ThrowStatement<'_> {}

impl UnstableAddress for TryStatement<'_> {}

impl UnstableAddress for CatchClause<'_> {}

impl UnstableAddress for CatchParameter<'_> {}

impl UnstableAddress for DebuggerStatement {}

impl UnstableAddress for BindingPattern<'_> {}

impl UnstableAddress for AssignmentPattern<'_> {}

impl UnstableAddress for ObjectPattern<'_> {}

impl UnstableAddress for BindingProperty<'_> {}

impl UnstableAddress for ArrayPattern<'_> {}

impl UnstableAddress for BindingRestElement<'_> {}

impl UnstableAddress for Function<'_> {}

impl UnstableAddress for FormalParameters<'_> {}

impl UnstableAddress for FormalParameter<'_> {}

impl UnstableAddress for FunctionBody<'_> {}

impl UnstableAddress for ArrowFunctionExpression<'_> {}

impl UnstableAddress for YieldExpression<'_> {}

impl UnstableAddress for Class<'_> {}

impl UnstableAddress for ClassBody<'_> {}

impl UnstableAddress for MethodDefinition<'_> {}

impl UnstableAddress for PropertyDefinition<'_> {}

impl UnstableAddress for PrivateIdentifier<'_> {}

impl UnstableAddress for StaticBlock<'_> {}

impl UnstableAddress for AccessorProperty<'_> {}

impl UnstableAddress for ImportExpression<'_> {}

impl UnstableAddress for ImportDeclaration<'_> {}

impl UnstableAddress for ImportSpecifier<'_> {}

impl UnstableAddress for ImportDefaultSpecifier<'_> {}

impl UnstableAddress for ImportNamespaceSpecifier<'_> {}

impl UnstableAddress for WithClause<'_> {}

impl UnstableAddress for ImportAttribute<'_> {}

impl UnstableAddress for ExportNamedDeclaration<'_> {}

impl UnstableAddress for ExportDefaultDeclaration<'_> {}

impl UnstableAddress for ExportAllDeclaration<'_> {}

impl UnstableAddress for ExportSpecifier<'_> {}

impl UnstableAddress for V8IntrinsicExpression<'_> {}

impl UnstableAddress for BooleanLiteral {}

impl UnstableAddress for NullLiteral {}

impl UnstableAddress for NumericLiteral<'_> {}

impl UnstableAddress for StringLiteral<'_> {}

impl UnstableAddress for BigIntLiteral<'_> {}

impl UnstableAddress for RegExpLiteral<'_> {}

impl UnstableAddress for JSXElement<'_> {}

impl UnstableAddress for JSXOpeningElement<'_> {}

impl UnstableAddress for JSXClosingElement<'_> {}

impl UnstableAddress for JSXFragment<'_> {}

impl UnstableAddress for JSXOpeningFragment {}

impl UnstableAddress for JSXClosingFragment {}

impl UnstableAddress for JSXNamespacedName<'_> {}

impl UnstableAddress for JSXMemberExpression<'_> {}

impl UnstableAddress for JSXExpressionContainer<'_> {}

impl UnstableAddress for JSXEmptyExpression {}

impl UnstableAddress for JSXAttribute<'_> {}

impl UnstableAddress for JSXSpreadAttribute<'_> {}

impl UnstableAddress for JSXIdentifier<'_> {}

impl UnstableAddress for JSXSpreadChild<'_> {}

impl UnstableAddress for JSXText<'_> {}

impl UnstableAddress for TSThisParameter<'_> {}

impl UnstableAddress for TSEnumDeclaration<'_> {}

impl UnstableAddress for TSEnumBody<'_> {}

impl UnstableAddress for TSEnumMember<'_> {}

impl UnstableAddress for TSTypeAnnotation<'_> {}

impl UnstableAddress for TSLiteralType<'_> {}

impl UnstableAddress for TSConditionalType<'_> {}

impl UnstableAddress for TSUnionType<'_> {}

impl UnstableAddress for TSIntersectionType<'_> {}

impl UnstableAddress for TSParenthesizedType<'_> {}

impl UnstableAddress for TSTypeOperator<'_> {}

impl UnstableAddress for TSArrayType<'_> {}

impl UnstableAddress for TSIndexedAccessType<'_> {}

impl UnstableAddress for TSTupleType<'_> {}

impl UnstableAddress for TSNamedTupleMember<'_> {}

impl UnstableAddress for TSOptionalType<'_> {}

impl UnstableAddress for TSRestType<'_> {}

impl UnstableAddress for TSAnyKeyword {}

impl UnstableAddress for TSStringKeyword {}

impl UnstableAddress for TSBooleanKeyword {}

impl UnstableAddress for TSNumberKeyword {}

impl UnstableAddress for TSNeverKeyword {}

impl UnstableAddress for TSIntrinsicKeyword {}

impl UnstableAddress for TSUnknownKeyword {}

impl UnstableAddress for TSNullKeyword {}

impl UnstableAddress for TSUndefinedKeyword {}

impl UnstableAddress for TSVoidKeyword {}

impl UnstableAddress for TSSymbolKeyword {}

impl UnstableAddress for TSThisType {}

impl UnstableAddress for TSObjectKeyword {}

impl UnstableAddress for TSBigIntKeyword {}

impl UnstableAddress for TSTypeReference<'_> {}

impl UnstableAddress for TSQualifiedName<'_> {}

impl UnstableAddress for TSTypeParameterInstantiation<'_> {}

impl UnstableAddress for TSTypeParameter<'_> {}

impl UnstableAddress for TSTypeParameterDeclaration<'_> {}

impl UnstableAddress for TSTypeAliasDeclaration<'_> {}

impl UnstableAddress for TSClassImplements<'_> {}

impl UnstableAddress for TSInterfaceDeclaration<'_> {}

impl UnstableAddress for TSInterfaceBody<'_> {}

impl UnstableAddress for TSPropertySignature<'_> {}

impl UnstableAddress for TSIndexSignature<'_> {}

impl UnstableAddress for TSCallSignatureDeclaration<'_> {}

impl UnstableAddress for TSMethodSignature<'_> {}

impl UnstableAddress for TSConstructSignatureDeclaration<'_> {}

impl UnstableAddress for TSIndexSignatureName<'_> {}

impl UnstableAddress for TSInterfaceHeritage<'_> {}

impl UnstableAddress for TSTypePredicate<'_> {}

impl UnstableAddress for TSModuleDeclaration<'_> {}

impl UnstableAddress for TSGlobalDeclaration<'_> {}

impl UnstableAddress for TSModuleBlock<'_> {}

impl UnstableAddress for TSTypeLiteral<'_> {}

impl UnstableAddress for TSInferType<'_> {}

impl UnstableAddress for TSTypeQuery<'_> {}

impl UnstableAddress for TSImportType<'_> {}

impl UnstableAddress for TSImportTypeQualifiedName<'_> {}

impl UnstableAddress for TSFunctionType<'_> {}

impl UnstableAddress for TSConstructorType<'_> {}

impl UnstableAddress for TSMappedType<'_> {}

impl UnstableAddress for TSTemplateLiteralType<'_> {}

impl UnstableAddress for TSAsExpression<'_> {}

impl UnstableAddress for TSSatisfiesExpression<'_> {}

impl UnstableAddress for TSTypeAssertion<'_> {}

impl UnstableAddress for TSImportEqualsDeclaration<'_> {}

impl UnstableAddress for TSExternalModuleReference<'_> {}

impl UnstableAddress for TSNonNullExpression<'_> {}

impl UnstableAddress for Decorator<'_> {}

impl UnstableAddress for TSExportAssignment<'_> {}

impl UnstableAddress for TSNamespaceExportDeclaration<'_> {}

impl UnstableAddress for TSInstantiationExpression<'_> {}

impl UnstableAddress for JSDocNullableType<'_> {}

impl UnstableAddress for JSDocNonNullableType<'_> {}

impl UnstableAddress for JSDocUnknownType {}
