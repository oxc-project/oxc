// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer_lazy.rs`.

'use strict';

const {
  Program,
  IdentifierName,
  IdentifierReference,
  BindingIdentifier,
  LabelIdentifier,
  ThisExpression,
  ArrayExpression,
  Elision,
  ObjectExpression,
  ObjectProperty,
  TemplateLiteral,
  TaggedTemplateExpression,
  TemplateElement,
  ComputedMemberExpression,
  StaticMemberExpression,
  PrivateFieldExpression,
  CallExpression,
  NewExpression,
  MetaProperty,
  SpreadElement,
  UpdateExpression,
  UnaryExpression,
  BinaryExpression,
  PrivateInExpression,
  LogicalExpression,
  ConditionalExpression,
  AssignmentExpression,
  ArrayAssignmentTarget,
  ObjectAssignmentTarget,
  AssignmentTargetWithDefault,
  AssignmentTargetPropertyIdentifier,
  AssignmentTargetPropertyProperty,
  SequenceExpression,
  Super,
  AwaitExpression,
  ChainExpression,
  ParenthesizedExpression,
  Hashbang,
  BlockStatement,
  VariableDeclaration,
  VariableDeclarator,
  EmptyStatement,
  ExpressionStatement,
  IfStatement,
  DoWhileStatement,
  WhileStatement,
  ForStatement,
  ForInStatement,
  ForOfStatement,
  ContinueStatement,
  BreakStatement,
  ReturnStatement,
  WithStatement,
  SwitchStatement,
  SwitchCase,
  LabeledStatement,
  ThrowStatement,
  TryStatement,
  CatchClause,
  DebuggerStatement,
  AssignmentPattern,
  ObjectPattern,
  BindingProperty,
  ArrayPattern,
  Function,
  FormalParameters,
  FunctionBody,
  ArrowFunctionExpression,
  YieldExpression,
  Class,
  ClassBody,
  MethodDefinition,
  PropertyDefinition,
  PrivateIdentifier,
  StaticBlock,
  AccessorProperty,
  ImportExpression,
  ImportDeclaration,
  ImportSpecifier,
  ImportDefaultSpecifier,
  ImportNamespaceSpecifier,
  ImportAttribute,
  ExportNamedDeclaration,
  ExportDefaultDeclaration,
  ExportAllDeclaration,
  ExportSpecifier,
  V8IntrinsicExpression,
  BooleanLiteral,
  NullLiteral,
  NumericLiteral,
  StringLiteral,
  BigIntLiteral,
  RegExpLiteral,
  JSXElement,
  JSXOpeningElement,
  JSXClosingElement,
  JSXFragment,
  JSXOpeningFragment,
  JSXClosingFragment,
  JSXNamespacedName,
  JSXMemberExpression,
  JSXExpressionContainer,
  JSXEmptyExpression,
  JSXAttribute,
  JSXSpreadAttribute,
  JSXIdentifier,
  JSXSpreadChild,
  JSXText,
  TSEnumDeclaration,
  TSEnumBody,
  TSEnumMember,
  TSTypeAnnotation,
  TSLiteralType,
  TSConditionalType,
  TSUnionType,
  TSIntersectionType,
  TSParenthesizedType,
  TSTypeOperator,
  TSArrayType,
  TSIndexedAccessType,
  TSTupleType,
  TSNamedTupleMember,
  TSOptionalType,
  TSRestType,
  TSAnyKeyword,
  TSStringKeyword,
  TSBooleanKeyword,
  TSNumberKeyword,
  TSNeverKeyword,
  TSIntrinsicKeyword,
  TSUnknownKeyword,
  TSNullKeyword,
  TSUndefinedKeyword,
  TSVoidKeyword,
  TSSymbolKeyword,
  TSThisType,
  TSObjectKeyword,
  TSBigIntKeyword,
  TSTypeReference,
  TSQualifiedName,
  TSTypeParameterInstantiation,
  TSTypeParameter,
  TSTypeParameterDeclaration,
  TSTypeAliasDeclaration,
  TSClassImplements,
  TSInterfaceDeclaration,
  TSInterfaceBody,
  TSPropertySignature,
  TSIndexSignature,
  TSCallSignatureDeclaration,
  TSMethodSignature,
  TSConstructSignatureDeclaration,
  TSIndexSignatureName,
  TSInterfaceHeritage,
  TSTypePredicate,
  TSModuleDeclaration,
  TSModuleBlock,
  TSTypeLiteral,
  TSInferType,
  TSTypeQuery,
  TSImportType,
  TSFunctionType,
  TSConstructorType,
  TSMappedType,
  TSTemplateLiteralType,
  TSAsExpression,
  TSSatisfiesExpression,
  TSTypeAssertion,
  TSImportEqualsDeclaration,
  TSExternalModuleReference,
  TSNonNullExpression,
  Decorator,
  TSExportAssignment,
  TSNamespaceExportDeclaration,
  TSInstantiationExpression,
  JSDocNullableType,
  JSDocNonNullableType,
  JSDocUnknownType,
} = require('./lazy.js').constructors;

module.exports = walkProgram;

function walkProgram(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 64) !== 0) {
    node = new Program(pos, ast);
    ({ enter, exit } = visitor.visitors[38]);
    if (enter !== null) enter(node);
  }

  walkOptionHashbang(pos + 48, ast, visitor);
  walkVecStatement(pos + 96, ast, visitor);

  if (exit !== null) exit(node);
}

function walkExpression(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxNullLiteral(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxNumericLiteral(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxBigIntLiteral(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxRegExpLiteral(pos + 8, ast, visitor);
      return;
    case 5:
      walkBoxStringLiteral(pos + 8, ast, visitor);
      return;
    case 6:
      walkBoxTemplateLiteral(pos + 8, ast, visitor);
      return;
    case 7:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 8:
      walkBoxMetaProperty(pos + 8, ast, visitor);
      return;
    case 9:
      walkBoxSuper(pos + 8, ast, visitor);
      return;
    case 10:
      walkBoxArrayExpression(pos + 8, ast, visitor);
      return;
    case 11:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitor);
      return;
    case 12:
      walkBoxAssignmentExpression(pos + 8, ast, visitor);
      return;
    case 13:
      walkBoxAwaitExpression(pos + 8, ast, visitor);
      return;
    case 14:
      walkBoxBinaryExpression(pos + 8, ast, visitor);
      return;
    case 15:
      walkBoxCallExpression(pos + 8, ast, visitor);
      return;
    case 16:
      walkBoxChainExpression(pos + 8, ast, visitor);
      return;
    case 17:
      walkBoxClass(pos + 8, ast, visitor);
      return;
    case 18:
      walkBoxConditionalExpression(pos + 8, ast, visitor);
      return;
    case 19:
      walkBoxFunction(pos + 8, ast, visitor);
      return;
    case 20:
      walkBoxImportExpression(pos + 8, ast, visitor);
      return;
    case 21:
      walkBoxLogicalExpression(pos + 8, ast, visitor);
      return;
    case 22:
      walkBoxNewExpression(pos + 8, ast, visitor);
      return;
    case 23:
      walkBoxObjectExpression(pos + 8, ast, visitor);
      return;
    case 24:
      walkBoxParenthesizedExpression(pos + 8, ast, visitor);
      return;
    case 25:
      walkBoxSequenceExpression(pos + 8, ast, visitor);
      return;
    case 26:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitor);
      return;
    case 27:
      walkBoxThisExpression(pos + 8, ast, visitor);
      return;
    case 28:
      walkBoxUnaryExpression(pos + 8, ast, visitor);
      return;
    case 29:
      walkBoxUpdateExpression(pos + 8, ast, visitor);
      return;
    case 30:
      walkBoxYieldExpression(pos + 8, ast, visitor);
      return;
    case 31:
      walkBoxPrivateInExpression(pos + 8, ast, visitor);
      return;
    case 32:
      walkBoxJSXElement(pos + 8, ast, visitor);
      return;
    case 33:
      walkBoxJSXFragment(pos + 8, ast, visitor);
      return;
    case 34:
      walkBoxTSAsExpression(pos + 8, ast, visitor);
      return;
    case 35:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitor);
      return;
    case 36:
      walkBoxTSTypeAssertion(pos + 8, ast, visitor);
      return;
    case 37:
      walkBoxTSNonNullExpression(pos + 8, ast, visitor);
      return;
    case 38:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitor);
      return;
    case 39:
      walkBoxV8IntrinsicExpression(pos + 8, ast, visitor);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitor);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitor);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Expression`);
  }
}

function walkIdentifierName(pos, ast, visitor) {
  if ((visitor.bitmap0 & 1) !== 0) {
    (0, visitor.visitors[0])(new IdentifierName(pos, ast));
  }
}

function walkIdentifierReference(pos, ast, visitor) {
  if ((visitor.bitmap0 & 2) !== 0) {
    (0, visitor.visitors[1])(new IdentifierReference(pos, ast));
  }
}

function walkBindingIdentifier(pos, ast, visitor) {
  if ((visitor.bitmap0 & 4) !== 0) {
    (0, visitor.visitors[2])(new BindingIdentifier(pos, ast));
  }
}

function walkLabelIdentifier(pos, ast, visitor) {
  if ((visitor.bitmap0 & 8) !== 0) {
    (0, visitor.visitors[3])(new LabelIdentifier(pos, ast));
  }
}

function walkThisExpression(pos, ast, visitor) {
  if ((visitor.bitmap0 & 16) !== 0) {
    (0, visitor.visitors[4])(new ThisExpression(pos, ast));
  }
}

function walkArrayExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 128) !== 0) {
    node = new ArrayExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[39]);
    if (enter !== null) enter(node);
  }

  walkVecArrayExpressionElement(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkArrayExpressionElement(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxNullLiteral(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxNumericLiteral(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxBigIntLiteral(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxRegExpLiteral(pos + 8, ast, visitor);
      return;
    case 5:
      walkBoxStringLiteral(pos + 8, ast, visitor);
      return;
    case 6:
      walkBoxTemplateLiteral(pos + 8, ast, visitor);
      return;
    case 7:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 8:
      walkBoxMetaProperty(pos + 8, ast, visitor);
      return;
    case 9:
      walkBoxSuper(pos + 8, ast, visitor);
      return;
    case 10:
      walkBoxArrayExpression(pos + 8, ast, visitor);
      return;
    case 11:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitor);
      return;
    case 12:
      walkBoxAssignmentExpression(pos + 8, ast, visitor);
      return;
    case 13:
      walkBoxAwaitExpression(pos + 8, ast, visitor);
      return;
    case 14:
      walkBoxBinaryExpression(pos + 8, ast, visitor);
      return;
    case 15:
      walkBoxCallExpression(pos + 8, ast, visitor);
      return;
    case 16:
      walkBoxChainExpression(pos + 8, ast, visitor);
      return;
    case 17:
      walkBoxClass(pos + 8, ast, visitor);
      return;
    case 18:
      walkBoxConditionalExpression(pos + 8, ast, visitor);
      return;
    case 19:
      walkBoxFunction(pos + 8, ast, visitor);
      return;
    case 20:
      walkBoxImportExpression(pos + 8, ast, visitor);
      return;
    case 21:
      walkBoxLogicalExpression(pos + 8, ast, visitor);
      return;
    case 22:
      walkBoxNewExpression(pos + 8, ast, visitor);
      return;
    case 23:
      walkBoxObjectExpression(pos + 8, ast, visitor);
      return;
    case 24:
      walkBoxParenthesizedExpression(pos + 8, ast, visitor);
      return;
    case 25:
      walkBoxSequenceExpression(pos + 8, ast, visitor);
      return;
    case 26:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitor);
      return;
    case 27:
      walkBoxThisExpression(pos + 8, ast, visitor);
      return;
    case 28:
      walkBoxUnaryExpression(pos + 8, ast, visitor);
      return;
    case 29:
      walkBoxUpdateExpression(pos + 8, ast, visitor);
      return;
    case 30:
      walkBoxYieldExpression(pos + 8, ast, visitor);
      return;
    case 31:
      walkBoxPrivateInExpression(pos + 8, ast, visitor);
      return;
    case 32:
      walkBoxJSXElement(pos + 8, ast, visitor);
      return;
    case 33:
      walkBoxJSXFragment(pos + 8, ast, visitor);
      return;
    case 34:
      walkBoxTSAsExpression(pos + 8, ast, visitor);
      return;
    case 35:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitor);
      return;
    case 36:
      walkBoxTSTypeAssertion(pos + 8, ast, visitor);
      return;
    case 37:
      walkBoxTSNonNullExpression(pos + 8, ast, visitor);
      return;
    case 38:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitor);
      return;
    case 39:
      walkBoxV8IntrinsicExpression(pos + 8, ast, visitor);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitor);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitor);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitor);
      return;
    case 64:
      walkBoxSpreadElement(pos + 8, ast, visitor);
      return;
    case 65:
      walkElision(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ArrayExpressionElement`);
  }
}

function walkElision(pos, ast, visitor) {
  if ((visitor.bitmap0 & 32) !== 0) {
    (0, visitor.visitors[5])(new Elision(pos, ast));
  }
}

function walkObjectExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 256) !== 0) {
    node = new ObjectExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[40]);
    if (enter !== null) enter(node);
  }

  walkVecObjectPropertyKind(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkObjectPropertyKind(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxObjectProperty(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxSpreadElement(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ObjectPropertyKind`);
  }
}

function walkObjectProperty(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 512) !== 0) {
    node = new ObjectProperty(pos, ast);
    ({ enter, exit } = visitor.visitors[41]);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkPropertyKey(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxNullLiteral(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxNumericLiteral(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxBigIntLiteral(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxRegExpLiteral(pos + 8, ast, visitor);
      return;
    case 5:
      walkBoxStringLiteral(pos + 8, ast, visitor);
      return;
    case 6:
      walkBoxTemplateLiteral(pos + 8, ast, visitor);
      return;
    case 7:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 8:
      walkBoxMetaProperty(pos + 8, ast, visitor);
      return;
    case 9:
      walkBoxSuper(pos + 8, ast, visitor);
      return;
    case 10:
      walkBoxArrayExpression(pos + 8, ast, visitor);
      return;
    case 11:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitor);
      return;
    case 12:
      walkBoxAssignmentExpression(pos + 8, ast, visitor);
      return;
    case 13:
      walkBoxAwaitExpression(pos + 8, ast, visitor);
      return;
    case 14:
      walkBoxBinaryExpression(pos + 8, ast, visitor);
      return;
    case 15:
      walkBoxCallExpression(pos + 8, ast, visitor);
      return;
    case 16:
      walkBoxChainExpression(pos + 8, ast, visitor);
      return;
    case 17:
      walkBoxClass(pos + 8, ast, visitor);
      return;
    case 18:
      walkBoxConditionalExpression(pos + 8, ast, visitor);
      return;
    case 19:
      walkBoxFunction(pos + 8, ast, visitor);
      return;
    case 20:
      walkBoxImportExpression(pos + 8, ast, visitor);
      return;
    case 21:
      walkBoxLogicalExpression(pos + 8, ast, visitor);
      return;
    case 22:
      walkBoxNewExpression(pos + 8, ast, visitor);
      return;
    case 23:
      walkBoxObjectExpression(pos + 8, ast, visitor);
      return;
    case 24:
      walkBoxParenthesizedExpression(pos + 8, ast, visitor);
      return;
    case 25:
      walkBoxSequenceExpression(pos + 8, ast, visitor);
      return;
    case 26:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitor);
      return;
    case 27:
      walkBoxThisExpression(pos + 8, ast, visitor);
      return;
    case 28:
      walkBoxUnaryExpression(pos + 8, ast, visitor);
      return;
    case 29:
      walkBoxUpdateExpression(pos + 8, ast, visitor);
      return;
    case 30:
      walkBoxYieldExpression(pos + 8, ast, visitor);
      return;
    case 31:
      walkBoxPrivateInExpression(pos + 8, ast, visitor);
      return;
    case 32:
      walkBoxJSXElement(pos + 8, ast, visitor);
      return;
    case 33:
      walkBoxJSXFragment(pos + 8, ast, visitor);
      return;
    case 34:
      walkBoxTSAsExpression(pos + 8, ast, visitor);
      return;
    case 35:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitor);
      return;
    case 36:
      walkBoxTSTypeAssertion(pos + 8, ast, visitor);
      return;
    case 37:
      walkBoxTSNonNullExpression(pos + 8, ast, visitor);
      return;
    case 38:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitor);
      return;
    case 39:
      walkBoxV8IntrinsicExpression(pos + 8, ast, visitor);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitor);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitor);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitor);
      return;
    case 64:
      walkBoxIdentifierName(pos + 8, ast, visitor);
      return;
    case 65:
      walkBoxPrivateIdentifier(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for PropertyKey`);
  }
}

function walkTemplateLiteral(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 1024) !== 0) {
    node = new TemplateLiteral(pos, ast);
    ({ enter, exit } = visitor.visitors[42]);
    if (enter !== null) enter(node);
  }

  walkVecTemplateElement(pos + 8, ast, visitor);
  walkVecExpression(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTaggedTemplateExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 2048) !== 0) {
    node = new TaggedTemplateExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[43]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);
  walkTemplateLiteral(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTemplateElement(pos, ast, visitor) {
  if ((visitor.bitmap0 & 64) !== 0) {
    (0, visitor.visitors[6])(new TemplateElement(pos, ast));
  }
}

function walkComputedMemberExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 4096) !== 0) {
    node = new ComputedMemberExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[44]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkStaticMemberExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 8192) !== 0) {
    node = new StaticMemberExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[45]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkIdentifierName(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkPrivateFieldExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 16384) !== 0) {
    node = new PrivateFieldExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[46]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkPrivateIdentifier(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkCallExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 32768) !== 0) {
    node = new CallExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[47]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);
  walkVecArgument(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkNewExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 65536) !== 0) {
    node = new NewExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[48]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);
  walkVecArgument(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkMetaProperty(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 131072) !== 0) {
    node = new MetaProperty(pos, ast);
    ({ enter, exit } = visitor.visitors[49]);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 8, ast, visitor);
  walkIdentifierName(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkSpreadElement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 262144) !== 0) {
    node = new SpreadElement(pos, ast);
    ({ enter, exit } = visitor.visitors[50]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkArgument(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxNullLiteral(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxNumericLiteral(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxBigIntLiteral(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxRegExpLiteral(pos + 8, ast, visitor);
      return;
    case 5:
      walkBoxStringLiteral(pos + 8, ast, visitor);
      return;
    case 6:
      walkBoxTemplateLiteral(pos + 8, ast, visitor);
      return;
    case 7:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 8:
      walkBoxMetaProperty(pos + 8, ast, visitor);
      return;
    case 9:
      walkBoxSuper(pos + 8, ast, visitor);
      return;
    case 10:
      walkBoxArrayExpression(pos + 8, ast, visitor);
      return;
    case 11:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitor);
      return;
    case 12:
      walkBoxAssignmentExpression(pos + 8, ast, visitor);
      return;
    case 13:
      walkBoxAwaitExpression(pos + 8, ast, visitor);
      return;
    case 14:
      walkBoxBinaryExpression(pos + 8, ast, visitor);
      return;
    case 15:
      walkBoxCallExpression(pos + 8, ast, visitor);
      return;
    case 16:
      walkBoxChainExpression(pos + 8, ast, visitor);
      return;
    case 17:
      walkBoxClass(pos + 8, ast, visitor);
      return;
    case 18:
      walkBoxConditionalExpression(pos + 8, ast, visitor);
      return;
    case 19:
      walkBoxFunction(pos + 8, ast, visitor);
      return;
    case 20:
      walkBoxImportExpression(pos + 8, ast, visitor);
      return;
    case 21:
      walkBoxLogicalExpression(pos + 8, ast, visitor);
      return;
    case 22:
      walkBoxNewExpression(pos + 8, ast, visitor);
      return;
    case 23:
      walkBoxObjectExpression(pos + 8, ast, visitor);
      return;
    case 24:
      walkBoxParenthesizedExpression(pos + 8, ast, visitor);
      return;
    case 25:
      walkBoxSequenceExpression(pos + 8, ast, visitor);
      return;
    case 26:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitor);
      return;
    case 27:
      walkBoxThisExpression(pos + 8, ast, visitor);
      return;
    case 28:
      walkBoxUnaryExpression(pos + 8, ast, visitor);
      return;
    case 29:
      walkBoxUpdateExpression(pos + 8, ast, visitor);
      return;
    case 30:
      walkBoxYieldExpression(pos + 8, ast, visitor);
      return;
    case 31:
      walkBoxPrivateInExpression(pos + 8, ast, visitor);
      return;
    case 32:
      walkBoxJSXElement(pos + 8, ast, visitor);
      return;
    case 33:
      walkBoxJSXFragment(pos + 8, ast, visitor);
      return;
    case 34:
      walkBoxTSAsExpression(pos + 8, ast, visitor);
      return;
    case 35:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitor);
      return;
    case 36:
      walkBoxTSTypeAssertion(pos + 8, ast, visitor);
      return;
    case 37:
      walkBoxTSNonNullExpression(pos + 8, ast, visitor);
      return;
    case 38:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitor);
      return;
    case 39:
      walkBoxV8IntrinsicExpression(pos + 8, ast, visitor);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitor);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitor);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitor);
      return;
    case 64:
      walkBoxSpreadElement(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Argument`);
  }
}

function walkUpdateExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 524288) !== 0) {
    node = new UpdateExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[51]);
    if (enter !== null) enter(node);
  }

  walkSimpleAssignmentTarget(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkUnaryExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 1048576) !== 0) {
    node = new UnaryExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[52]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkBinaryExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 2097152) !== 0) {
    node = new BinaryExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[53]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkPrivateInExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 4194304) !== 0) {
    node = new PrivateInExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[54]);
    if (enter !== null) enter(node);
  }

  walkPrivateIdentifier(pos + 8, ast, visitor);
  walkExpression(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkLogicalExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 8388608) !== 0) {
    node = new LogicalExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[55]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkConditionalExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 16777216) !== 0) {
    node = new ConditionalExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[56]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);
  walkExpression(pos + 40, ast, visitor);

  if (exit !== null) exit(node);
}

function walkAssignmentExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 33554432) !== 0) {
    node = new AssignmentExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[57]);
    if (enter !== null) enter(node);
  }

  walkAssignmentTarget(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkAssignmentTarget(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxTSAsExpression(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxTSNonNullExpression(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxTSTypeAssertion(pos + 8, ast, visitor);
      return;
    case 8:
      walkBoxArrayAssignmentTarget(pos + 8, ast, visitor);
      return;
    case 9:
      walkBoxObjectAssignmentTarget(pos + 8, ast, visitor);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitor);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitor);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for AssignmentTarget`);
  }
}

function walkSimpleAssignmentTarget(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxTSAsExpression(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxTSNonNullExpression(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxTSTypeAssertion(pos + 8, ast, visitor);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitor);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitor);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for SimpleAssignmentTarget`);
  }
}

function walkArrayAssignmentTarget(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 67108864) !== 0) {
    node = new ArrayAssignmentTarget(pos, ast);
    ({ enter, exit } = visitor.visitors[58]);
    if (enter !== null) enter(node);
  }

  walkVecOptionAssignmentTargetMaybeDefault(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkObjectAssignmentTarget(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 134217728) !== 0) {
    node = new ObjectAssignmentTarget(pos, ast);
    ({ enter, exit } = visitor.visitors[59]);
    if (enter !== null) enter(node);
  }

  walkVecAssignmentTargetProperty(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkAssignmentTargetMaybeDefault(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxTSAsExpression(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxTSNonNullExpression(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxTSTypeAssertion(pos + 8, ast, visitor);
      return;
    case 8:
      walkBoxArrayAssignmentTarget(pos + 8, ast, visitor);
      return;
    case 9:
      walkBoxObjectAssignmentTarget(pos + 8, ast, visitor);
      return;
    case 16:
      walkBoxAssignmentTargetWithDefault(pos + 8, ast, visitor);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitor);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitor);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for AssignmentTargetMaybeDefault`);
  }
}

