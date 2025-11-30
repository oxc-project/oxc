# Exit code
1

# stdout
```
  x language-options-plugin(lang): languageOptions:
  | sourceType: script
  | ecmaVersion: 2026
  | parserOptions: {"sourceType":"script"}
  | globals: null
   ,-[files/index.cjs:1:1]
 1 | let x;
   : ^
   `----

  x language-options-plugin(lang): parser:
  | object keys: name,version,parse,VisitorKeys,Syntax,latestEcmaVersion,supportedEcmaVersions
  | name: oxc
  | typeof version: string
  | typeof parse: function
  | latestEcmaVersion: 17
  | supportedEcmaVersions: 3,5,6,7,8,9,10,11,12,13,14,15,16,17
  | Syntax: {
  |   "DebuggerStatement": "DebuggerStatement",
  |   "EmptyStatement": "EmptyStatement",
  |   "Literal": "Literal",
  |   "PrivateIdentifier": "PrivateIdentifier",
  |   "Super": "Super",
  |   "TemplateElement": "TemplateElement",
  |   "ThisExpression": "ThisExpression",
  |   "JSXClosingFragment": "JSXClosingFragment",
  |   "JSXEmptyExpression": "JSXEmptyExpression",
  |   "JSXIdentifier": "JSXIdentifier",
  |   "JSXOpeningFragment": "JSXOpeningFragment",
  |   "JSXText": "JSXText",
  |   "TSAnyKeyword": "TSAnyKeyword",
  |   "TSBigIntKeyword": "TSBigIntKeyword",
  |   "TSBooleanKeyword": "TSBooleanKeyword",
  |   "TSIntrinsicKeyword": "TSIntrinsicKeyword",
  |   "TSJSDocUnknownType": "TSJSDocUnknownType",
  |   "TSNeverKeyword": "TSNeverKeyword",
  |   "TSNullKeyword": "TSNullKeyword",
  |   "TSNumberKeyword": "TSNumberKeyword",
  |   "TSObjectKeyword": "TSObjectKeyword",
  |   "TSStringKeyword": "TSStringKeyword",
  |   "TSSymbolKeyword": "TSSymbolKeyword",
  |   "TSThisType": "TSThisType",
  |   "TSUndefinedKeyword": "TSUndefinedKeyword",
  |   "TSUnknownKeyword": "TSUnknownKeyword",
  |   "TSVoidKeyword": "TSVoidKeyword",
  |   "AccessorProperty": "AccessorProperty",
  |   "ArrayExpression": "ArrayExpression",
  |   "ArrayPattern": "ArrayPattern",
  |   "ArrowFunctionExpression": "ArrowFunctionExpression",
  |   "AssignmentExpression": "AssignmentExpression",
  |   "AssignmentPattern": "AssignmentPattern",
  |   "AwaitExpression": "AwaitExpression",
  |   "BinaryExpression": "BinaryExpression",
  |   "BlockStatement": "BlockStatement",
  |   "BreakStatement": "BreakStatement",
  |   "CallExpression": "CallExpression",
  |   "CatchClause": "CatchClause",
  |   "ChainExpression": "ChainExpression",
  |   "ClassBody": "ClassBody",
  |   "ClassDeclaration": "ClassDeclaration",
  |   "ClassExpression": "ClassExpression",
  |   "ConditionalExpression": "ConditionalExpression",
  |   "ContinueStatement": "ContinueStatement",
  |   "Decorator": "Decorator",
  |   "DoWhileStatement": "DoWhileStatement",
  |   "ExportAllDeclaration": "ExportAllDeclaration",
  |   "ExportDefaultDeclaration": "ExportDefaultDeclaration",
  |   "ExportNamedDeclaration": "ExportNamedDeclaration",
  |   "ExportSpecifier": "ExportSpecifier",
  |   "ExpressionStatement": "ExpressionStatement",
  |   "ForInStatement": "ForInStatement",
  |   "ForOfStatement": "ForOfStatement",
  |   "ForStatement": "ForStatement",
  |   "FunctionDeclaration": "FunctionDeclaration",
  |   "FunctionExpression": "FunctionExpression",
  |   "Identifier": "Identifier",
  |   "IfStatement": "IfStatement",
  |   "ImportAttribute": "ImportAttribute",
  |   "ImportDeclaration": "ImportDeclaration",
  |   "ImportDefaultSpecifier": "ImportDefaultSpecifier",
  |   "ImportExpression": "ImportExpression",
  |   "ImportNamespaceSpecifier": "ImportNamespaceSpecifier",
  |   "ImportSpecifier": "ImportSpecifier",
  |   "LabeledStatement": "LabeledStatement",
  |   "LogicalExpression": "LogicalExpression",
  |   "MemberExpression": "MemberExpression",
  |   "MetaProperty": "MetaProperty",
  |   "MethodDefinition": "MethodDefinition",
  |   "NewExpression": "NewExpression",
  |   "ObjectExpression": "ObjectExpression",
  |   "ObjectPattern": "ObjectPattern",
  |   "ParenthesizedExpression": "ParenthesizedExpression",
  |   "Program": "Program",
  |   "Property": "Property",
  |   "PropertyDefinition": "PropertyDefinition",
  |   "RestElement": "RestElement",
  |   "ReturnStatement": "ReturnStatement",
  |   "SequenceExpression": "SequenceExpression",
  |   "SpreadElement": "SpreadElement",
  |   "StaticBlock": "StaticBlock",
  |   "SwitchCase": "SwitchCase",
  |   "SwitchStatement": "SwitchStatement",
  |   "TaggedTemplateExpression": "TaggedTemplateExpression",
  |   "TemplateLiteral": "TemplateLiteral",
  |   "ThrowStatement": "ThrowStatement",
  |   "TryStatement": "TryStatement",
  |   "UnaryExpression": "UnaryExpression",
  |   "UpdateExpression": "UpdateExpression",
  |   "V8IntrinsicExpression": "V8IntrinsicExpression",
  |   "VariableDeclaration": "VariableDeclaration",
  |   "VariableDeclarator": "VariableDeclarator",
  |   "WhileStatement": "WhileStatement",
  |   "WithStatement": "WithStatement",
  |   "YieldExpression": "YieldExpression",
  |   "JSXAttribute": "JSXAttribute",
  |   "JSXClosingElement": "JSXClosingElement",
  |   "JSXElement": "JSXElement",
  |   "JSXExpressionContainer": "JSXExpressionContainer",
  |   "JSXFragment": "JSXFragment",
  |   "JSXMemberExpression": "JSXMemberExpression",
  |   "JSXNamespacedName": "JSXNamespacedName",
  |   "JSXOpeningElement": "JSXOpeningElement",
  |   "JSXSpreadAttribute": "JSXSpreadAttribute",
  |   "JSXSpreadChild": "JSXSpreadChild",
  |   "TSAbstractAccessorProperty": "TSAbstractAccessorProperty",
  |   "TSAbstractMethodDefinition": "TSAbstractMethodDefinition",
  |   "TSAbstractPropertyDefinition": "TSAbstractPropertyDefinition",
  |   "TSArrayType": "TSArrayType",
  |   "TSAsExpression": "TSAsExpression",
  |   "TSCallSignatureDeclaration": "TSCallSignatureDeclaration",
  |   "TSClassImplements": "TSClassImplements",
  |   "TSConditionalType": "TSConditionalType",
  |   "TSConstructSignatureDeclaration": "TSConstructSignatureDeclaration",
  |   "TSConstructorType": "TSConstructorType",
  |   "TSDeclareFunction": "TSDeclareFunction",
  |   "TSEmptyBodyFunctionExpression": "TSEmptyBodyFunctionExpression",
  |   "TSEnumBody": "TSEnumBody",
  |   "TSEnumDeclaration": "TSEnumDeclaration",
  |   "TSEnumMember": "TSEnumMember",
  |   "TSExportAssignment": "TSExportAssignment",
  |   "TSExternalModuleReference": "TSExternalModuleReference",
  |   "TSFunctionType": "TSFunctionType",
  |   "TSImportEqualsDeclaration": "TSImportEqualsDeclaration",
  |   "TSImportType": "TSImportType",
  |   "TSIndexSignature": "TSIndexSignature",
  |   "TSIndexedAccessType": "TSIndexedAccessType",
  |   "TSInferType": "TSInferType",
  |   "TSInstantiationExpression": "TSInstantiationExpression",
  |   "TSInterfaceBody": "TSInterfaceBody",
  |   "TSInterfaceDeclaration": "TSInterfaceDeclaration",
  |   "TSInterfaceHeritage": "TSInterfaceHeritage",
  |   "TSIntersectionType": "TSIntersectionType",
  |   "TSJSDocNonNullableType": "TSJSDocNonNullableType",
  |   "TSJSDocNullableType": "TSJSDocNullableType",
  |   "TSLiteralType": "TSLiteralType",
  |   "TSMappedType": "TSMappedType",
  |   "TSMethodSignature": "TSMethodSignature",
  |   "TSModuleBlock": "TSModuleBlock",
  |   "TSModuleDeclaration": "TSModuleDeclaration",
  |   "TSNamedTupleMember": "TSNamedTupleMember",
  |   "TSNamespaceExportDeclaration": "TSNamespaceExportDeclaration",
  |   "TSNonNullExpression": "TSNonNullExpression",
  |   "TSOptionalType": "TSOptionalType",
  |   "TSParameterProperty": "TSParameterProperty",
  |   "TSParenthesizedType": "TSParenthesizedType",
  |   "TSPropertySignature": "TSPropertySignature",
  |   "TSQualifiedName": "TSQualifiedName",
  |   "TSRestType": "TSRestType",
  |   "TSSatisfiesExpression": "TSSatisfiesExpression",
  |   "TSTemplateLiteralType": "TSTemplateLiteralType",
  |   "TSTupleType": "TSTupleType",
  |   "TSTypeAliasDeclaration": "TSTypeAliasDeclaration",
  |   "TSTypeAnnotation": "TSTypeAnnotation",
  |   "TSTypeAssertion": "TSTypeAssertion",
  |   "TSTypeLiteral": "TSTypeLiteral",
  |   "TSTypeOperator": "TSTypeOperator",
  |   "TSTypeParameter": "TSTypeParameter",
  |   "TSTypeParameterDeclaration": "TSTypeParameterDeclaration",
  |   "TSTypeParameterInstantiation": "TSTypeParameterInstantiation",
  |   "TSTypePredicate": "TSTypePredicate",
  |   "TSTypeQuery": "TSTypeQuery",
  |   "TSTypeReference": "TSTypeReference",
  |   "TSUnionType": "TSUnionType"
  | }
  | VisitorKeys: {
  |   "DebuggerStatement": [],
  |   "EmptyStatement": [],
  |   "Literal": [],
  |   "PrivateIdentifier": [],
  |   "Super": [],
  |   "TemplateElement": [],
  |   "ThisExpression": [],
  |   "JSXClosingFragment": [],
  |   "JSXEmptyExpression": [],
  |   "JSXIdentifier": [],
  |   "JSXOpeningFragment": [],
  |   "JSXText": [],
  |   "TSAnyKeyword": [],
  |   "TSBigIntKeyword": [],
  |   "TSBooleanKeyword": [],
  |   "TSIntrinsicKeyword": [],
  |   "TSJSDocUnknownType": [],
  |   "TSNeverKeyword": [],
  |   "TSNullKeyword": [],
  |   "TSNumberKeyword": [],
  |   "TSObjectKeyword": [],
  |   "TSStringKeyword": [],
  |   "TSSymbolKeyword": [],
  |   "TSThisType": [],
  |   "TSUndefinedKeyword": [],
  |   "TSUnknownKeyword": [],
  |   "TSVoidKeyword": [],
  |   "AccessorProperty": [
  |     "decorators",
  |     "key",
  |     "typeAnnotation",
  |     "value"
  |   ],
  |   "ArrayExpression": [
  |     "elements"
  |   ],
  |   "ArrayPattern": [
  |     "decorators",
  |     "elements",
  |     "typeAnnotation"
  |   ],
  |   "ArrowFunctionExpression": [
  |     "typeParameters",
  |     "params",
  |     "returnType",
  |     "body"
  |   ],
  |   "AssignmentExpression": [
  |     "left",
  |     "right"
  |   ],
  |   "AssignmentPattern": [
  |     "decorators",
  |     "left",
  |     "right",
  |     "typeAnnotation"
  |   ],
  |   "AwaitExpression": [
  |     "argument"
  |   ],
  |   "BinaryExpression": [
  |     "left",
  |     "right"
  |   ],
  |   "BlockStatement": [
  |     "body"
  |   ],
  |   "BreakStatement": [
  |     "label"
  |   ],
  |   "CallExpression": [
  |     "callee",
  |     "typeArguments",
  |     "arguments"
  |   ],
  |   "CatchClause": [
  |     "param",
  |     "body"
  |   ],
  |   "ChainExpression": [
  |     "expression"
  |   ],
  |   "ClassBody": [
  |     "body"
  |   ],
  |   "ClassDeclaration": [
  |     "decorators",
  |     "id",
  |     "typeParameters",
  |     "superClass",
  |     "superTypeArguments",
  |     "implements",
  |     "body"
  |   ],
  |   "ClassExpression": [
  |     "decorators",
  |     "id",
  |     "typeParameters",
  |     "superClass",
  |     "superTypeArguments",
  |     "implements",
  |     "body"
  |   ],
  |   "ConditionalExpression": [
  |     "test",
  |     "consequent",
  |     "alternate"
  |   ],
  |   "ContinueStatement": [
  |     "label"
  |   ],
  |   "Decorator": [
  |     "expression"
  |   ],
  |   "DoWhileStatement": [
  |     "body",
  |     "test"
  |   ],
  |   "ExportAllDeclaration": [
  |     "exported",
  |     "source",
  |     "attributes"
  |   ],
  |   "ExportDefaultDeclaration": [
  |     "declaration"
  |   ],
  |   "ExportNamedDeclaration": [
  |     "declaration",
  |     "specifiers",
  |     "source",
  |     "attributes"
  |   ],
  |   "ExportSpecifier": [
  |     "local",
  |     "exported"
  |   ],
  |   "ExpressionStatement": [
  |     "expression"
  |   ],
  |   "ForInStatement": [
  |     "left",
  |     "right",
  |     "body"
  |   ],
  |   "ForOfStatement": [
  |     "left",
  |     "right",
  |     "body"
  |   ],
  |   "ForStatement": [
  |     "init",
  |     "test",
  |     "update",
  |     "body"
  |   ],
  |   "FunctionDeclaration": [
  |     "id",
  |     "typeParameters",
  |     "params",
  |     "returnType",
  |     "body"
  |   ],
  |   "FunctionExpression": [
  |     "id",
  |     "typeParameters",
  |     "params",
  |     "returnType",
  |     "body"
  |   ],
  |   "Identifier": [
  |     "decorators",
  |     "typeAnnotation"
  |   ],
  |   "IfStatement": [
  |     "test",
  |     "consequent",
  |     "alternate"
  |   ],
  |   "ImportAttribute": [
  |     "key",
  |     "value"
  |   ],
  |   "ImportDeclaration": [
  |     "specifiers",
  |     "source",
  |     "attributes"
  |   ],
  |   "ImportDefaultSpecifier": [
  |     "local"
  |   ],
  |   "ImportExpression": [
  |     "source",
  |     "options"
  |   ],
  |   "ImportNamespaceSpecifier": [
  |     "local"
  |   ],
  |   "ImportSpecifier": [
  |     "imported",
  |     "local"
  |   ],
  |   "LabeledStatement": [
  |     "label",
  |     "body"
  |   ],
  |   "LogicalExpression": [
  |     "left",
  |     "right"
  |   ],
  |   "MemberExpression": [
  |     "object",
  |     "property"
  |   ],
  |   "MetaProperty": [
  |     "meta",
  |     "property"
  |   ],
  |   "MethodDefinition": [
  |     "decorators",
  |     "key",
  |     "value"
  |   ],
  |   "NewExpression": [
  |     "callee",
  |     "typeArguments",
  |     "arguments"
  |   ],
  |   "ObjectExpression": [
  |     "properties"
  |   ],
  |   "ObjectPattern": [
  |     "decorators",
  |     "properties",
  |     "typeAnnotation"
  |   ],
  |   "ParenthesizedExpression": [
  |     "expression"
  |   ],
  |   "Program": [
  |     "body"
  |   ],
  |   "Property": [
  |     "key",
  |     "value"
  |   ],
  |   "PropertyDefinition": [
  |     "decorators",
  |     "key",
  |     "typeAnnotation",
  |     "value"
  |   ],
  |   "RestElement": [
  |     "decorators",
  |     "argument",
  |     "typeAnnotation"
  |   ],
  |   "ReturnStatement": [
  |     "argument"
  |   ],
  |   "SequenceExpression": [
  |     "expressions"
  |   ],
  |   "SpreadElement": [
  |     "argument"
  |   ],
  |   "StaticBlock": [
  |     "body"
  |   ],
  |   "SwitchCase": [
  |     "test",
  |     "consequent"
  |   ],
  |   "SwitchStatement": [
  |     "discriminant",
  |     "cases"
  |   ],
  |   "TaggedTemplateExpression": [
  |     "tag",
  |     "typeArguments",
  |     "quasi"
  |   ],
  |   "TemplateLiteral": [
  |     "quasis",
  |     "expressions"
  |   ],
  |   "ThrowStatement": [
  |     "argument"
  |   ],
  |   "TryStatement": [
  |     "block",
  |     "handler",
  |     "finalizer"
  |   ],
  |   "UnaryExpression": [
  |     "argument"
  |   ],
  |   "UpdateExpression": [
  |     "argument"
  |   ],
  |   "V8IntrinsicExpression": [
  |     "name",
  |     "arguments"
  |   ],
  |   "VariableDeclaration": [
  |     "declarations"
  |   ],
  |   "VariableDeclarator": [
  |     "id",
  |     "init"
  |   ],
  |   "WhileStatement": [
  |     "test",
  |     "body"
  |   ],
  |   "WithStatement": [
  |     "object",
  |     "body"
  |   ],
  |   "YieldExpression": [
  |     "argument"
  |   ],
  |   "JSXAttribute": [
  |     "name",
  |     "value"
  |   ],
  |   "JSXClosingElement": [
  |     "name"
  |   ],
  |   "JSXElement": [
  |     "openingElement",
  |     "children",
  |     "closingElement"
  |   ],
  |   "JSXExpressionContainer": [
  |     "expression"
  |   ],
  |   "JSXFragment": [
  |     "openingFragment",
  |     "children",
  |     "closingFragment"
  |   ],
  |   "JSXMemberExpression": [
  |     "object",
  |     "property"
  |   ],
  |   "JSXNamespacedName": [
  |     "namespace",
  |     "name"
  |   ],
  |   "JSXOpeningElement": [
  |     "name",
  |     "typeArguments",
  |     "attributes"
  |   ],
  |   "JSXSpreadAttribute": [
  |     "argument"
  |   ],
  |   "JSXSpreadChild": [
  |     "expression"
  |   ],
  |   "TSAbstractAccessorProperty": [
  |     "decorators",
  |     "key",
  |     "typeAnnotation"
  |   ],
  |   "TSAbstractMethodDefinition": [
  |     "key",
  |     "value"
  |   ],
  |   "TSAbstractPropertyDefinition": [
  |     "decorators",
  |     "key",
  |     "typeAnnotation"
  |   ],
  |   "TSArrayType": [
  |     "elementType"
  |   ],
  |   "TSAsExpression": [
  |     "expression",
  |     "typeAnnotation"
  |   ],
  |   "TSCallSignatureDeclaration": [
  |     "typeParameters",
  |     "params",
  |     "returnType"
  |   ],
  |   "TSClassImplements": [
  |     "expression",
  |     "typeArguments"
  |   ],
  |   "TSConditionalType": [
  |     "checkType",
  |     "extendsType",
  |     "trueType",
  |     "falseType"
  |   ],
  |   "TSConstructSignatureDeclaration": [
  |     "typeParameters",
  |     "params",
  |     "returnType"
  |   ],
  |   "TSConstructorType": [
  |     "typeParameters",
  |     "params",
  |     "returnType"
  |   ],
  |   "TSDeclareFunction": [
  |     "id",
  |     "typeParameters",
  |     "params",
  |     "returnType",
  |     "body"
  |   ],
  |   "TSEmptyBodyFunctionExpression": [
  |     "id",
  |     "typeParameters",
  |     "params",
  |     "returnType"
  |   ],
  |   "TSEnumBody": [
  |     "members"
  |   ],
  |   "TSEnumDeclaration": [
  |     "id",
  |     "body"
  |   ],
  |   "TSEnumMember": [
  |     "id",
  |     "initializer"
  |   ],
  |   "TSExportAssignment": [
  |     "expression"
  |   ],
  |   "TSExternalModuleReference": [
  |     "expression"
  |   ],
  |   "TSFunctionType": [
  |     "typeParameters",
  |     "params",
  |     "returnType"
  |   ],
  |   "TSImportEqualsDeclaration": [
  |     "id",
  |     "moduleReference"
  |   ],
  |   "TSImportType": [
  |     "argument",
  |     "options",
  |     "qualifier",
  |     "typeArguments"
  |   ],
  |   "TSIndexSignature": [
  |     "parameters",
  |     "typeAnnotation"
  |   ],
  |   "TSIndexedAccessType": [
  |     "objectType",
  |     "indexType"
  |   ],
  |   "TSInferType": [
  |     "typeParameter"
  |   ],
  |   "TSInstantiationExpression": [
  |     "expression",
  |     "typeArguments"
  |   ],
  |   "TSInterfaceBody": [
  |     "body"
  |   ],
  |   "TSInterfaceDeclaration": [
  |     "id",
  |     "typeParameters",
  |     "extends",
  |     "body"
  |   ],
  |   "TSInterfaceHeritage": [
  |     "expression",
  |     "typeArguments"
  |   ],
  |   "TSIntersectionType": [
  |     "types"
  |   ],
  |   "TSJSDocNonNullableType": [
  |     "typeAnnotation"
  |   ],
  |   "TSJSDocNullableType": [
  |     "typeAnnotation"
  |   ],
  |   "TSLiteralType": [
  |     "literal"
  |   ],
  |   "TSMappedType": [
  |     "key",
  |     "constraint",
  |     "nameType",
  |     "typeAnnotation"
  |   ],
  |   "TSMethodSignature": [
  |     "key",
  |     "typeParameters",
  |     "params",
  |     "returnType"
  |   ],
  |   "TSModuleBlock": [
  |     "body"
  |   ],
  |   "TSModuleDeclaration": [
  |     "id",
  |     "body"
  |   ],
  |   "TSNamedTupleMember": [
  |     "label",
  |     "elementType"
  |   ],
  |   "TSNamespaceExportDeclaration": [
  |     "id"
  |   ],
  |   "TSNonNullExpression": [
  |     "expression"
  |   ],
  |   "TSOptionalType": [
  |     "typeAnnotation"
  |   ],
  |   "TSParameterProperty": [
  |     "decorators",
  |     "parameter"
  |   ],
  |   "TSParenthesizedType": [
  |     "typeAnnotation"
  |   ],
  |   "TSPropertySignature": [
  |     "key",
  |     "typeAnnotation"
  |   ],
  |   "TSQualifiedName": [
  |     "left",
  |     "right"
  |   ],
  |   "TSRestType": [
  |     "typeAnnotation"
  |   ],
  |   "TSSatisfiesExpression": [
  |     "expression",
  |     "typeAnnotation"
  |   ],
  |   "TSTemplateLiteralType": [
  |     "quasis",
  |     "types"
  |   ],
  |   "TSTupleType": [
  |     "elementTypes"
  |   ],
  |   "TSTypeAliasDeclaration": [
  |     "id",
  |     "typeParameters",
  |     "typeAnnotation"
  |   ],
  |   "TSTypeAnnotation": [
  |     "typeAnnotation"
  |   ],
  |   "TSTypeAssertion": [
  |     "typeAnnotation",
  |     "expression"
  |   ],
  |   "TSTypeLiteral": [
  |     "members"
  |   ],
  |   "TSTypeOperator": [
  |     "typeAnnotation"
  |   ],
  |   "TSTypeParameter": [
  |     "name",
  |     "constraint",
  |     "default"
  |   ],
  |   "TSTypeParameterDeclaration": [
  |     "params"
  |   ],
  |   "TSTypeParameterInstantiation": [
  |     "params"
  |   ],
  |   "TSTypePredicate": [
  |     "parameterName",
  |     "typeAnnotation"
  |   ],
  |   "TSTypeQuery": [
  |     "exprName",
  |     "typeArguments"
  |   ],
  |   "TSTypeReference": [
  |     "typeName",
  |     "typeArguments"
  |   ],
  |   "TSUnionType": [
  |     "types"
  |   ]
  | }
   ,-[files/index.js:1:1]
 1 | let x;
   : ^
   `----

  x language-options-plugin(lang): languageOptions:
  | sourceType: module
  | ecmaVersion: 2026
  | parserOptions: {"sourceType":"module"}
  | globals: null
   ,-[files/index.js:1:1]
 1 | let x;
   : ^
   `----

  x language-options-plugin(lang): languageOptions:
  | sourceType: module
  | ecmaVersion: 2026
  | parserOptions: {"sourceType":"module"}
  | globals: null
   ,-[files/index.mjs:1:1]
 1 | let x;
   : ^
   `----

Found 0 warnings and 4 errors.
Finished in Xms on 3 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
