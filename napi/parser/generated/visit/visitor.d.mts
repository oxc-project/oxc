// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/estree_visit.rs`.

import * as ESTree from '@oxc-project/types';

export interface VisitorObject {
  DebuggerStatement?: (node: ESTree.DebuggerStatement) => void;
  'DebuggerStatement:exit'?: (node: ESTree.DebuggerStatement) => void;
  EmptyStatement?: (node: ESTree.EmptyStatement) => void;
  'EmptyStatement:exit'?: (node: ESTree.EmptyStatement) => void;
  Literal?: (
    node:
      | ESTree.BooleanLiteral
      | ESTree.NullLiteral
      | ESTree.NumericLiteral
      | ESTree.StringLiteral
      | ESTree.BigIntLiteral
      | ESTree.RegExpLiteral,
  ) => void;
  'Literal:exit'?: (
    node:
      | ESTree.BooleanLiteral
      | ESTree.NullLiteral
      | ESTree.NumericLiteral
      | ESTree.StringLiteral
      | ESTree.BigIntLiteral
      | ESTree.RegExpLiteral,
  ) => void;
  PrivateIdentifier?: (node: ESTree.PrivateIdentifier) => void;
  'PrivateIdentifier:exit'?: (node: ESTree.PrivateIdentifier) => void;
  Super?: (node: ESTree.Super) => void;
  'Super:exit'?: (node: ESTree.Super) => void;
  TemplateElement?: (node: ESTree.TemplateElement) => void;
  'TemplateElement:exit'?: (node: ESTree.TemplateElement) => void;
  ThisExpression?: (node: ESTree.ThisExpression) => void;
  'ThisExpression:exit'?: (node: ESTree.ThisExpression) => void;
  JSXClosingFragment?: (node: ESTree.JSXClosingFragment) => void;
  'JSXClosingFragment:exit'?: (node: ESTree.JSXClosingFragment) => void;
  JSXEmptyExpression?: (node: ESTree.JSXEmptyExpression) => void;
  'JSXEmptyExpression:exit'?: (node: ESTree.JSXEmptyExpression) => void;
  JSXIdentifier?: (node: ESTree.JSXIdentifier) => void;
  'JSXIdentifier:exit'?: (node: ESTree.JSXIdentifier) => void;
  JSXOpeningFragment?: (node: ESTree.JSXOpeningFragment) => void;
  'JSXOpeningFragment:exit'?: (node: ESTree.JSXOpeningFragment) => void;
  JSXText?: (node: ESTree.JSXText) => void;
  'JSXText:exit'?: (node: ESTree.JSXText) => void;
  TSAnyKeyword?: (node: ESTree.TSAnyKeyword) => void;
  'TSAnyKeyword:exit'?: (node: ESTree.TSAnyKeyword) => void;
  TSBigIntKeyword?: (node: ESTree.TSBigIntKeyword) => void;
  'TSBigIntKeyword:exit'?: (node: ESTree.TSBigIntKeyword) => void;
  TSBooleanKeyword?: (node: ESTree.TSBooleanKeyword) => void;
  'TSBooleanKeyword:exit'?: (node: ESTree.TSBooleanKeyword) => void;
  TSIntrinsicKeyword?: (node: ESTree.TSIntrinsicKeyword) => void;
  'TSIntrinsicKeyword:exit'?: (node: ESTree.TSIntrinsicKeyword) => void;
  TSJSDocUnknownType?: (node: ESTree.JSDocUnknownType) => void;
  'TSJSDocUnknownType:exit'?: (node: ESTree.JSDocUnknownType) => void;
  TSNeverKeyword?: (node: ESTree.TSNeverKeyword) => void;
  'TSNeverKeyword:exit'?: (node: ESTree.TSNeverKeyword) => void;
  TSNullKeyword?: (node: ESTree.TSNullKeyword) => void;
  'TSNullKeyword:exit'?: (node: ESTree.TSNullKeyword) => void;
  TSNumberKeyword?: (node: ESTree.TSNumberKeyword) => void;
  'TSNumberKeyword:exit'?: (node: ESTree.TSNumberKeyword) => void;
  TSObjectKeyword?: (node: ESTree.TSObjectKeyword) => void;
  'TSObjectKeyword:exit'?: (node: ESTree.TSObjectKeyword) => void;
  TSStringKeyword?: (node: ESTree.TSStringKeyword) => void;
  'TSStringKeyword:exit'?: (node: ESTree.TSStringKeyword) => void;
  TSSymbolKeyword?: (node: ESTree.TSSymbolKeyword) => void;
  'TSSymbolKeyword:exit'?: (node: ESTree.TSSymbolKeyword) => void;
  TSThisType?: (node: ESTree.TSThisType) => void;
  'TSThisType:exit'?: (node: ESTree.TSThisType) => void;
  TSUndefinedKeyword?: (node: ESTree.TSUndefinedKeyword) => void;
  'TSUndefinedKeyword:exit'?: (node: ESTree.TSUndefinedKeyword) => void;
  TSUnknownKeyword?: (node: ESTree.TSUnknownKeyword) => void;
  'TSUnknownKeyword:exit'?: (node: ESTree.TSUnknownKeyword) => void;
  TSVoidKeyword?: (node: ESTree.TSVoidKeyword) => void;
  'TSVoidKeyword:exit'?: (node: ESTree.TSVoidKeyword) => void;
  AccessorProperty?: (node: ESTree.AccessorProperty) => void;
  'AccessorProperty:exit'?: (node: ESTree.AccessorProperty) => void;
  ArrayExpression?: (node: ESTree.ArrayExpression) => void;
  'ArrayExpression:exit'?: (node: ESTree.ArrayExpression) => void;
  ArrayPattern?: (node: ESTree.ArrayPattern) => void;
  'ArrayPattern:exit'?: (node: ESTree.ArrayPattern) => void;
  ArrowFunctionExpression?: (node: ESTree.ArrowFunctionExpression) => void;
  'ArrowFunctionExpression:exit'?: (node: ESTree.ArrowFunctionExpression) => void;
  AssignmentExpression?: (node: ESTree.AssignmentExpression) => void;
  'AssignmentExpression:exit'?: (node: ESTree.AssignmentExpression) => void;
  AssignmentPattern?: (node: ESTree.AssignmentPattern) => void;
  'AssignmentPattern:exit'?: (node: ESTree.AssignmentPattern) => void;
  AwaitExpression?: (node: ESTree.AwaitExpression) => void;
  'AwaitExpression:exit'?: (node: ESTree.AwaitExpression) => void;
  BinaryExpression?: (node: ESTree.BinaryExpression) => void;
  'BinaryExpression:exit'?: (node: ESTree.BinaryExpression) => void;
  BlockStatement?: (node: ESTree.BlockStatement) => void;
  'BlockStatement:exit'?: (node: ESTree.BlockStatement) => void;
  BreakStatement?: (node: ESTree.BreakStatement) => void;
  'BreakStatement:exit'?: (node: ESTree.BreakStatement) => void;
  CallExpression?: (node: ESTree.CallExpression) => void;
  'CallExpression:exit'?: (node: ESTree.CallExpression) => void;
  CatchClause?: (node: ESTree.CatchClause) => void;
  'CatchClause:exit'?: (node: ESTree.CatchClause) => void;
  ChainExpression?: (node: ESTree.ChainExpression) => void;
  'ChainExpression:exit'?: (node: ESTree.ChainExpression) => void;
  ClassBody?: (node: ESTree.ClassBody) => void;
  'ClassBody:exit'?: (node: ESTree.ClassBody) => void;
  ClassDeclaration?: (node: ESTree.Class) => void;
  'ClassDeclaration:exit'?: (node: ESTree.Class) => void;
  ClassExpression?: (node: ESTree.Class) => void;
  'ClassExpression:exit'?: (node: ESTree.Class) => void;
  ConditionalExpression?: (node: ESTree.ConditionalExpression) => void;
  'ConditionalExpression:exit'?: (node: ESTree.ConditionalExpression) => void;
  ContinueStatement?: (node: ESTree.ContinueStatement) => void;
  'ContinueStatement:exit'?: (node: ESTree.ContinueStatement) => void;
  Decorator?: (node: ESTree.Decorator) => void;
  'Decorator:exit'?: (node: ESTree.Decorator) => void;
  DoWhileStatement?: (node: ESTree.DoWhileStatement) => void;
  'DoWhileStatement:exit'?: (node: ESTree.DoWhileStatement) => void;
  ExportAllDeclaration?: (node: ESTree.ExportAllDeclaration) => void;
  'ExportAllDeclaration:exit'?: (node: ESTree.ExportAllDeclaration) => void;
  ExportDefaultDeclaration?: (node: ESTree.ExportDefaultDeclaration) => void;
  'ExportDefaultDeclaration:exit'?: (node: ESTree.ExportDefaultDeclaration) => void;
  ExportNamedDeclaration?: (node: ESTree.ExportNamedDeclaration) => void;
  'ExportNamedDeclaration:exit'?: (node: ESTree.ExportNamedDeclaration) => void;
  ExportSpecifier?: (node: ESTree.ExportSpecifier) => void;
  'ExportSpecifier:exit'?: (node: ESTree.ExportSpecifier) => void;
  ExpressionStatement?: (node: ESTree.ExpressionStatement) => void;
  'ExpressionStatement:exit'?: (node: ESTree.ExpressionStatement) => void;
  ForInStatement?: (node: ESTree.ForInStatement) => void;
  'ForInStatement:exit'?: (node: ESTree.ForInStatement) => void;
  ForOfStatement?: (node: ESTree.ForOfStatement) => void;
  'ForOfStatement:exit'?: (node: ESTree.ForOfStatement) => void;
  ForStatement?: (node: ESTree.ForStatement) => void;
  'ForStatement:exit'?: (node: ESTree.ForStatement) => void;
  FunctionDeclaration?: (node: ESTree.Function) => void;
  'FunctionDeclaration:exit'?: (node: ESTree.Function) => void;
  FunctionExpression?: (node: ESTree.Function) => void;
  'FunctionExpression:exit'?: (node: ESTree.Function) => void;
  Identifier?: (
    node:
      | ESTree.IdentifierName
      | ESTree.IdentifierReference
      | ESTree.BindingIdentifier
      | ESTree.LabelIdentifier
      | ESTree.TSThisParameter
      | ESTree.TSIndexSignatureName,
  ) => void;
  'Identifier:exit'?: (
    node:
      | ESTree.IdentifierName
      | ESTree.IdentifierReference
      | ESTree.BindingIdentifier
      | ESTree.LabelIdentifier
      | ESTree.TSThisParameter
      | ESTree.TSIndexSignatureName,
  ) => void;
  IfStatement?: (node: ESTree.IfStatement) => void;
  'IfStatement:exit'?: (node: ESTree.IfStatement) => void;
  ImportAttribute?: (node: ESTree.ImportAttribute) => void;
  'ImportAttribute:exit'?: (node: ESTree.ImportAttribute) => void;
  ImportDeclaration?: (node: ESTree.ImportDeclaration) => void;
  'ImportDeclaration:exit'?: (node: ESTree.ImportDeclaration) => void;
  ImportDefaultSpecifier?: (node: ESTree.ImportDefaultSpecifier) => void;
  'ImportDefaultSpecifier:exit'?: (node: ESTree.ImportDefaultSpecifier) => void;
  ImportExpression?: (node: ESTree.ImportExpression) => void;
  'ImportExpression:exit'?: (node: ESTree.ImportExpression) => void;
  ImportNamespaceSpecifier?: (node: ESTree.ImportNamespaceSpecifier) => void;
  'ImportNamespaceSpecifier:exit'?: (node: ESTree.ImportNamespaceSpecifier) => void;
  ImportSpecifier?: (node: ESTree.ImportSpecifier) => void;
  'ImportSpecifier:exit'?: (node: ESTree.ImportSpecifier) => void;
  LabeledStatement?: (node: ESTree.LabeledStatement) => void;
  'LabeledStatement:exit'?: (node: ESTree.LabeledStatement) => void;
  LogicalExpression?: (node: ESTree.LogicalExpression) => void;
  'LogicalExpression:exit'?: (node: ESTree.LogicalExpression) => void;
  MemberExpression?: (node: ESTree.MemberExpression) => void;
  'MemberExpression:exit'?: (node: ESTree.MemberExpression) => void;
  MetaProperty?: (node: ESTree.MetaProperty) => void;
  'MetaProperty:exit'?: (node: ESTree.MetaProperty) => void;
  MethodDefinition?: (node: ESTree.MethodDefinition) => void;
  'MethodDefinition:exit'?: (node: ESTree.MethodDefinition) => void;
  NewExpression?: (node: ESTree.NewExpression) => void;
  'NewExpression:exit'?: (node: ESTree.NewExpression) => void;
  ObjectExpression?: (node: ESTree.ObjectExpression) => void;
  'ObjectExpression:exit'?: (node: ESTree.ObjectExpression) => void;
  ObjectPattern?: (node: ESTree.ObjectPattern) => void;
  'ObjectPattern:exit'?: (node: ESTree.ObjectPattern) => void;
  ParenthesizedExpression?: (node: ESTree.ParenthesizedExpression) => void;
  'ParenthesizedExpression:exit'?: (node: ESTree.ParenthesizedExpression) => void;
  Program?: (node: ESTree.Program) => void;
  'Program:exit'?: (node: ESTree.Program) => void;
  Property?: (
    node:
      | ESTree.ObjectProperty
      | ESTree.AssignmentTargetProperty
      | ESTree.AssignmentTargetPropertyProperty
      | ESTree.BindingProperty,
  ) => void;
  'Property:exit'?: (
    node:
      | ESTree.ObjectProperty
      | ESTree.AssignmentTargetProperty
      | ESTree.AssignmentTargetPropertyProperty
      | ESTree.BindingProperty,
  ) => void;
  PropertyDefinition?: (node: ESTree.PropertyDefinition) => void;
  'PropertyDefinition:exit'?: (node: ESTree.PropertyDefinition) => void;
  RestElement?: (node: ESTree.AssignmentTargetRest | ESTree.BindingRestElement | ESTree.FormalParameterRest) => void;
  'RestElement:exit'?: (
    node: ESTree.AssignmentTargetRest | ESTree.BindingRestElement | ESTree.FormalParameterRest,
  ) => void;
  ReturnStatement?: (node: ESTree.ReturnStatement) => void;
  'ReturnStatement:exit'?: (node: ESTree.ReturnStatement) => void;
  SequenceExpression?: (node: ESTree.SequenceExpression) => void;
  'SequenceExpression:exit'?: (node: ESTree.SequenceExpression) => void;
  SpreadElement?: (node: ESTree.SpreadElement) => void;
  'SpreadElement:exit'?: (node: ESTree.SpreadElement) => void;
  StaticBlock?: (node: ESTree.StaticBlock) => void;
  'StaticBlock:exit'?: (node: ESTree.StaticBlock) => void;
  SwitchCase?: (node: ESTree.SwitchCase) => void;
  'SwitchCase:exit'?: (node: ESTree.SwitchCase) => void;
  SwitchStatement?: (node: ESTree.SwitchStatement) => void;
  'SwitchStatement:exit'?: (node: ESTree.SwitchStatement) => void;
  TaggedTemplateExpression?: (node: ESTree.TaggedTemplateExpression) => void;
  'TaggedTemplateExpression:exit'?: (node: ESTree.TaggedTemplateExpression) => void;
  TemplateLiteral?: (node: ESTree.TemplateLiteral) => void;
  'TemplateLiteral:exit'?: (node: ESTree.TemplateLiteral) => void;
  ThrowStatement?: (node: ESTree.ThrowStatement) => void;
  'ThrowStatement:exit'?: (node: ESTree.ThrowStatement) => void;
  TryStatement?: (node: ESTree.TryStatement) => void;
  'TryStatement:exit'?: (node: ESTree.TryStatement) => void;
  UnaryExpression?: (node: ESTree.UnaryExpression) => void;
  'UnaryExpression:exit'?: (node: ESTree.UnaryExpression) => void;
  UpdateExpression?: (node: ESTree.UpdateExpression) => void;
  'UpdateExpression:exit'?: (node: ESTree.UpdateExpression) => void;
  V8IntrinsicExpression?: (node: ESTree.V8IntrinsicExpression) => void;
  'V8IntrinsicExpression:exit'?: (node: ESTree.V8IntrinsicExpression) => void;
  VariableDeclaration?: (node: ESTree.VariableDeclaration) => void;
  'VariableDeclaration:exit'?: (node: ESTree.VariableDeclaration) => void;
  VariableDeclarator?: (node: ESTree.VariableDeclarator) => void;
  'VariableDeclarator:exit'?: (node: ESTree.VariableDeclarator) => void;
  WhileStatement?: (node: ESTree.WhileStatement) => void;
  'WhileStatement:exit'?: (node: ESTree.WhileStatement) => void;
  WithStatement?: (node: ESTree.WithStatement) => void;
  'WithStatement:exit'?: (node: ESTree.WithStatement) => void;
  YieldExpression?: (node: ESTree.YieldExpression) => void;
  'YieldExpression:exit'?: (node: ESTree.YieldExpression) => void;
  JSXAttribute?: (node: ESTree.JSXAttribute) => void;
  'JSXAttribute:exit'?: (node: ESTree.JSXAttribute) => void;
  JSXClosingElement?: (node: ESTree.JSXClosingElement) => void;
  'JSXClosingElement:exit'?: (node: ESTree.JSXClosingElement) => void;
  JSXElement?: (node: ESTree.JSXElement) => void;
  'JSXElement:exit'?: (node: ESTree.JSXElement) => void;
  JSXExpressionContainer?: (node: ESTree.JSXExpressionContainer) => void;
  'JSXExpressionContainer:exit'?: (node: ESTree.JSXExpressionContainer) => void;
  JSXFragment?: (node: ESTree.JSXFragment) => void;
  'JSXFragment:exit'?: (node: ESTree.JSXFragment) => void;
  JSXMemberExpression?: (node: ESTree.JSXMemberExpression) => void;
  'JSXMemberExpression:exit'?: (node: ESTree.JSXMemberExpression) => void;
  JSXNamespacedName?: (node: ESTree.JSXNamespacedName) => void;
  'JSXNamespacedName:exit'?: (node: ESTree.JSXNamespacedName) => void;
  JSXOpeningElement?: (node: ESTree.JSXOpeningElement) => void;
  'JSXOpeningElement:exit'?: (node: ESTree.JSXOpeningElement) => void;
  JSXSpreadAttribute?: (node: ESTree.JSXSpreadAttribute) => void;
  'JSXSpreadAttribute:exit'?: (node: ESTree.JSXSpreadAttribute) => void;
  JSXSpreadChild?: (node: ESTree.JSXSpreadChild) => void;
  'JSXSpreadChild:exit'?: (node: ESTree.JSXSpreadChild) => void;
  TSAbstractAccessorProperty?: (node: ESTree.AccessorProperty) => void;
  'TSAbstractAccessorProperty:exit'?: (node: ESTree.AccessorProperty) => void;
  TSAbstractMethodDefinition?: (node: ESTree.MethodDefinition) => void;
  'TSAbstractMethodDefinition:exit'?: (node: ESTree.MethodDefinition) => void;
  TSAbstractPropertyDefinition?: (node: ESTree.PropertyDefinition) => void;
  'TSAbstractPropertyDefinition:exit'?: (node: ESTree.PropertyDefinition) => void;
  TSArrayType?: (node: ESTree.TSArrayType) => void;
  'TSArrayType:exit'?: (node: ESTree.TSArrayType) => void;
  TSAsExpression?: (node: ESTree.TSAsExpression) => void;
  'TSAsExpression:exit'?: (node: ESTree.TSAsExpression) => void;
  TSCallSignatureDeclaration?: (node: ESTree.TSCallSignatureDeclaration) => void;
  'TSCallSignatureDeclaration:exit'?: (node: ESTree.TSCallSignatureDeclaration) => void;
  TSClassImplements?: (node: ESTree.TSClassImplements) => void;
  'TSClassImplements:exit'?: (node: ESTree.TSClassImplements) => void;
  TSConditionalType?: (node: ESTree.TSConditionalType) => void;
  'TSConditionalType:exit'?: (node: ESTree.TSConditionalType) => void;
  TSConstructSignatureDeclaration?: (node: ESTree.TSConstructSignatureDeclaration) => void;
  'TSConstructSignatureDeclaration:exit'?: (node: ESTree.TSConstructSignatureDeclaration) => void;
  TSConstructorType?: (node: ESTree.TSConstructorType) => void;
  'TSConstructorType:exit'?: (node: ESTree.TSConstructorType) => void;
  TSDeclareFunction?: (node: ESTree.Function) => void;
  'TSDeclareFunction:exit'?: (node: ESTree.Function) => void;
  TSEmptyBodyFunctionExpression?: (node: ESTree.Function) => void;
  'TSEmptyBodyFunctionExpression:exit'?: (node: ESTree.Function) => void;
  TSEnumBody?: (node: ESTree.TSEnumBody) => void;
  'TSEnumBody:exit'?: (node: ESTree.TSEnumBody) => void;
  TSEnumDeclaration?: (node: ESTree.TSEnumDeclaration) => void;
  'TSEnumDeclaration:exit'?: (node: ESTree.TSEnumDeclaration) => void;
  TSEnumMember?: (node: ESTree.TSEnumMember) => void;
  'TSEnumMember:exit'?: (node: ESTree.TSEnumMember) => void;
  TSExportAssignment?: (node: ESTree.TSExportAssignment) => void;
  'TSExportAssignment:exit'?: (node: ESTree.TSExportAssignment) => void;
  TSExternalModuleReference?: (node: ESTree.TSExternalModuleReference) => void;
  'TSExternalModuleReference:exit'?: (node: ESTree.TSExternalModuleReference) => void;
  TSFunctionType?: (node: ESTree.TSFunctionType) => void;
  'TSFunctionType:exit'?: (node: ESTree.TSFunctionType) => void;
  TSImportEqualsDeclaration?: (node: ESTree.TSImportEqualsDeclaration) => void;
  'TSImportEqualsDeclaration:exit'?: (node: ESTree.TSImportEqualsDeclaration) => void;
  TSImportType?: (node: ESTree.TSImportType) => void;
  'TSImportType:exit'?: (node: ESTree.TSImportType) => void;
  TSIndexSignature?: (node: ESTree.TSIndexSignature) => void;
  'TSIndexSignature:exit'?: (node: ESTree.TSIndexSignature) => void;
  TSIndexedAccessType?: (node: ESTree.TSIndexedAccessType) => void;
  'TSIndexedAccessType:exit'?: (node: ESTree.TSIndexedAccessType) => void;
  TSInferType?: (node: ESTree.TSInferType) => void;
  'TSInferType:exit'?: (node: ESTree.TSInferType) => void;
  TSInstantiationExpression?: (node: ESTree.TSInstantiationExpression) => void;
  'TSInstantiationExpression:exit'?: (node: ESTree.TSInstantiationExpression) => void;
  TSInterfaceBody?: (node: ESTree.TSInterfaceBody) => void;
  'TSInterfaceBody:exit'?: (node: ESTree.TSInterfaceBody) => void;
  TSInterfaceDeclaration?: (node: ESTree.TSInterfaceDeclaration) => void;
  'TSInterfaceDeclaration:exit'?: (node: ESTree.TSInterfaceDeclaration) => void;
  TSInterfaceHeritage?: (node: ESTree.TSInterfaceHeritage) => void;
  'TSInterfaceHeritage:exit'?: (node: ESTree.TSInterfaceHeritage) => void;
  TSIntersectionType?: (node: ESTree.TSIntersectionType) => void;
  'TSIntersectionType:exit'?: (node: ESTree.TSIntersectionType) => void;
  TSJSDocNonNullableType?: (node: ESTree.JSDocNonNullableType) => void;
  'TSJSDocNonNullableType:exit'?: (node: ESTree.JSDocNonNullableType) => void;
  TSJSDocNullableType?: (node: ESTree.JSDocNullableType) => void;
  'TSJSDocNullableType:exit'?: (node: ESTree.JSDocNullableType) => void;
  TSLiteralType?: (node: ESTree.TSLiteralType) => void;
  'TSLiteralType:exit'?: (node: ESTree.TSLiteralType) => void;
  TSMappedType?: (node: ESTree.TSMappedType) => void;
  'TSMappedType:exit'?: (node: ESTree.TSMappedType) => void;
  TSMethodSignature?: (node: ESTree.TSMethodSignature) => void;
  'TSMethodSignature:exit'?: (node: ESTree.TSMethodSignature) => void;
  TSModuleBlock?: (node: ESTree.TSModuleBlock) => void;
  'TSModuleBlock:exit'?: (node: ESTree.TSModuleBlock) => void;
  TSModuleDeclaration?: (node: ESTree.TSModuleDeclaration) => void;
  'TSModuleDeclaration:exit'?: (node: ESTree.TSModuleDeclaration) => void;
  TSNamedTupleMember?: (node: ESTree.TSNamedTupleMember) => void;
  'TSNamedTupleMember:exit'?: (node: ESTree.TSNamedTupleMember) => void;
  TSNamespaceExportDeclaration?: (node: ESTree.TSNamespaceExportDeclaration) => void;
  'TSNamespaceExportDeclaration:exit'?: (node: ESTree.TSNamespaceExportDeclaration) => void;
  TSNonNullExpression?: (node: ESTree.TSNonNullExpression) => void;
  'TSNonNullExpression:exit'?: (node: ESTree.TSNonNullExpression) => void;
  TSOptionalType?: (node: ESTree.TSOptionalType) => void;
  'TSOptionalType:exit'?: (node: ESTree.TSOptionalType) => void;
  TSParameterProperty?: (node: ESTree.TSParameterProperty) => void;
  'TSParameterProperty:exit'?: (node: ESTree.TSParameterProperty) => void;
  TSParenthesizedType?: (node: ESTree.TSParenthesizedType) => void;
  'TSParenthesizedType:exit'?: (node: ESTree.TSParenthesizedType) => void;
  TSPropertySignature?: (node: ESTree.TSPropertySignature) => void;
  'TSPropertySignature:exit'?: (node: ESTree.TSPropertySignature) => void;
  TSQualifiedName?: (node: ESTree.TSQualifiedName) => void;
  'TSQualifiedName:exit'?: (node: ESTree.TSQualifiedName) => void;
  TSRestType?: (node: ESTree.TSRestType) => void;
  'TSRestType:exit'?: (node: ESTree.TSRestType) => void;
  TSSatisfiesExpression?: (node: ESTree.TSSatisfiesExpression) => void;
  'TSSatisfiesExpression:exit'?: (node: ESTree.TSSatisfiesExpression) => void;
  TSTemplateLiteralType?: (node: ESTree.TSTemplateLiteralType) => void;
  'TSTemplateLiteralType:exit'?: (node: ESTree.TSTemplateLiteralType) => void;
  TSTupleType?: (node: ESTree.TSTupleType) => void;
  'TSTupleType:exit'?: (node: ESTree.TSTupleType) => void;
  TSTypeAliasDeclaration?: (node: ESTree.TSTypeAliasDeclaration) => void;
  'TSTypeAliasDeclaration:exit'?: (node: ESTree.TSTypeAliasDeclaration) => void;
  TSTypeAnnotation?: (node: ESTree.TSTypeAnnotation) => void;
  'TSTypeAnnotation:exit'?: (node: ESTree.TSTypeAnnotation) => void;
  TSTypeAssertion?: (node: ESTree.TSTypeAssertion) => void;
  'TSTypeAssertion:exit'?: (node: ESTree.TSTypeAssertion) => void;
  TSTypeLiteral?: (node: ESTree.TSTypeLiteral) => void;
  'TSTypeLiteral:exit'?: (node: ESTree.TSTypeLiteral) => void;
  TSTypeOperator?: (node: ESTree.TSTypeOperator) => void;
  'TSTypeOperator:exit'?: (node: ESTree.TSTypeOperator) => void;
  TSTypeParameter?: (node: ESTree.TSTypeParameter) => void;
  'TSTypeParameter:exit'?: (node: ESTree.TSTypeParameter) => void;
  TSTypeParameterDeclaration?: (node: ESTree.TSTypeParameterDeclaration) => void;
  'TSTypeParameterDeclaration:exit'?: (node: ESTree.TSTypeParameterDeclaration) => void;
  TSTypeParameterInstantiation?: (node: ESTree.TSTypeParameterInstantiation) => void;
  'TSTypeParameterInstantiation:exit'?: (node: ESTree.TSTypeParameterInstantiation) => void;
  TSTypePredicate?: (node: ESTree.TSTypePredicate) => void;
  'TSTypePredicate:exit'?: (node: ESTree.TSTypePredicate) => void;
  TSTypeQuery?: (node: ESTree.TSTypeQuery) => void;
  'TSTypeQuery:exit'?: (node: ESTree.TSTypeQuery) => void;
  TSTypeReference?: (node: ESTree.TSTypeReference) => void;
  'TSTypeReference:exit'?: (node: ESTree.TSTypeReference) => void;
  TSUnionType?: (node: ESTree.TSUnionType) => void;
  'TSUnionType:exit'?: (node: ESTree.TSUnionType) => void;
}