function walkAssignmentTargetWithDefault(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 268435456) !== 0) {
    node = new AssignmentTargetWithDefault(pos, ast);
    ({ enter, exit } = visitor.visitors[60]);
    if (enter !== null) enter(node);
  }

  walkAssignmentTarget(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkAssignmentTargetProperty(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxAssignmentTargetPropertyIdentifier(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxAssignmentTargetPropertyProperty(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for AssignmentTargetProperty`);
  }
}

function walkAssignmentTargetPropertyIdentifier(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 536870912) !== 0) {
    node = new AssignmentTargetPropertyIdentifier(pos, ast);
    ({ enter, exit } = visitor.visitors[61]);
    if (enter !== null) enter(node);
  }

  walkIdentifierReference(pos + 8, ast, visitor);
  walkOptionExpression(pos + 40, ast, visitor);

  if (exit !== null) exit(node);
}

function walkAssignmentTargetPropertyProperty(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & 1073741824) !== 0) {
    node = new AssignmentTargetPropertyProperty(pos, ast);
    ({ enter, exit } = visitor.visitors[62]);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitor);
  walkAssignmentTargetMaybeDefault(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkSequenceExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap1 & -2147483648) !== 0) {
    node = new SequenceExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[63]);
    if (enter !== null) enter(node);
  }

  walkVecExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkSuper(pos, ast, visitor) {
  if ((visitor.bitmap0 & 128) !== 0) {
    (0, visitor.visitors[7])(new Super(pos, ast));
  }
}

function walkAwaitExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 1) !== 0) {
    node = new AwaitExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[64]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkChainExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 2) !== 0) {
    node = new ChainExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[65]);
    if (enter !== null) enter(node);
  }

  walkChainElement(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkChainElement(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxCallExpression(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxTSNonNullExpression(pos + 8, ast, visitor);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitor);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitor);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ChainElement`);
  }
}

function walkParenthesizedExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 4) !== 0) {
    node = new ParenthesizedExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[66]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkStatement(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBlockStatement(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxBreakStatement(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxContinueStatement(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxDebuggerStatement(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxDoWhileStatement(pos + 8, ast, visitor);
      return;
    case 5:
      walkBoxEmptyStatement(pos + 8, ast, visitor);
      return;
    case 6:
      walkBoxExpressionStatement(pos + 8, ast, visitor);
      return;
    case 7:
      walkBoxForInStatement(pos + 8, ast, visitor);
      return;
    case 8:
      walkBoxForOfStatement(pos + 8, ast, visitor);
      return;
    case 9:
      walkBoxForStatement(pos + 8, ast, visitor);
      return;
    case 10:
      walkBoxIfStatement(pos + 8, ast, visitor);
      return;
    case 11:
      walkBoxLabeledStatement(pos + 8, ast, visitor);
      return;
    case 12:
      walkBoxReturnStatement(pos + 8, ast, visitor);
      return;
    case 13:
      walkBoxSwitchStatement(pos + 8, ast, visitor);
      return;
    case 14:
      walkBoxThrowStatement(pos + 8, ast, visitor);
      return;
    case 15:
      walkBoxTryStatement(pos + 8, ast, visitor);
      return;
    case 16:
      walkBoxWhileStatement(pos + 8, ast, visitor);
      return;
    case 17:
      walkBoxWithStatement(pos + 8, ast, visitor);
      return;
    case 32:
      walkBoxVariableDeclaration(pos + 8, ast, visitor);
      return;
    case 33:
      walkBoxFunction(pos + 8, ast, visitor);
      return;
    case 34:
      walkBoxClass(pos + 8, ast, visitor);
      return;
    case 35:
      walkBoxTSTypeAliasDeclaration(pos + 8, ast, visitor);
      return;
    case 36:
      walkBoxTSInterfaceDeclaration(pos + 8, ast, visitor);
      return;
    case 37:
      walkBoxTSEnumDeclaration(pos + 8, ast, visitor);
      return;
    case 38:
      walkBoxTSModuleDeclaration(pos + 8, ast, visitor);
      return;
    case 39:
      walkBoxTSImportEqualsDeclaration(pos + 8, ast, visitor);
      return;
    case 64:
      walkBoxImportDeclaration(pos + 8, ast, visitor);
      return;
    case 65:
      walkBoxExportAllDeclaration(pos + 8, ast, visitor);
      return;
    case 66:
      walkBoxExportDefaultDeclaration(pos + 8, ast, visitor);
      return;
    case 67:
      walkBoxExportNamedDeclaration(pos + 8, ast, visitor);
      return;
    case 68:
      walkBoxTSExportAssignment(pos + 8, ast, visitor);
      return;
    case 69:
      walkBoxTSNamespaceExportDeclaration(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Statement`);
  }
}

function walkHashbang(pos, ast, visitor) {
  if ((visitor.bitmap0 & 256) !== 0) {
    (0, visitor.visitors[8])(new Hashbang(pos, ast));
  }
}

function walkBlockStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 8) !== 0) {
    node = new BlockStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[67]);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkDeclaration(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 32:
      walkBoxVariableDeclaration(pos + 8, ast, visitor);
      return;
    case 33:
      walkBoxFunction(pos + 8, ast, visitor);
      return;
    case 34:
      walkBoxClass(pos + 8, ast, visitor);
      return;
    case 35:
      walkBoxTSTypeAliasDeclaration(pos + 8, ast, visitor);
      return;
    case 36:
      walkBoxTSInterfaceDeclaration(pos + 8, ast, visitor);
      return;
    case 37:
      walkBoxTSEnumDeclaration(pos + 8, ast, visitor);
      return;
    case 38:
      walkBoxTSModuleDeclaration(pos + 8, ast, visitor);
      return;
    case 39:
      walkBoxTSImportEqualsDeclaration(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for Declaration`);
  }
}

function walkVariableDeclaration(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 16) !== 0) {
    node = new VariableDeclaration(pos, ast);
    ({ enter, exit } = visitor.visitors[68]);
    if (enter !== null) enter(node);
  }

  walkVecVariableDeclarator(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkVariableDeclarator(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 32) !== 0) {
    node = new VariableDeclarator(pos, ast);
    ({ enter, exit } = visitor.visitors[69]);
    if (enter !== null) enter(node);
  }

  walkBindingPattern(pos + 8, ast, visitor);
  walkOptionExpression(pos + 40, ast, visitor);

  if (exit !== null) exit(node);
}

function walkEmptyStatement(pos, ast, visitor) {
  if ((visitor.bitmap0 & 512) !== 0) {
    (0, visitor.visitors[9])(new EmptyStatement(pos, ast));
  }
}

function walkExpressionStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 64) !== 0) {
    node = new ExpressionStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[70]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkIfStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 128) !== 0) {
    node = new IfStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[71]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkStatement(pos + 24, ast, visitor);
  walkOptionStatement(pos + 40, ast, visitor);

  if (exit !== null) exit(node);
}

function walkDoWhileStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 256) !== 0) {
    node = new DoWhileStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[72]);
    if (enter !== null) enter(node);
  }

  walkStatement(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkWhileStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 512) !== 0) {
    node = new WhileStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[73]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkStatement(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkForStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 1024) !== 0) {
    node = new ForStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[74]);
    if (enter !== null) enter(node);
  }

  walkOptionForStatementInit(pos + 8, ast, visitor);
  walkOptionExpression(pos + 24, ast, visitor);
  walkOptionExpression(pos + 40, ast, visitor);
  walkStatement(pos + 56, ast, visitor);

  if (exit !== null) exit(node);
}

function walkForStatementInit(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxNullLiteral(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxNumericLiteral(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxBigIntLiteral(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxRegExpLiteral(pos + 8, ast, visitor);
      return;
    case 5:
      walkBoxStringLiteral(pos + 8, ast, visitor);
      return;
    case 6:
      walkBoxTemplateLiteral(pos + 8, ast, visitor);
      return;
    case 7:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 8:
      walkBoxMetaProperty(pos + 8, ast, visitor);
      return;
    case 9:
      walkBoxSuper(pos + 8, ast, visitor);
      return;
    case 10:
      walkBoxArrayExpression(pos + 8, ast, visitor);
      return;
    case 11:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitor);
      return;
    case 12:
      walkBoxAssignmentExpression(pos + 8, ast, visitor);
      return;
    case 13:
      walkBoxAwaitExpression(pos + 8, ast, visitor);
      return;
    case 14:
      walkBoxBinaryExpression(pos + 8, ast, visitor);
      return;
    case 15:
      walkBoxCallExpression(pos + 8, ast, visitor);
      return;
    case 16:
      walkBoxChainExpression(pos + 8, ast, visitor);
      return;
    case 17:
      walkBoxClass(pos + 8, ast, visitor);
      return;
    case 18:
      walkBoxConditionalExpression(pos + 8, ast, visitor);
      return;
    case 19:
      walkBoxFunction(pos + 8, ast, visitor);
      return;
    case 20:
      walkBoxImportExpression(pos + 8, ast, visitor);
      return;
    case 21:
      walkBoxLogicalExpression(pos + 8, ast, visitor);
      return;
    case 22:
      walkBoxNewExpression(pos + 8, ast, visitor);
      return;
    case 23:
      walkBoxObjectExpression(pos + 8, ast, visitor);
      return;
    case 24:
      walkBoxParenthesizedExpression(pos + 8, ast, visitor);
      return;
    case 25:
      walkBoxSequenceExpression(pos + 8, ast, visitor);
      return;
    case 26:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitor);
      return;
    case 27:
      walkBoxThisExpression(pos + 8, ast, visitor);
      return;
    case 28:
      walkBoxUnaryExpression(pos + 8, ast, visitor);
      return;
    case 29:
      walkBoxUpdateExpression(pos + 8, ast, visitor);
      return;
    case 30:
      walkBoxYieldExpression(pos + 8, ast, visitor);
      return;
    case 31:
      walkBoxPrivateInExpression(pos + 8, ast, visitor);
      return;
    case 32:
      walkBoxJSXElement(pos + 8, ast, visitor);
      return;
    case 33:
      walkBoxJSXFragment(pos + 8, ast, visitor);
      return;
    case 34:
      walkBoxTSAsExpression(pos + 8, ast, visitor);
      return;
    case 35:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitor);
      return;
    case 36:
      walkBoxTSTypeAssertion(pos + 8, ast, visitor);
      return;
    case 37:
      walkBoxTSNonNullExpression(pos + 8, ast, visitor);
      return;
    case 38:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitor);
      return;
    case 39:
      walkBoxV8IntrinsicExpression(pos + 8, ast, visitor);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitor);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitor);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitor);
      return;
    case 64:
      walkBoxVariableDeclaration(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ForStatementInit`);
  }
}

function walkForInStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 2048) !== 0) {
    node = new ForInStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[75]);
    if (enter !== null) enter(node);
  }

  walkForStatementLeft(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);
  walkStatement(pos + 40, ast, visitor);

  if (exit !== null) exit(node);
}

function walkForStatementLeft(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxTSAsExpression(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxTSNonNullExpression(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxTSTypeAssertion(pos + 8, ast, visitor);
      return;
    case 8:
      walkBoxArrayAssignmentTarget(pos + 8, ast, visitor);
      return;
    case 9:
      walkBoxObjectAssignmentTarget(pos + 8, ast, visitor);
      return;
    case 16:
      walkBoxVariableDeclaration(pos + 8, ast, visitor);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitor);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitor);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ForStatementLeft`);
  }
}

function walkForOfStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 4096) !== 0) {
    node = new ForOfStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[76]);
    if (enter !== null) enter(node);
  }

  walkForStatementLeft(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);
  walkStatement(pos + 40, ast, visitor);

  if (exit !== null) exit(node);
}

function walkContinueStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 8192) !== 0) {
    node = new ContinueStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[77]);
    if (enter !== null) enter(node);
  }

  walkOptionLabelIdentifier(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkBreakStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 16384) !== 0) {
    node = new BreakStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[78]);
    if (enter !== null) enter(node);
  }

  walkOptionLabelIdentifier(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkReturnStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 32768) !== 0) {
    node = new ReturnStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[79]);
    if (enter !== null) enter(node);
  }

  walkOptionExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkWithStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 65536) !== 0) {
    node = new WithStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[80]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkStatement(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkSwitchStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 131072) !== 0) {
    node = new SwitchStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[81]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkVecSwitchCase(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkSwitchCase(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 262144) !== 0) {
    node = new SwitchCase(pos, ast);
    ({ enter, exit } = visitor.visitors[82]);
    if (enter !== null) enter(node);
  }

  walkOptionExpression(pos + 8, ast, visitor);
  walkVecStatement(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkLabeledStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 524288) !== 0) {
    node = new LabeledStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[83]);
    if (enter !== null) enter(node);
  }

  walkLabelIdentifier(pos + 8, ast, visitor);
  walkStatement(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkThrowStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 1048576) !== 0) {
    node = new ThrowStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[84]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTryStatement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 2097152) !== 0) {
    node = new TryStatement(pos, ast);
    ({ enter, exit } = visitor.visitors[85]);
    if (enter !== null) enter(node);
  }

  walkBoxBlockStatement(pos + 8, ast, visitor);
  walkOptionBoxCatchClause(pos + 16, ast, visitor);
  walkOptionBoxBlockStatement(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkCatchClause(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 4194304) !== 0) {
    node = new CatchClause(pos, ast);
    ({ enter, exit } = visitor.visitors[86]);
    if (enter !== null) enter(node);
  }

  walkOptionCatchParameter(pos + 8, ast, visitor);
  walkBoxBlockStatement(pos + 48, ast, visitor);

  if (exit !== null) exit(node);
}

function walkCatchParameter(pos, ast, visitor) {
  walkBindingPattern(pos + 8, ast, visitor);
}

function walkDebuggerStatement(pos, ast, visitor) {
  if ((visitor.bitmap0 & 1024) !== 0) {
    (0, visitor.visitors[10])(new DebuggerStatement(pos, ast));
  }
}

function walkBindingPattern(pos, ast, visitor) {
  walkBindingPatternKind(pos, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 16, ast, visitor);
}

function walkBindingPatternKind(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBindingIdentifier(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxObjectPattern(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxArrayPattern(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxAssignmentPattern(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for BindingPatternKind`);
  }
}

function walkAssignmentPattern(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 8388608) !== 0) {
    node = new AssignmentPattern(pos, ast);
    ({ enter, exit } = visitor.visitors[87]);
    if (enter !== null) enter(node);
  }

  walkBindingPattern(pos + 8, ast, visitor);
  walkExpression(pos + 40, ast, visitor);

  if (exit !== null) exit(node);
}

function walkObjectPattern(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 16777216) !== 0) {
    node = new ObjectPattern(pos, ast);
    ({ enter, exit } = visitor.visitors[88]);
    if (enter !== null) enter(node);
  }

  walkVecBindingProperty(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkBindingProperty(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 33554432) !== 0) {
    node = new BindingProperty(pos, ast);
    ({ enter, exit } = visitor.visitors[89]);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitor);
  walkBindingPattern(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkArrayPattern(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 67108864) !== 0) {
    node = new ArrayPattern(pos, ast);
    ({ enter, exit } = visitor.visitors[90]);
    if (enter !== null) enter(node);
  }

  walkVecOptionBindingPattern(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkFunction(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 134217728) !== 0) {
    node = new Function(pos, ast);
    ({ enter, exit } = visitor.visitors[91]);
    if (enter !== null) enter(node);
  }

  walkOptionBindingIdentifier(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterDeclaration(pos + 40, ast, visitor);
  walkBoxFormalParameters(pos + 56, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 64, ast, visitor);
  walkOptionBoxFunctionBody(pos + 72, ast, visitor);

  if (exit !== null) exit(node);
}

function walkFormalParameters(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 268435456) !== 0) {
    node = new FormalParameters(pos, ast);
    ({ enter, exit } = visitor.visitors[92]);
    if (enter !== null) enter(node);
  }

  walkVecFormalParameter(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkFormalParameter(pos, ast, visitor) {
  walkVecDecorator(pos + 8, ast, visitor);
  walkBindingPattern(pos + 32, ast, visitor);
}

function walkFunctionBody(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 536870912) !== 0) {
    node = new FunctionBody(pos, ast);
    ({ enter, exit } = visitor.visitors[93]);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkArrowFunctionExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & 1073741824) !== 0) {
    node = new ArrowFunctionExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[94]);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitor);
  walkBoxFormalParameters(pos + 16, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 24, ast, visitor);
  walkBoxFunctionBody(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkYieldExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap2 & -2147483648) !== 0) {
    node = new YieldExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[95]);
    if (enter !== null) enter(node);
  }

  walkOptionExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkClass(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 1) !== 0) {
    node = new Class(pos, ast);
    ({ enter, exit } = visitor.visitors[96]);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 8, ast, visitor);
  walkOptionBindingIdentifier(pos + 32, ast, visitor);
  walkOptionBoxTSTypeParameterDeclaration(pos + 64, ast, visitor);
  walkOptionExpression(pos + 72, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 88, ast, visitor);
  walkVecTSClassImplements(pos + 96, ast, visitor);
  walkBoxClassBody(pos + 120, ast, visitor);

  if (exit !== null) exit(node);
}

function walkClassBody(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 2) !== 0) {
    node = new ClassBody(pos, ast);
    ({ enter, exit } = visitor.visitors[97]);
    if (enter !== null) enter(node);
  }

  walkVecClassElement(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkClassElement(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxStaticBlock(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxMethodDefinition(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxPropertyDefinition(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxAccessorProperty(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxTSIndexSignature(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ClassElement`);
  }
}

function walkMethodDefinition(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 4) !== 0) {
    node = new MethodDefinition(pos, ast);
    ({ enter, exit } = visitor.visitors[98]);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 8, ast, visitor);
  walkPropertyKey(pos + 32, ast, visitor);
  walkBoxFunction(pos + 48, ast, visitor);

  if (exit !== null) exit(node);
}

function walkPropertyDefinition(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 8) !== 0) {
    node = new PropertyDefinition(pos, ast);
    ({ enter, exit } = visitor.visitors[99]);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 8, ast, visitor);
  walkPropertyKey(pos + 32, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 48, ast, visitor);
  walkOptionExpression(pos + 56, ast, visitor);

  if (exit !== null) exit(node);
}

function walkPrivateIdentifier(pos, ast, visitor) {
  if ((visitor.bitmap0 & 2048) !== 0) {
    (0, visitor.visitors[11])(new PrivateIdentifier(pos, ast));
  }
}

function walkStaticBlock(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 16) !== 0) {
    node = new StaticBlock(pos, ast);
    ({ enter, exit } = visitor.visitors[100]);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkAccessorProperty(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 32) !== 0) {
    node = new AccessorProperty(pos, ast);
    ({ enter, exit } = visitor.visitors[101]);
    if (enter !== null) enter(node);
  }

  walkVecDecorator(pos + 8, ast, visitor);
  walkPropertyKey(pos + 32, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 48, ast, visitor);
  walkOptionExpression(pos + 56, ast, visitor);

  if (exit !== null) exit(node);
}

function walkImportExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 64) !== 0) {
    node = new ImportExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[102]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkOptionExpression(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkImportDeclaration(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 128) !== 0) {
    node = new ImportDeclaration(pos, ast);
    ({ enter, exit } = visitor.visitors[103]);
    if (enter !== null) enter(node);
  }

  walkOptionVecImportDeclarationSpecifier(pos + 8, ast, visitor);
  walkStringLiteral(pos + 32, ast, visitor);
  walkOptionBoxWithClause(pos + 80, ast, visitor);

  if (exit !== null) exit(node);
}

function walkImportDeclarationSpecifier(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxImportSpecifier(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxImportDefaultSpecifier(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxImportNamespaceSpecifier(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ImportDeclarationSpecifier`);
  }
}

function walkImportSpecifier(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 256) !== 0) {
    node = new ImportSpecifier(pos, ast);
    ({ enter, exit } = visitor.visitors[104]);
    if (enter !== null) enter(node);
  }

  walkModuleExportName(pos + 8, ast, visitor);
  walkBindingIdentifier(pos + 64, ast, visitor);

  if (exit !== null) exit(node);
}

function walkImportDefaultSpecifier(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 512) !== 0) {
    node = new ImportDefaultSpecifier(pos, ast);
    ({ enter, exit } = visitor.visitors[105]);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkImportNamespaceSpecifier(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 1024) !== 0) {
    node = new ImportNamespaceSpecifier(pos, ast);
    ({ enter, exit } = visitor.visitors[106]);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkWithClause(pos, ast, visitor) {
  walkVecImportAttribute(pos + 32, ast, visitor);
}

function walkImportAttribute(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 2048) !== 0) {
    node = new ImportAttribute(pos, ast);
    ({ enter, exit } = visitor.visitors[107]);
    if (enter !== null) enter(node);
  }

  walkImportAttributeKey(pos + 8, ast, visitor);
  walkStringLiteral(pos + 64, ast, visitor);

  if (exit !== null) exit(node);
}

function walkImportAttributeKey(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkIdentifierName(pos + 8, ast, visitor);
      return;
    case 1:
      walkStringLiteral(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ImportAttributeKey`);
  }
}

function walkExportNamedDeclaration(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 4096) !== 0) {
    node = new ExportNamedDeclaration(pos, ast);
    ({ enter, exit } = visitor.visitors[108]);
    if (enter !== null) enter(node);
  }

  walkOptionDeclaration(pos + 8, ast, visitor);
  walkVecExportSpecifier(pos + 24, ast, visitor);
  walkOptionStringLiteral(pos + 48, ast, visitor);
  walkOptionBoxWithClause(pos + 96, ast, visitor);

  if (exit !== null) exit(node);
}

function walkExportDefaultDeclaration(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 8192) !== 0) {
    node = new ExportDefaultDeclaration(pos, ast);
    ({ enter, exit } = visitor.visitors[109]);
    if (enter !== null) enter(node);
  }

  walkExportDefaultDeclarationKind(pos + 64, ast, visitor);

  if (exit !== null) exit(node);
}

function walkExportAllDeclaration(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 16384) !== 0) {
    node = new ExportAllDeclaration(pos, ast);
    ({ enter, exit } = visitor.visitors[110]);
    if (enter !== null) enter(node);
  }

  walkOptionModuleExportName(pos + 8, ast, visitor);
  walkStringLiteral(pos + 64, ast, visitor);
  walkOptionBoxWithClause(pos + 112, ast, visitor);

  if (exit !== null) exit(node);
}

function walkExportSpecifier(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 32768) !== 0) {
    node = new ExportSpecifier(pos, ast);
    ({ enter, exit } = visitor.visitors[111]);
    if (enter !== null) enter(node);
  }

  walkModuleExportName(pos + 8, ast, visitor);
  walkModuleExportName(pos + 64, ast, visitor);

  if (exit !== null) exit(node);
}

function walkExportDefaultDeclarationKind(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxNullLiteral(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxNumericLiteral(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxBigIntLiteral(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxRegExpLiteral(pos + 8, ast, visitor);
      return;
    case 5:
      walkBoxStringLiteral(pos + 8, ast, visitor);
      return;
    case 6:
      walkBoxTemplateLiteral(pos + 8, ast, visitor);
      return;
    case 7:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 8:
      walkBoxMetaProperty(pos + 8, ast, visitor);
      return;
    case 9:
      walkBoxSuper(pos + 8, ast, visitor);
      return;
    case 10:
      walkBoxArrayExpression(pos + 8, ast, visitor);
      return;
    case 11:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitor);
      return;
    case 12:
      walkBoxAssignmentExpression(pos + 8, ast, visitor);
      return;
    case 13:
      walkBoxAwaitExpression(pos + 8, ast, visitor);
      return;
    case 14:
      walkBoxBinaryExpression(pos + 8, ast, visitor);
      return;
    case 15:
      walkBoxCallExpression(pos + 8, ast, visitor);
      return;
    case 16:
      walkBoxChainExpression(pos + 8, ast, visitor);
      return;
    case 17:
      walkBoxClass(pos + 8, ast, visitor);
      return;
    case 18:
      walkBoxConditionalExpression(pos + 8, ast, visitor);
      return;
    case 19:
      walkBoxFunction(pos + 8, ast, visitor);
      return;
    case 20:
      walkBoxImportExpression(pos + 8, ast, visitor);
      return;
    case 21:
      walkBoxLogicalExpression(pos + 8, ast, visitor);
      return;
    case 22:
      walkBoxNewExpression(pos + 8, ast, visitor);
      return;
    case 23:
      walkBoxObjectExpression(pos + 8, ast, visitor);
      return;
    case 24:
      walkBoxParenthesizedExpression(pos + 8, ast, visitor);
      return;
    case 25:
      walkBoxSequenceExpression(pos + 8, ast, visitor);
      return;
    case 26:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitor);
      return;
    case 27:
      walkBoxThisExpression(pos + 8, ast, visitor);
      return;
    case 28:
      walkBoxUnaryExpression(pos + 8, ast, visitor);
      return;
    case 29:
      walkBoxUpdateExpression(pos + 8, ast, visitor);
      return;
    case 30:
      walkBoxYieldExpression(pos + 8, ast, visitor);
      return;
    case 31:
      walkBoxPrivateInExpression(pos + 8, ast, visitor);
      return;
    case 32:
      walkBoxJSXElement(pos + 8, ast, visitor);
      return;
    case 33:
      walkBoxJSXFragment(pos + 8, ast, visitor);
      return;
    case 34:
      walkBoxTSAsExpression(pos + 8, ast, visitor);
      return;
    case 35:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitor);
      return;
    case 36:
      walkBoxTSTypeAssertion(pos + 8, ast, visitor);
      return;
    case 37:
      walkBoxTSNonNullExpression(pos + 8, ast, visitor);
      return;
    case 38:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitor);
      return;
    case 39:
      walkBoxV8IntrinsicExpression(pos + 8, ast, visitor);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitor);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitor);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitor);
      return;
    case 64:
      walkBoxFunction(pos + 8, ast, visitor);
      return;
    case 65:
      walkBoxClass(pos + 8, ast, visitor);
      return;
    case 66:
      walkBoxTSInterfaceDeclaration(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ExportDefaultDeclarationKind`);
  }
}

function walkModuleExportName(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkIdentifierName(pos + 8, ast, visitor);
      return;
    case 1:
      walkIdentifierReference(pos + 8, ast, visitor);
      return;
    case 2:
      walkStringLiteral(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for ModuleExportName`);
  }
}

function walkV8IntrinsicExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 65536) !== 0) {
    node = new V8IntrinsicExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[112]);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 8, ast, visitor);
  walkVecArgument(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkBooleanLiteral(pos, ast, visitor) {
  if ((visitor.bitmap0 & 4096) !== 0) {
    (0, visitor.visitors[12])(new BooleanLiteral(pos, ast));
  }
}

function walkNullLiteral(pos, ast, visitor) {
  if ((visitor.bitmap0 & 8192) !== 0) {
    (0, visitor.visitors[13])(new NullLiteral(pos, ast));
  }
}

function walkNumericLiteral(pos, ast, visitor) {
  if ((visitor.bitmap0 & 16384) !== 0) {
    (0, visitor.visitors[14])(new NumericLiteral(pos, ast));
  }
}

function walkStringLiteral(pos, ast, visitor) {
  if ((visitor.bitmap0 & 32768) !== 0) {
    (0, visitor.visitors[15])(new StringLiteral(pos, ast));
  }
}

function walkBigIntLiteral(pos, ast, visitor) {
  if ((visitor.bitmap0 & 65536) !== 0) {
    (0, visitor.visitors[16])(new BigIntLiteral(pos, ast));
  }
}

function walkRegExpLiteral(pos, ast, visitor) {
  if ((visitor.bitmap0 & 131072) !== 0) {
    (0, visitor.visitors[17])(new RegExpLiteral(pos, ast));
  }
}

function walkJSXElement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 131072) !== 0) {
    node = new JSXElement(pos, ast);
    ({ enter, exit } = visitor.visitors[113]);
    if (enter !== null) enter(node);
  }

  walkBoxJSXOpeningElement(pos + 8, ast, visitor);
  walkVecJSXChild(pos + 16, ast, visitor);
  walkOptionBoxJSXClosingElement(pos + 40, ast, visitor);

  if (exit !== null) exit(node);
}

function walkJSXOpeningElement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 262144) !== 0) {
    node = new JSXOpeningElement(pos, ast);
    ({ enter, exit } = visitor.visitors[114]);
    if (enter !== null) enter(node);
  }

  walkJSXElementName(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);
  walkVecJSXAttributeItem(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkJSXClosingElement(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 524288) !== 0) {
    node = new JSXClosingElement(pos, ast);
    ({ enter, exit } = visitor.visitors[115]);
    if (enter !== null) enter(node);
  }

  walkJSXElementName(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkJSXFragment(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 1048576) !== 0) {
    node = new JSXFragment(pos, ast);
    ({ enter, exit } = visitor.visitors[116]);
    if (enter !== null) enter(node);
  }

  walkJSXOpeningFragment(pos + 8, ast, visitor);
  walkVecJSXChild(pos + 16, ast, visitor);
  walkJSXClosingFragment(pos + 40, ast, visitor);

  if (exit !== null) exit(node);
}

function walkJSXOpeningFragment(pos, ast, visitor) {
  if ((visitor.bitmap0 & 262144) !== 0) {
    (0, visitor.visitors[18])(new JSXOpeningFragment(pos, ast));
  }
}

function walkJSXClosingFragment(pos, ast, visitor) {
  if ((visitor.bitmap0 & 524288) !== 0) {
    (0, visitor.visitors[19])(new JSXClosingFragment(pos, ast));
  }
}

function walkJSXElementName(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxJSXIdentifier(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxJSXNamespacedName(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxJSXMemberExpression(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxThisExpression(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXElementName`);
  }
}

function walkJSXNamespacedName(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 2097152) !== 0) {
    node = new JSXNamespacedName(pos, ast);
    ({ enter, exit } = visitor.visitors[117]);
    if (enter !== null) enter(node);
  }

  walkJSXIdentifier(pos + 8, ast, visitor);
  walkJSXIdentifier(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkJSXMemberExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 4194304) !== 0) {
    node = new JSXMemberExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[118]);
    if (enter !== null) enter(node);
  }

  walkJSXMemberExpressionObject(pos + 8, ast, visitor);
  walkJSXIdentifier(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkJSXMemberExpressionObject(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxJSXMemberExpression(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxThisExpression(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXMemberExpressionObject`);
  }
}

function walkJSXExpressionContainer(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 8388608) !== 0) {
    node = new JSXExpressionContainer(pos, ast);
    ({ enter, exit } = visitor.visitors[119]);
    if (enter !== null) enter(node);
  }

  walkJSXExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkJSXExpression(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxNullLiteral(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxNumericLiteral(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxBigIntLiteral(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxRegExpLiteral(pos + 8, ast, visitor);
      return;
    case 5:
      walkBoxStringLiteral(pos + 8, ast, visitor);
      return;
    case 6:
      walkBoxTemplateLiteral(pos + 8, ast, visitor);
      return;
    case 7:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 8:
      walkBoxMetaProperty(pos + 8, ast, visitor);
      return;
    case 9:
      walkBoxSuper(pos + 8, ast, visitor);
      return;
    case 10:
      walkBoxArrayExpression(pos + 8, ast, visitor);
      return;
    case 11:
      walkBoxArrowFunctionExpression(pos + 8, ast, visitor);
      return;
    case 12:
      walkBoxAssignmentExpression(pos + 8, ast, visitor);
      return;
    case 13:
      walkBoxAwaitExpression(pos + 8, ast, visitor);
      return;
    case 14:
      walkBoxBinaryExpression(pos + 8, ast, visitor);
      return;
    case 15:
      walkBoxCallExpression(pos + 8, ast, visitor);
      return;
    case 16:
      walkBoxChainExpression(pos + 8, ast, visitor);
      return;
    case 17:
      walkBoxClass(pos + 8, ast, visitor);
      return;
    case 18:
      walkBoxConditionalExpression(pos + 8, ast, visitor);
      return;
    case 19:
      walkBoxFunction(pos + 8, ast, visitor);
      return;
    case 20:
      walkBoxImportExpression(pos + 8, ast, visitor);
      return;
    case 21:
      walkBoxLogicalExpression(pos + 8, ast, visitor);
      return;
    case 22:
      walkBoxNewExpression(pos + 8, ast, visitor);
      return;
    case 23:
      walkBoxObjectExpression(pos + 8, ast, visitor);
      return;
    case 24:
      walkBoxParenthesizedExpression(pos + 8, ast, visitor);
      return;
    case 25:
      walkBoxSequenceExpression(pos + 8, ast, visitor);
      return;
    case 26:
      walkBoxTaggedTemplateExpression(pos + 8, ast, visitor);
      return;
    case 27:
      walkBoxThisExpression(pos + 8, ast, visitor);
      return;
    case 28:
      walkBoxUnaryExpression(pos + 8, ast, visitor);
      return;
    case 29:
      walkBoxUpdateExpression(pos + 8, ast, visitor);
      return;
    case 30:
      walkBoxYieldExpression(pos + 8, ast, visitor);
      return;
    case 31:
      walkBoxPrivateInExpression(pos + 8, ast, visitor);
      return;
    case 32:
      walkBoxJSXElement(pos + 8, ast, visitor);
      return;
    case 33:
      walkBoxJSXFragment(pos + 8, ast, visitor);
      return;
    case 34:
      walkBoxTSAsExpression(pos + 8, ast, visitor);
      return;
    case 35:
      walkBoxTSSatisfiesExpression(pos + 8, ast, visitor);
      return;
    case 36:
      walkBoxTSTypeAssertion(pos + 8, ast, visitor);
      return;
    case 37:
      walkBoxTSNonNullExpression(pos + 8, ast, visitor);
      return;
    case 38:
      walkBoxTSInstantiationExpression(pos + 8, ast, visitor);
      return;
    case 39:
      walkBoxV8IntrinsicExpression(pos + 8, ast, visitor);
      return;
    case 48:
      walkBoxComputedMemberExpression(pos + 8, ast, visitor);
      return;
    case 49:
      walkBoxStaticMemberExpression(pos + 8, ast, visitor);
      return;
    case 50:
      walkBoxPrivateFieldExpression(pos + 8, ast, visitor);
      return;
    case 64:
      walkJSXEmptyExpression(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXExpression`);
  }
}

function walkJSXEmptyExpression(pos, ast, visitor) {
  if ((visitor.bitmap0 & 1048576) !== 0) {
    (0, visitor.visitors[20])(new JSXEmptyExpression(pos, ast));
  }
}

function walkJSXAttributeItem(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxJSXAttribute(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxJSXSpreadAttribute(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXAttributeItem`);
  }
}

function walkJSXAttribute(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 16777216) !== 0) {
    node = new JSXAttribute(pos, ast);
    ({ enter, exit } = visitor.visitors[120]);
    if (enter !== null) enter(node);
  }

  walkJSXAttributeName(pos + 8, ast, visitor);
  walkOptionJSXAttributeValue(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkJSXSpreadAttribute(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 33554432) !== 0) {
    node = new JSXSpreadAttribute(pos, ast);
    ({ enter, exit } = visitor.visitors[121]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkJSXAttributeName(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxJSXIdentifier(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxJSXNamespacedName(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXAttributeName`);
  }
}

function walkJSXAttributeValue(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxStringLiteral(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxJSXExpressionContainer(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxJSXElement(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxJSXFragment(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXAttributeValue`);
  }
}

function walkJSXIdentifier(pos, ast, visitor) {
  if ((visitor.bitmap0 & 2097152) !== 0) {
    (0, visitor.visitors[21])(new JSXIdentifier(pos, ast));
  }
}

function walkJSXChild(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxJSXText(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxJSXElement(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxJSXFragment(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxJSXExpressionContainer(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxJSXSpreadChild(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for JSXChild`);
  }
}

function walkJSXSpreadChild(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 67108864) !== 0) {
    node = new JSXSpreadChild(pos, ast);
    ({ enter, exit } = visitor.visitors[122]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkJSXText(pos, ast, visitor) {
  if ((visitor.bitmap0 & 4194304) !== 0) {
    (0, visitor.visitors[22])(new JSXText(pos, ast));
  }
}

function walkTSEnumDeclaration(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 134217728) !== 0) {
    node = new TSEnumDeclaration(pos, ast);
    ({ enter, exit } = visitor.visitors[123]);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitor);
  walkTSEnumBody(pos + 40, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSEnumBody(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 268435456) !== 0) {
    node = new TSEnumBody(pos, ast);
    ({ enter, exit } = visitor.visitors[124]);
    if (enter !== null) enter(node);
  }

  walkVecTSEnumMember(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSEnumMember(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 536870912) !== 0) {
    node = new TSEnumMember(pos, ast);
    ({ enter, exit } = visitor.visitors[125]);
    if (enter !== null) enter(node);
  }

  walkTSEnumMemberName(pos + 8, ast, visitor);
  walkOptionExpression(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSEnumMemberName(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierName(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxStringLiteral(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxStringLiteral(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxTemplateLiteral(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSEnumMemberName`);
  }
}

function walkTSTypeAnnotation(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & 1073741824) !== 0) {
    node = new TSTypeAnnotation(pos, ast);
    ({ enter, exit } = visitor.visitors[126]);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSLiteralType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap3 & -2147483648) !== 0) {
    node = new TSLiteralType(pos, ast);
    ({ enter, exit } = visitor.visitors[127]);
    if (enter !== null) enter(node);
  }

  walkTSLiteral(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSLiteral(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxBooleanLiteral(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxNumericLiteral(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxBigIntLiteral(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxStringLiteral(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxTemplateLiteral(pos + 8, ast, visitor);
      return;
    case 5:
      walkBoxUnaryExpression(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSLiteral`);
  }
}

function walkTSType(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxTSAnyKeyword(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxTSBigIntKeyword(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxTSBooleanKeyword(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxTSIntrinsicKeyword(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxTSNeverKeyword(pos + 8, ast, visitor);
      return;
    case 5:
      walkBoxTSNullKeyword(pos + 8, ast, visitor);
      return;
    case 6:
      walkBoxTSNumberKeyword(pos + 8, ast, visitor);
      return;
    case 7:
      walkBoxTSObjectKeyword(pos + 8, ast, visitor);
      return;
    case 8:
      walkBoxTSStringKeyword(pos + 8, ast, visitor);
      return;
    case 9:
      walkBoxTSSymbolKeyword(pos + 8, ast, visitor);
      return;
    case 10:
      walkBoxTSThisType(pos + 8, ast, visitor);
      return;
    case 11:
      walkBoxTSUndefinedKeyword(pos + 8, ast, visitor);
      return;
    case 12:
      walkBoxTSUnknownKeyword(pos + 8, ast, visitor);
      return;
    case 13:
      walkBoxTSVoidKeyword(pos + 8, ast, visitor);
      return;
    case 14:
      walkBoxTSArrayType(pos + 8, ast, visitor);
      return;
    case 15:
      walkBoxTSConditionalType(pos + 8, ast, visitor);
      return;
    case 16:
      walkBoxTSConstructorType(pos + 8, ast, visitor);
      return;
    case 17:
      walkBoxTSFunctionType(pos + 8, ast, visitor);
      return;
    case 18:
      walkBoxTSImportType(pos + 8, ast, visitor);
      return;
    case 19:
      walkBoxTSIndexedAccessType(pos + 8, ast, visitor);
      return;
    case 20:
      walkBoxTSInferType(pos + 8, ast, visitor);
      return;
    case 21:
      walkBoxTSIntersectionType(pos + 8, ast, visitor);
      return;
    case 22:
      walkBoxTSLiteralType(pos + 8, ast, visitor);
      return;
    case 23:
      walkBoxTSMappedType(pos + 8, ast, visitor);
      return;
    case 24:
      walkBoxTSNamedTupleMember(pos + 8, ast, visitor);
      return;
    case 26:
      walkBoxTSTemplateLiteralType(pos + 8, ast, visitor);
      return;
    case 27:
      walkBoxTSTupleType(pos + 8, ast, visitor);
      return;
    case 28:
      walkBoxTSTypeLiteral(pos + 8, ast, visitor);
      return;
    case 29:
      walkBoxTSTypeOperator(pos + 8, ast, visitor);
      return;
    case 30:
      walkBoxTSTypePredicate(pos + 8, ast, visitor);
      return;
    case 31:
      walkBoxTSTypeQuery(pos + 8, ast, visitor);
      return;
    case 32:
      walkBoxTSTypeReference(pos + 8, ast, visitor);
      return;
    case 33:
      walkBoxTSUnionType(pos + 8, ast, visitor);
      return;
    case 34:
      walkBoxTSParenthesizedType(pos + 8, ast, visitor);
      return;
    case 35:
      walkBoxJSDocNullableType(pos + 8, ast, visitor);
      return;
    case 36:
      walkBoxJSDocNonNullableType(pos + 8, ast, visitor);
      return;
    case 37:
      walkBoxJSDocUnknownType(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSType`);
  }
}

function walkTSConditionalType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 1) !== 0) {
    node = new TSConditionalType(pos, ast);
    ({ enter, exit } = visitor.visitors[128]);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);
  walkTSType(pos + 24, ast, visitor);
  walkTSType(pos + 40, ast, visitor);
  walkTSType(pos + 56, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSUnionType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 2) !== 0) {
    node = new TSUnionType(pos, ast);
    ({ enter, exit } = visitor.visitors[129]);
    if (enter !== null) enter(node);
  }

  walkVecTSType(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSIntersectionType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 4) !== 0) {
    node = new TSIntersectionType(pos, ast);
    ({ enter, exit } = visitor.visitors[130]);
    if (enter !== null) enter(node);
  }

  walkVecTSType(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSParenthesizedType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 8) !== 0) {
    node = new TSParenthesizedType(pos, ast);
    ({ enter, exit } = visitor.visitors[131]);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTypeOperator(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 16) !== 0) {
    node = new TSTypeOperator(pos, ast);
    ({ enter, exit } = visitor.visitors[132]);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSArrayType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 32) !== 0) {
    node = new TSArrayType(pos, ast);
    ({ enter, exit } = visitor.visitors[133]);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSIndexedAccessType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 64) !== 0) {
    node = new TSIndexedAccessType(pos, ast);
    ({ enter, exit } = visitor.visitors[134]);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);
  walkTSType(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTupleType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 128) !== 0) {
    node = new TSTupleType(pos, ast);
    ({ enter, exit } = visitor.visitors[135]);
    if (enter !== null) enter(node);
  }

  walkVecTSTupleElement(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSNamedTupleMember(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 256) !== 0) {
    node = new TSNamedTupleMember(pos, ast);
    ({ enter, exit } = visitor.visitors[136]);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 8, ast, visitor);
  walkTSTupleElement(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSOptionalType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 512) !== 0) {
    node = new TSOptionalType(pos, ast);
    ({ enter, exit } = visitor.visitors[137]);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSRestType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 1024) !== 0) {
    node = new TSRestType(pos, ast);
    ({ enter, exit } = visitor.visitors[138]);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTupleElement(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxTSAnyKeyword(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxTSBigIntKeyword(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxTSBooleanKeyword(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxTSIntrinsicKeyword(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxTSNeverKeyword(pos + 8, ast, visitor);
      return;
    case 5:
      walkBoxTSNullKeyword(pos + 8, ast, visitor);
      return;
    case 6:
      walkBoxTSNumberKeyword(pos + 8, ast, visitor);
      return;
    case 7:
      walkBoxTSObjectKeyword(pos + 8, ast, visitor);
      return;
    case 8:
      walkBoxTSStringKeyword(pos + 8, ast, visitor);
      return;
    case 9:
      walkBoxTSSymbolKeyword(pos + 8, ast, visitor);
      return;
    case 10:
      walkBoxTSThisType(pos + 8, ast, visitor);
      return;
    case 11:
      walkBoxTSUndefinedKeyword(pos + 8, ast, visitor);
      return;
    case 12:
      walkBoxTSUnknownKeyword(pos + 8, ast, visitor);
      return;
    case 13:
      walkBoxTSVoidKeyword(pos + 8, ast, visitor);
      return;
    case 14:
      walkBoxTSArrayType(pos + 8, ast, visitor);
      return;
    case 15:
      walkBoxTSConditionalType(pos + 8, ast, visitor);
      return;
    case 16:
      walkBoxTSConstructorType(pos + 8, ast, visitor);
      return;
    case 17:
      walkBoxTSFunctionType(pos + 8, ast, visitor);
      return;
    case 18:
      walkBoxTSImportType(pos + 8, ast, visitor);
      return;
    case 19:
      walkBoxTSIndexedAccessType(pos + 8, ast, visitor);
      return;
    case 20:
      walkBoxTSInferType(pos + 8, ast, visitor);
      return;
    case 21:
      walkBoxTSIntersectionType(pos + 8, ast, visitor);
      return;
    case 22:
      walkBoxTSLiteralType(pos + 8, ast, visitor);
      return;
    case 23:
      walkBoxTSMappedType(pos + 8, ast, visitor);
      return;
    case 24:
      walkBoxTSNamedTupleMember(pos + 8, ast, visitor);
      return;
    case 26:
      walkBoxTSTemplateLiteralType(pos + 8, ast, visitor);
      return;
    case 27:
      walkBoxTSTupleType(pos + 8, ast, visitor);
      return;
    case 28:
      walkBoxTSTypeLiteral(pos + 8, ast, visitor);
      return;
    case 29:
      walkBoxTSTypeOperator(pos + 8, ast, visitor);
      return;
    case 30:
      walkBoxTSTypePredicate(pos + 8, ast, visitor);
      return;
    case 31:
      walkBoxTSTypeQuery(pos + 8, ast, visitor);
      return;
    case 32:
      walkBoxTSTypeReference(pos + 8, ast, visitor);
      return;
    case 33:
      walkBoxTSUnionType(pos + 8, ast, visitor);
      return;
    case 34:
      walkBoxTSParenthesizedType(pos + 8, ast, visitor);
      return;
    case 35:
      walkBoxJSDocNullableType(pos + 8, ast, visitor);
      return;
    case 36:
      walkBoxJSDocNonNullableType(pos + 8, ast, visitor);
      return;
    case 37:
      walkBoxJSDocUnknownType(pos + 8, ast, visitor);
      return;
    case 64:
      walkBoxTSOptionalType(pos + 8, ast, visitor);
      return;
    case 65:
      walkBoxTSRestType(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTupleElement`);
  }
}

function walkTSAnyKeyword(pos, ast, visitor) {
  if ((visitor.bitmap0 & 8388608) !== 0) {
    (0, visitor.visitors[23])(new TSAnyKeyword(pos, ast));
  }
}

function walkTSStringKeyword(pos, ast, visitor) {
  if ((visitor.bitmap0 & 16777216) !== 0) {
    (0, visitor.visitors[24])(new TSStringKeyword(pos, ast));
  }
}

function walkTSBooleanKeyword(pos, ast, visitor) {
  if ((visitor.bitmap0 & 33554432) !== 0) {
    (0, visitor.visitors[25])(new TSBooleanKeyword(pos, ast));
  }
}

function walkTSNumberKeyword(pos, ast, visitor) {
  if ((visitor.bitmap0 & 67108864) !== 0) {
    (0, visitor.visitors[26])(new TSNumberKeyword(pos, ast));
  }
}

function walkTSNeverKeyword(pos, ast, visitor) {
  if ((visitor.bitmap0 & 134217728) !== 0) {
    (0, visitor.visitors[27])(new TSNeverKeyword(pos, ast));
  }
}

function walkTSIntrinsicKeyword(pos, ast, visitor) {
  if ((visitor.bitmap0 & 268435456) !== 0) {
    (0, visitor.visitors[28])(new TSIntrinsicKeyword(pos, ast));
  }
}

function walkTSUnknownKeyword(pos, ast, visitor) {
  if ((visitor.bitmap0 & 536870912) !== 0) {
    (0, visitor.visitors[29])(new TSUnknownKeyword(pos, ast));
  }
}

function walkTSNullKeyword(pos, ast, visitor) {
  if ((visitor.bitmap0 & 1073741824) !== 0) {
    (0, visitor.visitors[30])(new TSNullKeyword(pos, ast));
  }
}

function walkTSUndefinedKeyword(pos, ast, visitor) {
  if ((visitor.bitmap0 & -2147483648) !== 0) {
    (0, visitor.visitors[31])(new TSUndefinedKeyword(pos, ast));
  }
}

function walkTSVoidKeyword(pos, ast, visitor) {
  if ((visitor.bitmap1 & 1) !== 0) {
    (0, visitor.visitors[32])(new TSVoidKeyword(pos, ast));
  }
}

function walkTSSymbolKeyword(pos, ast, visitor) {
  if ((visitor.bitmap1 & 2) !== 0) {
    (0, visitor.visitors[33])(new TSSymbolKeyword(pos, ast));
  }
}

function walkTSThisType(pos, ast, visitor) {
  if ((visitor.bitmap1 & 4) !== 0) {
    (0, visitor.visitors[34])(new TSThisType(pos, ast));
  }
}

function walkTSObjectKeyword(pos, ast, visitor) {
  if ((visitor.bitmap1 & 8) !== 0) {
    (0, visitor.visitors[35])(new TSObjectKeyword(pos, ast));
  }
}

function walkTSBigIntKeyword(pos, ast, visitor) {
  if ((visitor.bitmap1 & 16) !== 0) {
    (0, visitor.visitors[36])(new TSBigIntKeyword(pos, ast));
  }
}

function walkTSTypeReference(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 2048) !== 0) {
    node = new TSTypeReference(pos, ast);
    ({ enter, exit } = visitor.visitors[139]);
    if (enter !== null) enter(node);
  }

  walkTSTypeName(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTypeName(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxTSQualifiedName(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTypeName`);
  }
}

function walkTSQualifiedName(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 4096) !== 0) {
    node = new TSQualifiedName(pos, ast);
    ({ enter, exit } = visitor.visitors[140]);
    if (enter !== null) enter(node);
  }

  walkTSTypeName(pos + 8, ast, visitor);
  walkIdentifierName(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTypeParameterInstantiation(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 8192) !== 0) {
    node = new TSTypeParameterInstantiation(pos, ast);
    ({ enter, exit } = visitor.visitors[141]);
    if (enter !== null) enter(node);
  }

  walkVecTSType(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTypeParameter(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 16384) !== 0) {
    node = new TSTypeParameter(pos, ast);
    ({ enter, exit } = visitor.visitors[142]);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitor);
  walkOptionTSType(pos + 40, ast, visitor);
  walkOptionTSType(pos + 56, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTypeParameterDeclaration(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 32768) !== 0) {
    node = new TSTypeParameterDeclaration(pos, ast);
    ({ enter, exit } = visitor.visitors[143]);
    if (enter !== null) enter(node);
  }

  walkVecTSTypeParameter(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTypeAliasDeclaration(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 65536) !== 0) {
    node = new TSTypeAliasDeclaration(pos, ast);
    ({ enter, exit } = visitor.visitors[144]);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterDeclaration(pos + 40, ast, visitor);
  walkTSType(pos + 48, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSClassImplements(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 131072) !== 0) {
    node = new TSClassImplements(pos, ast);
    ({ enter, exit } = visitor.visitors[145]);
    if (enter !== null) enter(node);
  }

  walkTSTypeName(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSInterfaceDeclaration(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 262144) !== 0) {
    node = new TSInterfaceDeclaration(pos, ast);
    ({ enter, exit } = visitor.visitors[146]);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterDeclaration(pos + 40, ast, visitor);
  walkVecTSInterfaceHeritage(pos + 48, ast, visitor);
  walkBoxTSInterfaceBody(pos + 72, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSInterfaceBody(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 524288) !== 0) {
    node = new TSInterfaceBody(pos, ast);
    ({ enter, exit } = visitor.visitors[147]);
    if (enter !== null) enter(node);
  }

  walkVecTSSignature(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSPropertySignature(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 1048576) !== 0) {
    node = new TSPropertySignature(pos, ast);
    ({ enter, exit } = visitor.visitors[148]);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSSignature(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxTSIndexSignature(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxTSPropertySignature(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxTSCallSignatureDeclaration(pos + 8, ast, visitor);
      return;
    case 3:
      walkBoxTSConstructSignatureDeclaration(pos + 8, ast, visitor);
      return;
    case 4:
      walkBoxTSMethodSignature(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSSignature`);
  }
}

function walkTSIndexSignature(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 2097152) !== 0) {
    node = new TSIndexSignature(pos, ast);
    ({ enter, exit } = visitor.visitors[149]);
    if (enter !== null) enter(node);
  }

  walkVecTSIndexSignatureName(pos + 8, ast, visitor);
  walkBoxTSTypeAnnotation(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSCallSignatureDeclaration(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 4194304) !== 0) {
    node = new TSCallSignatureDeclaration(pos, ast);
    ({ enter, exit } = visitor.visitors[150]);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitor);
  walkBoxFormalParameters(pos + 24, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSMethodSignature(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 8388608) !== 0) {
    node = new TSMethodSignature(pos, ast);
    ({ enter, exit } = visitor.visitors[151]);
    if (enter !== null) enter(node);
  }

  walkPropertyKey(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterDeclaration(pos + 24, ast, visitor);
  walkBoxFormalParameters(pos + 40, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 48, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSConstructSignatureDeclaration(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 16777216) !== 0) {
    node = new TSConstructSignatureDeclaration(pos, ast);
    ({ enter, exit } = visitor.visitors[152]);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitor);
  walkBoxFormalParameters(pos + 16, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSIndexSignatureName(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 33554432) !== 0) {
    node = new TSIndexSignatureName(pos, ast);
    ({ enter, exit } = visitor.visitors[153]);
    if (enter !== null) enter(node);
  }

  walkBoxTSTypeAnnotation(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSInterfaceHeritage(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 67108864) !== 0) {
    node = new TSInterfaceHeritage(pos, ast);
    ({ enter, exit } = visitor.visitors[154]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTypePredicate(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 134217728) !== 0) {
    node = new TSTypePredicate(pos, ast);
    ({ enter, exit } = visitor.visitors[155]);
    if (enter !== null) enter(node);
  }

  walkTSTypePredicateName(pos + 8, ast, visitor);
  walkOptionBoxTSTypeAnnotation(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTypePredicateName(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierName(pos + 8, ast, visitor);
      return;
    case 1:
      walkTSThisType(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTypePredicateName`);
  }
}

function walkTSModuleDeclaration(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 268435456) !== 0) {
    node = new TSModuleDeclaration(pos, ast);
    ({ enter, exit } = visitor.visitors[156]);
    if (enter !== null) enter(node);
  }

  walkTSModuleDeclarationName(pos + 8, ast, visitor);
  walkOptionTSModuleDeclarationBody(pos + 64, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSModuleDeclarationName(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBindingIdentifier(pos + 8, ast, visitor);
      return;
    case 1:
      walkStringLiteral(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSModuleDeclarationName`);
  }
}

function walkTSModuleDeclarationBody(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxTSModuleDeclaration(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxTSModuleBlock(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSModuleDeclarationBody`);
  }
}

function walkTSModuleBlock(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 536870912) !== 0) {
    node = new TSModuleBlock(pos, ast);
    ({ enter, exit } = visitor.visitors[157]);
    if (enter !== null) enter(node);
  }

  walkVecStatement(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTypeLiteral(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & 1073741824) !== 0) {
    node = new TSTypeLiteral(pos, ast);
    ({ enter, exit } = visitor.visitors[158]);
    if (enter !== null) enter(node);
  }

  walkVecTSSignature(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSInferType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap4 & -2147483648) !== 0) {
    node = new TSInferType(pos, ast);
    ({ enter, exit } = visitor.visitors[159]);
    if (enter !== null) enter(node);
  }

  walkBoxTSTypeParameter(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTypeQuery(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 1) !== 0) {
    node = new TSTypeQuery(pos, ast);
    ({ enter, exit } = visitor.visitors[160]);
    if (enter !== null) enter(node);
  }

  walkTSTypeQueryExprName(pos + 8, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTypeQueryExprName(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxTSQualifiedName(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxTSImportType(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSTypeQueryExprName`);
  }
}

function walkTSImportType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 2) !== 0) {
    node = new TSImportType(pos, ast);
    ({ enter, exit } = visitor.visitors[161]);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);
  walkOptionBoxObjectExpression(pos + 24, ast, visitor);
  walkOptionTSTypeName(pos + 32, ast, visitor);
  walkOptionBoxTSTypeParameterInstantiation(pos + 48, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSFunctionType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 4) !== 0) {
    node = new TSFunctionType(pos, ast);
    ({ enter, exit } = visitor.visitors[162]);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitor);
  walkBoxFormalParameters(pos + 24, ast, visitor);
  walkBoxTSTypeAnnotation(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSConstructorType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 8) !== 0) {
    node = new TSConstructorType(pos, ast);
    ({ enter, exit } = visitor.visitors[163]);
    if (enter !== null) enter(node);
  }

  walkOptionBoxTSTypeParameterDeclaration(pos + 8, ast, visitor);
  walkBoxFormalParameters(pos + 16, ast, visitor);
  walkBoxTSTypeAnnotation(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSMappedType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 16) !== 0) {
    node = new TSMappedType(pos, ast);
    ({ enter, exit } = visitor.visitors[164]);
    if (enter !== null) enter(node);
  }

  walkOptionTSType(pos + 16, ast, visitor);
  walkOptionTSType(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTemplateLiteralType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 32) !== 0) {
    node = new TSTemplateLiteralType(pos, ast);
    ({ enter, exit } = visitor.visitors[165]);
    if (enter !== null) enter(node);
  }

  walkVecTemplateElement(pos + 8, ast, visitor);
  walkVecTSType(pos + 32, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSAsExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 64) !== 0) {
    node = new TSAsExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[166]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkTSType(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSSatisfiesExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 128) !== 0) {
    node = new TSSatisfiesExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[167]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkTSType(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSTypeAssertion(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 256) !== 0) {
    node = new TSTypeAssertion(pos, ast);
    ({ enter, exit } = visitor.visitors[168]);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);
  walkExpression(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSImportEqualsDeclaration(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 512) !== 0) {
    node = new TSImportEqualsDeclaration(pos, ast);
    ({ enter, exit } = visitor.visitors[169]);
    if (enter !== null) enter(node);
  }

  walkBindingIdentifier(pos + 8, ast, visitor);
  walkTSModuleReference(pos + 40, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSModuleReference(pos, ast, visitor) {
  switch (ast.buffer[pos]) {
    case 0:
      walkBoxIdentifierReference(pos + 8, ast, visitor);
      return;
    case 1:
      walkBoxTSQualifiedName(pos + 8, ast, visitor);
      return;
    case 2:
      walkBoxTSExternalModuleReference(pos + 8, ast, visitor);
      return;
    default:
      throw new Error(`Unexpected discriminant ${ast.buffer[pos]} for TSModuleReference`);
  }
}

function walkTSExternalModuleReference(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 1024) !== 0) {
    node = new TSExternalModuleReference(pos, ast);
    ({ enter, exit } = visitor.visitors[170]);
    if (enter !== null) enter(node);
  }

  walkStringLiteral(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSNonNullExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 2048) !== 0) {
    node = new TSNonNullExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[171]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkDecorator(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 4096) !== 0) {
    node = new Decorator(pos, ast);
    ({ enter, exit } = visitor.visitors[172]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSExportAssignment(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 8192) !== 0) {
    node = new TSExportAssignment(pos, ast);
    ({ enter, exit } = visitor.visitors[173]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSNamespaceExportDeclaration(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 16384) !== 0) {
    node = new TSNamespaceExportDeclaration(pos, ast);
    ({ enter, exit } = visitor.visitors[174]);
    if (enter !== null) enter(node);
  }

  walkIdentifierName(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkTSInstantiationExpression(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 32768) !== 0) {
    node = new TSInstantiationExpression(pos, ast);
    ({ enter, exit } = visitor.visitors[175]);
    if (enter !== null) enter(node);
  }

  walkExpression(pos + 8, ast, visitor);
  walkBoxTSTypeParameterInstantiation(pos + 24, ast, visitor);

  if (exit !== null) exit(node);
}

function walkJSDocNullableType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 65536) !== 0) {
    node = new JSDocNullableType(pos, ast);
    ({ enter, exit } = visitor.visitors[176]);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkJSDocNonNullableType(pos, ast, visitor) {
  let node, enter, exit = null;
  if ((visitor.bitmap5 & 131072) !== 0) {
    node = new JSDocNonNullableType(pos, ast);
    ({ enter, exit } = visitor.visitors[177]);
    if (enter !== null) enter(node);
  }

  walkTSType(pos + 8, ast, visitor);

  if (exit !== null) exit(node);
}

function walkJSDocUnknownType(pos, ast, visitor) {
  if ((visitor.bitmap1 & 32) !== 0) {
    (0, visitor.visitors[37])(new JSDocUnknownType(pos, ast));
  }
}

function walkOptionHashbang(pos, ast, visitor) {
  if (!(ast.buffer.uint32[(pos + 8) >> 2] === 0 && ast.buffer.uint32[(pos + 12) >> 2] === 0)) {
    walkHashbang(pos, ast, visitor);
  }
}

function walkVecStatement(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkStatement(pos, ast, visitor);
    pos += 16;
  }
}

function walkBoxBooleanLiteral(pos, ast, visitor) {
  return walkBooleanLiteral(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxNullLiteral(pos, ast, visitor) {
  return walkNullLiteral(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxNumericLiteral(pos, ast, visitor) {
  return walkNumericLiteral(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxBigIntLiteral(pos, ast, visitor) {
  return walkBigIntLiteral(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxRegExpLiteral(pos, ast, visitor) {
  return walkRegExpLiteral(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxStringLiteral(pos, ast, visitor) {
  return walkStringLiteral(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTemplateLiteral(pos, ast, visitor) {
  return walkTemplateLiteral(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxIdentifierReference(pos, ast, visitor) {
  return walkIdentifierReference(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxMetaProperty(pos, ast, visitor) {
  return walkMetaProperty(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxSuper(pos, ast, visitor) {
  return walkSuper(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxArrayExpression(pos, ast, visitor) {
  return walkArrayExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxArrowFunctionExpression(pos, ast, visitor) {
  return walkArrowFunctionExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxAssignmentExpression(pos, ast, visitor) {
  return walkAssignmentExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxAwaitExpression(pos, ast, visitor) {
  return walkAwaitExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxBinaryExpression(pos, ast, visitor) {
  return walkBinaryExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxCallExpression(pos, ast, visitor) {
  return walkCallExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxChainExpression(pos, ast, visitor) {
  return walkChainExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxClass(pos, ast, visitor) {
  return walkClass(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxConditionalExpression(pos, ast, visitor) {
  return walkConditionalExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxFunction(pos, ast, visitor) {
  return walkFunction(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxImportExpression(pos, ast, visitor) {
  return walkImportExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxLogicalExpression(pos, ast, visitor) {
  return walkLogicalExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxNewExpression(pos, ast, visitor) {
  return walkNewExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxObjectExpression(pos, ast, visitor) {
  return walkObjectExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxParenthesizedExpression(pos, ast, visitor) {
  return walkParenthesizedExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxSequenceExpression(pos, ast, visitor) {
  return walkSequenceExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTaggedTemplateExpression(pos, ast, visitor) {
  return walkTaggedTemplateExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxThisExpression(pos, ast, visitor) {
  return walkThisExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxUnaryExpression(pos, ast, visitor) {
  return walkUnaryExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxUpdateExpression(pos, ast, visitor) {
  return walkUpdateExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxYieldExpression(pos, ast, visitor) {
  return walkYieldExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxPrivateInExpression(pos, ast, visitor) {
  return walkPrivateInExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxJSXElement(pos, ast, visitor) {
  return walkJSXElement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxJSXFragment(pos, ast, visitor) {
  return walkJSXFragment(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSAsExpression(pos, ast, visitor) {
  return walkTSAsExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSSatisfiesExpression(pos, ast, visitor) {
  return walkTSSatisfiesExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSTypeAssertion(pos, ast, visitor) {
  return walkTSTypeAssertion(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSNonNullExpression(pos, ast, visitor) {
  return walkTSNonNullExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSInstantiationExpression(pos, ast, visitor) {
  return walkTSInstantiationExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxV8IntrinsicExpression(pos, ast, visitor) {
  return walkV8IntrinsicExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkVecArrayExpressionElement(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkArrayExpressionElement(pos, ast, visitor);
    pos += 16;
  }
}

function walkBoxSpreadElement(pos, ast, visitor) {
  return walkSpreadElement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkVecObjectPropertyKind(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkObjectPropertyKind(pos, ast, visitor);
    pos += 16;
  }
}

function walkBoxObjectProperty(pos, ast, visitor) {
  return walkObjectProperty(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxIdentifierName(pos, ast, visitor) {
  return walkIdentifierName(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxPrivateIdentifier(pos, ast, visitor) {
  return walkPrivateIdentifier(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkVecTemplateElement(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 48;
  while (pos < endPos) {
    walkTemplateElement(pos, ast, visitor);
    pos += 48;
  }
}

function walkVecExpression(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkExpression(pos, ast, visitor);
    pos += 16;
  }
}

function walkBoxTSTypeParameterInstantiation(pos, ast, visitor) {
  return walkTSTypeParameterInstantiation(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkOptionBoxTSTypeParameterInstantiation(pos, ast, visitor) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0)) {
    walkBoxTSTypeParameterInstantiation(pos, ast, visitor);
  }
}

function walkBoxComputedMemberExpression(pos, ast, visitor) {
  return walkComputedMemberExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxStaticMemberExpression(pos, ast, visitor) {
  return walkStaticMemberExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxPrivateFieldExpression(pos, ast, visitor) {
  return walkPrivateFieldExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkVecArgument(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkArgument(pos, ast, visitor);
    pos += 16;
  }
}

function walkBoxArrayAssignmentTarget(pos, ast, visitor) {
  return walkArrayAssignmentTarget(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxObjectAssignmentTarget(pos, ast, visitor) {
  return walkObjectAssignmentTarget(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkOptionAssignmentTargetMaybeDefault(pos, ast, visitor) {
  if (!(ast.buffer[pos] === 51)) walkAssignmentTargetMaybeDefault(pos, ast, visitor);
}

function walkVecOptionAssignmentTargetMaybeDefault(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkOptionAssignmentTargetMaybeDefault(pos, ast, visitor);
    pos += 16;
  }
}

function walkVecAssignmentTargetProperty(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkAssignmentTargetProperty(pos, ast, visitor);
    pos += 16;
  }
}

function walkBoxAssignmentTargetWithDefault(pos, ast, visitor) {
  return walkAssignmentTargetWithDefault(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxAssignmentTargetPropertyIdentifier(pos, ast, visitor) {
  return walkAssignmentTargetPropertyIdentifier(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxAssignmentTargetPropertyProperty(pos, ast, visitor) {
  return walkAssignmentTargetPropertyProperty(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkOptionExpression(pos, ast, visitor) {
  if (!(ast.buffer[pos] === 51)) walkExpression(pos, ast, visitor);
}

function walkBoxBlockStatement(pos, ast, visitor) {
  return walkBlockStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxBreakStatement(pos, ast, visitor) {
  return walkBreakStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxContinueStatement(pos, ast, visitor) {
  return walkContinueStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxDebuggerStatement(pos, ast, visitor) {
  return walkDebuggerStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxDoWhileStatement(pos, ast, visitor) {
  return walkDoWhileStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxEmptyStatement(pos, ast, visitor) {
  return walkEmptyStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxExpressionStatement(pos, ast, visitor) {
  return walkExpressionStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxForInStatement(pos, ast, visitor) {
  return walkForInStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxForOfStatement(pos, ast, visitor) {
  return walkForOfStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxForStatement(pos, ast, visitor) {
  return walkForStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxIfStatement(pos, ast, visitor) {
  return walkIfStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxLabeledStatement(pos, ast, visitor) {
  return walkLabeledStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxReturnStatement(pos, ast, visitor) {
  return walkReturnStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxSwitchStatement(pos, ast, visitor) {
  return walkSwitchStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxThrowStatement(pos, ast, visitor) {
  return walkThrowStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTryStatement(pos, ast, visitor) {
  return walkTryStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxWhileStatement(pos, ast, visitor) {
  return walkWhileStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxWithStatement(pos, ast, visitor) {
  return walkWithStatement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxVariableDeclaration(pos, ast, visitor) {
  return walkVariableDeclaration(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSTypeAliasDeclaration(pos, ast, visitor) {
  return walkTSTypeAliasDeclaration(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSInterfaceDeclaration(pos, ast, visitor) {
  return walkTSInterfaceDeclaration(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSEnumDeclaration(pos, ast, visitor) {
  return walkTSEnumDeclaration(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSModuleDeclaration(pos, ast, visitor) {
  return walkTSModuleDeclaration(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSImportEqualsDeclaration(pos, ast, visitor) {
  return walkTSImportEqualsDeclaration(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkVecVariableDeclarator(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 64;
  while (pos < endPos) {
    walkVariableDeclarator(pos, ast, visitor);
    pos += 64;
  }
}

function walkOptionStatement(pos, ast, visitor) {
  if (!(ast.buffer[pos] === 70)) walkStatement(pos, ast, visitor);
}

function walkOptionForStatementInit(pos, ast, visitor) {
  if (!(ast.buffer[pos] === 65)) walkForStatementInit(pos, ast, visitor);
}

function walkOptionLabelIdentifier(pos, ast, visitor) {
  if (!(ast.buffer.uint32[(pos + 8) >> 2] === 0 && ast.buffer.uint32[(pos + 12) >> 2] === 0)) {
    walkLabelIdentifier(pos, ast, visitor);
  }
}

function walkVecSwitchCase(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 48;
  while (pos < endPos) {
    walkSwitchCase(pos, ast, visitor);
    pos += 48;
  }
}

function walkBoxCatchClause(pos, ast, visitor) {
  return walkCatchClause(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkOptionBoxCatchClause(pos, ast, visitor) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0)) {
    walkBoxCatchClause(pos, ast, visitor);
  }
}

function walkOptionBoxBlockStatement(pos, ast, visitor) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0)) {
    walkBoxBlockStatement(pos, ast, visitor);
  }
}

function walkOptionCatchParameter(pos, ast, visitor) {
  if (!(ast.buffer[pos + 32] === 2)) walkCatchParameter(pos, ast, visitor);
}

function walkBoxTSTypeAnnotation(pos, ast, visitor) {
  return walkTSTypeAnnotation(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkOptionBoxTSTypeAnnotation(pos, ast, visitor) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0)) {
    walkBoxTSTypeAnnotation(pos, ast, visitor);
  }
}

function walkBoxBindingIdentifier(pos, ast, visitor) {
  return walkBindingIdentifier(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxObjectPattern(pos, ast, visitor) {
  return walkObjectPattern(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxArrayPattern(pos, ast, visitor) {
  return walkArrayPattern(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxAssignmentPattern(pos, ast, visitor) {
  return walkAssignmentPattern(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkVecBindingProperty(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 64;
  while (pos < endPos) {
    walkBindingProperty(pos, ast, visitor);
    pos += 64;
  }
}

function walkOptionBindingPattern(pos, ast, visitor) {
  if (!(ast.buffer[pos + 24] === 2)) walkBindingPattern(pos, ast, visitor);
}

function walkVecOptionBindingPattern(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 32;
  while (pos < endPos) {
    walkOptionBindingPattern(pos, ast, visitor);
    pos += 32;
  }
}

function walkOptionBindingIdentifier(pos, ast, visitor) {
  if (!(ast.buffer.uint32[(pos + 8) >> 2] === 0 && ast.buffer.uint32[(pos + 12) >> 2] === 0)) {
    walkBindingIdentifier(pos, ast, visitor);
  }
}

function walkBoxTSTypeParameterDeclaration(pos, ast, visitor) {
  return walkTSTypeParameterDeclaration(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkOptionBoxTSTypeParameterDeclaration(pos, ast, visitor) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0)) {
    walkBoxTSTypeParameterDeclaration(pos, ast, visitor);
  }
}

function walkBoxFormalParameters(pos, ast, visitor) {
  return walkFormalParameters(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxFunctionBody(pos, ast, visitor) {
  return walkFunctionBody(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkOptionBoxFunctionBody(pos, ast, visitor) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0)) {
    walkBoxFunctionBody(pos, ast, visitor);
  }
}

function walkVecFormalParameter(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 72;
  while (pos < endPos) {
    walkFormalParameter(pos, ast, visitor);
    pos += 72;
  }
}

function walkVecDecorator(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 24;
  while (pos < endPos) {
    walkDecorator(pos, ast, visitor);
    pos += 24;
  }
}

function walkVecTSClassImplements(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 32;
  while (pos < endPos) {
    walkTSClassImplements(pos, ast, visitor);
    pos += 32;
  }
}

function walkBoxClassBody(pos, ast, visitor) {
  return walkClassBody(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkVecClassElement(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkClassElement(pos, ast, visitor);
    pos += 16;
  }
}

function walkBoxStaticBlock(pos, ast, visitor) {
  return walkStaticBlock(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxMethodDefinition(pos, ast, visitor) {
  return walkMethodDefinition(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxPropertyDefinition(pos, ast, visitor) {
  return walkPropertyDefinition(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxAccessorProperty(pos, ast, visitor) {
  return walkAccessorProperty(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSIndexSignature(pos, ast, visitor) {
  return walkTSIndexSignature(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxImportDeclaration(pos, ast, visitor) {
  return walkImportDeclaration(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxExportAllDeclaration(pos, ast, visitor) {
  return walkExportAllDeclaration(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxExportDefaultDeclaration(pos, ast, visitor) {
  return walkExportDefaultDeclaration(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxExportNamedDeclaration(pos, ast, visitor) {
  return walkExportNamedDeclaration(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSExportAssignment(pos, ast, visitor) {
  return walkTSExportAssignment(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSNamespaceExportDeclaration(pos, ast, visitor) {
  return walkTSNamespaceExportDeclaration(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkVecImportDeclarationSpecifier(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkImportDeclarationSpecifier(pos, ast, visitor);
    pos += 16;
  }
}

function walkOptionVecImportDeclarationSpecifier(pos, ast, visitor) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0)) {
    walkVecImportDeclarationSpecifier(pos, ast, visitor);
  }
}

function walkBoxWithClause(pos, ast, visitor) {
  return walkWithClause(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkOptionBoxWithClause(pos, ast, visitor) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0)) {
    walkBoxWithClause(pos, ast, visitor);
  }
}

function walkBoxImportSpecifier(pos, ast, visitor) {
  return walkImportSpecifier(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxImportDefaultSpecifier(pos, ast, visitor) {
  return walkImportDefaultSpecifier(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxImportNamespaceSpecifier(pos, ast, visitor) {
  return walkImportNamespaceSpecifier(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkVecImportAttribute(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 112;
  while (pos < endPos) {
    walkImportAttribute(pos, ast, visitor);
    pos += 112;
  }
}

function walkOptionDeclaration(pos, ast, visitor) {
  if (!(ast.buffer[pos] === 31)) walkDeclaration(pos, ast, visitor);
}

function walkVecExportSpecifier(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 128;
  while (pos < endPos) {
    walkExportSpecifier(pos, ast, visitor);
    pos += 128;
  }
}

function walkOptionStringLiteral(pos, ast, visitor) {
  if (!(ast.buffer[pos + 40] === 2)) walkStringLiteral(pos, ast, visitor);
}

function walkOptionModuleExportName(pos, ast, visitor) {
  if (!(ast.buffer[pos] === 3)) walkModuleExportName(pos, ast, visitor);
}

function walkBoxJSXOpeningElement(pos, ast, visitor) {
  return walkJSXOpeningElement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkVecJSXChild(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkJSXChild(pos, ast, visitor);
    pos += 16;
  }
}

function walkBoxJSXClosingElement(pos, ast, visitor) {
  return walkJSXClosingElement(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkOptionBoxJSXClosingElement(pos, ast, visitor) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0)) {
    walkBoxJSXClosingElement(pos, ast, visitor);
  }
}

function walkVecJSXAttributeItem(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkJSXAttributeItem(pos, ast, visitor);
    pos += 16;
  }
}

function walkBoxJSXIdentifier(pos, ast, visitor) {
  return walkJSXIdentifier(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxJSXNamespacedName(pos, ast, visitor) {
  return walkJSXNamespacedName(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxJSXMemberExpression(pos, ast, visitor) {
  return walkJSXMemberExpression(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxJSXAttribute(pos, ast, visitor) {
  return walkJSXAttribute(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxJSXSpreadAttribute(pos, ast, visitor) {
  return walkJSXSpreadAttribute(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkOptionJSXAttributeValue(pos, ast, visitor) {
  if (!(ast.buffer[pos] === 4)) walkJSXAttributeValue(pos, ast, visitor);
}

function walkBoxJSXExpressionContainer(pos, ast, visitor) {
  return walkJSXExpressionContainer(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxJSXText(pos, ast, visitor) {
  return walkJSXText(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxJSXSpreadChild(pos, ast, visitor) {
  return walkJSXSpreadChild(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkVecTSEnumMember(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 40;
  while (pos < endPos) {
    walkTSEnumMember(pos, ast, visitor);
    pos += 40;
  }
}

function walkBoxTSAnyKeyword(pos, ast, visitor) {
  return walkTSAnyKeyword(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSBigIntKeyword(pos, ast, visitor) {
  return walkTSBigIntKeyword(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSBooleanKeyword(pos, ast, visitor) {
  return walkTSBooleanKeyword(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSIntrinsicKeyword(pos, ast, visitor) {
  return walkTSIntrinsicKeyword(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSNeverKeyword(pos, ast, visitor) {
  return walkTSNeverKeyword(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSNullKeyword(pos, ast, visitor) {
  return walkTSNullKeyword(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSNumberKeyword(pos, ast, visitor) {
  return walkTSNumberKeyword(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSObjectKeyword(pos, ast, visitor) {
  return walkTSObjectKeyword(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSStringKeyword(pos, ast, visitor) {
  return walkTSStringKeyword(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSSymbolKeyword(pos, ast, visitor) {
  return walkTSSymbolKeyword(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSUndefinedKeyword(pos, ast, visitor) {
  return walkTSUndefinedKeyword(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSUnknownKeyword(pos, ast, visitor) {
  return walkTSUnknownKeyword(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSVoidKeyword(pos, ast, visitor) {
  return walkTSVoidKeyword(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSArrayType(pos, ast, visitor) {
  return walkTSArrayType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSConditionalType(pos, ast, visitor) {
  return walkTSConditionalType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSConstructorType(pos, ast, visitor) {
  return walkTSConstructorType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSFunctionType(pos, ast, visitor) {
  return walkTSFunctionType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSImportType(pos, ast, visitor) {
  return walkTSImportType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSIndexedAccessType(pos, ast, visitor) {
  return walkTSIndexedAccessType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSInferType(pos, ast, visitor) {
  return walkTSInferType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSIntersectionType(pos, ast, visitor) {
  return walkTSIntersectionType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSLiteralType(pos, ast, visitor) {
  return walkTSLiteralType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSMappedType(pos, ast, visitor) {
  return walkTSMappedType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSNamedTupleMember(pos, ast, visitor) {
  return walkTSNamedTupleMember(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSTemplateLiteralType(pos, ast, visitor) {
  return walkTSTemplateLiteralType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSThisType(pos, ast, visitor) {
  return walkTSThisType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSTupleType(pos, ast, visitor) {
  return walkTSTupleType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSTypeLiteral(pos, ast, visitor) {
  return walkTSTypeLiteral(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSTypeOperator(pos, ast, visitor) {
  return walkTSTypeOperator(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSTypePredicate(pos, ast, visitor) {
  return walkTSTypePredicate(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSTypeQuery(pos, ast, visitor) {
  return walkTSTypeQuery(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSTypeReference(pos, ast, visitor) {
  return walkTSTypeReference(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSUnionType(pos, ast, visitor) {
  return walkTSUnionType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSParenthesizedType(pos, ast, visitor) {
  return walkTSParenthesizedType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxJSDocNullableType(pos, ast, visitor) {
  return walkJSDocNullableType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxJSDocNonNullableType(pos, ast, visitor) {
  return walkJSDocNonNullableType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxJSDocUnknownType(pos, ast, visitor) {
  return walkJSDocUnknownType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkVecTSType(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkTSType(pos, ast, visitor);
    pos += 16;
  }
}

function walkVecTSTupleElement(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkTSTupleElement(pos, ast, visitor);
    pos += 16;
  }
}

function walkBoxTSOptionalType(pos, ast, visitor) {
  return walkTSOptionalType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSRestType(pos, ast, visitor) {
  return walkTSRestType(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSQualifiedName(pos, ast, visitor) {
  return walkTSQualifiedName(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkOptionTSType(pos, ast, visitor) {
  if (!(ast.buffer[pos] === 38)) walkTSType(pos, ast, visitor);
}

function walkVecTSTypeParameter(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 80;
  while (pos < endPos) {
    walkTSTypeParameter(pos, ast, visitor);
    pos += 80;
  }
}

function walkVecTSInterfaceHeritage(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 32;
  while (pos < endPos) {
    walkTSInterfaceHeritage(pos, ast, visitor);
    pos += 32;
  }
}

function walkBoxTSInterfaceBody(pos, ast, visitor) {
  return walkTSInterfaceBody(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkVecTSSignature(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 16;
  while (pos < endPos) {
    walkTSSignature(pos, ast, visitor);
    pos += 16;
  }
}

function walkBoxTSPropertySignature(pos, ast, visitor) {
  return walkTSPropertySignature(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSCallSignatureDeclaration(pos, ast, visitor) {
  return walkTSCallSignatureDeclaration(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSConstructSignatureDeclaration(pos, ast, visitor) {
  return walkTSConstructSignatureDeclaration(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSMethodSignature(pos, ast, visitor) {
  return walkTSMethodSignature(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkVecTSIndexSignatureName(pos, ast, visitor) {
  const { uint32 } = ast.buffer,
    pos32 = pos >> 2;
  pos = uint32[pos32];
  const endPos = pos + uint32[pos32 + 2] * 32;
  while (pos < endPos) {
    walkTSIndexSignatureName(pos, ast, visitor);
    pos += 32;
  }
}

function walkOptionTSModuleDeclarationBody(pos, ast, visitor) {
  if (!(ast.buffer[pos] === 2)) walkTSModuleDeclarationBody(pos, ast, visitor);
}

function walkBoxTSModuleBlock(pos, ast, visitor) {
  return walkTSModuleBlock(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkBoxTSTypeParameter(pos, ast, visitor) {
  return walkTSTypeParameter(ast.buffer.uint32[pos >> 2], ast, visitor);
}

function walkOptionBoxObjectExpression(pos, ast, visitor) {
  if (!(ast.buffer.uint32[pos >> 2] === 0 && ast.buffer.uint32[(pos + 4) >> 2] === 0)) {
    walkBoxObjectExpression(pos, ast, visitor);
  }
}

function walkOptionTSTypeName(pos, ast, visitor) {
  if (!(ast.buffer[pos] === 2)) walkTSTypeName(pos, ast, visitor);
}

function walkBoxTSExternalModuleReference(pos, ast, visitor) {
  return walkTSExternalModuleReference(ast.buffer.uint32[pos >> 2], ast, visitor);
}
